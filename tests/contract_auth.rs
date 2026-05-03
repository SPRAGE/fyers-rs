mod support;

use fyers_rs::models::auth::{
    AccessTokenResponse, GenerateAuthCodeRequest, RefreshTokenRequest, ValidateAuthCodeRequest,
};
use fyers_rs::models::common::ApiStatus;
use fyers_rs::{FyersClient, FyersError};
use pretty_assertions::assert_eq;
use url::Url;

#[test]
fn common_success_status_fixture_matches_model() {
    let status: ApiStatus = support::json_fixture("rest/common/status_ok.json");

    assert_eq!(status.s, "ok");
    assert_eq!(status.code, 200);
    assert_eq!(status.message, "");
}

#[test]
fn common_error_status_fixture_matches_model() {
    let status: ApiStatus = support::json_fixture("rest/common/status_error.json");

    assert_eq!(status.s, "error");
    assert_eq!(status.code, -50);
    assert_eq!(status.message, "Invalid input");
}

#[test]
fn logout_success_fixture_matches_api_status() {
    let status: ApiStatus = support::json_fixture("rest/user/logout/response_success.json");

    assert_eq!(status.s, "ok");
    assert_eq!(status.code, 200);
    assert_eq!(status.message, "you are successfully logged out");
}

#[test]
fn auth_code_query_fixture_records_documented_params() {
    let query = support::read_fixture("rest/auth/generate_auth_code/request_query.txt");

    assert!(query.contains("client_id=SPXXXXE7-100"));
    assert!(query.contains("redirect_uri="));
    assert!(query.contains("response_type=code"));
    assert!(query.contains("state=sample_state"));
}

#[test]
fn generate_auth_code_request_defaults_to_code_response_type() {
    let redirect_uri = Url::parse("https://trade.fyers.in/api-login/redirect-uri/index.html")
        .expect("redirect URI fixture should be valid");
    let request = GenerateAuthCodeRequest::new(redirect_uri.clone(), "sample_state");

    assert_eq!(request.redirect_uri, redirect_uri);
    assert_eq!(request.response_type, "code");
    assert_eq!(request.state, "sample_state");
}

#[test]
fn validate_authcode_request_fixture_matches_model() {
    let request: ValidateAuthCodeRequest =
        support::json_fixture("rest/auth/validate_authcode/request.json");
    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(request.grant_type, "authorization_code");
    assert_eq!(request.app_id_hash, "fake_sha256_app_id_plus_secret_hash");
    assert_eq!(request.code, "fake_auth_code");
    assert_eq!(
        value,
        support::json_value("rest/auth/validate_authcode/request.json")
    );
}

#[test]
fn refresh_token_request_fixture_matches_model() {
    let request: RefreshTokenRequest =
        support::json_fixture("rest/auth/refresh_token/request.json");
    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(request.grant_type, "refresh_token");
    assert_eq!(request.app_id_hash, "fake_sha256_app_id_plus_secret_hash");
    assert_eq!(request.refresh_token, "fake_refresh_token.jwt");
    assert_eq!(request.pin, "1234");
    assert_eq!(
        value,
        support::json_value("rest/auth/refresh_token/request.json")
    );
}

#[test]
fn auth_request_debug_output_redacts_sensitive_values() {
    let validate_request = ValidateAuthCodeRequest::new("secret_hash", "secret_auth_code");
    let refresh_request = RefreshTokenRequest::new("secret_hash", "secret_refresh", "1234");

    let debug = format!("{validate_request:?} {refresh_request:?}");

    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("secret_hash"));
    assert!(!debug.contains("secret_auth_code"));
    assert!(!debug.contains("secret_refresh"));
    assert!(!debug.contains("1234"));
}

#[test]
fn access_token_response_debug_output_redacts_tokens() {
    let response: AccessTokenResponse =
        support::json_fixture("rest/auth/validate_authcode/response_success.json");

    let debug = format!("{response:?}");

    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("fake_access_token.jwt"));
    assert!(!debug.contains("fake_refresh_token.jwt"));
}

#[test]
fn app_id_hash_uses_documented_sha256_input() {
    let client = FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .secret_key("APP_SECRET")
        .build()
        .expect("client should build");

    let hash = client
        .auth()
        .app_id_hash()
        .expect("hash should be generated");

    assert_eq!(
        hash,
        "de7477f2f5714deb570bd250f5158ed7b053844102cbaddfe48fdef96bb75ebe"
    );
}

#[test]
fn app_id_hash_requires_secret_key() {
    let client = FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .build()
        .expect("client should build");

    let err = client
        .auth()
        .app_id_hash()
        .expect_err("secret key should be required");

    assert!(matches!(
        err,
        FyersError::MissingConfig {
            field: "secret_key"
        }
    ));
}

#[test]
fn validate_authcode_success_fixture_matches_model() {
    let response: AccessTokenResponse =
        support::json_fixture("rest/auth/validate_authcode/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.message, "");
    assert_eq!(
        response.access_token.as_deref(),
        Some("fake_access_token.jwt")
    );
    assert_eq!(
        response.refresh_token.as_deref(),
        Some("fake_refresh_token.jwt")
    );
}

#[test]
fn validate_authcode_error_fixture_deserializes_without_tokens() {
    let response: AccessTokenResponse =
        support::json_fixture("rest/auth/validate_authcode/response_error.json");

    assert_eq!(response.s, "error");
    assert_eq!(response.code, -50);
    assert_eq!(response.message, "Invalid auth code");
    assert_eq!(response.access_token, None);
    assert_eq!(response.refresh_token, None);
}

#[test]
fn refresh_token_success_fixture_allows_missing_refresh_token() {
    let response: AccessTokenResponse =
        support::json_fixture("rest/auth/refresh_token/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(
        response.access_token.as_deref(),
        Some("fake_refreshed_access_token.jwt")
    );
    assert_eq!(response.refresh_token, None);
}
