//! `CreateJobsTable` — SeaORM migration creating the `jobs` table (D-04, D-05)
//! and its three composite indexes (D-06). Portable across SQLite + Postgres:
//! no backend-specific SQL, only the SchemaManager DDL builder.
//!
//! Consumers register it in their own `Migrator`:
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_queue::CreateJobsTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```

use sea_orm_migration::prelude::*;

/// Migration that creates the `jobs` table and its indexes.
pub struct CreateJobsTable;

impl sea_orm_migration::MigrationName for CreateJobsTable {
    fn name(&self) -> &str {
        "m_create_jobs_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateJobsTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Jobs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Jobs::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Jobs::Queue)
                            .string()
                            .not_null()
                            .default("default"),
                    )
                    .col(ColumnDef::new(Jobs::JobType).string().not_null())
                    .col(ColumnDef::new(Jobs::Payload).text().not_null())
                    .col(
                        ColumnDef::new(Jobs::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Jobs::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Jobs::MaxRetries)
                            .integer()
                            .not_null()
                            .default(3),
                    )
                    .col(ColumnDef::new(Jobs::IdempotencyKey).string().null())
                    .col(ColumnDef::new(Jobs::TenantId).big_integer().null())
                    .col(
                        ColumnDef::new(Jobs::AvailableAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Jobs::ClaimedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(Jobs::ClaimedBy).string().null())
                    .col(ColumnDef::new(Jobs::Error).text().null())
                    .col(
                        ColumnDef::new(Jobs::FailedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Jobs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // idx_jobs_claim: (queue, status, available_at, id) — primary claim SELECT path
        manager
            .create_index(
                Index::create()
                    .name("idx_jobs_claim")
                    .table(Jobs::Table)
                    .col(Jobs::Queue)
                    .col(Jobs::Status)
                    .col(Jobs::AvailableAt)
                    .col(Jobs::Id)
                    .to_owned(),
            )
            .await?;

        // idx_jobs_reaper: (status, claimed_at) — reaper visibility-timeout scan
        manager
            .create_index(
                Index::create()
                    .name("idx_jobs_reaper")
                    .table(Jobs::Table)
                    .col(Jobs::Status)
                    .col(Jobs::ClaimedAt)
                    .to_owned(),
            )
            .await?;

        // idx_jobs_idempotency: (job_type, idempotency_key) — deduplication on enqueue
        manager
            .create_index(
                Index::create()
                    .name("idx_jobs_idempotency")
                    .table(Jobs::Table)
                    .col(Jobs::JobType)
                    .col(Jobs::IdempotencyKey)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Dropping the table also drops its indexes on both SQLite and Postgres.
        manager
            .drop_table(Table::drop().table(Jobs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Jobs {
    Table,
    Id,
    Queue,
    JobType,
    Payload,
    Status,
    Attempts,
    MaxRetries,
    IdempotencyKey,
    TenantId,
    AvailableAt,
    ClaimedAt,
    ClaimedBy,
    Error,
    FailedAt,
    CreatedAt,
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(super::CreateJobsTable)]
        }
    }

    #[tokio::test]
    async fn migration_creates_jobs_table() {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");

        TestMigrator::up(&conn, None)
            .await
            .expect("run migration up");

        // Verify the jobs table exists.
        let table_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='jobs'".to_string(),
            ))
            .await
            .expect("query sqlite_master for table");
        assert!(table_row.is_some(), "jobs table not created by migration");

        // Verify the claim index exists.
        let idx_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_jobs_claim'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for idx_jobs_claim");
        assert!(idx_row.is_some(), "idx_jobs_claim not created by migration");

        // Verify all three indexes exist.
        for idx_name in &["idx_jobs_claim", "idx_jobs_reaper", "idx_jobs_idempotency"] {
            let row = conn
                .query_one(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    format!(
                        "SELECT name FROM sqlite_master WHERE type='index' AND name='{idx_name}'"
                    ),
                ))
                .await
                .expect("query sqlite_master for index");
            assert!(row.is_some(), "index {idx_name} not created by migration");
        }

        // Verify down() drops the table.
        TestMigrator::down(&conn, None)
            .await
            .expect("run migration down");
        let table_after_down = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='jobs'".to_string(),
            ))
            .await
            .expect("query sqlite_master after down");
        assert!(
            table_after_down.is_none(),
            "jobs table should be dropped by down()"
        );
    }
}
