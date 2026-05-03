//! Auth request and response models.

use std::fmt;

use serde::{Deserialize, Serialize};
use url::Url;

/// Documented OAuth response type for Fyers auth-code generation.
pub const AUTH_CODE_RESPONSE_TYPE: &str = "code";

/// Documented grant type for auth-code validation.
pub const AUTHORIZATION_CODE_GRANT: &str = "authorization_code";

/// Documented grant type for refresh-token validation.
pub const REFRESH_TOKEN_GRANT: &str = "refresh_token";

/// Request parameters used to build the documented auth-code login URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateAuthCodeRequest {
    /// Redirect URI registered for the Fyers app.
    pub redirect_uri: Url,
    /// OAuth response type. Fyers documents this as always `code`.
    pub response_type: String,
    /// Caller-provided state value returned by the login redirect.
    pub state: String,
}

impl GenerateAuthCodeRequest {
    /// Create a login URL request with the documented `code` response type.
    pub fn new(redirect_uri: Url, state: impl Into<String>) -> Self {
        Self {
            redirect_uri,
            response_type: AUTH_CODE_RESPONSE_TYPE.to_owned(),
            state: state.into(),
        }
    }
}

/// Request body for the documented validate-authcode endpoint.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateAuthCodeRequest {
    /// Grant type. Fyers documents this as `authorization_code`.
    pub grant_type: String,
    /// SHA-256 hash of `app_id:app_secret`.
    #[serde(rename = "appIdHash")]
    pub app_id_hash: String,
    /// Auth code returned by the login redirect.
    pub code: String,
}

impl ValidateAuthCodeRequest {
    /// Create a validate-authcode request with the documented grant type.
    pub fn new(app_id_hash: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            grant_type: AUTHORIZATION_CODE_GRANT.to_owned(),
            app_id_hash: app_id_hash.into(),
            code: code.into(),
        }
    }
}

impl fmt::Debug for ValidateAuthCodeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidateAuthCodeRequest")
            .field("grant_type", &self.grant_type)
            .field("app_id_hash", &"[redacted]")
            .field("code", &"[redacted]")
            .finish()
    }
}

/// Request body for the documented validate-refresh-token endpoint.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    /// Grant type. Fyers documents this as `refresh_token`.
    pub grant_type: String,
    /// SHA-256 hash of `app_id:app_secret`.
    #[serde(rename = "appIdHash")]
    pub app_id_hash: String,
    /// Refresh token returned by auth-code validation.
    pub refresh_token: String,
    /// Fyers PIN used by the refresh-token flow.
    pub pin: String,
}

impl RefreshTokenRequest {
    /// Create a refresh-token request with the documented grant type.
    pub fn new(
        app_id_hash: impl Into<String>,
        refresh_token: impl Into<String>,
        pin: impl Into<String>,
    ) -> Self {
        Self {
            grant_type: REFRESH_TOKEN_GRANT.to_owned(),
            app_id_hash: app_id_hash.into(),
            refresh_token: refresh_token.into(),
            pin: pin.into(),
        }
    }
}

impl fmt::Debug for RefreshTokenRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RefreshTokenRequest")
            .field("grant_type", &self.grant_type)
            .field("app_id_hash", &"[redacted]")
            .field("refresh_token", &"[redacted]")
            .field("pin", &"[redacted]")
            .finish()
    }
}

/// Response returned by documented access-token generation APIs.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string.
    pub s: String,
    /// Human-readable message.
    pub message: String,
    /// Access token returned by the broker.
    #[serde(default)]
    pub access_token: Option<String>,
    /// Refresh token returned by the broker, if present.
    #[serde(default)]
    pub refresh_token: Option<String>,
}

impl fmt::Debug for AccessTokenResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccessTokenResponse")
            .field("code", &self.code)
            .field("s", &self.s)
            .field("message", &self.message)
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "[redacted]"),
            )
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[redacted]"),
            )
            .finish()
    }
}
