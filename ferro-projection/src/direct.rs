//! Direct snapshot write/read API for `projection_snapshots`, decoupled from
//! the event-fold [`Projection`] trait.
//!
//! Two free functions — [`snapshot_write`] and [`snapshot_read`] — operate
//! over the existing `projection_snapshots` entity using the same SeaORM
//! upsert idiom as `apply_event`, but keyed by an arbitrary `(name, key)` with
//! no event, no `Default` state, and no incremental fold.
//!
//! **Purpose (D-01/D-02):** An offload result is a one-shot, arbitrary-`T`
//! value keyed by a UUID. It has no domain event to fold, so it does not fit
//! the `Projection` event-fold contract. This module provides the persistence
//! primitive the framework facade composes into `persist_result` /
//! `persist_error` / `read_result`.
//!
//! **Version semantics:** `version` is fixed at `1` on every write. The
//! `OnConflict` update clause omits `Column::Version`, so repeat writes do
//! not overwrite the version field. For one-shot results there is a single
//! writer per key, so version tracking adds no operational value (D-02).
//!
//! **Upsert semantics:** last-writer-wins on the composite PK
//! `(projection_name, key)`. Concurrent same-key writes succeed without
//! error (accepted per T-246-04).
//!
//! [`Projection`]: crate::projection::Projection

use chrono::Utc;
use sea_orm::{sea_query::OnConflict, ActiveValue, DatabaseConnection, EntityTrait};
use serde_json::Value as JsonValue;

use crate::entity::{ActiveModel, Column, Entity};
use crate::error::ProjectionError;
use crate::key::ProjectionKey;

/// Write a snapshot directly, bypassing the event-fold `Projection` trait.
///
/// Uses an upsert (`OnConflict` on the composite PK `(projection_name, key)`)
/// so repeat writes are idempotent and last-writer-wins. `version` is fixed at
/// `1` for one-shot values; re-writes do not change the version field.
///
/// # Errors
///
/// Returns [`ProjectionError::Db`] on a SeaORM error.
pub async fn snapshot_write(
    db: &DatabaseConnection,
    name: &str,
    key: &ProjectionKey,
    state: JsonValue,
) -> Result<(), ProjectionError> {
    let now = Utc::now().naive_utc();
    let am = ActiveModel {
        projection_name: ActiveValue::Set(name.to_string()),
        key: ActiveValue::Set(key.0.clone()),
        state: ActiveValue::Set(state),
        version: ActiveValue::Set(1_i64),
        updated_at: ActiveValue::Set(now),
    };
    Entity::insert(am)
        .on_conflict(
            OnConflict::columns([Column::ProjectionName, Column::Key])
                .update_columns([Column::State, Column::UpdatedAt])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}

/// Read a snapshot by `(name, key)`.
///
/// Returns `Ok(Some(state))` when the row exists, `Ok(None)` when absent.
/// The raw [`serde_json::Value`] is returned; callers deserialize to their
/// typed state.
///
/// # Errors
///
/// Returns [`ProjectionError::Db`] on a SeaORM error.
pub async fn snapshot_read(
    db: &DatabaseConnection,
    name: &str,
    key: &ProjectionKey,
) -> Result<Option<JsonValue>, ProjectionError> {
    let row = Entity::find_by_id((name.to_string(), key.0.clone()))
        .one(db)
        .await?;
    Ok(row.map(|m| m.state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;
    use serde_json::json;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(crate::migration::Migration)]
        }
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate");
        conn
    }

    /// Round-trip: write then read returns the same value.
    #[tokio::test]
    async fn direct_snapshot_round_trip() {
        let db = fresh_db().await;
        let key = ProjectionKey::new("k1");
        let state = json!({"status": "completed", "value": 42});

        snapshot_write(&db, "test.direct", &key, state.clone())
            .await
            .expect("write");

        let read_back = snapshot_read(&db, "test.direct", &key).await.expect("read");

        assert_eq!(read_back, Some(state));
    }

    /// Overwrite: a second write to the same (name, key) wins; no error.
    #[tokio::test]
    async fn direct_snapshot_overwrite() {
        let db = fresh_db().await;
        let key = ProjectionKey::new("k2");

        snapshot_write(&db, "test.direct", &key, json!({"n": 1}))
            .await
            .expect("first write");

        snapshot_write(&db, "test.direct", &key, json!({"n": 2}))
            .await
            .expect("second write");

        let read_back = snapshot_read(&db, "test.direct", &key).await.expect("read");

        assert_eq!(read_back, Some(json!({"n": 2})));
    }

    /// Absent key reads as `None`.
    #[tokio::test]
    async fn snapshot_read_returns_none_for_absent() {
        let db = fresh_db().await;
        let key = ProjectionKey::new("never-written");

        let result = snapshot_read(&db, "test.direct", &key).await.expect("read");

        assert_eq!(result, None);
    }
}
