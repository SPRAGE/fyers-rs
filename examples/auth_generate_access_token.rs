use fyers_rs::models::auth::ValidateAuthCodeRequest;
use fyers_rs::{FyersClient, FyersError};

#[tokio::main]
async fn main() -> Result<(), FyersError> {
    let client = FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .secret_key(std::env::var("FYERS_SECRET_KEY").expect("FYERS_SECRET_KEY is required"))
        .build()?;
    let request = ValidateAuthCodeRequest::new(
        client.auth().app_id_hash()?,
        std::env::var("FYERS_AUTH_CODE").expect("FYERS_AUTH_CODE is required"),
    );
    let token = client.auth().validate_auth_code(&request).await?;

    println!("status: s={} code={} message={:?}", token.s, token.code, token.message);
    println!(
        "access_token: {}",
        token.access_token.as_deref().unwrap_or("(none)")
    );
    println!(
        "refresh_token: {}",
        token.refresh_token.as_deref().unwrap_or("(none)")
    );
    eprintln!(
        "\nTreat both tokens as secrets. Paste them into .env as FYERS_ACCESS_TOKEN and FYERS_REFRESH_TOKEN."
    );
    Ok(())
}
