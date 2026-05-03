//! Smart order request and response models.

use serde::{Deserialize, Serialize};

/// Request body for documented smart limit order creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartLimitOrderRequest {
    pub symbol: String,
    pub side: i64,
    pub qty: i64,
    #[serde(rename = "productType")]
    pub product_type: String,
    #[serde(rename = "limitPrice")]
    pub limit_price: f64,
    #[serde(rename = "endTime")]
    pub end_time: i64,
    #[serde(rename = "orderType")]
    pub order_type: i64,
    #[serde(rename = "onExp")]
    pub on_exp: i64,
    #[serde(default, rename = "stopPrice", skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpp: Option<f64>,
}

/// Request body for documented smart trailing stop-loss creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartTrailOrderRequest {
    pub symbol: String,
    pub side: i64,
    pub qty: i64,
    #[serde(rename = "productType")]
    pub product_type: String,
    #[serde(rename = "orderType")]
    pub order_type: i64,
    #[serde(rename = "stopPrice")]
    pub stop_price: f64,
    pub jump_diff: f64,
    #[serde(
        default,
        rename = "limitPrice",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_price: Option<f64>,
    #[serde(
        default,
        rename = "target_price",
        skip_serializing_if = "Option::is_none"
    )]
    pub target_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpp: Option<f64>,
}

/// Request body for documented smart step order creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartStepOrderRequest {
    pub symbol: String,
    pub side: i64,
    pub qty: i64,
    #[serde(rename = "productType")]
    pub product_type: String,
    #[serde(rename = "orderType")]
    pub order_type: i64,
    pub avgqty: i64,
    pub avgdiff: f64,
    pub direction: i64,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    #[serde(rename = "endTime")]
    pub end_time: i64,
    #[serde(
        default,
        rename = "limitPrice",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_price: Option<f64>,
    #[serde(default, rename = "initQty", skip_serializing_if = "Option::is_none")]
    pub init_qty: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpp: Option<f64>,
}

/// Request body for documented smart SIP order creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartSipOrderRequest {
    pub symbol: String,
    #[serde(rename = "productType")]
    pub product_type: String,
    pub freq: i64,
    pub sip_day: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qty: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sip_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imd_start: Option<bool>,
    #[serde(default, rename = "endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_up_freq: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_up_qty: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_up_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exp_qty: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<i64>,
}

/// Request body for documented smart order modification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifySmartOrderRequest {
    #[serde(rename = "flowId")]
    pub flow_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qty: Option<i64>,
    #[serde(
        default,
        rename = "limitPrice",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_price: Option<f64>,
    #[serde(default, rename = "stopPrice", skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(
        default,
        rename = "disclosedQty",
        skip_serializing_if = "Option::is_none"
    )]
    pub disclosed_qty: Option<i64>,
    #[serde(default, rename = "endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(default, rename = "startTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpp: Option<f64>,
    #[serde(default, rename = "onExp", skip_serializing_if = "Option::is_none")]
    pub on_exp: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_diff: Option<f64>,
    #[serde(
        default,
        rename = "target_price",
        skip_serializing_if = "Option::is_none"
    )]
    pub target_price: Option<f64>,
    #[serde(
        default,
        rename = "unsetTargetPrice",
        skip_serializing_if = "Option::is_none"
    )]
    pub unset_target_price: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avgqty: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avgdiff: Option<f64>,
    #[serde(default, rename = "initQty", skip_serializing_if = "Option::is_none")]
    pub init_qty: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sip_day: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sip_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_up_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_up_qty: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exp_qty: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exp_amount: Option<f64>,
    #[serde(
        default,
        rename = "productType",
        skip_serializing_if = "Option::is_none"
    )]
    pub product_type: Option<String>,
}

/// Request body for smart order actions that only require a flow ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowIdRequest {
    #[serde(rename = "flowId")]
    pub flow_id: String,
}

/// Query parameters for the documented smart order book endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmartOrderBookQuery {
    pub exchange: Vec<String>,
    pub side: Vec<i64>,
    pub flowtype: Vec<i64>,
    pub product: Vec<String>,
    #[serde(rename = "messageType")]
    pub message_type: Vec<i64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub ord_by: Option<i64>,
    pub page_no: Option<i64>,
    pub page_size: Option<i64>,
}

/// Common smart order action response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmartOrderActionResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    #[serde(default, alias = "flowId")]
    pub id: Option<String>,
}

/// Response returned by the smart order book endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartOrderBookResponse {
    pub code: i64,
    pub s: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(rename = "orderBook")]
    pub order_book: Vec<SmartOrderBookItem>,
    pub count: i64,
    #[serde(rename = "filterCount")]
    pub filter_count: i64,
}

/// Smart order book row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartOrderBookItem {
    #[serde(rename = "flowId")]
    pub flow_id: String,
    pub symbol: String,
    #[serde(rename = "productType")]
    pub product_type: String,
    pub side: i64,
    pub qty: i64,
    pub flowtype: i64,
    pub status: i64,
    #[serde(rename = "filledQty")]
    pub filled_qty: i64,
    #[serde(default, rename = "limitPrice")]
    pub limit_price: Option<f64>,
    #[serde(default, rename = "stopPrice")]
    pub stop_price: Option<f64>,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "updatedTime")]
    pub updated_time: i64,
}
