# fyers-rs

Idiomatic async Rust client for the documented Fyers broker APIs.

> **Status:** pre-release. The market-data WebSocket and several REST endpoints have been live-verified end-to-end against the real Fyers V3 API. Other paths — most importantly **order placement, modification, and cancellation** — match the official Python SDK byte-for-byte but **have never been live-fired**. See the verification matrix below.

> **Safety warning:** this project is entirely AI-generated at this stage. It has not been independently audited or battle-tested. Review, test, and verify behavior yourself before using it with real Fyers credentials, market data, or orders. Live tests and examples are opt-in; no example should place an order unless you explicitly change/run it to do so.

## ⚠️ Order placement has not been tested live

`client.orders().place_sync(...)`, `place_async`, `modify_sync`, `cancel_sync`, the multi/multileg variants, GTT and smart-order endpoints **have never sent a real order through this crate**. The endpoint URLs and request payload shapes match the official Fyers Python SDK (`fyers-apiv3`), and the Fyers server returns expected validation errors when probed without algo-trading entitlement — but a live order on a real account has not yet been placed-and-cancelled to confirm round-trip behaviour.

If you intend to use this crate to place real orders, you **must** verify each path yourself first. Suggested approach:
1. Place one tiny throwaway limit order (e.g. 1 share at an obviously-non-marketable price) on an algo-trading-enabled account.
2. Cancel it immediately.
3. Compare the request body the crate sent against [the documented Fyers V3 API spec](https://myapi-docs.fyers.in/v3/) and the [`fyers-apiv3` Python SDK source](https://pypi.org/project/fyers-apiv3/).
4. Repeat per order type (sync, async, multi, multileg, GTT, smart-order) before relying on each.

Treat every order-mutation call as **irreversible broker interaction**.

## Verification status

| Surface | Status | Notes |
|---|---|---|
| Auth flow (login URL → auth code → access/refresh token → refresh) | ✅ Live-verified | |
| Profile / funds / holdings (REST) | ✅ Live-verified | |
| Data WebSocket — connect, auth handshake, full mode, channel resume, subscribe, snapshot decode, ack flow control | ✅ Live-verified end-to-end | Real `NSE:SBIN-EQ` snapshot decoded with correct OHLC/bid/ask/circuit bands. |
| Data WebSocket — lite mode | ✅ Live-verified | Single-field LTP frame. |
| Data WebSocket — update frames (`data_type=0x55`) during market hours | ⏳ Pending market open | Snapshot path is byte-validated; updates use the same field layout per Python SDK. |
| Data WebSocket — index updates (`if\|...`) | ⚠️ Synthetic test only | Field schema implemented from `index_val`; never seen a live index frame. |
| Data WebSocket — depth updates (`dp\|...`) | ⚠️ Synthetic test only | Field schema implemented from `depthvalue`; never seen a live depth frame. |
| Order WebSocket — connect, subscribe, ping/pong | ✅ Live-verified | Subscribe ack received. No live order events tested. |
| TBT/depth WebSocket | ⚠️ Implementation correct, untested live | Subscribe wire format and protobuf decoder match Python SDK. Account lacks TBT entitlement to validate. Dynamic URL fetching from `/indus/home/tbtws` is not yet implemented. |
| **REST order placement / modification / cancellation** | ❌ **Never live-fired** | URLs match Python SDK; payload shapes have not been round-tripped against a real account. |
| GTT, smart orders, multi-leg orders | ❌ Never live-fired | Same as above. |
| Portfolio operations (exit/convert positions, margin calculators) | ❌ Never live-fired | |
| Market data REST (history, quotes, depth, option chain, symbol master) | ⚠️ Compiled, fixture-tested | Not live-fired in this session. |
| Postback payloads | ⚠️ Models only | No live webhook capture. |
| WebSocket auto-reconnect | ❌ Not implemented | `DataSocketConfig::reconnect` is accepted but no internal loop consumes it. See `DataSocketConnection` rustdoc for the manual reconnect pattern. |

## Coverage

- Auth/session: login URL, auth-code validation, refresh token, logout.
- User and account: profile, funds, holdings.
- Transactions/reports: trades, orders, positions, order history, trade history.
- Orders: sync/async single, multi, multi-leg, modify, cancel, GTT, smart orders. **(See verification status above — none live-fired.)**
- Portfolio operations: exit/convert positions and margin calculators.
- Market data REST: market status, history, quotes, depth, option chain, symbol master.
- EDIS, price alerts, and incoming postback payload models.
- WebSockets: data, order, and TBT/depth protocol models plus async managers.

The scope boundary is the rendered Fyers V3 docs. SDK-only surfaces not found in the rendered docs are tracked separately and are not implemented as public behavior unless the scope changes.

## Install

This crate is not published yet. Use it from a local checkout:

```toml
[dependencies]
fyers-rs = { path = "../fyers-rs" }
```

Feature flags:

| Feature | Default | Purpose |
|---|---:|---|
| `rest` | yes | REST service accessors and typed models. |
| `ws` | yes | WebSocket protocol and manager APIs. |
| `auth` | yes | Auth helpers and token models. |
| `rustls-tls` | yes | Rustls TLS for HTTP/WebSocket clients. |
| `native-tls` | no | Native TLS alternative. |
| `tracing` | yes | Tracing integration points. |
| `live-tests` | no | Ignored live integration tests requiring credentials. |

## Quick start

```rust
use fyers_rs::FyersClient;

# fn main() -> Result<(), fyers_rs::FyersError> {
let client = FyersClient::builder()
    .client_id("APPID-100")
    .access_token("ACCESS_TOKEN")
    .build()?;

let _orders = client.orders();
let _market_data = client.market_data();
let _data_socket = client.data_socket();
# Ok(())
# }
```

REST calls and WebSocket managers are async:

```rust,no_run
use fyers_rs::FyersClient;
use fyers_rs::models::market_data::QuotesRequest;

# async fn run() -> Result<(), fyers_rs::FyersError> {
let client = FyersClient::builder()
    .client_id(std::env::var("FYERS_CLIENT_ID").unwrap())
    .access_token(std::env::var("FYERS_ACCESS_TOKEN").unwrap())
    .build()?;

let quotes = client
    .market_data()
    .quotes(&QuotesRequest {
        symbols: vec!["NSE:SBIN-EQ".to_owned()],
    })
    .await?;

println!("{quotes:?}");
# Ok(())
# }
```

## Auth flow

1. Build the login URL with `auth.generate_auth_code_url(...)`.
2. Redirect the user to Fyers and capture the returned auth code.
3. Validate the auth code with `auth.validate_auth_code(...)`.
4. Store access/refresh tokens securely outside this crate.
5. Refresh tokens with `auth.refresh_access_token(...)` when needed.

See the `examples/auth_*` files for compile-checked auth flow snippets.

## REST example

```rust,no_run
use fyers_rs::FyersClient;
use fyers_rs::models::orders::OrderBookQuery;

# async fn run() -> Result<(), fyers_rs::FyersError> {
let client = FyersClient::builder()
    .client_id(std::env::var("FYERS_CLIENT_ID").unwrap())
    .access_token(std::env::var("FYERS_ACCESS_TOKEN").unwrap())
    .build()?;

let orders = client.orders().list(&OrderBookQuery::default()).await?;
println!("{orders:?}");
# Ok(())
# }
```

Order-placement examples in this repository construct request bodies but do not place live orders by default. **No order-placement path in this crate has been live-fired against a real Fyers account.** Treat any live order call as irreversible broker interaction; verify the wire format yourself before relying on it (see [the order-placement warning above](#%EF%B8%8F-order-placement-has-not-been-tested-live)).

## WebSocket example

```rust,no_run
use fyers_rs::FyersClient;
use fyers_rs::models::ws::{DataSubscribeRequest, DataSubscriptionKind};

# async fn run() -> Result<(), fyers_rs::FyersError> {
let client = FyersClient::builder()
    .client_id(std::env::var("FYERS_CLIENT_ID").unwrap())
    .access_token(std::env::var("FYERS_ACCESS_TOKEN").unwrap())
    .build()?;

let mut socket = client.data_socket().connect().await?;
socket
    .subscribe(&DataSubscribeRequest {
        symbols: vec!["NSE:SBIN-EQ".to_owned()],
        data_type: DataSubscriptionKind::SymbolUpdate,
    })
    .await?;

if let Some(event) = socket.next_event().await? {
    println!("{event:?}");
}

socket.close().await?;
# Ok(())
# }
```

See the WebSocket examples for data and order socket usage patterns.

## Development

Enter the Nix shell:

```sh
nix develop
```

Run the full local validation:

```sh
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --all-features --no-deps
cargo fmt --all -- --check
```

In this repository, use the Nix/OpenSSL wrapper if your host cannot find `libssl.so.3`:

```sh
nix develop -c bash -lc 'export LD_LIBRARY_PATH="$(pkg-config --variable=libdir openssl):$LD_LIBRARY_PATH"; cargo test --all-features'
```

## Live tests

Live tests are ignored by default and gated behind the `live-tests` feature, so normal `cargo test` does not hit the network.

```sh
cargo test --features live-tests --test live_rest -- --ignored --nocapture
cargo test --features live-tests --test live_ws -- --ignored --nocapture
```

Tests never place, modify, or cancel live orders. Order-mutation examples remain outside the test suite and require explicit opt-in flags.

## Scope rule

This crate implements the documented broker API only. It does not include strategy engines, backtesting, persistence, dashboards, multi-broker abstractions, or order-management behavior beyond documented Fyers API calls.
