//! `CreateOauthClientsTable` — SeaORM migration that creates the `oauth_clients`
//! table (D-04) and its unique index on `client_id`.
//!
//! Consumers register this migration in their own `Migrator`:
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             // ... other migrations
//!             Box::new(ferro_mcp_oauth::CreateOauthClientsTable),
//!         ]
//!     }
//! }
//! ```

use sea_orm_migration::prelude::*;

/// Migration name derivation (table name used as the migration identifier).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OauthClients::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OauthClients::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OauthClients::ClientId).string().not_null())
                    .col(ColumnDef::new(OauthClients::ClientName).string().null())
                    .col(ColumnDef::new(OauthClients::RedirectUris).text().not_null())
                    .col(
                        ColumnDef::new(OauthClients::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_oauth_clients_client_id")
                    .table(OauthClients::Table)
                    .col(OauthClients::ClientId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OauthClients::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OauthClients {
    Table,
    Id,
    ClientId,
    ClientName,
    RedirectUris,
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
    async fn migration_creates_table_and_index() {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");

        TestMigrator::up(&conn, None)
            .await
            .expect("run migration up");

        // Verify the oauth_clients table exists.
        let table_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='oauth_clients'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for table");
        assert!(
            table_row.is_some(),
            "oauth_clients table not created by migration"
        );

        // Verify the unique index exists.
        let idx_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_oauth_clients_client_id'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for index");
        assert!(
            idx_row.is_some(),
            "idx_oauth_clients_client_id index not created by migration"
        );

        // Verify down() drops the table.
        TestMigrator::down(&conn, None)
            .await
            .expect("run migration down");
        let table_after_down = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='oauth_clients'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master after down");
        assert!(
            table_after_down.is_none(),
            "oauth_clients table should be dropped by down()"
        );
    }
}
