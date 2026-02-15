use azalea::prelude::*;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;
use regex::Regex;

mod config;
mod price_parser;
mod webhook;
mod inventory;

use config::Config;
use price_parser::parse_price;
use webhook::send_webhook;
use inventory::{open_auction_house, find_cheap_maps, purchase_map, list_maps, get_map_slots};

#[derive(Clone, Component)]
pub struct BotState {
    pub is_running: Arc<Mutex<bool>>,
    pub is_afk_detected: Arc<Mutex<bool>>,
    pub config: Arc<Config>,
}

impl Default for BotState {
    fn default() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            is_afk_detected: Arc::new(Mutex::new(false)),
            config: Arc::new(Config::from_env()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();

    println!("[STARTUP] DonutSMP Map Flipper Bot (Rust/Azalea)");
    
    // Load configuration
    let config = Config::load()?;
    println!("[CONFIG] Loaded configuration");
    println!("[STARTUP] Configuration: {:?}", config);

    let _state = BotState {
        is_running: Arc::new(Mutex::new(false)),
        is_afk_detected: Arc::new(Mutex::new(false)),
        config: Arc::new(config.clone()),
    };

    // Create account based on auth type
    let account = if config.auth == "microsoft" {
        println!("[AUTH] Using Microsoft authentication");
        println!("[AUTH] Please follow the prompts to authenticate...");
        Account::microsoft(&config.username).await?
    } else {
        println!("[AUTH] Using offline authentication");
        Account::offline(&config.username)
    };

    // Connect to server
    let address = format!("{}:{}", config.host, config.port);
    println!("[BOT] Connecting to {}...", address);

    let exit_code = ClientBuilder::new()
        .set_handler(handle_event)
        .start(account, address.as_str())
        .await;

    println!("[BOT] Exited with code: {:?}", exit_code);
    Ok(())
}

async fn handle_event(bot: Client, event: Event, state: BotState) -> Result<()> {
    match event {
        Event::Init => {
            println!("[BOT] Bot initialized");
        }
        Event::Login => {
            println!("[BOT] Logged in to server");
            
            // Send startup webhook
            if let Err(e) = send_webhook(
                &state.config,
                "startup",
                "🤖 Bot connected and spawned",
                0x2ecc71,
                vec![
                    ("Server".to_string(), state.config.host.clone(), true),
                    ("Username".to_string(), bot.username().to_string(), true),
                ],
            ).await {
                eprintln!("[WEBHOOK] Error sending startup webhook: {}", e);
            }
            
            // Wait before starting main loop
            println!("[BOT] Waiting {}ms before starting...", state.config.delay_after_join);
            sleep(Duration::from_millis(state.config.delay_after_join)).await;
            
            // Start main loop
            let mut is_running = state.is_running.lock();
            if !*is_running && !*state.is_afk_detected.lock() {
                *is_running = true;
                drop(is_running);
                println!("[BOT] Starting main loop");
                tokio::spawn(main_loop(bot.clone(), state.clone()));
            }
        }
        Event::Chat(m) => {
            let message = m.message().to_string();
            println!("[CHAT] {}", message);
            
            // Check for AFK detection
            let normalized = normalize_text(&message);
            if normalized.contains("teleported to") && normalized.contains("afk") {
                handle_afk_detection(bot.clone(), state.clone()).await;
            }
            
            // Check for map sale
            check_for_sale(&message, &state.config).await;
        }
        _ => {}
    }
    Ok(())
}

async fn main_loop(bot: Client, state: BotState) {
    loop {
        // Check if we should stop
        if *state.is_afk_detected.lock() {
            println!("[LOOP] AFK detected, pausing main loop");
            sleep(Duration::from_secs(1)).await;
            continue;
        }
        
        match run_cycle(bot.clone(), state.clone()).await {
            Ok(success) => {
                if success {
                    println!("[LOOP] Cycle completed successfully");
                } else {
                    println!("[LOOP] Cycle completed with no purchase");
                }
            }
            Err(e) => {
                eprintln!("[LOOP] Error in cycle: {}", e);
                
                // Send error webhook
                let _ = send_webhook(
                    &state.config,
                    "error",
                    &format!("⚠️ Bot encountered an error: {}", e),
                    0xe74c3c,
                    vec![],
                ).await;
                
                // Wait before retry
                sleep(Duration::from_millis(3000.max(state.config.delay_between_cycles))).await;
            }
        }
        
        // Wait between cycles
        sleep(Duration::from_millis(state.config.delay_between_cycles)).await;
    }
}

async fn run_cycle(bot: Client, state: BotState) -> Result<bool> {
    println!("[CYCLE] Starting new cycle");
    
    // Step 1: Open auction house
    match open_auction_house(&bot, &state.config).await {
        Ok(Some(menu)) => {
            println!("[AH] Auction house opened successfully");
            
            // Step 2: Find cheap maps
            if let Some(map) = find_cheap_maps(&menu, state.config.max_buy_price) {
                println!("[AH] Found cheap map: ${} from {}", map.price, map.seller);
                
                // Track which inventory slots have maps BEFORE purchase
                let maps_before = get_map_slots(&bot);
                println!("[CYCLE] Inventory snapshot: {} map(s) before purchase", maps_before.len());
                
                // Step 3: Attempt purchase
                match purchase_map(&bot, &map, &state.config).await {
                    Ok(true) => {
                        println!("[AH] Purchase successful!");
                        
                        // Send webhook notification
                        let _ = send_webhook(
                            &state.config,
                            "purchase",
                            &format!("💰 Purchased map for ${}", map.price),
                            0x2ecc71,
                            vec![
                                ("Price".to_string(), format!("${}", map.price), true),
                                ("Seller".to_string(), map.seller.clone(), true),
                            ],
                        ).await;
                        
                        // Step 4: List only NEWLY purchased maps
                        // Get current inventory state
                        let maps_after = get_map_slots(&bot);
                        println!("[CYCLE] Inventory snapshot: {} map(s) after purchase", maps_after.len());
                        
                        // Find slots that have maps now but didn't before
                        // Use HashSet for O(n) complexity instead of O(n²)
                        let maps_before_set: HashSet<usize> = maps_before.into_iter().collect();
                        let new_map_slots: Vec<usize> = maps_after
                            .into_iter()
                            .filter(|slot| !maps_before_set.contains(slot))
                            .collect();
                        
                        if !new_map_slots.is_empty() {
                            println!("[CYCLE] Found {} new map(s) to list", new_map_slots.len());
                            if let Err(e) = list_maps(&bot, &state.config, &new_map_slots).await {
                                eprintln!("[LISTING] Error listing maps: {}", e);
                            }
                        } else {
                            println!("[CYCLE] No new maps detected - purchase may have failed");
                        }
                        
                        return Ok(true);
                    }
                    Ok(false) => {
                        println!("[AH] Purchase failed (already bought or error)");
                    }
                    Err(e) => {
                        eprintln!("[AH] Purchase error: {}", e);
                    }
                }
            } else {
                println!("[AH] No cheap maps found under ${}", state.config.max_buy_price);
            }
        }
        Ok(None) => {
            println!("[AH] Auction house window did not open");
        }
        Err(e) => {
            eprintln!("[AH] Error opening auction house: {}", e);
        }
    }
    
    Ok(false)
}

async fn handle_afk_detection(bot: Client, state: BotState) {
    // Prevent multiple simultaneous AFK handling
    {
        let mut is_afk = state.is_afk_detected.lock();
        if *is_afk {
            println!("[AFK] Already handling AFK detection");
            return;
        }
        *is_afk = true;
    }
    
    println!("[AFK] Detected AFK teleport, returning to hub...");
    
    let _ = send_webhook(
        &state.config,
        "afk",
        "AFK detected! Returning to hub...",
        0xffff00,
        vec![],
    ).await;
    
    // Send /hub command
    bot.chat("/hub");
    sleep(Duration::from_secs(1)).await;
    
    // Wait for hub selection window - TODO: implement window handling
    sleep(Duration::from_secs(3)).await;
    
    // Wait 10 seconds before resuming
    println!("[AFK] Waiting 10 seconds before resuming...");
    sleep(Duration::from_secs(10)).await;
    
    {
        let mut is_afk = state.is_afk_detected.lock();
        *is_afk = false;
    }
    
    println!("[AFK] Resuming operations");
}

async fn check_for_sale(message: &str, config: &Config) {
    // Check for map sale - format: "Username bought your Map for $price"
    let re = Regex::new(r"(.+?)\s+bought your Map for \$([0-9,.]+)(K?)").unwrap();
    
    if let Some(caps) = re.captures(message) {
        let buyer = caps.get(1).map(|m| m.as_str()).unwrap_or("Unknown");
        let price_str = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
        let k_suffix = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        
        let full_price_str = format!("Price: ${}{}", price_str, k_suffix);
        
        if let Some(sale_price) = parse_price(&full_price_str) {
            println!("[SALE] {} bought a map for ${}", buyer, sale_price);
            
            let _ = send_webhook(
                config,
                "sale",
                "💰 Sold a map!",
                0x57eb8b,
                vec![
                    ("Buyer".to_string(), buyer.to_string(), true),
                    ("Price".to_string(), format!("${}", sale_price), true),
                ],
            ).await;
        }
    }
}

fn normalize_text(text: &str) -> String {
    // Map for small caps to ASCII
    let small_caps: HashMap<char, char> = [
        ('ᴀ', 'a'), ('ʙ', 'b'), ('ᴄ', 'c'), ('ᴅ', 'd'), ('ᴇ', 'e'), ('ꜰ', 'f'),
        ('ɢ', 'g'), ('ʜ', 'h'), ('ɪ', 'i'), ('ᴊ', 'j'), ('ᴋ', 'k'), ('ʟ', 'l'),
        ('ᴍ', 'm'), ('ɴ', 'n'), ('ᴏ', 'o'), ('ᴘ', 'p'), ('ʀ', 'r'), ('ꜱ', 's'),
        ('ᴛ', 't'), ('ᴜ', 'u'), ('ᴠ', 'v'), ('ᴡ', 'w'), ('ʏ', 'y'), ('ᴢ', 'z'),
    ].iter().cloned().collect();
    
    text.chars()
        .map(|c| small_caps.get(&c).copied().unwrap_or(c))
        .collect::<String>()
        .to_lowercase()
}
