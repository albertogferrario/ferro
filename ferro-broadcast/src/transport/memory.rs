//! In-memory shared transport (test/dev). Two `Broadcaster`s sharing one
//! `Arc<InMemoryTransport>` observe each other's published envelopes — the
//! deterministic CI substitute for a live Redis bus (D-11).

use crate::transport::{BroadcastTransport, BusEnvelope};
use crate::Error;
use tokio::sync::broadcast;
use tracing::warn;

/// In-memory fan-out bus. Clone/`Arc`-share to connect multiple `Broadcaster`s.
pub struct InMemoryTransport {
    tx: broadcast::Sender<BusEnvelope>,
}

impl InMemoryTransport {
    /// Create a bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }
}

#[async_trait::async_trait]
impl BroadcastTransport for InMemoryTransport {
    async fn publish(&self, envelope: &BusEnvelope) -> Result<(), Error> {
        // `send` errors only when there are no receivers — that is not a failure.
        let _ = self.tx.send(envelope.clone());
        Ok(())
    }

    async fn subscribe_loop(
        &self,
        sink: tokio::sync::mpsc::Sender<BusEnvelope>,
    ) -> Result<(), Error> {
        let mut rx = self.tx.subscribe();
        loop {
            match rx.recv().await {
                Ok(envelope) => {
                    if sink.send(envelope).await.is_err() {
                        break; // receiver dropped — clean shutdown
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(dropped = n, "in-memory broadcast bus lagged");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
        Ok(())
    }
}
