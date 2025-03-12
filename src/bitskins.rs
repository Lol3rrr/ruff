use crate::config::Item;

#[derive(serde::Deserialize, Debug)]
struct Response {
    created_at: serde_json::Value,
    list: Vec<ListEntry>,
}

#[derive(serde::Deserialize, Debug)]
struct ListEntry {
    name: String,
    price_avg: u64,
    price_max: u64,
    price_min: u64,
    quantity: u64,
    skin_id: u64,
}

#[tracing::instrument(name = "bitskins", skip(items, metrics))]
pub async fn gather(
    items: std::sync::Arc<arc_swap::ArcSwap<Vec<Item<'static>>>>,
    metrics: crate::Metrics,
) {
    tracing::info!("Starting Bitskins collector");

    let client = reqwest::Client::new();

    loop {
        tracing::info!("Collecting data");

        let start_time = std::time::Instant::now();

        //  https://api.bitskins.com/market/skin/730
        let response = match client
            .get("https://api.bitskins.com/market/insell/730")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Sending Request: {:?}", e);
                return;
            }
        };

        let response = match response.json::<Response>().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Deserialize JSON {:?}", e);
                return;
            }
        };

        for priced in response.list {
            let _entered = tracing::info_span!("Item", ?priced).entered();

            let item = match crate::Item::try_from(priced.name.as_str()) {
                Ok(i) => i,
                Err(e) => {
                    tracing::warn!("Could not parse item: {:?}", e);
                    continue;
                }
            };

            metrics.set_price("bitskins", &item, priced.price_min as f64 / 1000.0);
            metrics.set_count("bitskins", &item, priced.quantity as f64);
        }

        let elapsed = start_time.elapsed();
        tracing::info!("Done, took {:?}", elapsed);

        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}
