use fyers_rs::FyersClient;

fn main() -> Result<(), fyers_rs::FyersError> {
    let client = FyersClient::builder()
        .client_id("APPID-100")
        .access_token("ACCESS_TOKEN")
        .build()?;

    let _orders = client.orders();
    let _data_socket = client.data_socket();

    Ok(())
}
