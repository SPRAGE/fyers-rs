mod support;

use fyers_rs::FyersClient;
use fyers_rs::models::auth::{
    GenerateAuthCodeRequest, RefreshTokenRequest, ValidateAuthCodeRequest,
};
use pretty_assertions::assert_eq;
use serde_json::json;
use url::Url;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn auth_service_builds_documented_auth_code_url() {
    let client = FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .api_base_url_str("https://api-t1.fyers.in/api/v3")
        .expect("base URL should parse")
        .build()
        .expect("client should build");
    let redirect_uri = Url::parse("https://trade.fyers.in/api-login/redirect-uri/index.html")
        .expect("redirect URI should parse");
    let request = GenerateAuthCodeRequest::new(redirect_uri, "sample_state");

    let url = client
        .auth()
        .generate_auth_code_url(&request)
        .expect("auth-code URL should build");

    assert_eq!(url.scheme(), "https");
    assert_eq!(url.host_str(), Some("api-t1.fyers.in"));
    assert_eq!(url.path(), "/api/v3/generate-authcode");

    let query_pairs = url.query_pairs().collect::<Vec<_>>();
    assert!(
        query_pairs
            .iter()
            .any(|(key, value)| key == "client_id" && value == "SPXXXXE7-100")
    );
    assert!(query_pairs.iter().any(|(key, value)| key == "redirect_uri"
        && value == "https://trade.fyers.in/api-login/redirect-uri/index.html"));
    assert!(
        query_pairs
            .iter()
            .any(|(key, value)| key == "response_type" && value == "code")
    );
    assert!(
        query_pairs
            .iter()
            .any(|(key, value)| key == "state" && value == "sample_state")
    );
}

#[tokio::test]
async fn validate_auth_code_posts_documented_body() {
    let mock_server = MockServer::start().await;
    let request: ValidateAuthCodeRequest =
        support::json_fixture("rest/auth/validate_authcode/request.json");
    let response_body = support::read_fixture("rest/auth/validate_authcode/response_success.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/validate-authcode"))
        .and(body_json(json!({
            "grant_type": "authorization_code",
            "appIdHash": "fake_sha256_app_id_plus_secret_hash",
            "code": "fake_auth_code"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .api_base_url_str(format!("{}/api/v3", mock_server.uri()))
        .expect("base URL should parse")
        .build()
        .expect("client should build");

    let response = client
        .auth()
        .validate_auth_code(&request)
        .await
        .expect("validate-authcode should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.s, "ok");
    assert_eq!(
        response.access_token.as_deref(),
        Some("fake_access_token.jwt")
    );
    assert_eq!(
        response.refresh_token.as_deref(),
        Some("fake_refresh_token.jwt")
    );
}

#[tokio::test]
async fn refresh_access_token_posts_documented_body() {
    let mock_server = MockServer::start().await;
    let request: RefreshTokenRequest =
        support::json_fixture("rest/auth/refresh_token/request.json");
    let response_body = support::read_fixture("rest/auth/refresh_token/response_success.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/validate-refresh-token"))
        .and(body_json(json!({
            "grant_type": "refresh_token",
            "appIdHash": "fake_sha256_app_id_plus_secret_hash",
            "refresh_token": "fake_refresh_token.jwt",
            "pin": "1234"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .api_base_url_str(format!("{}/api/v3", mock_server.uri()))
        .expect("base URL should parse")
        .build()
        .expect("client should build");

    let response = client
        .auth()
        .refresh_access_token(&request)
        .await
        .expect("validate-refresh-token should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.s, "ok");
    assert_eq!(
        response.access_token.as_deref(),
        Some("fake_refreshed_access_token.jwt")
    );
    assert_eq!(response.refresh_token, None);
}

#[tokio::test]
async fn logout_posts_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/user/logout/response_success.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/logout"))
        .and(header(
            "authorization",
            "SPXXXXE7-100:fake_access_token.jwt",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .access_token("fake_access_token.jwt")
        .api_base_url_str(format!("{}/api/v3", mock_server.uri()))
        .expect("base URL should parse")
        .build()
        .expect("client should build");

    let response = client.auth().logout().await.expect("logout should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.s, "ok");
    assert_eq!(response.message, "you are successfully logged out");
}

#[tokio::test]
async fn logout_requires_access_token() {
    let client = FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .build()
        .expect("client should build");

    let err = client
        .auth()
        .logout()
        .await
        .expect_err("access token should be required");

    assert!(matches!(
        err,
        fyers_rs::FyersError::MissingConfig {
            field: "access_token"
        }
    ));
}
