use fyers_rs::{FyersClient, FyersError};

#[test]
fn builder_requires_client_id() {
    let err = FyersClient::builder()
        .build()
        .expect_err("client ID should be required");

    assert!(matches!(
        err,
        FyersError::MissingConfig { field: "client_id" }
    ));
}

#[test]
fn builder_sets_default_urls() {
    let client = FyersClient::builder()
        .client_id("APPID-100")
        .build()
        .expect("client should build with required client ID");

    assert_eq!(
        client.config().api_base_url().as_str(),
        "https://api-t1.fyers.in/api/v3"
    );
    assert_eq!(
        client.config().data_base_url().as_str(),
        "https://api-t1.fyers.in/data"
    );
    assert_eq!(
        client.config().api_v2_base_url().as_str(),
        "https://api.fyers.in/api/v2"
    );
    assert_eq!(
        client.config().symbols_base_url().as_str(),
        "https://public.fyers.in/sym_details"
    );
    assert_eq!(
        client.config().data_socket_url().as_str(),
        "wss://socket.fyers.in/hsm/v1-5/prod"
    );
    assert_eq!(
        client.config().order_socket_url().as_str(),
        "wss://socket.fyers.in/trade/v3"
    );
    assert_eq!(
        client.config().tbt_socket_url().as_str(),
        "wss://rtsocket-api.fyers.in/versova"
    );
}

#[test]
fn secrets_are_redacted_in_debug_output() {
    let client = FyersClient::builder()
        .client_id("APPID-100")
        .secret_key("super-secret")
        .access_token("access-token")
        .build()
        .expect("client should build");

    let debug = format!("{:?}", client.config());

    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("super-secret"));
    assert!(!debug.contains("access-token"));
}

#[test]
fn service_accessors_borrow_the_same_client() {
    let client = FyersClient::builder()
        .client_id("APPID-100")
        .build()
        .expect("client should build");

    assert_eq!(client.orders().client().config().client_id(), "APPID-100");
    assert_eq!(
        client.data_socket().client().config().client_id(),
        "APPID-100"
    );
    assert_eq!(
        client.tbt_socket().client().config().client_id(),
        "APPID-100"
    );
}
