pub mod buff;
pub mod openexchange;
pub mod steam;

pub mod config;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub buy_prices: prometheus::GaugeVec,
    pub buy_counts: prometheus::GaugeVec,
    pub buy_listings: prometheus::GaugeVec,
    pub sell_prices: prometheus::GaugeVec,
    pub bought_at_prices: prometheus::GaugeVec,
    pub last_update: prometheus::Gauge,
}

impl Metrics {
    pub fn new(registry: &prometheus::Registry) -> Self {
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

        let buy_counts = prometheus::GaugeVec::new(
            prometheus::Opts::new(
                "buy_counts",
                "The number of items that can be bought at the max Buy Order Price",
            ),
            &["item", "kind", "condition"],
        )
        .unwrap();
        registry.register(Box::new(buy_counts.clone())).unwrap();

        let buy_listings = prometheus::GaugeVec::new(
            prometheus::Opts::new(
                "buy_listings",
                "The number of listings that buy at the max Buy Order Price",
            ),
            &["item", "kind", "condition"],
        )
        .unwrap();
        registry.register(Box::new(buy_listings.clone())).unwrap();

        let bought_at_prices = prometheus::GaugeVec::new(
            prometheus::Opts::new("bought_at", "The Prices at which the items were bought"),
            &["item", "kind", "condition"],
        )
        .unwrap();
        registry
            .register(Box::new(bought_at_prices.clone()))
            .unwrap();

        let last_update =
            prometheus::Gauge::new("last_updated", "The Unix Timestamp of the last update")
                .unwrap();
        registry.register(Box::new(last_update.clone())).unwrap();

        Self {
            buy_prices,
            buy_counts,
            buy_listings,
            sell_prices,
            bought_at_prices,
            last_update,
        }
    }
}
