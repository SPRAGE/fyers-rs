#![allow(dead_code)]

use fyers_rs::models::orders::{CancelOrderRequest, ModifyOrderRequest};
use fyers_rs::{FyersClient, FyersError};

async fn run() -> Result<(), FyersError> {
    let client = env_client()?;
    let order_id = std::env::var("FYERS_ORDER_ID").expect("FYERS_ORDER_ID is required");
    let modify = ModifyOrderRequest {
        id: order_id.clone(),
        order_type: 1,
        qty: Some(1),
        side: Some(1),
        limit_price: Some(605.0),
        stop_price: Some(0.0),
        disclosed_qty: None,
        offline_order: None,
    };
    let cancel = CancelOrderRequest { id: order_id };

    println!("modify: {}", serde_json::to_string_pretty(&modify)?);
    println!("cancel: {}", serde_json::to_string_pretty(&cancel)?);
    if std::env::var("FYERS_MUTATE_LIVE_ORDER").as_deref() == Ok("modify") {
        println!("{:?}", client.orders().modify_async(&modify).await?);
    } else if std::env::var("FYERS_MUTATE_LIVE_ORDER").as_deref() == Ok("cancel") {
        println!("{:?}", client.orders().cancel_async(&cancel).await?);
    } else {
        println!(
            "Not modifying/cancelling. Set FYERS_MUTATE_LIVE_ORDER=modify or cancel only after review."
        );
    }

    Ok(())
}

fn env_client() -> Result<FyersClient, FyersError> {
    FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .access_token(std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required"))
        .build()
}

fn main() {
    println!(
        "This example only prints modify/cancel requests by default. Call run() from an async runtime to opt in."
    );
}
