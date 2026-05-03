//! Shared async WebSocket manager primitives.

use std::time::Duration;

use futures_util::{Sink, SinkExt, Stream, StreamExt};
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::{Bytes, Message};

use crate::config::FyersConfig;
use crate::error::{FyersError, Result};
use crate::ws::protocol::SocketKind;

/// Live WebSocket stream used by production connections.
pub type LiveWebSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// A parsed WebSocket event returned by a manager.
pub type SocketEventResult<E> = Result<Option<E>>;

/// Parser function used by a typed socket manager.
pub type SocketFrameParser<E> = fn(Message) -> SocketEventResult<E>;

/// Reconnect policy derived from documented SDK socket config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReconnectPolicy {
    enabled: bool,
    max_retries: usize,
    delay: Duration,
}

impl ReconnectPolicy {
    /// Create a bounded reconnect policy.
    pub const fn new(enabled: bool, max_retries: usize, delay: Duration) -> Self {
        Self {
            enabled,
            max_retries,
            delay,
        }
    }

    /// A disabled reconnect policy.
    pub const fn disabled() -> Self {
        Self::new(false, 0, Duration::ZERO)
    }

    /// Whether reconnect handling is enabled.
    pub const fn enabled(self) -> bool {
        self.enabled
    }

    /// Maximum retry attempts.
    pub const fn max_retries(self) -> usize {
        self.max_retries
    }

    /// Delay between retry attempts.
    pub const fn delay(self) -> Duration {
        self.delay
    }

    /// Return true when another reconnect attempt should be made.
    pub fn should_retry(self, attempts: usize) -> bool {
        self.enabled && attempts < self.max_retries
    }
}

/// Typed async WebSocket connection.
#[derive(Debug)]
pub struct ManagedSocket<S, E> {
    kind: SocketKind,
    stream: S,
    parser: SocketFrameParser<E>,
    reconnect_policy: ReconnectPolicy,
    replay_frames: Vec<Message>,
    closed: bool,
}

impl<S, E> ManagedSocket<S, E>
where
    S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    /// Create a manager from an already connected stream.
    pub fn from_stream(
        kind: SocketKind,
        stream: S,
        parser: SocketFrameParser<E>,
        reconnect_policy: ReconnectPolicy,
    ) -> Self {
        Self {
            kind,
            stream,
            parser,
            reconnect_policy,
            replay_frames: Vec::new(),
            closed: false,
        }
    }

    /// Socket family managed by this connection.
    pub const fn kind(&self) -> SocketKind {
        self.kind
    }

    /// Configured reconnect policy.
    pub const fn reconnect_policy(&self) -> ReconnectPolicy {
        self.reconnect_policy
    }

    /// Whether the socket has been closed locally or by the peer.
    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    /// Number of replayable frames recorded for resubscribe/replay after reconnect.
    pub fn replay_frame_count(&self) -> usize {
        self.replay_frames.len()
    }

    /// Send a raw text frame.
    pub async fn send_text(&mut self, text: impl Into<String>) -> Result<()> {
        self.send_frame(Message::Text(text.into().into()), false)
            .await
    }

    /// Send a raw binary frame.
    pub async fn send_binary(&mut self, bytes: impl Into<Vec<u8>>) -> Result<()> {
        let bytes: Vec<u8> = bytes.into();
        self.send_frame(Message::Binary(Bytes::from(bytes)), false)
            .await
    }

    /// Send a text frame and record it for replay after a future reconnect.
    pub async fn send_replayable_text(&mut self, text: impl Into<String>) -> Result<()> {
        self.send_frame(Message::Text(text.into().into()), true)
            .await
    }

    /// Send a ping frame.
    pub async fn send_ping(&mut self, bytes: impl Into<Vec<u8>>) -> Result<()> {
        let bytes: Vec<u8> = bytes.into();
        self.send_frame(Message::Ping(Bytes::from(bytes)), false)
            .await
    }

    /// Send a documented literal text ping.
    pub async fn send_literal_ping(&mut self) -> Result<()> {
        self.send_text("ping").await
    }

    /// Receive and parse the next typed event.
    pub async fn next_event(&mut self) -> SocketEventResult<E> {
        while let Some(message) = self.stream.next().await {
            let message = message?;
            match message {
                Message::Ping(payload) => {
                    self.stream.send(Message::Pong(payload)).await?;
                }
                Message::Pong(_) => {}
                Message::Close(frame) => {
                    self.closed = true;
                    if frame.is_some() {
                        self.stream.send(Message::Close(frame)).await?;
                    }
                    return Ok(None);
                }
                other => {
                    if let Some(event) = (self.parser)(other)? {
                        return Ok(Some(event));
                    }
                }
            }
        }

        self.closed = true;
        Ok(None)
    }

    /// Read the next non-control frame, transparently handling ping/pong/close.
    ///
    /// Use this when the caller wants the raw [`Message`] (for example to
    /// drive a stateful binary parser that the static [`SocketFrameParser`] hook
    /// cannot express).
    pub async fn next_raw_frame(&mut self) -> Result<Option<Message>> {
        while let Some(message) = self.stream.next().await {
            let message = message?;
            match message {
                Message::Ping(payload) => {
                    self.stream.send(Message::Pong(payload)).await?;
                }
                Message::Pong(_) => {}
                Message::Close(frame) => {
                    self.closed = true;
                    if frame.is_some() {
                        self.stream.send(Message::Close(frame)).await?;
                    }
                    return Ok(None);
                }
                other => return Ok(Some(other)),
            }
        }

        self.closed = true;
        Ok(None)
    }

    /// Close the socket.
    pub async fn close(&mut self) -> Result<()> {
        if !self.closed {
            self.stream.send(Message::Close(None)).await?;
            self.closed = true;
        }

        Ok(())
    }

    async fn send_frame(&mut self, frame: Message, replayable: bool) -> Result<()> {
        self.stream.send(frame.clone()).await?;
        if replayable {
            self.replay_frames.push(frame);
        }

        Ok(())
    }
}

/// Connect a live Fyers WebSocket with the documented `appid:access_token`
/// authorization header.
///
/// The data socket at `wss://socket.fyers.in/hsm/v1-5/prod` is an exception:
/// it accepts the WS upgrade unconditionally and expects in-band binary auth.
/// Use [`connect_live_socket_no_auth_header`] for that endpoint.
pub async fn connect_live_socket(config: &FyersConfig, kind: SocketKind) -> Result<LiveWebSocket> {
    let url = match kind {
        SocketKind::Data => config.data_socket_url(),
        SocketKind::Order => config.order_socket_url(),
        SocketKind::Tbt => config.tbt_socket_url(),
    };
    let mut request = url.as_str().into_client_request()?;
    let authorization = authorization_header_value(config)?;

    request.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&authorization).map_err(|err| {
            FyersError::Validation(format!("invalid WebSocket authorization header: {err}"))
        })?,
    );

    let (stream, _) = connect_async(request).await?;
    Ok(stream)
}

/// Connect a live Fyers WebSocket with no Authorization header.
///
/// Used by the data socket, which performs auth in-band via a binary frame
/// after the WS handshake completes (see [`crate::ws::data_protocol`]).
pub async fn connect_live_socket_no_auth_header(
    config: &FyersConfig,
    kind: SocketKind,
) -> Result<LiveWebSocket> {
    let url = match kind {
        SocketKind::Data => config.data_socket_url(),
        SocketKind::Order => config.order_socket_url(),
        SocketKind::Tbt => config.tbt_socket_url(),
    };
    let request = url.as_str().into_client_request()?;
    let (stream, _) = connect_async(request).await?;
    Ok(stream)
}

fn authorization_header_value(config: &FyersConfig) -> Result<String> {
    let access_token = config
        .access_token()
        .ok_or(FyersError::MissingConfig {
            field: "access_token",
        })?
        .expose_secret();

    if access_token.contains(':') {
        Ok(access_token.to_owned())
    } else {
        Ok(format!("{}:{access_token}", config.client_id()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FyersClient;

    #[test]
    fn websocket_authorization_prefixes_token_only_values() {
        let client = FyersClient::builder()
            .client_id("APPID-100")
            .access_token("ACCESS_TOKEN")
            .build()
            .unwrap();

        assert_eq!(
            authorization_header_value(client.config()).unwrap(),
            "APPID-100:ACCESS_TOKEN"
        );
    }

    #[test]
    fn websocket_authorization_preserves_full_token_values() {
        let client = FyersClient::builder()
            .client_id("APPID-100")
            .access_token("APPID-100:ACCESS_TOKEN")
            .build()
            .unwrap();

        assert_eq!(
            authorization_header_value(client.config()).unwrap(),
            "APPID-100:ACCESS_TOKEN"
        );
    }
}
