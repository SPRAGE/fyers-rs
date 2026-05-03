use std::time::{Duration, Instant};

use fyers_rs::models::ws::{DataSocketEvent, DataSubscribeRequest, DataSubscriptionKind};
use fyers_rs::{FyersClient, FyersError};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), FyersError> {
    let client = FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .access_token(std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required"))
        .build()?;

    let symbols: Vec<String> = std::env::var("FYERS_LIVE_SYMBOL")
        .unwrap_or_else(|_| "NSE:SBIN-EQ".to_owned())
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();

    println!("connecting to Fyers data socket...");
    let mut socket = client.data_socket().connect().await?;
    println!("connected. subscribing to {symbols:?}");

    socket
        .subscribe(&DataSubscribeRequest {
            symbols,
            data_type: DataSubscriptionKind::SymbolUpdate,
        })
        .await?;

    println!("subscribed. waiting for events (Ctrl-C to stop)...");
    let started = Instant::now();
    let mut ticks = 0_u64;
    loop {
        match timeout(Duration::from_secs(5), socket.next_event()).await {
            Ok(Ok(Some(event))) => {
                ticks += 1;
                match &event {
                    DataSocketEvent::SymbolUpdate(update) => {
                        println!(
                            "[{ticks:>5}] {symbol} ltp={ltp} prev_close={prev_close:?} oh_ohlc=({open:?},{high:?},{low:?})",
                            symbol = update.symbol,
                            ltp = update.ltp,
                            prev_close = update.prev_close_price,
                            open = update.open_price,
                            high = update.high_price,
                            low = update.low_price,
                        );
                    }
                    other => println!("[{ticks:>5}] {other:?}"),
                }
            }
            Ok(Ok(None)) => {
                println!("stream closed by server after {ticks} events");
                break;
            }
            Ok(Err(err)) => {
                eprintln!("socket error: {err}");
                break;
            }
            Err(_) => {
                println!(
                    "no events for 5s (total: {ticks} events, elapsed: {}s) — markets may be closed",
                    started.elapsed().as_secs()
                );
            }
        }
    }

    socket.close().await?;
    Ok(())
}
