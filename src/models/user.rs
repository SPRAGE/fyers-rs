//! User, funds, and holdings response models.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Response returned by the documented profile endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Profile details for the authenticated user.
    pub data: ProfileData,
}

/// Basic profile details for the authenticated user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileData {
    /// Name of the client.
    pub name: String,
    /// URL link to the user's profile picture, if any.
    pub image: String,
    /// Display name, if any, provided by the client.
    pub display_name: String,
    /// Email address of the client.
    pub email_id: String,
    /// PAN of the client. Fyers docs show both `PAN` in samples and `pan` in the table.
    #[serde(rename = "PAN", alias = "pan")]
    pub pan: String,
    /// Fyers user ID.
    pub fy_id: String,
    /// Date when the PIN was last updated.
    pub pin_change_date: String,
    /// Registered mobile number.
    pub mobile_number: String,
    /// TOTP status.
    pub totp: bool,
    /// Date when the password was last updated.
    pub pwd_change_date: String,
    /// Number of days until the current password expires.
    pub pwd_to_expire: i64,
    /// DDPI status.
    pub ddpi_enabled: bool,
    /// MTF status.
    pub mtf_enabled: bool,
}

/// Response returned by the documented funds endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FundsResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Fund ledger rows.
    pub fund_limit: Vec<FundLimit>,
}

/// Fund ledger row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FundLimit {
    /// Unique identity for the fund row.
    pub id: i64,
    /// Ledger title.
    pub title: String,
    /// Capital ledger amount for the title.
    #[serde(rename = "equityAmount")]
    pub equity_amount: f64,
    /// Commodity ledger amount for the title.
    #[serde(rename = "commodityAmount")]
    pub commodity_amount: f64,
}

/// Response returned by the documented holdings endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoldingsResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Individual holdings.
    pub holdings: Vec<Holding>,
    /// Aggregate holdings summary.
    pub overall: HoldingsOverall,
}

/// Individual holding row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holding {
    /// Type of holding.
    #[serde(rename = "holdingType")]
    pub holding_type: String,
    /// Quantity held at the beginning of the day.
    pub quantity: i64,
    /// Original buy price.
    #[serde(rename = "costPrice")]
    pub cost_price: f64,
    /// Current market value.
    #[serde(rename = "marketVal")]
    pub market_val: f64,
    /// Quantity remaining after sells during the day.
    #[serde(rename = "remainingQuantity")]
    pub remaining_quantity: i64,
    /// Profit and loss made.
    pub pl: f64,
    /// Last traded price.
    pub ltp: f64,
    /// Unique value for each holding.
    pub id: i64,
    /// Fyers token for the symbol.
    #[serde(
        rename = "fyToken",
        alias = "fytoken",
        deserialize_with = "deserialize_string_from_value"
    )]
    pub fy_token: String,
    /// Exchange code.
    pub exchange: i64,
    /// Trading symbol.
    pub symbol: String,
    /// Segment code.
    pub segment: i64,
    /// ISIN for the holding.
    pub isin: String,
    /// T+1 quantity.
    pub qty_t1: i64,
    /// Remaining pledged quantity.
    #[serde(rename = "remainingPledgeQuantity")]
    pub remaining_pledge_quantity: i64,
    /// Collateral quantity.
    #[serde(rename = "collateralQuantity")]
    pub collateral_quantity: i64,
}

/// Aggregate holdings summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoldingsOverall {
    /// Total number of holdings present.
    pub count_total: i64,
    /// Invested amount for current holdings.
    pub total_investment: f64,
    /// Present value of current holdings.
    pub total_current_value: f64,
    /// Total profit and loss made.
    pub total_pl: f64,
    /// Total P&L percentage.
    pub pnl_perc: f64,
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
