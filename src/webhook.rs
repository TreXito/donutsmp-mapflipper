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
    if !config.webhook.enabled || config.webhook.url.is_empty() {
        return Ok(());
    }
    
    // Check if this event type is enabled
    let event_enabled = match event {
        "purchase" => config.webhook.events.purchase,
        "listing" => config.webhook.events.listing,
        "sale" => config.webhook.events.sale,
        "afk" => config.webhook.events.afk,
        "error" => config.webhook.events.error,
        "startup" => config.webhook.events.startup,
        _ => false,
    };
    
    if !event_enabled {
        return Ok(());
    }
    
    let embed_fields: Vec<_> = fields
        .iter()
        .map(|(name, value, inline)| {
            json!({
                "name": name,
                "value": value,
                "inline": inline
            })
        })
        .collect();
    
    let payload = json!({
        "username": config.webhook.display_name,
        "embeds": [{
            "title": format!("{} Event", capitalize_first(event)),
            "description": message,
            "color": color,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "fields": embed_fields
        }]
    });
    
    let client = reqwest::Client::new();
    let response = client
        .post(&config.webhook.url)
        .json(&payload)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await?;
        eprintln!("[WEBHOOK] Failed to send webhook (status {}): {}", status, text);
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
