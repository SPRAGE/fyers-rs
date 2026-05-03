//! Market data REST API service.

use bytes::Bytes;

use crate::client::FyersClient;
use crate::error::Result;
use crate::models::market_data::{
    HistoryRequest, HistoryResponse, MarketDepthRequest, MarketDepthResponse, MarketStatusResponse,
    OptionChainRequest, OptionChainResponse, QuotesRequest, QuotesResponse,
    SymbolMasterExchangeSegment, SymbolMasterJson,
};
use crate::transport::{get_authenticated_base_json, get_authenticated_url_json, join_base_path};

/// Accessor for Fyers market data REST APIs.
#[derive(Debug, Clone, Copy)]
pub struct MarketDataService<'a> {
    client: &'a FyersClient,
}

impl<'a> MarketDataService<'a> {
    /// Create a new market data service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    pub async fn market_status(&self) -> Result<MarketStatusResponse> {
        get_authenticated_base_json(
            self.client.http(),
            self.client.config(),
            self.client.config().data_base_url(),
            "marketStatus",
        )
        .await
    }

    pub async fn history(&self, request: &HistoryRequest) -> Result<HistoryResponse> {
        let mut url = join_base_path(self.client.config().data_base_url(), "history");
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("symbol", &request.symbol);
            pairs.append_pair("resolution", &request.resolution);
            pairs.append_pair("date_format", &request.date_format.to_string());
            pairs.append_pair("range_from", &request.range_from);
            pairs.append_pair("range_to", &request.range_to);
            if let Some(cont_flag) = request.cont_flag {
                pairs.append_pair("cont_flag", &cont_flag.to_string());
            }
            if let Some(oi_flag) = request.oi_flag {
                pairs.append_pair("oi_flag", &oi_flag.to_string());
            }
        }

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }

    pub async fn quotes(&self, request: &QuotesRequest) -> Result<QuotesResponse> {
        let mut url = join_base_path(self.client.config().data_base_url(), "quotes");
        url.query_pairs_mut()
            .append_pair("symbols", &request.symbols.join(","));

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }

    pub async fn depth(&self, request: &MarketDepthRequest) -> Result<MarketDepthResponse> {
        let mut url = join_base_path(self.client.config().data_base_url(), "depth");
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("symbol", &request.symbol);
            pairs.append_pair("ohlcv_flag", &request.ohlcv_flag.to_string());
        }

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }

    pub async fn option_chain(&self, request: &OptionChainRequest) -> Result<OptionChainResponse> {
        let mut url = join_base_path(self.client.config().data_base_url(), "options-chain-v3");
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("symbol", &request.symbol);
            if let Some(strikecount) = request.strikecount {
                pairs.append_pair("strikecount", &strikecount.to_string());
            }
            if let Some(timestamp) = &request.timestamp {
                pairs.append_pair("timestamp", timestamp);
            }
            if let Some(greeks) = &request.greeks {
                pairs.append_pair("greeks", greeks);
            }
        }

        get_authenticated_url_json(self.client.http(), self.client.config(), url).await
    }

    pub async fn symbol_master_csv(&self, segment: SymbolMasterExchangeSegment) -> Result<Bytes> {
        let url = join_base_path(
            self.client.config().symbols_base_url(),
            segment.csv_file_name(),
        );
        let response = self.client.http().get(url).send().await?;

        Ok(response.bytes().await?)
    }

    pub async fn symbol_master_json(
        &self,
        segment: SymbolMasterExchangeSegment,
    ) -> Result<SymbolMasterJson> {
        let url = join_base_path(
            self.client.config().symbols_base_url(),
            segment.json_file_name(),
        );
        let response = self.client.http().get(url).send().await?;

        Ok(response.json::<SymbolMasterJson>().await?)
    }
}
