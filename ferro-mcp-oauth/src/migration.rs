//! Migrations for `ferro-mcp-oauth`.
//!
//! - [`Migration`] — `CreateOauthClientsTable`: creates the `oauth_clients` table.
//! - [`MigrationMcpApiKeys`] — `CreateMcpApiKeysTable`: creates the per-tenant `mcp_api_keys` table.
//!
//! Consumers register these migrations in their own `Migrator`:
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             // ... other migrations
//!             Box::new(ferro_mcp_oauth::CreateOauthClientsTable),
//!             Box::new(ferro_mcp_oauth::CreateMcpApiKeysTable),
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

// ── mcp_api_keys migration ────────────────────────────────────────────────────

/// Migration that creates the per-tenant `mcp_api_keys` table and its indexes.
///
/// Schema: `id`, `tenant_id`, `key_hash` (SHA-256 hex, unique), `scope`,
/// `revoked_at` (NULL = active), `created_at`, `updated_at`.
/// Exported as [`ferro_mcp_oauth::CreateMcpApiKeysTable`].
#[derive(DeriveMigrationName)]
pub struct MigrationMcpApiKeys;

#[async_trait::async_trait]
impl MigrationTrait for MigrationMcpApiKeys {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(McpApiKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(McpApiKeys::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(McpApiKeys::TenantId).big_integer().not_null())
                    .col(ColumnDef::new(McpApiKeys::KeyHash).string().not_null())
                    .col(
                        ColumnDef::new(McpApiKeys::Scope)
                            .string()
                            .not_null()
                            .default("read"),
                    )
                    .col(
                        ColumnDef::new(McpApiKeys::RevokedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(McpApiKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(McpApiKeys::UpdatedAt)
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
                    .name("idx_mcp_api_keys_key_hash")
                    .table(McpApiKeys::Table)
                    .col(McpApiKeys::KeyHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_mcp_api_keys_tenant_id")
                    .table(McpApiKeys::Table)
                    .col(McpApiKeys::TenantId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(McpApiKeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum McpApiKeys {
    Table,
    Id,
    TenantId,
    KeyHash,
    Scope,
    RevokedAt,
    CreatedAt,
    UpdatedAt,
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

    // ── mcp_api_keys migration test ───────────────────────────────────────────

    struct TestMigratorMcpApiKeys;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigratorMcpApiKeys {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![Box::new(super::MigrationMcpApiKeys)]
        }
    }

    #[tokio::test]
    async fn mcp_api_keys_migration_creates_table_and_indexes() {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");

        TestMigratorMcpApiKeys::up(&conn, None)
            .await
            .expect("run mcp_api_keys migration up");

        // Verify the mcp_api_keys table exists.
        let table_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='mcp_api_keys'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for table");
        assert!(
            table_row.is_some(),
            "mcp_api_keys table not created by migration"
        );

        // Verify the unique index on key_hash exists.
        let idx_hash_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_mcp_api_keys_key_hash'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for key_hash index");
        assert!(
            idx_hash_row.is_some(),
            "idx_mcp_api_keys_key_hash index not created by migration"
        );

        // Verify the index on tenant_id exists.
        let idx_tenant_row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_mcp_api_keys_tenant_id'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master for tenant_id index");
        assert!(
            idx_tenant_row.is_some(),
            "idx_mcp_api_keys_tenant_id index not created by migration"
        );

        // Verify down() drops the table.
        TestMigratorMcpApiKeys::down(&conn, None)
            .await
            .expect("run mcp_api_keys migration down");
        let table_after_down = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='mcp_api_keys'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master after down");
        assert!(
            table_after_down.is_none(),
            "mcp_api_keys table should be dropped by down()"
        );
    }
}
