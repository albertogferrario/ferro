//! `prune_older_than` — caller-driven retention (D-26).
//!
//! Deletes rows with `created_at < cutoff` from the `audit_log` table.
//! Returns the number of rows deleted. Scheduling is the consumer's
//! responsibility (typically a `ferro-queue` cron job).
//!
//! Operational note: audit trails are evidence. Aggressive pruning is
//! usually wrong. GDPR / privacy law may force it; in that case, 1–3 year
//! retention is the conventional default. The user-facing doc page
//! `docs/src/database/audit-log.md` covers this trade-off in full (D-27).

use chrono::NaiveDateTime;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::entity::{self, Column};
use crate::error::AuditError;

/// Delete all audit entries strictly older than `cutoff`. Returns the
/// number of rows deleted.
///
/// Rows with `created_at == cutoff` are PRESERVED (strict less-than).
pub async fn prune_older_than<C: ConnectionTrait>(
    cutoff: NaiveDateTime,
    conn: &C,
) -> Result<u64, AuditError> {
    let result = entity::Entity::delete_many()
        .filter(Column::CreatedAt.lt(cutoff))
        .exec(conn)
        .await?;
    Ok(result.rows_affected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::AuditEntry;
    use crate::target::AuditTarget;
    use chrono::Utc;
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

    // VALIDATION 153-09-01: prune_older_than returns count + deletes only old rows
    #[tokio::test]
    async fn prune_older_than_test() {
        let conn = fresh_db().await;

        // Insert 3 "old" entries
        for i in 0..3 {
            AuditEntry::record("test.old_event")
                .target(AuditTarget::new("test", format!("old{i}")))
                .write(&conn)
                .await
                .unwrap();
        }

        // Sleep 2 seconds, then capture the cutoff time
        tokio::time::sleep(Duration::from_secs(2)).await;
        let cutoff = Utc::now().naive_utc();
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Insert 2 "new" entries (created_at > cutoff)
        for i in 0..2 {
            AuditEntry::record("test.new_event")
                .target(AuditTarget::new("test", format!("new{i}")))
                .write(&conn)
                .await
                .unwrap();
        }

        // Verify pre-prune count: 5 rows
        let all_before = crate::query::recent(&conn, 100).await.unwrap();
        assert_eq!(all_before.len(), 5, "should have 5 rows pre-prune");

        // Run prune
        let deleted = prune_older_than(cutoff, &conn).await.unwrap();
        assert_eq!(deleted, 3, "should delete exactly the 3 old rows");

        // Verify post-prune count: 2 rows remain
        let all_after = crate::query::recent(&conn, 100).await.unwrap();
        assert_eq!(all_after.len(), 2, "should have 2 rows post-prune");
        assert!(
            all_after.iter().all(|e| e.action == "test.new_event"),
            "only new_event entries should survive"
        );

        // Running prune again with the same cutoff: 0 rows deleted (idempotent).
        let deleted_again = prune_older_than(cutoff, &conn).await.unwrap();
        assert_eq!(deleted_again, 0);
    }
}
