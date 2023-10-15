use std::collections::HashMap;

use serde::Deserialize;

pub struct Client {
    req_client: reqwest::Client,
}

#[derive(Debug)]
pub struct TargetItem {
    pub name: String,
    pub goods_id: u64,
}

#[derive(Debug)]
pub struct BuyOrderSummary {
    pub max: f64,
}

#[derive(Debug)]
pub struct SellOrderSummary {
    pub min: f64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "code")]
enum Response<D> {
    OK { data: D, msg: serde_json::Value },
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
    specific: serde_json::Value,
    state: String,
    state_text: String,
    tradable_cooldown: serde_json::Value,
    updated_at: usize,
    user_id: String,
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

impl Client {
    pub fn new() -> Self {
        Self {
            req_client: reqwest::Client::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn load_buyorders(&mut self, item: &TargetItem) -> Result<BuyOrderSummary, ()> {
        let url = format!(
            "https://buff.163.com/api/market/goods/buy_order?game=csgo&goods_id={}&page_num=1",
            item.goods_id
        );

        let req_res = match self.req_client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Making Request {:?}", e);
                return Err(());
            }
        };

        let res: Response<BuyOrderData> = match req_res.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Deserialzing Response {:?}", e);
                return Err(());
            }
        };

        match res {
            Response::OK { data, .. } => {
                let max = data
                    .items
                    .iter()
                    .filter_map(|item| item.price.parse::<f64>().ok())
                    .fold(0.0, |acc, val| if val > acc { val } else { acc });

                Ok(BuyOrderSummary { max })
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn load_sellorders(&mut self, item: &TargetItem) -> Result<SellOrderSummary, ()> {
        let url = format!(
            "https://buff.163.com/api/market/goods/sell_order?game=csgo&goods_id={}&page_num=1",
            item.goods_id
        );

        let req_res = match self.req_client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Making Request {:?}", e);
                return Err(());
            }
        };

        let res: Response<SellOrderData> = match req_res.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Deserialzing Response {:?}", e);
                return Err(());
            }
        };

        match res {
            Response::OK { data, .. } => {
                let min = data
                    .items
                    .iter()
                    .filter_map(|item| item.price.parse::<f64>().ok())
                    .fold(f64::MAX, |acc, val| if val < acc { val } else { acc });

                Ok(SellOrderSummary { min })
            }
        }
    }
}
