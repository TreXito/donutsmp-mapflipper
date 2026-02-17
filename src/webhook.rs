use crate::config::Config;
use serde_json::json;
use anyhow::Result;

pub async fn send_webhook(
    config: &Config,
    event: &str,
    message: &str,
    color: u32,
    fields: Vec<(String, String, bool)>,
) -> Result<()> {
    // Log webhook URL validation
    if !config.webhook.enabled {
        println!("[WEBHOOK] Webhooks disabled in config");
        return Ok(());
    }
    
    if config.webhook.url.is_empty() {
        println!("[WEBHOOK] ERROR: Webhook URL is empty! Please set webhook.url in config.json");
        return Ok(());
    }
    
    println!("[WEBHOOK] Using webhook URL: {}", 
        if config.webhook.url.len() > 60 {
            format!("{}...", &config.webhook.url[..60])
        } else {
            config.webhook.url.clone()
        }
    );
    
    // Check if this event type is enabled
    let event_enabled = match event {
        "purchase" => config.webhook.events.purchase,
        "listing" => config.webhook.events.listing,
        "sale" => config.webhook.events.sale,
        "afk" => config.webhook.events.afk,
        "error" => config.webhook.events.error,
        "startup" => config.webhook.events.startup,
        "shards" => true, // Always allow shards tracking
        _ => false,
    };
    
    if !event_enabled {
        println!("[WEBHOOK] Event type '{}' is disabled in config", event);
        return Ok(());
    }
    
    // Build embed fields ensuring all values are strings
    let embed_fields: Vec<_> = fields
        .iter()
        .map(|(name, value, inline)| {
            json!({
                "name": name,
                "value": value,  // Already a String from the Vec parameter
                "inline": inline
            })
        })
        .collect();
    
    // Build payload with proper Discord webhook format
    // embeds MUST be an array of objects
    // color MUST be an integer
    // field values MUST be strings
    let payload = json!({
        "username": config.webhook.display_name,
        "embeds": [{
            "title": format!("{} Event", capitalize_first(event)),
            "description": message,
            "color": color,  // Already a u32 integer
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "fields": embed_fields
        }]
    });
    
    // Log the full request body before sending
    println!("[WEBHOOK] Request body: {}", serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "Failed to serialize".to_string()));
    
    let client = reqwest::Client::new();
    let response = client
        .post(&config.webhook.url)
        .header("Content-Type", "application/json")  // Ensure correct Content-Type header
        .json(&payload)
        .send()
        .await?;
    
    // Log HTTP response status AND body
    let status = response.status();
    let response_body = response.text().await?;
    
    println!("[WEBHOOK] Response status: {}", status);
    println!("[WEBHOOK] Response body: {}", response_body);
    
    if !status.is_success() {
        eprintln!("[WEBHOOK] Failed to send webhook (status {}): {}", status, response_body);
        return Err(anyhow::anyhow!("Webhook request failed with status {}", status));
    } else {
        println!("[WEBHOOK] ✓ Successfully sent {} webhook", event);
    }
    
    Ok(())
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
