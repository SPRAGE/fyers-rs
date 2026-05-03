//! Reports API service.

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::reports::{
    OrderHistoryQuery, OrderHistoryResponse, TradeHistoryQuery, TradeHistoryResponse,
};
use crate::transport::{get_authenticated_url_json, join_base_path};
use url::Url;

/// Accessor for Fyers order/trade history report APIs.
#[derive(Debug, Clone, Copy)]
pub struct ReportsService<'a> {
    client: &'a FyersClient,
}

impl<'a> ReportsService<'a> {
    /// Create a new reports service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Fetch historical orders using the documented report filters.
    pub async fn order_history(&self, query: &OrderHistoryQuery) -> Result<OrderHistoryResponse> {
        let mut url = join_base_path(self.client.config().api_base_url(), "order-history");
        append_history_query(
            &mut url,
            HistoryQueryParams {
                exchange_type: query.exchange_type.as_deref(),
                segment_type: query.segment_type.as_deref(),
                status: query.status.as_deref(),
                symbol: query.symbol.as_deref(),
                from_date: query.from_date.as_deref(),
                to_date: query.to_date.as_deref(),
                page_no: query.page_no,
                page_size: query.page_size,
            },
        );

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }

    /// Fetch historical trades using the documented report filters.
    pub async fn trade_history(&self, query: &TradeHistoryQuery) -> Result<TradeHistoryResponse> {
        let mut url = join_base_path(self.client.config().api_base_url(), "trade-history");
        append_history_query(
            &mut url,
            HistoryQueryParams {
                exchange_type: query.exchange_type.as_deref(),
                segment_type: query.segment_type.as_deref(),
                status: None,
                symbol: query.symbol.as_deref(),
                from_date: query.from_date.as_deref(),
                to_date: query.to_date.as_deref(),
                page_no: query.page_no,
                page_size: query.page_size,
            },
        );

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }
}

struct HistoryQueryParams<'a> {
    exchange_type: Option<&'a str>,
    segment_type: Option<&'a str>,
    status: Option<&'a str>,
    symbol: Option<&'a str>,
    from_date: Option<&'a str>,
    to_date: Option<&'a str>,
    page_no: Option<u32>,
    page_size: Option<u32>,
}

fn append_history_query(url: &mut Url, params: HistoryQueryParams<'_>) {
    let mut query = url.query_pairs_mut();

    if let Some(exchange_type) = params.exchange_type {
        query.append_pair("exchange_type", exchange_type);
    }
    if let Some(segment_type) = params.segment_type {
        query.append_pair("segment_type", segment_type);
    }
    if let Some(status) = params.status {
        query.append_pair("status", status);
    }
    if let Some(symbol) = params.symbol {
        query.append_pair("symbol", symbol);
    }
    if let Some(from_date) = params.from_date {
        query.append_pair("from_date", from_date);
    }
    if let Some(to_date) = params.to_date {
        query.append_pair("to_date", to_date);
    }
    if let Some(page_no) = params.page_no {
        query.append_pair("page_no", &page_no.to_string());
    }
    if let Some(page_size) = params.page_size {
        query.append_pair("page_size", &page_size.to_string());
    }
}
