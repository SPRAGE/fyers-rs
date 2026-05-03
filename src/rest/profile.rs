//! Profile/user API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::user::ProfileResponse;
use crate::transport::get_authenticated_json;

/// Accessor for Fyers profile APIs.
#[derive(Debug, Clone, Copy)]
pub struct ProfileService<'a> {
    client: &'a FyersClient,
}

impl<'a> ProfileService<'a> {
    /// Create a new profile service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Fetch basic profile details for the authenticated user.
    pub async fn get(&self) -> Result<ProfileResponse> {
        get_authenticated_json(self.client.http(), self.client.config(), "profile").await
    }
}
