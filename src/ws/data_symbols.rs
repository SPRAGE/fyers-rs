//! Resolve user-facing symbols (`NSE:SBIN-EQ`) into HSM topic strings
//! (`sf|nse_cm|3045`) that the V3 data socket subscribe frame expects.
//!
//! Mirrors the Python SDK's `SymbolConversion.symbol_to_hsmtoken` flow: POST
//! to `https://api-t1.fyers.in/data/symbol-token`, take the returned
//! `validSymbol[<sym>]` fytoken, and slice it into segment + exchange-token
//! components using the documented exch_seg map.

use std::collections::HashMap;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::FyersClient;
use crate::error::{FyersError, Result};
use crate::models::ws::DataSubscriptionKind;
use crate::transport::{join_base_path, required_authorization_header};

/// Documented exchange-segment ID prefix → HSM segment string.
fn exch_segment(prefix: &str) -> Option<&'static str> {
    Some(match prefix {
        "1010" => "nse_cm",
        "1011" => "nse_fo",
        "1012" => "cde_fo",
        "1020" => "nse_com",
        "1120" => "mcx_fo",
        "1210" => "bse_cm",
        "1211" => "bse_fo",
        "1212" => "bcs_fo",
        _ => return None,
    })
}

#[derive(Debug, Clone, Serialize)]
struct SymbolTokenRequest<'a> {
    symbols: &'a [String],
}

/// One resolved `(input symbol, HSM topic)` pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSymbol {
    pub input_symbol: String,
    pub hsm_topic: String,
}

/// Outcome of a `resolve_hsm_symbols` call.
#[derive(Debug, Clone, Default)]
pub struct ResolvedSymbols {
    pub resolved: Vec<ResolvedSymbol>,
    pub invalid: Vec<String>,
}

impl ResolvedSymbols {
    /// HSM topic strings in resolution order.
    pub fn hsm_topics(&self) -> Vec<String> {
        self.resolved.iter().map(|r| r.hsm_topic.clone()).collect()
    }

    /// Map from HSM topic back to the input symbol string.
    pub fn topic_to_input(&self) -> HashMap<String, String> {
        self.resolved
            .iter()
            .map(|r| (r.hsm_topic.clone(), r.input_symbol.clone()))
            .collect()
    }
}

/// Resolve broker symbols into HSM topic strings for the given subscription kind.
pub async fn resolve_hsm_symbols(
    client: &FyersClient,
    symbols: &[String],
    kind: DataSubscriptionKind,
) -> Result<ResolvedSymbols> {
    if symbols.is_empty() {
        return Ok(ResolvedSymbols::default());
    }

    let url = join_base_path(client.config().data_base_url(), "symbol-token");
    let auth = required_authorization_header(client.config())?;
    let response = client
        .http()
        .post(url)
        .header(AUTHORIZATION, auth)
        .header(CONTENT_TYPE, "application/json")
        .json(&SymbolTokenRequest { symbols })
        .send()
        .await?;

    let status = response.status();
    let body: Value = response.json().await?;

    if !status.is_success() || body.get("s").and_then(Value::as_str) == Some("error") {
        let message = body
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("symbol-token request failed");
        return Err(FyersError::Validation(format!(
            "symbol-token: {message} (status={status})"
        )));
    }

    let valid = body
        .get("validSymbol")
        .and_then(Value::as_object)
        .ok_or_else(|| FyersError::Validation("symbol-token: missing validSymbol".to_owned()))?;
    let invalid_array = body
        .get("invalidSymbol")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut resolved = Vec::with_capacity(valid.len());
    for symbol in symbols {
        let Some(fytoken_v) = valid.get(symbol) else {
            continue;
        };
        let Some(fytoken) = fytoken_v.as_str() else {
            continue;
        };
        let topic = hsm_topic_from_fytoken(symbol, fytoken, kind)?;
        resolved.push(ResolvedSymbol {
            input_symbol: symbol.clone(),
            hsm_topic: topic,
        });
    }

    let invalid = invalid_array
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();

    Ok(ResolvedSymbols { resolved, invalid })
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct InvalidSymbol {
    symbol: String,
}

fn hsm_topic_from_fytoken(
    symbol: &str,
    fytoken: &str,
    kind: DataSubscriptionKind,
) -> Result<String> {
    if fytoken.len() < 11 {
        return Err(FyersError::Validation(format!(
            "symbol-token: fytoken too short for {symbol}: {fytoken}"
        )));
    }
    let ex_sg = &fytoken[..4];
    let segment = exch_segment(ex_sg).ok_or_else(|| {
        FyersError::Validation(format!(
            "symbol-token: unknown exchange-segment {ex_sg} for {symbol}"
        ))
    })?;
    let exch_token = &fytoken[10..];
    let is_index = symbol
        .rsplit('-')
        .next()
        .map(|seg| seg.eq_ignore_ascii_case("INDEX"))
        .unwrap_or(false);

    let prefix = match (kind, is_index) {
        (DataSubscriptionKind::SymbolUpdate, true) => "if",
        (DataSubscriptionKind::SymbolUpdate, false) => "sf",
        (DataSubscriptionKind::DepthUpdate, true) => {
            return Err(FyersError::Validation(format!(
                "symbol-token: depth updates are not available for index symbol {symbol}"
            )));
        }
        (DataSubscriptionKind::DepthUpdate, false) => "dp",
    };

    Ok(format!("{prefix}|{segment}|{exch_token}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fytoken_decodes_to_sf_topic_for_equity_symbol_update() {
        let topic = hsm_topic_from_fytoken(
            "NSE:SBIN-EQ",
            "10100000003045",
            DataSubscriptionKind::SymbolUpdate,
        )
        .unwrap();
        assert_eq!(topic, "sf|nse_cm|3045");
    }

    #[test]
    fn fytoken_decodes_to_if_topic_for_index_symbol_update() {
        // Format: 4 chars exch-seg + 6 chars padding + remaining = exch token.
        let fytoken = "101000000026000";
        assert_eq!(fytoken.len(), 15);
        let topic = hsm_topic_from_fytoken(
            "NSE:NIFTY50-INDEX",
            fytoken,
            DataSubscriptionKind::SymbolUpdate,
        )
        .unwrap();
        assert_eq!(topic, "if|nse_cm|26000");
    }

    #[test]
    fn fytoken_decodes_to_dp_topic_for_equity_depth() {
        let topic = hsm_topic_from_fytoken(
            "NSE:SBIN-EQ",
            "10100000003045",
            DataSubscriptionKind::DepthUpdate,
        )
        .unwrap();
        assert_eq!(topic, "dp|nse_cm|3045");
    }

    #[test]
    fn fytoken_rejects_depth_for_index() {
        let err = hsm_topic_from_fytoken(
            "NSE:NIFTY50-INDEX",
            "10100000026000",
            DataSubscriptionKind::DepthUpdate,
        )
        .unwrap_err();
        assert!(format!("{err}").contains("depth updates are not available"));
    }

    #[test]
    fn fytoken_rejects_unknown_segment() {
        let err = hsm_topic_from_fytoken(
            "NSE:SBIN-EQ",
            "99990000003045",
            DataSubscriptionKind::SymbolUpdate,
        )
        .unwrap_err();
        assert!(format!("{err}").contains("unknown exchange-segment 9999"));
    }

    #[test]
    fn fytoken_rejects_short_token() {
        let err = hsm_topic_from_fytoken(
            "NSE:X",
            "1010",
            DataSubscriptionKind::SymbolUpdate,
        )
        .unwrap_err();
        assert!(format!("{err}").contains("fytoken too short"));
    }
}
