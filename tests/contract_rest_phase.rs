mod support;

use fyers_rs::models::alerts::{
    CreatePriceAlertRequest, DeletePriceAlertRequest, ModifyPriceAlertRequest,
    PriceAlertActionResponse, PriceAlertsResponse, TogglePriceAlertRequest,
};
use fyers_rs::models::common::ApiStatus;
use fyers_rs::models::edis::{
    EdisDetailsResponse, EdisIndexRequest, EdisIndexResponse, EdisInquiryRequest,
    EdisInquiryResponse, EdisTpinResponse,
};
use fyers_rs::models::margin::{
    MultiOrderMarginRequest, MultiOrderMarginResponse, SpanMarginRequest, SpanMarginResponse,
};
use fyers_rs::models::market_data::{
    HistoryResponse, MarketDepthResponse, MarketStatusResponse, OptionChainResponse,
    QuotesResponse, SymbolMasterJson,
};
use fyers_rs::models::portfolio::{
    ConvertPositionRequest, ConvertPositionResponse, ExitAllPositionsRequest,
    ExitPositionsByFilterRequest, ExitPositionsByIdRequest, PendingOrderCancelRequest,
};
use fyers_rs::models::postback::OrderPostback;
use fyers_rs::models::smart_orders::{
    FlowIdRequest, ModifySmartOrderRequest, SmartLimitOrderRequest, SmartOrderActionResponse,
    SmartOrderBookResponse, SmartSipOrderRequest, SmartStepOrderRequest, SmartTrailOrderRequest,
};
use pretty_assertions::assert_eq;

#[test]
fn smart_order_fixtures_match_models() {
    assert_json_roundtrip::<SmartLimitOrderRequest>("rest/smart_orders/create_limit/request.json");
    assert_json_roundtrip::<SmartTrailOrderRequest>("rest/smart_orders/create_trail/request.json");
    assert_json_roundtrip::<SmartStepOrderRequest>("rest/smart_orders/create_step/request.json");
    assert_json_roundtrip::<SmartSipOrderRequest>("rest/smart_orders/create_sip/request.json");
    assert_json_roundtrip::<ModifySmartOrderRequest>("rest/smart_orders/modify/request.json");
    assert_json_roundtrip::<FlowIdRequest>("rest/smart_orders/cancel/request.json");
    assert_json_roundtrip::<FlowIdRequest>("rest/smart_orders/pause/request.json");
    assert_json_roundtrip::<FlowIdRequest>("rest/smart_orders/resume/request.json");

    for fixture in [
        "rest/smart_orders/create_limit/response_success.json",
        "rest/smart_orders/create_trail/response_success.json",
        "rest/smart_orders/create_step/response_success.json",
        "rest/smart_orders/create_sip/response_success.json",
        "rest/smart_orders/modify/response_success.json",
        "rest/smart_orders/cancel/response_success.json",
        "rest/smart_orders/pause/response_success.json",
        "rest/smart_orders/resume/response_success.json",
    ] {
        let response: SmartOrderActionResponse = support::json_fixture(fixture);
        assert_eq!(response.s, "ok");
        assert_eq!(response.code, 200);
    }

    let orderbook: SmartOrderBookResponse =
        support::json_fixture("rest/smart_orders/orderbook/response_success.json");
    assert_eq!(
        orderbook.order_book[0].flow_id,
        "9d12ded8-f046-440f-89f5-e750a37e6048"
    );
    assert_eq!(orderbook.filter_count, 1);
}

#[test]
fn position_margin_and_edis_fixtures_match_models() {
    assert_json_roundtrip::<ExitAllPositionsRequest>("rest/positions/exit_all/request.json");
    assert_json_roundtrip::<ExitPositionsByIdRequest>("rest/positions/exit_by_id/request.json");
    assert_json_roundtrip::<ExitPositionsByFilterRequest>(
        "rest/positions/exit_by_filters/request.json",
    );
    assert_json_roundtrip::<PendingOrderCancelRequest>(
        "rest/positions/pending_order_cancel/request.json",
    );
    assert_json_roundtrip::<ConvertPositionRequest>("rest/positions/convert/request.json");
    for fixture in [
        "rest/positions/exit_all/response_success.json",
        "rest/positions/exit_by_id/response_success.json",
        "rest/positions/exit_by_filters/response_success.json",
        "rest/positions/pending_order_cancel/response_success.json",
    ] {
        let response: ApiStatus = support::json_fixture(fixture);
        assert_eq!(response.s, "ok");
    }
    let convert: ConvertPositionResponse =
        support::json_fixture("rest/positions/convert/response_success.json");
    assert_eq!(convert.position_details, 1101);

    assert_json_roundtrip::<SpanMarginRequest>("rest/margin/span_margin/request.json");
    assert_json_roundtrip::<MultiOrderMarginRequest>("rest/margin/multiorder_margin/request.json");
    let span: SpanMarginResponse =
        support::json_fixture("rest/margin/span_margin/response_success.json");
    let multi: MultiOrderMarginResponse =
        support::json_fixture("rest/margin/multiorder_margin/response_success.json");
    assert_eq!(span.data.total_margin, 3000.0);
    assert_eq!(multi.data.margin_new_order, 147738.05634886527);

    let tpin: EdisTpinResponse = support::json_fixture("rest/edis/tpin/response_success.json");
    let details: EdisDetailsResponse =
        support::json_fixture("rest/edis/details/response_success.json");
    assert_json_roundtrip::<EdisIndexRequest>("rest/edis/index/request.json");
    let index: EdisIndexResponse = support::json_fixture("rest/edis/index/response_success.json");
    assert_json_roundtrip::<EdisInquiryRequest>("rest/edis/inquiry/request.json");
    let inquiry: EdisInquiryResponse =
        support::json_fixture("rest/edis/inquiry/response_success.json");
    assert_eq!(
        tpin.message,
        "Successfully sent request for BO Tpin generation"
    );
    assert_eq!(details.data[0].isin, "INE313D01013");
    assert!(index.data.contains("VerifyDIS"));
    assert_eq!(inquiry.data.success_count, 1);
}

#[test]
fn market_data_broker_alert_and_postback_fixtures_match_models() {
    let market_status: MarketStatusResponse =
        support::json_fixture("rest/broker/market_status/response_success.json");
    let symbol_json: SymbolMasterJson =
        support::json_fixture("rest/broker/symbol_master_json_sample.json");
    let history: HistoryResponse = support::json_fixture("rest/data/history/response_success.json");
    let quotes: QuotesResponse = support::json_fixture("rest/data/quotes/response_success.json");
    let depth: MarketDepthResponse = support::json_fixture("rest/data/depth/response_success.json");
    let option_chain: OptionChainResponse =
        support::json_fixture("rest/data/option_chain/response_success.json");

    assert_eq!(market_status.market_status[0].status, "OPEN");
    assert!(symbol_json.get("NSE:SBIN-EQ").is_some());
    assert_eq!(history.candles[0][4], 412.05);
    assert_eq!(quotes.data[0].v.symbol, "NSE:SBIN-EQ");
    assert!(depth.data.contains_key("NSE:SBIN-EQ"));
    assert_eq!(option_chain.data.put_oi, 108234555);
    assert_eq!(option_chain.data.options_chain[1].option_type, "PE");
    assert_eq!(
        option_chain.data.options_chain[1]
            .greeks
            .as_ref()
            .unwrap()
            .iv,
        23.7
    );

    assert_json_roundtrip::<CreatePriceAlertRequest>("rest/alerts/create/request.json");
    assert_json_roundtrip::<ModifyPriceAlertRequest>("rest/alerts/modify/request.json");
    assert_json_roundtrip::<DeletePriceAlertRequest>("rest/alerts/delete/request.json");
    assert_json_roundtrip::<TogglePriceAlertRequest>("rest/alerts/toggle/request.json");
    assert_eq!(
        support::read_fixture("rest/alerts/list/request_query.txt").trim_end(),
        "name=SBIN%20breakout&alertId=alert-001"
    );
    let alert_action: PriceAlertActionResponse =
        support::json_fixture("rest/alerts/create/response_success.json");
    let alerts: PriceAlertsResponse =
        support::json_fixture("rest/alerts/list/response_success.json");
    assert_eq!(alert_action.code, 120);
    assert!(alerts.data.contains_key("alert-001"));

    let postback: OrderPostback = support::json_fixture("rest/postback/order_update.json");
    assert_eq!(postback.id, "23071800238607");
    assert_eq!(postback.status, 2);
}

fn assert_json_roundtrip<T>(fixture: &str)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let request: T = support::json_fixture(fixture);
    let actual = serde_json::to_value(&request).expect("fixture should serialize");
    let expected = support::json_value(fixture);

    assert_eq!(actual, expected);
}
