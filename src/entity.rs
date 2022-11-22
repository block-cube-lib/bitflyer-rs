use crate::deserializer::{timestamp, timestamp_option};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn get_reverse(&self) -> Self {
        use Side::*;
        match *self {
            Buy => Sell,
            Sell => Buy,
        }
    }
}
impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(&self)
            .unwrap()
            .trim_matches('"')
            .to_string();
        write!(f, "{s}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParentOrderSide {
    Buy,
    Sell,
    BuySell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketType {
    Spot,
    #[serde(rename = "FX")]
    Fx,
    Futures,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProductCode {
    BtcJpy,
    XrpJpy,
    EthJpy,
    XlmJpy,
    MonaJpy,
    EthBtc,
    BchBtc,
    FxBtcJpy,
    #[serde(other)]
    Other,
}

impl std::string::ToString for ProductCode {
    fn to_string(&self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .trim_matches('"')
            .to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Health {
    Normal,
    Busy,
    VeryBusy,
    SuperBusy,
    NoOrder,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    Running,
    Closed,
    Starting,
    Preopen,
    #[serde(rename = "CIRCUT BREAK")]
    CircutBreak,
    #[serde(rename = "AWAITING SQ")]
    AwaitingSq,
    Matured,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "child_order_type")]
pub enum ChildOrderType {
    Limit { price: Decimal },
    Market,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParentOrderType {
    Limit,
    Market,
    Stop,
    StopLimit,
    Trail,
    Simple,
    Ifd,
    Oco,
    Ifdoco,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "order_method")]
pub enum ParentOrderMethod {
    Simple {
        parameters: [ParentOrderConditionType; 1],
    },
    Ifd {
        parameters: [ParentOrderConditionType; 2],
    },
    Oco {
        parameters: [ParentOrderConditionType; 2],
    },
    Ifdoco {
        parameters: [ParentOrderConditionType; 3],
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "condition_type")]
pub enum ParentOrderConditionType {
    Limit {
        product_code: ProductCode,
        side: Side,
        size: Decimal,
        price: Decimal,
    },
    Market {
        product_code: ProductCode,
        side: Side,
        size: Decimal,
    },
    Stop {
        product_code: ProductCode,
        side: Side,
        size: Decimal,
        trigger_price: Decimal,
    },
    StopLimit {
        product_code: ProductCode,
        side: Side,
        size: Decimal,
        price: Decimal,
        trigger_price: Decimal,
    },
    Trail {
        product_code: ProductCode,
        side: Side,
        size: Decimal,
        offset: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderState {
    Active,
    Completed,
    Canceled,
    Expired,
    Rejected,
}

impl std::string::ToString for OrderState {
    fn to_string(&self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .trim_matches('"')
            .to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct BoardElement {
    price: Decimal,
    size: Decimal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Board {
    mid_price: Decimal,
    bids: Vec<BoardElement>,
    asks: Vec<BoardElement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Market {
    product_code: ProductCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
    market_type: MarketType,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Ticker {
    pub product_code: ProductCode,
    pub state: State,
    #[serde(with = "timestamp")]
    pub timestamp: DateTime<Utc>,
    pub tick_id: Decimal,
    pub best_bid: Decimal,
    pub best_ask: Decimal,
    pub best_bid_size: Decimal,
    pub best_ask_size: Decimal,
    pub total_bid_depth: Decimal,
    pub total_ask_depth: Decimal,
    pub market_bid_size: Decimal,
    pub market_ask_size: Decimal,
    pub ltp: Decimal,
    pub volume: Decimal,
    pub volume_by_product: Decimal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Execution {
    id: u64,
    side: Side,
    price: Decimal,
    size: Decimal,
    #[serde(with = "timestamp")]
    exec_date: DateTime<Utc>,
    buy_child_order_acceptance_id: String,
    sell_child_order_acceptance_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct BoardState {
    health: Health,
    state: State,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<BoardStateData>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct BoardStateData {
    special_quotation: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct BoardHealth {
    status: Health,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Balance {
    currency_code: String,
    amount: Decimal,
    available: Decimal,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Collateral {
    pub collateral: Decimal,
    pub open_position_pnl: Decimal,
    pub require_collateral: Decimal,
    pub keep_rate: f64,
    pub margin_call_amount: Decimal,
    #[serde(with = "timestamp_option")]
    pub margin_call_due_date: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CollateralAccount {
    currency_code: String,
    amount: Decimal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ChildOrder {
    pub id: u64,
    pub child_order_id: String,
    pub product_code: ProductCode,
    pub side: Side,
    #[serde(flatten)]
    pub child_order_type: ChildOrderType,
    pub average_price: Decimal,
    pub size: Decimal,
    pub child_order_state: OrderState,
    #[serde(with = "timestamp")]
    pub expire_date: DateTime<Utc>,
    #[serde(with = "timestamp")]
    pub child_order_date: DateTime<Utc>,
    pub child_order_acceptance_id: String,
    pub outstanding_size: Decimal,
    pub cancel_size: Decimal,
    pub executed_size: Decimal,
    pub total_commission: Decimal,
    pub time_in_force: TimeInForce,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Position {
    pub product_code: ProductCode,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
    pub commission: Decimal,
    pub swap_point_accumulate: Decimal,
    pub require_collateral: Decimal,
    #[serde(with = "timestamp")]
    pub open_date: DateTime<Utc>,
    pub leverage: Decimal,
    pub pnl: Decimal,
    pub sfd: Decimal,
}
