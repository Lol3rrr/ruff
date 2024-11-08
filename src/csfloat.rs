use crate::config::Item;

use rand::{Rng, SeedableRng, seq::SliceRandom};
use std::time::Duration;
use tracing::Instrument;

mod data;

pub struct Client {
    pub req_client: reqwest::Client,
    token: String,
}

impl Client {
    pub fn new(api_token: impl Into<String>) -> Self {
        let token: String = api_token.into();

        Self {
            req_client: reqwest::Client::new(),
            token,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn load_listings(&mut self, item: &Item<'_>) -> Result<Vec<data::Listing>, ()> {
        tracing::debug!(?item, "Loading Item");

        let resp = self.req_client.get("https://csfloat.com/api/v1/listings").query(&[("sort_by", "lowest_price"), ("market_hash_name", &item.name)]).header("Authorization", &self.token).send().await.map_err(|e| ())?;

        if !resp.status().is_success() {
            tracing::error!("Error Response {:?}", resp);
            return Err(());
        }

        let data: data::ListingsResponse = resp.json().await
            .map_err(|e| {
                tracing::error!("Loading JSON {:?}", e);
                ()
            })?;

        Ok(data.data)
    }
}

#[tracing::instrument(skip(items, metrics, api_token))]
pub async fn gather(
    items: std::sync::Arc<arc_swap::ArcSwap<Vec<Item<'static>>>>,
    metrics: crate::Metrics,
    api_token: String,
) {
    let mut client = Client::new(api_token);

    let mut rng = rand::rngs::SmallRng::from_entropy();

    loop {
        tracing::info!("Loading Data");

        let start_time = std::time::Instant::now();

        let items = items.load();

        let shuffled = {
            let mut tmp: Vec<_> = (*items.as_ref()).clone();
            tmp.shuffle(&mut rng);
            tmp
        };

        for (i, item) in shuffled.iter().enumerate() {
            if item.kind == "case" {
                continue;
            }

            async {
                match client.load_listings(item).await {
                    Ok(listings) if listings.is_empty() => {
                        tracing::warn!("No listings found");
                    }
                    Ok(mut listings) => {
                        listings.sort_unstable_by_key(|l| l.price);
                        
                        let count = 5.min(listings.len());
                        let avg = listings.iter().take(count).map(|l| l.price).sum::<u64>() as f64 / count as f64;

                        let avg_price = avg / 100.0;

                        tracing::info!("Lowest price average: {:?}", avg_price);

                        metrics.sell_prices.with_label_values(&[&item.name, &item.kind, &item.condition, "csfloat"]).set(avg_price);
                    }
                    Err(e) => {
                        tracing::error!("Loading Listings {:?}", e);
                    }
                };

                tokio::time::sleep(Duration::from_millis(rng.gen_range(500..1500))).await;
            }
            .instrument(tracing::info_span!(
                "Updating Item Stats",
                item = item.name,
                current = i + 1,
                total_items = shuffled.len()
            ))
            .await;
        }

        let elapsed = start_time.elapsed();
        tracing::info!("Updating stats took {:?}", elapsed);

        let unix_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap();
        metrics.last_update.set(unix_timestamp.as_secs() as f64);

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
