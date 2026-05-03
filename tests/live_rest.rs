#![cfg(feature = "live-tests")]

mod support;

use fyers_rs::models::market_data::{
    HistoryRequest, MarketDepthRequest, OptionChainRequest, QuotesRequest,
};
use fyers_rs::models::orders::OrderBookQuery;
use fyers_rs::models::transactions::TradeBookQuery;

#[tokio::test]
#[ignore = "requires live Fyers credentials; read-only account endpoints"]
async fn live_account_reads_complete_or_return_broker_errors() {
    let Some(config) = support::LiveTestConfig::from_env("live_account_reads") else {
        return;
    };
    let client = config.client();

    let profile = client.profile().get().await.expect("profile response");
    let funds = client.funds().get().await.expect("funds response");
    let holdings = client.holdings().get().await.expect("holdings response");

    assert!(!profile.s.is_empty());
    assert!(!funds.s.is_empty());
    assert!(!holdings.s.is_empty());
}

#[tokio::test]
#[ignore = "requires live Fyers credentials; read-only market data endpoints"]
async fn live_market_data_reads_complete_or_return_broker_errors() {
    let Some(config) = support::LiveTestConfig::from_env("live_market_data_reads") else {
        return;
    };
    let client = config.client();

    let market_status = client
        .market_data()
        .market_status()
        .await
        .expect("market status response");
    let quotes = client
        .market_data()
        .quotes(&QuotesRequest {
            symbols: vec![config.symbol.clone()],
        })
        .await
        .expect("quotes response");
    let history = client
        .market_data()
        .history(&HistoryRequest {
            symbol: config.symbol.clone(),
            resolution: "D".to_owned(),
            date_format: 1,
            range_from: config.history_range_from,
            range_to: config.history_range_to,
            cont_flag: None,
            oi_flag: None,
        })
        .await
        .expect("history response");
    let depth = client
        .market_data()
        .depth(&MarketDepthRequest {
            symbol: config.symbol,
            ohlcv_flag: 1,
        })
        .await
        .expect("depth response");
    let option_chain = client
        .market_data()
        .option_chain(&OptionChainRequest {
            symbol: config.option_chain_symbol,
            strikecount: Some(1),
            timestamp: None,
            greeks: None,
        })
        .await
        .expect("option chain response");

    assert!(!market_status.s.is_empty());
    assert!(!quotes.s.is_empty());
    assert!(!history.s.is_empty());
    assert!(!depth.s.is_empty());
    assert!(!option_chain.s.is_empty());
}

#[tokio::test]
#[ignore = "requires live Fyers credentials; read-only transaction endpoints"]
async fn live_transaction_reads_complete_or_return_broker_errors() {
    let Some(config) = support::LiveTestConfig::from_env("live_transaction_reads") else {
        return;
    };
    let client = config.client();

    let orders = client
        .orders()
        .list(&OrderBookQuery::default())
        .await
        .expect("order book response");
    let trades = client
        .trades()
        .list(&TradeBookQuery::default())
        .await
        .expect("trade book response");
    let positions = client.positions().list().await.expect("positions response");

    assert!(!orders.s.is_empty());
    assert!(!trades.s.is_empty());
    assert!(!positions.s.is_empty());
}
