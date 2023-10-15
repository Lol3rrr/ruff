use std::time::Duration;

use tracing::Instrument;

fn main() {
    let subscriber = tracing_subscriber::fmt().with_ansi(false).finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    tracing::info!("Starting");

    let registry = prometheus::Registry::new();

    let sell_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("buff_sell_prices", "The minimum Sell Price (in RMB)"),
        &["item"],
    )
    .unwrap();
    registry.register(Box::new(sell_prices.clone())).unwrap();

    let buy_prices = prometheus::GaugeVec::new(
        prometheus::Opts::new("buff_buy_orders", "The max Buy Order Price (in RMB)"),
        &["item"],
    )
    .unwrap();
    registry.register(Box::new(buy_prices.clone())).unwrap();

    let app = axum::Router::new()
        .route("/metrics", axum::routing::get(metrics))
        .with_state(registry);

    runtime.spawn(async move {
        let mut client = ruff::Client::new();

        let items = [
            ruff::TargetItem {
                name: "Danger Zone Case".to_string(),
                goods_id: 763236,
            },
            ruff::TargetItem {
                name: "Butterfly Knife | Marble Fade (Factory New)".to_string(),
                goods_id: 42563,
            },
            ruff::TargetItem {
                name: "AK-47 | Vulcan (Field-Tested)".to_string(),
                goods_id: 33975,
            },
        ];

        loop {
            for item in &items {
                async {
                    match client.load_buyorders(&item).await {
                        Ok(buy_order) => {
                            tracing::info!("Buy Order Summary {:?}", buy_order);

                            buy_prices
                                .with_label_values(&[&item.name])
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
                                .with_label_values(&[&item.name])
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
