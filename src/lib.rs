//! Idiomatic async Rust client for the documented Fyers broker APIs.
//!
//! The crate is organized around a single [`FyersClient`] with typed accessors
//! for auth, REST, and WebSocket surfaces. The market-data WebSocket talks the
//! real Fyers V3 binary protocol (auth handshake, channel-mode set, channel
//! resume, HSM-token-resolved subscribe, `ack` flow control) — see
//! [`ws::DataSocketConnection`] and [`ws::data_protocol`].
//!
//! # Quick start
//!
//! ```
//! use fyers_rs::FyersClient;
//!
//! let client = FyersClient::builder()
//!     .client_id("APPID-100")
//!     .access_token("ACCESS_TOKEN")
//!     .build()?;
//!
//! let _orders = client.orders();
//! let _market_data = client.market_data();
//! let _data_socket = client.data_socket();
//! # Ok::<(), fyers_rs::FyersError>(())
//! ```
//!
//! # REST and WebSocket APIs
//!
//! REST services live under [`rest`] and typed request/response/event models live
//! under [`models`]. WebSocket managers are exposed through [`ws`].
//!
//! # Verification status
//!
//! The market-data WebSocket and the profile/funds/holdings REST endpoints
//! have been live-verified end-to-end. **REST order placement, modification,
//! and cancellation paths have never been live-fired** against a real Fyers
//! account — the URLs and payload shapes match the official Python SDK, but
//! a round-trip live order has not yet been performed through this crate.
//! See the README's verification matrix and the per-module status notes.
//!
//! # Warning
//!
//! This project is entirely AI-generated at this stage. It has not been
//! independently audited or battle-tested. Review, test, and verify all behavior
//! yourself before using it with real Fyers credentials, market data, or orders.

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]

pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod rest;
pub mod transport;
pub mod ws;

pub use client::{FyersClient, FyersClientBuilder};
pub use config::{FyersConfig, SecretString};
pub use error::{FyersError, Result};

pub use rest::{
    alerts, edis, funds, gtt, holdings, market_data, orders, positions, profile, reports,
    smart_orders, trades,
};
