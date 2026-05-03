//! Typed request/response/event models for documented Fyers APIs.
//!
//! Models are intentionally close to the documented wire format, using serde
//! renames where Fyers fields are camelCase or otherwise broker-specific.

pub mod alerts;
pub mod auth;
pub mod common;
pub mod edis;
pub mod gtt;
pub mod margin;
pub mod market_data;
pub mod orders;
pub mod portfolio;
pub mod postback;
pub mod reports;
pub mod smart_orders;
pub mod transactions;
pub mod user;
pub mod ws;
