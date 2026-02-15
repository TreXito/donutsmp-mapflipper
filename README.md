# DonutSMP Map Flipper Bot

A Mineflayer bot that automates buying cheap maps from the auction house on donutsmp.net and relisting them at a higher price.

## Features

- ✅ Automatic auction house monitoring
- ✅ Smart price parsing (handles $995, $5K, $9.9K formats)
- ✅ Anti-AFK detection and handling
- ✅ Auto-reconnect on disconnect
- ✅ Map unstacking for bulk listings
- ✅ Configurable buy/sell prices

## Requirements

- Node.js (v16 or higher)
- Minecraft account credentials

## Installation

```bash
npm install
```

## Configuration

The bot can be configured in two ways:

### Option 1: Using config.json (Recommended)

1. Copy `config.example.json` to `config.json`:
   ```bash
   cp config.example.json config.json
   ```

2. Edit `config.json` with your settings:
   ```json
   {
     "host": "donutsmp.net",
     "port": 25565,
     "username": "YourMinecraftUsername",
     "version": "1.21.11",
     "maxBuyPrice": 5000,
     "sellPrice": "9.9k",
     "delayBetweenCycles": 5000,
     "delayAfterJoin": 5000
   }
   ```

### Option 2: Using Environment Variables

Set environment variables to override defaults:
- `BOT_USERNAME`: Your Minecraft username
- `MAX_BUY_PRICE`: Maximum price to buy maps (default: 5000)
- `SELL_PRICE`: Price to list maps at (default: 9.9k)
- `DELAY_BETWEEN_CYCLES`: Wait time between auction checks in ms (default: 5000)
- `DELAY_AFTER_JOIN`: Wait time after spawning before starting (default: 5000)

**Note:** config.json settings take priority over environment variables.

### Configuration Options

- `host`: Server address (default: donutsmp.net)
- `port`: Server port (default: 25565)
- `username`: Your Minecraft username (required)
- `version`: Minecraft version (1.21.11)
- `maxBuyPrice`: Maximum price to buy maps (default: $5000)
- `sellPrice`: Price to list maps at (default: 9.9k)
- `delayBetweenCycles`: Wait time between auction checks in ms (default: 5000)
- `delayAfterJoin`: Wait time after spawning before starting (default: 5000)

## Usage

### Using config.json (Recommended):
```bash
# 1. Create and edit your config file
cp config.example.json config.json
# Edit config.json with your Minecraft username

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