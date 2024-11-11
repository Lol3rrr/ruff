pub type ItemsResponse = Vec<ItemEntry>;

/*
Object {
        "created_at": Number(1667040400),
        "currency": String("USD"),
        "item_page": String("https://skinport.com/item/sticker-syrson-glitter-rio-2022"),
        "market_hash_name": String("Sticker | syrsoN (Glitter) | Rio 2022"),
        "market_page": String("https://skinport.com/market?item=syrsoN%20(Glitter)%20%7C%20Rio%202022&cat=Sticker"),
        "max_price": Number(0.1),
        "mean_price": Number(0.1),
        "median_price": Number(0.1),
        "min_price": Number(0.1),
        "quantity": Number(1),
        "suggested_price": Number(0.08),
        "updated_at": Number(1731283259),
    },
*/
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct ItemEntry {
    pub created_at: serde_json::Value,
    pub currency: String,
    pub item_page: String,
    pub market_hash_name: String,
    pub market_page: String,
    pub max_price: Option<f64>,
    pub mean_price: Option<f64>,
    pub median_price: Option<f64>,
    pub min_price: Option<f64>,
    pub quantity: usize,
    pub suggested_price: Option<f64>,
    pub updated_at: serde_json::Value,
}
