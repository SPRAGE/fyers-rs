mod support;

use fyers_rs::{FyersClient, FyersError};
use pretty_assertions::assert_eq;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn profile_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/user/profile/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/profile"))
        .and(header(
            "authorization",
            "SPXXXXE7-100:fake_access_token.jwt",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .profile()
        .get()
        .await
        .expect("profile should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data.fy_id, "FX0011");
}

#[tokio::test]
async fn funds_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/user/funds/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/funds"))
        .and(header(
            "authorization",
            "SPXXXXE7-100:fake_access_token.jwt",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client.funds().get().await.expect("funds should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.fund_limit.len(), 10);
}

#[tokio::test]
async fn holdings_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/user/holdings/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/holdings"))
        .and(header(
            "authorization",
            "SPXXXXE7-100:fake_access_token.jwt",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .holdings()
        .get()
        .await
        .expect("holdings should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.holdings.len(), 2);
}

#[tokio::test]
async fn profile_requires_access_token() {
    let client = FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .build()
        .expect("client should build");

    let err = client
        .profile()
        .get()
        .await
        .expect_err("access token should be required");

    assert!(matches!(
        err,
        FyersError::MissingConfig {
            field: "access_token"
        }
    ));
}

#[tokio::test]
async fn broker_error_envelopes_are_returned_as_errors() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/common/status_error.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/profile"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let err = client
        .profile()
        .get()
        .await
        .expect_err("broker error envelope should not deserialize as success");

    match err {
        FyersError::Broker {
            status,
            code,
            s,
            message,
            ..
        } => {
            assert_eq!(status, reqwest::StatusCode::OK);
            assert_eq!(code, Some(-50));
            assert_eq!(s.as_deref(), Some("error"));
            assert_eq!(message.as_deref(), Some("Invalid input"));
        }
        other => panic!("expected broker error, got {other:?}"),
    }
}

#[tokio::test]
async fn non_success_non_json_responses_are_returned_as_broker_errors() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/profile"))
        .respond_with(
            ResponseTemplate::new(429)
                .append_header("Retry-After", "2")
                .append_header("X-Retry-After-Ms", "2500")
                .set_body_string("service unavailable"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let err = client
        .profile()
        .get()
        .await
        .expect_err("non-success HTTP status should be a broker error");

    match err {
        FyersError::Broker {
            status,
            retry_after,
            retry_after_ms,
            body,
            ..
        } => {
            assert_eq!(status, reqwest::StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(retry_after.as_deref(), Some("2"));
            assert_eq!(retry_after_ms.as_deref(), Some("2500"));
            assert_eq!(body.as_ref(), "service unavailable");
        }
        other => panic!("expected broker error, got {other:?}"),
    }
}

fn authenticated_client(mock_server: &MockServer) -> FyersClient {
    FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .access_token("fake_access_token.jwt")
        .api_base_url_str(format!("{}/api/v3", mock_server.uri()))
        .expect("base URL should parse")
        .build()
        .expect("client should build")
}
