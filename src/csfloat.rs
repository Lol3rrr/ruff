use crate::config::Item;

use rand::{Rng, SeedableRng, seq::SliceRandom};
use std::time::Duration;
use tracing::Instrument;

mod data;

pub struct Client {
    pub req_client: reqwest::Client,
    token: String,
    req_remaining: usize,
    req_limit: usize,
    resets: Option<u64>,
}

#[derive(Debug, PartialEq)]
pub enum LoadListingError {
    RateLimited(RateLimit),
    Other(&'static str),
}

#[derive(Debug,PartialEq)]
struct RateLimit {
    remaining: usize,
    limit: usize,
    resets_at: chrono::DateTime<chrono::Utc>,
}

impl RateLimit {
    pub fn from_headers(headers: &reqwest::header::HeaderMap<reqwest::header::HeaderValue>) -> Result<Self, ()> {
        let remaining: usize = headers.get("x-ratelimit-remaining").and_then(|remaining| {
            let raw_v = remaining.to_str().ok()?;
            raw_v.parse().ok()
        }).ok_or_else(|| {
                tracing::error!("Missing 'x-ratelimit-remaining' Header");
                ()
            })?;

        let limit: usize = headers.get("x-ratelimit-limit").and_then(|remaining| {
            let raw_v = remaining.to_str().ok()?;
            raw_v.parse().ok()
        }).ok_or_else(|| {
                tracing::error!("Missing 'x-ratelimit-limit' Header");
                ()
            })?;

        let resets: i64 = headers.get("x-ratelimit-reset").and_then(|remaining| {
            let raw_v = remaining.to_str().ok()?;
            raw_v.parse().ok()
        }).ok_or_else(|| {
                tracing::error!("Missing 'x-ratelimit-reset' Header");
                ()
            })?;

        let resets_at = chrono::DateTime::from_timestamp(resets, 0).ok_or_else(|| {
            tracing::error!("Converting Timestamp to DateTime");
            ()
        })?;        

        Ok(RateLimit {
            remaining,
            limit,
            resets_at
        })

    }
}

impl Client {
    pub fn new(api_token: impl Into<String>) -> Self {
        let token: String = api_token.into();

        Self {
            req_client: reqwest::Client::new(),
            token,
            req_remaining: usize::MAX,
            req_limit: 0,
            resets: None,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn load_listings(&mut self, item: &Item<'_>) -> Result<Vec<data::Listing>, LoadListingError> {
        tracing::debug!(?item, "Loading Item");

        let name = if item.kind == "knife" {
            format!("★ {}", item.name)
        } else {
            item.name.clone()
        };

        let resp = self.req_client.get("https://csfloat.com/api/v1/listings").query(&[("sort_by", "lowest_price"), ("market_hash_name", &name)]).header("Authorization", &self.token).send().await.map_err(|e| LoadListingError::Other("Send Request"))?;

        let resp_headers = resp.headers();
        
        if let Some(remaining) = resp_headers.get("x-ratelimit-remaining") {
            self.req_remaining = remaining.to_str().unwrap().parse().unwrap();
        }
        if let Some(limit) = resp_headers.get("x-ratelimit-limit") {
            self.req_limit = limit.to_str().unwrap().parse().unwrap();
        }
        if let Some(resets) = resp_headers.get("x-ratelimit-reset") {
            let reset_timestamp: u64 = resets.to_str().unwrap().parse().unwrap();
            self.resets = Some(reset_timestamp);
        }

        match resp.status() {
            resp if resp.is_success() => {}
            resp if resp.as_u16() == 429 => {
                let rate_limit = RateLimit::from_headers(resp_headers).map_err(|e| LoadListingError::Other("Parsing RateLimit"))?;

                return Err(LoadListingError::RateLimited(rate_limit));
            }
            resp => {
                tracing::error!("Error Response {:?}", resp);
                return Err(LoadListingError::Other("Non Success status"));
            }
        };

        let data: data::ListingsResponse = resp.json().await
            .map_err(|e| {
                tracing::error!("Loading JSON {:?}", e);
                LoadListingError::Other("Deserialize")
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

    let mut between_items = std::time::Duration::from_millis((60 * 60 * 1000) / 200);

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

            let should_break = async {
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
                    Err(LoadListingError::RateLimited(rate_limit)) => {
                        tracing::error!("Reached RateLimit");
                        
                        let now = chrono::Local::now();
                        let native_utc = now.naive_utc();
                        let offset = now.offset().clone();

                        let now = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(native_utc, offset).to_utc();

                        let delta = rate_limit.resets_at - now;

                        let wait_time = delta.to_std().unwrap_or(std::time::Duration::from_secs(1));

                        between_items = std::time::Duration::from_millis((60 * 60 * 1000) / rate_limit.limit as u64);
                        tracing::info!("Updating Betwee-Items-Interval to {:?}", between_items);

                        tracing::warn!("Waiting out Rate-Limit by sleeping {:?}", wait_time);
                        tokio::time::sleep(wait_time).await;

                        return true;
                    }
                    Err(e) => {
                        tracing::error!("Loading Listings {:?}", e);
                    }
                };

                tokio::time::sleep(between_items.clone().checked_add(
                    std::time::Duration::from_millis(rng.gen_range(125..500))
                ).unwrap()).await;

                false
            }
            .instrument(tracing::info_span!(
                "Updating Item Stats",
                item = item.name,
                current = i + 1,
                total_items = shuffled.len()
            ))
            .await;

            if should_break {
                break;
            }
        }

        let elapsed = start_time.elapsed();
        tracing::info!("Updating stats took {:?}", elapsed);

        let unix_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap();
        metrics.last_update.set(unix_timestamp.as_secs() as f64);
    }
}
