use std::time::Duration;

use tracing::Instrument;

#[derive(Debug, serde::Deserialize)]
struct Configuration {
    items: Vec<ruff::TargetItem>,
}

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
        &["item", "kind"],
    )
    .unwrap();
    registry.register(Box::new(sell_prices.clone())).unwrap();

    let buy_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("buy_orders", "The max Buy Order Price (in RMB)"),
        &["item", "kind"],
    )
    .unwrap();
    registry.register(Box::new(buy_prices.clone())).unwrap();

    let bought_at_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("bought_at", "The Prices at which the items were bought"),
        &["item", "kind"],
    )
    .unwrap();
    registry
        .register(Box::new(bought_at_prices.clone()))
        .unwrap();

    let app = axum::Router::new()
        .route("/metrics", axum::routing::get(metrics))
        .with_state(registry);

    runtime.spawn(async move {
        let mut client = ruff::Client::new();

        let items = config.items;

        loop {
            for item in &items {
                async {
                    let kind_str: &'static str = (&item.kind).into();

                    let labels = [&item.name, kind_str];

                    match client.load_buyorders(&item).await {
                        Ok(buy_order) => {
                            tracing::info!("Buy Order Summary {:?}", buy_order);

                            buy_prices.with_label_values(&labels).set(buy_order.max);
                        }
                        Err(e) => {
                            tracing::error!("Loading Buy Orders {:?}", e);
                        }
                    };

                    match client.load_sellorders(&item).await {
                        Ok(sell_order) => {
                            tracing::info!("Sell Order Summary {:?}", sell_order);

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
    });

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
