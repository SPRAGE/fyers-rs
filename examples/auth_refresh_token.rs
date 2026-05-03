#![allow(dead_code)]

use fyers_rs::models::auth::RefreshTokenRequest;
use fyers_rs::{FyersClient, FyersError};

async fn run() -> Result<(), FyersError> {
    let client = FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .secret_key(std::env::var("FYERS_SECRET_KEY").expect("FYERS_SECRET_KEY is required"))
        .build()?;
    let request = RefreshTokenRequest::new(
        client.auth().app_id_hash()?,
        std::env::var("FYERS_REFRESH_TOKEN").expect("FYERS_REFRESH_TOKEN is required"),
        std::env::var("FYERS_PIN").expect("FYERS_PIN is required"),
    );
    let token = client.auth().refresh_access_token(&request).await?;

    println!("{token:?}");
    Ok(())
}

fn main() {
    println!(
        "Set FYERS_CLIENT_ID, FYERS_SECRET_KEY, FYERS_REFRESH_TOKEN, and FYERS_PIN, then call run() from an async runtime."
    );
}
