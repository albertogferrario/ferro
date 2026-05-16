//! `Projection` — consumer-implemented live read-model contract (D-06).
//!
//! **Not to be confused with `ferro-projections` (plural).** That crate
//! is the Service Projection abstraction (`ServiceDef → IntentGraph →
//! JsonUiRenderer`). This trait is the live-read-model contract: fold
//! domain events into a per-key state, return a delta per apply, let
//! the runtime persist and broadcast.
//!
//! ## Authoring a Projection
//!
//! ```rust,ignore
//! use ferro_projection::{Projection, ProjectionKey, ProjectionRuntime};
//! use ferro_events::Event;
//! use ferro_broadcast::Broadcaster;
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! // Consumer event (already implements ferro_events::Event).
//! #[derive(Clone, Serialize, Deserialize)]
//! struct InventoryAdjusted { warehouse: String, sku: String, delta: i32 }
//!
//! impl Event for InventoryAdjusted {
//!     fn name(&self) -> &'static str { "InventoryAdjusted" }
//! }
//!
//! // Consumer projection state + delta.
//! #[derive(Clone, Default, Serialize, Deserialize)]
//! struct WarehouseDashboard {
//!     totals: std::collections::HashMap<String, i64>,
//! }
//!
//! #[derive(Clone, Serialize)]
//! struct WarehouseDelta { sku: String, new_total: i64 }
//!
//! // Consumer projection impl.
//! struct WarehouseProjection;
//!
//! impl Projection for WarehouseProjection {
//!     type Event = InventoryAdjusted;
//!     type State = WarehouseDashboard;
//!     type Delta = WarehouseDelta;
//!
//!     const NAME: &'static str = "inventory.dashboard";
//!
//!     fn key(&self, event: &Self::Event) -> ProjectionKey {
//!         ProjectionKey::new(event.warehouse.clone())
//!     }
//!
//!     fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
//!         let new_total = state.totals.entry(event.sku.clone()).or_insert(0);
//!         *new_total += event.delta as i64;
//!         WarehouseDelta { sku: event.sku.clone(), new_total: *new_total }
//!     }
//! }
//!
//! // Application setup (one-line wiring):
//! let runtime = Arc::new(ProjectionRuntime::new(
//!     db.clone(),
//!     broadcaster.clone(),
//!     WarehouseProjection,
//! ));
//! runtime.clone().register();
//!
//! // Anywhere in the app:
//! InventoryAdjusted { warehouse: "a".into(), sku: "sku-1".into(), delta: 5 }
//!     .dispatch()
//!     .await?;
//!
//! // Frontend subscribes to `projection.inventory.dashboard.a` and
//! // receives event `"delta"` with payload `{ "sku": "sku-1", "new_total": 5 }`.
//! ```
//!
//! ## Naming conventions
//!
//! - `NAME`: dotted namespace — `"inventory.dashboard"`,
//!   `"checkout.cart"`, `"orders.recent"`. Same convention as
//!   `ferro_audit`'s action namespace and `ferro_reservation::Resource::KIND`.
//! - `key()`: stringify any compound key — multi-tenancy lives in the
//!   key string (`"tenant-7:warehouse-a"`).

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::key::ProjectionKey;

/// Consumer-implemented live read-model (D-06).
///
/// Implementations are usually unit structs because the projection's
/// behaviour is encoded in `apply`. Carry state only if `apply` needs
/// configuration (e.g., a tenant-id allowlist).
pub trait Projection: Send + Sync + 'static {
    /// The domain event the projection folds. Must be a
    /// `ferro_events::Event` (`Clone + Send + Sync + 'static + name()`)
    /// plus Serde-round-trippable for `rebuild` from persisted event
    /// streams (D-09).
    type Event: ferro_events::Event + Serialize + DeserializeOwned;

    /// The materialized read-model. `Default` is required (D-07) so
    /// the runtime can initialize a fresh key without an explicit
    /// `Option<State>` first-apply path.
    type State: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static;

    /// The per-apply change broadcast to subscribers. Consumer's
    /// choice: full state, JSON Patch, a minimal struct, anything
    /// `Serialize`. Small payloads are friendlier to high-frequency
    /// streams; full-state deltas are simpler when frequency is low
    /// (D-10).
    type Delta: Serialize + Clone + Send + Sync + 'static;

    /// Dotted-namespace identifier (D-06). Persisted to
    /// `projection_snapshots.projection_name`. MUST be unique across
    /// all `Projection` impls in a single application.
    const NAME: &'static str;

    /// Derive the per-row key from the event (D-12). The runtime
    /// serializes apply per key.
    fn key(&self, event: &Self::Event) -> ProjectionKey;

    /// Fold the event into the running state and return the delta
    /// (D-08). Pure function — MUST NOT perform IO or block. The
    /// runtime calls this inside a per-key `tokio::sync::Mutex`;
    /// long-running work blocks ALL events for that key.
    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta;

    /// Reserved for v0.x event-log-backed snapshot mode (D-25). v0
    /// ignores this — every apply persists. Default returns 100 so
    /// v0.x can flip on this method without breaking consumers who
    /// never overrode it.
    fn snapshot_interval(&self) -> u32 {
        100
    }

    /// Event name used for the broadcast frame (D-39). Defaults to
    /// `"delta"`. Consumers can override if their frontend dispatches
    /// on the event name (`"dashboard_updated"`, `"cart_changed"`).
    fn broadcast_event_name(&self) -> &'static str {
        "delta"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    struct TestEvent {
        value: i32,
    }

    impl ferro_events::Event for TestEvent {
        fn name(&self) -> &'static str {
            "TestEvent"
        }
    }

    #[derive(Clone, Default, Serialize, Deserialize)]
    struct TestState {
        total: i64,
    }

    #[derive(Clone, Serialize)]
    struct TestDelta {
        new_total: i64,
    }

    struct TestProjection;

    impl Projection for TestProjection {
        type Event = TestEvent;
        type State = TestState;
        type Delta = TestDelta;

        const NAME: &'static str = "test.projection";

        fn key(&self, _event: &Self::Event) -> ProjectionKey {
            ProjectionKey::new("test-key")
        }

        fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
            state.total += event.value as i64;
            TestDelta {
                new_total: state.total,
            }
        }
    }

    #[test]
    fn snapshot_interval_default_is_100() {
        assert_eq!(TestProjection.snapshot_interval(), 100);
    }

    #[test]
    fn broadcast_event_name_default_is_delta() {
        assert_eq!(TestProjection.broadcast_event_name(), "delta");
    }
}
