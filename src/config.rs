use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvents {
    #[serde(default = "default_true")]
    pub purchase: bool,
    #[serde(default = "default_true")]
    pub listing: bool,
    #[serde(default = "default_true")]
    pub sale: bool,
    #[serde(default = "default_true")]
    pub afk: bool,
    #[serde(default = "default_true")]
    pub error: bool,
    #[serde(default = "default_true")]
    pub startup: bool,
}

impl Default for WebhookEvents {
    fn default() -> Self {
        Self {
            purchase: true,
            listing: true,
            sale: true,
            afk: true,
            error: true,
            startup: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_display_name")]
    pub display_name: String,
    #[serde(default)]
    pub events: WebhookEvents,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            display_name: default_display_name(),
            events: WebhookEvents::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default = "default_auth")]
    pub auth: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_max_buy_price", rename = "maxBuyPrice")]
    pub max_buy_price: u32,
    #[serde(default = "default_sell_price", rename = "sellPrice")]
    pub sell_price: String,
    #[serde(default = "default_max_listings_per_cycle", rename = "maxListingsPerCycle")]
    pub max_listings_per_cycle: u32,
    #[serde(default = "default_delay_between_cycles", rename = "delayBetweenCycles")]
    pub delay_between_cycles: u64,
    #[serde(default = "default_delay_after_join", rename = "delayAfterJoin")]
    pub delay_after_join: u64,
    #[serde(default = "default_window_timeout", rename = "windowTimeout")]
    pub window_timeout: u64,
    #[serde(default, rename = "debugEvents")]
    pub debug_events: bool,
    #[serde(default)]
    pub webhook: WebhookConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Path::new("config.json");
        
        if config_path.exists() {
            let config_data = fs::read_to_string(config_path)
                .context("Failed to read config.json")?;
            let mut config: Config = serde_json::from_str(&config_data)
                .context("Failed to parse config.json")?;
            
            // Override with environment variables if set
            if let Ok(username) = std::env::var("BOT_USERNAME") {
                config.username = username;
            }
            if let Ok(auth) = std::env::var("BOT_AUTH") {
                config.auth = auth;
            }
            if let Ok(max_buy) = std::env::var("MAX_BUY_PRICE") {
                if let Ok(price) = max_buy.parse() {
                    config.max_buy_price = price;
                }
            }
            if let Ok(sell) = std::env::var("SELL_PRICE") {
                config.sell_price = sell;
            }
            if let Ok(max_listings) = std::env::var("MAX_LISTINGS_PER_CYCLE") {
                if let Ok(limit) = max_listings.parse() {
                    config.max_listings_per_cycle = limit;
                }
            }
            if let Ok(delay) = std::env::var("DELAY_BETWEEN_CYCLES") {
                if let Ok(d) = delay.parse() {
                    config.delay_between_cycles = d;
                }
            }
            if let Ok(delay) = std::env::var("DELAY_AFTER_JOIN") {
                if let Ok(d) = delay.parse() {
                    config.delay_after_join = d;
                }
            }
            
            Ok(config)
        } else {
            println!("[CONFIG] No config.json found, using environment variables and defaults");
            Ok(Config::from_env())
        }
    }
    
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("BOT_HOST").unwrap_or_else(|_| default_host()),
            port: std::env::var("BOT_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(default_port),
            username: std::env::var("BOT_USERNAME").unwrap_or_else(|_| default_username()),
            auth: std::env::var("BOT_AUTH").unwrap_or_else(|_| default_auth()),
            version: std::env::var("BOT_VERSION").unwrap_or_else(|_| default_version()),
            max_buy_price: std::env::var("MAX_BUY_PRICE")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(default_max_buy_price),
            sell_price: std::env::var("SELL_PRICE").unwrap_or_else(|_| default_sell_price()),
            max_listings_per_cycle: std::env::var("MAX_LISTINGS_PER_CYCLE")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(default_max_listings_per_cycle),
            delay_between_cycles: std::env::var("DELAY_BETWEEN_CYCLES")
                .ok()
                .and_then(|d| d.parse().ok())
                .unwrap_or_else(default_delay_between_cycles),
            delay_after_join: std::env::var("DELAY_AFTER_JOIN")
                .ok()
                .and_then(|d| d.parse().ok())
                .unwrap_or_else(default_delay_after_join),
            window_timeout: std::env::var("WINDOW_TIMEOUT")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or_else(default_window_timeout),
            debug_events: std::env::var("DEBUG_EVENTS")
                .map(|v| v == "true")
                .unwrap_or(false),
            webhook: WebhookConfig::default(),
        }
    }
}

fn default_true() -> bool { true }
fn default_host() -> String { "donutsmp.net".to_string() }
fn default_port() -> u16 { 25565 }
fn default_username() -> String { "BOT_USERNAME".to_string() }
fn default_auth() -> String { "microsoft".to_string() }
fn default_version() -> String { "1.21.11".to_string() }
fn default_max_buy_price() -> u32 { 2500 }
fn default_sell_price() -> String { "9.9k".to_string() }
fn default_max_listings_per_cycle() -> u32 { 20 }
fn default_delay_between_cycles() -> u64 { 5000 }
fn default_delay_after_join() -> u64 { 5000 }
fn default_window_timeout() -> u64 { 15000 }
fn default_display_name() -> String { "DonutSMP Map Flipper".to_string() }
