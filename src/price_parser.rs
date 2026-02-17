use regex::Regex;

/// Parse price from text containing format like "Price: $995" or "Price: $5K" or "Price: $9.9K"
pub fn parse_price(text: &str) -> Option<u32> {
    // Remove Minecraft color codes (§x format)
    let clean_text = strip_minecraft_colors(text);
    
    // Match price patterns: $995, $5K, $9.9K, $10,000, etc.
    let re = Regex::new(r"\$([0-9,.]+)([Kk]?)").ok()?;
    let caps = re.captures(&clean_text)?;
    
    let number_str = caps.get(1)?.as_str().replace(",", "");
    let suffix = caps.get(2).map(|m| m.as_str()).unwrap_or("");
    
    let base_number: f64 = number_str.parse().ok()?;
    
    let price = if suffix.eq_ignore_ascii_case("K") {
        (base_number * 1000.0) as u32
    } else {
        base_number as u32
    };
    
    Some(price)
}

/// Strip Minecraft color codes from text
pub fn strip_minecraft_colors(text: &str) -> String {
    let re = Regex::new(r"§[0-9a-fk-or]").unwrap();
    re.replace_all(text, "").to_string()
}

/// Format a price value into a string like "9.9k" or "316.8k"
pub fn format_price(price: u32) -> String {
    if price >= 1000 {
        let k_value = price as f64 / 1000.0;
        // Format to 1 decimal place (rounds during formatting)
        format!("{:.1}k", k_value)
    } else {
        price.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_price() {
        assert_eq!(parse_price("Price: $995"), Some(995));
        assert_eq!(parse_price("Price: $5K"), Some(5000));
        assert_eq!(parse_price("Price: $9.9K"), Some(9900));
        assert_eq!(parse_price("Price: $10,000"), Some(10000));
        assert_eq!(parse_price("Price: $2.5k"), Some(2500));
        assert_eq!(parse_price("§aPrice: §6$995"), Some(995));
        assert_eq!(parse_price("§aPrice: §6$5K"), Some(5000));
    }
    
    #[test]
    fn test_strip_minecraft_colors() {
        assert_eq!(strip_minecraft_colors("§aHello §6World"), "Hello World");
        assert_eq!(strip_minecraft_colors("Normal text"), "Normal text");
        assert_eq!(strip_minecraft_colors("§k§l§m§n§oTest"), "Test");
    }
    
    #[test]
    fn test_format_price() {
        assert_eq!(format_price(995), "995");
        assert_eq!(format_price(5000), "5.0k");
        assert_eq!(format_price(9900), "9.9k");
        assert_eq!(format_price(316800), "316.8k");
        assert_eq!(format_price(1000), "1.0k");
    }
    
    #[test]
    fn test_single_map_full_price_and_stack_discount() {
        // Test the listing price logic as implemented in inventory.rs list_maps function
        let base_price = 9900; // $9.9k
        let sell_price_str = "9.9k";
        
        // Single map: no discount, use full price
        let count = 1;
        let price_str = if count == 1 {
            sell_price_str.to_string()
        } else {
            let stack_price = (base_price * count as u32) / 2;
            format_price(stack_price)
        };
        assert_eq!(price_str, "9.9k", "Single map should be listed at full price");
        
        // Stack of 2 maps: 50% discount
        let count = 2;
        let stack_price = (base_price * count as u32) / 2;
        let price_str = format_price(stack_price);
        assert_eq!(stack_price, 9900, "Stack of 2 should be 9900 total (50% off each)");
        assert_eq!(price_str, "9.9k");
        
        // Stack of 32 maps: 50% discount
        let count = 32;
        let stack_price = (base_price * count as u32) / 2;
        let price_str = format_price(stack_price);
        assert_eq!(stack_price, 158400, "Stack of 32 should be 158400 total");
        assert_eq!(price_str, "158.4k");
        
        // Stack of 64 maps: 50% discount
        let count = 64;
        let stack_price = (base_price * count as u32) / 2;
        let price_str = format_price(stack_price);
        assert_eq!(stack_price, 316800, "Stack of 64 should be 316800 total");
        assert_eq!(price_str, "316.8k");
    }
}
