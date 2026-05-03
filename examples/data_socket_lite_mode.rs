//! Lite-mode smoke check.
//!
//! Configures `DataSocketConfig { lite_mode: true, ... }` so that the auth
//! frame carries the lite-mode marker and the channel-mode set message uses
//! `0x4c` instead of `0x46`. Subscribes to one symbol and prints a few
//! events. In lite mode each tick is a one-field i32 array containing only
//! `ltp` (×100), so the typed `SymbolUpdate` will have all other fields
//! `None` / `0`.

use std::time::{Duration, Instant};

use fyers_rs::models::ws::{
    DataSocketConfig, DataSocketEvent, DataSubscribeRequest, DataSubscriptionKind,
};
use fyers_rs::{FyersClient, FyersError};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), FyersError> {
    let client = FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .access_token(std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required"))
        .build()?;

    let symbol =
        std::env::var("FYERS_LIVE_SYMBOL").unwrap_or_else(|_| "NSE:SBIN-EQ".to_owned());
    println!("connecting to data socket in LITE mode for {symbol:?}...");
    let config = DataSocketConfig {
        lite_mode: true,
        ..DataSocketConfig::default()
    };
    let mut socket = client.data_socket().connect_with_config(config).await?;

    socket
        .subscribe(&DataSubscribeRequest {
            symbols: vec![symbol.clone()],
            data_type: DataSubscriptionKind::SymbolUpdate,
        })
        .await?;
    println!("subscribed. waiting for lite events...");

    let started = Instant::now();
    let mut count = 0_u64;
    loop {
        match timeout(Duration::from_secs(5), socket.next_event()).await {
            Ok(Ok(Some(event))) => {
                count += 1;
                match &event {
                    DataSocketEvent::SymbolUpdate(update) => {
                        println!(
                            "[{count:>4}] LITE {sym} ltp={ltp} type={t}",
                            sym = update.symbol,
                            ltp = update.ltp,
                            t = update.event_type
                        );
                    }
                    other => println!("[{count:>4}] {other:?}"),
                }
            }
            Ok(Ok(None)) => {
                println!("stream closed after {count} events");
                break;
            }
            Ok(Err(err)) => {
                eprintln!("socket error: {err}");
                break;
            }
            Err(_) => {
                println!(
                    "no events for 5s ({count} so far, {}s elapsed)",
                    started.elapsed().as_secs()
                );
            }
        }
    }

    socket.close().await?;
    Ok(())
}
