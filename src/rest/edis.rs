//! EDIS API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::edis::{
    EdisDetailsResponse, EdisIndexRequest, EdisIndexResponse, EdisInquiryRequest,
    EdisInquiryResponse, EdisTpinResponse,
};
use crate::transport::{get_authenticated_base_json, post_authenticated_base_json};

/// Accessor for Fyers EDIS APIs.
#[derive(Debug, Clone, Copy)]
pub struct EdisService<'a> {
    client: &'a FyersClient,
}

impl<'a> EdisService<'a> {
    /// Create a new EDIS service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Generate/request a CDSL TPIN.
    pub async fn generate_tpin(&self) -> Result<EdisTpinResponse> {
        get_authenticated_base_json(
            self.client.http(),
            self.client.config(),
            self.client.config().api_v2_base_url(),
            "tpin",
        )
        .await
    }

    /// Get completed holding authorizations.
    pub async fn details(&self) -> Result<EdisDetailsResponse> {
        get_authenticated_base_json(
            self.client.http(),
            self.client.config(),
            self.client.config().api_v2_base_url(),
            "details",
        )
        .await
    }

    /// Generate the CDSL authorization page payload.
    pub async fn index(&self, request: &EdisIndexRequest) -> Result<EdisIndexResponse> {
        post_authenticated_base_json(
            self.client.http(),
            self.client.config(),
            self.client.config().api_v2_base_url(),
            "index",
            request,
        )
        .await
    }

    /// Get EDIS transaction status.
    pub async fn inquiry(&self, request: &EdisInquiryRequest) -> Result<EdisInquiryResponse> {
        post_authenticated_base_json(
            self.client.http(),
            self.client.config(),
            self.client.config().api_v2_base_url(),
            "inquiry",
            request,
        )
        .await
    }
}
