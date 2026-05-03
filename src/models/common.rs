//! Common response models shared across Fyers APIs.

use serde::{Deserialize, Serialize};

/// Common Fyers response status envelope fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiStatus {
    /// Broker-specific numeric code.
    pub code: i64,
    /// Broker status string, commonly `ok` or `error`.
    pub s: String,
    /// Human-readable broker message.
    pub message: String,
}
