// === MUST BE FIRST LINES OF FILE ===
const _stdoutWrite = process.stdout.write.bind(process.stdout);
const _stderrWrite = process.stderr.write.bind(process.stderr);
const _shouldFilterLog = (chunk) => {
  const s = typeof chunk === 'string' ? chunk : chunk.toString();
  return s.includes('Chunk size is') || s.includes('partial packet') || s.includes('player_info');
};
process.stdout.write = function(chunk, ...a) { if (_shouldFilterLog(chunk)) return true; return _stdoutWrite(chunk, ...a); };
process.stderr.write = function(chunk, ...a) { if (_shouldFilterLog(chunk)) return true; return _stderrWrite(chunk, ...a); };
// === END FILTER ===

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

// Default webhook configuration
const defaultWebhookConfig = {
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
};

// Merge webhook config with defaults
const webhookConfig = fileConfig.webhook ? {
  enabled: fileConfig.webhook.enabled ?? defaultWebhookConfig.enabled,
  url: fileConfig.webhook.url ?? defaultWebhookConfig.url,
  displayName: fileConfig.webhook.displayName ?? defaultWebhookConfig.displayName,
  events: {
    purchase: fileConfig.webhook.events?.purchase ?? defaultWebhookConfig.events.purchase,
    listing: fileConfig.webhook.events?.listing ?? defaultWebhookConfig.events.listing,
    sale: fileConfig.webhook.events?.sale ?? defaultWebhookConfig.events.sale,
    afk: fileConfig.webhook.events?.afk ?? defaultWebhookConfig.events.afk,
    error: fileConfig.webhook.events?.error ?? defaultWebhookConfig.events.error,
    startup: fileConfig.webhook.events?.startup ?? defaultWebhookConfig.events.startup
  }
} : defaultWebhookConfig;

const CONFIG = {
  host: fileConfig.host || 'donutsmp.net',
  port: fileConfig.port || 25565,
  username: fileConfig.username || process.env.BOT_USERNAME || 'BOT_USERNAME',
  auth: fileConfig.auth || process.env.BOT_AUTH || 'microsoft',
  version: fileConfig.version || '1.21.11',
  maxBuyPrice: fileConfig.maxBuyPrice || parseInt(process.env.MAX_BUY_PRICE) || 2500,
  sellPrice: fileConfig.sellPrice || process.env.SELL_PRICE || '9.9k',
  maxListingsPerCycle: fileConfig.maxListingsPerCycle || parseInt(process.env.MAX_LISTINGS_PER_CYCLE) || 20,
  delayBetweenCycles: fileConfig.delayBetweenCycles || parseInt(process.env.DELAY_BETWEEN_CYCLES) || 5000,
  delayAfterJoin: fileConfig.delayAfterJoin || parseInt(process.env.DELAY_AFTER_JOIN) || 5000,
  windowTimeout: fileConfig.windowTimeout || parseInt(process.env.WINDOW_TIMEOUT) || 15000,
  debugEvents: fileConfig.debugEvents || process.env.DEBUG_EVENTS === 'true' || false,
  mode: fileConfig.mode || process.env.BOT_MODE || 'normal', // 'normal' or 'sell-only'
  webhook: webhookConfig
};

// Log webhook configuration status
if (CONFIG.webhook.enabled) {
  console.log('[CONFIG] Webhook notifications: ENABLED');
  
  // Display webhook URL (truncate if too long)
  let urlDisplay;
  if (!CONFIG.webhook.url) {
    urlDisplay = 'NOT SET';
  } else if (CONFIG.webhook.url.length > 50) {
    urlDisplay = CONFIG.webhook.url.substring(0, 50) + '...';
  } else {
    urlDisplay = CONFIG.webhook.url;
  }
  console.log(`[CONFIG] Webhook URL: ${urlDisplay}`);
  
  const enabledEvents = Object.entries(CONFIG.webhook.events)
    .filter(([, enabled]) => enabled)
    .map(([name]) => name)
    .join(', ');
  console.log(`[CONFIG] Webhook events: ${enabledEvents}`);
} else {
  console.log('[CONFIG] Webhook notifications: DISABLED');
}

// Constants
const HOTBAR_START_SLOT = 36;
const HOTBAR_END_SLOT = 44;
const WARN_MAP_COUNT_THRESHOLD = 5;
const MAX_AH_LISTINGS = 27; // Maximum active listings allowed on auction house
const MIN_RETRY_DELAY = 3000; // Minimum delay before retry after errors (ms)
const REDUCED_CYCLE_DELAY = 2500; // Fast cycle delay when no maps found or purchase failed (ms)
const CLICK_CONFIRM_DELAY = 400; // Delay between clicking item and confirm button (ms)
const REFRESH_WAIT_DELAY = 500; // Delay after clicking refresh button (ms)
const WINDOW_CLOSE_TIMEOUT = 3000; // Timeout waiting for window to close (ms)
const WINDOW_CLEANUP_DELAY = 300; // Delay after closing window for cleanup (ms)
const REFRESH_BUTTON_SLOT = 49; // Slot containing the refresh button (anvil icon) in AH window
const MAX_LISTING_ITERATIONS = 50; // Maximum iterations in listMaps to prevent infinite loops
const MAX_LISTING_RETRIES = 3; // Maximum retries per map when listing fails
const MAINTENANCE_CYCLE_INTERVAL = 10; // Run sell-all maintenance every N buy cycles
const MAINTENANCE_TIME_INTERVAL = 180000; // Run sell-all maintenance every 3 minutes (ms)
const UNSTACK_DELAY = 200; // Delay between unstack operations (ms)

let bot;
let isAfkDetected = false;
let isRunning = false;
let lastStateId = 0; // Track stateId for window_click packets
let buyCycleCount = 0; // Track number of buy cycles for periodic maintenance
let lastMaintenanceTime = 0; // Track last maintenance run time
let lastMaintenanceCycle = 0; // Track last cycle when maintenance ran

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

/**
 * Count the number of filled_map items currently in inventory.
 * @returns {number} Total count of all map items (including stacks)
 */
function countMapsInInventory() {
  if (!bot || !bot.inventory) return 0;
  
  let totalMaps = 0;
  const inventory = bot.inventory;
  
  for (let slot = 0; slot < inventory.slots.length; slot++) {
    const item = inventory.slots[slot];
    // Check for filled_map specifically to avoid matching other items
    if (item?.name === 'filled_map') {
      totalMaps += item.count || 1;
    }
  }
  
  return totalMaps;
}

/**
 * Custom window click function that uses tracked stateId to prevent "Invalid sequence" kicks.
 * This is necessary because mineflayer's bot.clickWindow() doesn't track stateId correctly in 1.21.1+.
 * 
 * @param {number} windowId - The window ID (use window.id from the opened window)
 * @param {number} slot - The slot number to click (0-based index)
 * @param {number} mouseButton - Mouse button: 0 = left, 1 = right, 2 = middle
 * @param {number} mode - Click mode: 0 = normal click, 1 = shift-click, 2 = number key, etc.
 */
function clickWindowSlot(windowId, slot, mouseButton, mode) {
  if (!bot || !bot._client) {
    console.error('[CLICK] Bot or client not available for window click');
    return;
  }
  
  console.log(`[CLICK] Clicking window ${windowId}, slot ${slot}, stateId ${lastStateId}`);
  
  bot._client.write('window_click', {
    windowId: windowId,
    slot: slot,
    mouseButton: mouseButton,
    mode: mode,
    stateId: lastStateId,
    // cursorItem.present: false means no item is on the cursor
    cursorItem: { present: false },
    // changedSlots: [] tells the server we're not predicting slot changes - let the server handle it
    changedSlots: []
  });
}

// Setup stateId tracking from server packets
// Must be called after bot is created
function setupStateIdTracking() {
  if (!bot || !bot._client) {
    console.error('[STATEID] Bot or client not available for state tracking setup');
    return;
  }
  
  // Track stateId from window_items packets
  bot._client.on('window_items', (packet) => {
    if (packet.stateId !== undefined) {
      lastStateId = packet.stateId;
      console.log(`[STATEID] Updated from window_items: ${lastStateId}`);
    }
  });
  
  // Track stateId from set_slot packets
  bot._client.on('set_slot', (packet) => {
    if (packet.stateId !== undefined) {
      lastStateId = packet.stateId;
      console.log(`[STATEID] Updated from set_slot: ${lastStateId}`);
    }
  });
  
  console.log('[STATEID] State ID tracking initialized');
}

async function sendWebhook(event, data) {
  if (!CONFIG.webhook.enabled) {
    console.log(`[WEBHOOK] Webhook disabled, skipping ${event} event`);
    return;
  }
  if (!CONFIG.webhook.url) {
    console.error('[WEBHOOK] Webhook URL not configured');
    return;
  }
  if (!CONFIG.webhook.events[event]) {
    console.log(`[WEBHOOK] Event ${event} is disabled in configuration`);
    return;
  }
  
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
            console.log(`[WEBHOOK] Successfully sent ${event} webhook`);
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
        clickWindowSlot(window.id, 5, 0, 0);
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
  
  // Don't log scanning here - let the caller log to avoid duplicates
  
  // ONLY scan container slots, NOT player inventory
  for (let slot = 0; slot < containerSize; slot++) {
    const item = window.slots[slot];
    if (!item) continue;
    
    // Check if item has components (1.21.1+)
    if (!item.components) continue;
    
    try {
      // Extract lore using the new component-based function
      const loreLines = extractLore(item);
      
      if (loreLines.length === 0) continue;
      
      // Parse lore lines for price and seller
      let price = null;
      let seller = null;
      
      for (const lineText of loreLines) {
        if (!price && lineText.includes('Price:')) {
          price = parsePrice(lineText);
        }
        
        // Try to find seller name in lore
        if (!seller && lineText.includes('Seller:')) {
          const clean = stripMinecraftColors(lineText);
          const sellerMatch = clean.match(/Seller:\s*(.+)/i);
          if (sellerMatch) {
            seller = sellerMatch[1].trim();
          }
        }
        
        // Break early if we found both
        if (price !== null && seller) {
          break;
        }
      }
      
      if (price !== null && price < CONFIG.maxBuyPrice) {
        console.log(`[AH] ✓ Found cheap map at slot ${slot}: $${price} (seller: ${seller || 'unknown'})`);
        return { slot, price, seller };
      }
    } catch (error) {
      console.error(`[AH] Error parsing slot ${slot}:`, error.message);
      continue;
    }
  }
  
  // Don't log here - let the caller handle logging to avoid duplicates
  return null;
}

async function buyMap(window, mapSlot, mapPrice, mapSeller) {
  console.log(`[AH] Attempting to buy map at slot ${mapSlot}...`);
  
  try {
    // Click the map slot using manual window click with stateId
    clickWindowSlot(window.id, mapSlot, 0, 0);
    // Wait for the server to update the window state
    // Reduced from 1000ms - stateId tracking handles protocol correctness
    await sleep(CLICK_CONFIRM_DELAY);
    
    // Click confirm button at slot 15
    console.log('[AH] Clicking confirm button...');
    clickWindowSlot(window.id, 15, 0, 0);
    
    // Wait for the window to close naturally and check for "already bought" message
    return new Promise((resolve) => {
      let alreadyBoughtMessageReceived = false;
      let resolved = false; // Track if promise has been resolved to prevent double resolution
      
      const safeResolve = (value) => {
        if (!resolved) {
          resolved = true;
          resolve(value);
        }
      };
      
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
      
      // Declare timeout variable early so it can be referenced in handlers
      let timeout;
      
      const messageHandler = (msg) => {
        const normalized = normalizeText(msg);
        if (normalized.includes('already bought')) {
          console.log('[AH] Item was already bought, retrying...');
          alreadyBoughtMessageReceived = true;
          clearTimeout(timeout);
          bot.off('windowClose', windowCloseHandler);
          bot.off('messagestr', messageHandler);
          safeResolve(false);
        }
      };
      
      // Listen for window close
      const windowCloseHandler = () => {
        console.log('[AH] Window closed after purchase attempt');
        
        // Wait 2 seconds after window close to catch delayed "already bought" messages
        // The message might arrive slightly after the window closes
        setTimeout(() => {
          // If no "already bought" message appeared, the purchase was successful
          if (!alreadyBoughtMessageReceived) {
            clearTimeout(timeout);
            bot.off('messagestr', messageHandler);
            sendSuccessWebhook();
            safeResolve(true);
          } else {
            // Window closed but "already bought" message received - failed purchase
            console.log('[AH] Window closed but "already bought" message received');
            safeResolve(false);
          }
        }, 2000);
      };
      
      bot.once('windowClose', windowCloseHandler);
      
      // Timeout after 5 seconds if window doesn't close or verification doesn't complete
      timeout = setTimeout(() => {
        bot.off('windowClose', windowCloseHandler);
        bot.off('messagestr', messageHandler);
        
        // If we haven't received "already bought" message but window didn't close,
        // this is an ambiguous state - could be network delay or purchase issue
        if (!alreadyBoughtMessageReceived) {
          console.log('[AH] Warning: Window did not close in time - uncertain purchase state');
          console.log('[AH] Attempting to verify by checking inventory...');
          // Resolve as failed to trigger retry logic - safer than assuming success
          safeResolve(false);
        } else {
          // Already bought message received, definitely failed
          safeResolve(false);
        }
      }, 5000);
      
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
  // Map to include slot indices for efficient logging
  const items = inventory.items();
  const stackedMaps = [];
  
  for (const item of items) {
    // Check for filled_map specifically to avoid matching other items
    if (item.name && item.name === 'filled_map' && item.count > 1) {
      const slotIndex = inventory.slots.indexOf(item);
      stackedMaps.push({ item, slotIndex });
    }
  }
  
  if (stackedMaps.length === 0) {
    console.log('[INVENTORY] No stacked maps found, skipping unstack');
    return;
  }
  
  console.log(`[INVENTORY] Found ${stackedMaps.length} stacked map slot(s)`);
  for (const { item, slotIndex } of stackedMaps) {
    console.log(`[INVENTORY]   - ${item.count} maps in slot ${slotIndex}`);
  }
  
  // Note: Maps from the auction house typically come as single items
  // If stacked, they'll be split when moved to hotbar for listing
  console.log('[INVENTORY] Maps will be split during hotbar filling for listing');
}

/**
 * Comprehensive sell-all routine that handles accumulated maps.
 * This function:
 * 1. Scans entire inventory for all filled_map items
 * 2. Unstacks any stacked maps into singles
 * 3. Lists each single map with verification
 * 4. Logs results (X maps listed, Y failed)
 */
async function sellAllMaps() {
  console.log('[SELL-ALL] Starting comprehensive map cleanup...');
  
  const initialMapCount = countMapsInInventory();
  console.log(`[SELL-ALL] Initial inventory: ${initialMapCount} map(s)`);
  
  if (initialMapCount === 0) {
    console.log('[SELL-ALL] No maps to sell, skipping');
    return { listed: 0, failed: 0 };
  }
  
  let listedCount = 0;
  let failedCount = 0;
  
  // Step 1: Unstack all stacked maps into singles
  console.log('[SELL-ALL] Step 1: Unstacking all map stacks...');
  const inventory = bot.inventory;
  
  // Find all stacked maps
  const stackedMaps = [];
  for (let slot = 0; slot < inventory.slots.length; slot++) {
    const item = inventory.slots[slot];
    // Check for filled_map specifically to avoid matching other items
    if (item && item.name && item.name === 'filled_map' && item.count > 1) {
      stackedMaps.push({ slot, count: item.count });
    }
  }
  
  if (stackedMaps.length > 0) {
    console.log(`[SELL-ALL] Found ${stackedMaps.length} stacked map slot(s) to unstack`);
    
    for (const { slot: stackSlot, count } of stackedMaps) {
      console.log(`[SELL-ALL] Unstacking ${count} maps from slot ${stackSlot}...`);
      
      // Unstack by right-clicking to pick up 1, then left-clicking to place
      while (inventory.slots[stackSlot] && inventory.slots[stackSlot].count > 1) {
        // Find an empty slot
        let emptySlot = -1;
        for (let i = 0; i < inventory.slots.length; i++) {
          if (!inventory.slots[i]) {
            emptySlot = i;
            break;
          }
        }
        
        if (emptySlot === -1) {
          console.log('[SELL-ALL] Warning: No empty slots to unstack, inventory full');
          break;
        }
        
        try {
          // Right-click stack to pick up 1 map
          // Window ID 0 is the player inventory (this is standard in Minecraft protocol)
          clickWindowSlot(0, stackSlot, 1, 0);
          await sleep(UNSTACK_DELAY);
          
          // Left-click empty slot to place it
          clickWindowSlot(0, emptySlot, 0, 0);
          await sleep(UNSTACK_DELAY);
        } catch (error) {
          console.error(`[SELL-ALL] Error unstacking: ${error.message}`);
          break;
        }
      }
    }
    
    console.log('[SELL-ALL] Unstacking complete');
  } else {
    console.log('[SELL-ALL] No stacked maps found, all maps are singles');
  }
  
  // Step 2: List all single maps from inventory
  console.log('[SELL-ALL] Step 2: Listing all maps...');
  
  let iteration = 0;
  const maxIterations = 100; // Safety limit
  
  while (iteration < maxIterations) {
    iteration++;
    
    // Find next map to list
    let mapSlot = -1;
    for (let slot = 0; slot < inventory.slots.length; slot++) {
      const item = inventory.slots[slot];
      // Check for filled_map specifically to avoid matching other items
      if (item && item.name && item.name === 'filled_map') {
        mapSlot = slot;
        break;
      }
    }
    
    if (mapSlot === -1) {
      console.log('[SELL-ALL] No more maps in inventory');
      break;
    }
    
    // Move map to hotbar slot 0 if not already there
    const hotbarSlot = HOTBAR_START_SLOT; // Use slot 36 (hotbar 0)
    if (mapSlot !== hotbarSlot) {
      try {
        console.log(`[SELL-ALL] Moving map from slot ${mapSlot} to hotbar...`);
        await bot.moveSlotItem(mapSlot, hotbarSlot);
        await sleep(200);
      } catch (error) {
        console.error(`[SELL-ALL] Error moving map: ${error.message}`);
        failedCount++;
        continue;
      }
    }
    
    // List the map with verification and retry
    const listResult = await listSingleMapWithVerification(0); // Hotbar slot 0
    
    if (listResult) {
      listedCount++;
    } else {
      failedCount++;
    }
  }
  
  if (iteration >= maxIterations) {
    console.log('[SELL-ALL] Warning: Reached maximum iterations, may have maps remaining');
  }
  
  const finalMapCount = countMapsInInventory();
  console.log(`[SELL-ALL] Cleanup complete: ${listedCount} listed, ${failedCount} failed`);
  console.log(`[SELL-ALL] Final inventory: ${finalMapCount} map(s) remaining`);
  
  // Send webhook notification
  await sendWebhook('listing', {
    message: `🧹 Sell-all cleanup completed`,
    color: 3447003,
    fields: [
      { name: 'Listed', value: listedCount.toString(), inline: true },
      { name: 'Failed', value: failedCount.toString(), inline: true },
      { name: 'Remaining', value: finalMapCount.toString(), inline: true }
    ]
  });
  
  return { listed: listedCount, failed: failedCount };
}

/**
 * List a single map from a hotbar slot with verification and retry logic.
 * @param {number} hotbarSlotIndex - The hotbar slot index (0-8)
 * @returns {boolean} True if successfully listed, false otherwise
 */
async function listSingleMapWithVerification(hotbarSlotIndex) {
  const hotbarSlot = HOTBAR_START_SLOT + hotbarSlotIndex;
  
  for (let attempt = 1; attempt <= MAX_LISTING_RETRIES; attempt++) {
    console.log(`[LISTING] Attempt ${attempt}/${MAX_LISTING_RETRIES} to list map from hotbar slot ${hotbarSlotIndex}...`);
    
    // Count maps before listing
    const mapsBefore = countMapsInInventory();
    
    // Verify we still have a map in the hotbar slot
    const inventory = bot.inventory;
    const item = inventory.slots[hotbarSlot];
    
    // Check for filled_map specifically to avoid matching other items
    if (!item || !item.name || item.name !== 'filled_map') {
      console.log(`[LISTING] No map in hotbar slot ${hotbarSlotIndex}, skipping`);
      return false;
    }
    
    try {
      // Hold the map in hotbar
      bot.setQuickBarSlot(hotbarSlotIndex);
      await sleep(300);
      
      // Send /ah list command
      bot.chat(`/ah list ${CONFIG.sellPrice}`);
      
      // Wait for confirmation window to open
      const confirmWindow = await new Promise((resolve, reject) => {
        let timeout;
        
        const windowHandler = (window) => {
          clearTimeout(timeout);
          resolve(window);
        };
        
        timeout = setTimeout(() => {
          bot.off('windowOpen', windowHandler);
          reject(new Error('Timeout waiting for listing confirmation window'));
        }, CONFIG.windowTimeout);
        
        bot.once('windowOpen', windowHandler);
      });
      
      console.log(`[LISTING] Confirmation window opened (window ID: ${confirmWindow.id})`);
      
      // Wait for GUI to fully render before clicking
      await sleep(CLICK_CONFIRM_DELAY);
      
      // Click the confirm button at slot 15
      console.log('[LISTING] Clicking confirm button at slot 15...');
      clickWindowSlot(confirmWindow.id, 15, 0, 0);
      
      // Wait for window to close
      await new Promise((resolve) => {
        let closeTimeout;
        
        const closeHandler = () => {
          console.log('[LISTING] Listing confirmed, window closed');
          resolve();
        };
        
        closeTimeout = setTimeout(() => {
          bot.off('windowClose', closeHandler);
          console.log('[LISTING] Window did not close in time, continuing...');
          resolve();
        }, WINDOW_CLOSE_TIMEOUT);
        
        bot.once('windowClose', () => {
          clearTimeout(closeTimeout);
          closeHandler();
        });
      });
      
      // Wait a moment for inventory to update
      await sleep(500);
      
      // Verify listing worked by checking map count
      const mapsAfter = countMapsInInventory();
      
      if (mapsAfter < mapsBefore) {
        console.log(`[LISTING] ✓ Listing verified: ${mapsBefore} → ${mapsAfter} maps`);
        
        // Wait 1 second before next listing to avoid server cooldown
        console.log('[LISTING] Waiting 1 second before next listing (cooldown)...');
        await sleep(1000);
        
        return true;
      } else {
        console.log(`[LISTING] ✗ Listing FAILED: Map count unchanged (${mapsBefore} → ${mapsAfter})`);
        
        if (attempt < MAX_LISTING_RETRIES) {
          console.log(`[LISTING] Retrying... (attempt ${attempt + 1}/${MAX_LISTING_RETRIES})`);
          await sleep(1000); // Wait before retry
        }
      }
      
    } catch (error) {
      console.error(`[LISTING] Error during listing attempt ${attempt}: ${error.message}`);
      
      // Close any open window to prevent protocol conflicts
      if (bot.currentWindow) {
        try {
          bot.closeWindow(bot.currentWindow);
          await sleep(WINDOW_CLEANUP_DELAY);
        } catch (e) {
          // Ignore close errors
        }
      }
      
      if (attempt < MAX_LISTING_RETRIES) {
        await sleep(1000); // Wait before retry
      }
    }
  }
  
  console.log(`[LISTING] Failed to list map after ${MAX_LISTING_RETRIES} attempts`);
  return false;
}

async function listMaps() {
  console.log('[LISTING] Listing purchased maps...');
  
  let totalListedCount = 0;
  let batchNumber = 0;
  const maxBatches = Math.ceil(CONFIG.maxListingsPerCycle / 9); // 9 slots per hotbar
  
  // Keep listing in batches until we hit the limit or run out of maps
  while (totalListedCount < CONFIG.maxListingsPerCycle && batchNumber < maxBatches) {
    batchNumber++;
    
    // Step 1: Find all maps in non-hotbar inventory slots
    const inventory = bot.inventory;
    const mapsToMove = [];
    
    for (let slot = 0; slot < inventory.slots.length; slot++) {
      const item = inventory.slots[slot];
      // Skip hotbar slots (36-44) - we'll fill those
      if (slot >= HOTBAR_START_SLOT && slot <= HOTBAR_END_SLOT) continue;
      
      // Check for filled_map specifically to avoid matching other items
      if (item && item.name && item.name === 'filled_map') {
        mapsToMove.push({ item, slot });
      }
    }
    
    if (mapsToMove.length === 0) {
      console.log(`[LISTING] No more maps to list. Total listed: ${totalListedCount}`);
      break;
    }
    
    console.log(`[LISTING] Batch ${batchNumber}: Found ${mapsToMove.length} map(s) in inventory`);
    
    // Step 2: Fill hotbar with maps (up to 9 slots)
    const mapsToList = Math.min(9, mapsToMove.length, CONFIG.maxListingsPerCycle - totalListedCount);
    console.log(`[LISTING] Filling hotbar with ${mapsToList} map(s)...`);
    
    for (let i = 0; i < mapsToList; i++) {
      const { slot: sourceSlot } = mapsToMove[i];
      const hotbarSlot = HOTBAR_START_SLOT + i; // Fill hotbar slots 0-8
      
      try {
        // Move map to hotbar
        console.log(`[LISTING] Moving map from slot ${sourceSlot} to hotbar slot ${i}...`);
        await bot.moveSlotItem(sourceSlot, hotbarSlot);
        await sleep(200);
      } catch (error) {
        console.error(`[LISTING] Error moving map to hotbar: ${error.message}`);
        break;
      }
    }
    
    // Step 3: List all maps from hotbar
    console.log(`[LISTING] Listing ${mapsToList} map(s) from hotbar...`);
    
    for (let hotbarSlotIndex = 0; hotbarSlotIndex < mapsToList; hotbarSlotIndex++) {
      if (totalListedCount >= CONFIG.maxListingsPerCycle) {
        console.log(`[LISTING] Reached maximum listings per cycle (${CONFIG.maxListingsPerCycle})`);
        break;
      }
      
      const hotbarSlot = HOTBAR_START_SLOT + hotbarSlotIndex;
      const item = inventory.slots[hotbarSlot];
      
      // Check for filled_map specifically to avoid matching other items
      if (!item || !item.name || item.name !== 'filled_map') {
        console.log(`[LISTING] No map in hotbar slot ${hotbarSlotIndex}, skipping...`);
        continue;
      }
      
      // Use the new verification function to list with retry
      const listResult = await listSingleMapWithVerification(hotbarSlotIndex);
      
      if (listResult) {
        totalListedCount++;
      } else {
        console.log(`[LISTING] Failed to list map from hotbar slot ${hotbarSlotIndex}`);
      }
    }
    
    console.log(`[LISTING] Batch ${batchNumber} complete. Listed ${totalListedCount} total map(s) so far.`);
  }
  
  if (totalListedCount >= CONFIG.maxListingsPerCycle) {
    console.log(`[LISTING] Hit listing limit (${CONFIG.maxListingsPerCycle} per cycle). Remaining maps will be listed in next cycle.`);
  }
  
  if (totalListedCount > 0) {
    await sendWebhook('listing', {
      message: `📋 Listed ${totalListedCount} map(s) for sale`,
      color: 3447003,
      fields: [
        { name: 'Quantity', value: totalListedCount.toString(), inline: true },
        { name: 'Price Each', value: CONFIG.sellPrice, inline: true }
      ]
    });
  }
  
  console.log(`[LISTING] Successfully listed ${totalListedCount} map(s)`);
}

async function mainLoop() {
  if (!bot || isAfkDetected || isRunning) return;
  
  isRunning = true;
  
  try {
    // CRITICAL: Check for ANY maps in inventory before buying
    // The bot MUST have zero maps before starting a buy cycle
    const currentMapCount = countMapsInInventory();
    if (currentMapCount > 0) {
      console.log(`[INVENTORY] CRITICAL: Found ${currentMapCount} map(s) in inventory - cannot buy while holding maps!`);
      console.log('[INVENTORY] Running mandatory sell-all cleanup before buying...');
      
      // Run full sell-all routine: unstack all maps and list each one
      await sellAllMaps();
      
      // Verify inventory is now clear
      const remainingMaps = countMapsInInventory();
      if (remainingMaps > 0) {
        console.log(`[WARNING] Still have ${remainingMaps} map(s) after sell-all cleanup!`);
        console.log('[WARNING] Waiting before retry...');
        await sleep(5000);
      } else {
        console.log('[INVENTORY] ✓ Inventory clear, proceeding to buy cycle');
      }
      
      isRunning = false;
      setImmediate(() => mainLoop());
      return;
    }
    
    // Open auction house
    const window = await openAuctionHouse();
    
    // Find cheap map - initial scan
    console.log(`[AH] Scanning for cheap maps under $${CONFIG.maxBuyPrice}...`);
    let cheapMap = findCheapMap(window);
    
    if (!cheapMap) {
      // No cheap maps - use refresh button instead of reopening
      console.log(`[AH] No cheap maps found under $${CONFIG.maxBuyPrice}, refreshing...`);
      try {
        // Click refresh button (anvil icon in AH window)
        console.log(`[AH] Clicking refresh button (slot ${REFRESH_BUTTON_SLOT})...`);
        clickWindowSlot(window.id, REFRESH_BUTTON_SLOT, 0, 0);
        await sleep(REFRESH_WAIT_DELAY); // Wait for refresh to complete
        
        // Check again after refresh
        cheapMap = findCheapMap(window);
        
        if (!cheapMap) {
          // Still no cheap maps after refresh - close and wait
          console.log(`[AH] No cheap maps found under $${CONFIG.maxBuyPrice} after refresh`);
          if (bot.currentWindow) {
            bot.closeWindow(bot.currentWindow);
          }
          // Reduced delay when no cheap maps found
          await sleep(REDUCED_CYCLE_DELAY);
          isRunning = false;
          setImmediate(() => mainLoop());
          return;
        }
      } catch (e) {
        console.log('[AH] Error refreshing, will reopen next cycle:', e.message);
        if (bot.currentWindow) {
          bot.closeWindow(bot.currentWindow);
        }
        await sleep(REDUCED_CYCLE_DELAY);
        isRunning = false;
        setImmediate(() => mainLoop());
        return;
      }
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
          await sleep(WINDOW_CLEANUP_DELAY);
        } catch (e) {
          console.log('[AH] Error closing window:', e.message);
        }
      }
      
      // Unstack if needed
      await unstackMaps();
      
      // List the maps
      await listMaps();
      
      // Increment buy cycle count
      buyCycleCount++;
      
      // Check inventory after listing - if too many maps remain, something is wrong
      const remainingMaps = countMapsInInventory();
      console.log(`[INVENTORY] After listing: ${remainingMaps} map(s) remaining in inventory`);
      
      if (remainingMaps > WARN_MAP_COUNT_THRESHOLD) {
        console.log(`[WARNING] More than ${WARN_MAP_COUNT_THRESHOLD} maps in inventory after listing!`);
        console.log('[WARNING] Pausing buying and running sell-all cleanup...');
        
        await sendWebhook('error', {
          message: `⚠️ Too many maps in inventory (${remainingMaps})`,
          color: 15158332,
          fields: [
            { name: 'Action', value: 'Running sell-all cleanup', inline: true }
          ]
        });
        
        // Run sell-all to clean up
        await sellAllMaps();
      }
      
      // Periodic maintenance: Run sell-all every N cycles or every M minutes
      const currentTime = Date.now();
      const timeSinceLastMaintenance = currentTime - lastMaintenanceTime;
      
      // Check if we've reached the cycle interval and haven't run maintenance for this interval yet
      // Note: When buyCycleCount is 0, the modulo is 0 but lastMaintenanceCycle is also 0, so this won't trigger
      const shouldRunCycleMaintenance = buyCycleCount % MAINTENANCE_CYCLE_INTERVAL === 0 && 
                                        buyCycleCount !== lastMaintenanceCycle;
      
      // Check if enough time has passed since last maintenance
      const shouldRunTimeMaintenance = lastMaintenanceTime > 0 && 
                                       timeSinceLastMaintenance >= MAINTENANCE_TIME_INTERVAL;
      
      if (shouldRunCycleMaintenance || shouldRunTimeMaintenance) {
        console.log('[MAINTENANCE] Running periodic sell-all cleanup...');
        console.log(`[MAINTENANCE] Cycles since startup: ${buyCycleCount}, Time since last maintenance: ${Math.round(timeSinceLastMaintenance / 1000)}s`);
        
        await sellAllMaps();
        lastMaintenanceTime = currentTime;
        lastMaintenanceCycle = buyCycleCount;
      }
      
      // No delay after successful purchase - go immediately to next cycle
    } else {
      console.log('[AH] Failed to purchase map after retries');
      
      // Window should already be closed, verify
      if (bot.currentWindow) {
        console.log('[AH] Closing window after failed purchase');
        try {
          bot.closeWindow(bot.currentWindow);
          await sleep(WINDOW_CLEANUP_DELAY);
        } catch (e) {
          console.log('[AH] Window already closed');
        }
      }
      // Reduced delay when purchase fails
      await sleep(REDUCED_CYCLE_DELAY);
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
    const retryDelay = Math.max(REDUCED_CYCLE_DELAY, MIN_RETRY_DELAY);
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
  
  // Setup stateId tracking for manual window clicks
  setupStateIdTracking();
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
    try {
      console.log('[BOT] Spawned in game');
      console.log(`[BOT] Waiting ${CONFIG.delayAfterJoin}ms before starting...`);
      
      await sendWebhook('startup', {
        message: `🤖 Bot connected and spawned`,
        color: 3066993,
        fields: [
          { name: 'Server', value: CONFIG.host, inline: true },
          { name: 'Username', value: bot.username, inline: true },
          { name: 'Mode', value: CONFIG.mode, inline: true }
        ]
      });
      
      await sleep(CONFIG.delayAfterJoin);
      
      // Run startup cleanup to clear any leftover maps from previous sessions
      console.log('[STARTUP] Running startup cleanup to clear leftover maps...');
      const startupResult = await sellAllMaps();
      console.log(`[STARTUP] Cleanup complete: ${startupResult.listed} listed, ${startupResult.failed} failed`);
      
      // Initialize maintenance timer
      lastMaintenanceTime = Date.now();
      
      // Check if in sell-only mode
      if (CONFIG.mode === 'sell-only') {
        console.log('[SELL-ONLY] Sell-only mode complete. Exiting...');
        
        await sendWebhook('startup', {
          message: `✅ Sell-only cleanup completed`,
          color: 3066993,
          fields: [
            { name: 'Listed', value: startupResult.listed.toString(), inline: true },
            { name: 'Failed', value: startupResult.failed.toString(), inline: true }
          ]
        });
        
        // Disconnect and exit
        bot.quit('Sell-only mode completed');
        process.exit(0);
      }
      
      // Normal mode: start main loop
      if (!isRunning && !isAfkDetected) {
        console.log('[BOT] Starting main loop');
        mainLoop();
      }
    } catch (error) {
      console.error('[BOT] Error during spawn initialization:', error);
      
      // Try to send webhook notification, but don't let webhook errors cause additional problems
      try {
        await sendWebhook('error', {
          message: `⚠️ Bot encountered an error during startup`,
          color: 15158332,
          fields: [
            { name: 'Error', value: error.message || String(error), inline: false }
          ]
        });
      } catch (webhookError) {
        console.error('[BOT] Failed to send error webhook:', webhookError.message);
      }
      
      // Attempt to reconnect after error
      reconnect();
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
