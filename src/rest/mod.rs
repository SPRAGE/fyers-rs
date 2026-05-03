//! REST API service accessors.
//!
//! Each module exposes a lightweight service borrowed from [`crate::FyersClient`].
//! Methods use typed request/response structs from [`crate::models`] and apply
//! the documented `Authorization: app_id:access_token` header for authenticated
//! broker calls.

pub mod alerts;
pub mod edis;
pub mod funds;
pub mod gtt;
pub mod holdings;
pub mod market_data;
pub mod orders;
pub mod positions;
pub mod profile;
pub mod reports;
pub mod smart_orders;
pub mod trades;
