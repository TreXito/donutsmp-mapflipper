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

// Delay after moving items in inventory
const INVENTORY_MOVE_DELAY: u64 = 200;

// Delay after selecting hotbar slot
const HOTBAR_SELECTION_DELAY: u64 = 300;

/// Check if an ItemStack contains a map
/// Uses debug string matching as item.kind doesn't expose direct enum comparison
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
            // Only check slots 9-44 (main inventory + hotbar)
            (9..=44)
                .find(|&idx| idx < slots.len() && slots[idx].is_empty())
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
                                println!("[INVENTORY] Server rejected the click - will retry next iteration");
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
/// Flow:
/// 1. First unstack all maps so each map is in its own slot
/// 2. Find all single map slots
/// 3. For each map slot, list it individually using the /ah list command
///
/// The correct sell loop according to the problem statement:
/// - First: while slot.count > 1, unstack (right-click + left-click)
/// - Then: for each single map, select hotbar, /ah sell, confirm
pub async fn list_maps(bot: &Client, config: &Config, slots_to_list: &[usize]) -> Result<()> {
    if slots_to_list.is_empty() {
        println!("[LISTING] No maps to list");
        return Ok(());
    }
    
    println!("[LISTING] Starting to list {} map slot(s)...", slots_to_list.len());
    
    // Step 1: Unstack all maps first
    println!("[LISTING] Step 1: Unstacking maps...");
    if let Err(e) = unstack_maps(bot).await {
        eprintln!("[LISTING] Error during unstacking: {}", e);
        return Err(e);
    }
    
    // Step 2: Get fresh inventory snapshot after unstacking
    let inv = bot.get_inventory();
    let single_maps: Vec<usize> = if let Some(menu) = inv.menu() {
        let slots = menu.slots();
        slots.iter().enumerate()
            .filter(|(_, slot)| is_map_item(slot))
            .map(|(idx, _)| idx)
            .collect()
    } else {
        vec![]
    };
    
    if single_maps.is_empty() {
        println!("[LISTING] No maps found after unstacking");
        return Ok(());
    }
    
    println!("[LISTING] Step 2: Found {} single map(s) to list", single_maps.len());
    
    // Step 3: List each map one at a time
    let max_listings = config.max_listings_per_cycle as usize;
    let maps_to_list = single_maps.iter().take(max_listings).copied().collect::<Vec<_>>();
    
    println!("[LISTING] Step 3: Listing up to {} maps...", maps_to_list.len());
    
    for (idx, map_slot) in maps_to_list.iter().enumerate() {
        println!("[LISTING] Listing map {}/{} from slot {}...", idx + 1, maps_to_list.len(), map_slot);
        
        // Move map to hotbar slot 0 if not already there
        const HOTBAR_SLOT_0: usize = 36;
        if *map_slot != HOTBAR_SLOT_0 {
            let inv_handle = bot.get_inventory();
            inv_handle.left_click(*map_slot);
            sleep(Duration::from_millis(200)).await;
            
            let inv_handle2 = bot.get_inventory();
            inv_handle2.left_click(HOTBAR_SLOT_0);
            sleep(Duration::from_millis(200)).await;
        }
        
        // Select hotbar slot 0
        bot.set_selected_hotbar_slot(0);
        sleep(Duration::from_millis(300)).await;
        
        // Send /ah sell command (using sell_price from config, e.g., "9.9k")
        let command = format!("/ah sell {}", config.sell_price);
        println!("[LISTING] Sending command: {}", command);
        bot.chat(&command);
        sleep(Duration::from_millis(500)).await;
        
        // Wait for confirmation window
        let timeout_ticks = (config.window_timeout + 50 - 1) / 50;
        match bot.wait_for_container_open(Some(timeout_ticks as usize)).await {
            Some(confirm_container) => {
                // Wait before clicking confirm to avoid spam kick
                sleep(Duration::from_millis(300)).await;
                
                // Click confirm button (slot 15)
                println!("[LISTING] Clicking confirm button (slot 15)...");
                confirm_container.left_click(15_usize);
                
                // Forget the handle to prevent early closure
                std::mem::forget(confirm_container);
                
                // Wait for the window to close and server to update inventory
                // This is critical - the server needs time to process the listing
                sleep(Duration::from_millis(500)).await;
                
                // Wait additional time to ensure inventory state is updated
                sleep(Duration::from_millis(500)).await;
                
                // Verify map was consumed by checking inventory
                let verify_inv = bot.get_inventory();
                let mut listing_success = false;
                if let Some(menu) = verify_inv.menu() {
                    let slots = menu.slots();
                    if HOTBAR_SLOT_0 < slots.len() {
                        match &slots[HOTBAR_SLOT_0] {
                            ItemStack::Present(_) => {
                                if is_map_item(&slots[HOTBAR_SLOT_0]) {
                                    println!("[LISTING] WARNING: Map still in slot {} after listing - listing may have failed!", HOTBAR_SLOT_0);
                                    listing_success = false;
                                } else {
                                    // Different item in slot, consider it consumed
                                    println!("[LISTING] ✓ Verified: Different item in slot {}, map was consumed", HOTBAR_SLOT_0);
                                    listing_success = true;
                                }
                            }
                            ItemStack::Empty => {
                                println!("[LISTING] ✓ Verified: Map removed from slot {}", HOTBAR_SLOT_0);
                                listing_success = true;
                            }
                        }
                    }
                }
                
                if listing_success {
                    println!("[LISTING] ✓ Map {} listed successfully", idx + 1);
                } else {
                    println!("[LISTING] ✗ Map {} listing may have failed - map still present", idx + 1);
                }
                
                // Wait between listings to avoid command cooldown
                sleep(Duration::from_millis(config.delay_between_listings)).await;
            }
            None => {
                println!("[LISTING] ERROR: Confirmation window did not open for map at slot {}", map_slot);
                // Continue to next map instead of failing entire operation
            }
        }
    }
    
    println!("[LISTING] Finished listing {} map(s)", maps_to_list.len());
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
