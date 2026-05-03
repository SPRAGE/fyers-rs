//! WebSocket service accessors and manager types.
//!
//! Data, order, and TBT/depth sockets expose async managers for documented
//! connect, send, receive, ping, and close behavior. Frame parsing is delegated
//! to typed protocol models in [`crate::models::ws`].

pub mod data;
pub mod manager;
pub mod order;
pub mod protocol;
pub mod tbt;

pub use data::{DataSocketConnection, DataSocketService, LiveDataSocketConnection};
pub use manager::{LiveWebSocket, ManagedSocket, ReconnectPolicy};
pub use order::{LiveOrderSocketConnection, OrderSocketConnection, OrderSocketService};
pub use tbt::{LiveTbtSocketConnection, TbtSocketConnection, TbtSocketService};
