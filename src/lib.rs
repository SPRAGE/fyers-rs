//! Idiomatic async Rust client for the documented Fyers broker APIs.
//!
//! The crate is organized around a single [`FyersClient`] with typed accessors
//! for auth, REST, and WebSocket surfaces. Implemented coverage is tracked in
//! a local Fyers docs inventory and validated with fixture-backed contract tests
//! plus mock transport/socket tests.
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
//! under [`models`]. WebSocket managers are exposed through [`ws`] and separate
//! manager I/O from frame parsing so the protocol boundary remains testable.
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
