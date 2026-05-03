//! Main `FyersClient` entry point.

use url::Url;

use crate::auth::AuthService;
use crate::config::{
    FyersConfig, SecretString, default_api_base_url, default_api_v2_base_url,
    default_data_base_url, default_data_socket_url, default_order_socket_url,
    default_symbols_base_url, default_tbt_socket_url, default_timeout,
};
use crate::error::{FyersError, Result};
use crate::rest::alerts::AlertsService;
use crate::rest::edis::EdisService;
use crate::rest::funds::FundsService;
use crate::rest::gtt::GttService;
use crate::rest::holdings::HoldingsService;
use crate::rest::market_data::MarketDataService;
use crate::rest::orders::OrdersService;
use crate::rest::positions::PositionsService;
use crate::rest::profile::ProfileService;
use crate::rest::reports::ReportsService;
use crate::rest::smart_orders::SmartOrdersService;
use crate::rest::trades::TradesService;
use crate::ws::{DataSocketService, OrderSocketService, TbtSocketService};

/// Main async client for the Fyers API.
#[derive(Clone, Debug)]
pub struct FyersClient {
    config: FyersConfig,
    http: reqwest::Client,
}

impl FyersClient {
    /// Start building a new client.
    pub fn builder() -> FyersClientBuilder {
        FyersClientBuilder::default()
    }

    /// Create a client from an existing config and HTTP client.
    pub fn from_parts(config: FyersConfig, http: reqwest::Client) -> Self {
        Self { config, http }
    }

    /// Access the client configuration.
    pub fn config(&self) -> &FyersConfig {
        &self.config
    }

    /// Access the underlying HTTP client.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Auth/session API accessor.
    pub fn auth(&self) -> AuthService<'_> {
        AuthService::new(self)
    }

    /// Profile/user API accessor.
    pub fn profile(&self) -> ProfileService<'_> {
        ProfileService::new(self)
    }

    /// Funds API accessor.
    pub fn funds(&self) -> FundsService<'_> {
        FundsService::new(self)
    }

    /// Holdings API accessor.
    pub fn holdings(&self) -> HoldingsService<'_> {
        HoldingsService::new(self)
    }

    /// Orders API accessor.
    pub fn orders(&self) -> OrdersService<'_> {
        OrdersService::new(self)
    }

    /// Trades API accessor.
    pub fn trades(&self) -> TradesService<'_> {
        TradesService::new(self)
    }

    /// Positions API accessor.
    pub fn positions(&self) -> PositionsService<'_> {
        PositionsService::new(self)
    }

    /// GTT orders API accessor.
    pub fn gtt(&self) -> GttService<'_> {
        GttService::new(self)
    }

    /// Smart orders API accessor.
    pub fn smart_orders(&self) -> SmartOrdersService<'_> {
        SmartOrdersService::new(self)
    }

    /// Market data REST API accessor.
    pub fn market_data(&self) -> MarketDataService<'_> {
        MarketDataService::new(self)
    }

    /// Price alerts API accessor.
    pub fn alerts(&self) -> AlertsService<'_> {
        AlertsService::new(self)
    }

    /// Reports API accessor.
    pub fn reports(&self) -> ReportsService<'_> {
        ReportsService::new(self)
    }

    /// EDIS API accessor.
    pub fn edis(&self) -> EdisService<'_> {
        EdisService::new(self)
    }

    /// Market data WebSocket accessor.
    pub fn data_socket(&self) -> DataSocketService<'_> {
        DataSocketService::new(self)
    }

    /// Order WebSocket accessor.
    pub fn order_socket(&self) -> OrderSocketService<'_> {
        OrderSocketService::new(self)
    }

    /// TBT/depth WebSocket accessor.
    pub fn tbt_socket(&self) -> TbtSocketService<'_> {
        TbtSocketService::new(self)
    }
}

/// Builder for [`FyersClient`].
#[derive(Debug, Default)]
pub struct FyersClientBuilder {
    client_id: Option<String>,
    secret_key: Option<String>,
    redirect_uri: Option<Url>,
    access_token: Option<String>,
    api_base_url: Option<Url>,
    api_v2_base_url: Option<Url>,
    data_base_url: Option<Url>,
    symbols_base_url: Option<Url>,
    data_socket_url: Option<Url>,
    order_socket_url: Option<Url>,
    tbt_socket_url: Option<Url>,
    http: Option<reqwest::Client>,
    timeout: Option<std::time::Duration>,
}

impl FyersClientBuilder {
    /// Set the Fyers app/client ID.
    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Set the Fyers app secret key.
    pub fn secret_key(mut self, secret_key: impl Into<String>) -> Self {
        self.secret_key = Some(secret_key.into());
        self
    }

    /// Set the redirect URI.
    pub fn redirect_uri(mut self, redirect_uri: Url) -> Self {
        self.redirect_uri = Some(redirect_uri);
        self
    }

    /// Parse and set the redirect URI.
    pub fn redirect_uri_str(mut self, redirect_uri: impl AsRef<str>) -> Result<Self> {
        self.redirect_uri = Some(Url::parse(redirect_uri.as_ref())?);
        Ok(self)
    }

    /// Set the current access token.
    pub fn access_token(mut self, access_token: impl Into<String>) -> Self {
        self.access_token = Some(access_token.into());
        self
    }

    /// Override the REST API base URL.
    pub fn api_base_url(mut self, api_base_url: Url) -> Self {
        self.api_base_url = Some(api_base_url);
        self
    }

    /// Parse and override the REST API base URL.
    pub fn api_base_url_str(mut self, api_base_url: impl AsRef<str>) -> Result<Self> {
        self.api_base_url = Some(Url::parse(api_base_url.as_ref())?);
        Ok(self)
    }

    /// Override the legacy v2 REST API base URL.
    pub fn api_v2_base_url(mut self, api_v2_base_url: Url) -> Self {
        self.api_v2_base_url = Some(api_v2_base_url);
        self
    }

    /// Parse and override the legacy v2 REST API base URL.
    pub fn api_v2_base_url_str(mut self, api_v2_base_url: impl AsRef<str>) -> Result<Self> {
        self.api_v2_base_url = Some(Url::parse(api_v2_base_url.as_ref())?);
        Ok(self)
    }

    /// Override the market-data REST base URL.
    pub fn data_base_url(mut self, data_base_url: Url) -> Self {
        self.data_base_url = Some(data_base_url);
        self
    }

    /// Parse and override the market-data REST base URL.
    pub fn data_base_url_str(mut self, data_base_url: impl AsRef<str>) -> Result<Self> {
        self.data_base_url = Some(Url::parse(data_base_url.as_ref())?);
        Ok(self)
    }

    /// Override the public symbol-master base URL.
    pub fn symbols_base_url(mut self, symbols_base_url: Url) -> Self {
        self.symbols_base_url = Some(symbols_base_url);
        self
    }

    /// Parse and override the public symbol-master base URL.
    pub fn symbols_base_url_str(mut self, symbols_base_url: impl AsRef<str>) -> Result<Self> {
        self.symbols_base_url = Some(Url::parse(symbols_base_url.as_ref())?);
        Ok(self)
    }

    /// Override the market-data WebSocket URL.
    pub fn data_socket_url(mut self, data_socket_url: Url) -> Self {
        self.data_socket_url = Some(data_socket_url);
        self
    }

    /// Parse and override the market-data WebSocket URL.
    pub fn data_socket_url_str(mut self, data_socket_url: impl AsRef<str>) -> Result<Self> {
        self.data_socket_url = Some(Url::parse(data_socket_url.as_ref())?);
        Ok(self)
    }

    /// Override the order WebSocket URL.
    pub fn order_socket_url(mut self, order_socket_url: Url) -> Self {
        self.order_socket_url = Some(order_socket_url);
        self
    }

    /// Parse and override the order WebSocket URL.
    pub fn order_socket_url_str(mut self, order_socket_url: impl AsRef<str>) -> Result<Self> {
        self.order_socket_url = Some(Url::parse(order_socket_url.as_ref())?);
        Ok(self)
    }

    /// Override the TBT/depth WebSocket URL.
    pub fn tbt_socket_url(mut self, tbt_socket_url: Url) -> Self {
        self.tbt_socket_url = Some(tbt_socket_url);
        self
    }

    /// Parse and override the TBT/depth WebSocket URL.
    pub fn tbt_socket_url_str(mut self, tbt_socket_url: impl AsRef<str>) -> Result<Self> {
        self.tbt_socket_url = Some(Url::parse(tbt_socket_url.as_ref())?);
        Ok(self)
    }

    /// Provide a preconfigured HTTP client.
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    /// Set the HTTP timeout.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<FyersClient> {
        let client_id = self
            .client_id
            .ok_or(FyersError::MissingConfig { field: "client_id" })?;
        let timeout = self.timeout.unwrap_or_else(default_timeout);
        let config = FyersConfig::new(
            client_id,
            self.secret_key.map(SecretString::new),
            self.redirect_uri,
            self.access_token.map(SecretString::new),
            self.api_base_url.unwrap_or_else(default_api_base_url),
            self.api_v2_base_url.unwrap_or_else(default_api_v2_base_url),
            self.data_base_url.unwrap_or_else(default_data_base_url),
            self.symbols_base_url
                .unwrap_or_else(default_symbols_base_url),
            self.data_socket_url.unwrap_or_else(default_data_socket_url),
            self.order_socket_url
                .unwrap_or_else(default_order_socket_url),
            self.tbt_socket_url.unwrap_or_else(default_tbt_socket_url),
            timeout,
        );
        let http = match self.http {
            Some(http) => http,
            None => reqwest::Client::builder().timeout(timeout).build()?,
        };

        Ok(FyersClient::from_parts(config, http))
    }
}
