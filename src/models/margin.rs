//! Margin calculator request and response models.

use serde::{Deserialize, Serialize};

/// Request body for the documented legacy span-margin endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanMarginRequest {
    pub data: Vec<SpanMarginOrder>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanMarginOrder {
    pub symbol: String,
    pub qty: i64,
    pub side: i64,
    #[serde(rename = "type")]
    pub order_type: i64,
    #[serde(rename = "productType")]
    pub product_type: String,
    #[serde(rename = "limitPrice")]
    pub limit_price: f64,
    #[serde(rename = "stopLoss")]
    pub stop_loss: f64,
}

/// Response returned by the documented legacy span-margin endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanMarginResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    pub data: SpanMarginData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanMarginData {
    pub span_margin: f64,
    pub exposure_margin: f64,
    pub total_margin: f64,
}

/// Request body for the documented multiorder margin endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiOrderMarginRequest {
    pub data: Vec<MultiOrderMarginOrder>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiOrderMarginOrder {
    pub symbol: String,
    pub qty: i64,
    pub side: i64,
    #[serde(rename = "type")]
    pub order_type: i64,
    #[serde(rename = "productType")]
    pub product_type: String,
    #[serde(rename = "limitPrice")]
    pub limit_price: f64,
    #[serde(rename = "stopLoss")]
    pub stop_loss: f64,
    #[serde(rename = "stopPrice")]
    pub stop_price: f64,
    #[serde(rename = "takeProfit")]
    pub take_profit: f64,
}

/// Response returned by the documented multiorder margin endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiOrderMarginResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    pub data: MultiOrderMarginData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiOrderMarginData {
    pub margin_avail: f64,
    pub margin_total: f64,
    pub margin_new_order: f64,
}
