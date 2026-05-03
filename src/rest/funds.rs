//! Funds API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::user::FundsResponse;
use crate::transport::get_authenticated_json;

/// Accessor for Fyers funds APIs.
#[derive(Debug, Clone, Copy)]
pub struct FundsService<'a> {
    client: &'a FyersClient,
}

impl<'a> FundsService<'a> {
    /// Create a new funds service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Fetch available capital and commodity balances for the authenticated user.
    pub async fn get(&self) -> Result<FundsResponse> {
        get_authenticated_json(self.client.http(), self.client.config(), "funds").await
    }
}
