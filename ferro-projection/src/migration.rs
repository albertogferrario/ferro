//! SeaORM migration for the `projection_snapshots` table (D-23, D-24).
//!
//! Schema columns (D-24):
//! - `projection_name VARCHAR NOT NULL` — `P::NAME` (D-06)
//! - `key VARCHAR NOT NULL` — `ProjectionKey::as_str()` (D-11)
//! - `state JSON NOT NULL` — serialized `P::State` (D-26)
//! - `version BIGINT NOT NULL` — monotonic counter (D-25)
//! - `updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP` (D-27)
//!
//! Primary key is the composite `(projection_name, key)` (D-24) — every
//! lookup path hits it directly; no secondary indexes in v0.

use sea_orm_migration::prelude::*;

pub struct Migration;

impl sea_orm_migration::MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260514_000001_create_projection_snapshots_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectionSnapshots::Table)
                    .if_not_exists()
                    // projection_name VARCHAR NOT NULL — part of composite PK (D-24)
                    .col(
                        ColumnDef::new(ProjectionSnapshots::ProjectionName)
                            .string()
                            .not_null(),
                    )
                    // key VARCHAR NOT NULL — part of composite PK (D-24)
                    .col(ColumnDef::new(ProjectionSnapshots::Key).string().not_null())
                    // state JSON NOT NULL (D-26)
                    .col(ColumnDef::new(ProjectionSnapshots::State).json().not_null())
                    // version BIGINT NOT NULL (D-25)
                    .col(
                        ColumnDef::new(ProjectionSnapshots::Version)
                            .big_integer()
                            .not_null(),
                    )
                    // updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP (D-27)
                    .col(
                        ColumnDef::new(ProjectionSnapshots::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Composite primary key (D-24) — only lookup path
                    .primary_key(
                        Index::create()
                            .col(ProjectionSnapshots::ProjectionName)
                            .col(ProjectionSnapshots::Key),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectionSnapshots::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectionSnapshots {
    Table,
    ProjectionName,
    Key,
    State,
    Version,
    UpdatedAt,
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(super::Migration)]
        }
    }

    #[tokio::test]
    async fn migration_creates_projection_snapshots_table() {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate up");

        let row = conn
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='projection_snapshots'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master");
        assert!(row.is_some(), "projection_snapshots table not created");
    }
}
