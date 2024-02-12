use std::time::Duration;

use serde::Deserialize;

mod inventory;
pub use inventory::Inventory;
use tracing::Instrument;

#[derive(Debug, Deserialize)]
pub struct ItemOrderHistogram {
    success: serde_json::Value,
    sell_order_table: String,
    sell_order_summary: String,
    buy_order_table: String,
    buy_order_summary: String,
    highest_buy_order: serde_json::Value,
    lowest_sell_order: serde_json::Value,
    buy_order_graph: Vec<Vec<serde_json::Value>>,
    sell_order_graph: Vec<Vec<serde_json::Value>>,
    graph_max_y: f64,
    graph_min_x: f64,
    graph_max_x: f64,
    price_prefix: String,
    price_suffix: String,
}

#[derive(Debug, Deserialize)]
pub struct PriceOverview {
    success: bool,
    lowest_price: String,
}

pub async fn load_item(item: &str, client: &reqwest::Client) {
    let url = format!(
        "https://steamcommunity.com/market/priceoverview/?appid=730&currency=3&market_hash_name={}",
        item
    );

    let resp = match client.get(&url).header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7").send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Sending Request: {:?}", e);
            return;
        }
    };

    let status = resp.status();
    if !status.is_success() {
        tracing::error!("Non Success Response: {:?}", status);
        return;
    }

    let content: PriceOverview = match resp.json().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Deserializing {:?}", e);
            return;
        }
    };

    tracing::info!("Loaded Histogram: {:#?}", content);
}

#[tracing::instrument(skip(items, metrics))]
pub async fn gather(items: Vec<crate::config::ConfigItem>, metrics: crate::Metrics) {
    let mut client = crate::buff::Client::new();

    loop {
        for item in &items {
            async {
                let tmp = crate::steam::load_item(&item.name, &client.req_client).await;

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            .instrument(tracing::info_span!("Updating Item Stats", ?item))
            .await;
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
