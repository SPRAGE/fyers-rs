#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use fyers_rs::FyersClient;

pub fn fixture_path(relative_path: impl AsRef<Path>) -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative_path.as_ref());

    assert!(path.exists(), "fixture does not exist: {}", path.display());

    path
}

pub fn read_fixture(relative_path: impl AsRef<Path>) -> String {
    let path = fixture_path(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("failed to read fixture {}: {err}", path.display());
    })
}

pub fn read_fixture_bytes(relative_path: impl AsRef<Path>) -> Vec<u8> {
    let path = fixture_path(relative_path);
    fs::read(&path).unwrap_or_else(|err| {
        panic!("failed to read fixture {}: {err}", path.display());
    })
}

pub fn json_fixture<T>(relative_path: impl AsRef<Path>) -> T
where
    T: DeserializeOwned,
{
    let path = relative_path.as_ref().to_path_buf();
    let fixture = read_fixture(&path);

    serde_json::from_str(&fixture).unwrap_or_else(|err| {
        panic!("failed to parse JSON fixture {}: {err}", path.display());
    })
}

pub fn json_value(relative_path: impl AsRef<Path>) -> serde_json::Value {
    json_fixture(relative_path)
}

#[derive(Debug, Clone)]
pub struct LiveTestConfig {
    pub client_id: String,
    pub access_token: String,
    pub symbol: String,
    pub index_symbol: String,
    pub option_chain_symbol: String,
    pub history_range_from: String,
    pub history_range_to: String,
}

impl LiveTestConfig {
    pub fn from_env(test_name: &str) -> Option<Self> {
        let client_id = optional_env("FYERS_CLIENT_ID", test_name)?;
        let access_token = optional_env("FYERS_ACCESS_TOKEN", test_name)?;

        Some(Self {
            client_id,
            access_token,
            symbol: std::env::var("FYERS_LIVE_SYMBOL").unwrap_or_else(|_| "NSE:SBIN-EQ".to_owned()),
            index_symbol: std::env::var("FYERS_LIVE_INDEX_SYMBOL")
                .unwrap_or_else(|_| "NSE:NIFTY50-INDEX".to_owned()),
            option_chain_symbol: std::env::var("FYERS_LIVE_OPTION_CHAIN_SYMBOL")
                .unwrap_or_else(|_| "NSE:NIFTY50-INDEX".to_owned()),
            history_range_from: std::env::var("FYERS_LIVE_HISTORY_FROM")
                .unwrap_or_else(|_| "2024-01-01".to_owned()),
            history_range_to: std::env::var("FYERS_LIVE_HISTORY_TO")
                .unwrap_or_else(|_| "2024-01-31".to_owned()),
        })
    }

    pub fn client(&self) -> FyersClient {
        FyersClient::builder()
            .client_id(self.client_id.clone())
            .access_token(self.access_token.clone())
            .build()
            .expect("live client should build")
    }
}

fn optional_env(name: &str, test_name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => {
            eprintln!("skipping {test_name}: {name} is not set");
            None
        }
    }
}
