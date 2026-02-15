# README FIRST - Answer to "Why doesn't everything work?"

## The Simple Answer

**The repository has TWO versions of the bot:**

1. **JavaScript version** ✅ **WORKS PERFECTLY** - Use this!
2. **Rust version** ❌ **INCOMPLETE** - Don't use yet (only ~40% done)

---

## Get The Bot Working NOW (5 minutes)

```bash
# Step 1: Install dependencies
npm install

# Step 2: Edit config.json with your Microsoft email
nano config.json
# Change "your-email@example.com" to your actual email

# Step 3: Run the bot
npm start
```

**That's it!** The JavaScript version is fully functional.

For detailed instructions, see: **[QUICKSTART.md](QUICKSTART.md)**

---

## Check What You Have

Run this to see what's installed and what's needed:

```bash
./check-system.sh
```

It will tell you:
- ✓ What's working
- ✗ What's missing  
- → What to do next

---

## Understanding The Two Versions

### JavaScript Version (Mineflayer)
- **Location**: `bot.js`, `package.json`
- **Status**: ✅ 100% functional
- **Features**: All working (connection, purchasing, listing, webhooks, AFK detection)
- **Use when**: You want the bot to work NOW
- **Setup**: `npm install && npm start`

### Rust Version (Azalea)
- **Location**: `src/`, `Cargo.toml`
- **Status**: ⚠️ ~40% complete
- **Working**: Connection, auth, config, webhooks, chat
- **Missing**: Auction house interaction, purchasing, listing
- **Use when**: You're a Rust developer wanting to contribute
- **Effort needed**: 12-15 hours of development

---

## Why Is Rust Incomplete?

The Rust version needs **auction house window interaction** implemented.

This requires:
- Azalea inventory/menu API knowledge
- Minecraft 1.21.1 protocol understanding
- Rust async/await expertise
- 12-15 hours of development time

**Details**: See [RUST-TODO.md](RUST-TODO.md) for complete implementation guide.

---

## Which Version Should I Use?

### Use JavaScript If:
- ✓ You want the bot to work **immediately**
- ✓ You don't know Rust
- ✓ You just want to flip maps
- ✓ You value stability over performance

**→ See [QUICKSTART.md](QUICKSTART.md)**

### Use Rust If:
- ✓ You're a Rust developer
- ✓ You want to contribute to development
- ✓ You're willing to spend 12-15 hours implementing features
- ✓ You understand Minecraft protocol

**→ See [RUST-TODO.md](RUST-TODO.md)**

### Not Sure?
**Use JavaScript.** It works perfectly.

---

## Quick Links

| Document | Purpose |
|----------|---------|
| **[QUICKSTART.md](QUICKSTART.md)** | Get JavaScript version working in 5 minutes |
| **[STATUS.md](STATUS.md)** | Detailed explanation of what works/doesn't |
| **[RUST-TODO.md](RUST-TODO.md)** | Complete guide to finishing Rust version |
| **[README.md](README.md)** | Full documentation for both versions |
| **[MIGRATION.md](MIGRATION.md)** | Comparison between JavaScript and Rust |
| **check-system.sh** | Automated system check script |

---

## Common Questions

### Q: Why doesn't the bot buy/sell maps?
**A:** You're probably trying to use the Rust version, which is incomplete. Use the JavaScript version instead.

### Q: How do I get it working?
**A:** Run `npm install && npm start` (see [QUICKSTART.md](QUICKSTART.md))

### Q: What's the Rust version for?
**A:** It's a partial rewrite for better performance, but it's not finished yet. Unless you're a Rust developer wanting to complete it, use JavaScript.

### Q: When will Rust version be complete?
**A:** When someone implements the auction house interaction (~12-15 hours of work). See [RUST-TODO.md](RUST-TODO.md) if you want to help.

### Q: Can I use both versions?
**A:** Yes, but only the JavaScript version actually works for flipping maps.

---

## TL;DR

```
Need bot to work? → Use JavaScript
Want to develop?  → Help finish Rust
Not sure?         → Use JavaScript
```

**JavaScript setup**: `npm install && npm start`

**For help**: Read [QUICKSTART.md](QUICKSTART.md)

---

## Still Confused?

Run this:
```bash
./check-system.sh
```

It will tell you exactly what you have and what to do next.

Or read: **[STATUS.md](STATUS.md)** for detailed explanation.
