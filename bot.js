const mineflayer = require('mineflayer');
const fs = require('fs');
const path = require('path');

// Load configuration from config.json if it exists
let fileConfig = {};
const configPath = path.join(__dirname, 'config.json');
if (fs.existsSync(configPath)) {
  try {
    const configData = fs.readFileSync(configPath, 'utf8');
    fileConfig = JSON.parse(configData);
    console.log('[CONFIG] Loaded configuration from config.json');
  } catch (error) {
    console.error('[CONFIG] Error loading config.json:', error.message);
    console.log('[CONFIG] Falling back to environment variables and defaults');
  }
} else {
  console.log('[CONFIG] No config.json found, using environment variables and defaults');
}

const CONFIG = {
  host: fileConfig.host || 'donutsmp.net',
  port: fileConfig.port || 25565,
  username: fileConfig.username || process.env.BOT_USERNAME || 'BOT_USERNAME',
  auth: fileConfig.auth || process.env.BOT_AUTH || 'microsoft',
  version: fileConfig.version || '1.21.11',
  maxBuyPrice: fileConfig.maxBuyPrice || parseInt(process.env.MAX_BUY_PRICE) || 5000,
  sellPrice: fileConfig.sellPrice || process.env.SELL_PRICE || '9.9k',
  delayBetweenCycles: fileConfig.delayBetweenCycles || parseInt(process.env.DELAY_BETWEEN_CYCLES) || 5000,
  delayAfterJoin: fileConfig.delayAfterJoin || parseInt(process.env.DELAY_AFTER_JOIN) || 5000,
};

// Constants
const HOTBAR_START_SLOT = 36;
const HOTBAR_END_SLOT = 44;
const WARN_MAP_COUNT_THRESHOLD = 5;

let bot;
let isAfkDetected = false;
let isRunning = false;

// Small caps to ASCII mapping for AFK detection
const SMALL_CAPS_MAP = {
  'ᴀ': 'a', 'ʙ': 'b', 'ᴄ': 'c', 'ᴅ': 'd', 'ᴇ': 'e', 'ꜰ': 'f', 'ɢ': 'g', 'ʜ': 'h',
  'ɪ': 'i', 'ᴊ': 'j', 'ᴋ': 'k', 'ʟ': 'l', 'ᴍ': 'm', 'ɴ': 'n', 'ᴏ': 'o', 'ᴘ': 'p',
  'ʀ': 'r', 'ꜱ': 's', 'ᴛ': 't', 'ᴜ': 'u', 'ᴠ': 'v', 'ᴡ': 'w', 'ʏ': 'y', 'ᴢ': 'z'
};

function normalizeText(text) {
  // Strip Minecraft formatting codes
  let normalized = text.replace(/§[0-9a-fk-or]/gi, '');
  
  // Convert small caps to ASCII
  for (const [smallCap, ascii] of Object.entries(SMALL_CAPS_MAP)) {
    normalized = normalized.replace(new RegExp(smallCap, 'g'), ascii);
  }
  
  return normalized.toLowerCase();
}

function parsePrice(loreString) {
  const clean = loreString.replace(/§[0-9a-fk-or]/gi, '');
  const match = clean.match(/Price:\s*\$([0-9,.]+)(K?)/i);
  if (!match) return null;
  
  // Remove commas (thousands separators)
  let price = parseFloat(match[1].replace(/,/g, ''));
  
  // Apply K multiplier if present
  if (match[2] && match[2].toUpperCase() === 'K') {
    price *= 1000;
  }
  
  return price;
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function handleAfkDetection() {
  // Prevent multiple simultaneous AFK handling
  if (isAfkDetected) {
    console.log('[AFK] Already handling AFK detection');
    return;
  }
  
  console.log('[AFK] Detected AFK teleport, returning to hub...');
  isAfkDetected = true;
  isRunning = false;
  
  try {
    // Send /hub command
    bot.chat('/hub');
    await sleep(1000);
    
    // Wait for hub selection window
    await new Promise((resolve) => {
      const timeout = setTimeout(resolve, 3000);
      bot.once('windowOpen', async (window) => {
        clearTimeout(timeout);
        console.log('[AFK] Hub window opened, selecting slot 5');
        await sleep(200);
        bot.clickWindow(5, 0, 0);
        await sleep(500);
        resolve();
      });
    });
    
    // Stand still for 10 seconds
    console.log('[AFK] Waiting 10 seconds before resuming...');
    await sleep(10000);
    
    isAfkDetected = false;
    console.log('[AFK] Resuming main loop');
    
    // Resume main loop
    setImmediate(() => mainLoop());
  } catch (error) {
    console.error('[AFK] Error handling AFK:', error);
    isAfkDetected = false;
  }
}

async function openAuctionHouse() {
  console.log('[AH] Opening auction house...');
  bot.chat('/ah map');
  
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      reject(new Error('Timeout waiting for auction house window'));
    }, 5000);
    
    bot.once('windowOpen', (window) => {
      clearTimeout(timeout);
      console.log(`[AH] Auction house opened, ${window.slots.length} slots`);
      resolve(window);
    });
  });
}

function findCheapMap(window) {
  for (let slot = 0; slot < window.slots.length; slot++) {
    const item = window.slots[slot];
    if (!item) continue;
    
    // Check if item has lore with price information
    try {
      const nbt = item.nbt;
      if (!nbt || !nbt.value) continue;
      
      // Try to access display data
      let loreArray = null;
      
      if (nbt.value.display && nbt.value.display.value && nbt.value.display.value.Lore) {
        const loreData = nbt.value.display.value.Lore;
        if (loreData.value && loreData.value.value) {
          loreArray = loreData.value.value;
        } else if (loreData.value) {
          loreArray = loreData.value;
        }
      }
      
      if (!loreArray || !Array.isArray(loreArray)) continue;
      
      // Parse lore lines
      for (const loreLine of loreArray) {
        let lineText = '';
        if (typeof loreLine === 'string') {
          lineText = loreLine;
        } else if (loreLine && typeof loreLine.toString === 'function') {
          lineText = loreLine.toString();
        }
        
        if (lineText.includes('Price:')) {
          const price = parsePrice(lineText);
          if (price !== null && price < CONFIG.maxBuyPrice) {
            console.log(`[AH] Found cheap map at slot ${slot}: $${price}`);
            return { slot, price };
          }
        }
      }
    } catch (error) {
      // Skip items with parsing errors
      continue;
    }
  }
  
  return null;
}

async function buyMap(window, mapSlot) {
  console.log(`[AH] Attempting to buy map at slot ${mapSlot}...`);
  
  try {
    // Click the map slot
    await bot.clickWindow(mapSlot, 0, 0);
    await sleep(500);
    
    // Click confirm button at slot 15
    console.log('[AH] Clicking confirm button...');
    await bot.clickWindow(15, 0, 0);
    await sleep(1000);
    
    // Wait to see if "already bought" message appears
    return new Promise((resolve) => {
      const timeout = setTimeout(() => {
        resolve(true); // Assume success if no error message
      }, 2000);
      
      const messageHandler = (msg) => {
        const normalized = normalizeText(msg);
        if (normalized.includes('already bought')) {
          clearTimeout(timeout);
          bot.off('messagestr', messageHandler);
          console.log('[AH] Item was already bought, retrying...');
          resolve(false);
        }
      };
      
      bot.on('messagestr', messageHandler);
    });
  } catch (error) {
    console.error('[AH] Error buying map:', error);
    return false;
  }
}

async function unstackMaps() {
  console.log('[INVENTORY] Checking for stacked maps...');
  
  // Open inventory if not already open
  if (!bot.currentWindow) {
    bot.openInventory();
    await sleep(500);
  }
  
  const window = bot.currentWindow;
  if (!window) {
    console.log('[INVENTORY] Could not open inventory');
    return;
  }
  
  // Find map stacks
  for (let slot = 0; slot < window.slots.length; slot++) {
    const item = window.slots[slot];
    if (item && item.name && item.name.includes('map') && item.count > 1) {
      console.log(`[INVENTORY] Found map stack of ${item.count} at slot ${slot}`);
      
      // Unstack to hotbar slots (HOTBAR_START_SLOT-HOTBAR_END_SLOT in inventory window)
      for (let i = 0; i < item.count - 1; i++) {
        const hotbarSlot = HOTBAR_START_SLOT + i;
        if (hotbarSlot <= HOTBAR_END_SLOT) {
          // Right-click to pick up one
          await bot.clickWindow(slot, 1, 0);
          await sleep(100);
          // Click hotbar slot to place
          await bot.clickWindow(hotbarSlot, 0, 0);
          await sleep(100);
        }
      }
    }
  }
  
  bot.closeWindow(window);
  await sleep(200);
}

async function listMaps() {
  console.log('[LISTING] Listing purchased maps...');
  
  // Find maps in hotbar (slots HOTBAR_START_SLOT-HOTBAR_END_SLOT correspond to hotbar 0-8)
  const inventory = bot.inventory;
  const maps = [];
  
  for (let i = 0; i < 9; i++) {
    const item = inventory.slots[HOTBAR_START_SLOT + i];
    if (item && item.name && item.name.includes('map')) {
      maps.push(i);
    }
  }
  
  console.log(`[LISTING] Found ${maps.length} map(s) in hotbar`);
  
  // Check if we might hit the 27 slot limit
  // This is a rough estimate - ideally we'd query actual AH slots
  if (maps.length > WARN_MAP_COUNT_THRESHOLD) {
    console.log('[LISTING] Warning: Listing many maps at once - may hit slot limit');
  }
  
  for (const hotbarSlot of maps) {
    console.log(`[LISTING] Listing map from hotbar slot ${hotbarSlot}...`);
    bot.setQuickBarSlot(hotbarSlot);
    await sleep(200);
    bot.chat(`/ah sell ${CONFIG.sellPrice}`);
    await sleep(500);
  }
  
  console.log('[LISTING] All maps listed successfully');
}

async function mainLoop() {
  if (!bot || isAfkDetected || isRunning) return;
  
  isRunning = true;
  
  try {
    // Open auction house
    const window = await openAuctionHouse();
    
    // Find cheap map
    let cheapMap = findCheapMap(window);
    
    if (!cheapMap) {
      console.log('[AH] No cheap maps found, waiting before retry...');
      try {
        if (bot.currentWindow) {
          bot.closeWindow(bot.currentWindow);
        }
      } catch (e) {
        console.log('[AH] Window already closed');
      }
      await sleep(CONFIG.delayBetweenCycles);
      isRunning = false;
      setImmediate(() => mainLoop());
      return;
    }
    
    // Attempt to buy the map
    let purchased = false;
    let attempts = 0;
    const maxAttempts = 5;
    
    while (!purchased && attempts < maxAttempts) {
      purchased = await buyMap(window, cheapMap.slot);
      attempts++;
      
      if (!purchased) {
        // Try to find another cheap map in the refreshed window
        await sleep(500);
        const nextMap = findCheapMap(window);
        if (!nextMap) {
          console.log('[AH] No more cheap maps available');
          break;
        }
        // Reassign the entire cheapMap object
        cheapMap = nextMap;
      }
    }
    
    if (purchased) {
      console.log('[AH] Successfully purchased map!');
      
      // Close auction house safely
      try {
        if (bot.currentWindow) {
          bot.closeWindow(bot.currentWindow);
        }
      } catch (e) {
        console.log('[AH] Window already closed');
      }
      await sleep(500);
      
      // Unstack if needed
      await unstackMaps();
      
      // List the maps
      await listMaps();
      
      // Small delay before next cycle
      await sleep(1000);
    } else {
      console.log('[AH] Failed to purchase map after retries');
      try {
        if (bot.currentWindow) {
          bot.closeWindow(bot.currentWindow);
        }
      } catch (e) {
        console.log('[AH] Window already closed');
      }
      await sleep(CONFIG.delayBetweenCycles);
    }
    
    isRunning = false;
    
    // Continue the loop
    if (!isAfkDetected) {
      setImmediate(() => mainLoop());
    }
  } catch (error) {
    console.error('[ERROR] Error in main loop:', error);
    isRunning = false;
    
    // Retry after delay
    await sleep(CONFIG.delayBetweenCycles);
    if (!isAfkDetected) {
      setImmediate(() => mainLoop());
    }
  }
}

function createBot() {
  console.log('[BOT] Creating bot...');
  
  const botOptions = {
    host: CONFIG.host,
    port: CONFIG.port,
    username: CONFIG.username,
    version: CONFIG.version,
  };
  
  // Add auth if specified (microsoft or offline)
  if (CONFIG.auth && CONFIG.auth.toLowerCase() === 'microsoft') {
    botOptions.auth = 'microsoft';
    console.log('[BOT] Using microsoft authentication');
  } else if (!CONFIG.auth || CONFIG.auth.toLowerCase() === 'offline') {
    console.log('[BOT] Using offline/cracked authentication');
  } else {
    console.warn(`[BOT] Unknown auth type '${CONFIG.auth}', falling back to offline authentication`);
  }
  
  bot = mineflayer.createBot(botOptions);
  
  bot.on('login', () => {
    console.log(`[BOT] Logged in as ${bot.username}`);
  });
  
  bot.on('spawn', async () => {
    console.log('[BOT] Spawned in game');
    console.log(`[BOT] Waiting ${CONFIG.delayAfterJoin}ms before starting...`);
    await sleep(CONFIG.delayAfterJoin);
    
    if (!isRunning && !isAfkDetected) {
      console.log('[BOT] Starting main loop');
      mainLoop();
    }
  });
  
  bot.on('messagestr', (msg) => {
    console.log(`[CHAT] ${msg}`);
    
    // Check for AFK detection
    const normalized = normalizeText(msg);
    if (normalized.includes('teleported to') && normalized.includes('afk')) {
      handleAfkDetection();
    }
  });
  
  bot.on('kicked', (reason) => {
    console.log(`[BOT] Kicked: ${reason}`);
    reconnect();
  });
  
  bot.on('end', () => {
    console.log('[BOT] Connection ended');
    reconnect();
  });
  
  bot.on('error', (err) => {
    console.error('[BOT] Error:', err);
  });
}

function reconnect() {
  console.log('[BOT] Reconnecting in 30 seconds...');
  isRunning = false;
  isAfkDetected = false;
  setTimeout(() => {
    createBot();
  }, 30000);
}

// Start the bot
console.log('[STARTUP] DonutSMP Map Flipper Bot');
console.log('[STARTUP] Configuration:', CONFIG);
createBot();
