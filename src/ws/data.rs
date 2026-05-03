//! Market data WebSocket service.

use std::collections::BTreeSet;

use futures_util::{Sink, Stream};
use tokio_tungstenite::tungstenite::Message;

use crate::client::FyersClient;
use crate::error::{FyersError, Result};
use crate::models::ws::{
    DataSocketConfig, DataSocketEvent, DataSubscribeRequest, DataUnsubscribeRequest,
    parse_data_event,
};
use crate::ws::manager::{LiveWebSocket, ManagedSocket, ReconnectPolicy, connect_live_socket};
use crate::ws::protocol::SocketKind;

const MAX_DATA_SOCKET_SYMBOLS: usize = 5000;

/// Live market-data WebSocket connection.
pub type LiveDataSocketConnection = DataSocketConnection<LiveWebSocket>;

/// Accessor for Fyers market data WebSocket APIs.
#[derive(Debug, Clone, Copy)]
pub struct DataSocketService<'a> {
    client: &'a FyersClient,
}

impl<'a> DataSocketService<'a> {
    /// Create a new data socket service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Connect to the market-data socket with default manager configuration.
    pub async fn connect(&self) -> Result<LiveDataSocketConnection> {
        self.connect_with_config(DataSocketConfig::default()).await
    }

    /// Connect to the market-data socket with explicit manager configuration.
    pub async fn connect_with_config(
        &self,
        config: DataSocketConfig,
    ) -> Result<LiveDataSocketConnection> {
        let stream = connect_live_socket(self.client.config(), SocketKind::Data).await?;

        Ok(DataSocketConnection::from_stream(stream, config))
    }

    /// Build a market-data manager from an already connected stream.
    pub fn connect_with_stream<S>(&self, stream: S) -> DataSocketConnection<S>
    where
        S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin,
    {
        DataSocketConnection::from_stream(stream, DataSocketConfig::default())
    }
}

/// Typed market-data WebSocket manager.
#[derive(Debug)]
pub struct DataSocketConnection<S = LiveWebSocket> {
    socket: ManagedSocket<S, DataSocketEvent>,
    subscriptions: Vec<DataSubscribeRequest>,
    config: DataSocketConfig,
}

impl<S> DataSocketConnection<S>
where
    S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    /// Create a data-socket manager from an already connected stream.
    pub fn from_stream(stream: S, config: DataSocketConfig) -> Self {
        let reconnect_policy = ReconnectPolicy::new(
            config.reconnect,
            config.reconnect_retry,
            config.queue_process_interval.as_duration(),
        );
        Self {
            socket: ManagedSocket::from_stream(
                SocketKind::Data,
                stream,
                parse_data_message,
                reconnect_policy,
            ),
            subscriptions: Vec::new(),
            config,
        }
    }

    /// Manager configuration.
    pub const fn config(&self) -> &DataSocketConfig {
        &self.config
    }

    /// Underlying generic socket manager.
    pub const fn socket(&self) -> &ManagedSocket<S, DataSocketEvent> {
        &self.socket
    }

    /// Mutable access to the underlying generic socket manager.
    pub const fn socket_mut(&mut self) -> &mut ManagedSocket<S, DataSocketEvent> {
        &mut self.socket
    }

    /// Subscribe to symbol or depth updates.
    pub async fn subscribe(&mut self, request: &DataSubscribeRequest) -> Result<()> {
        if request.symbols.len() > MAX_DATA_SOCKET_SYMBOLS
            || active_symbol_count_after(&self.subscriptions, request) > MAX_DATA_SOCKET_SYMBOLS
        {
            return Err(FyersError::Validation(
                "data WebSocket subscriptions cannot exceed 5000 symbols".to_owned(),
            ));
        }

        self.socket
            .send_text(serde_json::to_string(request)?)
            .await?;
        if !self.subscriptions.contains(request) {
            self.subscriptions.push(request.clone());
        }

        Ok(())
    }

    /// Unsubscribe from symbol or depth updates.
    pub async fn unsubscribe(&mut self, request: &DataUnsubscribeRequest) -> Result<()> {
        self.socket
            .send_text(serde_json::to_string(request)?)
            .await?;
        self.subscriptions.retain(|existing| existing != request);

        Ok(())
    }

    /// Current active subscription commands to replay after reconnect.
    pub fn resubscribe_frames(&self) -> Result<Vec<String>> {
        self.subscriptions
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(FyersError::from)
    }

    /// Receive the next typed market-data event.
    pub async fn next_event(&mut self) -> Result<Option<DataSocketEvent>> {
        self.socket.next_event().await
    }

    /// Close the socket.
    pub async fn close(&mut self) -> Result<()> {
        self.socket.close().await
    }
}

fn parse_data_message(message: Message) -> Result<Option<DataSocketEvent>> {
    match message {
        Message::Text(text) => parse_data_event(text.as_str())
            .map(Some)
            .map_err(FyersError::Validation),
        Message::Binary(_) => Err(FyersError::Validation(
            "market-data socket received unexpected binary frame".to_owned(),
        )),
        _ => Ok(None),
    }
}

fn active_symbol_count_after(
    subscriptions: &[DataSubscribeRequest],
    request: &DataSubscribeRequest,
) -> usize {
    subscriptions
        .iter()
        .flat_map(|subscription| subscription.symbols.iter())
        .chain(request.symbols.iter())
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        .len()
}
