//! Smart orders API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::smart_orders::{
    FlowIdRequest, ModifySmartOrderRequest, SmartLimitOrderRequest, SmartOrderActionResponse,
    SmartOrderBookQuery, SmartOrderBookResponse, SmartSipOrderRequest, SmartStepOrderRequest,
    SmartTrailOrderRequest,
};
use crate::transport::{
    delete_authenticated_json, get_authenticated_url_json, join_base_path,
    patch_authenticated_json, post_authenticated_json,
};

/// Accessor for Fyers smart order APIs.
#[derive(Debug, Clone, Copy)]
pub struct SmartOrdersService<'a> {
    client: &'a FyersClient,
}

impl<'a> SmartOrdersService<'a> {
    /// Create a new smart orders service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    pub async fn create_limit(
        &self,
        request: &SmartLimitOrderRequest,
    ) -> Result<SmartOrderActionResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "smart-order/limit",
            request,
        )
        .await
    }

    pub async fn create_trail(
        &self,
        request: &SmartTrailOrderRequest,
    ) -> Result<SmartOrderActionResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "smart-order/trail",
            request,
        )
        .await
    }

    pub async fn create_step(
        &self,
        request: &SmartStepOrderRequest,
    ) -> Result<SmartOrderActionResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "smart-order/step",
            request,
        )
        .await
    }

    pub async fn create_sip(
        &self,
        request: &SmartSipOrderRequest,
    ) -> Result<SmartOrderActionResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "smart-order/sip",
            request,
        )
        .await
    }

    pub async fn modify(
        &self,
        request: &ModifySmartOrderRequest,
    ) -> Result<SmartOrderActionResponse> {
        patch_authenticated_json(
            self.client.http(),
            self.client.config(),
            "smart-order/modify",
            request,
        )
        .await
    }

    pub async fn cancel(&self, request: &FlowIdRequest) -> Result<SmartOrderActionResponse> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "smart-order/cancel",
            request,
        )
        .await
    }

    pub async fn pause(&self, request: &FlowIdRequest) -> Result<SmartOrderActionResponse> {
        patch_authenticated_json(
            self.client.http(),
            self.client.config(),
            "smart-order/pause",
            request,
        )
        .await
    }

    pub async fn resume(&self, request: &FlowIdRequest) -> Result<SmartOrderActionResponse> {
        patch_authenticated_json(
            self.client.http(),
            self.client.config(),
            "smart-order/resume",
            request,
        )
        .await
    }

    pub async fn orderbook(&self, query: &SmartOrderBookQuery) -> Result<SmartOrderBookResponse> {
        let mut url = join_base_path(self.client.config().api_base_url(), "smart-order/orderbook");
        {
            let mut pairs = url.query_pairs_mut();
            for exchange in &query.exchange {
                pairs.append_pair("exchange", exchange);
            }
            for side in &query.side {
                pairs.append_pair("side", &side.to_string());
            }
            for flowtype in &query.flowtype {
                pairs.append_pair("flowtype", &flowtype.to_string());
            }
            for product in &query.product {
                pairs.append_pair("product", product);
            }
            for message_type in &query.message_type {
                pairs.append_pair("messageType", &message_type.to_string());
            }
            if let Some(search) = &query.search {
                pairs.append_pair("search", search);
            }
            if let Some(sort_by) = &query.sort_by {
                pairs.append_pair("sort_by", sort_by);
            }
            if let Some(ord_by) = query.ord_by {
                pairs.append_pair("ord_by", &ord_by.to_string());
            }
            if let Some(page_no) = query.page_no {
                pairs.append_pair("page_no", &page_no.to_string());
            }
            if let Some(page_size) = query.page_size {
                pairs.append_pair("page_size", &page_size.to_string());
            }
        }

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }
}
