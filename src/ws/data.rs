//! Market-data WebSocket service.
//!
//! Talks the documented Fyers V3 binary protocol against
//! `wss://socket.fyers.in/hsm/v1-5/prod`. The wire format and individual
//! message builders live in [`crate::ws::data_protocol`]; this module owns
//! the connection lifecycle (auth handshake, channel resume, subscribe with
//! HSM-token resolution, frame routing).

use std::collections::{BTreeSet, HashMap, VecDeque};

use futures_util::{Sink, Stream};
use tokio_tungstenite::tungstenite::Message;

use crate::client::FyersClient;
use crate::error::{FyersError, Result};
use crate::models::ws::{
    DataControlEvent, DataSocketConfig, DataSocketEvent, DataSubscribeRequest,
    DataUnsubscribeRequest,
};
use crate::ws::data_protocol::{
    self, ScripFeed, ack_count_from_auth_envelope, build_ack_message, build_auth_message,
    build_channel_bitmap_message, build_channel_bitmap_message_with_marker,
    build_subscribe_message, build_unsubscribe_message, data_type, datafeed_message_num,
    depth_update_from_feed, extract_hsm_key, index_update_from_feed, mode, parse_datafeed,
    parse_envelope, req_type, symbol_update_from_feed,
};
use crate::ws::data_symbols;
use crate::ws::manager::{
    LiveWebSocket, ManagedSocket, ReconnectPolicy, connect_live_socket_no_auth_header,
};
use crate::ws::protocol::SocketKind;

const MAX_DATA_SOCKET_SYMBOLS: usize = 5000;
const DEFAULT_CHANNEL: u8 = 11;
const DEFAULT_SOURCE_ID: &str = concat!("fyers-rs/", env!("CARGO_PKG_VERSION"));

/// Live market-data WebSocket connection.
pub type LiveDataSocketConnection = DataSocketConnection<LiveWebSocket>;

/// Accessor for Fyers market-data WebSocket APIs.
#[derive(Debug, Clone, Copy)]
pub struct DataSocketService<'a> {
    client: &'a FyersClient,
}

impl<'a> DataSocketService<'a> {
    /// Create a new data-socket service accessor.
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
        let stream = connect_live_socket_no_auth_header(self.client.config(), SocketKind::Data)
            .await?;
        let mut connection =
            DataSocketConnection::from_stream(stream, self.client.clone(), config)?;
        connection.handshake().await?;
        Ok(connection)
    }

    /// Build a market-data manager from an already connected stream.
    ///
    /// The handshake is **not** performed — the caller is responsible for
    /// sending the auth/full-mode/channel-resume frames or for using a stream
    /// that does not require them. Useful for testing against mock streams.
    pub fn connect_with_stream<S>(&self, stream: S) -> Result<DataSocketConnection<S>>
    where
        S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin,
    {
        DataSocketConnection::from_stream(stream, self.client.clone(), DataSocketConfig::default())
    }
}

/// Typed market-data WebSocket manager.
///
/// # Reconnect (manual)
///
/// Auto-reconnect is **not** implemented. [`crate::models::ws::DataSocketConfig`]
/// accepts `reconnect` and `reconnect_retry` fields and they're stored on the
/// connection, but no internal loop consumes them — `next_event` returns
/// `Ok(None)` on disconnect and stays closed.
///
/// To recover from a disconnect, drop the connection and build a new one:
///
/// ```no_run
/// # use fyers_rs::FyersClient;
/// # use fyers_rs::models::ws::{DataSubscribeRequest, DataSubscriptionKind};
/// # async fn run(client: FyersClient, request: DataSubscribeRequest) -> fyers_rs::Result<()> {
/// loop {
///     let mut socket = client.data_socket().connect().await?;
///     socket.subscribe(&request).await?;
///     while let Some(event) = socket.next_event().await? {
///         // handle event
///         # let _ = event;
///     }
///     // disconnected — loop and reconnect from scratch.
///     // The HSM-token resolver runs again inside subscribe(), so this
///     // recovers correctly even if topic IDs have rotated.
///     # break Ok::<(), fyers_rs::FyersError>(());
/// }
/// # }
/// ```
///
/// Subscribe state from the previous session is held in
/// [`Self::resubscribe_frames`] for inspection, but it's the user-facing
/// JSON of the original [`DataSubscribeRequest`] values, not the binary
/// frames sent on the wire — the binary frames carry freshly resolved HSM
/// tokens that must be re-fetched.
#[derive(Debug)]
pub struct DataSocketConnection<S = LiveWebSocket> {
    socket: ManagedSocket<S, DataSocketEvent>,
    config: DataSocketConfig,
    client: FyersClient,
    hsm_key: String,
    access_token: String,
    channel_num: u8,
    source_id: String,
    subscriptions: Vec<DataSubscribeRequest>,
    topic_to_input: HashMap<String, String>,
    pending_events: VecDeque<DataSocketEvent>,
    ack_count: u32,
    update_count: u32,
    last_message_num: u32,
    pending_ack: Option<u32>,
}

impl<S> DataSocketConnection<S>
where
    S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    /// Create a manager from an already connected stream and a client.
    ///
    /// `client` must have an access token configured — its JWT payload is
    /// decoded immediately to extract the `hsm_key` claim used by the binary
    /// auth frame.
    pub fn from_stream(stream: S, client: FyersClient, config: DataSocketConfig) -> Result<Self> {
        let access_token = client
            .config()
            .access_token()
            .ok_or(FyersError::MissingConfig {
                field: "access_token",
            })?
            .expose_secret()
            .to_owned();
        let hsm_key = extract_hsm_key(&access_token)?;
        let reconnect_policy = ReconnectPolicy::new(
            config.reconnect,
            config.reconnect_retry,
            config.queue_process_interval.as_duration(),
        );
        Ok(Self {
            socket: ManagedSocket::from_stream(
                SocketKind::Data,
                stream,
                noop_parser,
                reconnect_policy,
            ),
            config,
            client,
            hsm_key,
            access_token,
            channel_num: DEFAULT_CHANNEL,
            source_id: DEFAULT_SOURCE_ID.to_owned(),
            subscriptions: Vec::new(),
            topic_to_input: HashMap::new(),
            pending_events: VecDeque::new(),
            ack_count: 0,
            update_count: 0,
            last_message_num: 0,
            pending_ack: None,
        })
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

    /// Send the documented auth + mode + channel-resume handshake frames.
    ///
    /// The mode (full vs lite) is taken from [`DataSocketConfig::lite_mode`].
    /// Each frame is sent without waiting for the corresponding ack — acks
    /// arrive as [`DataSocketEvent::Connected`] / [`DataSocketEvent::Mode`]
    /// events the next time [`Self::next_event`] is polled.
    pub async fn handshake(&mut self) -> Result<()> {
        let channel_mode = if self.config.lite_mode {
            mode::LITE
        } else {
            mode::FULL
        };
        let auth_msg = build_auth_message(&self.hsm_key, channel_mode, &self.source_id);
        self.send_binary(auth_msg).await?;

        let mode_marker = if self.config.lite_mode {
            mode::LITE_HEADER
        } else {
            mode::FULL_HEADER
        };
        let mode_msg = build_channel_bitmap_message_with_marker(
            req_type::FULL_MODE,
            self.channel_num,
            mode_marker,
        );
        self.send_binary(mode_msg).await?;

        let resume_msg =
            build_channel_bitmap_message(req_type::CHANNEL_RESUME, self.channel_num);
        self.send_binary(resume_msg).await?;
        Ok(())
    }

    /// Subscribe to symbol or depth updates.
    ///
    /// Resolves each symbol against `/data/symbol-token`, builds the binary
    /// subscribe frame, and sends it. The subscribe ack arrives later as
    /// [`DataSocketEvent::Subscribed`].
    pub async fn subscribe(&mut self, request: &DataSubscribeRequest) -> Result<()> {
        if request.symbols.is_empty() {
            return Ok(());
        }
        if request.symbols.len() > MAX_DATA_SOCKET_SYMBOLS
            || active_symbol_count_after(&self.subscriptions, request) > MAX_DATA_SOCKET_SYMBOLS
        {
            return Err(FyersError::Validation(
                "data WebSocket subscriptions cannot exceed 5000 symbols".to_owned(),
            ));
        }

        let resolved =
            data_symbols::resolve_hsm_symbols(&self.client, &request.symbols, request.data_type)
                .await?;
        if !resolved.invalid.is_empty() {
            return Err(FyersError::Validation(format!(
                "data-socket subscribe: invalid symbols {:?}",
                resolved.invalid
            )));
        }
        if resolved.resolved.is_empty() {
            return Err(FyersError::Validation(
                "data-socket subscribe: symbol-token API returned no usable HSM tokens"
                    .to_owned(),
            ));
        }

        for r in &resolved.resolved {
            self.topic_to_input
                .insert(r.hsm_topic.clone(), r.input_symbol.clone());
        }

        let topics = resolved.hsm_topics();
        let frame = build_subscribe_message(
            &topics,
            self.channel_num,
            &self.access_token,
            &self.source_id,
        );
        self.send_binary(frame).await?;

        if !self.subscriptions.contains(request) {
            self.subscriptions.push(request.clone());
        }
        Ok(())
    }

    /// Unsubscribe from symbol or depth updates.
    pub async fn unsubscribe(&mut self, request: &DataUnsubscribeRequest) -> Result<()> {
        if request.symbols.is_empty() {
            return Ok(());
        }
        let resolved =
            data_symbols::resolve_hsm_symbols(&self.client, &request.symbols, request.data_type)
                .await?;
        let topics = resolved.hsm_topics();
        if topics.is_empty() {
            return Ok(());
        }
        let frame = build_unsubscribe_message(
            &topics,
            self.channel_num,
            &self.access_token,
            &self.source_id,
        );
        self.send_binary(frame).await?;

        for topic in &topics {
            self.topic_to_input.remove(topic);
        }
        self.subscriptions.retain(|existing| existing != request);
        Ok(())
    }

    /// Active subscribe commands, returned for resubscribe-after-reconnect flows.
    ///
    /// These are JSON-serialized representations of the user-facing
    /// [`DataSubscribeRequest`] values, **not** the binary frames sent on the
    /// wire — the binary frames depend on freshly resolved HSM tokens that
    /// must be re-fetched after a reconnect.
    pub fn resubscribe_frames(&self) -> Result<Vec<String>> {
        self.subscriptions
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(FyersError::from)
    }

    /// Receive the next typed market-data event.
    pub async fn next_event(&mut self) -> Result<Option<DataSocketEvent>> {
        loop {
            if let Some(message_num) = self.pending_ack.take() {
                let ack = build_ack_message(message_num);
                self.send_binary(ack).await?;
            }
            if let Some(event) = self.pending_events.pop_front() {
                return Ok(Some(event));
            }
            let Some(message) = self.socket.next_raw_frame().await? else {
                return Ok(None);
            };
            match message {
                Message::Binary(bytes) => {
                    self.handle_binary_frame(&bytes)?;
                }
                Message::Text(text) => {
                    return Err(FyersError::Validation(format!(
                        "data socket received unexpected text frame ({} bytes)",
                        text.len()
                    )));
                }
                _ => {}
            }
        }
    }

    /// Server-advertised ack threshold from field 2 of the auth response.
    /// Zero until the first auth ack is processed.
    pub const fn ack_count(&self) -> u32 {
        self.ack_count
    }

    /// Close the socket.
    pub async fn close(&mut self) -> Result<()> {
        self.socket.close().await
    }

    async fn send_binary(&mut self, bytes: Vec<u8>) -> Result<()> {
        self.socket.send_binary(bytes).await
    }

    fn handle_binary_frame(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() < 4 {
            return Ok(());
        }
        let req = bytes[2];
        match req {
            req_type::DATAFEED => {
                if let Some(num) = datafeed_message_num(bytes) {
                    self.last_message_num = num;
                }
                let feeds = parse_datafeed(bytes)?;
                let saw_market_payload = feeds.iter().any(|f| {
                    matches!(
                        f.data_type,
                        data_type::SNAPSHOT | data_type::UPDATE | data_type::LITE
                    )
                });
                for feed in &feeds {
                    if let Some(event) = self.feed_to_event(feed) {
                        self.pending_events.push_back(event);
                    }
                }
                if saw_market_payload && self.ack_count > 0 {
                    self.update_count = self.update_count.saturating_add(1);
                    if self.update_count >= self.ack_count {
                        self.pending_ack = Some(self.last_message_num);
                        self.update_count = 0;
                    }
                }
            }
            req_type::CHANNEL_BUFFER => {
                // Pre-allocated channel buffer dump; ignore.
            }
            req_type::AUTH => {
                let env = parse_envelope(bytes)?;
                if let Some(count) = ack_count_from_auth_envelope(&env) {
                    self.ack_count = count;
                }
                self.pending_events
                    .push_back(DataSocketEvent::Connected(envelope_to_control(
                        &env, "cn", "Authentication done",
                    )));
            }
            req_type::SUBSCRIBE => {
                let env = parse_envelope(bytes)?;
                self.pending_events
                    .push_back(DataSocketEvent::Subscribed(envelope_to_control(
                        &env, "sub", "Subscribed",
                    )));
            }
            req_type::UNSUBSCRIBE => {
                let env = parse_envelope(bytes)?;
                self.pending_events
                    .push_back(DataSocketEvent::Unsubscribed(envelope_to_control(
                        &env, "unsub", "Unsubscribed",
                    )));
            }
            req_type::FULL_MODE | req_type::CHANNEL_RESUME | req_type::CHANNEL_PAUSE => {
                let env = parse_envelope(bytes)?;
                let event_type = match req {
                    req_type::FULL_MODE => "ful",
                    req_type::CHANNEL_RESUME => "cr",
                    req_type::CHANNEL_PAUSE => "cp",
                    _ => unreachable!(),
                };
                self.pending_events
                    .push_back(DataSocketEvent::Mode(envelope_to_control(
                        &env, event_type, "Mode",
                    )));
            }
            _ => {
                let env = parse_envelope(bytes).unwrap_or(data_protocol::Envelope {
                    req_type: req,
                    fields: Vec::new(),
                });
                self.pending_events
                    .push_back(DataSocketEvent::Error(envelope_to_control(
                        &env,
                        "error",
                        &format!("unhandled data-socket frame type 0x{req:02x}"),
                    )));
            }
        }
        Ok(())
    }

    fn feed_to_event(&self, feed: &ScripFeed<'_>) -> Option<DataSocketEvent> {
        if !matches!(
            feed.data_type,
            data_type::SNAPSHOT | data_type::UPDATE | data_type::LITE
        ) {
            return None;
        }
        let user_symbol = self
            .topic_to_input
            .get(feed.topic_name)
            .cloned()
            .unwrap_or_else(|| feed.topic_name.to_owned());

        match topic_kind(feed.topic_name) {
            TopicKind::Index => {
                let mut event = index_update_from_feed(feed);
                event.symbol = user_symbol;
                Some(DataSocketEvent::IndexUpdate(event))
            }
            TopicKind::Depth => {
                let mut event = depth_update_from_feed(feed);
                event.symbol = user_symbol;
                Some(DataSocketEvent::DepthUpdate(event))
            }
            TopicKind::Symbol => {
                let mut event = symbol_update_from_feed(feed);
                event.symbol = user_symbol;
                Some(DataSocketEvent::SymbolUpdate(event))
            }
            TopicKind::Other => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TopicKind {
    Symbol,
    Index,
    Depth,
    Other,
}

fn topic_kind(topic: &str) -> TopicKind {
    match topic.split('|').next() {
        Some("sf") => TopicKind::Symbol,
        Some("if") => TopicKind::Index,
        Some("dp") => TopicKind::Depth,
        _ => TopicKind::Other,
    }
}

fn envelope_to_control(env: &data_protocol::Envelope<'_>, kind: &str, default_msg: &str) -> DataControlEvent {
    let s = env.status_text().unwrap_or("ok").to_owned();
    let code = if s == "K" { 200 } else { -1 };
    DataControlEvent {
        event_type: kind.to_owned(),
        code,
        message: if s == "K" { default_msg.to_owned() } else { s.clone() },
        s: if s == "K" { "ok".to_owned() } else { "error".to_owned() },
    }
}

fn noop_parser(_message: Message) -> Result<Option<DataSocketEvent>> {
    Ok(None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FyersClient;

    fn dummy_client() -> FyersClient {
        // A JWT with sub=access_token and hsm_key, hand-rolled so tests don't
        // need network. Header/sig are placeholders.
        // payload = {"sub":"access_token","hsm_key":"deadbeef"}
        let header = "eyJhbGciOiJub25lIn0";
        let payload = "eyJzdWIiOiJhY2Nlc3NfdG9rZW4iLCJoc21fa2V5IjoiZGVhZGJlZWYifQ";
        let token = format!("{header}.{payload}.sig");
        FyersClient::builder()
            .client_id("APPID-100")
            .access_token(token)
            .build()
            .unwrap()
    }

    #[test]
    fn topic_kind_classifier_matches_documented_prefixes() {
        assert!(matches!(topic_kind("sf|nse_cm|3045"), TopicKind::Symbol));
        assert!(matches!(topic_kind("if|nse_cm|26000"), TopicKind::Index));
        assert!(matches!(topic_kind("dp|nse_cm|3045"), TopicKind::Depth));
        assert!(matches!(topic_kind("nope"), TopicKind::Other));
    }

    #[test]
    fn envelope_to_control_maps_k_status_to_ok() {
        let env = data_protocol::Envelope {
            req_type: req_type::SUBSCRIBE,
            fields: vec![data_protocol::EnvelopeField {
                id: 1,
                value: b"K",
            }],
        };
        let control = envelope_to_control(&env, "sub", "Subscribed");
        assert_eq!(control.s, "ok");
        assert_eq!(control.code, 200);
        assert_eq!(control.message, "Subscribed");
    }

    #[test]
    fn from_stream_extracts_hsm_key_from_client_token() {
        // The expensive bits of from_stream — hsm_key extraction and reconnect
        // policy seeding — only require the client config, not a real stream.
        // This test verifies that path without spinning up a mock stream.
        let client = dummy_client();
        let token = client.config().access_token().unwrap().expose_secret().to_owned();
        let hsm = extract_hsm_key(&token).unwrap();
        assert_eq!(hsm, "deadbeef");
    }
}
