# DonutSMP Map Flipper Bot

A Mineflayer bot that automates buying cheap maps from the auction house on donutsmp.net and relisting them at a higher price.

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

## Requirements

- Node.js (v16 or higher)
- Minecraft Java Edition account (Microsoft account required for online servers)

## Installation

```bash
npm install
```

## Configuration

The bot can be configured in two ways:

### Option 1: Using config.json (Recommended)

1. Copy the template to create your config:
   ```bash
   cp config.template.json config.json
   ```

2. Edit `config.json` with your settings:
   ```json
   {
     "host": "donutsmp.net",
     "port": 25565,
     "username": "your-email@example.com",
     "auth": "microsoft",
     "version": "1.21.11",
     "maxBuyPrice": 5000,
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

### Option 2: Using Environment Variables

Set environment variables to override defaults:
- `BOT_USERNAME`: Your Minecraft email/username
- `BOT_AUTH`: Authentication method ('microsoft' or 'offline', default: 'microsoft')
- `MAX_BUY_PRICE`: Maximum price to buy maps (default: 5000)
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
- `maxBuyPrice`: Maximum price to buy maps (default: $5000)
- `sellPrice`: Price to list maps at (default: 9.9k)
- `delayBetweenCycles`: Wait time between auction checks in ms (default: 5000)
- `delayAfterJoin`: Wait time after spawning before starting (default: 5000)
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
# 1. Copy template and edit with your settings
cp config.template.json config.json
# Edit config.json with your Minecraft username
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
2. **Monitor Auction House**: Opens `/ah map` and scans for maps under $5,000
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

## Technical Details

- Built with [Mineflayer](https://github.com/PrismarineJS/mineflayer)
- Compatible with Minecraft 1.21.11
- Uses Minecraft chat commands and GUI window interaction
- Parses item NBT data for price information

## License

MIT