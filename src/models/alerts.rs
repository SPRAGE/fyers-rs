//! Price alert request and response models.

use serde::{Deserialize, Serialize};

/// Request body for creating or modifying a price alert.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceAlertRequest {
    pub symbol: String,
    pub agent: String,
    #[serde(rename = "alert-type")]
    pub alert_type: i64,
    #[serde(rename = "comparisonType")]
    pub comparison_type: String,
    pub condition: String,
    pub value: f64,
    pub name: String,
    #[serde(default, rename = "alertId", skip_serializing_if = "Option::is_none")]
    pub alert_id: Option<String>,
}

pub type CreatePriceAlertRequest = PriceAlertRequest;
pub type ModifyPriceAlertRequest = PriceAlertRequest;

/// Request body for deleting a price alert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletePriceAlertRequest {
    #[serde(rename = "alertId")]
    pub alert_id: String,
    pub agent: String,
}

/// Request body for toggling a price alert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TogglePriceAlertRequest {
    #[serde(rename = "alertId")]
    pub alert_id: String,
    pub agent: String,
}

/// Query parameters for listing price alerts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceAlertQuery {
    pub name: Option<String>,
    #[serde(default, rename = "alertId", skip_serializing_if = "Option::is_none")]
    pub alert_id: Option<String>,
}

/// Common response returned by price-alert mutations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceAlertActionResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
}

/// Response returned by the price-alert listing endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceAlertsResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    pub data: std::collections::BTreeMap<String, AlertItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertItem {
    #[serde(rename = "fyToken")]
    pub fy_token: String,
    pub symbol: String,
    pub alert: AlertDetails,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertDetails {
    #[serde(rename = "comparisonType")]
    pub comparison_type: String,
    pub condition: String,
    pub name: String,
    #[serde(rename = "type")]
    pub alert_type: String,
    pub value: f64,
    #[serde(rename = "triggeredAt")]
    pub triggered_at: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(default, rename = "modifiedAt")]
    pub modified_at: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub status: i64,
    #[serde(default, rename = "triggeredEpoch")]
    pub triggered_epoch: Option<i64>,
    #[serde(default, rename = "createdEpoch")]
    pub created_epoch: Option<i64>,
    #[serde(default, rename = "modifiedEpoch")]
    pub modified_epoch: Option<i64>,
}
