use azalea::prelude::*;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use anyhow::{anyhow, Result};
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
    println!("[STARTUP] DonutSMP Map Flipper Bot (Rust/Azalea)");
    
    // Load configuration
    let config = Config::load()?;
    println!("[CONFIG] Loaded configuration");
    
    // Log webhook configuration status
    if config.webhook.enabled {
        println!("[CONFIG] Webhook notifications: ENABLED");
        if !config.webhook.url.is_empty() {
            let url_display = if config.webhook.url.len() > 50 {
                format!("{}...", &config.webhook.url[..50])
            } else {
                config.webhook.url.clone()
            };
            println!("[CONFIG] Webhook URL: {}", url_display);
        } else {
            println!("[CONFIG] Webhook URL: NOT SET - webhooks will not be sent!");
        }
        
        // Collect enabled events
        let events = [
            ("purchase", config.webhook.events.purchase),
            ("listing", config.webhook.events.listing),
            ("sale", config.webhook.events.sale),
            ("afk", config.webhook.events.afk),
            ("error", config.webhook.events.error),
            ("startup", config.webhook.events.startup),
        ];
        let enabled_events: Vec<&str> = events.iter()
            .filter_map(|(name, enabled)| enabled.then_some(*name))
            .collect();
        println!("[CONFIG] Webhook events: {}", enabled_events.join(", "));
    } else {
        println!("[CONFIG] Webhook notifications: DISABLED");
    }

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

/// Start AFK farming by sending /afk command and clicking slot 49
/// This allows the bot to farm shards while flipping auctions
async fn start_afk_farming(bot: Client, config: &Config) -> Result<()> {
    println!("[AFK] Starting AFK farming setup...");
    
    // Step 1: Send /afk command
    println!("[AFK] Sending /afk command...");
    bot.chat("/afk");
    
    // Step 2: Wait for protocol timing (300ms to prevent "Invalid sequence" kick)
    sleep(Duration::from_millis(300)).await;
    
    // Step 3: Wait for AFK menu to open with timeout
    println!("[AFK] Waiting for AFK menu to open...");
    let timeout_ticks = (config.window_timeout + 50 - 1) / 50; // Convert ms to ticks, round up
    
    match bot.wait_for_container_open(Some(timeout_ticks as usize)).await {
        Some(afk_menu) => {
            let menu_id = afk_menu.id();
            println!("[AFK] AFK menu opened with container ID: {}", menu_id);
            
            // Step 4: Click slot 49 to go to random AFK location
            println!("[AFK] Clicking slot 49 to teleport to random AFK location...");
            afk_menu.left_click(49_usize);
            
            // Step 5: Wait for the click to be processed
            sleep(Duration::from_millis(300)).await;
            
            // Step 6: Properly close the container by dropping the handle
            // This ensures Azalea sends the close packet to the server
            drop(afk_menu);
            println!("[AFK] AFK menu closed");
            
            // Step 7: Wait for teleportation to complete
            sleep(Duration::from_millis(2000)).await;
            
            println!("[AFK] AFK farming setup completed successfully");
            println!("[AFK] Bot will now farm shards while flipping auctions");
            Ok(())
        }
        None => {
            println!("[AFK] Timeout waiting for AFK menu to open ({}ms)", config.window_timeout);
            Err(anyhow!("AFK menu did not open after /afk command"))
        }
    }
}

async fn handle_event(bot: Client, event: Event, state: BotState) -> Result<()> {
    match event {
        Event::Init => {
            println!("[BOT] Bot initialized");
        }
        Event::Login => {
            println!("[BOT] Logged in to server");
            
            // Check if this is a reconnection
            let was_running = {
                let is_running = state.is_running.lock();
                *is_running
            };
            
            if was_running {
                println!("[BOT] Reconnected after disconnect - resuming operations");
            }
            
            // Send startup webhook
            if let Err(e) = send_webhook(
                &state.config,
                "startup",
                if was_running {
                    "🔄 Bot reconnected and resuming operations"
                } else {
                    "🤖 Bot connected and spawned"
                },
                if was_running { 0xf39c12 } else { 0x2ecc71 },
                vec![
                    ("Server".to_string(), state.config.host.clone(), true),
                    ("Username".to_string(), bot.username().to_string(), true),
                ],
            ).await {
                eprintln!("[WEBHOOK] Error sending startup webhook: {}", e);
            }
            
            // Execute AFK startup action if enabled
            if state.config.enable_afk_farming {
                if let Err(e) = start_afk_farming(bot.clone(), &state.config).await {
                    eprintln!("[AFK] Failed to start AFK farming: {}", e);
                }
            }
            
            // Wait before starting main loop
            println!("[BOT] Waiting {}ms before starting...", state.config.delay_after_join);
            sleep(Duration::from_millis(state.config.delay_after_join)).await;
            
            // Start or restart main loop
            // For reconnections, we always restart the loop regardless of the flag
            // because the previous loop task was terminated when we disconnected
            let mut is_running = state.is_running.lock();
            *is_running = true;
            drop(is_running);
            
            if was_running {
                println!("[BOT] Restarting main loop after reconnection");
            } else {
                println!("[BOT] Starting main loop");
            }
            tokio::spawn(main_loop(bot.clone(), state.clone()));
        }
        Event::Chat(m) => {
            let message = m.message().to_string();
            println!("[CHAT] {}", message);
            
            // Check for AFK teleport notification
            let normalized = normalize_text(&message);
            if normalized.contains("teleported to") && normalized.contains("afk") {
                println!("[AFK] Detected AFK teleport - continuing operations in AFK zone");
                
                // Send webhook notification
                let _ = send_webhook(
                    &state.config,
                    "afk",
                    "🌙 Teleported to AFK zone - continuing to flip auctions",
                    0x9b59b6,
                    vec![],
                ).await;
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
                        
                        // Step 4: List ALL maps in inventory
                        // The new list_maps function handles stacks naturally by listing them one at a time
                        // This efficiently clears the entire inventory without needing a separate unstacking step
                        let all_maps = get_map_slots(&bot);
                        
                        if !all_maps.is_empty() {
                            println!("[CYCLE] Found {} map slot(s) in inventory (including stacks)", all_maps.len());
                            println!("[CYCLE] Listing all maps to clear inventory...");
                            
                            if let Err(e) = list_maps(&bot, &state.config, &all_maps).await {
                                eprintln!("[LISTING] Error listing maps: {}", e);
                            }
                        } else {
                            println!("[CYCLE] No maps in inventory - purchase may have failed");
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
