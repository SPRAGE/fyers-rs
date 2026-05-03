use fyers_rs::{FyersClient, FyersError};

#[tokio::main]
async fn main() -> Result<(), FyersError> {
    let client = FyersClient::builder()
        .client_id(std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required"))
        .access_token(std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required"))
        .build()?;

    println!("--- profile ---");
    match client.profile().get().await {
        Ok(profile) => println!("{profile:#?}"),
        Err(err) => eprintln!("profile: {err}"),
    }

    println!("\n--- funds ---");
    match client.funds().get().await {
        Ok(funds) => println!("{funds:#?}"),
        Err(err) => eprintln!("funds: {err}"),
    }

    println!("\n--- holdings ---");
    match client.holdings().get().await {
        Ok(holdings) => println!("{holdings:#?}"),
        Err(err) => eprintln!("holdings: {err}"),
    }
    Ok(())
}
