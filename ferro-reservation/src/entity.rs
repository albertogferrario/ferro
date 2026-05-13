//! SeaORM `Entity` / `Model` / `ActiveModel` / `Column` / `Relation` for
//! the `reservations` table.
//!
//! Schema authority is `migration.rs` (`CreateReservationsTable`). This
//! module's `Model` shape must match the migration's column declarations
//! exactly.

use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

/// One row of the `reservations` table. Persisted by `ReservationKernel::hold`
/// and updated by the state-transition methods.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "reservations")]
pub struct Model {
    /// Reservation id — client-generated UUIDv4 (D-41).
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Dotted-namespace resource kind (D-08).
    pub resource_kind: String,

    /// Serialized `Resource::Key` (D-05). NOT NULL.
    pub resource_key: JsonValue,

    /// Serialized `Resource::Window`. NULL when `Window = ()`.
    pub window: Option<JsonValue>,

    /// Reserved units count. Stored as `i32` per SeaORM INTEGER mapping;
    /// the kernel casts to `u32` at the `ReservationHandle` API boundary
    /// (RESEARCH.md Pitfall 6).
    pub quantity: i32,

    /// One of `"held"`, `"committed"`, `"released"`, `"expired"` (D-16).
    pub status: String,

    /// TTL expiry instant — mutated by `extend`.
    pub expires_at: DateTime,

    /// DB-stamped hold time (`DEFAULT CURRENT_TIMESTAMP`, D-42).
    pub held_at: DateTime,

    /// Set on commit (app-supplied `Utc::now()` inside the GuardedUpdate).
    pub committed_at: Option<DateTime>,

    /// Set on release.
    pub released_at: Option<DateTime>,

    /// Serialized `ReleaseReason` tag (set on release; D-18).
    pub release_reason: Option<String>,

    /// Optional tenant scope (D-36 stringly-typed).
    pub tenant_id: Option<String>,
}

/// No foreign keys in v0 — ferro-reservation owns the `reservations` table
/// independently of any consumer schema.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use sea_orm::{ActiveModelTrait, ActiveValue, Database, DatabaseConnection, EntityTrait};
    use sea_orm_migration::MigratorTrait;
    use serde_json::json;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(crate::migration::Migration)]
        }
    }

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate");
        conn
    }

    #[tokio::test]
    async fn model_round_trips_through_active_model() {
        let conn = fresh_db().await;
        let id = uuid::Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let expires = (Utc::now() + Duration::seconds(900)).naive_utc();

        // INSERT via ActiveModel
        let am = ActiveModel {
            id: ActiveValue::Set(id),
            resource_kind: ActiveValue::Set("inventory.unit".into()),
            resource_key: ActiveValue::Set(json!({"product": "abc", "tenant": "t1"})),
            window: ActiveValue::Set(Some(json!({"date": "2026-05-13"}))),
            quantity: ActiveValue::Set(3),
            status: ActiveValue::Set("held".into()),
            expires_at: ActiveValue::Set(expires),
            held_at: ActiveValue::Set(now),
            committed_at: ActiveValue::Set(None),
            released_at: ActiveValue::Set(None),
            release_reason: ActiveValue::Set(None),
            tenant_id: ActiveValue::Set(Some("t1".into())),
        };
        am.insert(&conn).await.expect("insert");

        // SELECT back and assert all 12 fields
        let fetched = Entity::find_by_id(id)
            .one(&conn)
            .await
            .expect("query")
            .expect("found");
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.resource_kind, "inventory.unit");
        assert_eq!(
            fetched.resource_key,
            json!({"product": "abc", "tenant": "t1"})
        );
        assert_eq!(fetched.window, Some(json!({"date": "2026-05-13"})));
        assert_eq!(fetched.quantity, 3);
        assert_eq!(fetched.status, "held");
        assert_eq!(fetched.expires_at, expires);
        assert_eq!(fetched.held_at, now);
        assert_eq!(fetched.committed_at, None);
        assert_eq!(fetched.released_at, None);
        assert_eq!(fetched.release_reason, None);
        assert_eq!(fetched.tenant_id, Some("t1".into()));
    }
}
