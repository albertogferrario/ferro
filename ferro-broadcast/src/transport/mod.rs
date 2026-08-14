//! Shared fan-out transport abstraction for cross-process broadcast delivery.

use crate::message::ServerMessage;
use crate::Error;
use serde::{Deserialize, Serialize};

pub mod memory;

#[cfg(feature = "redis-transport")]
pub mod redis;

/// Envelope carried on the shared bus for cross-process delivery (D-10).
///
/// Wraps the existing `ServerMessage` (already `Serialize + Deserialize`) with the
/// routing metadata each replica needs: the logical channel name and the
/// process-unique origin id used for echo suppression (D-03, D-08).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusEnvelope {
    /// Process-unique origin id (uuid v4) of the publishing `Broadcaster`.
    pub origin: String,
    /// Logical ferro-broadcast channel name (e.g. `projection.orders.42`).
    pub channel: String,
    /// The server-side message payload (only `ServerMessage::Event` is fanned out, D-04).
    pub message: ServerMessage,
}

/// A shared fan-out transport: publishes envelopes to a bus reachable by other
/// processes and delivers received envelopes back into local subscriber routing.
///
/// Object-safe so `Broadcaster` can hold `Option<Arc<dyn BroadcastTransport + Send + Sync>>`.
#[async_trait::async_trait]
pub trait BroadcastTransport: Send + Sync {
    /// Publish an envelope to the shared bus.
    ///
    /// Implementations MUST NOT fail the caller on a transient bus error (D-06);
    /// the `Broadcaster` logs the returned error and continues, but implementations
    /// should also degrade gracefully internally where possible.
    async fn publish(&self, envelope: &BusEnvelope) -> Result<(), Error>;

    /// Run the background SUBSCRIBE loop, forwarding every received envelope to
    /// `sink`. Returns when `sink` is dropped (clean shutdown) or on a fatal error.
    /// Origin filtering (echo suppression) is performed by the caller, not here.
    async fn subscribe_loop(
        &self,
        sink: tokio::sync::mpsc::Sender<BusEnvelope>,
    ) -> Result<(), Error>;
}
