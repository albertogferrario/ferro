//! `ProjectionError` — the single error type for the ferro-projection crate.
//!
//! Every variant's `Display` impl prefixes `"projection: …"` so production
//! log greps stay surgical (matches `"guarded: …"`, `"audit: …"`,
//! `"reservation: …"`, `"config: …"`).

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    /// Underlying SeaORM database error (snapshot upsert, find, delete).
    #[error("projection: db error: {0}")]
    Db(#[from] sea_orm::DbErr),

    /// JSON serialization / deserialization error on `P::State`
    /// round-trips against the persisted JSON column.
    #[error("projection: json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Broadcast publish error. Stringly-typed because
    /// `ferro_broadcast::Error` doesn't compose cleanly through
    /// `thiserror::From` — same pragmatic choice Phase 149 made for
    /// `ferro_notifications::Error::Broadcast(String)`.
    ///
    /// Per D-21: broadcast failure does NOT roll back state — the
    /// snapshot row is already persisted. Surfaces this error so
    /// consumers can alarm on it; subscribers reconcile by re-reading
    /// the snapshot.
    #[error("projection: broadcast error: {0}")]
    Broadcast(String),

    /// Event-bus error surfaced from `ferro_events::Error`. Hand-rolled
    /// `From` impl mirrors `Broadcast` above (D-29).
    #[error("projection: events error: {0}")]
    Events(String),

    /// Reserved for an explicit `read_required` helper if a consumer
    /// wants `Result<State, _>` rather than `Result<Option<State>, _>`.
    /// Not used by `read` itself (which returns `Result<Option<_>, _>`).
    #[error("projection: state not found for {name}/{key}")]
    StateNotFound { name: &'static str, key: String },
}

impl From<ferro_broadcast::Error> for ProjectionError {
    fn from(e: ferro_broadcast::Error) -> Self {
        Self::Broadcast(e.to_string())
    }
}

impl From<ferro_events::Error> for ProjectionError {
    fn from(e: ferro_events::Error) -> Self {
        Self::Events(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_from_sea_orm_dberr() {
        let db_err = sea_orm::DbErr::Custom("test".into());
        let e: ProjectionError = ProjectionError::from(db_err);
        assert!(matches!(e, ProjectionError::Db(_)));
        assert!(e.to_string().starts_with("projection: db error: "));
    }

    #[test]
    fn json_from_serde_json_error() {
        let j: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let e: ProjectionError = ProjectionError::from(j);
        assert!(matches!(e, ProjectionError::Json(_)));
        assert!(e.to_string().starts_with("projection: json error: "));
    }

    #[test]
    fn broadcast_display() {
        let e = ProjectionError::Broadcast("oops".into());
        assert_eq!(e.to_string(), "projection: broadcast error: oops");
    }

    #[test]
    fn events_display() {
        let e = ProjectionError::Events("oops".into());
        assert_eq!(e.to_string(), "projection: events error: oops");
    }

    #[test]
    fn state_not_found_display() {
        let e = ProjectionError::StateNotFound {
            name: "inventory.dashboard",
            key: "warehouse-a".into(),
        };
        assert_eq!(
            e.to_string(),
            "projection: state not found for inventory.dashboard/warehouse-a"
        );
    }
}
