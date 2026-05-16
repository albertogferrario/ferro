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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use serde_json::json;

    #[test]
    fn handle_serde_round_trips() {
        let id = Uuid::new_v4();
        let held_at = Utc::now();
        let expires_at = held_at + Duration::seconds(900);
        let original = ReservationHandle {
            id,
            resource_kind: "inventory.unit".into(),
            resource_key: json!({"product": "abc", "tenant": "t1"}),
            window: Some(json!({"date": "2026-05-13", "slot": "morning"})),
            quantity: 3,
            held_at,
            expires_at,
            tenant_id: Some("t1".into()),
        };

        let s = serde_json::to_string(&original).expect("serialize");
        let decoded: ReservationHandle = serde_json::from_str(&s).expect("deserialize");

        assert_eq!(decoded.id, id);
        assert_eq!(decoded.resource_kind, "inventory.unit");
        assert_eq!(
            decoded.resource_key,
            json!({"product": "abc", "tenant": "t1"})
        );
        assert_eq!(
            decoded.window,
            Some(json!({"date": "2026-05-13", "slot": "morning"}))
        );
        assert_eq!(decoded.quantity, 3);
        assert_eq!(decoded.held_at, held_at);
        assert_eq!(decoded.expires_at, expires_at);
        assert_eq!(decoded.tenant_id.as_deref(), Some("t1"));
    }

    #[test]
    fn handle_serde_round_trips_with_no_window_no_tenant() {
        let original = ReservationHandle {
            id: Uuid::new_v4(),
            resource_kind: "api.quota".into(),
            resource_key: json!("client_abc"),
            window: None,
            quantity: 1,
            held_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(60),
            tenant_id: None,
        };
        let s = serde_json::to_string(&original).expect("serialize");
        let decoded: ReservationHandle = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(decoded.window, None);
        assert_eq!(decoded.tenant_id, None);
    }
}
