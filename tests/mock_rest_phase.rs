mod support;

use fyers_rs::FyersClient;
use fyers_rs::models::alerts::{
    CreatePriceAlertRequest, DeletePriceAlertRequest, ModifyPriceAlertRequest, PriceAlertQuery,
    TogglePriceAlertRequest,
};
use fyers_rs::models::edis::{EdisIndexRequest, EdisInquiryRequest};
use fyers_rs::models::margin::{MultiOrderMarginRequest, SpanMarginRequest};
use fyers_rs::models::market_data::{
    HistoryRequest, MarketDepthRequest, OptionChainRequest, QuotesRequest,
    SymbolMasterExchangeSegment,
};
use fyers_rs::models::portfolio::{
    ConvertPositionRequest, ExitAllPositionsRequest, ExitPositionsByFilterRequest,
    ExitPositionsByIdRequest, PendingOrderCancelRequest,
};
use fyers_rs::models::smart_orders::{
    FlowIdRequest, ModifySmartOrderRequest, SmartLimitOrderRequest, SmartOrderBookQuery,
    SmartSipOrderRequest, SmartStepOrderRequest, SmartTrailOrderRequest,
};
use pretty_assertions::assert_eq;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn smart_order_endpoints_use_documented_methods_paths_and_bodies() {
    let mock_server = MockServer::start().await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v3/smart-order/limit",
        Some("rest/smart_orders/create_limit/request.json"),
        "rest/smart_orders/create_limit/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v3/smart-order/trail",
        Some("rest/smart_orders/create_trail/request.json"),
        "rest/smart_orders/create_trail/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v3/smart-order/step",
        Some("rest/smart_orders/create_step/request.json"),
        "rest/smart_orders/create_step/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v3/smart-order/sip",
        Some("rest/smart_orders/create_sip/request.json"),
        "rest/smart_orders/create_sip/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "PATCH",
        "/api/v3/smart-order/modify",
        Some("rest/smart_orders/modify/request.json"),
        "rest/smart_orders/modify/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "DELETE",
        "/api/v3/smart-order/cancel",
        Some("rest/smart_orders/cancel/request.json"),
        "rest/smart_orders/cancel/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "PATCH",
        "/api/v3/smart-order/pause",
        Some("rest/smart_orders/pause/request.json"),
        "rest/smart_orders/pause/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "PATCH",
        "/api/v3/smart-order/resume",
        Some("rest/smart_orders/resume/request.json"),
        "rest/smart_orders/resume/response_success.json",
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/api/v3/smart-order/orderbook"))
        .and(query_param("page_no", "1"))
        .and(query_param("page_size", "15"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::read_fixture("rest/smart_orders/orderbook/response_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);
    let limit: SmartLimitOrderRequest =
        support::json_fixture("rest/smart_orders/create_limit/request.json");
    let trail: SmartTrailOrderRequest =
        support::json_fixture("rest/smart_orders/create_trail/request.json");
    let step: SmartStepOrderRequest =
        support::json_fixture("rest/smart_orders/create_step/request.json");
    let sip: SmartSipOrderRequest =
        support::json_fixture("rest/smart_orders/create_sip/request.json");
    let modify: ModifySmartOrderRequest =
        support::json_fixture("rest/smart_orders/modify/request.json");
    let flow: FlowIdRequest = support::json_fixture("rest/smart_orders/cancel/request.json");
    let query = smart_orderbook_query();

    assert_eq!(
        client
            .smart_orders()
            .create_limit(&limit)
            .await
            .unwrap()
            .code,
        200
    );
    assert_eq!(
        client
            .smart_orders()
            .create_trail(&trail)
            .await
            .unwrap()
            .code,
        200
    );
    assert_eq!(
        client.smart_orders().create_step(&step).await.unwrap().code,
        200
    );
    assert_eq!(
        client.smart_orders().create_sip(&sip).await.unwrap().code,
        200
    );
    assert_eq!(
        client.smart_orders().modify(&modify).await.unwrap().code,
        200
    );
    assert_eq!(client.smart_orders().cancel(&flow).await.unwrap().code, 200);
    assert_eq!(client.smart_orders().pause(&flow).await.unwrap().code, 200);
    assert_eq!(client.smart_orders().resume(&flow).await.unwrap().code, 200);
    assert_eq!(
        client
            .smart_orders()
            .orderbook(&query)
            .await
            .unwrap()
            .order_book
            .len(),
        1
    );
}

#[tokio::test]
async fn positions_and_margin_endpoints_use_documented_methods_paths_and_bodies() {
    let mock_server = MockServer::start().await;
    mount_json_endpoint(
        &mock_server,
        "DELETE",
        "/api/v3/positions",
        Some("rest/positions/exit_all/request.json"),
        "rest/positions/exit_all/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "DELETE",
        "/api/v3/positions",
        Some("rest/positions/exit_by_id/request.json"),
        "rest/positions/exit_by_id/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "DELETE",
        "/api/v3/positions",
        Some("rest/positions/exit_by_filters/request.json"),
        "rest/positions/exit_by_filters/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "DELETE",
        "/api/v3/positions",
        Some("rest/positions/pending_order_cancel/request.json"),
        "rest/positions/pending_order_cancel/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v3/positions",
        Some("rest/positions/convert/request.json"),
        "rest/positions/convert/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v2/span_margin",
        Some("rest/margin/span_margin/request.json"),
        "rest/margin/span_margin/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v3/multiorder/margin",
        Some("rest/margin/multiorder_margin/request.json"),
        "rest/margin/multiorder_margin/response_success.json",
    )
    .await;

    let client = authenticated_client(&mock_server);
    let exit_all: ExitAllPositionsRequest =
        support::json_fixture("rest/positions/exit_all/request.json");
    let exit_by_id: ExitPositionsByIdRequest =
        support::json_fixture("rest/positions/exit_by_id/request.json");
    let exit_by_filters: ExitPositionsByFilterRequest =
        support::json_fixture("rest/positions/exit_by_filters/request.json");
    let pending_cancel: PendingOrderCancelRequest =
        support::json_fixture("rest/positions/pending_order_cancel/request.json");
    let convert: ConvertPositionRequest =
        support::json_fixture("rest/positions/convert/request.json");
    let span: SpanMarginRequest = support::json_fixture("rest/margin/span_margin/request.json");
    let multi: MultiOrderMarginRequest =
        support::json_fixture("rest/margin/multiorder_margin/request.json");

    assert_eq!(
        client.positions().exit_all(&exit_all).await.unwrap().code,
        200
    );
    assert_eq!(
        client
            .positions()
            .exit_by_id(&exit_by_id)
            .await
            .unwrap()
            .code,
        200
    );
    assert_eq!(
        client
            .positions()
            .exit_by_filters(&exit_by_filters)
            .await
            .unwrap()
            .code,
        200
    );
    assert_eq!(
        client
            .positions()
            .exit_with_pending_order_cancel(&pending_cancel)
            .await
            .unwrap()
            .code,
        200
    );
    assert_eq!(
        client.positions().convert(&convert).await.unwrap().code,
        200
    );
    assert_eq!(client.orders().span_margin(&span).await.unwrap().code, 200);
    assert_eq!(
        client
            .orders()
            .multiorder_margin(&multi)
            .await
            .unwrap()
            .code,
        200
    );
}

#[tokio::test]
async fn broker_data_and_edis_endpoints_use_documented_bases() {
    let mock_server = MockServer::start().await;
    mount_json_endpoint(
        &mock_server,
        "GET",
        "/data/marketStatus",
        None,
        "rest/broker/market_status/response_success.json",
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/sym_details/NSE_CM.csv"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::read_fixture("rest/broker/symbol_master_csv_sample.csv"),
            "text/csv",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/sym_details/NSE_CM_sym_master.json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::read_fixture("rest/broker/symbol_master_json_sample.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;
    mount_json_endpoint(
        &mock_server,
        "GET",
        "/api/v2/tpin",
        None,
        "rest/edis/tpin/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "GET",
        "/api/v2/details",
        None,
        "rest/edis/details/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v2/index",
        Some("rest/edis/index/request.json"),
        "rest/edis/index/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v2/inquiry",
        Some("rest/edis/inquiry/request.json"),
        "rest/edis/inquiry/response_success.json",
    )
    .await;

    let client = authenticated_client(&mock_server);
    let index: EdisIndexRequest = support::json_fixture("rest/edis/index/request.json");
    let inquiry: EdisInquiryRequest = support::json_fixture("rest/edis/inquiry/request.json");

    assert_eq!(
        client
            .market_data()
            .market_status()
            .await
            .unwrap()
            .market_status
            .len(),
        1
    );
    assert!(
        !client
            .market_data()
            .symbol_master_csv(SymbolMasterExchangeSegment::NseCm)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .market_data()
            .symbol_master_json(SymbolMasterExchangeSegment::NseCm)
            .await
            .unwrap()
            .get("NSE:SBIN-EQ")
            .is_some()
    );
    assert_eq!(client.edis().generate_tpin().await.unwrap().code, 200);
    assert_eq!(client.edis().details().await.unwrap().data.len(), 1);
    assert_eq!(client.edis().index(&index).await.unwrap().code, 200);
    assert_eq!(
        client
            .edis()
            .inquiry(&inquiry)
            .await
            .unwrap()
            .data
            .success_count,
        1
    );
}

#[tokio::test]
async fn market_data_and_alert_endpoints_use_documented_shapes() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/data/history"))
        .and(query_param("symbol", "NSE:SBIN-EQ"))
        .and(query_param("resolution", "D"))
        .and(query_param("date_format", "1"))
        .and(query_param("range_from", "2026-04-01"))
        .and(query_param("range_to", "2026-04-29"))
        .and(query_param("cont_flag", "1"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::read_fixture("rest/data/history/response_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/data/quotes"))
        .and(query_param("symbols", "NSE:SBIN-EQ,NSE:ITC-EQ"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::read_fixture("rest/data/quotes/response_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/data/depth"))
        .and(query_param("symbol", "NSE:SBIN-EQ"))
        .and(query_param("ohlcv_flag", "1"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::read_fixture("rest/data/depth/response_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/data/options-chain-v3"))
        .and(query_param("symbol", "NSE:SBIN-EQ"))
        .and(query_param("strikecount", "2"))
        .and(query_param("greeks", "1"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::read_fixture("rest/data/option_chain/response_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;
    mount_json_endpoint(
        &mock_server,
        "POST",
        "/api/v3/price-alert",
        Some("rest/alerts/create/request.json"),
        "rest/alerts/create/response_success.json",
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/api/v3/price-alert"))
        .and(query_param("name", "SBIN breakout"))
        .and(query_param("alertId", "alert-001"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::read_fixture("rest/alerts/list/response_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;
    mount_json_endpoint(
        &mock_server,
        "PUT",
        "/api/v3/price-alert",
        Some("rest/alerts/modify/request.json"),
        "rest/alerts/modify/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "DELETE",
        "/api/v3/price-alert",
        Some("rest/alerts/delete/request.json"),
        "rest/alerts/delete/response_success.json",
    )
    .await;
    mount_json_endpoint(
        &mock_server,
        "PUT",
        "/api/v3/toggle-alert",
        Some("rest/alerts/toggle/request.json"),
        "rest/alerts/toggle/response_success.json",
    )
    .await;

    let client = authenticated_client(&mock_server);
    let history = HistoryRequest {
        symbol: "NSE:SBIN-EQ".to_owned(),
        resolution: "D".to_owned(),
        date_format: 1,
        range_from: "2026-04-01".to_owned(),
        range_to: "2026-04-29".to_owned(),
        cont_flag: Some(1),
        oi_flag: None,
    };
    let quotes = QuotesRequest {
        symbols: vec!["NSE:SBIN-EQ".to_owned(), "NSE:ITC-EQ".to_owned()],
    };
    let depth = MarketDepthRequest {
        symbol: "NSE:SBIN-EQ".to_owned(),
        ohlcv_flag: 1,
    };
    let option_chain = OptionChainRequest {
        symbol: "NSE:SBIN-EQ".to_owned(),
        strikecount: Some(2),
        timestamp: None,
        greeks: Some("1".to_owned()),
    };
    let create_alert: CreatePriceAlertRequest =
        support::json_fixture("rest/alerts/create/request.json");
    let modify_alert: ModifyPriceAlertRequest =
        support::json_fixture("rest/alerts/modify/request.json");
    let delete_alert: DeletePriceAlertRequest =
        support::json_fixture("rest/alerts/delete/request.json");
    let toggle_alert: TogglePriceAlertRequest =
        support::json_fixture("rest/alerts/toggle/request.json");
    let alert_query = PriceAlertQuery {
        name: Some("SBIN breakout".to_owned()),
        alert_id: Some("alert-001".to_owned()),
    };

    assert_eq!(
        client
            .market_data()
            .history(&history)
            .await
            .unwrap()
            .candles
            .len(),
        1
    );
    assert_eq!(
        client
            .market_data()
            .quotes(&quotes)
            .await
            .unwrap()
            .data
            .len(),
        1
    );
    assert!(
        client
            .market_data()
            .depth(&depth)
            .await
            .unwrap()
            .data
            .contains_key("NSE:SBIN-EQ")
    );
    assert_eq!(
        client
            .market_data()
            .option_chain(&option_chain)
            .await
            .unwrap()
            .data
            .options_chain
            .len(),
        2
    );
    assert_eq!(
        client.alerts().create(&create_alert).await.unwrap().code,
        120
    );
    assert_eq!(
        client.alerts().list(&alert_query).await.unwrap().data.len(),
        1
    );
    assert_eq!(
        client.alerts().modify(&modify_alert).await.unwrap().code,
        123
    );
    assert_eq!(
        client.alerts().delete(&delete_alert).await.unwrap().code,
        121
    );
    assert_eq!(
        client.alerts().toggle(&toggle_alert).await.unwrap().code,
        123
    );
}

async fn mount_json_endpoint(
    mock_server: &MockServer,
    http_method: &'static str,
    endpoint_path: &'static str,
    request_fixture: Option<&'static str>,
    response_fixture: &'static str,
) {
    let mut mock = Mock::given(method(http_method))
        .and(path(endpoint_path))
        .and(auth_header());
    if let Some(request_fixture) = request_fixture {
        mock = mock.and(body_json(support::json_value(request_fixture)));
    }

    mock.respond_with(
        ResponseTemplate::new(200)
            .set_body_raw(support::read_fixture(response_fixture), "application/json"),
    )
    .expect(1)
    .mount(mock_server)
    .await;
}

fn smart_orderbook_query() -> SmartOrderBookQuery {
    SmartOrderBookQuery {
        exchange: vec![],
        side: vec![],
        flowtype: vec![],
        product: vec![],
        message_type: vec![],
        search: None,
        sort_by: None,
        ord_by: None,
        page_no: Some(1),
        page_size: Some(15),
    }
}

fn authenticated_client(mock_server: &MockServer) -> FyersClient {
    FyersClient::builder()
        .client_id("APPID-100")
        .access_token("ACCESS_TOKEN")
        .api_base_url_str(format!("{}/api/v3", mock_server.uri()))
        .expect("API base URL should parse")
        .api_v2_base_url_str(format!("{}/api/v2", mock_server.uri()))
        .expect("API v2 base URL should parse")
        .data_base_url_str(format!("{}/data", mock_server.uri()))
        .expect("data base URL should parse")
        .symbols_base_url_str(format!("{}/sym_details", mock_server.uri()))
        .expect("symbols base URL should parse")
        .build()
        .expect("client should build")
}

fn auth_header() -> impl wiremock::Match {
    header("authorization", "APPID-100:ACCESS_TOKEN")
}
