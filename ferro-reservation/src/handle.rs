//! `ReservationHandle` — opaque token returned by `ReservationKernel::hold`
//! (D-34, D-35).
//!
//! Carries the persisted row's `id` plus a full snapshot of hold-time
//! fields. Pass the handle to `commit`, `release`, or `extend`. The
//! struct is `Serialize + Deserialize` so callers can embed it in Stripe
//! payment intent metadata, a queued-job payload, or any other side-channel.
//!
//! `correlation_id` is NOT carried on the handle (D-35) — that lives in
//! `ReservationContext`, which is per-call. A reservation can have a
//! different actor at commit time (e.g. Stripe webhook system actor) than
//! at hold time (user actor).

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReservationHandle {
    pub id: Uuid,
    pub resource_kind: String,
    pub resource_key: JsonValue,
    pub window: Option<JsonValue>,
    pub quantity: u32,
    pub held_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub tenant_id: Option<String>,
}
