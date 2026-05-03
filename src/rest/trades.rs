//! Trades API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::transactions::{TradeBookQuery, TradeBookResponse};
use crate::transport::{get_authenticated_url_json, join_base_path};

/// Accessor for Fyers trade APIs.
#[derive(Debug, Clone, Copy)]
pub struct TradesService<'a> {
    client: &'a FyersClient,
}

impl<'a> TradesService<'a> {
    /// Create a new trades service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Fetch the authenticated user's trade book, optionally filtered by order tag.
    pub async fn list(&self, query: &TradeBookQuery) -> Result<TradeBookResponse> {
        let mut url = join_base_path(self.client.config().api_base_url(), "tradebook");
        if let Some(order_tag) = &query.order_tag {
            url.query_pairs_mut().append_pair("order_tag", order_tag);
        }

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }
}
