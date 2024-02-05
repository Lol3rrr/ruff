use serde::Deserialize;

pub struct Inventory {}

#[derive(Debug, Deserialize)]
struct InventoryResponse {
    assets: Vec<InventoryResponseAsset>,
    descriptions: Vec<InventoryResponseDescription>,
    success: isize,
    rwgrsn: isize,
    total_inventory_count: usize,
}

#[derive(Debug, Deserialize)]
struct InventoryResponseAsset {
    amount: String,
    appid: usize,
    assetid: String,
    classid: String,
    contextid: String,
    instanceid: String,
}

#[derive(Debug, Deserialize)]
struct InventoryResponseDescription {
    actions: Vec<serde_json::Value>,
    appid: usize,
    background_color: String,
    classid: String,
    commodity: usize,
    currency: usize,
    descriptions: Vec<serde_json::Value>,
    icon_url: String,
    icon_url_large: String,
    instanceid: String,
    market_actions: Vec<serde_json::Value>,
    market_hash_name: String,
    market_name: String,
    market_tradable_restriction: usize,
    marketable: usize,
    name: String,
    name_color: String,
    owner_descriptions: Vec<serde_json::Value>,
    tags: Vec<serde_json::Value>,
    tradable: usize,
    #[serde(rename = "type")]
    ty: String,
}

impl Inventory {
    pub async fn load(http_client: &reqwest::Client, user: &str) -> Result<(), ()> {
        let url = format!("https://steamcommunity.com/inventory/{user}/730/2?l=english&count=75");

        let resp = http_client
            .get(url)
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Accept", "*/*")
            .send()
            .await
            .map_err(|e| ())?;

        let status = resp.status();
        if !status.is_success() {
            dbg!(&status);
            return Err(());
        }

        dbg!(resp.text().await);

        /*
        let raw_content: serde_json::Value = resp.json().await.map_err(|e| {
            dbg!(e);
            ()
        })?;

        dbg!(&raw_content);
        */

        Ok(())
    }
}
