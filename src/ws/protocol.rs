//! Shared WebSocket protocol helpers.
//!
//! Concrete command/event models live in [`crate::models::ws`]. Socket managers
//! use [`SocketKind`] to select the documented data, order, or TBT endpoint and
//! to apply the matching frame parser.

/// Broad Fyers socket family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketKind {
    /// Market data socket.
    Data,
    /// Order/trade/position socket.
    Order,
    /// TBT/depth socket, if confirmed as documented.
    Tbt,
}
