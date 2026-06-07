//! `CreateDeploymentsTable` + `CreateDeploymentPointersTable` —
//! SeaORM migrations creating the `deployments` and `deployment_pointers`
//! tables. Portable across SQLite + Postgres: no backend-specific SQL,
//! only the SchemaManager DDL builder.
//!
//! Consumers register both in their own `Migrator` in order:
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_deployments::CreateDeploymentsTable),
//!             Box::new(ferro_deployments::CreateDeploymentPointersTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```

use sea_orm_migration::prelude::*;

/// Migration that creates the `deployments` table.
pub struct CreateDeploymentsTable;

impl sea_orm_migration::MigrationName for CreateDeploymentsTable {
    fn name(&self) -> &str {
        "m_create_deployments_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateDeploymentsTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Deployments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Deployments::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Deployments::Identifier)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Deployments::OwnerKey).string().not_null())
                    .col(ColumnDef::new(Deployments::SourceRef).string().null())
                    .col(
                        ColumnDef::new(Deployments::ArtifactLocation)
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(Deployments::ByteSize).big_integer().null())
                    .col(
                        ColumnDef::new(Deployments::Status)
                            .string()
                            .not_null()
                            .default("building"),
                    )
                    .col(
                        ColumnDef::new(Deployments::ArtifactDeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Deployments::TerminatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Deployments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Deployments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Deployments {
    Table,
    Id,
    Identifier,
    OwnerKey,
    SourceRef,
    ArtifactLocation,
    ByteSize,
    Status,
    ArtifactDeletedAt,
    TerminatedAt,
    CreatedAt,
}

/// Migration that creates the `deployment_pointers` table.
pub struct CreateDeploymentPointersTable;

impl sea_orm_migration::MigrationName for CreateDeploymentPointersTable {
    fn name(&self) -> &str {
        "m_create_deployment_pointers_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateDeploymentPointersTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DeploymentPointers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeploymentPointers::OwnerKey)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DeploymentPointers::DeploymentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeploymentPointers::PreviousDeploymentId)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DeploymentPointers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DeploymentPointers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum DeploymentPointers {
    Table,
    OwnerKey,
    DeploymentId,
    PreviousDeploymentId,
    UpdatedAt,
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![
                Box::new(super::CreateDeploymentsTable),
                Box::new(super::CreateDeploymentPointersTable),
            ]
        }
    }

    #[tokio::test]
    async fn migration_creates_deployments_table() {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");

        TestMigrator::up(&conn, None)
            .await
            .expect("run migration up");

        // Verify the deployments table exists.
        let table_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='deployments'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for deployments table");
        assert!(
            table_row.is_some(),
            "deployments table not created by migration"
        );

        // Verify the deployment_pointers table exists.
        let pointers_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='deployment_pointers'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for deployment_pointers table");
        assert!(
            pointers_row.is_some(),
            "deployment_pointers table not created by migration"
        );

        // Verify artifact_deleted_at column is present in deployments table.
        let pragma_rows = conn
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "PRAGMA table_info(deployments)".to_string(),
            ))
            .await
            .expect("PRAGMA table_info(deployments)");
        let col_names: Vec<String> = pragma_rows
            .iter()
            .filter_map(|row| row.try_get_by::<String, _>("name").ok())
            .collect();
        assert!(
            col_names.iter().any(|c| c == "artifact_deleted_at"),
            "artifact_deleted_at column not found in deployments table; columns: {col_names:?}"
        );

        // Verify down() drops both tables.
        TestMigrator::down(&conn, None)
            .await
            .expect("run migration down");

        let deployments_after_down = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='deployments'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master after down");
        assert!(
            deployments_after_down.is_none(),
            "deployments table should be dropped by down()"
        );

        let pointers_after_down = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='deployment_pointers'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for deployment_pointers after down");
        assert!(
            pointers_after_down.is_none(),
            "deployment_pointers table should be dropped by down()"
        );
    }
}
