//! Shared REST transport primitives.
//!
//! Endpoint-specific modules use this layer for HTTP method/path/query/body
//! construction, auth header injection, and typed response deserialization.

use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

use crate::config::{FyersConfig, SecretString};
use crate::error::{FyersError, Result};

/// HTTP methods used by documented Fyers REST APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestMethod {
    /// HTTP GET.
    Get,
    /// HTTP POST.
    Post,
    /// HTTP PUT.
    Put,
    /// HTTP PATCH.
    Patch,
    /// HTTP DELETE.
    Delete,
}

/// Join a path to a configured Fyers base URL without losing a base path such as `/api/v3`.
pub(crate) fn join_base_path(base: &Url, path: &str) -> Url {
    let mut url = base.clone();
    let base_path = base.path().trim_end_matches('/');
    let path = path.trim_start_matches('/');
    let joined_path = if base_path.is_empty() {
        format!("/{path}")
    } else if path.is_empty() {
        base_path.to_owned()
    } else {
        format!("{base_path}/{path}")
    };

    url.set_path(&joined_path);
    url
}

/// Build the documented `Authorization: app_id:access_token` header value.
pub(crate) fn authorization_header_value(
    client_id: &str,
    access_token: &SecretString,
) -> Result<HeaderValue> {
    HeaderValue::from_str(&format!("{client_id}:{}", access_token.expose_secret()))
        .map_err(|err| FyersError::Validation(format!("invalid authorization header value: {err}")))
}

/// Build an authorization header from client config, requiring an access token.
pub(crate) fn required_authorization_header(config: &FyersConfig) -> Result<HeaderValue> {
    let access_token = config.access_token().ok_or(FyersError::MissingConfig {
        field: "access_token",
    })?;

    authorization_header_value(config.client_id(), access_token)
}

/// Execute an authenticated GET request and deserialize a JSON response.
pub(crate) async fn get_authenticated_json<T>(
    http: &reqwest::Client,
    config: &FyersConfig,
    path: &str,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let url = join_base_path(config.api_base_url(), path);

    get_authenticated_url_json(http, config, url).await
}

/// Execute an authenticated GET request for a pre-built URL and deserialize a JSON response.
pub(crate) async fn get_authenticated_url_json<T>(
    http: &reqwest::Client,
    config: &FyersConfig,
    url: Url,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let response = http
        .get(url)
        .header(AUTHORIZATION, required_authorization_header(config)?)
        .send()
        .await?;

    decode_json_response(response).await
}

/// Execute an authenticated GET request against a specific base URL.
pub(crate) async fn get_authenticated_base_json<T>(
    http: &reqwest::Client,
    config: &FyersConfig,
    base: &Url,
    path: &str,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let url = join_base_path(base, path);

    get_authenticated_url_json(http, config, url).await
}

/// Execute an authenticated POST request with a JSON body and deserialize a JSON response.
pub(crate) async fn post_authenticated_json<T, U>(
    http: &reqwest::Client,
    config: &FyersConfig,
    path: &str,
    request: &U,
) -> Result<T>
where
    T: DeserializeOwned,
    U: Serialize + ?Sized,
{
    let url = join_base_path(config.api_base_url(), path);
    let response = http
        .post(url)
        .header(AUTHORIZATION, required_authorization_header(config)?)
        .json(request)
        .send()
        .await?;

    decode_json_response(response).await
}

/// Execute an authenticated POST request against a specific base URL.
pub(crate) async fn post_authenticated_base_json<T, U>(
    http: &reqwest::Client,
    config: &FyersConfig,
    base: &Url,
    path: &str,
    request: &U,
) -> Result<T>
where
    T: DeserializeOwned,
    U: Serialize + ?Sized,
{
    let url = join_base_path(base, path);
    let response = http
        .post(url)
        .header(AUTHORIZATION, required_authorization_header(config)?)
        .json(request)
        .send()
        .await?;

    decode_json_response(response).await
}

/// Execute an authenticated PUT request with a JSON body and deserialize a JSON response.
pub(crate) async fn put_authenticated_json<T, U>(
    http: &reqwest::Client,
    config: &FyersConfig,
    path: &str,
    request: &U,
) -> Result<T>
where
    T: DeserializeOwned,
    U: Serialize + ?Sized,
{
    let url = join_base_path(config.api_base_url(), path);
    let response = http
        .put(url)
        .header(AUTHORIZATION, required_authorization_header(config)?)
        .json(request)
        .send()
        .await?;

    decode_json_response(response).await
}

/// Execute an authenticated PATCH request with a JSON body and deserialize a JSON response.
pub(crate) async fn patch_authenticated_json<T, U>(
    http: &reqwest::Client,
    config: &FyersConfig,
    path: &str,
    request: &U,
) -> Result<T>
where
    T: DeserializeOwned,
    U: Serialize + ?Sized,
{
    let url = join_base_path(config.api_base_url(), path);
    let response = http
        .patch(url)
        .header(AUTHORIZATION, required_authorization_header(config)?)
        .json(request)
        .send()
        .await?;

    decode_json_response(response).await
}

/// Execute an authenticated DELETE request with a JSON body and deserialize a JSON response.
pub(crate) async fn delete_authenticated_json<T, U>(
    http: &reqwest::Client,
    config: &FyersConfig,
    path: &str,
    request: &U,
) -> Result<T>
where
    T: DeserializeOwned,
    U: Serialize + ?Sized,
{
    let url = join_base_path(config.api_base_url(), path);
    let response = http
        .delete(url)
        .header(AUTHORIZATION, required_authorization_header(config)?)
        .json(request)
        .send()
        .await?;

    decode_json_response(response).await
}

/// Execute an authenticated DELETE request without a body and deserialize a JSON response.
pub(crate) async fn delete_authenticated_empty_json<T>(
    http: &reqwest::Client,
    config: &FyersConfig,
    path: &str,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let url = join_base_path(config.api_base_url(), path);
    let response = http
        .delete(url)
        .header(AUTHORIZATION, required_authorization_header(config)?)
        .send()
        .await?;

    decode_json_response(response).await
}

/// Decode a Fyers JSON response and surface documented broker error envelopes.
pub(crate) async fn decode_json_response<T>(response: reqwest::Response) -> Result<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    let retry_after = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_owned().into_boxed_str());
    let retry_after_ms = response
        .headers()
        .get("x-retry-after-ms")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_owned().into_boxed_str());
    let body = response.text().await?;
    let value = match serde_json::from_str::<Value>(&body) {
        Ok(value) => value,
        Err(_) if !status.is_success() => {
            return Err(FyersError::Broker {
                status,
                retry_after,
                retry_after_ms,
                code: None,
                s: None,
                message: None,
                body: body.into_boxed_str(),
            });
        }
        Err(err) => return Err(err.into()),
    };

    let s = value
        .get("s")
        .and_then(Value::as_str)
        .map(|value| value.to_owned().into_boxed_str());
    let code = value.get("code").and_then(Value::as_i64);
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .map(|value| value.to_owned().into_boxed_str());

    if !status.is_success() || matches!(s.as_deref(), Some("error")) {
        return Err(FyersError::Broker {
            status,
            retry_after,
            retry_after_ms,
            code,
            s,
            message,
            body: body.into_boxed_str(),
        });
    }

    Ok(serde_json::from_value(value)?)
}
