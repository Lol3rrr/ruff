mod data;

struct Client {
    req_client: reqwest::Client,
    api_token: String,
}

impl Client {
    pub fn new(token: String) -> Self {
        Self {
            req_client: reqwest::Client::new(),
            api_token: token,
        }
    }

    pub async fn load_items(&self) -> Result<data::ItemsResponse, ()> {
        let resp = self
            .req_client
            .get("https://api.skinport.com/v1/items")
            .query(&[("app_id", "730"), ("currency", "USD")])
            .send()
            .await
            .map_err(|e| ())?;

        if !resp.status().is_success() {
            dbg!(resp);
            return Err(());
        }

        resp.json::<data::ItemsResponse>().await.map_err(|e| {
            dbg!(e);
            ()
        })
    }
}

#[tracing::instrument(skip(metrics, api_token))]
pub async fn gather(metrics: crate::Metrics, api_token: String) {
    let client = Client::new(api_token);

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
            Err(e) => {
                tracing::error!("Loading Items: {:?}", e);
            }
        };

        // The endpoint we use is cached every 5 minutes anyway, so polling from it every 5 minutes
        // seems fine
        tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
    }
}
