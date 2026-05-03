//! Auth/session API service.

use serde::Serialize;
use sha2::{Digest, Sha256};
use url::Url;

use crate::client::FyersClient;
use crate::error::{FyersError, Result};
use crate::models::auth::{
    AccessTokenResponse, GenerateAuthCodeRequest, RefreshTokenRequest, ValidateAuthCodeRequest,
};
use crate::models::common::ApiStatus;
use crate::transport::{decode_json_response, join_base_path, required_authorization_header};

/// Accessor for Fyers authentication and session APIs.
#[derive(Debug, Clone, Copy)]
pub struct AuthService<'a> {
    client: &'a FyersClient,
}

impl<'a> AuthService<'a> {
    /// Create a new auth service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Build the documented auth-code login URL.
    pub fn generate_auth_code_url(&self, request: &GenerateAuthCodeRequest) -> Result<Url> {
        let mut url = join_base_path(self.client.config().api_base_url(), "generate-authcode");

        {
            let mut query = url.query_pairs_mut();
            query
                .clear()
                .append_pair("client_id", self.client.config().client_id())
                .append_pair("redirect_uri", request.redirect_uri.as_str())
                .append_pair("response_type", &request.response_type)
                .append_pair("state", &request.state);
        }

        Ok(url)
    }

    /// Generate the documented SHA-256 hash of `app_id:app_secret`.
    pub fn app_id_hash(&self) -> Result<String> {
        let secret_key = self
            .client
            .config()
            .secret_key()
            .ok_or(FyersError::MissingConfig {
                field: "secret_key",
            })?;
        let mut hasher = Sha256::new();
        hasher.update(self.client.config().client_id().as_bytes());
        hasher.update(b":");
        hasher.update(secret_key.expose_secret().as_bytes());

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Validate an auth code and generate access/refresh tokens.
    pub async fn validate_auth_code(
        &self,
        request: &ValidateAuthCodeRequest,
    ) -> Result<AccessTokenResponse> {
        self.post_json("validate-authcode", request).await
    }

    /// Validate a refresh token and generate a new access token.
    pub async fn refresh_access_token(
        &self,
        request: &RefreshTokenRequest,
    ) -> Result<AccessTokenResponse> {
        self.post_json("validate-refresh-token", request).await
    }

    /// Logout and invalidate the configured access token for this app.
    pub async fn logout(&self) -> Result<ApiStatus> {
        let url = join_base_path(self.client.config().api_base_url(), "logout");
        let response = self
            .client
            .http()
            .post(url)
            .header(
                reqwest::header::AUTHORIZATION,
                required_authorization_header(self.client.config())?,
            )
            .send()
            .await?;

        decode_json_response(response).await
    }

    async fn post_json<T>(&self, path: &str, request: &T) -> Result<AccessTokenResponse>
    where
        T: Serialize + ?Sized,
    {
        let url = join_base_path(self.client.config().api_base_url(), path);
        let response = self.client.http().post(url).json(request).send().await?;

        decode_json_response(response).await
    }
}
