mod support;

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::{Sink, Stream};
use fyers_rs::models::ws::{
    DataSocketEvent, DataSubscribeRequest, OrderSocketEvent, TbtEvent, TbtSwitchChannelRequest,
};
use fyers_rs::ws::ReconnectPolicy;
use fyers_rs::{FyersClient, FyersError};
use pretty_assertions::assert_eq;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

#[tokio::test]
async fn live_connect_requires_access_token_before_network_io() {
    let client = FyersClient::builder()
        .client_id("APPID-100")
        .build()
        .unwrap();
    let error = client.data_socket().connect().await.unwrap_err();

    assert!(matches!(
        error,
        FyersError::MissingConfig {
            field: "access_token"
        }
    ));
}

#[tokio::test]
async fn data_manager_decodes_captured_binary_snapshot() {
    let snapshot = include_bytes!(
        "../fixtures/ws/data/captured/snapshot_NSE_SBIN-EQ.bin"
    );
    let (stream, sent) = MockSocket::new(vec![Message::Binary(snapshot.to_vec().into())]);
    let client = test_client_with_jwt();
    let mut socket = client
        .data_socket()
        .connect_with_stream(stream)
        .expect("connect_with_stream");

    let event = socket.next_event().await.unwrap().unwrap();
    match event {
        DataSocketEvent::SymbolUpdate(update) => {
            assert_eq!(update.symbol, "sf|nse_cm|3045");
            assert!((update.ltp - 1068.45).abs() < f64::EPSILON);
            assert_eq!(update.prev_close_price, Some(1086.90));
        }
        other => panic!("unexpected data event: {other:?}"),
    }

    socket.close().await.unwrap();
    assert!(socket.socket().is_closed());
    assert!(
        sent_messages(&sent)
            .iter()
            .any(|message| matches!(message, Message::Close(None)))
    );
}

#[tokio::test]
async fn data_manager_pongs_pings_between_binary_frames() {
    let snapshot = include_bytes!(
        "../fixtures/ws/data/captured/snapshot_NSE_SBIN-EQ.bin"
    );
    let (stream, sent) = MockSocket::new(vec![
        Message::Ping(vec![1, 2, 3].into()),
        Message::Binary(snapshot.to_vec().into()),
    ]);
    let client = test_client_with_jwt();
    let mut socket = client
        .data_socket()
        .connect_with_stream(stream)
        .expect("connect_with_stream");

    let event = socket.next_event().await.unwrap().unwrap();
    assert!(matches!(event, DataSocketEvent::SymbolUpdate(_)));
    assert!(
        sent_messages(&sent).iter().any(
            |message| matches!(message, Message::Pong(payload) if payload.as_ref() == [1, 2, 3])
        )
    );
}

#[tokio::test]
async fn data_manager_rejects_subscribe_request_over_5000_symbols() {
    let (stream, _) = MockSocket::new(Vec::new());
    let client = test_client_with_jwt();
    let mut socket = client
        .data_socket()
        .connect_with_stream(stream)
        .expect("connect_with_stream");
    let too_many = DataSubscribeRequest {
        symbols: (0..5001)
            .map(|index| format!("NSE:SYM{index}-EQ"))
            .collect(),
        data_type: fyers_rs::models::ws::DataSubscriptionKind::SymbolUpdate,
    };

    assert!(matches!(
        socket.subscribe(&too_many).await,
        Err(FyersError::Validation(_))
    ));
}

#[tokio::test]
async fn order_manager_sends_commands_tracks_replay_and_yields_events() {
    let (stream, sent) = MockSocket::new(vec![Message::Text(
        support::read_fixture("ws/order/event_order.json").into(),
    )]);
    let client = test_client();
    let mut socket = client.order_socket().connect_with_stream(stream);
    let actions = vec!["orders".to_owned(), "trades".to_owned()];

    socket.subscribe(actions.clone()).await.unwrap();
    socket.ping().await.unwrap();

    let replay = socket.resubscribe_frames().unwrap();
    assert_eq!(replay.len(), 1);
    assert_eq!(
        sent_texts(&sent),
        vec![
            r#"{"T":"SUB_ORD","SLIST":["orders","trades"],"SUB_T":1}"#.to_owned(),
            "ping".to_owned()
        ]
    );

    let event = socket.next_event().await.unwrap().unwrap();
    assert!(matches!(event, OrderSocketEvent::Order(_)));

    socket.unsubscribe(vec!["orders".to_owned()]).await.unwrap();
    assert_eq!(
        socket.resubscribe_frames().unwrap(),
        vec![r#"{"T":"SUB_ORD","SLIST":["trades"],"SUB_T":1}"#.to_owned()]
    );
}

#[tokio::test]
async fn order_manager_accepts_documented_sdk_action_aliases() {
    let (stream, sent) = MockSocket::new(Vec::new());
    let client = test_client();
    let mut socket = client.order_socket().connect_with_stream(stream);

    socket
        .subscribe(vec![
            "OnOrders,OnTrades".to_owned(),
            "OnPositions".to_owned(),
            "OnGeneral".to_owned(),
        ])
        .await
        .unwrap();

    assert_eq!(
        sent_texts(&sent),
        vec![
            r#"{"T":"SUB_ORD","SLIST":["orders","trades","positions","login"],"SUB_T":1}"#
                .to_owned()
        ]
    );
}

#[tokio::test]
async fn order_manager_rejects_undocumented_actions() {
    let (stream, _) = MockSocket::new(Vec::new());
    let client = test_client();
    let mut socket = client.order_socket().connect_with_stream(stream);

    assert!(matches!(
        socket.subscribe(vec!["unknown".to_owned()]).await,
        Err(FyersError::Validation(_))
    ));
}

#[tokio::test]
async fn tbt_manager_sends_commands_and_decodes_binary_events() {
    let (stream, sent) = MockSocket::new(vec![Message::Binary(
        hex_fixture("ws/tbt/depth_snapshot.hex").into(),
    )]);
    let client = test_client();
    let mut socket = client.tbt_socket().connect_with_stream(stream);
    let switch: TbtSwitchChannelRequest = support::json_fixture("ws/tbt/switch_channel.json");

    socket
        .subscribe_depth(vec!["NSE:SBIN-EQ".to_owned()], "1")
        .await
        .unwrap();
    socket.switch_channel(&switch).await.unwrap();
    socket.ping().await.unwrap();

    assert_eq!(
        sent_texts(&sent),
        vec![
            r#"{"type":1,"data":{"subs":1,"symbols":["NSE:SBIN-EQ"],"mode":"depth","channel":"1"}}"#
                .to_owned(),
            serde_json::to_string(&switch).unwrap(),
            "ping".to_owned()
        ]
    );
    assert_eq!(socket.resubscribe_frames().unwrap().len(), 1);

    let event = socket.next_event().await.unwrap().unwrap();
    match event {
        TbtEvent::SocketMessage(message) => {
            assert!(message.snapshot);
            assert!(message.feeds.contains_key("NSE:SBIN-EQ"));
        }
        other => panic!("unexpected TBT event: {other:?}"),
    }
}

#[tokio::test]
async fn tbt_manager_enforces_documented_depth_limits() {
    let (stream, _) = MockSocket::new(Vec::new());
    let client = test_client();
    let mut socket = client.tbt_socket().connect_with_stream(stream);

    let too_many_symbols = (0..6)
        .map(|index| format!("NSE:SBIN{index}-EQ"))
        .collect::<Vec<_>>();
    assert!(matches!(
        socket.subscribe_depth(too_many_symbols, "1").await,
        Err(FyersError::Validation(_))
    ));
    assert!(matches!(
        socket
            .subscribe_depth(vec!["BSE:SBIN-EQ".to_owned()], "1")
            .await,
        Err(FyersError::Validation(_))
    ));
    assert!(matches!(
        socket
            .subscribe_depth(vec!["NSE:SBIN-EQ".to_owned()], "51")
            .await,
        Err(FyersError::Validation(_))
    ));
}

#[tokio::test]
async fn malformed_frames_return_errors_without_panicking() {
    let (stream, _) = MockSocket::new(vec![Message::Text(r#"{"type":"unknown"}"#.into())]);
    let client = test_client_with_jwt();
    let mut socket = client
        .data_socket()
        .connect_with_stream(stream)
        .expect("connect_with_stream");

    assert!(matches!(
        socket.next_event().await,
        Err(FyersError::Validation(_))
    ));
}

#[test]
fn reconnect_policy_is_bounded() {
    let policy = ReconnectPolicy::new(true, 2, Duration::from_millis(10));

    assert!(policy.should_retry(0));
    assert!(policy.should_retry(1));
    assert!(!policy.should_retry(2));
    assert!(!ReconnectPolicy::disabled().should_retry(0));
}

fn test_client() -> FyersClient {
    FyersClient::builder()
        .client_id("APPID-100")
        .access_token("ACCESS_TOKEN")
        .build()
        .unwrap()
}

/// Test client with a synthetic JWT carrying a `hsm_key` claim. Required by
/// the data-socket binary protocol because [`DataSocketConnection::from_stream`]
/// extracts `hsm_key` from the access token at construction.
fn test_client_with_jwt() -> FyersClient {
    // header={"alg":"none"}, payload={"sub":"access_token","hsm_key":"deadbeef"}
    let token = "eyJhbGciOiJub25lIn0.eyJzdWIiOiJhY2Nlc3NfdG9rZW4iLCJoc21fa2V5IjoiZGVhZGJlZWYifQ.sig";
    FyersClient::builder()
        .client_id("APPID-100")
        .access_token(token)
        .build()
        .unwrap()
}

fn sent_messages(sent: &Arc<Mutex<Vec<Message>>>) -> Vec<Message> {
    sent.lock()
        .expect("sent messages lock should not be poisoned")
        .clone()
}

fn sent_texts(sent: &Arc<Mutex<Vec<Message>>>) -> Vec<String> {
    sent_messages(sent)
        .into_iter()
        .filter_map(|message| match message {
            Message::Text(text) => Some(text.to_string()),
            _ => None,
        })
        .collect()
}

fn hex_fixture(path: &str) -> Vec<u8> {
    let hex = support::read_fixture(path);
    let hex = hex.trim();

    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).expect("valid hex byte"))
        .collect()
}

#[derive(Debug)]
struct MockSocket {
    incoming: VecDeque<std::result::Result<Message, WsError>>,
    sent: Arc<Mutex<Vec<Message>>>,
}

impl MockSocket {
    fn new(incoming: Vec<Message>) -> (Self, Arc<Mutex<Vec<Message>>>) {
        let sent = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                incoming: incoming.into_iter().map(Ok).collect(),
                sent: Arc::clone(&sent),
            },
            sent,
        )
    }
}

impl Stream for MockSocket {
    type Item = std::result::Result<Message, WsError>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.incoming.pop_front())
    }
}

impl Sink<Message> for MockSocket {
    type Error = WsError;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.sent
            .lock()
            .expect("sent messages lock should not be poisoned")
            .push(item);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
