#![allow(dead_code)]

use fyers_rs::{FyersClient, FyersError};

async fn run() -> Result<(), FyersError> {
    let client = env_client()?;
    let mut socket = client.order_socket().connect().await?;

    socket
        .subscribe(vec![
            "orders".to_owned(),
            "trades".to_owned(),
            "positions".to_owned(),
        ])
        .await?;
    socket.ping().await?;
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
