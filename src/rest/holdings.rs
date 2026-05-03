//! Holdings API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::user::HoldingsResponse;
use crate::transport::get_authenticated_json;

/// Accessor for Fyers holdings APIs.
#[derive(Debug, Clone, Copy)]
pub struct HoldingsService<'a> {
    client: &'a FyersClient,
}

impl<'a> HoldingsService<'a> {
    /// Create a new holdings service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Fetch equity and mutual fund holdings for the authenticated user.
    pub async fn get(&self) -> Result<HoldingsResponse> {
        get_authenticated_json(self.client.http(), self.client.config(), "holdings").await
    }
}
