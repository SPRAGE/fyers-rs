#![allow(dead_code)]

use fyers_rs::models::market_data::HistoryRequest;
use fyers_rs::{FyersClient, FyersError};

async fn run() -> Result<(), FyersError> {
    let client = env_client()?;
    let history = client
        .market_data()
        .history(&HistoryRequest {
            symbol: "NSE:SBIN-EQ".to_owned(),
            resolution: "D".to_owned(),
            date_format: 1,
            range_from: "2024-01-01".to_owned(),
            range_to: "2024-01-31".to_owned(),
            cont_flag: None,
            oi_flag: None,
        })
        .await?;

    println!("{history:?}");
    Ok(())
}

fn env_client() -> Result<FyersClient, FyersError> {
    FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .access_token(std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required"))
        .build()
}

fn main() {
    println!("Set FYERS_CLIENT_ID and FYERS_ACCESS_TOKEN, then call run() from an async runtime.");
}
