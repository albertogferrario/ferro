//! `ReservationEvent` — emitted via ferro-events on every state transition
//! (D-25, D-26, D-27).
//!
//! This file is a STUB. Plan 154-04 lands the `#[derive(Clone, Debug,
//! serde::Serialize, serde::Deserialize)]` + `#[serde(rename_all =
//! "snake_case", tag = "kind")]` attributes and the `impl
//! ferro_events::Event for ReservationEvent { fn name }` block.

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum ReservationEvent {
    Held {
        id: Uuid,
        resource_kind: String,
        resource_key: JsonValue,
        window: Option<JsonValue>,
        quantity: u32,
        expires_at: DateTime<Utc>,
    },
    Committed {
        id: Uuid,
        resource_kind: String,
        resource_key: JsonValue,
    },
    Released {
        id: Uuid,
        resource_kind: String,
        resource_key: JsonValue,
        reason: ReleaseReason,
    },
    Expired {
        id: Uuid,
        resource_kind: String,
        resource_key: JsonValue,
    },
}

/// Reason recorded on the audit log + emitted in `ReservationEvent::Released`
/// (D-18). Serde-derived in plan 154-04 with
/// `#[serde(rename_all = "snake_case", tag = "reason")]`.
#[derive(Clone, Debug)]
pub enum ReleaseReason {
    UserCancelled,
    PaymentFailed,
    AdminOverride,
    Other(String),
}
