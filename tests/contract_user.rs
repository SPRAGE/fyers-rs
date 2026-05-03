mod support;

use fyers_rs::models::user::{FundsResponse, HoldingsResponse, ProfileResponse};
use pretty_assertions::assert_eq;

#[test]
fn profile_success_fixture_matches_model() {
    let response: ProfileResponse =
        support::json_fixture("rest/user/profile/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.message, "");
    assert_eq!(response.data.name, "XASHXX G H");
    assert_eq!(response.data.display_name, "Y2K");
    assert_eq!(response.data.email_id, "txxxxxxxxxxx2@gmail.com");
    assert_eq!(response.data.pan, "FYxxxxxx0S");
    assert_eq!(response.data.fy_id, "FX0011");
    assert!(response.data.totp);
    assert!(!response.data.ddpi_enabled);
    assert!(!response.data.mtf_enabled);
}

#[test]
fn funds_success_fixture_matches_model() {
    let response: FundsResponse = support::json_fixture("rest/user/funds/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.fund_limit.len(), 10);

    let total_balance = &response.fund_limit[0];
    assert_eq!(total_balance.id, 1);
    assert_eq!(total_balance.title, "Total Balance");
    assert_eq!(total_balance.equity_amount, 58.150000000000006);
    assert_eq!(total_balance.commodity_amount, 0.0);

    let realized_pnl = &response.fund_limit[3];
    assert_eq!(realized_pnl.title, "Realized Profit and Loss");
    assert_eq!(realized_pnl.equity_amount, -0.3);
}

#[test]
fn holdings_success_fixture_matches_model() {
    let response: HoldingsResponse =
        support::json_fixture("rest/user/holdings/response_success.json");

    assert_eq!(response.s, "ok");
    assert_eq!(response.code, 200);
    assert_eq!(response.holdings.len(), 2);

    let first_holding = &response.holdings[0];
    assert_eq!(first_holding.holding_type, "HLD");
    assert_eq!(first_holding.quantity, 1);
    assert_eq!(first_holding.cost_price, 1.55);
    assert_eq!(first_holding.market_val, 3.75);
    assert_eq!(first_holding.remaining_quantity, 1);
    assert_eq!(first_holding.pl, 2.2);
    assert_eq!(first_holding.ltp, 3.75);
    assert_eq!(first_holding.fy_token, "101000000011460");
    assert_eq!(first_holding.symbol, "NSE:JPASSOCIAT-EQ");
    assert_eq!(first_holding.qty_t1, 1);
    assert_eq!(first_holding.remaining_pledge_quantity, -1);
    assert_eq!(first_holding.collateral_quantity, 0);

    assert_eq!(response.overall.count_total, 2);
    assert_eq!(response.overall.total_investment, 194.15);
    assert_eq!(response.overall.total_current_value, 153.45);
    assert_eq!(response.overall.total_pl, -40.7);
    assert_eq!(response.overall.pnl_perc, -10.48);
}
