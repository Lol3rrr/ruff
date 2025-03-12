use super::Item;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub buy_prices: prometheus::GaugeVec,
    pub buy_counts: prometheus::GaugeVec,
    pub buy_listings: prometheus::GaugeVec,
    sell_prices: prometheus::GaugeVec,
    pub sell_counts: prometheus::GaugeVec,
    pub bought_at_prices: prometheus::GaugeVec,
    pub last_update: prometheus::Gauge,
}

impl Metrics {
    pub fn new(registry: &prometheus::Registry) -> Self {
        let sell_prices = prometheus::GaugeVec::new(
            prometheus::Opts::new("sell_prices", "The minimum Sell Price (in RMB)"),
            &[
                "item",
                "kind",
                "condition",
                "souvenir",
                "stattrak",
                "weapon",
                "skin",
                "marketplace",
            ],
        )
        .unwrap();
        registry.register(Box::new(sell_prices.clone())).unwrap();

        let sell_counts = prometheus::GaugeVec::new(
            prometheus::Opts::new("sell_count", "The number of skins being sold for this"),
            &[
                "item",
                "kind",
                "condition",
                "souvenir",
                "stattrak",
                "weapon",
                "skin",
                "marketplace",
            ],
        )
        .unwrap();
        registry.register(Box::new(sell_counts.clone())).unwrap();

        let buy_prices = prometheus::GaugeVec::new(
            prometheus::Opts::new("buy_orders", "The max Buy Order Price (in RMB)"),
            &["item", "kind", "condition", "marketplace"],
        )
        .unwrap();
        registry.register(Box::new(buy_prices.clone())).unwrap();

        let buy_counts = prometheus::GaugeVec::new(
            prometheus::Opts::new(
                "buy_counts",
                "The number of items that can be bought at the max Buy Order Price",
            ),
            &["item", "kind", "condition", "marketplace"],
        )
        .unwrap();
        registry.register(Box::new(buy_counts.clone())).unwrap();

        let buy_listings = prometheus::GaugeVec::new(
            prometheus::Opts::new(
                "buy_listings",
                "The number of listings that buy at the max Buy Order Price",
            ),
            &["item", "kind", "condition", "marketplace"],
        )
        .unwrap();
        registry.register(Box::new(buy_listings.clone())).unwrap();

        let bought_at_prices = prometheus::GaugeVec::new(
            prometheus::Opts::new("bought_at", "The Prices at which the items were bought"),
            &["item", "kind", "condition", "marketplace"],
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
            sell_counts,
            bought_at_prices,
            last_update,
        }
    }

    fn item_to_labels<'own, 'market, 'item, 'item_src, 'output>(
        &'own self,
        marketplace: &'market str,
        item: &'item Item<'item_src>,
    ) -> Option<[&'output str; 8]>
    where
        'own: 'output,
        'market: 'output,
        'item: 'output,
        'item_src: 'output,
    {
        match item {
            Item::Case { name } => Some([name, "case", "", "false", "false", "", "", marketplace]),
            Item::Package { name } => {
                Some([name, "package", "", "false", "false", "", "", marketplace])
            }
            Item::Capsule { name } => Some([name, "capsule", "", "false", "false", "", "", marketplace]),
            Item::PatchPack { name } => Some([name, "patchpack", "", "false", "false", "", "", marketplace]),
            Item::PinsCapsule { name } => Some([name, "pins-capsule", "", "false", "false", "", "", marketplace]),
            Item::MusicKitBox { name } => Some([name, "music-kit-box", "", "false", "false", "", "", marketplace]),
            Item::GraffitiBox { name } => Some([name, "graffiti-box", "", "false", "false", "", "", marketplace]),
            Item::Agent { name } => Some([name, "agent", "", "false", "false", "", "", marketplace]),
            Item::Weapon {
                name,
                weapon,
                skin,
                condition,
                stattrak,
                souvenir,
            } => Some([
                name,
                "weapon",
                condition.as_str(),
                if *souvenir { "true" } else { "false" },
                if *stattrak { "true" } else { "false" },
                weapon,
                skin,
                marketplace,
            ]),
            Item::Special {
                name,
                weapon,
                skin,
                condition,
                stattrak,
            } => Some([
                name,
                "special",
                condition.as_str(),
                "false",
                if *stattrak { "true" } else { "false" },
                weapon,
                skin,
                marketplace,
            ]),
            Item::Sticker { name } => {
                Some([name, "sticker", "", "false", "false", "", "", marketplace])
            }
            Item::Patch { name } => {
                Some([name, "patch", "", "false", "false", "", "", marketplace])
            }
            Item::Charm { name } => {
                Some([name, "charm", "", "false", "false", "", "", marketplace])
            }
            Item::Other { name } => {
                Some([name, "other", "", "false", "false", "", "", marketplace])
            }
        }
    }

    pub fn set_price(&self, marketplace: &str, item: &Item<'_>, value: f64) {
        // ["item", "kind", "condition", "souvenir", "stattrak", "weapon", "skin", "marketplace"]
        let labels = match self.item_to_labels(marketplace, item) {
            Some(l) => l,
            None => return,
        };

        self.sell_prices.with_label_values(&labels).set(value);
    }

    pub fn set_count(&self, marketplace: &str, item: &Item<'_>, count: f64) {
        let labels = match self.item_to_labels(marketplace, item) {
            Some(l) => l,
            None => return,
        };

        self.sell_counts.with_label_values(&labels).set(count);
    }
}
