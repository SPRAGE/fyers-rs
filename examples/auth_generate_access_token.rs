#![allow(dead_code)]

use fyers_rs::models::auth::ValidateAuthCodeRequest;
use fyers_rs::{FyersClient, FyersError};

async fn run() -> Result<(), FyersError> {
    let client = FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .secret_key(std::env::var("FYERS_SECRET_KEY").expect("FYERS_SECRET_KEY is required"))
        .build()?;
    let request = ValidateAuthCodeRequest::new(
        client.auth().app_id_hash()?,
        std::env::var("FYERS_AUTH_CODE").expect("FYERS_AUTH_CODE is required"),
    );
    let token = client.auth().validate_auth_code(&request).await?;

    println!("{token:?}");
    Ok(())
}

fn main() {
    println!(
        "Set FYERS_CLIENT_ID, FYERS_SECRET_KEY, and FYERS_AUTH_CODE, then call run() from an async runtime."
    );
}
