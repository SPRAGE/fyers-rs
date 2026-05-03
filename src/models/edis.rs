//! EDIS request and response models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdisTpinResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdisDetailsResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    pub data: Vec<EdisDetail>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdisDetail {
    #[serde(rename = "clientId")]
    pub client_id: String,
    pub isin: String,
    pub qty: f64,
    #[serde(rename = "qtyUtlize")]
    pub qty_utlize: f64,
    #[serde(rename = "entryDate")]
    pub entry_date: String,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
    #[serde(rename = "noOfDays")]
    pub no_of_days: i64,
    pub source: String,
    pub status: String,
    pub reason: String,
    #[serde(rename = "internalTxnId")]
    pub internal_txn_id: String,
    #[serde(rename = "dpTxnId")]
    pub dp_txn_id: String,
    #[serde(rename = "errCode")]
    pub err_code: String,
    #[serde(rename = "errorCount")]
    pub error_count: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdisIndexRequest {
    #[serde(rename = "recordLst")]
    pub record_lst: Vec<EdisIndexRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdisIndexRecord {
    pub isin_code: String,
    pub qty: String,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdisIndexResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdisInquiryRequest {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdisInquiryResponse {
    pub code: i64,
    pub s: String,
    pub message: String,
    pub data: EdisInquiryData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdisInquiryData {
    #[serde(rename = "FAILED_CNT")]
    pub failed_count: i64,
    #[serde(rename = "SUCEESS_CNT")]
    pub success_count: i64,
}
