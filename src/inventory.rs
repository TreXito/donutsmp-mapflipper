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

// Delay between unstack operations (kept for potential future use)
const UNSTACK_DELAY: u64 = 100;

// Delay after moving items in inventory
const INVENTORY_MOVE_DELAY: u64 = 200;

// Delay after selecting hotbar slot
const HOTBAR_SELECTION_DELAY: u64 = 300;

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

/// Unstack all stacked maps in inventory into individual slots
///
/// NOTE: This function is currently UNUSED as we now handle stacks naturally during listing.
/// Kept for potential future use or alternative unstacking strategies.
///
/// Minecraft maps can sometimes come in stacks (multiple maps in one slot).
/// This function separates stacked maps into individual slots so they can be listed separately.
/// 
/// Reference: bot.js lines 721-777
#[allow(dead_code)]
pub async fn unstack_maps(bot: &Client) -> Result<()> {
    println!("[INVENTORY] Checking for stacked maps...");
    
    // Get a fresh inventory handle to check current state
    let inventory_handle = bot.get_inventory();
    
    if let Some(menu) = inventory_handle.menu() {
        let slots = menu.slots();
        
        // Find all stacked maps (maps with count > 1)
        let mut stacked_slots = Vec::new();
        for (idx, slot) in slots.iter().enumerate() {
            if let ItemStack::Present(data) = slot {
                let item_name = format!("{:?}", data.kind);
                if item_name.to_lowercase().contains("map") && data.count > 1 {
                    stacked_slots.push((idx, data.count));
                }
            }
        }
        
        if stacked_slots.is_empty() {
            println!("[INVENTORY] No stacked maps found, all maps are singles");
            return Ok(());
        }
        
        println!("[INVENTORY] Found {} stacked map slot(s) to unstack", stacked_slots.len());
        
        for (stack_slot, count) in stacked_slots {
            println!("[INVENTORY] Unstacking {} maps from slot {}...", count, stack_slot);
            
            // Unstack by right-clicking to pick up 1, then left-clicking to place in empty slot
            // We need to do this (count - 1) times to separate all maps
            for _ in 0..(count - 1) {
                // Get a fresh inventory handle for each click operation
                // This ensures we use the current container ID which may change between operations
                let inv_handle = bot.get_inventory();
                
                // Find an empty slot in the current inventory state
                let empty_slot = if let Some(menu) = inv_handle.menu() {
                    let slots = menu.slots();
                    let mut found_slot = None;
                    for (idx, slot) in slots.iter().enumerate() {
                        if slot.is_empty() {
                            found_slot = Some(idx);
                            break;
                        }
                    }
                    found_slot
                } else {
                    None
                };
                
                match empty_slot {
                    Some(empty_idx) => {
                        // Right-click the stack to pick up 1 map
                        inv_handle.right_click(stack_slot);
                        sleep(Duration::from_millis(UNSTACK_DELAY)).await;
                        
                        // Get another fresh handle for the left-click to ensure correct container ID
                        let inv_handle_placement = bot.get_inventory();
                        
                        // Left-click the empty slot to place it
                        inv_handle_placement.left_click(empty_idx);
                        sleep(Duration::from_millis(UNSTACK_DELAY)).await;
                    }
                    None => {
                        println!("[INVENTORY] Warning: No empty slots to unstack, inventory full");
                        break;
                    }
                }
            }
        }
        
        println!("[INVENTORY] Unstacking complete");
    }
    
    Ok(())
}

/// List all maps from inventory on auction house
///
/// This is a bot-friendly approach that lists ALL maps in inventory, handling stacks naturally.
/// For each map found (whether stacked or single), it repeatedly lists until the slot is empty.
/// This efficiently clears the entire inventory without needing a separate unstacking step.
///
/// Flow for each map slot:
/// 1. While slot has maps: Move ONE map to hotbar -> Hold -> List -> Repeat
/// 2. This naturally handles stacks by listing them one at a time
///
/// Reference: bot.js lines 974-1064
pub async fn list_maps(bot: &Client, config: &Config, slots_to_list: &[usize]) -> Result<()> {
    if slots_to_list.is_empty() {
        println!("[LISTING] No maps to list");
        return Ok(());
    }
    
    println!("[LISTING] Starting to list maps from {} slot(s)...", slots_to_list.len());
    
    // The hotbar starts at slot 36 in Minecraft inventory
    // Slot 36 is hotbar index 0 (the leftmost hotbar slot)
    const HOTBAR_SLOT_0: usize = 36;
    
    let mut total_listed = 0;
    let max_listings = config.max_listings_per_cycle as usize;
    
    // Process each slot that contains maps
    for &slot_idx in slots_to_list {
        // Keep listing from this slot until it's empty or we hit the listing limit
        loop {
            // Check if we've hit the listing limit
            if total_listed >= max_listings {
                println!("[LISTING] Reached listing limit of {} maps per cycle", max_listings);
                println!("[LISTING] Remaining maps will be listed in the next cycle");
                return Ok(());
            }
            
            // Get fresh inventory handle to check current state
            let inventory_handle = bot.get_inventory();
            
            if let Some(menu) = inventory_handle.menu() {
                let slots = menu.slots();
                
                if slot_idx >= slots.len() {
                    break; // Slot out of bounds, move to next
                }
                
                let slot = &slots[slot_idx];
                
                // Check if slot still has maps
                if !slot.is_present() {
                    break; // Slot is now empty, move to next slot
                }
                
                // Verify it's still a map
                let is_map = if let ItemStack::Present(data) = slot {
                    let item_name = format!("{:?}", data.kind);
                    let count = data.count;
                    if item_name.to_lowercase().contains("map") {
                        if count > 1 {
                            println!("[LISTING] Slot {} has {} stacked maps, listing one...", slot_idx, count);
                        } else {
                            println!("[LISTING] Slot {} has 1 map, listing it...", slot_idx);
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                if !is_map {
                    break; // Not a map anymore, move to next slot
                }
                
                // List ONE map from this slot (works for both stacks and singles)
                
                // Step 1: Move ONE map to hotbar slot 0
                if slot_idx != HOTBAR_SLOT_0 {
                    // Right-click to pick up half (or just 1 if it's a single)
                    // This works well for both stacks and single maps
                    inventory_handle.right_click(slot_idx);
                    sleep(Duration::from_millis(INVENTORY_MOVE_DELAY)).await;
                    
                    // Get fresh handle and place it in hotbar
                    let inv_handle2 = bot.get_inventory();
                    inv_handle2.left_click(HOTBAR_SLOT_0);
                    sleep(Duration::from_millis(INVENTORY_MOVE_DELAY)).await;
                }
                
                // Step 2: Hold the item in hotbar slot 0
                bot.set_selected_hotbar_slot(0);
                sleep(Duration::from_millis(HOTBAR_SELECTION_DELAY)).await;
                
                // Step 3: Send /ah list command
                bot.chat(&format!("/ah list {}", config.sell_price));
                sleep(Duration::from_millis(500)).await;
                
                // Step 4: Wait for confirmation GUI
                let timeout_ticks = (config.window_timeout + MS_PER_TICK - 1) / MS_PER_TICK;
                
                match bot.wait_for_container_open(Some(timeout_ticks as usize)).await {
                    Some(confirm_container) => {
                        sleep(Duration::from_millis(300)).await;
                        
                        // Step 5: Click confirm
                        confirm_container.left_click(15_usize);
                        std::mem::forget(confirm_container);
                        
                        total_listed += 1;
                        println!("[LISTING] ✓ Map listed successfully ({}/{})", total_listed, max_listings);
                        
                        // Step 6: Delay before next listing
                        sleep(Duration::from_millis(config.delay_between_listings)).await;
                    }
                    None => {
                        println!("[LISTING] ERROR: Confirmation GUI did not open, skipping this map");
                        break; // Move to next slot on error
                    }
                }
            } else {
                break; // No menu available
            }
        }
    }
    
    println!("[LISTING] Finished listing {} map(s)", total_listed);
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
