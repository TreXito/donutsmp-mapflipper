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

The bot is configured through environment variables or by editing the CONFIG object in `bot.js`:

- `BOT_USERNAME`: Your Minecraft username (can be set via environment variable)
- `host`: Server address (default: donutsmp.net)
- `port`: Server port (default: 25565)
- `version`: Minecraft version (1.21.1)
- `maxBuyPrice`: Maximum price to buy maps (default: $5000)
- `sellPrice`: Price to list maps at (default: 9.9k)
- `delayBetweenCycles`: Wait time between auction checks in ms (default: 5000)
- `delayAfterJoin`: Wait time after spawning before starting (default: 5000)

## Usage

### Using environment variable for username:
```bash
BOT_USERNAME=YourMinecraftUsername node bot.js
```

### Or edit the CONFIG in bot.js and run:
```bash
npm start
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
- Compatible with Minecraft 1.21.1
- Uses Minecraft chat commands and GUI window interaction
- Parses item NBT data for price information

## License

MIT