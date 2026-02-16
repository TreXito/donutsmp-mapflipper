# Quick Start Guide - Get The Bot Working NOW

## Step 1: Choose Your Version

### ⚡ FASTEST: Use JavaScript Version (Recommended)

The JavaScript version is **fully functional** and ready to use.

```bash
# Install dependencies (first time only)
npm install

# Edit config.json with your settings
nano config.json
# or
code config.json

# Run the bot
npm start
```

That's it! The bot should connect and start working.

---

## Step 2: Configure

Edit `config.json`:

```json
{
  "host": "donutsmp.net",
  "port": 25565,
  "username": "your-email@example.com",  ← Your Microsoft email
  "auth": "microsoft",                    ← Keep as "microsoft"
  "version": "1.21.11",
  "maxBuyPrice": 2500,                    ← Max price to buy maps
  "sellPrice": "9.9k",                    ← Price to list maps
  "delayBetweenCycles": 5000,
  "delayAfterJoin": 5000,
  "webhook": {
    "enabled": false                       ← Set to true for Discord notifications
  }
}
```

**Important**: Use your Microsoft account email for `username` if the server requires online mode.

---

## Step 3: First Run

```bash
npm start
```

**What you'll see**:

1. **First time with Microsoft auth**: The bot will display:
   ```
   [AUTH] Please follow the prompts to authenticate...
   Go to: https://www.microsoft.com/link
   Enter code: ABC12345
   ```
   - Open that URL in your browser
   - Enter the code shown
   - Sign in with your Microsoft account
   - Tokens are cached, so you only do this once

2. **Bot connects**:
   ```
   [BOT] Connecting to donutsmp.net:25565...
   [BOT] Logged in to server
   [BOT] Spawned in game
   [BOT] Starting main loop
   ```

3. **Bot starts working**:
   ```
   [CYCLE] Starting new cycle
   [AH] Opening auction house...
   [AH] Scanning 54 container slots...
   [AH] ✓ Found cheap map at slot 15: $1995
   [AH] Attempting to buy map...
   [AH] Purchase successful!
   ```

---

## Common Issues

### "Cannot find module 'mineflayer'"
```bash
npm install
```

### "Invalid credentials" or "Failed to authenticate"
- Make sure you're using your Microsoft email
- Try removing cached auth: `rm -rf ~/.minecraft/nmp-cache`
- Try again: `npm start`

### "Connection refused"
- Check your internet connection
- Verify server is online: `donutsmp.net`
- Check config.json has correct host/port

### "Timeout waiting for auction house window"
- Server might be slow
- Edit config.json: `"windowTimeout": 20000` (increase to 20 seconds)

---

## What About The Rust Version?

**The Rust version is incomplete** and can't buy/sell maps yet.

- ✅ Core infrastructure works (connection, auth, chat)
- ❌ Main functionality missing (auction house interaction)

**Use JavaScript version** unless you're a Rust developer wanting to contribute to completing it.

---

## Verify It's Working

1. **Bot connects** - You should see login messages
2. **Bot spawns** - You should see "Spawned in game"
3. **Bot opens /ah map** - Check in-game if bot is opening auction house
4. **Bot finds maps** - Watch console for "Found cheap map" messages
5. **Bot buys maps** - Watch for "Purchase successful" messages
6. **Bot lists maps** - Watch for "Listing map" messages

---

## Enable Discord Notifications (Optional)

1. Create a Discord webhook:
   - Server Settings → Integrations → Webhooks
   - Click "New Webhook"
   - Copy the webhook URL

2. Edit `config.json`:
   ```json
   {
     "webhook": {
       "enabled": true,
       "url": "https://discord.com/api/webhooks/YOUR_WEBHOOK_URL",
       "displayName": "Map Flipper Bot",
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

3. Restart the bot: `npm start`

4. **Verify it's working** - Check logs on startup:
   ```
   [CONFIG] Webhook notifications: ENABLED
   [CONFIG] Webhook URL: https://discord.com/api/webhooks/...
   ```

You'll now get Discord notifications for all bot events!

**Note**: Only `enabled` and `url` are required. Other fields use smart defaults if omitted.

---

## Still Not Working?

**Check these**:

1. Node.js version:
   ```bash
   node --version
   # Should be v16 or higher
   ```

2. Dependencies installed:
   ```bash
   ls node_modules/
   # Should see mineflayer, prismarine-auth, etc.
   ```

3. Config file valid:
   ```bash
   cat config.json | jq .
   # Should show your config
   ```

4. Bot.js exists:
   ```bash
   ls -l bot.js
   # Should show bot.js file
   ```

**Share these details if still stuck**:
- Error messages from console
- Output of `node --version`
- Output of `npm list`
- Contents of config.json (remove your email)

---

## Summary

✅ **JavaScript version** = Fully functional, use this  
❌ **Rust version** = Incomplete, don't use yet (unless developing)

**To get started RIGHT NOW**:
```bash
npm install
# Edit config.json with your email
npm start
```

That's it!
