use azalea::prelude::*;
use azalea::container::ContainerClientExt;
use azalea::inventory::{ItemStack, Menu};
use azalea::inventory::components::Lore;
use anyhow::{Result, anyhow};
use std::time::Duration;
use tokio::time::sleep;
use crate::config::Config;

// Minecraft server tick rate: 1 tick = 50 milliseconds
const MS_PER_TICK: u64 = 50;

pub struct MapSlot {
    pub slot: usize,
    pub price: u32,
    pub seller: String,
}

/// Parse price and seller from item lore
pub fn parse_item_info(lore: &[String]) -> Option<(u32, String)> {
    use crate::price_parser::{parse_price, strip_minecraft_colors};
    
    let mut price = None;
    let mut seller = String::from("unknown");
    
    for line in lore {
        // Look for price
        if line.contains("Price:") {
            if let Some(p) = parse_price(line) {
                price = Some(p);
            }
        }
        
        // Look for seller
        if line.contains("Seller:") {
            let clean = strip_minecraft_colors(line);
            if let Some(seller_match) = clean.strip_prefix("Seller:") {
                seller = seller_match.trim().to_string();
            }
        }
    }
    
    price.map(|p| (p, seller))
}

/// Extract lore text from an item
/// In Minecraft 1.21.1+, lore is stored in components, not NBT
pub fn extract_lore(item: &ItemStack) -> Vec<String> {
    // Check if item is present and has lore component
    if let Some(lore_component) = item.get_component::<Lore>() {
        // Extract text from each lore line
        lore_component.lines.iter()
            .map(|formatted_text| {
                // Convert FormattedText to plain string
                format!("{}", formatted_text)
            })
            .collect()
    } else {
        vec![]
    }
}

/// Open the auction house window
///
/// Reference: bot.js lines 323-359
pub async fn open_auction_house(bot: &Client, config: &Config) -> Result<Option<Menu>> {
    println!("[AH] Opening auction house...");
    
    // Send the /ah map command
    bot.chat("/ah map");
    
    // Wait for protocol timing (300ms to prevent "Invalid sequence" kick)
    sleep(Duration::from_millis(300)).await;
    
    // Wait for container to open with timeout (convert ms to ticks: ms/50, round up)
    let timeout_ticks = (config.window_timeout + MS_PER_TICK - 1) / MS_PER_TICK;
    
    println!("[AH] Waiting for auction house window to open (timeout: {}ms)...", config.window_timeout);
    
    // Use Azalea's wait_for_container_open function
    match bot.wait_for_container_open(Some(timeout_ticks as usize)).await {
        Some(container_handle) => {
            // Get the menu from the container
            if let Some(menu) = container_handle.menu() {
                println!("[AH] Auction house opened successfully");
                // Don't close container - we need to keep it open to interact
                // Drop the handle without closing by using std::mem::forget
                std::mem::forget(container_handle);
                Ok(Some(menu))
            } else {
                Err(anyhow!("Container opened but menu is not available"))
            }
        }
        None => {
            Err(anyhow!("Timeout waiting for auction house window ({}ms)", config.window_timeout))
        }
    }
}

/// Find cheap maps in the auction house container
///
/// Reference: bot.js lines 361-436
pub fn find_cheap_maps(menu: &Menu, max_price: u32) -> Option<MapSlot> {
    println!("[AH] Scanning for cheap maps under ${}...", max_price);
    
    // Determine container size based on menu type
    let container_size = match menu {
        Menu::Generic9x6 { contents, .. } => contents.len(),
        Menu::Generic9x5 { contents, .. } => contents.len(),
        Menu::Generic9x4 { contents, .. } => contents.len(),
        Menu::Generic9x3 { contents, .. } => contents.len(),
        Menu::Generic9x2 { contents, .. } => contents.len(),
        Menu::Generic9x1 { contents, .. } => contents.len(),
        _ => {
            println!("[AH] Unknown menu type, cannot scan");
            return None;
        }
    };
    
    println!("[AH] Scanning {} container slots...", container_size);
    
    // Get the actual slots from the menu
    let slots = menu.slots();
    
    // ONLY scan container slots, NOT player inventory
    for slot_index in 0..container_size {
        if slot_index >= slots.len() {
            break;
        }
        
        let item = &slots[slot_index];
        
        // Skip empty slots
        if item.is_empty() {
            continue;
        }
        
        // Extract lore from item
        let lore_lines = extract_lore(item);
        
        if lore_lines.is_empty() {
            continue;
        }
        
        // Parse price and seller from lore
        if let Some((price, seller)) = parse_item_info(&lore_lines) {
            if price < max_price {
                println!("[AH] ✓ Found cheap map at slot {}: ${} (seller: {})", slot_index, price, seller);
                return Some(MapSlot {
                    slot: slot_index,
                    price,
                    seller,
                });
            }
        }
    }
    
    println!("[AH] No cheap maps found under ${}", max_price);
    None
}

/// Purchase a map from the auction house
///
/// This function now properly handles the container ID change that happens when
/// clicking on a map in the auction house. The flow is:
/// 1. Click the map slot in the AH container (ID N)
/// 2. Server closes AH container and opens confirm screen container (ID N+1)
/// 3. Wait for the NEW container to open
/// 4. Click confirm button in the NEW container
///
/// Reference: bot.js lines 438-537
pub async fn purchase_map(
    bot: &Client,
    map: &MapSlot,
    config: &Config,
) -> Result<bool> {
    println!("[AH] Attempting to purchase map at slot {} for ${}...", map.slot, map.price);
    
    // Get the current container ID before clicking
    let initial_container = bot.get_inventory();
    let initial_container_id = initial_container.id();
    println!("[AH] Current container ID: {}", initial_container_id);
    
    // Step 1: Click the map slot in the auction house
    println!("[AH] Clicking map slot {}...", map.slot);
    initial_container.left_click(map.slot as usize);
    
    // Step 2: Wait for the NEW container to open (the confirm screen)
    // The server will close the current container and open a new one with incremented ID
    println!("[AH] Waiting for confirm screen to open...");
    
    // Convert timeout from milliseconds to ticks (round up to ensure we wait long enough)
    let timeout_ticks = (config.window_timeout + MS_PER_TICK - 1) / MS_PER_TICK;
    
    match bot.wait_for_container_open(Some(timeout_ticks as usize)).await {
        Some(confirm_container) => {
            let confirm_container_id = confirm_container.id();
            println!("[AH] Confirm screen opened with container ID: {}", confirm_container_id);
            
            // Verify the container ID actually changed
            if confirm_container_id == initial_container_id {
                println!("[AH] WARNING: Container ID did not change! This might cause issues.");
            }
            
            // Step 3: Click the confirm button (slot 15) in the NEW container
            println!("[AH] Clicking confirm button (slot 15) in container ID {}...", confirm_container_id);
            confirm_container.left_click(15_usize);
            
            // Keep the container handle alive without closing it
            // We use forget() here because Azalea will close the container when handle is dropped,
            // but the server may need time to process the purchase before we close it
            std::mem::forget(confirm_container);
            
            // Step 4: Wait for the window to close and purchase to complete
            sleep(Duration::from_millis(2000)).await;
            
            // For now, assume success
            // TODO: In the future, add chat message monitoring to verify purchase
            // We would need to listen for "already bought" or similar messages
            println!("[AH] Purchase completed");
            Ok(true)
        }
        None => {
            println!("[AH] Timeout waiting for confirm screen to open ({}ms)", config.window_timeout);
            Err(anyhow!("Confirm screen did not open after clicking map slot"))
        }
    }
}

/// List maps from inventory on auction house
///
/// This function now only lists maps from specific slots to avoid listing
/// the same maps over and over. Limits listings to maxListingsPerCycle.
///
/// Reference: bot.js lines 572-610
pub async fn list_maps(bot: &Client, config: &Config, slots_to_list: &[usize]) -> Result<()> {
    if slots_to_list.is_empty() {
        println!("[LISTING] No new maps to list");
        return Ok(());
    }
    
    // Apply the listing limit
    let max_listings = config.max_listings_per_cycle as usize;
    let total_available = slots_to_list.len();
    let slots_to_process: Vec<usize> = slots_to_list.iter()
        .take(max_listings)
        .copied()
        .collect();
    
    println!("[LISTING] Listing {} new map(s)...", slots_to_process.len());
    
    if total_available > max_listings {
        println!("[LISTING] Limited to {} maps per cycle (configured max: {}). {} maps will be listed in next cycle.", 
                 max_listings, config.max_listings_per_cycle, total_available - max_listings);
    }
    
    // Get bot's inventory
    let inventory_handle = bot.get_inventory();
    
    // Get the menu to check what items we have
    if let Some(menu) = inventory_handle.menu() {
        // Get all slots from the menu
        let slots = menu.slots();
        
        for &slot_idx in &slots_to_process {
            if slot_idx >= slots.len() {
                continue;
            }
            
            let slot = &slots[slot_idx];
            
            if slot.is_present() {
                // Check if this is a map item
                if let ItemStack::Present(data) = slot {
                    let item_name = format!("{:?}", data.kind);
                    if item_name.to_lowercase().contains("map") {
                        println!("[LISTING] Listing map from slot {} at ${}...", slot_idx, config.sell_price);
                        
                        // Send /ah list command
                        bot.chat(&format!("/ah list {}", config.sell_price));
                        
                        // Wait between listings to avoid spam
                        sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        }
    }
    
    println!("[LISTING] Finished listing maps");
    Ok(())
}

/// Get a snapshot of which inventory slots contain maps
///
/// This is used to track which maps are new after a purchase
pub fn get_map_slots(bot: &Client) -> Vec<usize> {
    let inventory_handle = bot.get_inventory();
    let mut map_slots = Vec::new();
    
    if let Some(menu) = inventory_handle.menu() {
        let slots = menu.slots();
        
        for (idx, slot) in slots.iter().enumerate() {
            if slot.is_present() {
                if let ItemStack::Present(data) = slot {
                    let item_name = format!("{:?}", data.kind);
                    if item_name.to_lowercase().contains("map") {
                        map_slots.push(idx);
                    }
                }
            }
        }
    }
    
    map_slots
}

/*
 * IMPLEMENTATION GUIDE
 * ====================
 *
 * To complete this module, you need to understand Azalea's APIs for:
 *
 * 1. Inventory/Container/Menu Access
 *    - How to access opened containers
 *    - How to read container slots
 *    - How to get item data from slots
 *
 * 2. Item Data Structure (MC 1.21.1)
 *    - Items use component-based format
 *    - Lore is in item.components array
 *    - Need to find component with type === 'lore'
 *    - Parse component.data array
 *
 * 3. Packet Sending
 *    - How to send window_click packets
 *    - How to track state IDs from server packets
 *    - Structure of click packets in Azalea
 *
 * 4. Event System
 *    - Container/menu open events
 *    - Container update events
 *    - Window close events
 *    - Chat message events
 *
 * Key Resources:
 * - Azalea docs: https://docs.rs/azalea/
 * - Azalea GitHub: https://github.com/azalea-rs/azalea
 * - Azalea examples: https://github.com/azalea-rs/azalea/tree/main/azalea/examples
 * - JavaScript reference: bot.js in this repository
 *
 * State ID Tracking is CRITICAL:
 * - MC 1.21.1 requires accurate state IDs in window clicks
 * - Wrong state ID = "Invalid sequence" disconnect
 * - Must listen to window_items and set_slot packets
 * - See bot.js lines 161-216 for implementation pattern
 */
