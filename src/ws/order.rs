//! Order/trade/position WebSocket service.

use futures_util::{Sink, Stream};
use tokio_tungstenite::tungstenite::Message;

use crate::client::FyersClient;
use crate::error::{FyersError, Result};
use crate::models::ws::{
    OrderSocketConfig, OrderSocketEvent, OrderSubscribeRequest, parse_order_event,
};
use crate::ws::manager::{LiveWebSocket, ManagedSocket, ReconnectPolicy, connect_live_socket};
use crate::ws::protocol::SocketKind;

/// Live order WebSocket connection.
pub type LiveOrderSocketConnection = OrderSocketConnection<LiveWebSocket>;

/// Accessor for Fyers order WebSocket APIs.
#[derive(Debug, Clone, Copy)]
pub struct OrderSocketService<'a> {
    client: &'a FyersClient,
}

impl<'a> OrderSocketService<'a> {
    /// Create a new order socket service accessor.
    pub(crate) const fn new(client: &'a FyersClient) -> Self {
        Self { client }
    }

    /// Access the underlying client.
    pub const fn client(&self) -> &'a FyersClient {
        self.client
    }

    /// Connect to the order socket with default manager configuration.
    pub async fn connect(&self) -> Result<LiveOrderSocketConnection> {
        self.connect_with_config(OrderSocketConfig::default()).await
    }

    /// Connect to the order socket with explicit manager configuration.
    pub async fn connect_with_config(
        &self,
        config: OrderSocketConfig,
    ) -> Result<LiveOrderSocketConnection> {
        let stream = connect_live_socket(self.client.config(), SocketKind::Order).await?;

        Ok(OrderSocketConnection::from_stream(stream, config))
    }

    /// Build an order socket manager from an already connected stream.
    pub fn connect_with_stream<S>(&self, stream: S) -> OrderSocketConnection<S>
    where
        S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin,
    {
        OrderSocketConnection::from_stream(stream, OrderSocketConfig::default())
    }
}

/// Typed order WebSocket manager.
#[derive(Debug)]
pub struct OrderSocketConnection<S = LiveWebSocket> {
    socket: ManagedSocket<S, OrderSocketEvent>,
    active_actions: Vec<String>,
    config: OrderSocketConfig,
}

impl<S> OrderSocketConnection<S>
where
    S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    /// Create an order-socket manager from an already connected stream.
    pub fn from_stream(stream: S, config: OrderSocketConfig) -> Self {
        let reconnect_policy = ReconnectPolicy::new(
            config.reconnect,
            config.reconnect_retry,
            config.ping_interval,
        );
        Self {
            socket: ManagedSocket::from_stream(
                SocketKind::Order,
                stream,
                parse_order_message,
                reconnect_policy,
            ),
            active_actions: Vec::new(),
            config,
        }
    }

    /// Manager configuration.
    pub const fn config(&self) -> &OrderSocketConfig {
        &self.config
    }

    /// Underlying generic socket manager.
    pub const fn socket(&self) -> &ManagedSocket<S, OrderSocketEvent> {
        &self.socket
    }

    /// Mutable access to the underlying generic socket manager.
    pub const fn socket_mut(&mut self) -> &mut ManagedSocket<S, OrderSocketEvent> {
        &mut self.socket
    }

    /// Subscribe to documented order socket actions.
    pub async fn subscribe(&mut self, actions: Vec<String>) -> Result<()> {
        let actions = normalize_order_actions(actions)?;
        let request = OrderSubscribeRequest::subscribe(actions.clone());
        self.socket
            .send_text(serde_json::to_string(&request)?)
            .await?;

        for action in actions {
            if !self.active_actions.contains(&action) {
                self.active_actions.push(action);
            }
        }

        Ok(())
    }

    /// Unsubscribe from documented order socket actions.
    pub async fn unsubscribe(&mut self, actions: Vec<String>) -> Result<()> {
        let actions = normalize_order_actions(actions)?;
        let request = OrderSubscribeRequest::unsubscribe(actions.clone());
        self.socket
            .send_text(serde_json::to_string(&request)?)
            .await?;
        self.active_actions
            .retain(|action| !actions.contains(action));

        Ok(())
    }

    /// Send the documented literal ping frame.
    pub async fn ping(&mut self) -> Result<()> {
        self.socket.send_literal_ping().await
    }

    /// Current active subscription command to replay after reconnect.
    pub fn resubscribe_frames(&self) -> Result<Vec<String>> {
        if self.active_actions.is_empty() {
            return Ok(Vec::new());
        }

        Ok(vec![serde_json::to_string(
            &OrderSubscribeRequest::subscribe(self.active_actions.clone()),
        )?])
    }

    /// Receive the next typed order event.
    pub async fn next_event(&mut self) -> Result<Option<OrderSocketEvent>> {
        self.socket.next_event().await
    }

    /// Close the socket.
    pub async fn close(&mut self) -> Result<()> {
        self.socket.close().await
    }
}

fn normalize_order_actions(actions: Vec<String>) -> Result<Vec<String>> {
    actions
        .into_iter()
        .flat_map(|action| {
            action
                .split(',')
                .map(str::trim)
                .filter(|action| !action.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .map(|action| match action.as_str() {
            "orders" | "OnOrders" => Ok("orders".to_owned()),
            "trades" | "OnTrades" => Ok("trades".to_owned()),
            "positions" | "OnPositions" => Ok("positions".to_owned()),
            "edis" | "OnEdis" | "OnEDIS" => Ok("edis".to_owned()),
            "pricealerts" | "OnPriceAlerts" => Ok("pricealerts".to_owned()),
            "login" | "OnGeneral" => Ok("login".to_owned()),
            _ => Err(FyersError::Validation(format!(
                "unsupported order WebSocket action: {action}"
            ))),
        })
        .collect()
}

fn parse_order_message(message: Message) -> Result<Option<OrderSocketEvent>> {
    match message {
        Message::Text(text) => parse_order_event(text.as_str())
            .map(Some)
            .map_err(FyersError::Validation),
        Message::Binary(_) => Err(FyersError::Validation(
            "order socket received unexpected binary frame".to_owned(),
        )),
        _ => Ok(None),
    }
}
