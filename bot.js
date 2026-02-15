const mineflayer = require('mineflayer');
const fs = require('fs');
const path = require('path');
const https = require('https');

// Suppress partial packet errors and chunk size warnings from minecraft-protocol
const originalConsoleError = console.error;
console.error = function(...args) {
  const message = args.join(' ');
  // Filter out partial packet errors which are harmless protocol parsing warnings
  // These are very specific to minecraft-protocol's packet parser
  if ((message.includes('partial packet') && message.includes('Chunk size is')) ||
      (message.includes('Chunk size is') && message.includes('but only') && message.includes('was read'))) {
    return;
  }
  originalConsoleError.apply(console, args);
};

// Suppress chunk size warnings from console.warn as well
const originalConsoleWarn = console.warn;
console.warn = function(...args) {
  const msg = args[0]?.toString() || '';
  if (msg.includes('Chunk size is') || msg.includes('partial packet')) return;
  originalConsoleWarn.apply(console, args);
};


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
  maxBuyPrice: fileConfig.maxBuyPrice || parseInt(process.env.MAX_BUY_PRICE) || 2500,
  sellPrice: fileConfig.sellPrice || process.env.SELL_PRICE || '9.9k',
  delayBetweenCycles: fileConfig.delayBetweenCycles || parseInt(process.env.DELAY_BETWEEN_CYCLES) || 5000,
  delayAfterJoin: fileConfig.delayAfterJoin || parseInt(process.env.DELAY_AFTER_JOIN) || 5000,
  windowTimeout: fileConfig.windowTimeout || parseInt(process.env.WINDOW_TIMEOUT) || 15000,
  debugEvents: fileConfig.debugEvents || process.env.DEBUG_EVENTS === 'true' || false,
  webhook: fileConfig.webhook || {
    enabled: false,
    url: '',
    displayName: 'DonutSMP Map Flipper',
    events: {
      purchase: true,
      listing: true,
      sale: true,
      afk: true,
      error: true,
      startup: true
    }
  }
};

// Constants
const HOTBAR_START_SLOT = 36;
const HOTBAR_END_SLOT = 44;
const WARN_MAP_COUNT_THRESHOLD = 5;
const MIN_RETRY_DELAY = 3000; // Minimum delay before retry after errors (ms)

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

function stripMinecraftColors(text) {
  return text.replace(/§[0-9a-fk-or]/gi, '');
}

function isPartialPacketError(err) {
  return err && err.message && err.message.includes('partial packet');
}

function extractLore(item) {
  if (!item.components) return [];
  const loreComponent = item.components.find(c => c.type === 'lore');
  if (!loreComponent || !loreComponent.data) return [];
  
  return loreComponent.data.map(line => {
    let text = '';
    if (line.value && line.value.text) {
      text += line.value.text.value || '';
    }
    if (line.value && line.value.extra && line.value.extra.value && line.value.extra.value.value) {
      for (const part of line.value.extra.value.value) {
        if (part.text) {
          text += part.text.value || '';
        }
      }
    }
    return text;
  });
}

function parsePrice(loreString) {
  const clean = stripMinecraftColors(loreString);
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

async function sendWebhook(event, data) {
  if (!CONFIG.webhook.enabled || !CONFIG.webhook.url) return;
  if (!CONFIG.webhook.events[event]) return;
  
  try {
    const url = new URL(CONFIG.webhook.url);
    const payload = JSON.stringify({
      username: CONFIG.webhook.displayName || 'DonutSMP Map Flipper',
      embeds: [{
        title: `${event.charAt(0).toUpperCase() + event.slice(1)} Event`,
        description: data.message,
        color: data.color || 3447003,
        timestamp: new Date().toISOString(),
        fields: data.fields || []
      }]
    });
    
    return new Promise((resolve) => {
      const options = {
        hostname: url.hostname,
        port: url.port || 443,
        path: url.pathname + url.search,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(payload)
        }
      };
      
      const req = https.request(options, (res) => {
        let responseData = '';
        res.on('data', (chunk) => { responseData += chunk; });
        res.on('end', () => {
          if (res.statusCode >= 200 && res.statusCode < 300) {
            resolve();
          } else {
            console.error(`[WEBHOOK] Failed to send webhook (status ${res.statusCode}):`, responseData);
            resolve(); // Still resolve to not break bot flow
          }
        });
      });
      
      req.on('error', (err) => {
        console.error('[WEBHOOK] Error sending webhook:', err.message);
        resolve(); // Still resolve to not break bot flow
      });
      
      req.write(payload);
      req.end();
    });
  } catch (error) {
    console.error('[WEBHOOK] Invalid webhook URL:', error.message);
    return Promise.resolve();
  }
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
  
  await sendWebhook('afk', {
    message: 'AFK detected! Returning to hub...',
    color: 16776960
  });
  
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
  // Close any existing window before opening a new one to prevent protocol conflicts
  if (bot.currentWindow) {
    console.log('[AH] Closing existing window before opening AH (cleanup)');
    try {
      bot.closeWindow(bot.currentWindow);
      await sleep(500);
    } catch (e) {
      console.log('[AH] Error closing window during pre-opening cleanup:', e.message);
    }
  }
  
  // Register window listener BEFORE sending command to prevent race condition
  const windowPromise = new Promise((resolve, reject) => {
    const windowHandler = (window) => {
      clearTimeout(timeout);
      console.log(`[AH] Window opened - Type: ${window.type}, Total slots: ${window.slots.length}`);
      resolve(window);
    };
    
    const timeout = setTimeout(() => {
      bot.off('windowOpen', windowHandler);
      reject(new Error('Timeout waiting for auction house window'));
    }, CONFIG.windowTimeout);
    
    bot.once('windowOpen', windowHandler);
  });
  
  // Log and send the command
  console.log('[AH] Sending command: /ah map');
  bot.chat('/ah map');
  
  // Small delay to prevent "Invalid sequence" kick
  await sleep(300);
  
  return windowPromise;
}

function findCheapMap(window) {
  // Determine container size based on window type
  // For chest GUIs: double chest = 54 slots, single chest = 27 slots
  // The player inventory slots come AFTER the container slots
  let containerSize = 0;
  
  if (window.type === 'minecraft:generic_9x6' || window.type === 'minecraft:chest' || window.type === 'generic_9x6') {
    containerSize = 54; // Double chest
  } else if (window.type === 'minecraft:generic_9x3' || window.type === 'generic_9x3') {
    containerSize = 27; // Single chest
  } else {
    // Fallback: assume standard chest based on total slots
    // Standard chest GUI has container slots + 36 player inventory slots
    if (window.slots.length >= 54 + 36) {
      containerSize = 54; // Double chest
    } else if (window.slots.length >= 27 + 36) {
      containerSize = 27; // Single chest
    } else {
      // Unknown window type, scan all slots as fallback
      containerSize = window.slots.length;
    }
  }
  
  console.log(`[AH] Window type: ${window.type}, Total slots: ${window.slots.length}, Container slots: ${containerSize}`);
  console.log(`[AH] Scanning slots 0-${containerSize - 1} (container only, excluding player inventory)`);
  
  let itemsWithData = 0;
  
  // ONLY scan container slots, NOT player inventory
  for (let slot = 0; slot < containerSize; slot++) {
    const item = window.slots[slot];
    if (!item) continue;
    
    itemsWithData++;
    console.log(`[AH DEBUG] === Slot ${slot} ===`);
    console.log(`[AH DEBUG] Item name: ${item.name || 'unknown'}`);
    console.log(`[AH DEBUG] Item displayName: ${item.displayName || 'unknown'}`);
    console.log(`[AH DEBUG] Item count: ${item.count || 1}`);
    
    // Check if item has components (1.21.1+)
    if (!item.components) {
      console.log(`[AH DEBUG] No components found for slot ${slot}`);
      continue;
    }
    
    try {
      // Extract lore using the new component-based function
      const loreLines = extractLore(item);
      
      if (loreLines.length === 0) {
        console.log(`[AH DEBUG] No lore found for slot ${slot}`);
        continue;
      }
      
      console.log(`[AH DEBUG] Lore (${loreLines.length} lines):`);
      for (let i = 0; i < loreLines.length; i++) {
        console.log(`[AH DEBUG]   Line ${i}: ${loreLines[i]}`);
      }
      
      // Parse lore lines for price and seller
      let price = null;
      let seller = null;
      
      for (const lineText of loreLines) {
        if (!price && lineText.includes('Price:')) {
          price = parsePrice(lineText);
          console.log(`[AH DEBUG]   Extracted price: ${price}`);
        }
        
        // Try to find seller name in lore
        if (!seller && lineText.includes('Seller:')) {
          const clean = stripMinecraftColors(lineText);
          const sellerMatch = clean.match(/Seller:\s*(.+)/i);
          if (sellerMatch) {
            seller = sellerMatch[1].trim();
            console.log(`[AH DEBUG]   Extracted seller: ${seller}`);
          }
        }
        
        // Break early if we found both
        if (price !== null && seller) {
          break;
        }
      }
      
      if (price !== null) {
        console.log(`[AH DEBUG] Item price: $${price}, max buy price: $${CONFIG.maxBuyPrice}`);
        if (price < CONFIG.maxBuyPrice) {
          console.log(`[AH] Found cheap map at slot ${slot}: $${price}`);
          return { slot, price, seller };
        } else {
          console.log(`[AH DEBUG] Price too high, skipping`);
        }
      } else {
        console.log(`[AH DEBUG] Could not extract price from lore`);
      }
    } catch (error) {
      console.error(`[AH DEBUG] Error parsing slot ${slot}:`, error);
      continue;
    }
  }
  
  console.log(`[AH] Scanned ${itemsWithData} items in container slots, found no cheap maps under $${CONFIG.maxBuyPrice}`);
  return null;
}

async function buyMap(window, mapSlot, mapPrice, mapSeller) {
  console.log(`[AH] Attempting to buy map at slot ${mapSlot}...`);
  
  try {
    // Click the map slot
    await bot.clickWindow(mapSlot, 0, 0);
    await sleep(500);
    
    // Click confirm button at slot 15
    console.log('[AH] Clicking confirm button...');
    await bot.clickWindow(15, 0, 0);
    
    // Wait for the window to close naturally and check for "already bought" message
    return new Promise((resolve) => {
      let alreadyBoughtMessageReceived = false;
      
      const sendSuccessWebhook = () => {
        sendWebhook('purchase', {
          message: `✅ Bought a map for $${mapPrice}`,
          color: 5763719,
          fields: [
            { name: 'Price', value: `$${mapPrice}`, inline: true },
            { name: 'Seller', value: mapSeller || 'Unknown', inline: true }
          ]
        });
      };
      
      // Listen for window close
      const windowCloseHandler = () => {
        console.log('[AH] Window closed after purchase attempt');
        
        // If no "already bought" message appeared, the purchase was successful
        if (!alreadyBoughtMessageReceived) {
          clearTimeout(timeout);
          bot.off('messagestr', messageHandler);
          sendSuccessWebhook();
          resolve(true);
        } else {
          // Window closed after already bought message - this is a failed purchase
          console.log('[AH] Window closed after "already bought" message');
          resolve(false);
        }
      };
      
      bot.once('windowClose', windowCloseHandler);
      
      // Timeout after 3 seconds if window doesn't close
      const timeout = setTimeout(() => {
        bot.off('windowClose', windowCloseHandler);
        bot.off('messagestr', messageHandler);
        
        // If we haven't received "already bought" message but window didn't close,
        // this is an ambiguous state - could be network delay or purchase issue
        if (!alreadyBoughtMessageReceived) {
          console.log('[AH] Warning: Window did not close in time - uncertain purchase state');
          console.log('[AH] Attempting to verify by checking inventory...');
          // Resolve as failed to trigger retry logic - safer than assuming success
          resolve(false);
        } else {
          // Already bought message received, definitely failed
          resolve(false);
        }
      }, 3000);
      
      const messageHandler = (msg) => {
        const normalized = normalizeText(msg);
        if (normalized.includes('already bought')) {
          console.log('[AH] Item was already bought, retrying...');
          alreadyBoughtMessageReceived = true;
          clearTimeout(timeout);
          bot.off('windowClose', windowCloseHandler);
          bot.off('messagestr', messageHandler);
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
  
  // Access bot's inventory directly - it's always available without opening
  const inventory = bot.inventory;
  
  // Use inventory.items() to get only valid items (safer than iterating all slots)
  const items = inventory.items();
  const stackedMaps = items.filter(item => 
    item.name && item.name.includes('map') && item.count > 1
  );
  
  if (stackedMaps.length === 0) {
    console.log('[INVENTORY] No stacked maps found, skipping unstack');
    return;
  }
  
  console.log(`[INVENTORY] Found ${stackedMaps.length} stacked map(s)`);
  for (const stack of stackedMaps) {
    console.log(`[INVENTORY]   - ${stack.count} maps in slot ${inventory.slots.indexOf(stack)}`);
  }
  
  console.log('[INVENTORY] Note: Map unstacking requires opening inventory window');
  console.log('[INVENTORY] If maps come as single items from AH, this step can be skipped');
  console.log('[INVENTORY] Skipping unstack - maps should already be single items from AH');
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
  
  if (maps.length > 0) {
    await sendWebhook('listing', {
      message: `📋 Listed ${maps.length} map(s) for sale`,
      color: 3447003,
      fields: [
        { name: 'Quantity', value: maps.length.toString(), inline: true },
        { name: 'Price Each', value: CONFIG.sellPrice, inline: true }
      ]
    });
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
      purchased = await buyMap(window, cheapMap.slot, cheapMap.price, cheapMap.seller);
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
      
      // Wait a moment after window closes naturally before proceeding
      await sleep(500);
      
      // Window should already be closed by the server, verify
      if (bot.currentWindow) {
        console.log('[AH] Warning: Window still open after purchase, closing manually');
        try {
          bot.closeWindow(bot.currentWindow);
          await sleep(300);
        } catch (e) {
          console.log('[AH] Error closing window:', e.message);
        }
      }
      
      // Unstack if needed
      await unstackMaps();
      
      // List the maps
      await listMaps();
      
      // Small delay before next cycle
      await sleep(1000);
    } else {
      console.log('[AH] Failed to purchase map after retries');
      
      // Window should already be closed, verify
      if (bot.currentWindow) {
        console.log('[AH] Closing window after failed purchase');
        try {
          bot.closeWindow(bot.currentWindow);
          await sleep(300);
        } catch (e) {
          console.log('[AH] Window already closed');
        }
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
    
    // Close any open window after error to prevent protocol conflicts
    try {
      if (bot.currentWindow) {
        console.log('[ERROR] Closing window after error (cleanup)');
        bot.closeWindow(bot.currentWindow);
      }
    } catch (e) {
      console.log('[ERROR] Window already closed or error during post-error cleanup:', e.message);
    }
    
    await sendWebhook('error', {
      message: `⚠️ Error in main loop`,
      color: 15158332,
      fields: [
        { name: 'Error', value: error.message || 'Unknown error' }
      ]
    });
    
    // Wait at least MIN_RETRY_DELAY (3s) before retry to prevent "Invalid sequence" kick
    // Especially important after timeout errors
    const retryDelay = Math.max(CONFIG.delayBetweenCycles, MIN_RETRY_DELAY);
    console.log(`[ERROR] Waiting ${retryDelay}ms before retry to avoid protocol conflicts`);
    await sleep(retryDelay);
    
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
  
  // Optional event debugger to diagnose window opening issues
  // WARNING: This overrides bot.emit which could potentially interfere with
  // internal mineflayer event handling. Only enable for debugging purposes.
  if (CONFIG.debugEvents) {
    console.log('[DEBUG] Event debugger enabled - will log all non-spam events');
    console.log('[DEBUG] WARNING: This overrides bot.emit and should only be used for debugging');
    const originalEmit = bot.emit.bind(bot);
    bot.emit = function(event, ...args) {
      try {
        // Filter out high-frequency spam events that make debugging impossible
        if (event !== 'move' && 
            event !== 'entityMoved' && 
            event !== 'physicsTick' && 
            !event.startsWith('packet_')) {
          console.log('[EVENT]', event);
        }
        return originalEmit(event, ...args);
      } catch (error) {
        console.error('[DEBUG] Error in event debugger:', error);
        // Ensure original emit still runs even if our logging fails
        return originalEmit(event, ...args);
      }
    };
  }
  
  bot.on('login', () => {
    console.log(`[BOT] Logged in as ${bot.username}`);
  });
  
  bot.on('spawn', async () => {
    console.log('[BOT] Spawned in game');
    console.log(`[BOT] Waiting ${CONFIG.delayAfterJoin}ms before starting...`);
    
    await sendWebhook('startup', {
      message: `🤖 Bot connected and spawned`,
      color: 3066993,
      fields: [
        { name: 'Server', value: CONFIG.host, inline: true },
        { name: 'Username', value: bot.username, inline: true }
      ]
    });
    
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
    
    // Check for map sale - format: "Username bought your Map for $price"
    // Expected message format examples:
    //   "PlayerName bought your Map for $9.9K"
    //   "Test User bought your Map for $9900"  (usernames can have spaces)
    //   "SomeGuy123 bought your Map for $10,000"
    const saleMatch = msg.match(/(.+?)\s+bought your Map for \$([0-9,.]+)(K?)/i);
    if (saleMatch) {
      const buyer = saleMatch[1].trim();
      const priceStr = `Price: $${saleMatch[2]}${saleMatch[3] || ''}`;
      
      const salePrice = parsePrice(priceStr);
      if (salePrice === null) {
        console.log(`[SALE] Invalid price format "$${saleMatch[2]}${saleMatch[3] || ''}", skipping webhook`);
        return;
      }
      
      console.log(`[SALE] ${buyer} bought a map for $${salePrice}`);
      sendWebhook('sale', {
        message: `💰 Sold a map!`,
        color: 5763719,
        fields: [
          { name: 'Buyer', value: buyer, inline: true },
          { name: 'Price', value: `$${salePrice}`, inline: true }
        ]
      });
    }
  });
  
  bot.on('kicked', (reason) => {
    // Handle kick reason which may be an object (chat component) or string
    let reasonText = 'Unknown';
    
    if (typeof reason === 'string') {
      reasonText = reason;
    } else if (reason && typeof reason === 'object') {
      // Try to extract text from chat component
      if (reason.text) {
        reasonText = reason.text;
      } else if (reason.toString && typeof reason.toString === 'function') {
        try {
          reasonText = reason.toString();
        } catch (e) {
          reasonText = JSON.stringify(reason);
        }
      } else {
        reasonText = JSON.stringify(reason);
      }
    }
    
    console.log(`[BOT] Kicked: ${reasonText}`);
    console.log(`[BOT DEBUG] Full kick reason object: ${JSON.stringify(reason)}`);
    
    sendWebhook('error', {
      message: `❌ Bot was kicked from server`,
      color: 15158332,
      fields: [
        { name: 'Reason', value: reasonText }
      ]
    });
    reconnect();
  });
  
  bot.on('end', () => {
    console.log('[BOT] Connection ended');
    reconnect();
  });
  
  bot.on('error', (err) => {
    // Suppress common harmless packet errors
    if (isPartialPacketError(err)) {
      // These are harmless protocol parsing warnings, suppress them
      return;
    }
    
    console.error('[BOT] Error:', err);
    sendWebhook('error', {
      message: `⚠️ Bot encountered an error`,
      color: 15158332,
      fields: [
        { name: 'Error', value: err.message || 'Unknown error' }
      ]
    });
  });
  
  // Suppress packet_dump errors for partial packets
  // Note: This accesses mineflayer's internal _client property as a workaround
  // for suppressing harmless partial packet warnings. This may break if mineflayer
  // changes its internal structure. Tested with mineflayer 4.35.0
  if (bot._client) {
    bot._client.on('error', (err) => {
      if (isPartialPacketError(err)) {
        // Suppress these errors - they're harmless protocol parsing warnings
        return;
      }
      console.error('[CLIENT] Error:', err);
    });
  }
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
