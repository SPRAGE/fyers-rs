mod support;

use fyers_rs::models::gtt::{
    CancelGttOrderRequest, GttActionResponse, GttOrderBookResponse, GttOrderRequest,
    ModifyGttOrderRequest,
};
use fyers_rs::models::orders::{
    AsyncMultiOrderActionResponse, AsyncOrderActionResponse, CancelOrderRequest,
    ModifyOrderRequest, MultiLegOrderRequest, MultiOrderActionResponse, OrderActionResponse,
    OrderBookResponse, PlaceOrderRequest,
};
use fyers_rs::models::reports::{OrderHistoryResponse, TradeHistoryResponse};
use fyers_rs::models::transactions::{PositionsResponse, TradeBookResponse};
use pretty_assertions::assert_eq;

#[test]
fn tradebook_success_fixture_matches_model() {
    let response: TradeBookResponse =
        support::json_fixture("rest/trades/tradebook/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.trade_book.len(), 3);

    let first_trade = &response.trade_book[0];
    assert_eq!(first_trade.client_id, "FXXXXX");
    assert_eq!(first_trade.order_date_time, "07-Aug-2020 13:51:12");
    assert_eq!(first_trade.order_number, "120080789075");
    assert_eq!(first_trade.exchange_order_no, "1200000009204725");
    assert_eq!(first_trade.exchange, 10);
    assert_eq!(first_trade.side, 1);
    assert_eq!(first_trade.order_type, 2);
    assert_eq!(first_trade.fy_token, "101000000010666");
    assert_eq!(first_trade.traded_qty, 10);
    assert_eq!(first_trade.trade_price, 32.7);
    assert_eq!(first_trade.trade_value, 327.0);
    assert_eq!(first_trade.trade_number, "52605023");
    assert_eq!(first_trade.row, 52605023);
    assert_eq!(first_trade.symbol, "NSE:PNB-EQ");
    assert_eq!(first_trade.order_tag, "1:Ordertag");
}

#[test]
fn orderbook_success_fixture_matches_model() {
    let response: OrderBookResponse =
        support::json_fixture("rest/orders/orderbook/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.order_book.len(), 1);

    let order = &response.order_book[0];
    assert_eq!(order.client_id, "X******");
    assert_eq!(order.id, "23030900015105");
    assert_eq!(
        order.id_fyers.as_deref(),
        Some("c6697c04-d9ab-4a7c-a6f4-b0cc4ca698f6")
    );
    assert_eq!(order.exch_ord_id, "1100000001089341");
    assert_eq!(order.qty, 1);
    assert_eq!(order.remaining_quantity, 0);
    assert_eq!(order.filled_qty, 1);
    assert_eq!(order.limit_price, 6.95);
    assert_eq!(order.order_type, 1);
    assert_eq!(order.fy_token, "101000000014366");
    assert_eq!(order.symbol, "NSE:IDEA-EQ");
    assert_eq!(order.offline_order, false);
    assert_eq!(order.order_validity, "DAY");
    assert_eq!(order.product_type, "CNC");
    assert_eq!(order.status, 2);
    assert_eq!(order.order_tag, "1:Ordertag");
}

#[test]
fn place_sync_request_fixture_matches_model() {
    let request: PlaceOrderRequest = support::json_fixture("rest/orders/place_sync/request.json");
    let actual = serde_json::to_value(&request).expect("request should serialize");
    let expected = support::json_value("rest/orders/place_sync/request.json");

    assert_eq!(actual, expected);
    assert_eq!(request.symbol, "NSE:IDEA-EQ");
    assert_eq!(request.qty, 1);
    assert_eq!(request.order_type, 1);
    assert_eq!(request.side, 1);
    assert_eq!(request.product_type, "CNC");
    assert_eq!(request.validity, "DAY");
    assert_eq!(request.order_tag.as_deref(), Some("1:Ordertag"));
    assert_eq!(request.is_slice_order, Some(false));
}

#[test]
fn place_sync_success_fixture_matches_model() {
    let response: OrderActionResponse =
        support::json_fixture("rest/orders/place_sync/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 1101);
    assert_eq!(
        response.message,
        "Order submitted successfully. Your Order Ref. No.23030900015105"
    );
    assert_eq!(response.id, "23030900015105");
}

#[test]
fn place_async_request_fixture_matches_model() {
    let request: PlaceOrderRequest = support::json_fixture("rest/orders/place_async/request.json");
    let actual = serde_json::to_value(&request).expect("request should serialize");
    let expected = support::json_value("rest/orders/place_async/request.json");

    assert_eq!(actual, expected);
    assert_eq!(request.symbol, "NSE:IDEA-EQ");
    assert_eq!(request.qty, 1);
    assert_eq!(request.order_type, 1);
    assert_eq!(request.order_tag.as_deref(), Some("1:Ordertag"));
}

#[test]
fn place_async_success_fixture_matches_model() {
    let response: AsyncOrderActionResponse =
        support::json_fixture("rest/orders/place_async/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 1101);
    assert_eq!(response.message, "Order queued successfully.");
    assert_eq!(response.id_fyers, "c6697c04-d9ab-4a7c-a6f4-b0cc4ca698f6");
}

#[test]
fn modify_order_request_fixtures_match_model() {
    for fixture in [
        "rest/orders/modify_sync/request.json",
        "rest/orders/modify_async/request.json",
    ] {
        let request: ModifyOrderRequest = support::json_fixture(fixture);
        let actual = serde_json::to_value(&request).expect("request should serialize");
        let expected = support::json_value(fixture);

        assert_eq!(actual, expected);
        assert_eq!(request.id, "23030900015105");
        assert_eq!(request.qty, Some(10));
        assert_eq!(request.order_type, 1);
    }
}

#[test]
fn cancel_order_request_fixtures_match_model() {
    for fixture in [
        "rest/orders/cancel_sync/request.json",
        "rest/orders/cancel_async/request.json",
    ] {
        let request: CancelOrderRequest = support::json_fixture(fixture);
        let actual = serde_json::to_value(&request).expect("request should serialize");
        let expected = support::json_value(fixture);

        assert_eq!(actual, expected);
        assert_eq!(request.id, "23030900015105");
    }
}

#[test]
fn single_order_action_response_fixtures_match_model() {
    for (fixture, code, message) in [
        (
            "rest/orders/modify_sync/response_success.json",
            1101,
            "Successfully modified order",
        ),
        (
            "rest/orders/cancel_sync/response_success.json",
            1103,
            "Successfully cancelled order",
        ),
        (
            "rest/orders/cancel_sync_by_path/response_success.json",
            1103,
            "Successfully cancelled order",
        ),
    ] {
        let response: OrderActionResponse = support::json_fixture(fixture);

        assert_eq!(response.s, "ok");
        assert_eq!(response.code, code);
        assert_eq!(response.message, message);
        assert_eq!(response.id, "23030900015105");
    }
}

#[test]
fn async_order_action_response_fixtures_match_model() {
    for (fixture, code, message) in [
        (
            "rest/orders/modify_async/response_success.json",
            1101,
            "Order modification queued successfully.",
        ),
        (
            "rest/orders/cancel_async/response_success.json",
            1103,
            "Order cancellation queued successfully.",
        ),
    ] {
        let response: AsyncOrderActionResponse = support::json_fixture(fixture);

        assert_eq!(response.s, "ok");
        assert_eq!(response.code, code);
        assert_eq!(response.message, message);
        assert_eq!(response.id_fyers, "c6697c04-d9ab-4a7c-a6f4-b0cc4ca698f6");
    }
}

#[test]
fn place_multi_sync_request_fixture_matches_model() {
    let requests: Vec<PlaceOrderRequest> =
        support::json_fixture("rest/orders/place_multi_sync/request.json");
    let actual = serde_json::to_value(&requests).expect("requests should serialize");
    let expected = support::json_value("rest/orders/place_multi_sync/request.json");

    assert_eq!(actual, expected);
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].symbol, "NSE:IDEA-EQ");
    assert_eq!(requests[0].order_tag.as_deref(), Some("1:Ordertag"));
    assert_eq!(requests[1].symbol, "NSE:ITC-EQ");
    assert_eq!(requests[1].order_tag.as_deref(), Some("2:Ordertag"));
}

#[test]
fn place_multi_sync_success_fixture_matches_model() {
    let response: MultiOrderActionResponse =
        support::json_fixture("rest/orders/place_multi_sync/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.message, "");
    assert_eq!(response.data.len(), 2);
    assert_eq!(response.data[0].status_code, 200);
    assert_eq!(response.data[0].body.id, "120080778988");
    assert_eq!(response.data[1].body.id, "120080777359");
}

#[test]
fn place_multi_async_success_fixture_matches_model() {
    let response: AsyncMultiOrderActionResponse =
        support::json_fixture("rest/orders/place_multi_async/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.message, "Multi order request queued");
    assert_eq!(response.data.len(), 2);
    assert_eq!(
        response.data[0].body.id_fyers,
        "c6697c04-d9ab-4a7c-a6f4-b0cc4ca698f6"
    );
}

#[test]
fn multi_modify_request_fixtures_match_model() {
    for fixture in [
        "rest/orders/modify_multi_sync/request.json",
        "rest/orders/modify_multi_async/request.json",
    ] {
        let requests: Vec<ModifyOrderRequest> = support::json_fixture(fixture);
        let actual = serde_json::to_value(&requests).expect("requests should serialize");
        let expected = support::json_value(fixture);

        assert_eq!(actual, expected);
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].id, "23030900015105");
        assert_eq!(requests[1].id, "23030900015106");
    }
}

#[test]
fn multi_cancel_request_fixtures_match_model() {
    for fixture in [
        "rest/orders/cancel_multi_sync/request.json",
        "rest/orders/cancel_multi_async/request.json",
    ] {
        let requests: Vec<CancelOrderRequest> = support::json_fixture(fixture);
        let actual = serde_json::to_value(&requests).expect("requests should serialize");
        let expected = support::json_value(fixture);

        assert_eq!(actual, expected);
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].id, "23030900015105");
        assert_eq!(requests[1].id, "23030900015106");
    }
}

#[test]
fn multi_order_action_response_fixtures_match_model() {
    for (fixture, expected_id) in [
        (
            "rest/orders/modify_multi_sync/response_success.json",
            "8102710298291",
        ),
        (
            "rest/orders/cancel_multi_sync/response_success.json",
            "808058117761",
        ),
    ] {
        let response: MultiOrderActionResponse = support::json_fixture(fixture);

        assert_eq!(response.s, "ok");
        assert_eq!(response.code, 200);
        assert_eq!(response.message, "");
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].status_description, "HTTP OK");
        assert_eq!(response.data[0].body.id, expected_id);
    }
}

#[test]
fn async_multi_order_action_response_fixtures_match_model() {
    for (fixture, message) in [
        (
            "rest/orders/modify_multi_async/response_success.json",
            "Multi order modification queued",
        ),
        (
            "rest/orders/cancel_multi_async/response_success.json",
            "Multi order cancellation queued",
        ),
    ] {
        let response: AsyncMultiOrderActionResponse = support::json_fixture(fixture);

        assert_eq!(response.s, "ok");
        assert_eq!(response.code, 200);
        assert_eq!(response.message, message);
        assert_eq!(response.data.len(), 2);
        assert_eq!(
            response.data[0].body.id_fyers,
            "c6697c04-d9ab-4a7c-a6f4-b0cc4ca698f6"
        );
    }
}

#[test]
fn place_multileg_sync_request_fixture_matches_model() {
    let request: MultiLegOrderRequest =
        support::json_fixture("rest/orders/place_multileg_sync/request.json");
    let actual = serde_json::to_value(&request).expect("request should serialize");
    let expected = support::json_value("rest/orders/place_multileg_sync/request.json");

    assert_eq!(actual, expected);
    assert_eq!(request.order_tag.as_deref(), Some("tag1"));
    assert_eq!(request.product_type, "MARGIN");
    assert_eq!(request.order_type, "3L");
    assert_eq!(request.validity, "IOC");
    assert_eq!(request.legs.leg1.symbol, "NSE:SBIN24JUNFUT");
    assert_eq!(
        request.legs.leg3.as_ref().expect("leg3").symbol,
        "NSE:SBIN24JUN900CE"
    );
}

#[test]
fn place_multileg_sync_success_fixture_matches_model() {
    let response: OrderActionResponse =
        support::json_fixture("rest/orders/place_multileg_sync/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 1101);
    assert_eq!(response.message, "Successfully placed order");
    assert_eq!(response.id, "23030900015107");
}

#[test]
fn gtt_order_request_fixtures_match_model() {
    for (fixture, expected_leg2) in [
        ("rest/gtt/place_single/request.json", false),
        ("rest/gtt/place_oco/request.json", true),
    ] {
        let request: GttOrderRequest = support::json_fixture(fixture);
        let actual = serde_json::to_value(&request).expect("request should serialize");
        let expected = support::json_value(fixture);

        assert_eq!(actual, expected);
        assert_eq!(request.side, 1);
        assert_eq!(request.symbol, "NSE:SBIN-EQ");
        assert_eq!(request.product_type, "CNC");
        assert_eq!(
            request.order_info.leg1.qty,
            if expected_leg2 { 3 } else { 1 }
        );
        assert_eq!(request.order_info.leg2.is_some(), expected_leg2);
    }
}

#[test]
fn modify_gtt_order_request_fixture_matches_model() {
    let request: ModifyGttOrderRequest = support::json_fixture("rest/gtt/modify/request.json");
    let actual = serde_json::to_value(&request).expect("request should serialize");
    let expected = support::json_value("rest/gtt/modify/request.json");

    assert_eq!(actual, expected);
    assert_eq!(request.id, "26021800001427");
    assert_eq!(request.order_info.leg1.price, 1750.0);
    assert!(request.order_info.leg2.is_some());
}

#[test]
fn cancel_gtt_order_request_fixture_matches_model() {
    let request: CancelGttOrderRequest = support::json_fixture("rest/gtt/cancel/request.json");
    let actual = serde_json::to_value(&request).expect("request should serialize");
    let expected = support::json_value("rest/gtt/cancel/request.json");

    assert_eq!(actual, expected);
    assert_eq!(request.id, "25111300002007");
}

#[test]
fn gtt_action_response_fixtures_match_model() {
    for (fixture, code, id) in [
        (
            "rest/gtt/place_single/response_success.json",
            1101,
            "25111300002007",
        ),
        (
            "rest/gtt/place_oco/response_success.json",
            1101,
            "25111300002008",
        ),
        (
            "rest/gtt/modify/response_success.json",
            1102,
            "26021800001427",
        ),
        (
            "rest/gtt/cancel/response_success.json",
            1103,
            "25111300002007",
        ),
    ] {
        let response: GttActionResponse = support::json_fixture(fixture);

        assert_eq!(response.s, "ok");
        assert_eq!(response.code, code);
        assert_eq!(response.id.as_deref(), Some(id));
    }
}

#[test]
fn gtt_orderbook_success_fixture_matches_model() {
    let response: GttOrderBookResponse =
        support::json_fixture("rest/gtt/orderbook/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.order_book.len(), 1);

    let order = &response.order_book[0];
    assert_eq!(order.client_id, "X******");
    assert_eq!(order.id, "25111300002007");
    assert_eq!(order.symbol, "NSE:SBIN-EQ");
    assert_eq!(order.product_type, "CNC");
    assert_eq!(order.gtt_oco_ind, 0);
}

#[test]
fn positions_success_fixture_matches_model() {
    let response: PositionsResponse =
        support::json_fixture("rest/positions/list/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.net_positions.len(), 1);

    let position = &response.net_positions[0];
    assert_eq!(position.net_qty, 1);
    assert_eq!(position.qty, 1);
    assert_eq!(position.avg_price, 72256.0);
    assert_eq!(position.net_avg, 71856.0);
    assert_eq!(position.product_type, "MARGIN");
    assert_eq!(position.realized_profit, 400.0);
    assert_eq!(position.unrealized_profit, 461.0);
    assert_eq!(position.pl, 861.0);
    assert_eq!(position.buy_qty, 2);
    assert_eq!(position.sell_qty, 1);
    assert_eq!(position.fy_token, "1120200831217406");
    assert_eq!(position.cross_currency, "N");
    assert_eq!(position.symbol, "MCX:SILVERMIC20AUGFUT");
    assert_eq!(position.id, "MCX:SILVERMIC20AUGFUT-MARGIN");
    assert_eq!(position.day_sell_qty, 1);

    assert_eq!(response.overall.count_total, 1);
    assert_eq!(response.overall.count_open, 1);
    assert_eq!(response.overall.pl_total, 861.0);
    assert_eq!(response.overall.pl_realized, 400.0);
    assert_eq!(response.overall.pl_unrealized, 461.0);
}

#[test]
fn order_history_query_fixture_records_documented_filters() {
    let pairs = query_pairs("rest/reports/order_history/request_query.txt");

    assert_eq!(
        pairs,
        vec![
            ("exchange_type".to_owned(), "0".to_owned()),
            ("segment_type".to_owned(), "0".to_owned()),
            ("status".to_owned(), "0".to_owned()),
            ("symbol".to_owned(), "INFIBEAM".to_owned()),
            ("from_date".to_owned(), "2025-04-01".to_owned()),
            ("to_date".to_owned(), "2025-12-22".to_owned()),
            ("page_no".to_owned(), "1".to_owned()),
            ("page_size".to_owned(), "100".to_owned()),
        ]
    );
}

#[test]
fn order_history_success_fixture_matches_model() {
    let response: OrderHistoryResponse =
        support::json_fixture("rest/reports/order_history/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.data.len(), 1);

    let order = &response.data[0];
    assert_eq!(order.client_id, "XXXXXX");
    assert_eq!(order.id_fyers, "26020100012037");
    assert_eq!(order.symbol, "MCX:GOLDPETAL26FEBFUT");
    assert_eq!(order.status, "Executed");
    assert_eq!(order.traded_qty, 1);
}

#[test]
fn trade_history_query_fixture_records_documented_filters() {
    let pairs = query_pairs("rest/reports/trade_history/request_query.txt");

    assert_eq!(
        pairs,
        vec![
            ("exchange_type".to_owned(), "0".to_owned()),
            ("segment_type".to_owned(), "0".to_owned()),
            ("symbol".to_owned(), "NSE:SUZLON-A-EQ".to_owned()),
            ("from_date".to_owned(), "2025-04-01".to_owned()),
            ("to_date".to_owned(), "2025-12-22".to_owned()),
            ("page_no".to_owned(), "1".to_owned()),
            ("page_size".to_owned(), "10".to_owned()),
        ]
    );
}

#[test]
fn trade_history_success_fixture_matches_model() {
    let response: TradeHistoryResponse =
        support::json_fixture("rest/reports/trade_history/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, Some(200));
    assert_eq!(response.data.len(), 1);

    let trade = &response.data[0];
    assert_eq!(trade.client_id, "XXXXXX");
    assert_eq!(trade.order_number, "25061300346219");
    assert_eq!(trade.symbol, "MCX:GOLDPETAL25JUNFUT");
    assert_eq!(trade.traded_qty, 1);
    assert_eq!(trade.product_type, "Overnight");
}

fn query_pairs(relative_path: &str) -> Vec<(String, String)> {
    let fixture = support::read_fixture(relative_path);
    url::form_urlencoded::parse(fixture.trim().as_bytes())
        .into_owned()
        .collect()
}
