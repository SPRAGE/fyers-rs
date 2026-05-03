mod support;

use std::time::Duration;

use fyers_rs::models::ws::{
    DataSocketConfig, DataSocketEvent, DataSubscribeRequest, DataSubscriptionKind,
    DataUnsubscribeRequest, MessageType, OrderSocketConfig, OrderSocketEvent,
    OrderSubscribeRequest, QueueProcessInterval, TbtEvent, TbtSocketConfig, TbtSubscribeRequest,
    TbtSwitchChannelRequest, parse_data_event, parse_order_event, parse_tbt_event,
};
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn data_socket_connection_config_matches_documented_defaults() {
    let fixture = support::json_value("ws/data/connect_ok.json");
    let config = DataSocketConfig::default();

    assert_eq!(
        fixture["socket_url"].as_str(),
        Some("wss://socket.fyers.in/hsm/v1-5/prod")
    );
    assert_eq!(
        fixture["authorization_format"].as_str(),
        Some("appid:access_token")
    );
    assert!(!config.lite_mode);
    assert!(config.reconnect);
    assert_eq!(config.reconnect_retry, 50);
    assert_eq!(
        config.queue_process_interval.as_duration(),
        Duration::from_millis(1)
    );
}

#[test]
fn data_socket_subscribe_commands_serialize_to_documented_json() {
    for path in [
        "ws/data/subscribe_symbol_update.json",
        "ws/data/subscribe_index_update.json",
        "ws/data/subscribe_depth_update.json",
    ] {
        let command: DataSubscribeRequest = support::json_fixture(path);
        let fixture = support::json_value(path);

        assert_eq!(serde_json::to_value(command).unwrap(), fixture);
    }
}

#[test]
fn data_socket_unsubscribe_command_preserves_symbols_and_data_type() {
    let command: DataUnsubscribeRequest = support::json_fixture("ws/data/unsubscribe_ok.json");

    assert_eq!(command.symbols, vec!["NSE:SBIN-EQ"]);
    assert_eq!(command.data_type, DataSubscriptionKind::SymbolUpdate);
    assert_eq!(
        serde_json::to_value(command).unwrap(),
        support::json_value("ws/data/unsubscribe_ok.json")
    );
}

#[test]
fn data_socket_lite_mode_config_and_event_fixture_are_supported() {
    let config = DataSocketConfig {
        lite_mode: true,
        ..DataSocketConfig::default()
    };
    let event = parse_data_event(&support::read_fixture(
        "ws/data/lite_mode_symbol_update.json",
    ))
    .expect("lite-mode symbol event should parse");

    assert!(config.lite_mode);
    match event {
        DataSocketEvent::SymbolUpdate(update) => {
            assert_eq!(update.ltp, 500.55);
            assert_eq!(update.prev_close_price, None);
        }
        _ => panic!("unexpected lite-mode event variant"),
    }
}

#[test]
fn data_socket_queue_process_interval_enforces_documented_bounds() {
    assert_eq!(
        QueueProcessInterval::from_millis(1).unwrap().as_duration(),
        Duration::from_millis(1)
    );
    assert_eq!(
        QueueProcessInterval::from_millis(2000)
            .unwrap()
            .as_duration(),
        Duration::from_millis(2000)
    );
    assert!(QueueProcessInterval::from_millis(0).is_err());
    assert!(QueueProcessInterval::from_millis(2001).is_err());
    assert!(support::read_fixture("ws/data/advanced_config.md").contains("1ms to 2000ms"));
}

#[test]
fn data_socket_event_fixtures_route_to_typed_variants() {
    let cases = [
        (
            "ws/data/event_symbol_update.json",
            "NSE:SBIN-EQ",
            "SymbolUpdate",
        ),
        (
            "ws/data/event_index_update.json",
            "NSE:NIFTY50-INDEX",
            "IndexUpdate",
        ),
        (
            "ws/data/event_depth_update.json",
            "NSE:SBIN-EQ",
            "DepthUpdate",
        ),
    ];

    for (path, symbol, kind) in cases {
        let event =
            parse_data_event(&support::read_fixture(path)).expect("data event should parse");
        match (kind, event) {
            ("SymbolUpdate", DataSocketEvent::SymbolUpdate(update)) => {
                assert_eq!(update.symbol, symbol);
                assert!(update.ltp > 0.0);
                assert_eq!(update.bid_size, Some(2081));
            }
            ("IndexUpdate", DataSocketEvent::IndexUpdate(update)) => {
                assert_eq!(update.symbol, symbol);
                assert!(update.ltp > 0.0);
                assert_eq!(update.exch_feed_time, Some(1727428424));
            }
            ("DepthUpdate", DataSocketEvent::DepthUpdate(update)) => {
                assert_eq!(update.symbol, symbol);
                assert_eq!(update.bid_price1, 606.25);
                assert_eq!(update.ask_order5, 17);
            }
            _ => panic!("unexpected data event variant"),
        }
    }
}

#[test]
fn data_socket_control_events_parse_to_control_variants() {
    let controls = support::json_value("ws/data/event_control.json")
        .as_array()
        .expect("control fixture should be an array")
        .clone();

    let parsed = controls
        .iter()
        .map(|value| parse_data_event(&value.to_string()).expect("control event should parse"))
        .collect::<Vec<_>>();

    assert!(matches!(parsed[0], DataSocketEvent::Connected(_)));
    assert!(matches!(parsed[1], DataSocketEvent::Subscribed(_)));
    assert!(matches!(parsed[2], DataSocketEvent::Unsubscribed(_)));
    assert!(matches!(parsed[3], DataSocketEvent::Error(_)));
    assert!(matches!(parsed[4], DataSocketEvent::Mode(_)));
}

#[test]
fn order_socket_connection_config_and_ping_match_docs() {
    let fixture = support::json_value("ws/order/connect_ok.json");
    let config = OrderSocketConfig::default();
    let ping = support::read_fixture("ws/order/ping.txt");

    assert_eq!(
        fixture["socket_url"].as_str(),
        Some("wss://socket.fyers.in/trade/v3")
    );
    assert_eq!(
        fixture["authorization_format"].as_str(),
        Some("appid:access_token")
    );
    assert!(config.reconnect);
    assert_eq!(config.reconnect_retry, 50);
    assert_eq!(config.ping_interval, Duration::from_secs(10));
    assert_eq!(ping.trim_end(), "ping");
}

#[test]
fn order_socket_subscribe_and_unsubscribe_commands_match_docs() {
    let subscribe: OrderSubscribeRequest = support::json_fixture("ws/order/subscribe.json");
    let unsubscribe: OrderSubscribeRequest = support::json_fixture("ws/order/unsubscribe.json");
    let actions = vec![
        "orders".to_owned(),
        "trades".to_owned(),
        "positions".to_owned(),
        "edis".to_owned(),
        "pricealerts".to_owned(),
        "login".to_owned(),
    ];

    assert_eq!(subscribe, OrderSubscribeRequest::subscribe(actions.clone()));
    assert_eq!(unsubscribe, OrderSubscribeRequest::unsubscribe(actions));
    assert_eq!(
        serde_json::to_value(subscribe).unwrap(),
        support::json_value("ws/order/subscribe.json")
    );
    assert_eq!(
        serde_json::to_value(unsubscribe).unwrap(),
        support::json_value("ws/order/unsubscribe.json")
    );
}

#[test]
fn order_socket_domain_event_fixtures_route_to_typed_variants() {
    let cases = [
        ("ws/order/event_order.json", "Order"),
        ("ws/order/event_trade.json", "Trade"),
        ("ws/order/event_position.json", "Position"),
        ("ws/order/event_general.json", "General"),
        ("ws/order/event_edis.json", "Edis"),
        ("ws/order/event_price_alert.json", "PriceAlert"),
    ];

    for (path, kind) in cases {
        let event =
            parse_order_event(&support::read_fixture(path)).expect("order event should parse");
        match (kind, event) {
            ("Order", OrderSocketEvent::Order(update)) => {
                assert_eq!(update.s, "ok");
                assert_eq!(update.orders.client_id.as_deref(), Some("XV20986"));
                assert_eq!(update.orders.fy_token.as_deref(), Some("101000000014366"));
            }
            ("Trade", OrderSocketEvent::Trade(update)) => {
                assert_eq!(update.s, "ok");
                assert_eq!(
                    update.trades.trade_number.as_deref(),
                    Some("23080400089344-21726639")
                );
                assert_eq!(update.trades.trade_value, Some(7.95));
            }
            ("Position", OrderSocketEvent::Position(update)) => {
                assert_eq!(update.s, "ok");
                assert_eq!(update.positions.product_type.as_deref(), Some("INTRADAY"));
                assert_eq!(update.positions.realized_profit, Some(-0.04999999999999982));
            }
            ("General", OrderSocketEvent::General(update)) => assert_eq!(update.s, "ok"),
            ("Edis", OrderSocketEvent::Edis(update)) => assert_eq!(update.s, "ok"),
            ("PriceAlert", OrderSocketEvent::PriceAlert(update)) => assert_eq!(update.s, "ok"),
            _ => panic!("unexpected order event variant"),
        }
    }
}

#[test]
fn order_socket_control_events_parse_to_control_variants() {
    let controls = support::json_value("ws/order/event_control.json")
        .as_array()
        .expect("control fixture should be an array")
        .clone();

    let parsed = controls
        .iter()
        .map(|value| parse_order_event(&value.to_string()).expect("control event should parse"))
        .collect::<Vec<_>>();

    assert!(matches!(parsed[0], OrderSocketEvent::Subscribed(_)));
    assert!(matches!(parsed[1], OrderSocketEvent::Unsubscribed(_)));
    assert!(matches!(parsed[2], OrderSocketEvent::Error(_)));
    assert!(matches!(parsed[3], OrderSocketEvent::Closed(_)));
}

#[test]
fn tbt_socket_connection_config_and_ping_match_docs() {
    let fixture = support::json_value("ws/tbt/connect_ok.json");
    let config = TbtSocketConfig::default();
    let ping = support::read_fixture_bytes("ws/tbt/ping.txt");

    assert_eq!(
        fixture["socket_url"].as_str(),
        Some("wss://rtsocket-api.fyers.in/versova")
    );
    assert_eq!(
        fixture["authorization_format"].as_str(),
        Some("appid:access_token")
    );
    assert_eq!(fixture["max_connections_per_app_user"].as_i64(), Some(3));
    assert_eq!(
        fixture["max_symbols_per_depth_connection"].as_i64(),
        Some(5)
    );
    assert_eq!(fixture["max_channels"].as_i64(), Some(50));
    assert!(config.reconnect);
    assert_eq!(config.reconnect_retry, 50);
    assert!(!config.diff_only);
    assert_eq!(config.ping_interval, Duration::from_secs(30));
    assert_eq!(ping, b"ping\n");
}

#[test]
fn tbt_subscribe_and_unsubscribe_depth_commands_match_docs() {
    for path in [
        "ws/tbt/subscribe_depth.json",
        "ws/tbt/unsubscribe_depth.json",
        "ws/tbt/command_subscribe_depth.json",
    ] {
        let command: TbtSubscribeRequest = support::json_fixture(path);
        let fixture = support::json_value(path);

        assert_eq!(command.request_type, 1);
        assert_eq!(command.data.mode, "depth");
        assert_eq!(command.data.channel, "1");
        assert_eq!(serde_json::to_value(command).unwrap(), fixture);
    }
}

#[test]
fn tbt_switch_channel_command_serializes_camel_case_and_accepts_sdk_aliases() {
    let command: TbtSwitchChannelRequest = support::json_fixture("ws/tbt/switch_channel.json");
    let sdk_alias: TbtSwitchChannelRequest = serde_json::from_value(json!({
        "type": 2,
        "data": {
            "resume_channels": ["1"],
            "pause_channels": ["2"]
        }
    }))
    .unwrap();

    assert_eq!(command, sdk_alias);
    assert_eq!(
        serde_json::to_value(command).unwrap(),
        support::json_value("ws/tbt/switch_channel.json")
    );
}

#[test]
fn tbt_protobuf_socket_message_and_depth_snapshot_decode() {
    let message_bytes = hex_fixture("ws/tbt/socket_message.hex");
    let depth_bytes = hex_fixture("ws/tbt/depth_snapshot.hex");

    let message = parse_tbt_event(&message_bytes).expect("socket message should decode");
    let depth = parse_tbt_event(&depth_bytes).expect("depth snapshot should decode");

    match message {
        TbtEvent::SocketMessage(message) => {
            assert_eq!(message.message_type, MessageType::Depth as i32);
            assert_eq!(message.msg, "snapshot");
        }
        TbtEvent::Error { .. } => panic!("socket message fixture should not be an error"),
    }

    match depth {
        TbtEvent::SocketMessage(message) => {
            assert_eq!(message.message_type, MessageType::Depth as i32);
            assert!(message.snapshot);
            let feed = message
                .feeds
                .get("NSE:SBIN-EQ")
                .expect("feed should be keyed by ticker");
            let depth = feed
                .depth
                .as_ref()
                .expect("market feed should contain depth");

            assert_eq!(feed.ticker, "NSE:SBIN-EQ");
            assert_eq!(feed.token, "10100000003045");
            assert_eq!(depth.asks.len(), 1);
            assert_eq!(depth.bids.len(), 1);
            assert_eq!(
                depth.asks[0].price.as_ref().map(|price| price.value),
                Some(67545)
            );
            assert_eq!(depth.tbq.as_ref().map(|qty| qty.value), Some(1000));
        }
        TbtEvent::Error { .. } => panic!("depth fixture should not be an error"),
    }
}

#[test]
fn tbt_error_packet_decodes_to_error_variant() {
    let event = parse_tbt_event(&hex_fixture("ws/tbt/error.hex")).expect("error should decode");

    assert_eq!(
        event,
        TbtEvent::Error {
            msg: "bad symbol".to_owned()
        }
    );
}

#[test]
fn malformed_frames_return_errors() {
    assert!(parse_data_event("not json").is_err());
    assert!(parse_data_event(r#"{"type":"unknown"}"#).is_err());
    assert!(parse_order_event("not json").is_err());
    assert!(parse_tbt_event(b"not protobuf").is_err());
}

fn hex_fixture(path: &str) -> Vec<u8> {
    let hex = support::read_fixture(path);
    let hex = hex.trim();

    assert_eq!(hex.len() % 2, 0, "hex fixture should have an even length");

    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).expect("valid hex byte"))
        .collect()
}
