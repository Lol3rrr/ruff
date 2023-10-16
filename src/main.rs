use std::time::Duration;

use tracing::Instrument;

#[derive(Debug, serde::Deserialize)]
struct Configuration {
    items: Vec<ruff::TargetItem>,
}

fn main() {
    let subscriber = tracing_subscriber::fmt().with_ansi(false).finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    tracing::info!("Starting");

    tracing::info!("Loading Configuration...");

    let config = runtime.block_on(async move {
        let res = match tokio::fs::read("./config.yaml").await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Could not read './config.yaml' {:?}", e);
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

    tracing::info!("Loaded Configuration: {:#?}", config);

    let registry = prometheus::Registry::new();

    let sell_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("buff_sell_prices", "The minimum Sell Price (in RMB)"),
        &["item", "kind"],
    )
    .unwrap();
    registry.register(Box::new(sell_prices.clone())).unwrap();

    let buy_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("buff_buy_orders", "The max Buy Order Price (in RMB)"),
        &["item", "kind"],
    )
    .unwrap();
    registry.register(Box::new(buy_prices.clone())).unwrap();

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

                    match client.load_buyorders(&item).await {
                        Ok(buy_order) => {
                            tracing::info!("Buy Order Summary {:?}", buy_order);

                            buy_prices
                                .with_label_values(&[&item.name, kind_str])
                                .set(buy_order.max);
                        }
                        Err(e) => {
                            tracing::error!("Loading Buy Orders {:?}", e);
                        }
                    };

                    match client.load_sellorders(&item).await {
                        Ok(sell_order) => {
                            tracing::info!("Sell Order Summary {:?}", sell_order);

                            sell_prices
                                .with_label_values(&[&item.name, kind_str])
                                .set(sell_order.min);
                        }
                        Err(e) => {
                            tracing::error!("Loading Sell Orders {:?}", e);
                        }
                    };
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
