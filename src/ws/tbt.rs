//! Tick-by-tick/depth WebSocket service.

use futures_util::{Sink, Stream};
use tokio_tungstenite::tungstenite::Message;

use crate::client::FyersClient;
use crate::error::{FyersError, Result};
use crate::models::ws::{
    TbtEvent, TbtSocketConfig, TbtSubscribeData, TbtSubscribeRequest, TbtSwitchChannelRequest,
    parse_tbt_event,
};
use crate::ws::manager::{LiveWebSocket, ManagedSocket, ReconnectPolicy, connect_live_socket};
use crate::ws::protocol::SocketKind;

/// Live TBT/depth WebSocket connection.
pub type LiveTbtSocketConnection = TbtSocketConnection<LiveWebSocket>;

/// Accessor for Fyers TBT/depth WebSocket APIs.
#[derive(Debug, Clone, Copy)]
pub struct TbtSocketService<'a> {
    client: &'a FyersClient,
}

impl<'a> TbtSocketService<'a> {
    /// Create a new TBT socket service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Connect to the TBT/depth socket with default manager configuration.
    pub async fn connect(&self) -> Result<LiveTbtSocketConnection> {
        self.connect_with_config(TbtSocketConfig::default()).await
    }

    /// Connect to the TBT/depth socket with explicit manager configuration.
    pub async fn connect_with_config(
        &self,
        config: TbtSocketConfig,
    ) -> Result<LiveTbtSocketConnection> {
        let stream = connect_live_socket(self.client.config(), SocketKind::Tbt).await?;

        Ok(TbtSocketConnection::from_stream(stream, config))
    }

    /// Build a TBT/depth manager from an already connected stream.
    pub fn connect_with_stream<S>(&self, stream: S) -> TbtSocketConnection<S>
    where
        S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin,
    {
        TbtSocketConnection::from_stream(stream, TbtSocketConfig::default())
    }
}

/// Typed TBT/depth WebSocket manager.
#[derive(Debug)]
pub struct TbtSocketConnection<S = LiveWebSocket> {
    socket: ManagedSocket<S, TbtEvent>,
    depth_subscriptions: Vec<TbtSubscribeRequest>,
    config: TbtSocketConfig,
}

impl<S> TbtSocketConnection<S>
where
    S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    /// Create a TBT/depth socket manager from an already connected stream.
    pub fn from_stream(stream: S, config: TbtSocketConfig) -> Self {
        let reconnect_policy = ReconnectPolicy::new(
            config.reconnect,
            config.reconnect_retry,
            config.ping_interval,
        );
        Self {
            socket: ManagedSocket::from_stream(
                SocketKind::Tbt,
                stream,
                parse_tbt_message,
                reconnect_policy,
            ),
            depth_subscriptions: Vec::new(),
            config,
        }
    }

    /// Manager configuration.
    pub const fn config(&self) -> &TbtSocketConfig {
        &self.config
    }

    /// Underlying generic socket manager.
    pub const fn socket(&self) -> &ManagedSocket<S, TbtEvent> {
        &self.socket
    }

    /// Mutable access to the underlying generic socket manager.
    pub const fn socket_mut(&mut self) -> &mut ManagedSocket<S, TbtEvent> {
        &mut self.socket
    }

    /// Subscribe to TBT depth updates on a channel.
    pub async fn subscribe_depth(
        &mut self,
        symbols: Vec<String>,
        channel: impl Into<String>,
    ) -> Result<()> {
        validate_depth_symbols(&symbols)?;
        let channel = channel.into();
        validate_channel(&channel)?;

        let request = TbtSubscribeRequest {
            request_type: 1,
            data: TbtSubscribeData {
                subs: 1,
                symbols,
                mode: "depth".to_owned(),
                channel,
            },
        };
        self.send_subscription(&request).await?;
        if !self.depth_subscriptions.contains(&request) {
            self.depth_subscriptions.push(request);
        }

        Ok(())
    }

    /// Unsubscribe from TBT depth updates on a channel.
    pub async fn unsubscribe_depth(
        &mut self,
        symbols: Vec<String>,
        channel: impl Into<String>,
    ) -> Result<()> {
        let channel = channel.into();
        validate_depth_symbols(&symbols)?;
        validate_channel(&channel)?;

        let request = TbtSubscribeRequest {
            request_type: 1,
            data: TbtSubscribeData {
                subs: -1,
                symbols,
                mode: "depth".to_owned(),
                channel,
            },
        };
        self.send_subscription(&request).await?;
        self.depth_subscriptions.retain(|existing| {
            existing.data.channel != request.data.channel
                || existing.data.mode != request.data.mode
                || existing.data.symbols != request.data.symbols
        });

        Ok(())
    }

    /// Send a documented TBT subscription command.
    pub async fn send_subscription(&mut self, request: &TbtSubscribeRequest) -> Result<()> {
        self.socket.send_text(serde_json::to_string(request)?).await
    }

    /// Switch active/paused TBT channels.
    pub async fn switch_channel(&mut self, request: &TbtSwitchChannelRequest) -> Result<()> {
        for channel in request
            .data
            .resume_channels
            .iter()
            .chain(request.data.pause_channels.iter())
        {
            validate_channel(channel)?;
        }

        self.socket.send_text(serde_json::to_string(request)?).await
    }

    /// Send the documented literal ping frame.
    pub async fn ping(&mut self) -> Result<()> {
        self.socket.send_literal_ping().await
    }

    /// Current active TBT depth subscription commands to replay after reconnect.
    pub fn resubscribe_frames(&self) -> Result<Vec<String>> {
        self.depth_subscriptions
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(FyersError::from)
    }

    /// Receive the next typed TBT event.
    pub async fn next_event(&mut self) -> Result<Option<TbtEvent>> {
        self.socket.next_event().await
    }

    /// Close the socket.
    pub async fn close(&mut self) -> Result<()> {
        self.socket.close().await
    }
}

fn validate_depth_symbols(symbols: &[String]) -> Result<()> {
    if symbols.is_empty() || symbols.len() > 5 {
        return Err(FyersError::Validation(
            "TBT market-depth subscriptions require 1 to 5 symbols".to_owned(),
        ));
    }

    if let Some(symbol) = symbols
        .iter()
        .find(|symbol| !symbol.starts_with("NSE:") && !symbol.starts_with("NFO:"))
    {
        return Err(FyersError::Validation(format!(
            "TBT market-depth supports NSE/NFO symbols only: {symbol}"
        )));
    }

    Ok(())
}

fn validate_channel(channel: &str) -> Result<()> {
    match channel.parse::<u8>() {
        Ok(1..=50) => Ok(()),
        _ => Err(FyersError::Validation(
            "TBT channel must be a number between 1 and 50".to_owned(),
        )),
    }
}

fn parse_tbt_message(message: Message) -> Result<Option<TbtEvent>> {
    match message {
        Message::Binary(bytes) => parse_tbt_event(&bytes)
            .map(Some)
            .map_err(FyersError::Validation),
        Message::Text(text) if text == "pong" => Ok(None),
        Message::Text(_) => Err(FyersError::Validation(
            "TBT socket received unexpected text frame".to_owned(),
        )),
        _ => Ok(None),
    }
}
