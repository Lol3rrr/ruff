use std::{collections::HashMap, time::Duration};

use rand::{seq::SliceRandom, Rng, SeedableRng};
use serde::Deserialize;
use tracing::Instrument;

use crate::config::Item;

pub struct Client {
    pub req_client: reqwest::Client,
}

#[derive(Debug)]
pub struct BuyOrderSummary {
    pub max: f64,
    pub count: usize,
    pub listings: usize,
}

#[derive(Debug)]
pub struct SellOrderSummary {
    pub min: f64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "code")]
enum Response<D> {
    #[serde(rename = "OK")]
    Ok {
        data: D,
        #[serde(rename = "msg")]
        _msg: serde_json::Value,
    },
    #[serde(rename = "Login Required")]
    LoginRequired {
        error: String,
        extra: Option<serde_json::Value>,
    },
}

#[derive(Debug, Deserialize)]
struct BuyOrderData {
    items: Vec<BuyOrderItem>,
    page_num: usize,
    page_size: usize,
    show_pay_method_icon: bool,
    total_count: usize,
    total_page: usize,
    user_infos: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct BuyOrderItem {
    allow_tradable_cooldown: usize,
    appid: usize,
    created_at: usize,
    expire_time: serde_json::Value,
    fee: String,
    frozen_amount: String,
    frozen_num: usize,
    game: String,
    goods_id: usize,
    icon_url: String,
    id: String,
    num: usize,
    pay_expire_timeout: serde_json::Value,
    pay_method: usize,
    pay_method_text: String,
    price: String,
    real_num: usize,
    specific: Vec<BuyOrderSpecific>,
    state: String,
    state_text: String,
    tradable_cooldown: serde_json::Value,
    updated_at: usize,
    user_id: String,
}

#[derive(Debug, Deserialize)]
struct BuyOrderSpecific {
    color: String,
    simple_text: String,
    text: String,
    #[serde(flatten)]
    ty: BuyOrderSpecificType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "values")]
enum BuyOrderSpecificType {
    #[serde(rename = "paintwear")]
    PaintWear(Vec<String>),
    #[serde(rename = "unlock_style")]
    UnlockStyle(serde_json::Value),
}

#[derive(Debug, Deserialize)]
struct SellOrderData {
    fop_str: String,
    goods_infos: serde_json::Value,
    has_market_stores: serde_json::Value,
    items: Vec<SellOrderItem>,
    page_num: usize,
    page_size: usize,
    preview_screenshots: serde_json::Value,
    show_game_cms_icon: bool,
    show_pay_method_icon: bool,
    sort_by: String,
    src_url_background: String,
    total_count: usize,
    total_page: usize,
    user_infos: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct SellOrderItem {
    allow_bargain: bool,
    appid: usize,
    asset_info: serde_json::Value,
    background_image_url: String,
    bookmarked: bool,
    can_bargain: bool,
    can_use_inspect_trn_url: bool,
    cannot_bargain_reason: String,
    created_at: usize,
    description: String,
    featured: serde_json::Value,
    fee: String,
    game: String,
    goods_id: usize,
    id: String,
    img_src: String,
    income: String,
    lowest_bargain_price: String,
    mode: usize,
    price: String,
    recent_average_duration: serde_json::Value,
    recent_deliver_rate: serde_json::Value,
    state: usize,
    supported_pay_methods: serde_json::Value,
    tradable_cooldown: serde_json::Value,
    updated_at: usize,
    user_id: String,
}

#[derive(Debug)]
pub enum LoadError {
    SendingRequest(reqwest::Error),
    GettingContent(reqwest::Error),
    StatusCode(reqwest::StatusCode),
    Deserialzing(serde_json::Error),
    ErrorResponse { msg: String },
}

impl Client {
    pub fn new() -> Self {
        Self {
            req_client: reqwest::Client::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn load_buyorders(&mut self, item: &Item<'_>) -> Result<BuyOrderSummary, LoadError> {
        let url = format!(
            "https://buff.163.com/api/market/goods/buy_order?game=csgo&goods_id={}&page_num=1&min_paintwear=-1&max_paintwear=-1&tag_ids=-1",
            item.buff_id
        );

        let req_res = self
            .req_client
            .get(&url)
            .send()
            .await
            .map_err(|e| LoadError::SendingRequest(e))?;

        let status = req_res.status();
        if !status.is_success() {
            return Err(LoadError::StatusCode(status));
        }

        let raw_content = req_res
            .bytes()
            .await
            .map_err(|e| LoadError::GettingContent(e))?;

        let res: Response<BuyOrderData> =
            serde_json::from_slice(&raw_content).map_err(|e| LoadError::Deserialzing(e))?;

        match res {
            Response::Ok { data, .. } => {
                tracing::trace!("BuyOrderData: {:#?}", data);

                let mut items: Vec<_> = data
                    .items
                    .iter()
                    .filter_map(|item| {
                        let price: f64 = item.price.parse().ok()?;

                        Some((price, item.num.saturating_sub(item.real_num)))
                    })
                    .collect();

                let max =
                    items
                        .iter()
                        .map(|(p, _)| p)
                        .fold(0.0, |acc, val| if *val > acc { *val } else { acc });

                items.retain(|(p, _)| *p == max);

                if items.len() == 10 {
                    tracing::info!("Need to load more buy orders, to see if there are more listings at the max price");
                }

                Ok(BuyOrderSummary {
                    max,
                    count: items.iter().map(|(_, c)| *c).sum(),
                    listings: items.len(),
                })
            }
            Response::LoginRequired {
                error,
                extra: _extra,
            } => {
                return Err(LoadError::ErrorResponse { msg: error });
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn load_sellorders(
        &mut self,
        item: &Item<'_>,
    ) -> Result<SellOrderSummary, LoadError> {
        let url = format!(
            "https://buff.163.com/api/market/goods/sell_order?game=csgo&goods_id={}&page_num=1",
            item.buff_id
        );

        let req_res = self
            .req_client
            .get(&url)
            .send()
            .await
            .map_err(|e| LoadError::SendingRequest(e))?;

        let status = req_res.status();
        if !status.is_success() {
            return Err(LoadError::StatusCode(status));
        }

        let raw_content = req_res
            .bytes()
            .await
            .map_err(|e| LoadError::GettingContent(e))?;

        let res: Response<SellOrderData> =
            serde_json::from_slice(&raw_content).map_err(|e| LoadError::Deserialzing(e))?;

        match res {
            Response::Ok { data, .. } => {
                let min = data
                    .items
                    .iter()
                    .filter_map(|item| item.price.parse::<f64>().ok())
                    .fold(f64::MAX, |acc, val| if val < acc { val } else { acc });

                Ok(SellOrderSummary { min })
            }
            Response::LoginRequired {
                error,
                extra: _extra,
            } => {
                return Err(LoadError::ErrorResponse { msg: error });
            }
        }
    }
}

#[tracing::instrument(skip(items, metrics))]
pub async fn gather(
    items: std::sync::Arc<arc_swap::ArcSwap<Vec<Item<'static>>>>,
    metrics: crate::Metrics,
) {
    let mut client = Client::new();

    let mut rng = rand::rngs::SmallRng::from_entropy();

    loop {
        tracing::info!("Loading Buff Data");

        let start_time = std::time::Instant::now();

        let items = items.load();

        let shuffled = {
            let mut tmp: Vec<_> = (*items.as_ref()).clone();
            tmp.shuffle(&mut rng);
            tmp
        };

        for (i, item) in shuffled.iter().enumerate() {
            async {
                let kind_str: &str = &item.kind;
                let condition_str: &str = item.condition;

                let labels = [&item.name, kind_str, condition_str];

                match client.load_buyorders(&item).await {
                    Ok(buy_order) => {
                        tracing::info!("Buy Order Summary {:?}", buy_order,);

                        metrics
                            .buy_prices
                            .with_label_values(&labels)
                            .set(buy_order.max);
                        metrics
                            .buy_counts
                            .with_label_values(&labels)
                            .set(buy_order.count as f64);
                        metrics
                            .buy_listings
                            .with_label_values(&labels)
                            .set(buy_order.listings as f64);
                    }
                    Err(e) => {
                        tracing::error!("Loading Buy Orders {:?}", e);
                    }
                };

                tokio::time::sleep(Duration::from_millis(rng.gen_range(2500..4500))).await;

                match client.load_sellorders(&item).await {
                    Ok(sell_order) => {
                        tracing::info!("Sell Order Summary {:?}", sell_order,);

                        metrics
                            .sell_prices
                            .with_label_values(&labels)
                            .set(sell_order.min);
                    }
                    Err(e) => {
                        tracing::error!("Loading Sell Orders {:?}", e);
                    }
                };

                if let Some(bought_price) = item.bought_at.as_ref() {
                    metrics
                        .bought_at_prices
                        .with_label_values(&labels)
                        .set(*bought_price);
                }

                tokio::time::sleep(Duration::from_millis(rng.gen_range(2500..4500))).await;
            }
            .instrument(tracing::info_span!(
                "Updating Item Stats",
                item = item.name,
                current = i + 1,
                total_items = shuffled.len()
            ))
            .await;
        }

        let elapsed = start_time.elapsed();
        tracing::info!("Updating stats took {:?}", elapsed);

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
