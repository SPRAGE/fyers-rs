#![allow(dead_code)]

use fyers_rs::models::ws::{DataSubscribeRequest, DataSubscriptionKind};
use fyers_rs::{FyersClient, FyersError};

async fn run() -> Result<(), FyersError> {
    let client = env_client()?;
    let mut socket = client.data_socket().connect().await?;
    let request = DataSubscribeRequest {
        symbols: vec!["NSE:SBIN-EQ".to_owned()],
        data_type: DataSubscriptionKind::SymbolUpdate,
    };

    socket.subscribe(&request).await?;
    if let Some(event) = socket.next_event().await? {
        println!("{event:?}");
    }
    socket.close().await?;

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
