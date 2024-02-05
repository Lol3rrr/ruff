use std::{ascii::escape_default, time::Duration};

use tracing::Instrument;

#[derive(Debug, serde::Deserialize)]
struct Configuration {
    items: Vec<ruff::ConfigItem>,
    open_exchange_app: Option<String>,
}

const STEAM_LOADING: bool = false;

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        //.with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    tracing::info!("Starting");

    let config_path = std::env::var("CONFIG_PATH").unwrap_or("./config.yaml".to_string());

    tracing::info!("Loading Configuration from '{}'...", config_path);

    let config = runtime.block_on(async move {
        let res = match tokio::fs::read(&config_path).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Could not read '{}' {:?}", config_path, e);
                panic!();
            }
        };

        match serde_yaml::from_slice::<Configuration>(&res) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Parsing Configuration {:?}", e);
                panic!();
            }
        }
    });

    let registry = prometheus::Registry::new_custom(Some("buff".to_string()), None).unwrap();

    let items = prometheus::GaugeVec::new(
        prometheus::Opts::new("items", "The Items being tracked"),
        &["item", "kind"],
    )
    .unwrap();
    registry.register(Box::new(items.clone())).unwrap();

    for item in config.items.iter() {
        let kind: &str = (&item.kind).into();
        items.with_label_values(&[&item.name, kind]).set(1.0);
    }

    let sell_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("sell_prices", "The minimum Sell Price (in RMB)"),
        &["item", "kind", "condition"],
    )
    .unwrap();
    registry.register(Box::new(sell_prices.clone())).unwrap();

    let buy_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("buy_orders", "The max Buy Order Price (in RMB)"),
        &["item", "kind", "condition"],
    )
    .unwrap();
    registry.register(Box::new(buy_prices.clone())).unwrap();

    let bought_at_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("bought_at", "The Prices at which the items were bought"),
        &["item", "kind", "condition"],
    )
    .unwrap();
    registry
        .register(Box::new(bought_at_prices.clone()))
        .unwrap();

    let conversions_metric = prometheus::GaugeVec::new(
        prometheus::Opts::new("conversions", "The Conversion Rates"),
        &["from", "to"],
    )
    .unwrap();
    registry
        .register(Box::new(conversions_metric.clone()))
        .unwrap();

    if let Some(app_id) = config.open_exchange_app {
        let exchange_config = ruff::openexchange::Config::new(app_id);

        runtime.spawn(async move {
            let client = reqwest::Client::new();

            loop {
                match exchange_config.load_rates(&client).await {
                    Ok(conversion_rates) => {
                        tracing::info!("Conversion-Rates: {:#?}", conversion_rates);

                        conversions_metric
                            .with_label_values(&["CNY", "EUR"])
                            .set(conversion_rates.rmb_to_euro);
                    }
                    Err(e) => {
                        tracing::error!("Loading Conversion-Rates: {:?}", e);
                    }
                };

                tokio::time::sleep(Duration::from_secs(60 * 60 * 24)).await;
            }
        });
    }

    let app = axum::Router::new()
        .route("/metrics", axum::routing::get(metrics))
        .with_state(registry);

    runtime.spawn(gather_buff(
        config.items.clone(),
        buy_prices.clone(),
        sell_prices.clone(),
        bought_at_prices.clone(),
    ));

    if STEAM_LOADING {
        runtime.spawn(gather_steam(
            config.items.clone(),
            buy_prices,
            sell_prices,
            bought_at_prices,
        ));
    }

    tracing::info!("Starting to listen on 0.0.0.0:80");

    runtime.block_on(async move {
        axum::Server::bind(&"0.0.0.0:80".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}

#[tracing::instrument(skip(registry))]
async fn metrics(
    axum::extract::State(registry): axum::extract::State<prometheus::Registry>,
) -> String {
    tracing::trace!("Getting metrics");

    let encoder = prometheus::TextEncoder::new();
    let metrics_families = registry.gather();
    match encoder.encode_to_string(&metrics_families) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Encoding Metrics {:?}", e);

            String::new()
        }
    }
}

#[tracing::instrument(skip(items, buy_prices, sell_prices, bought_at_prices))]
async fn gather_buff(
    items: Vec<ruff::ConfigItem>,
    buy_prices: prometheus::GaugeVec,
    sell_prices: prometheus::GaugeVec,
    bought_at_prices: prometheus::GaugeVec,
) {
    let mut client = ruff::buff::Client::new();

    loop {
        tracing::info!("Loading Buff Data");

        for item in &items {
            async {
                let kind_str: &'static str = (&item.kind).into();
                let condition_str: &'static str =
                    item.condition.as_ref().map(|c| c.into()).unwrap_or("");

                let labels = [&item.name, kind_str, condition_str];

                match client.load_buyorders(&item).await {
                    Ok(buy_order) => {
                        tracing::info!("Buy Order Summary {:?}", buy_order,);

                        buy_prices.with_label_values(&labels).set(buy_order.max);
                    }
                    Err(e) => {
                        tracing::error!("Loading Buy Orders {:?}", e);
                    }
                };

                match client.load_sellorders(&item).await {
                    Ok(sell_order) => {
                        tracing::info!("Sell Order Summary {:?}", sell_order,);

                        sell_prices.with_label_values(&labels).set(sell_order.min);
                    }
                    Err(e) => {
                        tracing::error!("Loading Sell Orders {:?}", e);
                    }
                };

                if let Some(bought_price) = item.bought_at.as_ref() {
                    bought_at_prices
                        .with_label_values(&labels)
                        .set(*bought_price);
                }
            }
            .instrument(tracing::info_span!("Updating Item Stats", ?item))
            .await;
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

#[tracing::instrument(skip(items, buy_prices, sell_prices, bought_at_prices))]
async fn gather_steam(
    items: Vec<ruff::ConfigItem>,
    buy_prices: prometheus::GaugeVec,
    sell_prices: prometheus::GaugeVec,
    bought_at_prices: prometheus::GaugeVec,
) {
    let mut client = ruff::buff::Client::new();

    loop {
        for item in &items {
            async {
                let tmp = ruff::steam::load_item(&item.name, &client.req_client).await;

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            .instrument(tracing::info_span!("Updating Item Stats", ?item))
            .await;
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
