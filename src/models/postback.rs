//! Incoming postback/webhook payload models.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Incoming order-status postback payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderPostback {
    pub id: String,
    #[serde(rename = "exchOrdId")]
    pub exch_ord_id: String,
    pub symbol: String,
    pub qty: i64,
    #[serde(rename = "remainingQuantity")]
    pub remaining_quantity: i64,
    #[serde(rename = "filledQty")]
    pub filled_qty: i64,
    pub status: i64,
    pub message: String,
    pub segment: i64,
    #[serde(rename = "limitPrice")]
    pub limit_price: f64,
    #[serde(rename = "stopPrice")]
    pub stop_price: f64,
    #[serde(rename = "productType")]
    pub product_type: String,
    #[serde(rename = "type")]
    pub order_type: i64,
    pub side: i64,
    #[serde(rename = "discloseQty", alias = "disclosedQty")]
    pub disclose_qty: i64,
    #[serde(rename = "dqQtyRem")]
    pub dq_qty_rem: i64,
    #[serde(rename = "orderValidity")]
    pub order_validity: String,
    #[serde(rename = "orderDateTime")]
    pub order_date_time: String,
    #[serde(rename = "tradedPrice")]
    pub traded_price: f64,
    pub source: String,
    #[serde(rename = "fyToken", alias = "fytoken")]
    pub fy_token: String,
    #[serde(rename = "offlineOrder")]
    pub offline_order: bool,
    pub pan: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    pub exchange: i64,
    /// Docs table says string while the sample sends a number, so retain the exact JSON value.
    pub instrument: Value,
    #[serde(rename = "orderNumStatus")]
    pub order_num_status: String,
}
