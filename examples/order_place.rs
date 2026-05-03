#![allow(dead_code)]

use fyers_rs::models::orders::PlaceOrderRequest;
use fyers_rs::{FyersClient, FyersError};

async fn run() -> Result<(), FyersError> {
    let client = env_client()?;
    let request = sample_order();

    println!("{}", serde_json::to_string_pretty(&request)?);
    if std::env::var("FYERS_PLACE_LIVE_ORDER").as_deref() == Ok("1") {
        let response = client.orders().place_async(&request).await?;
        println!("{response:?}");
    } else {
        println!(
            "Not placing an order. Set FYERS_PLACE_LIVE_ORDER=1 only after reviewing the request."
        );
    }

    Ok(())
}

fn sample_order() -> PlaceOrderRequest {
    PlaceOrderRequest {
        symbol: "NSE:SBIN-EQ".to_owned(),
        qty: 1,
        order_type: 2,
        side: 1,
        product_type: "INTRADAY".to_owned(),
        limit_price: 0.0,
        stop_price: 0.0,
        validity: "DAY".to_owned(),
        disclosed_qty: 0,
        offline_order: false,
        stop_loss: Some(0.0),
        take_profit: Some(0.0),
        order_tag: Some("fyers-rs-example".to_owned()),
        is_slice_order: Some(false),
    }
}

fn env_client() -> Result<FyersClient, FyersError> {
    FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .access_token(std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required"))
        .build()
}

fn main() {
    println!(
        "This example only prints an order by default. Set FYERS_PLACE_LIVE_ORDER=1 and call run() from an async runtime to place it."
    );
}
