# DonutSMP Map Flipper Bot (Rust/Azalea Edition)

A Minecraft bot written in Rust using Azalea that automates buying cheap maps from the auction house on donutsmp.net and relisting them at a higher price.

## Features

- ✅ Rust-based implementation using azalea-rs
- ✅ Automatic auction house monitoring
- ✅ Smart price parsing (handles $995, $5K, $9.9K formats)
- ✅ **AFK Farming** - Automatically teleports to random AFK location at startup to farm shards while flipping
- ✅ Anti-AFK detection and handling
- ✅ Auto-reconnect on disconnect (via Azalea)
- ✅ Configurable buy/sell prices
- ✅ Microsoft authentication support
- ✅ Discord webhook notifications for purchases, sales, and events
- ⚠️ **Note**: Full window/inventory interaction is still in development

## Requirements

- Rust nightly toolchain (required by Azalea)
- Minecraft Java Edition account (Microsoft account required for online servers)

## Quick Start

### Installation

1. **Install Rust nightly:**
   ```bash
   rustup install nightly
   rustup default nightly
   ```

2. **Clone the repository:**
   ```bash
   git clone https://github.com/TreXito/donutsmp-mapflipper.git
   cd donutsmp-mapflipper
   ```

3. **Build the project:**
   ```bash
   cargo build --release
   ```

## Configuration

A default `config.json` file is included in the repository. Simply edit it with your settings:

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
  "windowTimeout": 15000,
  "debugEvents": false,
  "enableAfkFarming": true,
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

### Configuration Options

- `host`: Server address (default: donutsmp.net)
- `port`: Server port (default: 25565)
- `username`: Your Minecraft email (for Microsoft auth) or username (for offline)
- `auth`: Authentication method - `"microsoft"` for Microsoft accounts (default), `"offline"` for cracked servers
- `version`: Minecraft version (1.21.11)
- `maxBuyPrice`: Maximum price to buy maps (default: $2500)
- `sellPrice`: Price to list maps at (default: 9.9k)
- `delayBetweenCycles`: Wait time between auction checks in ms (default: 5000)
- `delayAfterJoin`: Wait time after spawning before starting (default: 5000)
- `windowTimeout`: Timeout for window opening operations in ms (default: 15000)
- `debugEvents`: Enable event debugging (default: false)
- `enableAfkFarming`: Enable automatic AFK farming at startup (default: true)
- `webhook`: Webhook configuration for Discord notifications

### Alternative: Using Environment Variables

Set environment variables to override defaults:
- `BOT_USERNAME`: Your Minecraft email/username
- `BOT_AUTH`: Authentication method ('microsoft' or 'offline', default: 'microsoft')
- `MAX_BUY_PRICE`: Maximum price to buy maps (default: 2500)
- `SELL_PRICE`: Price to list maps at (default: 9.9k)
- `DELAY_BETWEEN_CYCLES`: Wait time between auction checks in ms (default: 5000)
- `DELAY_AFTER_JOIN`: Wait time after spawning before starting (default: 5000)

**Note:** config.json settings take priority over environment variables.

## Usage

### Run the bot:
```bash
cargo run --release
```

### Or using environment variables:
```bash
BOT_USERNAME=your-email@example.com cargo run --release
```

## Authentication

### Microsoft Authentication (Default)

For servers requiring online mode (like donutsmp.net), you need Microsoft authentication:

1. Set `"auth": "microsoft"` in config.json (this is the default)
2. Use your Microsoft account email as the username
3. **First-time setup**: When you first run the bot, it will display:
   - A code (e.g., `ABC12345`)
   - A URL: `https://www.microsoft.com/link`
4. Open the URL in your browser, enter the code, and sign in with your Microsoft account
5. The authentication tokens are cached, so you only need to do this once

### Offline/Cracked Authentication

For offline/cracked servers that don't require authentication:

1. Set `"auth": "offline"` in config.json
2. Use any username you want

## Webhook Setup (Optional)

To receive Discord notifications for bot events:

1. Create a Discord webhook:
   - Open your Discord server settings
   - Go to Integrations → Webhooks
   - Click "New Webhook"
   - Choose a channel and copy the webhook URL

2. Enable webhooks in `config.json`:
   ```json
   {
     "webhook": {
       "enabled": true,
       "url": "https://discord.com/api/webhooks/YOUR_WEBHOOK_URL",
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

### Webhook Troubleshooting

If webhooks aren't working:

1. **Check the logs on startup** - The bot will display webhook status:
   ```
   [CONFIG] Webhook notifications: ENABLED
   [CONFIG] Webhook URL: https://discord.com/api/webhooks/...
   [CONFIG] Webhook events: purchase, listing, sale, afk, error, startup
   ```

2. **Verify your webhook URL**:
   - Must start with `https://discord.com/api/webhooks/`
   - Copy the entire URL from Discord (it's long)
   - Don't include any extra spaces or characters

3. **Check event-specific logs**:
   - Each webhook attempt logs: `[WEBHOOK] Successfully sent {event} webhook`
   - Failures show: `[WEBHOOK] Failed to send webhook (status XXX): ...`

4. **Minimal working configuration**:
   ```json
   {
     "webhook": {
       "enabled": true,
       "url": "https://discord.com/api/webhooks/YOUR_WEBHOOK_URL"
     }
   }
   ```
   All other fields (displayName, events) are optional and will use defaults.

## Logging Configuration

The bot uses Rust's standard logging system via the `RUST_LOG` environment variable. By default, it shows all messages at `info` level and above.

### Suppress Azalea Protocol Warnings

You may see harmless warnings like "The Boolean value was not 0 or 1, but 145" from the Azalea protocol parser. These can be safely suppressed:

```bash
# Linux/macOS
RUST_LOG=warn,azalea_buf=error cargo run --release

# Windows (PowerShell)
$env:RUST_LOG="warn,azalea_buf=error"; cargo run --release

# Windows (CMD)
set RUST_LOG=warn,azalea_buf=error && cargo run --release
```

### Log Level Options

- `error` - Only errors
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debugging info
- `trace` - Very detailed tracing

### Examples

```bash
# Show only bot-specific logs at info level, suppress everything else
RUST_LOG=donutsmp_mapflipper=info cargo run --release

# Show all warnings, but suppress azalea_buf warnings completely
RUST_LOG=warn,azalea_buf=error cargo run --release

# Debug mode - show everything
RUST_LOG=debug cargo run --release
```

## How It Works

1. **Connect to Server**: Bot joins donutsmp.net and waits after spawning
2. **Monitor Auction House**: Opens `/ah map` and scans for maps under the configured price
3. **Purchase Maps**: Buys affordable maps one at a time
4. **Relist for Profit**: Lists purchased maps at the configured sell price
5. **Loop**: Continues buying and listing maps automatically
6. **Anti-AFK**: Detects AFK teleport messages and returns to hub automatically

## Anti-AFK Detection

The bot monitors chat for messages containing "teleported to" and "afk" (including unicode small caps like ᴀꜰᴋ). When detected:
- Stops current operations
- Sends `/hub` command
- Waits and resumes operations

## Technical Details

- Built with [Azalea](https://github.com/azalea-rs/azalea) - Rust library for Minecraft bots
- Compatible with Minecraft 1.21.11
- Uses async/await with Tokio runtime
- Efficient price parsing with regex
- Discord webhooks via reqwest HTTP client

## Development

### Building from source:
```bash
cargo build --release
```

### Running tests:
```bash
cargo test
```

### Running in debug mode:
```bash
cargo run
```

## Differences from JavaScript Version

This Rust port offers several improvements:
- **Type safety**: Compile-time guarantees prevent many runtime errors
- **Performance**: Rust's zero-cost abstractions and native compilation
- **Memory efficiency**: No garbage collection overhead
- **Modern async**: Built on Tokio for efficient async I/O
- **Better error handling**: Comprehensive Result types

## Limitations

⚠️ **Work in Progress**: The following features are partially implemented:
- Full window/inventory interaction for auction house
- Map purchasing and listing automation

The bot currently connects, authenticates, and handles basic chat/AFK detection, but the auction house interaction is still being developed.

## Porting Notes

This is a complete rewrite from the JavaScript/Mineflayer version to Rust/Azalea. The original functionality is being preserved while taking advantage of Rust's type system and performance characteristics.

### Migration from JavaScript Version

If you're coming from the JavaScript version:
1. Install Rust nightly instead of Node.js
2. The config.json format remains the same
3. Run `cargo run --release` instead of `npm start`
4. Build times are longer initially but the compiled binary is faster

## License

MIT
