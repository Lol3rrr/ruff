mod data;

struct Client {
    req_client: reqwest::Client,
    client_id: String,
    client_secret: String,
}

#[derive(Debug)]
enum LoadError {
    SendRequest(reqwest::Error),
    RateLimited { retry_after: Option<u64> },
    DeserializeResponse(reqwest::Error),
    Other(&'static str),
}

impl Client {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            req_client: reqwest::Client::new(),
            client_id,
            client_secret,
        }
    }

    pub async fn load_items(&self) -> Result<data::ItemsResponse, LoadError> {
        let resp = self
            .req_client
            .get("https://api.skinport.com/v1/items")
            .query(&[("app_id", "730"), ("currency", "USD")])
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .send()
            .await
            .map_err(|e| LoadError::SendRequest(e))?;

        if resp.status().as_u16() == 429 {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .map(|h| h.to_str().ok())
                .flatten()
                .map(|v| v.parse::<u64>().ok())
                .flatten();
            return Err(LoadError::RateLimited { retry_after });
        }

        if !resp.status().is_success() {
            tracing::debug!("Non-200 response: {:?}", resp);
            return Err(LoadError::Other("Unsuccessful response"));
        }

        resp.json::<data::ItemsResponse>()
            .await
            .map_err(|e| LoadError::DeserializeResponse(e))
    }
}

#[tracing::instrument(skip(metrics, client_id, client_secret))]
pub async fn gather(metrics: crate::Metrics, client_id: String, client_secret: String) {
    let client = Client::new(client_id, client_secret);

    loop {
        tracing::info!("Loading Data");

        match client.load_items().await {
            Ok(items) => {
                for priced in items {
                    let item = match crate::Item::try_from(priced.market_hash_name.as_str()) {
                        Ok(i) => i,
                        Err(e) => {
                            tracing::error!("Parsing Item from List: {:?}", priced);
                            continue;
                        }
                    };

                    if let Some(price) = priced.min_price {
                        metrics.set_price("skinport", &item, price);
                    }

                    metrics.set_count("skinport", &item, priced.quantity as f64);
                }
            }
            Err(LoadError::RateLimited { retry_after }) => {
                tracing::error!("Being rate-limited");

                if let Some(wait_time) = retry_after {
                    let wait_dur = std::time::Duration::from_secs(wait_time);
                    let diff_dur = wait_dur
                        .checked_sub(std::time::Duration::from_secs(60 * 5))
                        .unwrap_or(std::time::Duration::from_secs(1));

                    tracing::warn!("Sleeping for {:?} before retrying", diff_dur);
                    tokio::time::sleep(diff_dur).await;
                }
            }
            Err(e) => {
                tracing::error!("Loading Items: {:?}", e);
            }
        };

        // The endpoint we use is cached every 5 minutes anyway, so polling from it every 5 minutes
        // seems fine
        tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
    }
}
