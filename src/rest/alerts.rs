//! Price alerts API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::alerts::{
    CreatePriceAlertRequest, DeletePriceAlertRequest, ModifyPriceAlertRequest,
    PriceAlertActionResponse, PriceAlertQuery, PriceAlertsResponse, TogglePriceAlertRequest,
};
use crate::transport::{
    delete_authenticated_json, get_authenticated_url_json, join_base_path, post_authenticated_json,
    put_authenticated_json,
};

/// Accessor for Fyers price alert APIs.
#[derive(Debug, Clone, Copy)]
pub struct AlertsService<'a> {
    client: &'a FyersClient,
}

impl<'a> AlertsService<'a> {
    /// Create a new alerts service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    pub async fn create(
        &self,
        request: &CreatePriceAlertRequest,
    ) -> Result<PriceAlertActionResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "price-alert",
            request,
        )
        .await
    }

    pub async fn list(&self, query: &PriceAlertQuery) -> Result<PriceAlertsResponse> {
        let mut url = join_base_path(self.client.config().api_base_url(), "price-alert");
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(name) = &query.name {
                pairs.append_pair("name", name);
            }
            if let Some(alert_id) = &query.alert_id {
                pairs.append_pair("alertId", alert_id);
            }
        }

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }

    pub async fn modify(
        &self,
        request: &ModifyPriceAlertRequest,
    ) -> Result<PriceAlertActionResponse> {
        put_authenticated_json(
            self.client.http(),
            self.client.config(),
            "price-alert",
            request,
        )
        .await
    }

    pub async fn delete(
        &self,
        request: &DeletePriceAlertRequest,
    ) -> Result<PriceAlertActionResponse> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "price-alert",
            request,
        )
        .await
    }

    pub async fn toggle(
        &self,
        request: &TogglePriceAlertRequest,
    ) -> Result<PriceAlertActionResponse> {
        put_authenticated_json(
            self.client.http(),
            self.client.config(),
            "toggle-alert",
            request,
        )
        .await
    }
}
