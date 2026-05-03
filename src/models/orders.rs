//! Order-related request and response models.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Query parameters for the documented order book endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBookQuery {
    /// Optional order ID filter.
    pub id: Option<String>,
    /// Optional order tag filter.
    pub order_tag: Option<String>,
}

impl OrderBookQuery {
    /// Query a single order by documented order ID parameter.
    pub fn by_id(id: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            order_tag: None,
        }
    }

    /// Query orders by documented order tag parameter.
    pub fn by_order_tag(order_tag: impl Into<String>) -> Self {
        Self {
            id: None,
            order_tag: Some(order_tag.into()),
        }
    }
}

/// Request body for the documented synchronous single-order placement endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
    /// Trading symbol.
    pub symbol: String,
    /// Order quantity.
    pub qty: i64,
    /// Documented order type code.
    #[serde(rename = "type")]
    pub order_type: i64,
    /// Buy/sell side code.
    pub side: i64,
    /// Product type.
    #[serde(rename = "productType")]
    pub product_type: String,
    /// Limit price.
    #[serde(rename = "limitPrice")]
    pub limit_price: f64,
    /// Stop price.
    #[serde(rename = "stopPrice")]
    pub stop_price: f64,
    /// Order validity.
    pub validity: String,
    /// Disclosed quantity.
    #[serde(rename = "disclosedQty")]
    pub disclosed_qty: i64,
    /// Offline/AMO flag.
    #[serde(rename = "offlineOrder")]
    pub offline_order: bool,
    /// Stop-loss price for bracket/cover orders.
    #[serde(default, rename = "stopLoss", skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<f64>,
    /// Take-profit price for bracket orders.
    #[serde(
        default,
        rename = "takeProfit",
        skip_serializing_if = "Option::is_none"
    )]
    pub take_profit: Option<f64>,
    /// Caller-defined order tag.
    #[serde(default, rename = "orderTag", skip_serializing_if = "Option::is_none")]
    pub order_tag: Option<String>,
    /// Slice-order flag.
    #[serde(
        default,
        rename = "isSliceOrder",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_slice_order: Option<bool>,
}

/// Request body for documented single-order modification endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifyOrderRequest {
    /// Broker order ID to modify.
    pub id: String,
    /// Updated order type code.
    #[serde(rename = "type")]
    pub order_type: i64,
    /// Updated order quantity, when quantity is being changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qty: Option<i64>,
    /// Updated buy/sell side code, when supplied by documented examples.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<i64>,
    /// Updated limit price, required for limit/stop-limit modifications.
    #[serde(
        default,
        rename = "limitPrice",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_price: Option<f64>,
    /// Updated stop price, required for stop/stop-limit modifications.
    #[serde(default, rename = "stopPrice", skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    /// Optional updated disclosed quantity.
    #[serde(
        default,
        rename = "disclosedQty",
        skip_serializing_if = "Option::is_none"
    )]
    pub disclosed_qty: Option<i64>,
    /// Optional offline/AMO flag used by documented async multi-modify examples.
    #[serde(
        default,
        rename = "offlineOrder",
        skip_serializing_if = "Option::is_none"
    )]
    pub offline_order: Option<bool>,
}

/// Request body for documented order cancellation endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    /// Broker order ID to cancel.
    pub id: String,
}

/// Response returned by the documented single-order placement endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderActionResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Broker order ID.
    #[serde(deserialize_with = "deserialize_string_from_value")]
    pub id: String,
}

/// Immediate response returned by documented asynchronous single-order endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsyncOrderActionResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Fyers reference ID used to correlate the queued async order.
    pub id_fyers: String,
}

/// Immediate response returned by documented asynchronous multi-order endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsyncMultiOrderActionResponse {
    /// Broker-specific numeric code for the batch request.
    pub code: i64,
    /// Broker status string for the batch request.
    pub s: String,
    /// Human-readable batch message.
    pub message: String,
    /// Per-order queued action outcomes.
    pub data: Vec<MultiOrderItemResponse<AsyncOrderActionResponse>>,
}

/// Response returned by documented synchronous multi-order action endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiOrderActionResponse {
    /// Broker-specific numeric code for the batch request.
    pub code: i64,
    /// Broker status string for the batch request.
    pub s: String,
    /// Human-readable batch message.
    pub message: String,
    /// Per-order action outcomes.
    pub data: Vec<MultiOrderItemResponse<OrderActionResponse>>,
}

/// Per-item response wrapper returned by documented multi-order endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiOrderItemResponse<T> {
    /// HTTP status code for this individual order action.
    #[serde(rename = "statusCode")]
    pub status_code: i64,
    /// The broker action response body for this individual order action.
    pub body: T,
    /// HTTP status description for this individual order action.
    #[serde(rename = "statusDescription")]
    pub status_description: String,
}

/// Request body for the documented synchronous multi-leg order placement endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiLegOrderRequest {
    /// Optional caller-defined order tag.
    #[serde(default, rename = "orderTag", skip_serializing_if = "Option::is_none")]
    pub order_tag: Option<String>,
    /// Product type.
    #[serde(rename = "productType")]
    pub product_type: String,
    /// Offline/AMO flag.
    #[serde(rename = "offlineOrder")]
    pub offline_order: bool,
    /// Multi-leg order type, documented as `2L` or `3L`.
    #[serde(rename = "orderType")]
    pub order_type: String,
    /// Order validity, documented as `IOC` for multi-leg orders.
    pub validity: String,
    /// Leg details for the multi-leg order.
    pub legs: MultiLegOrderLegs,
}

/// Container for two-leg and three-leg order legs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiLegOrderLegs {
    /// First leg.
    pub leg1: MultiLegOrderLeg,
    /// Second leg.
    pub leg2: MultiLegOrderLeg,
    /// Third leg, required when `orderType` is `3L`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leg3: Option<MultiLegOrderLeg>,
}

/// Individual multi-leg order leg.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiLegOrderLeg {
    /// Trading symbol.
    pub symbol: String,
    /// Order quantity.
    pub qty: i64,
    /// Buy/sell side code.
    pub side: i64,
    /// Documented order type code.
    #[serde(rename = "type")]
    pub order_type: i64,
    /// Limit price.
    #[serde(rename = "limitPrice")]
    pub limit_price: f64,
}

/// Response returned by the documented order book endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Order book rows.
    #[serde(rename = "orderBook")]
    pub order_book: Vec<OrderBookEntry>,
}

/// Order book row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookEntry {
    /// Fyers user/client ID.
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// Unique order ID assigned for the order.
    pub id: String,
    /// Fyers system-generated unique identifier for the order.
    #[serde(default)]
    pub id_fyers: Option<String>,
    /// Exchange order ID.
    #[serde(rename = "exchOrdId")]
    pub exch_ord_id: String,
    /// Original order quantity.
    pub qty: i64,
    /// Remaining quantity.
    #[serde(rename = "remainingQuantity")]
    pub remaining_quantity: i64,
    /// Filled quantity after partial trades.
    #[serde(rename = "filledQty")]
    pub filled_qty: i64,
    /// Disclosed quantity.
    #[serde(rename = "discloseQty")]
    pub disclose_qty: i64,
    /// Limit price.
    #[serde(rename = "limitPrice")]
    pub limit_price: f64,
    /// Stop price.
    #[serde(rename = "stopPrice")]
    pub stop_price: f64,
    /// Average traded price.
    #[serde(rename = "tradedPrice")]
    pub traded_price: f64,
    /// Documented order type field.
    #[serde(rename = "type")]
    pub order_type: i64,
    /// Fyers token for the symbol.
    #[serde(rename = "fyToken")]
    pub fy_token: String,
    /// Exchange code.
    pub exchange: i64,
    /// Segment code.
    pub segment: i64,
    /// Trading symbol.
    pub symbol: String,
    /// Exchange instrument type.
    pub instrument: i64,
    /// Error/status message for the order.
    pub message: String,
    /// Offline/AMO flag.
    #[serde(rename = "offlineOrder")]
    pub offline_order: bool,
    /// Order time as documented by Fyers.
    #[serde(rename = "orderDateTime")]
    pub order_date_time: String,
    /// Parent order ID for applicable CO/BO child orders.
    #[serde(default, rename = "parentId")]
    pub parent_id: Option<String>,
    /// Order validity.
    #[serde(rename = "orderValidity")]
    pub order_validity: String,
    /// PAN of the client, masked when present.
    pub pan: String,
    /// Product type.
    #[serde(rename = "productType")]
    pub product_type: String,
    /// Order side.
    pub side: i64,
    /// Order status.
    pub status: i64,
    /// Order source.
    pub source: String,
    /// Exchange symbol.
    pub ex_sym: String,
    /// Symbol description.
    pub description: String,
    /// Price change.
    pub ch: f64,
    /// Price change percentage.
    pub chp: f64,
    /// Last price.
    pub lp: f64,
    /// Sort serial number.
    #[serde(rename = "slNo")]
    pub sl_no: i64,
    /// Remaining disclosed quantity.
    #[serde(rename = "dqQtyRem")]
    pub dq_qty_rem: i64,
    /// Combined order number and status.
    #[serde(rename = "orderNumStatus")]
    pub order_num_status: String,
    /// Disclosed quantity.
    #[serde(rename = "disclosedQty")]
    pub disclosed_qty: i64,
    /// Documented order tag.
    #[serde(rename = "orderTag")]
    pub order_tag: String,
    /// Take-profit price for bracket orders.
    #[serde(rename = "takeProfit")]
    pub take_profit: f64,
    /// Stop-loss price for bracket/cover orders.
    #[serde(rename = "stopLoss")]
    pub stop_loss: f64,
}

fn deserialize_string_from_value<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::String(value) => Ok(value),
        Value::Number(value) => Ok(value.to_string()),
        other => Err(serde::de::Error::custom(format!(
            "expected string or number, got {other}"
        ))),
    }
}
