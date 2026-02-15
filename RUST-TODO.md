# What's Needed to Complete the Rust Version

## Overview
The Rust version has ~40% of functionality implemented. Here's exactly what's missing and what needs to be done.

---

## Missing Components

### 1. Inventory/Menu Event Handling

**File**: `src/main.rs` (expand `handle_event` function)

**What's needed**:
```rust
async fn handle_event(bot: Client, event: Event, state: BotState) -> Result<()> {
    match event {
        Event::Chat(m) => { /* Already implemented */ },
        
        // TODO: Add these event handlers
        Event::MenuOpened(menu) => {
            // Handle auction house window opening
            // Check if it's the /ah map window
            // Start scanning for maps
        },
        
        Event::SetContainerContent(container) => {
            // Handle container updates
            // Parse item data
            // Look for cheap maps
        },
        
        // ... other inventory events
        _ => {}
    }
    Ok(())
}
```

**Challenge**: Azalea's Event enum may not have these exact variants. Need to explore Azalea documentation.

---

### 2. Inventory Module

**File**: `src/inventory.rs` (NEW FILE)

**What's needed**:

```rust
use azalea::prelude::*;
use azalea::inventory::*;

/// Open the auction house and return the menu
pub async fn open_auction_house(bot: &Client) -> Result<Menu> {
    // 1. Close any existing windows
    // 2. Send /ah map command
    // 3. Wait for menu open event
    // 4. Return the menu
    todo!("Implement based on Azalea's inventory API")
}

/// Find cheap maps in the auction house menu
pub fn find_cheap_maps(menu: &Menu, max_price: u32) -> Vec<MapSlot> {
    // 1. Iterate through menu slots
    // 2. Parse item lore for prices
    // 3. Filter maps under max_price
    // 4. Return list of purchasable maps
    todo!("Need to understand Azalea's Menu structure")
}

/// Click a slot in the menu
pub async fn click_slot(bot: &Client, menu: &Menu, slot: usize) -> Result<()> {
    // 1. Send window click packet
    // 2. Track state ID for protocol compliance
    // 3. Wait for server confirmation
    todo!("Need Azalea's packet sending API")
}

pub struct MapSlot {
    pub slot_index: usize,
    pub price: u32,
    pub seller: String,
}
```

**Challenges**:
- Azalea's Menu API structure unknown
- Slot clicking mechanism different from Mineflayer
- State ID tracking for Minecraft 1.21.1

---

### 3. Item Parsing Module

**File**: `src/items.rs` (NEW FILE)

**What's needed**:

```rust
use azalea::inventory::ItemStack;

/// Extract lore from Minecraft 1.21.1 component-based items
pub fn extract_lore(item: &ItemStack) -> Vec<String> {
    // In Minecraft 1.21.1+, lore is in item.components
    // Not in item.nbt like older versions
    
    // 1. Find component with type === 'lore'
    // 2. Parse component.data array
    // 3. Extract text from each line
    // 4. Return as Vec<String>
    todo!("Need to understand Azalea's ItemStack structure")
}

/// Parse price from lore lines
pub fn extract_price(lore: &[String]) -> Option<u32> {
    // 1. Find line containing "Price: $"
    // 2. Use regex to extract price
    // 3. Handle K suffix (5K = 5000)
    // Already have this logic in price_parser.rs
    // Just need to integrate
}

/// Extract seller name from lore
pub fn extract_seller(lore: &[String]) -> Option<String> {
    // 1. Find line containing "Seller:"
    // 2. Extract and return seller name
    todo!()
}
```

**Challenges**:
- Azalea's ItemStack structure unknown
- Component-based data format (MC 1.21.1)
- May need NBT parsing library

---

### 4. Purchase Flow

**File**: `src/main.rs` (integrate into `run_cycle`)

**What's needed**:

```rust
async fn run_cycle(bot: Client, state: BotState) -> Result<bool> {
    // 1. Open auction house
    let menu = open_auction_house(&bot).await?;
    
    // 2. Find cheap maps
    let maps = find_cheap_maps(&menu, state.config.max_buy_price);
    
    if maps.is_empty() {
        println!("[AH] No cheap maps found");
        return Ok(false);
    }
    
    // 3. Try to buy first map
    let map = &maps[0];
    println!("[AH] Found map at ${}, attempting purchase...", map.price);
    
    // 4. Click map slot
    click_slot(&bot, &menu, map.slot_index).await?;
    
    // 5. Wait for window update
    sleep(Duration::from_millis(1000)).await;
    
    // 6. Click confirm button (usually slot 15)
    click_slot(&bot, &menu, 15).await?;
    
    // 7. Wait for purchase confirmation
    // Check chat for "already bought" message
    // Send webhook notification
    
    Ok(true)
}
```

**Challenges**:
- Proper error handling
- Window closing detection
- Purchase verification (chat messages)

---

### 5. Listing Flow

**File**: `src/inventory.rs` or `src/main.rs`

**What's needed**:

```rust
async fn list_maps(bot: &Client, config: &Config) -> Result<()> {
    // 1. Get bot's inventory
    let inventory = bot.inventory();
    
    // 2. Find maps in hotbar slots 0-8
    let maps = inventory.find_items("map");
    
    // 3. For each map:
    for (slot, map) in maps {
        // 4. Select hotbar slot
        bot.set_held_item(slot);
        
        // 5. Send /ah sell command
        bot.chat(&format!("/ah sell {}", config.sell_price));
        
        // 6. Wait for confirmation
        sleep(Duration::from_millis(500)).await;
    }
    
    Ok(())
}
```

**Challenges**:
- Inventory access API in Azalea
- Hotbar slot selection
- Timing between listings

---

## Technical Challenges

### 1. Azalea API Differences
- **Mineflayer**: `bot.clickWindow(window, slot, ...)`
- **Azalea**: Unknown, need to research

### 2. Minecraft 1.21.1 Protocol
- Item data moved from NBT to components
- Different structure for lore/display names
- Need to understand component format

### 3. State Tracking
- Minecraft 1.21.1 requires accurate stateId tracking
- Each window click needs correct stateId
- Missing stateId = "Invalid sequence" kick

### 4. Event Handling
- Need to identify correct Azalea Event variants
- May need to use lower-level packet handling
- Async event coordination

---

## Resources Needed

### Documentation
- [ ] Azalea inventory module docs
- [ ] Azalea Event enum reference
- [ ] Minecraft 1.21.1 protocol changes
- [ ] Component-based item format

### Examples
- [ ] Azalea bot that interacts with chests
- [ ] Azalea bot that clicks menu items
- [ ] Item parsing in Minecraft 1.21.1

### Tools
- [ ] Minecraft packet logger
- [ ] NBT/Component inspector
- [ ] Azalea debugging tools

---

## Estimated Effort

| Task | Complexity | Time Est. | Blocker |
|------|-----------|-----------|---------|
| Inventory events | Medium | 2-3h | Azalea docs |
| Menu interaction | High | 3-4h | API unknown |
| Item parsing | Medium | 2-3h | Component format |
| Purchase flow | Medium | 2h | Previous tasks |
| Listing flow | Easy | 1h | Previous tasks |
| Testing | Medium | 2h | Server access |
| **Total** | - | **12-15h** | - |

**Prerequisites**:
- Strong Rust knowledge
- Understanding of async/await
- Minecraft protocol familiarity
- Azalea framework experience (or willingness to learn)

---

## How to Contribute

### Step 1: Research Phase
1. Read Azalea documentation thoroughly
2. Find examples of inventory interaction
3. Identify relevant Event variants
4. Study Azalea's ItemStack structure

### Step 2: Spike Implementation
1. Create minimal working example
2. Open a chest and read contents
3. Click an item in a menu
4. Verify it works on test server

### Step 3: Integrate
1. Adapt spike code to this project
2. Add error handling
3. Add logging
4. Test thoroughly

### Step 4: Complete Features
1. Implement purchase flow
2. Implement listing flow
3. Add verification logic
4. Test on live server

---

## Alternative Approach

If Azalea's high-level API isn't sufficient:

### Use Lower-Level Packet Handling

```rust
// Send custom packets directly
bot.send_packet(WindowClickPacket {
    window_id: menu.id(),
    slot: 15,
    button: 0,
    mode: ClickMode::Click,
    state_id: tracked_state_id,
    ...
});

// Listen for packets
bot.on_packet::<SetContainerContentPacket>(|packet| {
    // Handle container updates
});
```

This gives more control but requires deeper protocol knowledge.

---

## Questions to Answer

Before starting implementation:

1. **What Azalea version are we using?** (0.15.0+mc1.21.11)
2. **Does Azalea support inventory interaction?** (Research needed)
3. **What events does Azalea emit for menus?** (Check Event enum)
4. **How do we access ItemStack data?** (Check Azalea docs)
5. **Can we send custom packets?** (Fallback option)

---

## Summary

**To complete the Rust version, you need**:
1. Azalea inventory API knowledge
2. Minecraft 1.21.1 protocol understanding  
3. 12-15 hours of development time
4. Rust async/await proficiency

**OR just use the JavaScript version** which works perfectly!

---

## Need Help?

Tell us:
- Your Rust experience level
- Familiarity with Azalea
- Minecraft protocol knowledge
- Specific questions about implementation

We can provide:
- Code examples
- API guidance
- Architecture advice
- Code review
