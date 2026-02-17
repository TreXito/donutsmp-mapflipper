use azalea::prelude::*;
use azalea::container::ContainerClientExt;
use azalea::inventory::{ItemStack, Menu};
use azalea::inventory::components::Lore;
use anyhow::{Result, anyhow};
use std::time::Duration;
use tokio::time::sleep;
use crate::config::Config;
use crate::price_parser::{parse_price, format_price};

// Minecraft server tick rate: 1 tick = 50 milliseconds
const MS_PER_TICK: u64 = 50;

// Delay after moving items in inventory
const INVENTORY_MOVE_DELAY: u64 = 200;

// Delay after selecting hotbar slot
const HOTBAR_SELECTION_DELAY: u64 = 300;

/// Check if an ItemStack contains a map
/// 
/// Note: Uses debug string matching as Azalea's item.kind doesn't expose direct enum comparison.
/// This is a known limitation that may require updates if the debug format changes in future versions.
fn is_map_item(item: &ItemStack) -> bool {
    if let ItemStack::Present(data) = item {
        let item_name = format!("{:?}", data.kind);
        item_name.to_lowercase().contains("map")
    } else {
        false
    }
}

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
/// Minecraft maps can sometimes come in stacks (multiple maps in one slot).
/// This function separates stacked maps into individual slots so they can be listed separately.
/// 
/// IMPORTANT: Right-click picks up HALF the stack (rounded up), NOT one item.
/// For example:
/// - Stack of 10: right-click picks up 5, leaves 5
/// - Stack of 9: right-click picks up 5, leaves 4
/// - Stack of 2: right-click picks up 1, leaves 1
/// 
/// Strategy: Repeatedly right-click stacks and place them in empty slots until all maps are singles.
pub async fn unstack_maps(bot: &Client) -> Result<()> {
    println!("[INVENTORY] Checking for stacked maps...");
    
    let max_iterations = 100; // Safety limit to prevent infinite loops
    let mut iteration = 0;
    
    loop {
        iteration += 1;
        if iteration > max_iterations {
            println!("[INVENTORY] Warning: Hit max iterations ({}), stopping unstacking", max_iterations);
            break;
        }
        
        // Get a fresh inventory handle to check current state
        let inventory_handle = bot.get_inventory();
        
        let stacked_slot = if let Some(menu) = inventory_handle.menu() {
            let slots = menu.slots();
            
            // Find the first stacked map (count > 1)
            let mut found = None;
            for (idx, slot) in slots.iter().enumerate() {
                if let ItemStack::Present(data) = slot {
                    if is_map_item(slot) && data.count > 1 {
                        found = Some((idx, data.count));
                        break;
                    }
                }
            }
            found
        } else {
            None
        };
        
        // If no stacked maps found, we're done
        if stacked_slot.is_none() {
            println!("[INVENTORY] ✓ All maps are now singles (after {} iteration(s))", iteration - 1);
            break;
        }
        
        let (stack_slot, count) = stacked_slot.unwrap();
        println!("[INVENTORY] Found stack of {} maps at slot {}, splitting...", count, stack_slot);
        
        // Find an empty slot in main inventory (9-35) or hotbar (36-44)
        // NEVER use slots 0-8 (crafting output + grid, armor) or 45 (offhand)
        let inv_handle = bot.get_inventory();
        let empty_slot = if let Some(menu) = inv_handle.menu() {
            let slots = menu.slots();
            // Only check slots 9-44 (main inventory + hotbar), but respect actual slot count
            (9..slots.len().min(45))
                .find(|&idx| slots[idx].is_empty())
        } else {
            None
        };
        
        match empty_slot {
            Some(empty_idx) => {
                // Right-click the stack to pick up HALF (rounded up)
                // This splits the stack into two parts
                println!("[INVENTORY] Right-clicking slot {} to pick up half...", stack_slot);
                inv_handle.right_click(stack_slot);
                sleep(Duration::from_millis(300)).await;
                
                // Get a fresh handle and left-click empty slot to place the half
                let inv_handle2 = bot.get_inventory();
                println!("[INVENTORY] Left-clicking empty slot {} to place...", empty_idx);
                inv_handle2.left_click(empty_idx);
                
                // Wait for server to acknowledge the change (300ms)
                sleep(Duration::from_millis(300)).await;
                
                // Verify the split actually worked by checking the original slot count
                let verify_handle = bot.get_inventory();
                if let Some(menu) = verify_handle.menu() {
                    let slots = menu.slots();
                    if stack_slot < slots.len() {
                        if let ItemStack::Present(data) = &slots[stack_slot] {
                            let new_count = data.count;
                            if new_count == count {
                                println!("[INVENTORY] WARNING: Split operation failed - count didn't change (still {})", count);
                                println!("[INVENTORY] Server rejected the click - operation will be retried in next loop iteration");
                            } else if new_count > count {
                                println!("[INVENTORY] WARNING: Unexpected behavior - count increased from {} to {}", count, new_count);
                            } else {
                                println!("[INVENTORY] ✓ Split verified: slot {} now has {} maps (was {})", stack_slot, new_count, count);
                            }
                        }
                    }
                }
            }
            None => {
                println!("[INVENTORY] ERROR: No empty slots available to unstack!");
                println!("[INVENTORY] Inventory is full - cannot continue unstacking");
                return Err(anyhow!("Inventory full, cannot unstack maps"));
            }
        }
    }
    
    Ok(())
}

/// List all maps from inventory on auction house
///
/// UPDATED STRATEGY: List entire stacks without unstacking
/// When holding a stack, /ah sell lists the ENTIRE STACK as one listing
/// Price must be fair: (single_price × stack_count × 0.5)
///
/// Flow:
/// 1. Find all map slots (including stacks)
/// 2. For each slot, move to hotbar and list the entire stack at calculated price
/// 3. Price calculation: base_price × count × 0.5
pub async fn list_maps(bot: &Client, config: &Config, slots_to_list: &[usize]) -> Result<()> {
    if slots_to_list.is_empty() {
        println!("[LISTING] No maps to list");
        return Ok(());
    }
    
    println!("[LISTING] Starting to list maps (listing stacks without unstacking)...");
    
    // Parse the base sell price from config (e.g., "9.9k" -> 9900)
    let base_price_str = format!("${}", config.sell_price);
    let base_price = match parse_price(&base_price_str) {
        Some(price) => price,
        None => {
            return Err(anyhow!("Failed to parse sell price from config: {}", config.sell_price));
        }
    };
    println!("[LISTING] Base single map price: ${} ({})", base_price, config.sell_price);
    
    // Get fresh inventory snapshot
    let inv = bot.get_inventory();
    let map_slots: Vec<(usize, i32)> = if let Some(menu) = inv.menu() {
        let slots = menu.slots();
        slots.iter().enumerate()
            .filter_map(|(idx, slot)| {
                if is_map_item(slot) {
                    if let ItemStack::Present(data) = slot {
                        Some((idx, data.count))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    };
    
    if map_slots.is_empty() {
        println!("[LISTING] No maps found in inventory");
        return Ok(());
    }
    
    println!("[LISTING] Found {} map slot(s)", map_slots.len());
    
    // Track how many listings we've made
    let mut listings_made = 0;
    let max_listings = config.max_listings_per_cycle as usize;
    
    // List each stack
    for (slot_idx, stack_count) in map_slots {
        if listings_made >= max_listings {
            println!("[LISTING] Reached max listings per cycle ({})", max_listings);
            break;
        }
        
        println!("[LISTING] Processing slot {} with {} map(s)...", slot_idx, stack_count);
        
        // Calculate fair price for the stack: base_price × count × 0.5
        // Using integer arithmetic to avoid floating-point precision issues
        let stack_price = (base_price * stack_count as u32) / 2;
        let price_str = format_price(stack_price);
        
        println!("[LISTING] Stack of {} maps: ${} each × {} × 0.5 = ${} total ({})", 
                 stack_count, base_price, stack_count, stack_price, price_str);
        
        // Move stack to hotbar slot 0
        const HOTBAR_SLOT_0: usize = 36;
        if slot_idx != HOTBAR_SLOT_0 {
            println!("[LISTING] Moving stack from slot {} to hotbar slot 0...", slot_idx);
            
            // Log before first click
            println!("[INVENTORY DEBUG] About to left-click slot {} (pickup stack)", slot_idx);
            let inv_handle = bot.get_inventory();
            let window_id = inv_handle.id();
            println!("[INVENTORY DEBUG] Window ID: {}", window_id);
            inv_handle.left_click(slot_idx);
            sleep(Duration::from_millis(200)).await;
            
            // Log before second click
            println!("[INVENTORY DEBUG] About to left-click slot {} (place stack)", HOTBAR_SLOT_0);
            let inv_handle2 = bot.get_inventory();
            inv_handle2.left_click(HOTBAR_SLOT_0);
            sleep(Duration::from_millis(200)).await;
            
            // Verify the move
            let verify_handle = bot.get_inventory();
            if let Some(menu) = verify_handle.menu() {
                let slots = menu.slots();
                if HOTBAR_SLOT_0 < slots.len() {
                    match &slots[HOTBAR_SLOT_0] {
                        ItemStack::Present(data) if is_map_item(&slots[HOTBAR_SLOT_0]) => {
                            println!("[INVENTORY DEBUG] ✓ Verified: {} maps now in hotbar slot 0", data.count);
                        }
                        ItemStack::Empty => {
                            println!("[INVENTORY DEBUG] ✗ WARNING: Hotbar slot 0 is empty after move!");
                        }
                        _ => {
                            println!("[INVENTORY DEBUG] ✗ WARNING: Different item in hotbar slot 0!");
                        }
                    }
                }
            }
        } else {
            println!("[LISTING] Stack already in hotbar slot 0");
        }
        
        // Select hotbar slot 0 to hold the stack
        println!("[LISTING] Selecting hotbar slot 0...");
        bot.set_selected_hotbar_slot(0);
        sleep(Duration::from_millis(300)).await;
        
        // Log what we're about to list
        let pre_list_inv = bot.get_inventory();
        if let Some(menu) = pre_list_inv.menu() {
            let slots = menu.slots();
            if HOTBAR_SLOT_0 < slots.len() {
                match &slots[HOTBAR_SLOT_0] {
                    ItemStack::Present(data) if is_map_item(&slots[HOTBAR_SLOT_0]) => {
                        println!("[LISTING DEBUG] Holding {} maps in selected hotbar slot 0", data.count);
                    }
                    ItemStack::Empty => {
                        println!("[LISTING DEBUG] ✗ ERROR: Hotbar slot 0 is EMPTY before command!");
                    }
                    ItemStack::Present(data) => {
                        println!("[LISTING DEBUG] ✗ ERROR: Holding wrong item: {:?}", data.kind);
                    }
                }
            }
        }
        
        // Send /ah sell command with calculated price for the stack
        let command = format!("/ah sell {}", price_str);
        println!("[LISTING] Sending command: {}", command);
        bot.chat(&command);
        sleep(Duration::from_millis(500)).await;
        
        // Wait for confirmation window
        let timeout_ticks = (config.window_timeout + 50 - 1) / 50;
        println!("[LISTING] Waiting for confirmation window (timeout: {}ms)...", config.window_timeout);
        match bot.wait_for_container_open(Some(timeout_ticks as usize)).await {
            Some(confirm_container) => {
                let container_id = confirm_container.id();
                println!("[LISTING] ✓ Confirmation window opened (container ID: {})", container_id);
                
                // Log what's in the confirmation window
                if let Some(menu) = confirm_container.menu() {
                    let slots = menu.slots();
                    println!("[LISTING DEBUG] Confirmation window has {} slots", slots.len());
                    // The item to be listed should be visible in the confirmation window
                    // Usually in a specific slot depending on server implementation
                }
                
                // Wait before clicking confirm to avoid spam kick
                sleep(Duration::from_millis(300)).await;
                
                // Click confirm button (slot 15)
                println!("[LISTING] Clicking confirm button (slot 15)...");
                println!("[INVENTORY DEBUG] Container {}: left-click slot 15 (confirm)", container_id);
                confirm_container.left_click(15_usize);
                
                // Forget the handle to prevent early closure
                std::mem::forget(confirm_container);
                
                // Wait for the window to close and server to update inventory
                sleep(Duration::from_millis(1000)).await;
                
                // Verify listing by checking if slot is now empty or changed
                let verify_inv = bot.get_inventory();
                let mut listing_success = false;
                if let Some(menu) = verify_inv.menu() {
                    let slots = menu.slots();
                    if HOTBAR_SLOT_0 < slots.len() {
                        match &slots[HOTBAR_SLOT_0] {
                            ItemStack::Empty => {
                                println!("[LISTING] ✓ Slot now empty - stack listed successfully");
                                listing_success = true;
                            }
                            ItemStack::Present(data) if is_map_item(&slots[HOTBAR_SLOT_0]) => {
                                let remaining = data.count;
                                if remaining < stack_count {
                                    // Partial listing occurred - this might indicate server only listed some maps
                                    // This could happen if /ah sell command lists ONE map at a time instead of the whole stack
                                    println!("[LISTING] ⚠ Partial consumption: {} maps remain (was {})", remaining, stack_count);
                                    println!("[LISTING] This suggests server listed only {} map(s), not the entire stack", stack_count - remaining);
                                    listing_success = true; // Consider it partially successful
                                } else {
                                    println!("[LISTING] ✗ Stack unchanged - listing failed (still {} maps)", remaining);
                                }
                            }
                            _ => {
                                println!("[LISTING] ✓ Different item in slot, maps were consumed");
                                listing_success = true;
                            }
                        }
                    }
                }
                
                if listing_success {
                    listings_made += 1;
                    println!("[LISTING] ✓ Successfully listed stack {} / {}", listings_made, max_listings);
                } else {
                    println!("[LISTING] ✗ Listing verification failed - skipping");
                }
                
                // Wait between listings to avoid command cooldown
                sleep(Duration::from_millis(config.delay_between_listings)).await;
            }
            None => {
                println!("[LISTING] ERROR: Confirmation window did not open for stack at slot {}", slot_idx);
            }
        }
    }
    
    println!("[LISTING] Finished - listed {} stack(s)", listings_made);
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
            if slot.is_present() && is_map_item(slot) {
                map_slots.push(idx);
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
