//! Orders API service.

use crate::client::FyersClient;
use crate::error::{FyersError, Result};
use crate::models::margin::{
    MultiOrderMarginRequest, MultiOrderMarginResponse, SpanMarginRequest, SpanMarginResponse,
};
use crate::models::orders::{
    AsyncMultiOrderActionResponse, AsyncOrderActionResponse, CancelOrderRequest,
    ModifyOrderRequest, MultiLegOrderRequest, MultiOrderActionResponse, OrderActionResponse,
    OrderBookQuery, OrderBookResponse, PlaceOrderRequest,
};
use crate::transport::{
    delete_authenticated_empty_json, delete_authenticated_json, get_authenticated_url_json,
    join_base_path, patch_authenticated_json, post_authenticated_base_json,
    post_authenticated_json,
};

const MAX_MULTI_ORDER_COUNT: usize = 10;

/// Accessor for Fyers order APIs.
#[derive(Debug, Clone, Copy)]
pub struct OrdersService<'a> {
    client: &'a FyersClient,
}

impl<'a> OrdersService<'a> {
    /// Create a new orders service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Fetch the authenticated user's order book, optionally filtered by order ID or order tag.
    pub async fn list(&self, query: &OrderBookQuery) -> Result<OrderBookResponse> {
        let mut url = join_base_path(self.client.config().api_base_url(), "orders");

        match (&query.id, &query.order_tag) {
            (Some(_), Some(_)) => {
                return Err(FyersError::Validation(
                    "order book query supports either id or order_tag, not both".to_owned(),
                ));
            }
            (Some(id), None) => {
                url.query_pairs_mut().append_pair("id", id);
            }
            (None, Some(order_tag)) => {
                url.query_pairs_mut().append_pair("order_tag", order_tag);
            }
            (None, None) => {}
        }

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }

    /// Fetch an order by the documented order ID query parameter.
    pub async fn get_by_id(&self, id: impl Into<String>) -> Result<OrderBookResponse> {
        self.list(&OrderBookQuery::by_id(id)).await
    }

    /// Place a synchronous single order.
    pub async fn place_sync(&self, request: &PlaceOrderRequest) -> Result<OrderActionResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "orders/sync",
            request,
        )
        .await
    }

    /// Queue an asynchronous single order.
    pub async fn place_async(
        &self,
        request: &PlaceOrderRequest,
    ) -> Result<AsyncOrderActionResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "orders/async",
            request,
        )
        .await
    }

    /// Queue an asynchronous single-order modification.
    pub async fn modify_async(
        &self,
        request: &ModifyOrderRequest,
    ) -> Result<AsyncOrderActionResponse> {
        patch_authenticated_json(
            self.client.http(),
            self.client.config(),
            "orders/async",
            request,
        )
        .await
    }

    /// Queue an asynchronous single-order cancellation.
    pub async fn cancel_async(
        &self,
        request: &CancelOrderRequest,
    ) -> Result<AsyncOrderActionResponse> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "orders/async",
            request,
        )
        .await
    }

    /// Place up to 10 synchronous orders in one request.
    pub async fn place_multi_sync(
        &self,
        requests: &[PlaceOrderRequest],
    ) -> Result<MultiOrderActionResponse> {
        if requests.is_empty() {
            return Err(FyersError::Validation(
                "multi-order placement requires at least one order".to_owned(),
            ));
        }
        if requests.len() > MAX_MULTI_ORDER_COUNT {
            return Err(FyersError::Validation(format!(
                "multi-order placement supports at most {MAX_MULTI_ORDER_COUNT} orders"
            )));
        }

        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "multi-order/sync",
            requests,
        )
        .await
    }

    /// Queue up to 10 asynchronous orders in one request.
    pub async fn place_multi_async(
        &self,
        requests: &[PlaceOrderRequest],
    ) -> Result<AsyncMultiOrderActionResponse> {
        validate_multi_order_count(requests.len(), "multi-order placement")?;

        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "multi-order/async",
            requests,
        )
        .await
    }

    /// Modify up to 10 synchronous orders in one request.
    pub async fn modify_multi_sync(
        &self,
        requests: &[ModifyOrderRequest],
    ) -> Result<MultiOrderActionResponse> {
        validate_multi_order_count(requests.len(), "multi-order modification")?;

        patch_authenticated_json(
            self.client.http(),
            self.client.config(),
            "multi-order/sync",
            requests,
        )
        .await
    }

    /// Queue up to 10 asynchronous order modifications in one request.
    pub async fn modify_multi_async(
        &self,
        requests: &[ModifyOrderRequest],
    ) -> Result<AsyncMultiOrderActionResponse> {
        validate_multi_order_count(requests.len(), "multi-order modification")?;

        patch_authenticated_json(
            self.client.http(),
            self.client.config(),
            "multi-order/async",
            requests,
        )
        .await
    }

    /// Cancel up to 10 synchronous orders in one request.
    pub async fn cancel_multi_sync(
        &self,
        requests: &[CancelOrderRequest],
    ) -> Result<MultiOrderActionResponse> {
        validate_multi_order_count(requests.len(), "multi-order cancellation")?;

        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "multi-order/sync",
            requests,
        )
        .await
    }

    /// Queue up to 10 asynchronous order cancellations in one request.
    pub async fn cancel_multi_async(
        &self,
        requests: &[CancelOrderRequest],
    ) -> Result<AsyncMultiOrderActionResponse> {
        validate_multi_order_count(requests.len(), "multi-order cancellation")?;

        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "multi-order/async",
            requests,
        )
        .await
    }

    /// Place a synchronous multi-leg order.
    pub async fn place_multileg_sync(
        &self,
        request: &MultiLegOrderRequest,
    ) -> Result<OrderActionResponse> {
        if request.order_type == "3L" && request.legs.leg3.is_none() {
            return Err(FyersError::Validation(
                "leg3 is required for 3L multi-leg orders".to_owned(),
            ));
        }

        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "multileg/orders/sync",
            request,
        )
        .await
    }

    /// Modify a synchronous single order.
    pub async fn modify_sync(&self, request: &ModifyOrderRequest) -> Result<OrderActionResponse> {
        patch_authenticated_json(
            self.client.http(),
            self.client.config(),
            "orders/sync",
            request,
        )
        .await
    }

    /// Cancel a synchronous single order using the documented JSON body form.
    pub async fn cancel_sync(&self, request: &CancelOrderRequest) -> Result<OrderActionResponse> {
        delete_authenticated_json(
            self.client.http(),
            self.client.config(),
            "orders/sync",
            request,
        )
        .await
    }

    /// Cancel a synchronous single order using the documented path form.
    pub async fn cancel_sync_by_id(&self, id: impl AsRef<str>) -> Result<OrderActionResponse> {
        let id = id.as_ref();
        if id.is_empty() {
            return Err(FyersError::Validation(
                "order id is required for path cancellation".to_owned(),
            ));
        }

        delete_authenticated_empty_json(
            self.client.http(),
            self.client.config(),
            &format!("orders/{id}/sync"),
        )
        .await
    }

    /// Calculate legacy span/exposure margin.
    pub async fn span_margin(&self, request: &SpanMarginRequest) -> Result<SpanMarginResponse> {
        post_authenticated_base_json(
            self.client.http(),
            self.client.config(),
            self.client.config().api_v2_base_url(),
            "span_margin",
            request,
        )
        .await
    }

    /// Calculate margin for multiple order bodies.
    pub async fn multiorder_margin(
        &self,
        request: &MultiOrderMarginRequest,
    ) -> Result<MultiOrderMarginResponse> {
        post_authenticated_json(
            self.client.http(),
            self.client.config(),
            "multiorder/margin",
            request,
        )
        .await
    }
}

fn validate_multi_order_count(count: usize, operation: &str) -> Result<()> {
    if count == 0 {
        return Err(FyersError::Validation(format!(
            "{operation} requires at least one order"
        )));
    }
    if count > MAX_MULTI_ORDER_COUNT {
        return Err(FyersError::Validation(format!(
            "{operation} supports at most {MAX_MULTI_ORDER_COUNT} orders"
        )));
    }

    Ok(())
}
