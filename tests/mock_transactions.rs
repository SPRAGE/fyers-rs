mod support;

use fyers_rs::FyersClient;
use fyers_rs::FyersError;
use fyers_rs::models::gtt::{CancelGttOrderRequest, GttOrderRequest, ModifyGttOrderRequest};
use fyers_rs::models::orders::{
    CancelOrderRequest, ModifyOrderRequest, MultiLegOrderRequest, OrderBookQuery, PlaceOrderRequest,
};
use fyers_rs::models::reports::{OrderHistoryQuery, TradeHistoryQuery};
use fyers_rs::models::transactions::TradeBookQuery;
use pretty_assertions::assert_eq;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn tradebook_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/trades/tradebook/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/tradebook"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .trades()
        .list(&TradeBookQuery::default())
        .await
        .expect("tradebook should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.trade_book.len(), 3);
}

#[tokio::test]
async fn tradebook_filters_by_order_tag_query_param() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/trades/tradebook/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/tradebook"))
        .and(query_param("order_tag", "1:Ordertag"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .trades()
        .list(&TradeBookQuery::by_order_tag("1:Ordertag"))
        .await
        .expect("filtered tradebook should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.trade_book[0].order_tag, "1:Ordertag");
}

#[tokio::test]
async fn orderbook_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/orderbook/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/orders"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .list(&OrderBookQuery::default())
        .await
        .expect("orderbook should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.order_book.len(), 1);
}

#[tokio::test]
async fn orders_filter_by_id_query_param() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/orderbook/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/orders"))
        .and(query_param("id", "sample_order_id"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .get_by_id("sample_order_id")
        .await
        .expect("order by id should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.order_book[0].id, "23030900015105");
}

#[tokio::test]
async fn orders_filter_by_order_tag_query_param() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/orderbook/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/orders"))
        .and(query_param("order_tag", "1:Ordertag"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .list(&OrderBookQuery::by_order_tag("1:Ordertag"))
        .await
        .expect("orders by tag should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.order_book[0].order_tag, "1:Ordertag");
}

#[tokio::test]
async fn place_sync_posts_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/place_sync/response_success.json");
    let request: PlaceOrderRequest = support::json_fixture("rest/orders/place_sync/request.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/orders/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/place_sync/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .place_sync(&request)
        .await
        .expect("single sync order placement should succeed");

    assert_eq!(response.code, 1101);
    assert_eq!(response.id, "23030900015105");
}

#[tokio::test]
async fn place_async_posts_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/place_async/response_success.json");
    let request: PlaceOrderRequest = support::json_fixture("rest/orders/place_async/request.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/orders/async"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/place_async/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .place_async(&request)
        .await
        .expect("async order placement should succeed");

    assert_eq!(response.code, 1101);
    assert_eq!(response.id_fyers, "c6697c04-d9ab-4a7c-a6f4-b0cc4ca698f6");
}

#[tokio::test]
async fn modify_sync_patches_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/modify_sync/response_success.json");
    let request: ModifyOrderRequest = support::json_fixture("rest/orders/modify_sync/request.json");

    Mock::given(method("PATCH"))
        .and(path("/api/v3/orders/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/modify_sync/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .modify_sync(&request)
        .await
        .expect("sync order modification should succeed");

    assert_eq!(response.code, 1101);
    assert_eq!(response.id, "23030900015105");
}

#[tokio::test]
async fn modify_async_patches_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/modify_async/response_success.json");
    let request: ModifyOrderRequest =
        support::json_fixture("rest/orders/modify_async/request.json");

    Mock::given(method("PATCH"))
        .and(path("/api/v3/orders/async"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/modify_async/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .modify_async(&request)
        .await
        .expect("async order modification should succeed");

    assert_eq!(response.code, 1101);
    assert_eq!(response.id_fyers, "c6697c04-d9ab-4a7c-a6f4-b0cc4ca698f6");
}

#[tokio::test]
async fn cancel_sync_deletes_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/cancel_sync/response_success.json");
    let request: CancelOrderRequest = support::json_fixture("rest/orders/cancel_sync/request.json");

    Mock::given(method("DELETE"))
        .and(path("/api/v3/orders/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/cancel_sync/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .cancel_sync(&request)
        .await
        .expect("sync order cancellation should succeed");

    assert_eq!(response.code, 1103);
    assert_eq!(response.id, "23030900015105");
}

#[tokio::test]
async fn cancel_async_deletes_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/cancel_async/response_success.json");
    let request: CancelOrderRequest =
        support::json_fixture("rest/orders/cancel_async/request.json");

    Mock::given(method("DELETE"))
        .and(path("/api/v3/orders/async"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/cancel_async/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .cancel_async(&request)
        .await
        .expect("async order cancellation should succeed");

    assert_eq!(response.code, 1103);
    assert_eq!(response.id_fyers, "c6697c04-d9ab-4a7c-a6f4-b0cc4ca698f6");
}

#[tokio::test]
async fn cancel_sync_by_id_deletes_documented_path_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body =
        support::read_fixture("rest/orders/cancel_sync_by_path/response_success.json");

    Mock::given(method("DELETE"))
        .and(path("/api/v3/orders/23030900015105/sync"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .cancel_sync_by_id("23030900015105")
        .await
        .expect("sync path order cancellation should succeed");

    assert_eq!(response.code, 1103);
    assert_eq!(response.id, "23030900015105");
}

#[tokio::test]
async fn place_multi_sync_posts_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/orders/place_multi_sync/response_success.json");
    let requests: Vec<PlaceOrderRequest> =
        support::json_fixture("rest/orders/place_multi_sync/request.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/multi-order/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/place_multi_sync/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .place_multi_sync(&requests)
        .await
        .expect("multi sync order placement should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data.len(), 2);
    assert_eq!(response.data[0].body.id, "120080778988");
}

#[tokio::test]
async fn place_multi_async_posts_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body =
        support::read_fixture("rest/orders/place_multi_async/response_success.json");
    let requests: Vec<PlaceOrderRequest> =
        support::json_fixture("rest/orders/place_multi_async/request.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/multi-order/async"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/place_multi_async/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .place_multi_async(&requests)
        .await
        .expect("async multi order placement should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data.len(), 2);
}

#[tokio::test]
async fn modify_multi_sync_patches_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body =
        support::read_fixture("rest/orders/modify_multi_sync/response_success.json");
    let requests: Vec<ModifyOrderRequest> =
        support::json_fixture("rest/orders/modify_multi_sync/request.json");

    Mock::given(method("PATCH"))
        .and(path("/api/v3/multi-order/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/modify_multi_sync/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .modify_multi_sync(&requests)
        .await
        .expect("sync multi order modification should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data.len(), 2);
}

#[tokio::test]
async fn modify_multi_async_patches_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body =
        support::read_fixture("rest/orders/modify_multi_async/response_success.json");
    let requests: Vec<ModifyOrderRequest> =
        support::json_fixture("rest/orders/modify_multi_async/request.json");

    Mock::given(method("PATCH"))
        .and(path("/api/v3/multi-order/async"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/modify_multi_async/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .modify_multi_async(&requests)
        .await
        .expect("async multi order modification should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data.len(), 2);
}

#[tokio::test]
async fn cancel_multi_sync_deletes_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body =
        support::read_fixture("rest/orders/cancel_multi_sync/response_success.json");
    let requests: Vec<CancelOrderRequest> =
        support::json_fixture("rest/orders/cancel_multi_sync/request.json");

    Mock::given(method("DELETE"))
        .and(path("/api/v3/multi-order/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/cancel_multi_sync/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .cancel_multi_sync(&requests)
        .await
        .expect("sync multi order cancellation should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data.len(), 2);
}

#[tokio::test]
async fn cancel_multi_async_deletes_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body =
        support::read_fixture("rest/orders/cancel_multi_async/response_success.json");
    let requests: Vec<CancelOrderRequest> =
        support::json_fixture("rest/orders/cancel_multi_async/request.json");

    Mock::given(method("DELETE"))
        .and(path("/api/v3/multi-order/async"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/cancel_multi_async/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .cancel_multi_async(&requests)
        .await
        .expect("async multi order cancellation should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data.len(), 2);
}

#[tokio::test]
async fn place_multi_sync_rejects_more_than_ten_orders_before_http() {
    let mock_server = MockServer::start().await;
    let request: PlaceOrderRequest = support::json_fixture("rest/orders/place_sync/request.json");
    let requests = vec![request; 11];
    let client = authenticated_client(&mock_server);

    let err = client
        .orders()
        .place_multi_sync(&requests)
        .await
        .expect_err("more than 10 orders should fail validation");

    assert!(matches!(
        err,
        FyersError::Validation(message)
            if message == "multi-order placement supports at most 10 orders"
    ));
}

#[tokio::test]
async fn place_multileg_sync_posts_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body =
        support::read_fixture("rest/orders/place_multileg_sync/response_success.json");
    let request: MultiLegOrderRequest =
        support::json_fixture("rest/orders/place_multileg_sync/request.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/multileg/orders/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/orders/place_multileg_sync/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .orders()
        .place_multileg_sync(&request)
        .await
        .expect("multi-leg sync order placement should succeed");

    assert_eq!(response.code, 1101);
    assert_eq!(response.id, "23030900015107");
}

#[tokio::test]
async fn place_multileg_sync_rejects_three_leg_without_leg3_before_http() {
    let mock_server = MockServer::start().await;
    let mut request: MultiLegOrderRequest =
        support::json_fixture("rest/orders/place_multileg_sync/request.json");
    request.legs.leg3 = None;
    let client = authenticated_client(&mock_server);

    let err = client
        .orders()
        .place_multileg_sync(&request)
        .await
        .expect_err("3L order without leg3 should fail validation");

    assert!(matches!(
        err,
        FyersError::Validation(message)
            if message == "leg3 is required for 3L multi-leg orders"
    ));
}

#[tokio::test]
async fn gtt_place_single_posts_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/gtt/place_single/response_success.json");
    let request: GttOrderRequest = support::json_fixture("rest/gtt/place_single/request.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/gtt/orders/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/gtt/place_single/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .gtt()
        .place_single(&request)
        .await
        .expect("single GTT placement should succeed");

    assert_eq!(response.code, 1101);
    assert_eq!(response.id.as_deref(), Some("25111300002007"));
}

#[tokio::test]
async fn gtt_place_single_rejects_oco_body_before_http() {
    let mock_server = MockServer::start().await;
    let request: GttOrderRequest = support::json_fixture("rest/gtt/place_oco/request.json");
    let client = authenticated_client(&mock_server);

    let err = client
        .gtt()
        .place_single(&request)
        .await
        .expect_err("single GTT placement with leg2 should fail validation");

    assert!(matches!(
        err,
        FyersError::Validation(message) if message == "single GTT order does not support leg2"
    ));
}

#[tokio::test]
async fn gtt_place_oco_posts_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/gtt/place_oco/response_success.json");
    let request: GttOrderRequest = support::json_fixture("rest/gtt/place_oco/request.json");

    Mock::given(method("POST"))
        .and(path("/api/v3/gtt/orders/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/gtt/place_oco/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .gtt()
        .place_oco(&request)
        .await
        .expect("OCO GTT placement should succeed");

    assert_eq!(response.code, 1101);
    assert_eq!(response.id.as_deref(), Some("25111300002008"));
}

#[tokio::test]
async fn gtt_place_oco_requires_leg2_before_http() {
    let mock_server = MockServer::start().await;
    let request: GttOrderRequest = support::json_fixture("rest/gtt/place_single/request.json");
    let client = authenticated_client(&mock_server);

    let err = client
        .gtt()
        .place_oco(&request)
        .await
        .expect_err("OCO GTT placement without leg2 should fail validation");

    assert!(matches!(
        err,
        FyersError::Validation(message) if message == "leg2 is required for OCO GTT orders"
    ));
}

#[tokio::test]
async fn gtt_modify_patches_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/gtt/modify/response_success.json");
    let request: ModifyGttOrderRequest = support::json_fixture("rest/gtt/modify/request.json");

    Mock::given(method("PATCH"))
        .and(path("/api/v3/gtt/orders/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/gtt/modify/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .gtt()
        .modify(&request)
        .await
        .expect("GTT modification should succeed");

    assert_eq!(response.code, 1102);
    assert_eq!(response.id.as_deref(), Some("26021800001427"));
}

#[tokio::test]
async fn gtt_cancel_deletes_documented_body_with_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/gtt/cancel/response_success.json");
    let request: CancelGttOrderRequest = support::json_fixture("rest/gtt/cancel/request.json");

    Mock::given(method("DELETE"))
        .and(path("/api/v3/gtt/orders/sync"))
        .and(auth_header())
        .and(body_json(support::json_value(
            "rest/gtt/cancel/request.json",
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .gtt()
        .cancel(&request)
        .await
        .expect("GTT cancellation should succeed");

    assert_eq!(response.code, 1103);
    assert_eq!(response.id.as_deref(), Some("25111300002007"));
}

#[tokio::test]
async fn gtt_orderbook_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/gtt/orderbook/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/gtt/orders"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .gtt()
        .orderbook()
        .await
        .expect("GTT orderbook should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.order_book.len(), 1);
}

#[tokio::test]
async fn positions_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/positions/list/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/positions"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .positions()
        .list()
        .await
        .expect("positions should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.net_positions.len(), 1);
}

#[tokio::test]
async fn order_history_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/reports/order_history/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/order-history"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .reports()
        .order_history(&OrderHistoryQuery::default())
        .await
        .expect("order history should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data.len(), 1);
}

#[tokio::test]
async fn order_history_filters_with_documented_query_params() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/reports/order_history/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/order-history"))
        .and(query_param("exchange_type", "0"))
        .and(query_param("segment_type", "0"))
        .and(query_param("status", "0"))
        .and(query_param("symbol", "INFIBEAM"))
        .and(query_param("from_date", "2025-04-01"))
        .and(query_param("to_date", "2025-12-22"))
        .and(query_param("page_no", "1"))
        .and(query_param("page_size", "100"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);
    let query = OrderHistoryQuery {
        exchange_type: Some("0".to_owned()),
        segment_type: Some("0".to_owned()),
        status: Some("0".to_owned()),
        symbol: Some("INFIBEAM".to_owned()),
        from_date: Some("2025-04-01".to_owned()),
        to_date: Some("2025-12-22".to_owned()),
        page_no: Some(1),
        page_size: Some(100),
    };

    let response = client
        .reports()
        .order_history(&query)
        .await
        .expect("filtered order history should succeed");

    assert_eq!(response.code, 200);
    assert_eq!(response.data[0].symbol, "MCX:GOLDPETAL26FEBFUT");
}

#[tokio::test]
async fn trade_history_gets_with_documented_authorization_header() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/reports/trade_history/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/trade-history"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);

    let response = client
        .reports()
        .trade_history(&TradeHistoryQuery::default())
        .await
        .expect("trade history should succeed");

    assert_eq!(response.code, Some(200));
    assert_eq!(response.data.len(), 1);
}

#[tokio::test]
async fn trade_history_filters_with_documented_query_params() {
    let mock_server = MockServer::start().await;
    let response_body = support::read_fixture("rest/reports/trade_history/response_success.json");

    Mock::given(method("GET"))
        .and(path("/api/v3/trade-history"))
        .and(query_param("exchange_type", "0"))
        .and(query_param("segment_type", "0"))
        .and(query_param("symbol", "NSE:SUZLON-A-EQ"))
        .and(query_param("from_date", "2025-04-01"))
        .and(query_param("to_date", "2025-12-22"))
        .and(query_param("page_no", "1"))
        .and(query_param("page_size", "10"))
        .and(auth_header())
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_body, "application/json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = authenticated_client(&mock_server);
    let query = TradeHistoryQuery {
        exchange_type: Some("0".to_owned()),
        segment_type: Some("0".to_owned()),
        symbol: Some("NSE:SUZLON-A-EQ".to_owned()),
        from_date: Some("2025-04-01".to_owned()),
        to_date: Some("2025-12-22".to_owned()),
        page_no: Some(1),
        page_size: Some(10),
    };

    let response = client
        .reports()
        .trade_history(&query)
        .await
        .expect("filtered trade history should succeed");

    assert_eq!(response.code, Some(200));
    assert_eq!(response.data[0].symbol, "MCX:GOLDPETAL25JUNFUT");
}

fn auth_header() -> impl wiremock::Match {
    header("authorization", "SPXXXXE7-100:fake_access_token.jwt")
}

fn authenticated_client(mock_server: &MockServer) -> FyersClient {
    FyersClient::builder()
        .client_id("SPXXXXE7-100")
        .access_token("fake_access_token.jwt")
        .api_base_url_str(format!("{}/api/v3", mock_server.uri()))
        .expect("base URL should parse")
        .build()
        .expect("client should build")
}
