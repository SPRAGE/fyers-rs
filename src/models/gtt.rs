//! GTT order request and response models.

use serde::{Deserialize, Serialize};

/// Request body for documented single-leg GTT order placement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GttOrderRequest {
    /// Buy/sell side code.
    pub side: i64,
    /// Trading symbol.
    pub symbol: String,
    /// Product type.
    #[serde(rename = "productType")]
    pub product_type: String,
    /// Optional caller-defined GTT order tag.
    #[serde(default, rename = "orderTag", skip_serializing_if = "Option::is_none")]
    pub order_tag: Option<String>,
    /// Trigger leg details.
    #[serde(rename = "orderInfo")]
    pub order_info: GttOrderInfo,
}

/// Request body for documented OCO GTT order placement.
pub type GttOcoOrderRequest = GttOrderRequest;

/// Nested GTT trigger legs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GttOrderInfo {
    /// First GTT leg.
    pub leg1: GttOrderLeg,
    /// Optional second GTT leg for OCO orders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leg2: Option<GttOrderLeg>,
}

/// Individual GTT leg.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GttOrderLeg {
    /// Limit price submitted when the trigger fires.
    pub price: f64,
    /// Price at which this leg triggers.
    #[serde(rename = "triggerPrice")]
    pub trigger_price: f64,
    /// Triggered order quantity.
    pub qty: i64,
}

/// Request body for documented GTT modification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifyGttOrderRequest {
    /// GTT order ID to modify.
    pub id: String,
    /// Updated trigger leg details.
    #[serde(rename = "orderInfo")]
    pub order_info: GttOrderInfo,
}

/// Request body for documented GTT cancellation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelGttOrderRequest {
    /// GTT order ID to cancel.
    pub id: String,
}

/// Response returned by documented GTT create/modify/cancel endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GttActionResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Optional top-level GTT order ID.
    #[serde(default)]
    pub id: Option<String>,
}

/// Response returned by the documented GTT order book endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GttOrderBookResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// GTT order book rows.
    #[serde(rename = "orderBook")]
    pub order_book: Vec<GttOrderBookItem>,
}

/// GTT order book row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GttOrderBookItem {
    /// Fyers user/client ID.
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// Exchange code.
    pub exchange: i64,
    /// Fyers token.
    pub fy_token: String,
    /// Fyers system-generated order identifier.
    pub id_fyers: String,
    /// GTT order ID.
    pub id: String,
    /// Instrument code.
    pub instrument: i64,
    /// Lot size.
    pub lot_size: i64,
    /// Contract multiplier.
    pub multiplier: i64,
    /// Order status code.
    pub ord_status: i64,
    /// Price precision.
    pub precision: i64,
    /// First leg limit price.
    pub price_limit: f64,
    /// Second leg limit price.
    pub price2_limit: f64,
    /// First leg trigger price.
    pub price_trigger: f64,
    /// Second leg trigger price.
    pub price2_trigger: f64,
    /// Product type.
    pub product_type: String,
    /// First leg quantity.
    pub qty: i64,
    /// Second leg quantity.
    pub qty2: i64,
    /// Report type.
    pub report_type: String,
    /// Segment code.
    pub segment: i64,
    /// Trading symbol.
    pub symbol: String,
    /// Symbol description.
    pub symbol_desc: String,
    /// Exchange symbol.
    pub symbol_exch: String,
    /// Tick size.
    pub tick_size: f64,
    /// Transaction side code.
    pub tran_side: i64,
    /// GTT/OCO indicator.
    pub gtt_oco_ind: i64,
    /// Creation time.
    pub create_time: String,
    /// Creation time epoch.
    pub create_time_epoch: i64,
    /// OMS message.
    pub oms_msg: String,
    /// LTP change.
    pub ltp_ch: f64,
    /// LTP percentage change.
    pub ltp_chp: f64,
    /// Last traded price.
    pub ltp: f64,
}
