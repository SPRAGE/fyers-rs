#![cfg(feature = "live-tests")]

mod support;

use fyers_rs::FyersClient;
use fyers_rs::models::ws::{DataSubscribeRequest, DataSubscriptionKind};
use tokio::time::{Duration, timeout};

fn live_client(config: &support::LiveTestConfig) -> FyersClient {
    config.client()
}

#[tokio::test]
#[ignore = "requires live Fyers WebSocket credentials; read-only data socket"]
async fn live_data_socket_can_connect_subscribe_wait_and_close() {
    let Some(config) = support::LiveTestConfig::from_env("live_data_socket") else {
        return;
    };
    let client = live_client(&config);
    let mut socket = client.data_socket().connect().await.expect("connect");
    let request = DataSubscribeRequest {
        symbols: vec![config.symbol],
        data_type: DataSubscriptionKind::SymbolUpdate,
    };

    socket.subscribe(&request).await.expect("subscribe");
    let _ = timeout(Duration::from_secs(10), socket.next_event()).await;
    socket.close().await.expect("close");
}

#[tokio::test]
#[ignore = "requires live Fyers WebSocket credentials; read-only order socket"]
async fn live_order_socket_can_connect_subscribe_ping_and_close() {
    let Some(config) = support::LiveTestConfig::from_env("live_order_socket") else {
        return;
    };
    let client = live_client(&config);
    let mut socket = client.order_socket().connect().await.expect("connect");

    socket
        .subscribe(vec![
            "orders".to_owned(),
            "trades".to_owned(),
            "positions".to_owned(),
        ])
        .await
        .expect("subscribe");
    socket.ping().await.expect("ping");
    let _ = timeout(Duration::from_secs(10), socket.next_event()).await;
    socket.close().await.expect("close");
}
