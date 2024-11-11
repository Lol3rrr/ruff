mod data;

struct Client {
    req_client: reqwest::Client,
    client_id: String,
    client_secret: String,
}

impl Client {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            req_client: reqwest::Client::new(),
            client_id,
            client_secret
        }
    }

    pub async fn load_items(&self) -> Result<data::ItemsResponse, ()> {
        let resp = self
            .req_client
            .get("https://api.skinport.com/v1/items")
            .query(&[("app_id", "730"), ("currency", "USD")])
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .send()
            .await
            .map_err(|e| ())?;

        if resp.status().as_u16() == 429 {
            tracing::error!("Got rate-limited");
            return Err(());
        }

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
            Err(e) => {
                tracing::error!("Loading Items: {:?}", e);
            }
        };

        // The endpoint we use is cached every 5 minutes anyway, so polling from it every 5 minutes
        // seems fine
        tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
    }
}
