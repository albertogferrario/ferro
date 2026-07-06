//! `ReservationError` — the single error type for the ferro-reservation crate.
//!
//! Every variant's `Display` impl prefixes `"reservation: …"` so production
//! log greps stay surgical (matches `"guarded: …"`, `"audit: …"`, `"config: …"`).

use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ReservationError {
    /// `hold` was called for `quantity` units but the resource has fewer
    /// `available` (= `capacity - held`) units left. The three values
    /// surface to telemetry and UI ("3 units left, you asked for 5").
    #[error("reservation: insufficient capacity (requested {requested}, available {available} of {capacity})")]
    Insufficient {
        requested: u32,
        available: u32,
        capacity: u32,
    },

    /// State-transition predicate failed — the row was not in the expected
    /// state at update time (already committed/released/expired by a
    /// concurrent caller or the sweeper, or never existed). The kernel
    /// maps `GuardedError::NoRowsAffected` to this variant explicitly
    /// before the `?` operator in every state-transition method (D-46).
    #[error("reservation: id={id} not in expected state '{expected}'")]
    ConflictingState { id: Uuid, expected: &'static str },

    /// Reservation row was not found by id (rare; usually surfaces only
    /// in introspection / debug paths, not in the standard
    /// hold/commit/release flow).
    #[error("reservation: id={id} not found")]
    NotFound { id: Uuid },

    /// Underlying SeaORM database error.
    #[error("reservation: db error: {0}")]
    Db(#[from] sea_orm::DbErr),

    /// `ferro-orm` guarded update error other than `NoRowsAffected`
    /// (which the kernel intercepts and maps to `ConflictingState`).
    /// `EmptyUpdate` and `TooManyRows` are programming bugs and surface
    /// through this variant for visibility.
    #[error("reservation: guarded update error: {0}")]
    Guarded(#[from] ferro_orm::GuardedError),

    /// `ferro-audit` write error. Per D-30, audit failure does NOT roll
    /// back the state transition — the DB row is already updated; this
    /// error surfaces so consumers can alarm on it.
    #[error("reservation: audit error: {0}")]
    Audit(#[from] ferro_audit::AuditError),

    /// JSON serialization / deserialization error on `Resource::Key` /
    /// `Resource::Window` round-trips against the persisted JSON columns.
    #[error("reservation: json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn insufficient_display() {
        let e = ReservationError::Insufficient {
            requested: 5,
            available: 3,
            capacity: 5,
        };
        assert_eq!(
            e.to_string(),
            "reservation: insufficient capacity (requested 5, available 3 of 5)"
        );
    }

    #[test]
    fn conflicting_state_display() {
        let id = Uuid::new_v4();
        let e = ReservationError::ConflictingState {
            id,
            expected: "held",
        };
        let s = e.to_string();
        assert!(s.starts_with("reservation: id="), "got: {s}");
        assert!(s.ends_with(" not in expected state 'held'"), "got: {s}");
    }

    #[test]
    fn not_found_display() {
        let id = Uuid::new_v4();
        let e = ReservationError::NotFound { id };
        let s = e.to_string();
        assert!(s.starts_with("reservation: id="), "got: {s}");
        assert!(s.ends_with(" not found"), "got: {s}");
    }

    #[test]
    fn db_from_sea_orm_dberr() {
        let db_err = sea_orm::DbErr::Custom("test".into());
        let e: ReservationError = ReservationError::from(db_err);
        assert!(matches!(e, ReservationError::Db(_)));
        assert!(e.to_string().starts_with("reservation: db error: "));
    }

    #[test]
    fn guarded_from_ferro_orm_error() {
        let g = ferro_orm::GuardedError::NoRowsAffected;
        let e: ReservationError = ReservationError::from(g);
        assert!(matches!(e, ReservationError::Guarded(_)));
        assert!(e
            .to_string()
            .starts_with("reservation: guarded update error: "));
    }

    #[test]
    fn audit_from_ferro_audit_error() {
        let a = ferro_audit::AuditError::MissingAction;
        let e: ReservationError = ReservationError::from(a);
        assert!(matches!(e, ReservationError::Audit(_)));
        assert!(e.to_string().starts_with("reservation: audit error: "));
    }

    #[test]
    fn json_from_serde_json_error() {
        let j: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let e: ReservationError = ReservationError::from(j);
        assert!(matches!(e, ReservationError::Json(_)));
        assert!(e
            .to_string()
            .starts_with("reservation: json serialization error: "));
    }
}
