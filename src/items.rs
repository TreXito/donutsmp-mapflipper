use crate::price_parser::{parse_price, strip_minecraft_colors};

/// Extract lore text from an item
/// In Minecraft 1.21.1+, lore is stored in components, not NBT
/// 
/// TODO: This needs to be implemented once we understand Azalea's item structure
/// The item parameter type needs to match whatever Azalea uses for inventory items
pub fn extract_lore(_item: &()) -> Vec<String> {
    // For now, return empty vec as we need to understand Azalea's item structure
    // This will be implemented once we can inspect the actual item data
    vec![]
}

/// Parse price and seller from item lore
pub fn parse_item_info(lore: &[String]) -> Option<(u32, String)> {
    let mut price = None;
    let mut seller = String::from("unknown");
    
    for line in lore {
        // Look for price
        if line.contains("Price:") {
            if let Some(p) = parse_price(line) {
                price = Some(p);
            }
        }
        
        // Look for seller
        if line.contains("Seller:") {
            let clean = strip_minecraft_colors(line);
            if let Some(seller_match) = clean.strip_prefix("Seller:") {
                seller = seller_match.trim().to_string();
            }
        }
    }
    
    price.map(|p| (p, seller))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_item_info() {
        let lore = vec![
            "§7Price: §6$1500".to_string(),
            "§7Seller: §aTestPlayer".to_string(),
        ];
        
        let result = parse_item_info(&lore);
        assert!(result.is_some());
        
        let (price, seller) = result.unwrap();
        assert_eq!(price, 1500);
        assert_eq!(seller, "TestPlayer");
    }
}
