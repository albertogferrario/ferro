//! SeaORM `Entity` / `Model` / `ActiveModel` / `Column` / `Relation` for
//! the `projection_snapshots` table (D-24, D-25, D-26, D-27).
//!
//! Schema authority is `migration.rs` (`CreateProjectionSnapshotsTable`).
//! This module's `Model` shape must match the migration's column
//! declarations exactly.
//!
//! Composite primary key on `(projection_name, key)` is signaled to
//! `DeriveEntityModel` by annotating BOTH fields with
//! `#[sea_orm(primary_key, auto_increment = false)]`. SeaORM generates
//! the composite `PrimaryKey` impl automatically. Composite-PK lookups
//! use a tuple: `Entity::find_by_id((name_value, key_value))`.

use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "projection_snapshots")]
pub struct Model {
    /// Projection logical name, e.g. `"inventory.dashboard"`. First
    /// half of the composite primary key (D-24).
    #[sea_orm(primary_key, auto_increment = false)]
    pub projection_name: String,

    /// Per-row key inside the projection, e.g. `"warehouse-a"`.
    /// Second half of the composite primary key (D-24).
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,

    /// Serialized `P::State` (D-26 — JSON column).
    pub state: JsonValue,

    /// Monotonic counter (D-25); +1 per apply, reset on rebuild.
    pub version: i64,

    /// App-set `Utc::now()` inside the upsert (D-27).
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ActiveValue, Database, EntityTrait};
    use sea_orm_migration::MigratorTrait;

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

    #[tokio::test]
    async fn round_trip_with_composite_pk() {
        let conn = fresh_db().await;

        let name = "test.projection";
        let key = "test-key";
        let state = serde_json::json!({ "count": 7 });
        let version: i64 = 1;
        let updated_at = chrono::Utc::now().naive_utc();

        let am = ActiveModel {
            projection_name: ActiveValue::Set(name.to_string()),
            key: ActiveValue::Set(key.to_string()),
            state: ActiveValue::Set(state.clone()),
            version: ActiveValue::Set(version),
            updated_at: ActiveValue::Set(updated_at),
        };
        Entity::insert(am).exec(&conn).await.expect("insert");

        // Composite-PK lookup form: tuple of (projection_name, key)
        let fetched = Entity::find_by_id((name.to_string(), key.to_string()))
            .one(&conn)
            .await
            .expect("query")
            .expect("found");

        assert_eq!(fetched.projection_name, name);
        assert_eq!(fetched.key, key);
        assert_eq!(fetched.state, state);
        assert_eq!(fetched.version, version);
        assert_eq!(fetched.updated_at, updated_at);
    }

    #[tokio::test]
    async fn duplicate_composite_pk_is_constraint_violation() {
        let conn = fresh_db().await;

        let name = "dup.projection";
        let key = "dup-key";
        let state = serde_json::json!({});
        let now = chrono::Utc::now().naive_utc();

        let am1 = ActiveModel {
            projection_name: ActiveValue::Set(name.to_string()),
            key: ActiveValue::Set(key.to_string()),
            state: ActiveValue::Set(state.clone()),
            version: ActiveValue::Set(1),
            updated_at: ActiveValue::Set(now),
        };
        Entity::insert(am1).exec(&conn).await.expect("first insert");

        // Second insert with same (name, key) MUST fail — proves the
        // composite PK constraint actually fires at the DB level.
        let am2 = ActiveModel {
            projection_name: ActiveValue::Set(name.to_string()),
            key: ActiveValue::Set(key.to_string()),
            state: ActiveValue::Set(state),
            version: ActiveValue::Set(2),
            updated_at: ActiveValue::Set(now),
        };
        let result = Entity::insert(am2).exec(&conn).await;
        assert!(
            result.is_err(),
            "second insert with same composite PK should fail"
        );
    }
}
