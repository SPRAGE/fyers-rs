//! Market data request and response models.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Response returned by the market-status endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketStatusResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    #[serde(rename = "marketStatus")]
    pub market_status: Vec<MarketStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketStatus {
    pub exchange: i64,
    pub market_type: String,
    pub segment: i64,
    pub status: String,
}

/// Query parameters for historical candles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryRequest {
    pub symbol: String,
    pub resolution: String,
    pub date_format: i64,
    pub range_from: String,
    pub range_to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cont_flag: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oi_flag: Option<i64>,
}

/// Historical candle response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryResponse {
    #[serde(default)]
    pub code: Option<i64>,
    pub s: String,
    #[serde(default)]
    pub message: Option<String>,
    pub candles: Vec<Vec<f64>>,
}

/// Query parameters for quotes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotesRequest {
    pub symbols: Vec<String>,
}

/// Quotes response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuotesResponse {
    pub code: i64,
    pub s: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(rename = "d")]
    pub data: Vec<StockQuote>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockQuote {
    pub n: String,
    pub s: String,
    pub v: QuoteValues,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteValues {
    pub ch: f64,
    pub chp: f64,
    pub lp: f64,
    pub spread: f64,
    pub ask: f64,
    pub bid: f64,
    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub prev_close_price: f64,
    pub atp: f64,
    pub volume: i64,
    pub short_name: String,
    pub exchange: String,
    pub description: String,
    pub original_name: String,
    pub symbol: String,
    #[serde(rename = "fyToken", alias = "fytoken")]
    pub fy_token: String,
    #[serde(deserialize_with = "deserialize_string_from_value")]
    pub tt: String,
}

/// Query parameters for market depth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDepthRequest {
    pub symbol: String,
    pub ohlcv_flag: i64,
}

/// Market depth response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketDepthResponse {
    #[serde(default)]
    pub code: Option<i64>,
    pub s: String,
    pub message: String,
    #[serde(rename = "d")]
    pub data: std::collections::BTreeMap<String, MarketDepthSymbol>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketDepthSymbol {
    pub totalbuyqty: i64,
    pub totalsellqty: i64,
    pub ask: Vec<DepthLevel>,
    pub bids: Vec<DepthLevel>,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub chp: f64,
    #[serde(rename = "tick_Size")]
    pub tick_size: f64,
    pub ch: f64,
    pub ltq: i64,
    pub ltt: i64,
    pub ltp: f64,
    pub v: i64,
    pub atp: f64,
    pub lower_ckt: f64,
    pub upper_ckt: f64,
    pub expiry: String,
    pub oi: f64,
    pub oiflag: bool,
    pub pdoi: i64,
    pub oipercent: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepthLevel {
    pub price: f64,
    pub volume: i64,
    pub ord: i64,
}

/// Query parameters for option-chain data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionChainRequest {
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikecount: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub greeks: Option<String>,
}

/// Option-chain response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionChainResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    pub data: OptionChainData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionChainData {
    #[serde(rename = "callOi")]
    pub call_oi: i64,
    #[serde(rename = "putOi")]
    pub put_oi: i64,
    #[serde(rename = "expiryData")]
    pub expiry_data: Vec<ExpiryData>,
    #[serde(rename = "indiavixData")]
    pub indiavix_data: OptionChainQuote,
    #[serde(rename = "optionsChain")]
    pub options_chain: Vec<OptionChainQuote>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpiryData {
    pub date: String,
    pub expiry: String,
    #[serde(default)]
    pub expiry_flag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionChainQuote {
    pub ask: f64,
    pub bid: f64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub ex_symbol: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_from_value")]
    pub exchange: Option<String>,
    #[serde(default)]
    pub fp: Option<f64>,
    #[serde(default)]
    pub fpch: Option<f64>,
    #[serde(default)]
    pub fpchp: Option<f64>,
    #[serde(rename = "fyToken", alias = "fytoken")]
    pub fy_token: String,
    pub ltp: f64,
    pub ltpch: f64,
    pub ltpchp: f64,
    pub option_type: String,
    pub strike_price: f64,
    pub symbol: String,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub oi: Option<i64>,
    #[serde(default)]
    pub oich: Option<i64>,
    #[serde(default)]
    pub oichp: Option<f64>,
    #[serde(default)]
    pub prev_oi: Option<i64>,
    #[serde(default)]
    pub greeks: Option<OptionGreeks>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionGreeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub iv: f64,
}

/// Supported public symbol-master exchange/segment files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolMasterExchangeSegment {
    NseCd,
    NseFo,
    NseCom,
    NseCm,
    BseCm,
    BseFo,
    McxCom,
}

impl SymbolMasterExchangeSegment {
    pub const fn csv_file_name(self) -> &'static str {
        match self {
            Self::NseCd => "NSE_CD.csv",
            Self::NseFo => "NSE_FO.csv",
            Self::NseCom => "NSE_COM.csv",
            Self::NseCm => "NSE_CM.csv",
            Self::BseCm => "BSE_CM.csv",
            Self::BseFo => "BSE_FO.csv",
            Self::McxCom => "MCX_COM.csv",
        }
    }

    pub const fn json_file_name(self) -> &'static str {
        match self {
            Self::NseCd => "NSE_CD_sym_master.json",
            Self::NseFo => "NSE_FO_sym_master.json",
            Self::NseCom => "NSE_COM_sym_master.json",
            Self::NseCm => "NSE_CM_sym_master.json",
            Self::BseCm => "BSE_CM_sym_master.json",
            Self::BseFo => "BSE_FO_sym_master.json",
            Self::McxCom => "MCX_COM_sym_master.json",
        }
    }
}

/// Symbol-master JSON payload.
pub type SymbolMasterJson = serde_json::Value;

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

fn deserialize_optional_string_from_value<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(Value::Number(value)) => Ok(Some(value.to_string())),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected string, number, or null, got {other}"
        ))),
    }
}
