# Current Status: Why Doesn't Everything Work?

## TL;DR
**The Rust version is incomplete.** Use the **JavaScript version** for full functionality.

---

## What You Have

This repository contains **TWO separate implementations**:

### 1. JavaScript/Mineflayer Version ✅ FULLY FUNCTIONAL
**Location**: `bot.js`, `package.json`  
**Status**: Production-ready, all features working  
**Use this if**: You want the bot to work NOW

**How to run**:
```bash
npm install
npm start
```

### 2. Rust/Azalea Version ⚠️ INCOMPLETE
**Location**: `src/`, `Cargo.toml`  
**Status**: Core infrastructure only, main features missing  
**Use this if**: You want to contribute to development or learn Rust

**How to run**:
```bash
cargo run --release
```

---

## Why Rust Version Doesn't Work

### ✅ What's Implemented (Works)
- Bot connection and authentication
- Configuration loading from config.json
- Chat message monitoring
- AFK detection
- Discord webhook notifications
- Price parsing

### ❌ What's Missing (Doesn't Work)
The **entire auction house automation** is missing:

1. **Window Interaction** - Can't open/read auction house windows
2. **Map Searching** - Can't search for cheap maps
3. **Purchasing** - Can't buy maps
4. **Listing** - Can't list maps for sale
5. **Inventory Management** - Can't manage map inventory

### Why It's Missing
The Rust version requires implementing Azalea's inventory/menu system, which is:
- Complex (different API than Mineflayer)
- Underdocumented in Azalea
- Requires specialized Rust + Minecraft protocol knowledge

---

## What Do You Need?

### Option 1: Use JavaScript Version (5 minutes)

**Recommended if**: You just want the bot to work

```bash
cd /home/runner/work/donutsmp-mapflipper/donutsmp-mapflipper
npm install
npm start
```

Edit `config.json` with your Minecraft email/settings first.

---

### Option 2: Complete Rust Version (4-8 hours)

**Recommended if**: You're a Rust developer who wants to contribute

**What needs to be implemented**:

1. **Inventory Event Handling** (`src/main.rs`)
   - Listen for container/menu open events in Azalea
   - Handle `Event::*` for inventory changes
   
2. **Window Interaction Module** (new file: `src/inventory.rs`)
   ```rust
   // Need to implement:
   - open_auction_house() -> Menu
   - find_cheap_maps(menu: &Menu) -> Vec<MapSlot>
   - click_slot(menu: &mut Menu, slot: usize)
   - verify_purchase() -> bool
   ```

3. **Item Parsing** (new file: `src/items.rs`)
   ```rust
   // Need to implement:
   - parse_item_lore(item: &ItemStack) -> Vec<String>
   - extract_price(lore: &[String]) -> Option<u32>
   - extract_seller(lore: &[String]) -> Option<String>
   ```

4. **Integration** (`src/main.rs`)
   - Connect inventory module to main loop
   - Handle purchase flow
   - Handle listing flow

**Challenges**:
- Azalea's inventory API is different from Mineflayer
- Minecraft 1.21.1 uses component-based item data (not NBT)
- Need to track window state IDs for protocol compliance
- Limited documentation/examples available

**Reference**:
- JavaScript implementation: `bot.js` lines 323-600
- Azalea docs: https://docs.rs/azalea/latest/azalea/

---

### Option 3: Ask for Specific Help

If you want to complete the Rust version but need help:

**Tell us**:
1. What Rust experience do you have?
2. What Minecraft protocol knowledge do you have?
3. Have you worked with Azalea before?
4. What specific part do you need help with?

We can provide:
- Code examples
- Step-by-step guidance
- Pair programming assistance
- Code review

---

## Quick Decision Tree

```
Do you need the bot working NOW?
├─ YES → Use JavaScript version (npm start)
└─ NO → Do you know Rust well?
    ├─ YES → Want to implement Azalea inventory system?
    │   ├─ YES → Follow "Complete Rust Version" above
    │   └─ NO → Use JavaScript version
    └─ NO → Use JavaScript version
```

---

## Current State Summary

| Feature | JavaScript | Rust |
|---------|-----------|------|
| Connection | ✅ | ✅ |
| Authentication | ✅ | ✅ |
| Configuration | ✅ | ✅ |
| Chat monitoring | ✅ | ✅ |
| AFK detection | ✅ | ✅ |
| Webhooks | ✅ | ✅ |
| **Window interaction** | ✅ | ❌ |
| **Map purchasing** | ✅ | ❌ |
| **Map listing** | ✅ | ❌ |
| **Inventory management** | ✅ | ❌ |

**Bottom line**: JavaScript has 100% functionality, Rust has ~40% functionality.

---

## Next Steps

1. **Decide which version to use** (see decision tree above)
2. **If JavaScript**: 
   - Run `npm install && npm start`
   - Check logs for any issues
   - Verify connection to server
3. **If Rust development**:
   - Let us know your skill level
   - Ask for specific help with Azalea
   - Start with one module at a time

---

## Need More Help?

Reply with:
- Which version you want to use
- What error messages you're seeing (if any)
- What you're trying to accomplish
- Your level of Rust/JavaScript experience

We can provide targeted assistance based on your needs!
