use std::{collections::HashMap, time::Duration};

pub struct Config {
    app_id: String,
}

#[derive(Debug)]
pub struct Conversions {
    pub rmb_to_euro: f64,
    pub usd_to_euro: f64,
}

#[derive(Debug, serde::Deserialize)]
struct LatestResponse {
    base: String,
    disclaimer: String,
    license: String,
    rates: Rates,
    timestamp: u128,
}

#[derive(Debug, serde::Deserialize)]
struct Rates {
    EUR: f64,
    CNY: f64,
    USD: f64,
    #[serde(flatten)]
    others: HashMap<String, f64>,
}

impl Config {
    pub fn new(app_id: String) -> Self {
        Self { app_id }
    }

    pub async fn load_rates(&self, http_client: &reqwest::Client) -> Result<Conversions, ()> {
        let url = format!(
            "https://openexchangerates.org/api/latest.json?app_id={}",
            self.app_id
        );

        let requested = http_client.get(url).send().await.map_err(|e| ())?;

        let status = requested.status();
        if !status.is_success() {
            return Err(());
        }

        let content: LatestResponse = requested.json().await.map_err(|e| ())?;

        Ok(Conversions {
            rmb_to_euro: content.rates.EUR / content.rates.CNY,
            usd_to_euro: content.rates.EUR / content.rates.USD,
        })
    }
}

pub async fn run(config: Config, conversions_metric: prometheus::GaugeVec) -> ! {
    let client = reqwest::Client::new();

    loop {
        match config.load_rates(&client).await {
            Ok(conversion_rates) => {
                tracing::info!("Conversion-Rates: {:#?}", conversion_rates);

                conversions_metric
                    .with_label_values(&["CNY", "EUR"])
                    .set(conversion_rates.rmb_to_euro);
                conversions_metric
                    .with_label_values(&["USD", "EUR"])
                    .set(conversion_rates.usd_to_euro);
            }
            Err(e) => {
                tracing::error!("Loading Conversion-Rates: {:?}", e);
            }
        };

        tokio::time::sleep(Duration::from_secs(60 * 60 * 24)).await;
    }
}
