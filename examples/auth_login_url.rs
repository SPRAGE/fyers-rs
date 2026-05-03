use fyers_rs::models::auth::GenerateAuthCodeRequest;
use fyers_rs::{FyersClient, FyersError};
use url::Url;

fn main() -> Result<(), FyersError> {
    let client_id = std::env::var("FYERS_CLIENT_ID").unwrap_or_else(|_| "APPID-100".to_owned());
    let redirect_uri = std::env::var("FYERS_REDIRECT_URI")
        .unwrap_or_else(|_| "https://example.com/fyers/callback".to_owned());
    let client = FyersClient::builder().client_id(client_id).build()?;
    let request = GenerateAuthCodeRequest::new(Url::parse(&redirect_uri)?, "state-123");
    let login_url = client.auth().generate_auth_code_url(&request)?;

    println!("{login_url}");
    Ok(())
}
