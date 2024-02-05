use std::collections::HashMap;

pub struct Config {
    app_id: String,
}

#[derive(Debug)]
pub struct Conversions {
    pub rmb_to_euro: f64,
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
        })
    }
}
