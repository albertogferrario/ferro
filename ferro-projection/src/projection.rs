//! `Projection` — consumer-implemented live read-model.
//!
//! This file is a STUB. Plan 155-04 adds the full rustdoc example
//! showing a consumer impl + the disambiguation paragraph (D-51 final
//! pass) + a `#[cfg(test)]` block exercising trait method defaults.

#![allow(dead_code)]

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::key::ProjectionKey;

/// Consumer-implemented live read-model (D-06).
///
/// `Event` is the domain event the projection folds; `State` is the
/// materialized read-model; `Delta` is the per-apply change broadcast
/// to subscribers. `NAME` is a `&'static str` const — dotted-namespace
/// convention: `"inventory.dashboard"`, `"checkout.cart"`.
///
/// **Not to be confused with `ferro-projections` (plural).** That crate
/// is the Service Projection abstraction (`ServiceDef → IntentGraph →
/// JsonUiRenderer`). This trait is the live-read-model contract.
pub trait Projection: Send + Sync + 'static {
    type Event: ferro_events::Event + Serialize + DeserializeOwned;
    type State: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static;
    type Delta: Serialize + Clone + Send + Sync + 'static;

    const NAME: &'static str;

    fn key(&self, event: &Self::Event) -> ProjectionKey;

    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta;

    fn snapshot_interval(&self) -> u32 {
        100
    }

    fn broadcast_event_name(&self) -> &'static str {
        "delta"
    }
}
