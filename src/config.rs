//! Client configuration and secret handling.

use std::fmt;
use std::time::Duration;

use url::Url;

const DEFAULT_API_BASE_URL: &str = "https://api-t1.fyers.in/api/v3";
const DEFAULT_API_V2_BASE_URL: &str = "https://api.fyers.in/api/v2";
const DEFAULT_DATA_BASE_URL: &str = "https://api-t1.fyers.in/data";
const DEFAULT_SYMBOLS_BASE_URL: &str = "https://public.fyers.in/sym_details";
const DEFAULT_DATA_SOCKET_URL: &str = "wss://socket.fyers.in/hsm/v1-5/prod";
const DEFAULT_ORDER_SOCKET_URL: &str = "wss://socket.fyers.in/trade/v3";
const DEFAULT_TBT_SOCKET_URL: &str = "wss://rtsocket-api.fyers.in/versova";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// A secret string that redacts its value in debug output.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    /// Wrap a secret value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Expose the secret value for transport/authentication code.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString([redacted])")
    }
}

/// Shared configuration for [`crate::FyersClient`].
#[derive(Clone, Debug)]
pub struct FyersConfig {
    client_id: String,
    secret_key: Option<SecretString>,
    redirect_uri: Option<Url>,
    access_token: Option<SecretString>,
    api_base_url: Url,
    api_v2_base_url: Url,
    data_base_url: Url,
    symbols_base_url: Url,
    data_socket_url: Url,
    order_socket_url: Url,
    tbt_socket_url: Url,
    timeout: Duration,
}

impl FyersConfig {
    /// Create a config from all validated values.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        client_id: String,
        secret_key: Option<SecretString>,
        redirect_uri: Option<Url>,
        access_token: Option<SecretString>,
        api_base_url: Url,
        api_v2_base_url: Url,
        data_base_url: Url,
        symbols_base_url: Url,
        data_socket_url: Url,
        order_socket_url: Url,
        tbt_socket_url: Url,
        timeout: Duration,
    ) -> Self {
        Self {
            client_id,
            secret_key,
            redirect_uri,
            access_token,
            api_base_url,
            api_v2_base_url,
            data_base_url,
            symbols_base_url,
            data_socket_url,
            order_socket_url,
            tbt_socket_url,
            timeout,
        }
    }

    /// Fyers app/client ID.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Optional app secret key.
    pub fn secret_key(&self) -> Option<&SecretString> {
        self.secret_key.as_ref()
    }

    /// Optional redirect URI for auth flows.
    pub fn redirect_uri(&self) -> Option<&Url> {
        self.redirect_uri.as_ref()
    }

    /// Optional access token.
    pub fn access_token(&self) -> Option<&SecretString> {
        self.access_token.as_ref()
    }

    /// REST API base URL.
    pub fn api_base_url(&self) -> &Url {
        &self.api_base_url
    }

    /// Legacy v2 REST API base URL.
    pub fn api_v2_base_url(&self) -> &Url {
        &self.api_v2_base_url
    }

    /// Market-data REST base URL.
    pub fn data_base_url(&self) -> &Url {
        &self.data_base_url
    }

    /// Public symbol-master file base URL.
    pub fn symbols_base_url(&self) -> &Url {
        &self.symbols_base_url
    }

    /// Market-data WebSocket URL.
    pub fn data_socket_url(&self) -> &Url {
        &self.data_socket_url
    }

    /// Order WebSocket URL.
    pub fn order_socket_url(&self) -> &Url {
        &self.order_socket_url
    }

    /// TBT/depth WebSocket URL.
    pub fn tbt_socket_url(&self) -> &Url {
        &self.tbt_socket_url
    }

    /// HTTP timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Default Fyers REST API base URL.
pub fn default_api_base_url() -> Url {
    Url::parse(DEFAULT_API_BASE_URL).expect("default API base URL is valid")
}

/// Default Fyers legacy v2 REST API base URL.
pub fn default_api_v2_base_url() -> Url {
    Url::parse(DEFAULT_API_V2_BASE_URL).expect("default API v2 base URL is valid")
}

/// Default Fyers market-data REST base URL.
pub fn default_data_base_url() -> Url {
    Url::parse(DEFAULT_DATA_BASE_URL).expect("default data base URL is valid")
}

/// Default Fyers public symbol-master base URL.
pub fn default_symbols_base_url() -> Url {
    Url::parse(DEFAULT_SYMBOLS_BASE_URL).expect("default symbols base URL is valid")
}

/// Default Fyers market-data WebSocket URL.
pub fn default_data_socket_url() -> Url {
    Url::parse(DEFAULT_DATA_SOCKET_URL).expect("default data socket URL is valid")
}

/// Default Fyers order WebSocket URL.
pub fn default_order_socket_url() -> Url {
    Url::parse(DEFAULT_ORDER_SOCKET_URL).expect("default order socket URL is valid")
}

/// Default Fyers TBT/depth WebSocket URL.
pub fn default_tbt_socket_url() -> Url {
    Url::parse(DEFAULT_TBT_SOCKET_URL).expect("default TBT socket URL is valid")
}

/// Default client timeout.
pub fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_TIMEOUT_SECS)
}
