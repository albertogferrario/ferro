//! `CreateAuditLogTable` — SeaORM migration that creates the `audit_log`
//! table (D-18..D-22) and its two composite indexes (D-20).
//!
//! Consumers register this migration in their own `Migrator`:
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_audit::CreateAuditLogTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLog::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuditLog::TenantId).string().null())
                    .col(ColumnDef::new(AuditLog::ActorKind).string().not_null())
                    .col(ColumnDef::new(AuditLog::ActorId).string().null())
                    .col(ColumnDef::new(AuditLog::Action).string().not_null())
                    .col(ColumnDef::new(AuditLog::TargetKind).string().null())
                    .col(ColumnDef::new(AuditLog::TargetId).string().null())
                    .col(ColumnDef::new(AuditLog::Before).json().null())
                    .col(ColumnDef::new(AuditLog::After).json().null())
                    .col(ColumnDef::new(AuditLog::Reason).string().null())
                    .col(ColumnDef::new(AuditLog::CorrelationId).uuid().null())
                    .col(
                        ColumnDef::new(AuditLog::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // idx_audit_target: (tenant_id, target_kind, target_id, created_at)
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_target")
                    .table(AuditLog::Table)
                    .col(AuditLog::TenantId)
                    .col(AuditLog::TargetKind)
                    .col(AuditLog::TargetId)
                    .col(AuditLog::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // idx_audit_actor: (tenant_id, actor_kind, actor_id, created_at)
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_actor")
                    .table(AuditLog::Table)
                    .col(AuditLog::TenantId)
                    .col(AuditLog::ActorKind)
                    .col(AuditLog::ActorId)
                    .col(AuditLog::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLog::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLog {
    Table,
    Id,
    TenantId,
    ActorKind,
    ActorId,
    Action,
    TargetKind,
    TargetId,
    Before,
    After,
    Reason,
    CorrelationId,
    CreatedAt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![Box::new(super::Migration)]
        }
    }

    #[tokio::test]
    async fn migration_creates_table_and_indexes() {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");

        TestMigrator::up(&conn, None)
            .await
            .expect("run migration up");

        // Verify the audit_log table exists by querying sqlite_master.
        let table_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='audit_log'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for table");
        assert!(
            table_row.is_some(),
            "audit_log table not created by migration"
        );

        // Verify both indexes exist.
        for idx_name in &["idx_audit_target", "idx_audit_actor"] {
            let idx_row = conn
                .query_one(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    format!(
                        "SELECT name FROM sqlite_master WHERE type='index' AND name='{idx_name}'"
                    ),
                ))
                .await
                .expect("query sqlite_master for index");
            assert!(
                idx_row.is_some(),
                "index {idx_name} not created by migration"
            );
        }

        // Verify the migration's `down()` cleans up.
        TestMigrator::down(&conn, None)
            .await
            .expect("run migration down");
        let table_after_down = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='audit_log'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master after down");
        assert!(
            table_after_down.is_none(),
            "audit_log table should be dropped by down()"
        );
    }
}
