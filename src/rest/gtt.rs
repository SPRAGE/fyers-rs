//! GTT orders API service.

use crate::client::FyersClient;
use crate::error::{FyersError, Result};
use crate::models::gtt::{
    CancelGttOrderRequest, GttActionResponse, GttOcoOrderRequest, GttOrderBookResponse,
    GttOrderRequest, ModifyGttOrderRequest,
};
use crate::transport::{
    delete_authenticated_json, get_authenticated_json, patch_authenticated_json,
    post_authenticated_json,
};

/// Accessor for Fyers GTT order APIs.
#[derive(Debug, Clone, Copy)]
pub struct GttService<'a> {
    client: &'a FyersClient,
}

impl<'a> GttService<'a> {
    /// Create a new GTT service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Place a documented single-leg GTT order.
    pub async fn place_single(&self, request: &GttOrderRequest) -> Result<GttActionResponse> {
        if request.order_info.leg2.is_some() {
            return Err(FyersError::Validation(
                "single GTT order does not support leg2".to_owned(),
            ));
        }

        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "gtt/orders/sync",
            request,
        )
        .await
    }

    /// Place a documented OCO GTT order.
    pub async fn place_oco(&self, request: &GttOcoOrderRequest) -> Result<GttActionResponse> {
        if request.order_info.leg2.is_none() {
            return Err(FyersError::Validation(
                "leg2 is required for OCO GTT orders".to_owned(),
            ));
        }

        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "gtt/orders/sync",
            request,
        )
        .await
    }

    /// Modify a documented GTT order.
    pub async fn modify(&self, request: &ModifyGttOrderRequest) -> Result<GttActionResponse> {
        patch_authenticated_json(
            self.client.http(),
            self.client.config(),
            "gtt/orders/sync",
            request,
        )
        .await
    }

    /// Cancel a documented GTT order.
    pub async fn cancel(&self, request: &CancelGttOrderRequest) -> Result<GttActionResponse> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "gtt/orders/sync",
            request,
        )
        .await
    }

    /// Fetch the authenticated user's GTT order book.
    pub async fn orderbook(&self) -> Result<GttOrderBookResponse> {
        get_authenticated_json(self.client.http(), self.client.config(), "gtt/orders").await
    }
}
