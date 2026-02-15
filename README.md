# DonutSMP Map Flipper Bot

A Minecraft bot that automates buying cheap maps from the auction house on donutsmp.net and relisting them at a higher price.

## 🚀 Quick Start

**Want to get started immediately?** → [QUICKSTART.md](QUICKSTART.md)

**Wondering what works?** → [STATUS.md](STATUS.md)

**Want to complete Rust version?** → [COMPLETION-SUMMARY.md](COMPLETION-SUMMARY.md) ⭐ NEW

**Check your system:** Run `./check-system.sh`

---

## 🎯 Available Versions

This project is available in two implementations:

### 🦀 **Rust/Azalea Version (RECOMMENDED)**
- **Status**: ✅ **PRODUCTION READY** - Pre-built releases available!
- **Performance**: Fast, efficient, and resource-light
- **No dependencies**: Statically linked binaries
- **Get started**: Download from [Releases](https://github.com/TreXito/donutsmp-mapflipper/releases)

### 📦 **JavaScript/Mineflayer Version**
- **Status**: ✅ **FULLY FUNCTIONAL** - Available in source
- **Mature implementation** with all features working
- **Easy to modify** and understand
- **Node.js based** - cross-platform
- **Get started**: Clone repository and run with Node.js

---

## JavaScript/Mineflayer Version hgi

A Mineflayer bot for flipping maps on DonutSMP auction house.

## Features

- ✅ Automatic auction house monitoring
- ✅ Smart price parsing (handles $995, $5K, $9.9K formats)
- ✅ Anti-AFK detection and handling
- ✅ Auto-reconnect on disconnect
- ✅ Map unstacking for bulk listings
- ✅ Configurable buy/sell prices
- ✅ Microsoft authentication support
- ✅ Discord webhook notifications for purchases, sales, and events
- ✅ Partial packet error suppression
- ✅ Automated Rust releases for Windows, Linux, and macOS (x86_64 and ARM64)

## Requirements

- Node.js (v16 or higher)
- Minecraft Java Edition account (Microsoft account required for online servers)

## Quick Start

### Option 1: Download Pre-built Release (Recommended)

1. Go to the [Releases](https://github.com/TreXito/donutsmp-mapflipper/releases) page
2. Download the latest release for your operating system:
   - **Rust Version (Production Ready)**:
     - **Linux (x86_64)**: `donutsmp-mapflipper-rust-vX.X.X-linux-x86_64.tar.gz`
     - **Windows (x86_64)**: `donutsmp-mapflipper-rust-vX.X.X-windows-x86_64.zip`
     - **macOS (Intel)**: `donutsmp-mapflipper-rust-vX.X.X-macos-x86_64.tar.gz`
     - **macOS (Apple Silicon)**: `donutsmp-mapflipper-rust-vX.X.X-macos-aarch64.tar.gz`
3. Extract the archive
4. Edit `config.json` with your settings (Microsoft email for auth)
5. Run the bot:
   - **Linux/Mac**: `./donutsmp-mapflipper`
   - **Windows**: `donutsmp-mapflipper.exe`

### Option 2: Run Node.js Version from Source

The Node.js version is still available in the repository for development:

```bash
git clone https://github.com/TreXito/donutsmp-mapflipper.git
cd donutsmp-mapflipper
npm install
npm start
```

### Option 3: Build Rust Version from Source

```bash
git clone https://github.com/TreXito/donutsmp-mapflipper.git
cd donutsmp-mapflipper
cargo build --release
./target/release/donutsmp-mapflipper
```

## Installation

```bash
npm install
```

## Configuration

A default `config.json` file is included in the repository. Simply edit it with your settings:

> **⚠️ Important:** The `config.json` file is tracked in git. If you plan to commit changes, be careful not to commit your actual credentials. Consider keeping your credentials in environment variables instead, or be mindful when committing.

1. Edit `config.json` with your settings:
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

### Alternative: Using Environment Variables

Set environment variables to override defaults:
- `BOT_USERNAME`: Your Minecraft email/username
- `BOT_AUTH`: Authentication method ('microsoft' or 'offline', default: 'microsoft')
- `MAX_BUY_PRICE`: Maximum price to buy maps (default: 2500)
- `SELL_PRICE`: Price to list maps at (default: 9.9k)
- `DELAY_BETWEEN_CYCLES`: Wait time between auction checks in ms (default: 5000)
- `DELAY_AFTER_JOIN`: Wait time after spawning before starting (default: 5000)

**Note:** config.json settings take priority over environment variables.

### Configuration Options

- `host`: Server address (default: donutsmp.net)
- `port`: Server port (default: 25565)
- `username`: Your Minecraft email (for Microsoft auth) or username (for offline)
- `auth`: Authentication method - `'microsoft'` for Microsoft accounts (default), `'offline'` for cracked servers
- `version`: Minecraft version (1.21.11)
- `maxBuyPrice`: Maximum price to buy maps (default: $2500)
- `sellPrice`: Price to list maps at (default: 9.9k)
- `delayBetweenCycles`: Wait time between auction checks in ms (default: 5000)
- `delayAfterJoin`: Wait time after spawning before starting (default: 5000)
- `windowTimeout`: Timeout for window opening operations in ms (default: 15000)
- `debugEvents`: Enable event debugging to diagnose window opening issues (default: false) - **Warning: Only use for debugging**
- `webhook`: Webhook configuration for Discord notifications
  - `enabled`: Enable webhook notifications (default: false)
  - `url`: Discord webhook URL
  - `displayName`: Bot display name in Discord (default: 'DonutSMP Map Flipper')
  - `events`: Control which events trigger notifications
    - `purchase`: Notify when bot buys a map
    - `listing`: Notify when bot lists maps for sale
    - `sale`: Notify when someone buys from bot
    - `afk`: Notify when AFK is detected
    - `error`: Notify on errors and kicks
    - `startup`: Notify when bot connects

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

3. Customize which events you want to be notified about by setting them to `true` or `false`

### Webhook Events

- **purchase**: Notifies when the bot buys a map, includes price and seller
- **listing**: Notifies when the bot lists maps for sale
- **sale**: Notifies when someone buys a map from the bot
- **afk**: Notifies when AFK detection triggers
- **error**: Notifies on errors and when bot is kicked
- **startup**: Notifies when bot successfully connects to server

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

**Example config.json for Microsoft auth:**
```json
{
  "username": "your-email@example.com",
  "auth": "microsoft"
}
```

### Offline/Cracked Authentication

For offline/cracked servers that don't require authentication:

1. Set `"auth": "offline"` in config.json
2. Use any username you want

**Example config.json for offline auth:**
```json
{
  "username": "YourUsername",
  "auth": "offline"
}
```

## Usage

### Using config.json (Recommended):
```bash
# 1. Edit config.json with your Minecraft username and settings
# Enable webhooks if desired

# 2. Run the bot
npm start
```

### Using environment variable for username:
```bash
BOT_USERNAME=YourMinecraftUsername npm start
```

### Using the start script (Linux/Mac):
```bash
BOT_USERNAME=YourMinecraftUsername ./start.sh
```

## WSL (Windows Subsystem for Linux) Users

If you encounter errors like `UNC paths are not supported` when running from WSL, this is because Node.js on Windows doesn't support UNC paths (paths like `\\wsl.localhost\...`). Here are solutions:

**Option 1: Run from Linux side (Recommended)**
```bash
# Inside WSL terminal, navigate to your home directory
cd ~/donutsmp-mapflipper
npm start
```

**Option 2: Run from Windows side**
```cmd
# From Windows Command Prompt or PowerShell
cd C:\Users\YourUsername\path\to\donutsmp-mapflipper
npm start
```

**Option 3: Use the start.sh script from WSL**
```bash
./start.sh
```

## How It Works

1. **Connect to Server**: Bot joins donutsmp.net and waits 5 seconds after spawning
2. **Monitor Auction House**: Opens `/ah map` and scans for maps under $2,500
3. **Purchase Maps**: Buys one affordable map at a time
4. **Relist for Profit**: Immediately lists the purchased map for $9.9K
5. **Loop**: Continues buying and listing maps automatically
6. **Anti-AFK**: Detects AFK teleport messages (with unicode small caps support) and returns to hub automatically

## Anti-AFK Detection

The bot monitors chat for messages containing "teleported to" and "afk" (including unicode small caps like ᴀꜰᴋ). When detected:
- Stops current operations
- Sends `/hub` command
- Clicks slot 5 to select hub
- Waits 10 seconds
- Resumes the main loop

## Edge Cases Handled

- **Already Bought**: If an item is already purchased by someone else, the bot retries with the next available map
- **Connection Loss**: Auto-reconnects after 30 seconds
- **Map Stacking**: Automatically unstacks multiple maps to list individually
- **Race Conditions**: Handles purchase confirmation delays and window updates
- **Protocol Errors**: Prevents "Invalid sequence" kicks with proper timing and window management
- **Window Opening Issues**: Registers listeners before commands to avoid race conditions

## Troubleshooting

### Bot Can't Open Auction House Window

If the bot times out waiting for the auction house window to open:

1. **Increase the timeout**: Edit your config.json to increase `windowTimeout`:
   ```json
   {
     "windowTimeout": 20000
   }
   ```
   The default is 15 seconds (15000ms). Increase to 20 or 30 seconds if your server is slow.

2. **Enable debug events** to see what events are being triggered:
   ```json
   {
     "debugEvents": true
   }
   ```
   This will log all non-spam events to help diagnose the issue. **Warning: This is only for debugging and may interfere with bot operation.**

3. **Verify command format**: The bot sends `/ah map` - ensure this is the correct command for your server.

4. **Check for window conflicts**: The bot now automatically closes any existing windows before opening the auction house, but manual intervention may be needed if the bot gets stuck.

### "Invalid Sequence" Kick

The bot now includes several protections against this protocol error:
- Closes existing windows before opening new ones
- Waits 300ms after sending commands (client-side delay to prevent rapid sequential operations)
- Adds a minimum 3-second delay before retrying after errors
- Increased timeout to 15 seconds for slow servers

If you still experience this issue, try increasing `delayBetweenCycles` in your config.

### Chunk Size Warnings

The bot automatically suppresses harmless "Chunk size is X but only Y was read" messages. These are protocol parsing warnings that don't affect functionality.

## Technical Details

- Built with [Mineflayer](https://github.com/PrismarineJS/mineflayer)
- Compatible with Minecraft 1.21.11
- Uses Minecraft chat commands and GUI window interaction
- Parses item NBT data for price information

## Development & Releases

### Automated Releases

This project uses GitHub Actions to automatically create Rust releases for Windows, Linux, and macOS (both Intel and Apple Silicon) whenever a pull request is merged to the main branch. Each release includes:

- Pre-compiled Rust binaries (statically linked)
- Configuration file template
- Complete documentation

The Node.js version is still available in the repository for development and can be run from source.

### Creating a Manual Release

Repository maintainers can create a release manually:

1. Go to the **Actions** tab in GitHub
2. Select the **Rust Release** workflow
3. Click **Run workflow**
4. Enter the desired version tag (e.g., `v1.0.0`)
5. Click **Run workflow**

The workflow will build and publish Rust releases for all four platforms (Linux x86_64, Windows x86_64, macOS x86_64, and macOS ARM64) automatically.

## License

MIT
