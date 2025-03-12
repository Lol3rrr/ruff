use std::time::Duration;

use tracing::Instrument;

#[derive(Debug, serde::Deserialize)]
struct Configuration {
    items: Vec<ruff::config::ConfigItem>,
    open_exchange_app: Option<String>,
}

const STEAM_LOADING: bool = false;
const BITSKINS_LOADING: bool = true;

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

    let config = runtime.block_on(async {
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

    let item_list_orig = std::sync::Arc::new(arc_swap::ArcSwap::from(std::sync::Arc::new(
        config
            .items
            .iter()
            .flat_map(|i| i.to_items())
            .collect::<Vec<_>>(),
    )));

    let registry = prometheus::Registry::new_custom(Some("ruff".to_string()), None).unwrap();

    let items = prometheus::GaugeVec::new(
        prometheus::Opts::new("items", "The Items being tracked"),
        &["item", "kind", "condition"],
    )
    .unwrap();
    registry.register(Box::new(items.clone())).unwrap();

    for item in config.items.iter().flat_map(|i| i.to_items().into_iter()) {
        items
            .with_label_values(&[&item.name, &item.kind, &item.condition])
            .set(1.0);
    }

    let metrics_collection = ruff::Metrics::new(&registry);

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
        runtime.spawn(ruff::openexchange::run(exchange_config, conversions_metric));
    }

    let app = axum::Router::new()
        .route("/metrics", axum::routing::get(metrics))
        .with_state(registry);

    if false {
        runtime.spawn(ruff::buff::gather(
            item_list_orig.clone(),
            metrics_collection.clone(),
        ));
    }

    if STEAM_LOADING {
        runtime.spawn(ruff::steam::gather(
            config.items.clone(),
            metrics_collection.clone(),
        ));
    }

    if let Ok(api_token) = std::env::var("CSFLOAT_API_TOKEN") {
        runtime.spawn(ruff::csfloat::gather(
            item_list_orig.clone(),
            metrics_collection.clone(),
            api_token,
        ));
    }

    if BITSKINS_LOADING {
        runtime.spawn(ruff::bitskins::gather(
            item_list_orig.clone(),
            metrics_collection.clone(),
        ));
    }

    if let Some((client_id, client_secret)) = std::env::var("SKINPORT_CLIENT_ID")
        .ok()
        .zip(std::env::var("SKINPORT_CLIENT_SECRET").ok())
    {
        runtime.spawn(ruff::skinport::gather(
            metrics_collection,
            client_id,
            client_secret,
        ));
    }

    // Use SIGHUP to dynamically reload configuration
    let item_list = item_list_orig.clone();
    runtime.spawn(
        async move {
            let mut signal_stream =
                match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Getting SIGHUP stream: {:?}", e);
                        return;
                    }
                };

            loop {
                match signal_stream.recv().await {
                    Some(_) => {}
                    None => {
                        tracing::error!("Signal Stream Stopped");
                        return;
                    }
                };

                // Add a small time delay to make sure that the new file will be read
                tokio::time::sleep(Duration::from_secs(1)).await;

                let res = match tokio::fs::read(&config_path).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("Could not read '{}' {:?}", config_path, e);
                        continue;
                    }
                };

                let config = match serde_yaml::from_slice::<Configuration>(&res) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("Parsing Configuration {:?}", e);
                        continue;
                    }
                };

                let new_items = config
                    .items
                    .iter()
                    .flat_map(|i| i.to_items())
                    .collect::<Vec<_>>();

                let previous_item_list = item_list.load_full();
                tracing::info!(
                    "Items {:?} -> {:?}",
                    previous_item_list.len(),
                    new_items.len()
                );

                item_list.store(std::sync::Arc::new(new_items));

                items.reset();
                for item in config.items.iter().flat_map(|i| i.to_items().into_iter()) {
                    items
                        .with_label_values(&[&item.name, &item.kind, &item.condition])
                        .set(1.0);
                }

                tracing::info!("Reloaded configuration");
            }
        }
        .instrument(tracing::info_span!("Dynamic Config loader")),
    );

    let addr = "0.0.0.0:80";

    tracing::info!("Starting to listen on {}", addr);

    runtime.block_on(async move {
        axum::Server::bind(&addr.parse().unwrap())
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
