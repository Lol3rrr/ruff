#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct PagedResponse<D>{
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
