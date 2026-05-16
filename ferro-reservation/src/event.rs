//! `ReservationEvent` (D-25) + `ReleaseReason` (D-18).
//!
//! `ReservationEvent` is dispatched via [`ferro_events::dispatch`] AFTER
//! every successful state transition (D-26). Subscribers re-deserialize
//! `resource_key` / `window` against their typed `Resource::Key` /
//! `Resource::Window` if needed; at the event-bus boundary the typed key
//! becomes opaque JSON (the kernel is generic over `R: Resource`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
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

impl ferro_events::Event for ReservationEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Held { .. } => "ReservationHeld",
            Self::Committed { .. } => "ReservationCommitted",
            Self::Released { .. } => "ReservationReleased",
            Self::Expired { .. } => "ReservationExpired",
        }
    }
}

/// Reason recorded on the audit log + emitted in
/// [`ReservationEvent::Released`] (D-18). Serde-derived with
/// `#[serde(rename_all = "snake_case")]`. Unit variants serialize as plain
/// strings (e.g. `"user_cancelled"`); `Other(String)` serializes as
/// `{"other": "…"}`. The `tag = "reason"` form is incompatible with newtype
/// variants containing a plain string value in serde's internal-tag format.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseReason {
    UserCancelled,
    PaymentFailed,
    AdminOverride,
    /// Free-form reason recorded as `{"reason": "other", ...}`. Use this
    /// for app-specific reasons not captured by the closed variants.
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_events::Event;
    use serde_json::json;

    #[test]
    fn event_name_per_variant() {
        let held = ReservationEvent::Held {
            id: Uuid::new_v4(),
            resource_kind: "inventory.unit".into(),
            resource_key: json!({"k": "v"}),
            window: None,
            quantity: 1,
            expires_at: Utc::now(),
        };
        assert_eq!(held.name(), "ReservationHeld");

        let committed = ReservationEvent::Committed {
            id: Uuid::new_v4(),
            resource_kind: "inventory.unit".into(),
            resource_key: json!({}),
        };
        assert_eq!(committed.name(), "ReservationCommitted");

        let released = ReservationEvent::Released {
            id: Uuid::new_v4(),
            resource_kind: "inventory.unit".into(),
            resource_key: json!({}),
            reason: ReleaseReason::UserCancelled,
        };
        assert_eq!(released.name(), "ReservationReleased");

        let expired = ReservationEvent::Expired {
            id: Uuid::new_v4(),
            resource_kind: "inventory.unit".into(),
            resource_key: json!({}),
        };
        assert_eq!(expired.name(), "ReservationExpired");
    }

    #[test]
    fn event_serde_round_trip_held() {
        let id = Uuid::new_v4();
        let expires = Utc::now();
        let original = ReservationEvent::Held {
            id,
            resource_kind: "inventory.unit".into(),
            resource_key: json!({"product": "abc"}),
            window: Some(json!({"date": "2026-05-13"})),
            quantity: 3,
            expires_at: expires,
        };
        let s = serde_json::to_string(&original).expect("serialize");
        assert!(s.contains(r#""kind":"held""#), "got: {s}");
        let decoded: ReservationEvent = serde_json::from_str(&s).expect("deserialize");
        match decoded {
            ReservationEvent::Held {
                id: did,
                resource_kind,
                resource_key,
                window,
                quantity,
                expires_at,
            } => {
                assert_eq!(did, id);
                assert_eq!(resource_kind, "inventory.unit");
                assert_eq!(resource_key, json!({"product": "abc"}));
                assert_eq!(window, Some(json!({"date": "2026-05-13"})));
                assert_eq!(quantity, 3);
                assert_eq!(expires_at, expires);
            }
            other => panic!("expected Held, got {other:?}"),
        }
    }

    #[test]
    fn release_reason_serde_round_trip_all_variants() {
        for variant in [
            ReleaseReason::UserCancelled,
            ReleaseReason::PaymentFailed,
            ReleaseReason::AdminOverride,
            ReleaseReason::Other("custom reason".into()),
        ] {
            let s = serde_json::to_string(&variant).expect("serialize");
            let decoded: ReleaseReason = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(decoded, variant);
        }
    }
}
