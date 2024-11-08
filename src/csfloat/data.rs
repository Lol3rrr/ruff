#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct PagedResponse<D> {
    pub cursor: Option<String>,
    pub data: D,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Listing {
    pub created_at: String,
    pub id: String,
    pub is_seller: bool,
    pub is_watchlisted: bool,
    pub item: serde_json::Value,
    pub max_offer_discount: Option<serde_json::Value>,
    pub min_offer_price: Option<serde_json::Value>,
    pub price: u64,
    pub reference: serde_json::Value,
    pub seller: serde_json::Value,
    pub state: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub watchers: serde_json::Value,

    #[serde(flatten)]
    pub remaining: std::collections::HashMap<String, serde_json::Value>,
}

pub type ListingsResponse = PagedResponse<Vec<Listing>>;

pub type PriceListResponse = Vec<PriceListEntry>;

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct PriceListEntry {
    pub market_hash_name: String,
    pub min_price: u64,
    pub qty: usize,
}

impl PriceListEntry {
    pub fn condition(&self) -> Option<&'static str> {
        let (last_brace_idx, _) = self.market_hash_name.char_indices().rev().find(|(_, c)| *c == '(')?;
        
        let trailing_stuff = &self.market_hash_name[last_brace_idx..];

        match trailing_stuff {
            "(Factory New)" => Some("factory-new"),
            "(Minimal Wear)" => Some("minimal-wear"),
            "(Field-Tested)" => Some("field-tested"),
            "(Well-Worn)" => Some("well-worn"),
            "(Battle-Scarred)" => Some("battle-scarred"),
            _ => None,
        }
    }

    pub fn is_souvenir(&self) -> bool {
        self.market_hash_name.contains("Souvenir")
    }

    pub fn is_stattrak(&self) -> bool {
        self.market_hash_name.contains("StatTrak™")
    }

    pub fn is_special(&self) -> bool {
        self.market_hash_name.contains("★")
    }

    pub fn name(&self) -> &str {
        let name = self.market_hash_name.as_str();
        let name = name.strip_prefix("Souvenir").unwrap_or(name).trim();
        let name = name.strip_prefix("★").unwrap_or(name).trim();
        let name = name.strip_prefix("StatTrak™").unwrap_or(name).trim();

        name
    }
}
