//! Transaction read response models for trades and positions.

use serde::{Deserialize, Serialize};

/// Query parameters for the documented trade book endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeBookQuery {
    /// Optional order tag filter.
    pub order_tag: Option<String>,
}

impl TradeBookQuery {
    /// Query trades by documented order tag parameter.
    pub fn by_order_tag(order_tag: impl Into<String>) -> Self {
        Self {
            order_tag: Some(order_tag.into()),
        }
    }
}

/// Response returned by the documented trade book endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeBookResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Trade book rows.
    #[serde(rename = "tradeBook")]
    pub trade_book: Vec<TradeBookEntry>,
}

/// Trade book row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeBookEntry {
    /// Fyers user/client ID.
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// Order date and time.
    #[serde(rename = "orderDateTime")]
    pub order_date_time: String,
    /// Broker order number.
    #[serde(rename = "orderNumber")]
    pub order_number: String,
    /// Exchange order number.
    #[serde(rename = "exchangeOrderNo")]
    pub exchange_order_no: String,
    /// Exchange code.
    pub exchange: i64,
    /// Trade side.
    pub side: i64,
    /// Segment code.
    pub segment: i64,
    /// Order type.
    #[serde(rename = "orderType")]
    pub order_type: i64,
    /// Fyers token for the symbol.
    #[serde(rename = "fyToken")]
    pub fy_token: String,
    /// Product type.
    #[serde(rename = "productType")]
    pub product_type: String,
    /// Traded quantity.
    #[serde(rename = "tradedQty")]
    pub traded_qty: i64,
    /// Trade price.
    #[serde(rename = "tradePrice")]
    pub trade_price: f64,
    /// Trade value.
    #[serde(rename = "tradeValue")]
    pub trade_value: f64,
    /// Exchange trade number.
    #[serde(rename = "tradeNumber")]
    pub trade_number: String,
    /// Row ID.
    pub row: i64,
    /// Trading symbol.
    pub symbol: String,
    /// Documented order tag.
    #[serde(rename = "orderTag")]
    pub order_tag: String,
}

/// Response returned by the documented positions endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionsResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Net position rows.
    #[serde(rename = "netPositions")]
    pub net_positions: Vec<NetPosition>,
    /// Aggregate positions summary.
    pub overall: PositionsOverall,
}

/// Net position row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetPosition {
    /// Net quantity.
    #[serde(rename = "netQty")]
    pub net_qty: i64,
    /// Absolute value of net quantity.
    pub qty: i64,
    /// Average price from the documented sample.
    #[serde(rename = "avgPrice")]
    pub avg_price: f64,
    /// Net average price.
    #[serde(rename = "netAvg")]
    pub net_avg: f64,
    /// Position side.
    pub side: i64,
    /// Product type.
    #[serde(rename = "productType")]
    pub product_type: String,
    /// Realized profit and loss.
    pub realized_profit: f64,
    /// Unrealized profit and loss.
    pub unrealized_profit: f64,
    /// Total profit and loss.
    pub pl: f64,
    /// Last traded price.
    pub ltp: f64,
    /// Total buy quantity.
    #[serde(rename = "buyQty")]
    pub buy_qty: i64,
    /// Average buy price.
    #[serde(rename = "buyAvg")]
    pub buy_avg: f64,
    /// Buy value.
    #[serde(rename = "buyVal")]
    pub buy_val: f64,
    /// Total sell quantity.
    #[serde(rename = "sellQty")]
    pub sell_qty: i64,
    /// Average sell price.
    #[serde(rename = "sellAvg")]
    pub sell_avg: f64,
    /// Sell value.
    #[serde(rename = "sellVal")]
    pub sell_val: f64,
    /// Deprecated position sort field.
    #[serde(rename = "slNo")]
    pub sl_no: i64,
    /// Fyers token for the symbol.
    #[serde(rename = "fyToken")]
    pub fy_token: String,
    /// Cross-currency flag.
    #[serde(rename = "crossCurrency")]
    pub cross_currency: String,
    /// RBI reference rate.
    #[serde(rename = "rbiRefRate")]
    pub rbi_ref_rate: f64,
    /// Commodity multiplier.
    #[serde(rename = "qtyMulti_com")]
    pub qty_multi_com: f64,
    /// Segment code.
    pub segment: i64,
    /// Trading symbol.
    pub symbol: String,
    /// Unique position ID.
    pub id: String,
    /// Carry-forward buy quantity.
    #[serde(rename = "cfBuyQty")]
    pub cf_buy_qty: i64,
    /// Carry-forward sell quantity.
    #[serde(rename = "cfSellQty")]
    pub cf_sell_qty: i64,
    /// Day buy quantity.
    #[serde(rename = "dayBuyQty")]
    pub day_buy_qty: i64,
    /// Day sell quantity.
    #[serde(rename = "daySellQty")]
    pub day_sell_qty: i64,
    /// Exchange code.
    pub exchange: i64,
}

/// Aggregate positions summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionsOverall {
    /// Total number of positions.
    pub count_total: i64,
    /// Total number of open positions.
    pub count_open: i64,
    /// Total profit and loss.
    pub pl_total: f64,
    /// Realized profit and loss.
    pub pl_realized: f64,
    /// Unrealized profit and loss.
    pub pl_unrealized: f64,
}
