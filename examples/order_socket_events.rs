use std::time::Duration;

use fyers_rs::{FyersClient, FyersError};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), FyersError> {
    let client = FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .access_token(std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required"))
        .build()?;

    println!("connecting to Fyers order socket...");
    let mut socket = client.order_socket().connect().await?;
    println!("connected. subscribing to orders/trades/positions...");

    socket
        .subscribe(vec![
            "orders".to_owned(),
            "trades".to_owned(),
            "positions".to_owned(),
        ])
        .await?;
    socket.ping().await?;
    println!("subscribed + pinged. waiting up to 15s for any frame...");

    for tick in 1..=15 {
        match timeout(Duration::from_secs(1), socket.next_event()).await {
            Ok(Ok(Some(event))) => {
                println!("[{tick:>2}s] event: {event:?}");
            }
            Ok(Ok(None)) => {
                println!("[{tick:>2}s] stream closed by server");
                break;
            }
            Ok(Err(err)) => {
                eprintln!("[{tick:>2}s] socket error: {err}");
                break;
            }
            Err(_) => {
                println!("[{tick:>2}s] no frame yet");
            }
        }
    }

    socket.close().await?;
    Ok(())
}
