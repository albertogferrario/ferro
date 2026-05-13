//! `Resource` — consumer-implemented capacity model.
//!
//! This file declares the trait. Plan 154-04 adds the rustdoc example
//! showing a consumer impl + a `#[cfg(test)]` block exercising the trait
//! via an inline `TestResource` against in-memory SQLite.

#![allow(dead_code)]

use async_trait::async_trait;
use sea_orm::ConnectionTrait;
use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;

use crate::error::ReservationError;

/// Consumer-implemented capacity model (D-05).
///
/// `Key` identifies a resource instance; `Window` scopes capacity to a
/// time range, seat category, or any other dimension. Use `Window = ()`
/// for non-windowed resources (atomic counters, simple capacity).
///
/// `KIND` is a `&'static str` const (D-08) — dotted-namespace convention:
/// `"inventory.unit"`, `"checkout.slot"`, `"api.quota"`.
#[async_trait]
pub trait Resource: Send + Sync + 'static {
    type Key: Hash + Eq + Clone + Send + Sync + Serialize + DeserializeOwned;
    type Window: PartialEq + Clone + Send + Sync + Serialize + DeserializeOwned;

    const KIND: &'static str;

    async fn capacity<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        window: &Self::Window,
    ) -> Result<u32, ReservationError>;

    async fn held<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        window: &Self::Window,
    ) -> Result<u32, ReservationError>;
}
