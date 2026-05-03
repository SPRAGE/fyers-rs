//! WebSocket command and event protocol models.

use std::collections::HashMap;
use std::time::Duration;

use prost::Message;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Market-data socket connection/config options that affect protocol behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSocketConfig {
    /// Enable lighter LTP-focused updates.
    pub lite_mode: bool,
    /// Enable reconnect handling in the future manager layer.
    pub reconnect: bool,
    /// Maximum reconnect attempts for the future manager layer.
    pub reconnect_retry: usize,
    /// Queue processing interval used by documented SDK configuration.
    pub queue_process_interval: QueueProcessInterval,
}

impl Default for DataSocketConfig {
    fn default() -> Self {
        Self {
            lite_mode: false,
            reconnect: true,
            reconnect_retry: 50,
            queue_process_interval: QueueProcessInterval::default(),
        }
    }
}

/// Documented market-data queue process interval, constrained to 1ms..=2000ms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueProcessInterval(Duration);

impl QueueProcessInterval {
    /// Create a queue process interval from milliseconds.
    pub fn from_millis(millis: u64) -> Result<Self, String> {
        if !(1..=2000).contains(&millis) {
            return Err("queue process interval must be between 1ms and 2000ms".to_owned());
        }

        Ok(Self(Duration::from_millis(millis)))
    }

    /// Return the duration value.
    pub const fn as_duration(self) -> Duration {
        self.0
    }
}

impl Default for QueueProcessInterval {
    fn default() -> Self {
        Self(Duration::from_millis(1))
    }
}

/// Data socket subscription kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSubscriptionKind {
    #[serde(rename = "SymbolUpdate")]
    SymbolUpdate,
    #[serde(rename = "DepthUpdate")]
    DepthUpdate,
}

/// Data socket subscribe/unsubscribe command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSubscribeRequest {
    pub symbols: Vec<String>,
    pub data_type: DataSubscriptionKind,
}

/// Data socket unsubscribe command.
pub type DataUnsubscribeRequest = DataSubscribeRequest;

/// Parsed market-data socket event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum DataSocketEvent {
    Connected(DataControlEvent),
    Subscribed(DataControlEvent),
    Unsubscribed(DataControlEvent),
    Mode(DataControlEvent),
    Error(DataControlEvent),
    SymbolUpdate(SymbolUpdate),
    IndexUpdate(IndexUpdate),
    DepthUpdate(DepthUpdate),
}

/// Common data-socket control event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataControlEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub code: i64,
    pub message: String,
    pub s: String,
}

/// Symbol update event (`type = "sf"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolUpdate {
    #[serde(rename = "type")]
    pub event_type: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub ltp: f64,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub prev_close_price: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub high_price: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub low_price: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub open_price: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub ch: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub chp: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub vol_traded_today: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub last_traded_time: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub exch_feed_time: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub bid_size: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub ask_size: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub bid_price: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub ask_price: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub last_traded_qty: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub tot_buy_qty: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub tot_sell_qty: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_f64_from_value")]
    pub avg_trade_price: Option<f64>,
}

/// Index update event (`type = "if"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexUpdate {
    #[serde(rename = "type")]
    pub event_type: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub ltp: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub prev_close_price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub high_price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub low_price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub open_price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub ch: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub chp: f64,
    #[serde(default, deserialize_with = "deserialize_optional_i64_from_value")]
    pub exch_feed_time: Option<i64>,
}

/// Depth update event (`type = "dp"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepthUpdate {
    #[serde(rename = "type")]
    pub event_type: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub bid_price1: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub bid_price2: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub bid_price3: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub bid_price4: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub bid_price5: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub ask_price1: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub ask_price2: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub ask_price3: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub ask_price4: f64,
    #[serde(deserialize_with = "deserialize_f64_from_value")]
    pub ask_price5: f64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_size1: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_size2: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_size3: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_size4: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_size5: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_size1: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_size2: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_size3: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_size4: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_size5: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_order1: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_order2: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_order3: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_order4: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub bid_order5: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_order1: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_order2: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_order3: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_order4: i64,
    #[serde(deserialize_with = "deserialize_i64_from_value")]
    pub ask_order5: i64,
}

/// Parse a data-socket text frame into a typed event.
pub fn parse_data_event(input: &str) -> Result<DataSocketEvent, String> {
    let value: serde_json::Value =
        serde_json::from_str(input).map_err(|err| format!("invalid data event JSON: {err}"))?;
    let event_type = value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "data event is missing string type".to_owned())?;

    match event_type {
        "cn" => from_value(value, DataSocketEvent::Connected),
        "sub" => from_value(value, DataSocketEvent::Subscribed),
        "unsub" => from_value(value, DataSocketEvent::Unsubscribed),
        "lit" | "ful" => from_value(value, DataSocketEvent::Mode),
        "error" => from_value(value, DataSocketEvent::Error),
        "sf" => from_value(value, DataSocketEvent::SymbolUpdate),
        "if" => from_value(value, DataSocketEvent::IndexUpdate),
        "dp" => from_value(value, DataSocketEvent::DepthUpdate),
        other => Err(format!("unknown data event type: {other}")),
    }
}

/// Order socket connection/config options that affect protocol behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderSocketConfig {
    pub reconnect: bool,
    pub reconnect_retry: usize,
    pub ping_interval: Duration,
}

impl Default for OrderSocketConfig {
    fn default() -> Self {
        Self {
            reconnect: true,
            reconnect_retry: 50,
            ping_interval: Duration::from_secs(10),
        }
    }
}

/// Order-socket subscribe/unsubscribe command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderSubscribeRequest {
    #[serde(rename = "T")]
    pub command_type: String,
    #[serde(rename = "SLIST")]
    pub actions: Vec<String>,
    #[serde(rename = "SUB_T")]
    pub subscribe_type: i64,
}

impl OrderSubscribeRequest {
    pub fn subscribe(actions: Vec<String>) -> Self {
        Self {
            command_type: "SUB_ORD".to_owned(),
            actions,
            subscribe_type: 1,
        }
    }

    pub fn unsubscribe(actions: Vec<String>) -> Self {
        Self {
            command_type: "SUB_ORD".to_owned(),
            actions,
            subscribe_type: -1,
        }
    }
}

/// Parsed order socket event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum OrderSocketEvent {
    Order(OrderUpdate),
    Trade(TradeUpdate),
    Position(PositionUpdate),
    General(GeneralUpdate),
    Edis(EdisUpdate),
    PriceAlert(PriceAlertUpdate),
    Subscribed(OrderControlEvent),
    Unsubscribed(OrderControlEvent),
    Error(OrderControlEvent),
    Closed(OrderControlEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderUpdate {
    pub s: String,
    pub orders: OrderSocketOrder,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeUpdate {
    pub s: String,
    pub trades: OrderSocketTrade,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub s: String,
    pub positions: OrderSocketPosition,
}

/// Documented order WebSocket order update payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderSocketOrder {
    #[serde(default, rename = "clientId", alias = "client_id")]
    pub client_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "exchOrdId", alias = "id_exchange")]
    pub exch_ord_id: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub qty: Option<i64>,
    #[serde(default, rename = "remainingQuantity", alias = "qty_remaining")]
    pub remaining_quantity: Option<i64>,
    #[serde(default, rename = "filledQty", alias = "qty_filled")]
    pub filled_qty: Option<i64>,
    #[serde(default, alias = "org_ord_status")]
    pub status: Option<i64>,
    #[serde(default, alias = "oms_msg", alias = "status_msg")]
    pub message: Option<String>,
    #[serde(default)]
    pub segment: Option<i64>,
    #[serde(default, rename = "limitPrice", alias = "price_limit")]
    pub limit_price: Option<f64>,
    #[serde(default, rename = "stopPrice", alias = "price_stop")]
    pub stop_price: Option<f64>,
    #[serde(default, rename = "productType", alias = "product_type")]
    pub product_type: Option<String>,
    #[serde(default, rename = "type", alias = "ord_type")]
    pub order_type: Option<i64>,
    #[serde(default, alias = "tran_side")]
    pub side: Option<i64>,
    #[serde(default, rename = "orderValidity", alias = "validity")]
    pub order_validity: Option<String>,
    #[serde(default, rename = "orderDateTime", alias = "time_oms")]
    pub order_date_time: Option<String>,
    #[serde(default, rename = "parentId", alias = "id_parent")]
    pub parent_id: Option<String>,
    #[serde(default, rename = "tradedPrice", alias = "price_traded")]
    pub traded_price: Option<f64>,
    #[serde(default, alias = "ord_source")]
    pub source: Option<String>,
    #[serde(default, rename = "fyToken", alias = "fy_token", alias = "fytoken")]
    pub fy_token: Option<String>,
    #[serde(default, rename = "offlineOrder", alias = "offline_flag")]
    pub offline_order: Option<bool>,
    #[serde(default)]
    pub pan: Option<String>,
    #[serde(default)]
    pub exchange: Option<i64>,
    #[serde(default)]
    pub instrument: Option<i64>,
    #[serde(default)]
    pub id_fyers: Option<String>,
    #[serde(default, alias = "symbol_exch")]
    pub ex_sym: Option<String>,
    #[serde(default, alias = "symbol_desc")]
    pub description: Option<String>,
    #[serde(default, rename = "orderNumStatus")]
    pub order_num_status: Option<String>,
}

/// Documented order WebSocket trade update payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderSocketTrade {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "orderDateTime", alias = "fill_time")]
    pub order_date_time: Option<String>,
    #[serde(default, rename = "orderNumber")]
    pub order_number: Option<String>,
    #[serde(default, rename = "tradeNumber", alias = "id_fill")]
    pub trade_number: Option<String>,
    #[serde(default, rename = "tradePrice", alias = "price_traded")]
    pub trade_price: Option<f64>,
    #[serde(default, rename = "tradeValue", alias = "traded_val")]
    pub trade_value: Option<f64>,
    #[serde(default, rename = "tradedQty", alias = "qty_traded")]
    pub traded_qty: Option<i64>,
    #[serde(default, alias = "tran_side")]
    pub side: Option<i64>,
    #[serde(default, rename = "productType", alias = "product_type")]
    pub product_type: Option<String>,
    #[serde(default, rename = "exchangeOrderNo", alias = "id_exchange")]
    pub exchange_order_no: Option<String>,
    #[serde(default)]
    pub segment: Option<i64>,
    #[serde(default)]
    pub exchange: Option<i64>,
    #[serde(default, rename = "fyToken", alias = "fy_token", alias = "fytoken")]
    pub fy_token: Option<String>,
    #[serde(default)]
    pub id_fyers: Option<String>,
    #[serde(default, rename = "orderType", alias = "ord_type")]
    pub order_type: Option<i64>,
    #[serde(default, rename = "clientId", alias = "client_id")]
    pub client_id: Option<String>,
}

/// Documented order WebSocket position update payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderSocketPosition {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "buyAvg", alias = "buy_avg")]
    pub buy_avg: Option<f64>,
    #[serde(default, rename = "buyQty", alias = "buy_qty")]
    pub buy_qty: Option<i64>,
    #[serde(default, rename = "buyVal", alias = "buy_val")]
    pub buy_val: Option<f64>,
    #[serde(default, rename = "sellAvg", alias = "sell_avg")]
    pub sell_avg: Option<f64>,
    #[serde(default, rename = "sellQty", alias = "sell_qty")]
    pub sell_qty: Option<i64>,
    #[serde(default, rename = "sellVal", alias = "sell_val")]
    pub sell_val: Option<f64>,
    #[serde(default, rename = "netAvg", alias = "net_avg")]
    pub net_avg: Option<f64>,
    #[serde(default, rename = "netQty", alias = "net_qty")]
    pub net_qty: Option<i64>,
    #[serde(default, alias = "tran_side")]
    pub side: Option<i64>,
    #[serde(default)]
    pub qty: Option<i64>,
    #[serde(default, rename = "productType", alias = "product_type")]
    pub product_type: Option<String>,
    #[serde(default, alias = "pl_realized")]
    pub realized_profit: Option<f64>,
    #[serde(default, rename = "crossCurrency", alias = "cross_curr_flag")]
    pub cross_currency: Option<String>,
    #[serde(default, rename = "rbiRefRate", alias = "rbirefrate")]
    pub rbi_ref_rate: Option<f64>,
    #[serde(default, rename = "qtyMulti_com", alias = "qty_multiplier")]
    pub qty_multi_com: Option<f64>,
    #[serde(default)]
    pub segment: Option<i64>,
    #[serde(default)]
    pub exchange: Option<i64>,
    #[serde(default, rename = "slNo")]
    pub sl_no: Option<i64>,
    #[serde(default, rename = "fyToken", alias = "fy_token", alias = "fytoken")]
    pub fy_token: Option<String>,
    #[serde(default, rename = "cfBuyQty", alias = "cf_buy_qty")]
    pub cf_buy_qty: Option<i64>,
    #[serde(default, rename = "cfSellQty", alias = "cf_sell_qty")]
    pub cf_sell_qty: Option<i64>,
    #[serde(default, rename = "dayBuyQty", alias = "day_buy_qty")]
    pub day_buy_qty: Option<i64>,
    #[serde(default, rename = "daySellQty", alias = "day_sell_qty")]
    pub day_sell_qty: Option<i64>,
    #[serde(default, alias = "pl_total")]
    pub pl: Option<f64>,
    #[serde(default, alias = "pl_unrealized")]
    pub unrealized_profit: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralUpdate {
    pub s: String,
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdisUpdate {
    pub s: String,
    pub edis: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceAlertUpdate {
    pub s: String,
    pub pricealerts: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderControlEvent {
    pub s: String,
    pub code: i64,
    pub message: String,
}

/// Parse an order-socket text frame into a typed event.
pub fn parse_order_event(input: &str) -> Result<OrderSocketEvent, String> {
    let value: serde_json::Value =
        serde_json::from_str(input).map_err(|err| format!("invalid order event JSON: {err}"))?;

    if value.get("orders").is_some() {
        return from_value(value, OrderSocketEvent::Order);
    }
    if value.get("trades").is_some() {
        return from_value(value, OrderSocketEvent::Trade);
    }
    if value.get("positions").is_some() {
        return from_value(value, OrderSocketEvent::Position);
    }
    if value.get("edis").is_some() {
        return from_value(value, OrderSocketEvent::Edis);
    }
    if value.get("pricealerts").is_some() {
        return from_value(value, OrderSocketEvent::PriceAlert);
    }

    let code = value
        .get("code")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default();
    let message = value
        .get("message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    if code == 1606 || message.contains("unsubscribed") {
        return from_value(value, OrderSocketEvent::Unsubscribed);
    }
    if code == 1605 || message.contains("subscribed") {
        return from_value(value, OrderSocketEvent::Subscribed);
    }
    if value.get("code").is_some() && message.contains("error") {
        return from_value(value, OrderSocketEvent::Error);
    }
    if message.contains("closed") {
        return from_value(value, OrderSocketEvent::Closed);
    }

    from_value(value, OrderSocketEvent::General)
}

/// TBT/depth socket connection/config options that affect protocol behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TbtSocketConfig {
    pub reconnect: bool,
    pub reconnect_retry: usize,
    pub diff_only: bool,
    pub ping_interval: Duration,
}

impl Default for TbtSocketConfig {
    fn default() -> Self {
        Self {
            reconnect: true,
            reconnect_retry: 50,
            diff_only: false,
            ping_interval: Duration::from_secs(30),
        }
    }
}

/// TBT subscription command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TbtSubscribeRequest {
    #[serde(rename = "type")]
    pub request_type: i64,
    pub data: TbtSubscribeData,
}

/// TBT subscription command body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TbtSubscribeData {
    pub subs: i64,
    pub symbols: Vec<String>,
    pub mode: String,
    pub channel: String,
}

/// TBT switch-channel command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TbtSwitchChannelRequest {
    #[serde(rename = "type")]
    pub request_type: i64,
    pub data: TbtSwitchChannelData,
}

/// TBT switch-channel command body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TbtSwitchChannelData {
    #[serde(rename = "resumeChannels", alias = "resume_channels")]
    pub resume_channels: Vec<String>,
    #[serde(rename = "pauseChannels", alias = "pause_channels")]
    pub pause_channels: Vec<String>,
}

/// Parsed TBT binary event.
#[derive(Debug, Clone, PartialEq)]
pub enum TbtEvent {
    SocketMessage(SocketMessage),
    Error { msg: String },
}

/// Parse a TBT protobuf frame.
pub fn parse_tbt_event(input: &[u8]) -> Result<TbtEvent, String> {
    let message =
        SocketMessage::decode(input).map_err(|err| format!("invalid TBT protobuf frame: {err}"))?;
    if message.error {
        return Ok(TbtEvent::Error { msg: message.msg });
    }

    Ok(TbtEvent::SocketMessage(message))
}

/// Protobuf enum from the documented TBT `msg.proto`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum MessageType {
    Ping = 0,
    Quote = 1,
    ExtendedQuote = 2,
    DailyQuote = 3,
    MarketLevel = 4,
    Ohlcv = 5,
    Depth = 6,
    All = 7,
    Response = 8,
}

/// Protobuf `MarketLevel`.
#[derive(Clone, PartialEq, Message)]
pub struct Int64Value {
    #[prost(int64, tag = "1")]
    pub value: i64,
}

/// Protobuf wrapper for `uint32` fields used by the documented TBT schema.
#[derive(Clone, PartialEq, Message)]
pub struct UInt32Value {
    #[prost(uint32, tag = "1")]
    pub value: u32,
}

/// Protobuf wrapper for `uint64` fields used by the documented TBT schema.
#[derive(Clone, PartialEq, Message)]
pub struct UInt64Value {
    #[prost(uint64, tag = "1")]
    pub value: u64,
}

/// Protobuf `MarketLevel`.
#[derive(Clone, PartialEq, Message)]
pub struct MarketLevel {
    #[prost(message, optional, tag = "1")]
    pub price: Option<Int64Value>,
    #[prost(message, optional, tag = "2")]
    pub qty: Option<UInt32Value>,
    #[prost(message, optional, tag = "3")]
    pub nord: Option<UInt32Value>,
    #[prost(message, optional, tag = "4")]
    pub num: Option<UInt32Value>,
}

/// Protobuf `Depth`.
#[derive(Clone, PartialEq, Message)]
pub struct TbtDepth {
    #[prost(message, optional, tag = "1")]
    pub tbq: Option<UInt64Value>,
    #[prost(message, optional, tag = "2")]
    pub tsq: Option<UInt64Value>,
    #[prost(message, repeated, tag = "3")]
    pub asks: Vec<MarketLevel>,
    #[prost(message, repeated, tag = "4")]
    pub bids: Vec<MarketLevel>,
}

/// Protobuf `Quote`.
#[derive(Clone, PartialEq, Message)]
pub struct Quote {
    #[prost(message, optional, tag = "1")]
    pub ltp: Option<Int64Value>,
    #[prost(message, optional, tag = "2")]
    pub ltt: Option<UInt32Value>,
    #[prost(message, optional, tag = "3")]
    pub ltq: Option<UInt32Value>,
    #[prost(message, optional, tag = "4")]
    pub vtt: Option<UInt64Value>,
    #[prost(message, optional, tag = "5")]
    pub vtt_diff: Option<UInt64Value>,
    #[prost(message, optional, tag = "6")]
    pub oi: Option<UInt64Value>,
    #[prost(message, optional, tag = "7")]
    pub ltpc: Option<Int64Value>,
}

/// Protobuf `ExtendedQuote`.
#[derive(Clone, PartialEq, Message)]
pub struct ExtendedQuote {
    #[prost(message, optional, tag = "1")]
    pub atp: Option<Int64Value>,
    #[prost(message, optional, tag = "2")]
    pub cp: Option<Int64Value>,
    #[prost(message, optional, tag = "3")]
    pub lc: Option<UInt32Value>,
    #[prost(message, optional, tag = "4")]
    pub uc: Option<UInt32Value>,
    #[prost(message, optional, tag = "5")]
    pub yh: Option<Int64Value>,
    #[prost(message, optional, tag = "6")]
    pub yl: Option<Int64Value>,
    #[prost(message, optional, tag = "7")]
    pub poi: Option<UInt64Value>,
    #[prost(message, optional, tag = "8")]
    pub oich: Option<Int64Value>,
    #[prost(message, optional, tag = "9")]
    pub pc: Option<UInt32Value>,
}

/// Protobuf `DailyQuote`.
#[derive(Clone, PartialEq, Message)]
pub struct DailyQuote {
    #[prost(message, optional, tag = "1")]
    pub day_open: Option<Int64Value>,
    #[prost(message, optional, tag = "2")]
    pub day_high: Option<Int64Value>,
    #[prost(message, optional, tag = "3")]
    pub day_low: Option<Int64Value>,
    #[prost(message, optional, tag = "4")]
    pub day_close: Option<Int64Value>,
    #[prost(message, optional, tag = "5")]
    pub dhoi: Option<UInt64Value>,
    #[prost(message, optional, tag = "6")]
    pub dloi: Option<UInt64Value>,
}

/// Protobuf `OHLCV`.
#[derive(Clone, PartialEq, Message)]
pub struct Ohlcv {
    #[prost(message, optional, tag = "1")]
    pub open: Option<Int64Value>,
    #[prost(message, optional, tag = "2")]
    pub high: Option<Int64Value>,
    #[prost(message, optional, tag = "3")]
    pub low: Option<Int64Value>,
    #[prost(message, optional, tag = "4")]
    pub close: Option<Int64Value>,
    #[prost(message, optional, tag = "5")]
    pub volume: Option<UInt32Value>,
    #[prost(message, optional, tag = "6")]
    pub epoch: Option<UInt32Value>,
}

/// Protobuf `SymDetail`.
#[derive(Clone, PartialEq, Message)]
pub struct SymDetail {
    #[prost(string, tag = "1")]
    pub ticksize: String,
}

/// Protobuf `MarketFeed`.
#[derive(Clone, PartialEq, Message)]
pub struct MarketFeed {
    #[prost(message, optional, tag = "1")]
    pub quote: Option<Quote>,
    #[prost(message, optional, tag = "2")]
    pub eq: Option<ExtendedQuote>,
    #[prost(message, optional, tag = "3")]
    pub dq: Option<DailyQuote>,
    #[prost(message, optional, tag = "4")]
    pub ohlcv: Option<Ohlcv>,
    #[prost(message, optional, tag = "5")]
    pub depth: Option<TbtDepth>,
    #[prost(message, optional, tag = "6")]
    pub feed_time: Option<UInt64Value>,
    #[prost(message, optional, tag = "7")]
    pub send_time: Option<UInt64Value>,
    #[prost(string, tag = "8")]
    pub token: String,
    #[prost(uint64, tag = "9")]
    pub sequence_no: u64,
    #[prost(bool, tag = "10")]
    pub snapshot: bool,
    #[prost(string, tag = "11")]
    pub ticker: String,
    #[prost(message, optional, tag = "12")]
    pub symdetail: Option<SymDetail>,
}

/// Protobuf `SocketMessage`.
#[derive(Clone, PartialEq, Message)]
pub struct SocketMessage {
    #[prost(enumeration = "MessageType", tag = "1")]
    pub message_type: i32,
    #[prost(map = "string, message", tag = "2")]
    pub feeds: HashMap<String, MarketFeed>,
    #[prost(bool, tag = "3")]
    pub snapshot: bool,
    #[prost(string, tag = "4")]
    pub msg: String,
    #[prost(bool, tag = "5")]
    pub error: bool,
}

fn deserialize_f64_from_value<'de, D>(deserializer: D) -> std::result::Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Number(value) => value
            .as_f64()
            .ok_or_else(|| serde::de::Error::custom("number is not representable as f64")),
        Value::String(value) => value.parse::<f64>().map_err(serde::de::Error::custom),
        other => Err(serde::de::Error::custom(format!(
            "expected string or number, got {other}"
        ))),
    }
}

fn deserialize_i64_from_value<'de, D>(deserializer: D) -> std::result::Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Number(value) => value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
            .ok_or_else(|| serde::de::Error::custom("number is not representable as i64")),
        Value::String(value) => value.parse::<i64>().map_err(serde::de::Error::custom),
        other => Err(serde::de::Error::custom(format!(
            "expected string or number, got {other}"
        ))),
    }
}

fn deserialize_optional_f64_from_value<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value
            .as_f64()
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("number is not representable as f64")),
        Some(Value::String(value)) => value
            .parse::<f64>()
            .map(Some)
            .map_err(serde::de::Error::custom),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected string, number, or null, got {other}"
        ))),
    }
}

fn deserialize_optional_i64_from_value<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("number is not representable as i64")),
        Some(Value::String(value)) => value
            .parse::<i64>()
            .map(Some)
            .map_err(serde::de::Error::custom),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected string, number, or null, got {other}"
        ))),
    }
}

fn from_value<T, U>(value: serde_json::Value, wrap: impl FnOnce(T) -> U) -> Result<U, String>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(value)
        .map(wrap)
        .map_err(|err| format!("failed to decode WebSocket event: {err}"))
}
