#![allow(dead_code)]

use fyers_rs::{FyersClient, FyersError};

async fn run() -> Result<(), FyersError> {
    let client = env_client()?;

    let profile = client.profile().get().await?;
    let funds = client.funds().get().await?;
    let holdings = client.holdings().get().await?;

    println!("{profile:?}");
    println!("{funds:?}");
    println!("{holdings:?}");
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
