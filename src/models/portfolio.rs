//! Portfolio, funds, holdings, trades, and position mutation models.

use serde::{Deserialize, Serialize};

/// Request body for exiting all open positions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitAllPositionsRequest {
    pub exit_all: i64,
}

/// Request body for exiting positions by ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitPositionsByIdRequest {
    pub id: PositionExitId,
}

/// Position ID selector accepted by the documented exit-position endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PositionExitId {
    One(String),
    Many(Vec<String>),
}

/// Request body for exiting positions by segment/side/product filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitPositionsByFilterRequest {
    pub segment: Vec<i64>,
    pub side: Vec<i64>,
    #[serde(rename = "productType")]
    pub product_type: Vec<String>,
}

/// Request body for cancelling pending orders during position exit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingOrderCancelRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub pending_orders_cancel: i64,
}

/// Request body for converting a position product type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertPositionRequest {
    pub symbol: String,
    #[serde(rename = "positionSide")]
    pub position_side: i64,
    #[serde(rename = "convertQty")]
    pub convert_qty: i64,
    #[serde(rename = "convertFrom")]
    pub convert_from: String,
    #[serde(rename = "convertTo")]
    pub convert_to: String,
    pub overnight: i64,
}

/// Response returned by the documented position conversion endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertPositionResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    #[serde(rename = "positionDetails")]
    pub position_details: i64,
}
