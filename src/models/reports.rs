//! Report history request and response models.

use serde::{Deserialize, Serialize};

/// Query parameters for the documented order history endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderHistoryQuery {
    /// Optional exchange filter.
    #[serde(
        default,
        rename = "exchange_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub exchange_type: Option<String>,
    /// Optional segment filter.
    #[serde(
        default,
        rename = "segment_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub segment_type: Option<String>,
    /// Optional order status filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Optional symbol filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Optional start date filter in the broker-documented format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_date: Option<String>,
    /// Optional end date filter in the broker-documented format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
    /// Optional page number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_no: Option<u32>,
    /// Optional page size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

impl OrderHistoryQuery {
    /// Query order history by symbol.
    pub fn by_symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: Some(symbol.into()),
            ..Self::default()
        }
    }
}

/// Response returned by the documented order history endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderHistoryResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Historical order rows.
    pub data: Vec<OrderHistoryEntry>,
}

/// Historical order row returned by the documented order-history endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderHistoryEntry {
    /// Trading symbol.
    pub symbol: String,
    /// Fyers user/client ID.
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// Fyers system-generated unique identifier for the order.
    pub id_fyers: String,
    /// Exchange order ID.
    #[serde(rename = "exchOrdId")]
    pub exch_ord_id: String,
    /// Exchange code.
    pub exchange: i64,
    /// Segment code.
    pub segment: i64,
    /// Exchange instrument type.
    pub instrument: i64,
    /// Symbol description.
    pub description: String,
    /// Epoch time when the trade/order occurred.
    pub trade_date_time: i64,
    /// Trade/order time in IST as documented. The docs sample includes a trailing-space key.
    #[serde(rename = "trade_date ", alias = "trade_date")]
    pub trade_date: String,
    /// Transaction type, such as BUY or SELL.
    pub transaction_type: String,
    /// Product type.
    pub product_type: String,
    /// Status text.
    pub status: String,
    /// Order type text.
    pub ordertype: String,
    /// Ordered quantity.
    pub qty: i64,
    /// Total traded quantity.
    #[serde(rename = "tradedqty", alias = "tradedQty")]
    pub traded_qty: i64,
    /// Traded price.
    pub traded_price: f64,
    /// Limit price.
    pub limit_price: f64,
    /// Order source.
    pub ord_source: String,
    /// Rejection reason, if any.
    pub rejection_reason: String,
    /// Whether the symbol is currently active.
    pub is_symbol_active: bool,
}

/// Query parameters for the documented trade history endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeHistoryQuery {
    /// Optional exchange filter.
    #[serde(
        default,
        rename = "exchange_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub exchange_type: Option<String>,
    /// Optional segment filter.
    #[serde(
        default,
        rename = "segment_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub segment_type: Option<String>,
    /// Optional symbol filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Optional start date filter in the broker-documented format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_date: Option<String>,
    /// Optional end date filter in the broker-documented format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
    /// Optional page number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_no: Option<u32>,
    /// Optional page size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

impl TradeHistoryQuery {
    /// Query trade history by symbol.
    pub fn by_symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: Some(symbol.into()),
            ..Self::default()
        }
    }
}

/// Response returned by the documented trade history endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeHistoryResponse {
    /// Broker-specific numeric code, omitted in some documented samples.
    #[serde(default)]
    pub code: Option<i64>,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Historical trade rows.
    pub data: Vec<TradeHistoryEntry>,
}

/// Historical trade row returned by the documented trade-history endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeHistoryEntry {
    /// Trading symbol.
    pub symbol: String,
    /// Fyers user/client ID.
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// Symbol description.
    pub description: String,
    /// Order date and time.
    #[serde(rename = "orderDateTime")]
    pub order_date_time: String,
    /// Broker order number.
    #[serde(rename = "orderNumber")]
    pub order_number: String,
    /// Exchange trade number.
    #[serde(rename = "tradeNumber")]
    pub trade_number: String,
    /// Exchange order number.
    #[serde(rename = "exchangeOrderNo")]
    pub exchange_order_no: String,
    /// Trade side.
    pub side: i64,
    /// Exchange code.
    pub exchange: i64,
    /// Segment code.
    pub segment: i64,
    /// Product type.
    pub product_type: String,
    /// Traded quantity.
    pub traded_qty: i64,
    /// Trade price.
    pub trade_price: f64,
    /// Trade value.
    pub trade_value: f64,
    /// Whether the symbol is currently active.
    pub is_symbol_active: bool,
}
