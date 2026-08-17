//! Redis pub/sub shared transport (feature `redis-transport`).
//!
//! Single fan-out channel with per-process local routing (D-08). PUBLISH uses a
//! multiplexed `ConnectionManager` (auto-reconnect, D-07); SUBSCRIBE uses a
//! dedicated pub/sub connection because a pub/sub connection cannot also issue
//! commands (D-09).
//!
//! # Deployment trust boundary (T-246.1-06)
//!
//! The Redis instance is assumed to be a trusted, network-isolated resource shared
//! with `ferro-cache` under the same `REDIS_URL`. A compromised Redis can inject or
//! observe deltas. No in-code envelope signing is applied in this version; operators
//! must restrict network access to the Redis instance. This is an accepted deployment
//! assumption, consistent with how `ferro-cache` treats the same bus.
#![cfg(feature = "redis-transport")]

use crate::transport::{BroadcastTransport, BusEnvelope};
use crate::Error;
use futures_util::StreamExt;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use tracing::{debug, warn};

/// App-neutral default fan-out channel. No tenant or application identity
/// (project-agnostic-crate rule).
pub const DEFAULT_CHANNEL: &str = "ferro:broadcast";

/// Redis-backed shared fan-out transport.
///
/// Holds both a `ConnectionManager` for PUBLISH (cloneable, multiplexed,
/// auto-reconnect) and the original `Client` for SUBSCRIBE (`get_async_pubsub()`
/// lives on `Client`, not on `ConnectionManager`).
pub struct RedisTransport {
    manager: ConnectionManager, // PUBLISH — cloneable, multiplexed
    client: Client,             // SUBSCRIBE — get_async_pubsub()
    channel: String,            // single fan-out channel (D-08)
}

impl RedisTransport {
    /// Connect and build the transport. `channel` defaults to `DEFAULT_CHANNEL`
    /// when the empty string is passed; callers should prefer [`connect`](Self::connect)
    /// for the common case.
    pub async fn new(url: &str, channel: impl Into<String>) -> Result<Self, Error> {
        let client = Client::open(url).map_err(|e| Error::transport(e.to_string()))?;
        let manager = ConnectionManager::new(client.clone())
            .await
            .map_err(|e| Error::transport(e.to_string()))?;
        let channel = channel.into();
        debug!(channel = %channel, "redis broadcast transport connected");
        Ok(Self {
            manager,
            client,
            channel,
        })
    }

    /// Connect using the app-neutral default channel (`ferro:broadcast`).
    pub async fn connect(url: &str) -> Result<Self, Error> {
        Self::new(url, DEFAULT_CHANNEL).await
    }
}

#[async_trait::async_trait]
impl BroadcastTransport for RedisTransport {
    async fn publish(&self, envelope: &BusEnvelope) -> Result<(), Error> {
        let payload = serde_json::to_string(envelope)?;
        let mut conn = self.manager.clone(); // ConnectionManager is Clone — one clone per call
        conn.publish::<_, _, ()>(&self.channel, payload).await?; // ? uses Error::Redis via #[from]
        Ok(())
    }

    async fn subscribe_loop(
        &self,
        sink: tokio::sync::mpsc::Sender<BusEnvelope>,
        ready: tokio::sync::oneshot::Sender<()>,
    ) -> Result<(), Error> {
        // Dedicated pub/sub connection (D-09) — cannot share the multiplexed manager.
        let mut pubsub = self.client.get_async_pubsub().await?;
        pubsub.subscribe(&self.channel).await?; // subscription established
        let _ = ready.send(()); // D-01: signal before entering receive loop; Err = caller timed out
        let mut stream = pubsub.into_on_message();
        while let Some(msg) = stream.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    warn!(error = %e, "dropping unreadable redis pub/sub payload");
                    continue; // never panic on a malformed payload
                }
            };
            // Strict deserialization: a malformed or hostile envelope is dropped,
            // never partially delivered (threat T-246.1-03).
            match serde_json::from_str::<BusEnvelope>(&payload) {
                Ok(envelope) => {
                    if sink.send(envelope).await.is_err() {
                        break; // receiver dropped — clean shutdown
                    }
                }
                Err(e) => {
                    warn!(error = %e, "dropping malformed bus envelope from redis");
                    continue;
                }
            }
        }
        Ok(())
    }
}
