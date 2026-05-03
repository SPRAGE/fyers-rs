//! Positions API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::common::ApiStatus;
use crate::models::portfolio::{
    ConvertPositionRequest, ConvertPositionResponse, ExitAllPositionsRequest,
    ExitPositionsByFilterRequest, ExitPositionsByIdRequest, PendingOrderCancelRequest,
};
use crate::models::transactions::PositionsResponse;
use crate::transport::{
    delete_authenticated_json, get_authenticated_json, post_authenticated_json,
};

/// Accessor for Fyers position APIs.
#[derive(Debug, Clone, Copy)]
pub struct PositionsService<'a> {
    client: &'a FyersClient,
}

impl<'a> PositionsService<'a> {
    /// Create a new positions service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Fetch current open and closed positions for the authenticated user.
    pub async fn list(&self) -> Result<PositionsResponse> {
        get_authenticated_json(self.client.http(), self.client.config(), "positions").await
    }

    /// Exit all open positions.
    pub async fn exit_all(&self, request: &ExitAllPositionsRequest) -> Result<ApiStatus> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "positions",
            request,
        )
        .await
    }

    /// Exit positions by documented ID selector.
    pub async fn exit_by_id(&self, request: &ExitPositionsByIdRequest) -> Result<ApiStatus> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "positions",
            request,
        )
        .await
    }

    /// Exit positions by documented segment/side/product filters.
    pub async fn exit_by_filters(
        &self,
        request: &ExitPositionsByFilterRequest,
    ) -> Result<ApiStatus> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "positions",
            request,
        )
        .await
    }

    /// Cancel pending orders while exiting positions.
    pub async fn exit_with_pending_order_cancel(
        &self,
        request: &PendingOrderCancelRequest,
    ) -> Result<ApiStatus> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "positions",
            request,
        )
        .await
    }

    /// Convert an open position to a different product type.
    pub async fn convert(
        &self,
        request: &ConvertPositionRequest,
    ) -> Result<ConvertPositionResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "positions",
            request,
        )
        .await
    }
}
