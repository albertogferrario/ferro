//! Query helpers for the `audit_log` table (D-23).
//!
//! `history_for_target` — full history of a specific target, ordered ASC
//! (hits `idx_audit_target`).
//!
//! `recent_by_actor` — recent activity by a specific actor, ordered DESC
//! with limit (hits `idx_audit_actor`).
//!
//! `recent` — most recent entries globally, ordered DESC with limit.
//!
//! For pagination or more exotic filters, consumers can `use
//! ferro_audit::AuditLogEntity;` and drop into sea-orm directly (D-25).

use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder, QuerySelect,
};

use crate::actor::AuditActor;
use crate::entity::{self, Column};
use crate::entry::AuditEntry;
use crate::error::AuditError;
use crate::target::AuditTarget;

/// Return every audit entry for `target`, ordered ascending by `created_at`.
///
/// Hits `idx_audit_target` for fast lookup. No limit applied — for
/// pagination, drop into sea-orm directly via [`crate::AuditLogEntity`].
pub async fn history_for_target<C: ConnectionTrait>(
    target: &AuditTarget,
    conn: &C,
) -> Result<Vec<AuditEntry>, AuditError> {
    let entries = entity::Entity::find()
        .filter(Column::TargetKind.eq(target.kind.clone()))
        .filter(Column::TargetId.eq(target.id.clone()))
        .order_by(Column::CreatedAt, Order::Asc)
        .all(conn)
        .await?;
    Ok(entries)
}

/// Return the `limit` most recent audit entries for `actor`, ordered
/// descending by `created_at`.
///
/// Hits `idx_audit_actor` for fast lookup. For `AuditActor::System` /
/// `AuditActor::Anonymous` (no id), filters on `actor_id IS NULL` to
/// match the SQL NULL persisted by the write builder.
pub async fn recent_by_actor<C: ConnectionTrait>(
    actor: &AuditActor,
    conn: &C,
    limit: u64,
) -> Result<Vec<AuditEntry>, AuditError> {
    let mut query = entity::Entity::find().filter(Column::ActorKind.eq(actor.kind()));

    match actor.id() {
        Some(id) => {
            query = query.filter(Column::ActorId.eq(id.to_string()));
        }
        None => {
            query = query.filter(Column::ActorId.is_null());
        }
    }

    let entries = query
        .order_by(Column::CreatedAt, Order::Desc)
        .limit(limit)
        .all(conn)
        .await?;
    Ok(entries)
}

/// Return the `limit` most recent audit entries globally, ordered
/// descending by `created_at`. Useful for an admin "recent activity" panel.
pub async fn recent<C: ConnectionTrait>(
    conn: &C,
    limit: u64,
) -> Result<Vec<AuditEntry>, AuditError> {
    let entries = entity::Entity::find()
        .order_by(Column::CreatedAt, Order::Desc)
        .limit(limit)
        .all(conn)
        .await?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection};
    use sea_orm_migration::prelude::*;
    use std::time::Duration;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(crate::migration::Migration)]
        }
    }

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect sqlite::memory:");
        TestMigrator::up(&conn, None).await.expect("run migration");
        conn
    }

    // VALIDATION 153-07-01: history_for_target ordering
    #[tokio::test]
    async fn history_ordering() {
        let conn = fresh_db().await;
        let target = AuditTarget::new("inventory.unit", "abc");

        // Insert three entries with a small sleep between to encourage
        // distinct created_at values. SQLite default-CURRENT_TIMESTAMP is
        // typically second-precision; if all three share a second, the
        // returned set is still complete and the test still proves the
        // filter works.
        let e1 = AuditEntry::record("inventory.unit.created")
            .target(target.clone())
            .write(&conn)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1100)).await;

        let e2 = AuditEntry::record("inventory.unit.updated")
            .target(target.clone())
            .write(&conn)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1100)).await;

        let e3 = AuditEntry::record("inventory.unit.deleted")
            .target(target.clone())
            .write(&conn)
            .await
            .unwrap();

        let entries = history_for_target(&target, &conn).await.unwrap();
        assert_eq!(entries.len(), 3);

        // Ordering: by created_at ASC. Allow `<=` to tolerate same-second
        // groupings; verify each pair is non-decreasing.
        assert!(entries[0].created_at <= entries[1].created_at);
        assert!(entries[1].created_at <= entries[2].created_at);

        // Verify the IDs are present (no entries dropped).
        let ids: std::collections::HashSet<_> = entries.iter().map(|e| e.id).collect();
        assert!(ids.contains(&e1.id));
        assert!(ids.contains(&e2.id));
        assert!(ids.contains(&e3.id));

        // Verify a different target returns 0 entries.
        let other_target = AuditTarget::new("inventory.unit", "xyz");
        let other = history_for_target(&other_target, &conn).await.unwrap();
        assert_eq!(other.len(), 0);
    }

    // VALIDATION 153-07-02: recent_by_actor ordering + limit + actor-id-NULL filter
    #[tokio::test]
    async fn recent_by_actor_test() {
        let conn = fresh_db().await;
        let alice = AuditActor::User("alice".into());
        let bob = AuditActor::User("bob".into());

        // 3 entries by Alice, 2 by Bob
        for i in 0..3 {
            AuditEntry::record("test.event")
                .actor(alice.clone())
                .target(AuditTarget::new("test", format!("a{i}")))
                .write(&conn)
                .await
                .unwrap();
        }
        for i in 0..2 {
            AuditEntry::record("test.event")
                .actor(bob.clone())
                .target(AuditTarget::new("test", format!("b{i}")))
                .write(&conn)
                .await
                .unwrap();
        }

        // Filter by Alice — get exactly 3
        let alice_entries = recent_by_actor(&alice, &conn, 10).await.unwrap();
        assert_eq!(alice_entries.len(), 3);
        assert!(alice_entries
            .iter()
            .all(|e| e.actor_id == Some("alice".into())));

        // Filter by Alice with limit=2 — get exactly 2
        let alice_limited = recent_by_actor(&alice, &conn, 2).await.unwrap();
        assert_eq!(alice_limited.len(), 2);

        // Filter by Bob — get exactly 2
        let bob_entries = recent_by_actor(&bob, &conn, 10).await.unwrap();
        assert_eq!(bob_entries.len(), 2);

        // Test the actor_id-NULL filter case: System actor
        AuditEntry::record("system.cleanup")
            .actor(AuditActor::System)
            .target(AuditTarget::new("system", "task"))
            .write(&conn)
            .await
            .unwrap();
        let system_entries = recent_by_actor(&AuditActor::System, &conn, 10)
            .await
            .unwrap();
        assert_eq!(system_entries.len(), 1);
        assert_eq!(system_entries[0].actor_kind, "system");
        assert_eq!(system_entries[0].actor_id, None);
    }

    // VALIDATION 153-07-03: recent global limit enforcement
    #[tokio::test]
    async fn recent_global() {
        let conn = fresh_db().await;

        // 5 entries
        for i in 0..5 {
            AuditEntry::record("test.event")
                .target(AuditTarget::new("test", format!("t{i}")))
                .write(&conn)
                .await
                .unwrap();
        }

        let entries = recent(&conn, 3).await.unwrap();
        assert_eq!(entries.len(), 3);

        // Ordering DESC: each created_at >= the next
        assert!(entries[0].created_at >= entries[1].created_at);
        assert!(entries[1].created_at >= entries[2].created_at);

        // Limit 0 returns 0
        let none = recent(&conn, 0).await.unwrap();
        assert_eq!(none.len(), 0);

        // Limit 100 returns all 5
        let all = recent(&conn, 100).await.unwrap();
        assert_eq!(all.len(), 5);
    }
}
