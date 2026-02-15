# Migration Guide: JavaScript/Mineflayer → Rust/Azalea

This document explains the differences between the JavaScript and Rust implementations and helps you choose the right version for your needs.

## Quick Comparison

| Feature | JavaScript/Mineflayer | Rust/Azalea |
|---------|----------------------|-------------|
| **Language** | JavaScript (Node.js) | Rust (nightly) |
| **Runtime** | Interpreted | Native compiled |
| **Performance** | Good | Excellent |
| **Memory Usage** | Higher (GC overhead) | Lower (no GC) |
| **Type Safety** | Runtime | Compile-time |
| **Maturity** | ✅ Fully functional | ⚠️ Window interaction WIP |
| **Ease of Modification** | Easy | Moderate (requires Rust knowledge) |
| **Binary Size** | N/A (Node.js + modules) | ~20-30MB (static) |
| **Startup Time** | Fast | Very fast |
| **Cross-platform** | ✅ All platforms with Node.js | ✅ Native binaries |

## When to Use Each Version

### Use JavaScript/Mineflayer If:
- ✅ You need **full functionality immediately**
- ✅ You're **familiar with JavaScript/Node.js**
- ✅ You want to **quickly modify** the bot's behavior
- ✅ You're **prototyping** or learning
- ✅ You prefer **mature, battle-tested** code

### Use Rust/Azalea If:
- ✅ You want **maximum performance**
- ✅ You care about **low memory usage**
- ✅ You value **type safety** and compile-time guarantees
- ✅ You're **learning Rust** or building Rust projects
- ✅ You need **native executables** (no runtime required)
- ✅ You're willing to **wait for full functionality**

## Configuration Compatibility

Both versions use the **same config.json format**, so you can easily switch between them:

```json
{
  "host": "donutsmp.net",
  "port": 25565,
  "username": "your-email@example.com",
  "auth": "microsoft",
  "version": "1.21.11",
  "maxBuyPrice": 2500,
  "sellPrice": "9.9k",
  "delayBetweenCycles": 5000,
  "delayAfterJoin": 5000,
  "webhook": {
    "enabled": false,
    "url": "",
    "displayName": "DonutSMP Map Flipper",
    "events": {
      "purchase": true,
      "listing": true,
      "sale": true,
      "afk": true,
      "error": true,
      "startup": true
    }
  }
}
```

## Feature Parity Status

### ✅ Fully Implemented in Both Versions:
- Microsoft and offline authentication
- Configuration loading (JSON + environment variables)
- Price parsing ($995, $5K, $9.9K, etc.)
- Discord webhook notifications
- Chat event handling
- AFK detection with Unicode support
- Sale notifications
- Error handling

### ⚠️ JavaScript Only (For Now):
- Auction house window interaction
- Map purchasing automation
- Map listing/selling automation
- Inventory management
- Complete main loop

## Running Both Versions

You can have both versions installed and switch between them:

### JavaScript Version:
```bash
npm install
npm start
```

### Rust Version:
```bash
cargo build --release
./target/release/donutsmp-mapflipper
```

## Code Architecture Differences

### JavaScript (Mineflayer):
```javascript
// Event-driven with callbacks
bot.on('windowOpen', async (window) => {
  // Handle window
});

// Promises with async/await
const result = await buyMap(window, slot, price);
```

### Rust (Azalea):
```rust
// Type-safe event matching
async fn handle_event(bot: Client, event: Event, state: State) -> Result<()> {
    match event {
        Event::Chat(m) => { /* Handle chat */ },
        _ => {}
    }
    Ok(())
}

// Explicit error handling with Result types
let result = buy_map(window, slot, price).await?;
```

## Performance Benchmarks

*Note: Benchmarks will be added once full functionality is implemented*

Expected improvements with Rust version:
- **Startup time**: ~50% faster
- **Memory usage**: ~60% lower
- **CPU usage**: ~40% lower
- **Response time**: ~30% faster

## Migration Checklist

If migrating from JavaScript to Rust:

- [ ] Install Rust nightly toolchain
- [ ] Your config.json works as-is (no changes needed)
- [ ] Rebuild authentication if using Microsoft (tokens are separate)
- [ ] Test bot connection and basic functionality
- [ ] Wait for full auction house implementation (or contribute!)

## Contributing to Rust Version

The Rust port is open for contributions! The main area needing work is:

**Window/Inventory Interaction**: Implementing auction house menu interaction using Azalea's inventory API. This requires:
- Understanding Azalea's Menu/Inventory system
- Handling container events
- Implementing slot clicking with proper state tracking
- Porting item lore parsing for Minecraft 1.21.1 components

See the original JavaScript implementation in `bot.js` for reference.

## Need Help?

- **JavaScript version**: Original README.md
- **Rust version**: README-RUST.md
- **Issues**: GitHub Issues page
- **Rust resources**: [Rust Book](https://doc.rust-lang.org/book/), [Azalea Docs](https://docs.rs/azalea/)

## Future Plans

### Short-term (Rust version):
1. Complete auction house window interaction
2. Port map purchasing logic
3. Port listing/selling functionality
4. Add comprehensive testing
5. Performance benchmarking

### Long-term:
- Consider making Rust version the primary implementation
- Maintain JavaScript version for backwards compatibility
- Add more features to both versions
- Community plugins/extensions

## Conclusion

Both versions serve different needs:
- **JavaScript/Mineflayer**: Production-ready, easy to modify
- **Rust/Azalea**: High-performance, type-safe, future-focused

Choose based on your requirements and comfort level with each language!
