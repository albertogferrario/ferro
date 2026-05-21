//! `CreateReservationsTable` — SeaORM migration that creates the
//! `reservations` table and its two composite indexes (D-38..D-42).
//!
//! Consumers register this migration in their own `Migrator`, alongside
//! `ferro_audit::CreateAuditLogTable`:
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_audit::CreateAuditLogTable),
//!             Box::new(ferro_reservation::CreateReservationsTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```

use sea_orm_migration::prelude::*;

pub struct Migration;

impl sea_orm_migration::MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260513_000001_create_reservations_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Reservations::Table)
                    .if_not_exists()
                    // id UUID PRIMARY KEY — client-generated (D-41)
                    .col(
                        ColumnDef::new(Reservations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    // resource_kind VARCHAR NOT NULL — "inventory.unit", "checkout.slot"
                    .col(
                        ColumnDef::new(Reservations::ResourceKind)
                            .string()
                            .not_null(),
                    )
                    // resource_key JSONB NOT NULL on Postgres / JSON on SQLite —
                    // serialized Resource::Key. `json_binary()` maps to Postgres `jsonb`
                    // (btree-indexable) and to SQLite TEXT affinity (permissive). The
                    // plain `json()` type does NOT support btree indexes on Postgres
                    // (SQLSTATE 42704 at index creation), so `json_binary` is required
                    // for the composite index below to apply cross-backend.
                    .col(
                        ColumnDef::new(Reservations::ResourceKey)
                            .json_binary()
                            .not_null(),
                    )
                    // window JSONB NULL on Postgres / JSON on SQLite —
                    // serialized Resource::Window; NULL when Window = (). Same
                    // rationale as ResourceKey above.
                    .col(ColumnDef::new(Reservations::Window).json_binary().null())
                    // quantity INTEGER NOT NULL — u32 stored as INTEGER
                    .col(ColumnDef::new(Reservations::Quantity).integer().not_null())
                    // status VARCHAR NOT NULL — D-16 stringly-typed (not SeaORM ActiveEnum)
                    .col(ColumnDef::new(Reservations::Status).string().not_null())
                    // expires_at TIMESTAMP NOT NULL — set at hold; mutated by extend
                    .col(
                        ColumnDef::new(Reservations::ExpiresAt)
                            .timestamp()
                            .not_null(),
                    )
                    // held_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP (D-42)
                    .col(
                        ColumnDef::new(Reservations::HeldAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // committed_at TIMESTAMP NULL — set on commit
                    .col(ColumnDef::new(Reservations::CommittedAt).timestamp().null())
                    // released_at TIMESTAMP NULL — set on release
                    .col(ColumnDef::new(Reservations::ReleasedAt).timestamp().null())
                    // release_reason VARCHAR NULL — serialized ReleaseReason tag
                    .col(ColumnDef::new(Reservations::ReleaseReason).string().null())
                    // tenant_id VARCHAR NULL (D-36 stringly-typed)
                    .col(ColumnDef::new(Reservations::TenantId).string().null())
                    .to_owned(),
            )
            .await?;

        // idx_reservations_kind_key_window_status — Resource::held lookup path (D-40)
        manager
            .create_index(
                Index::create()
                    .name("idx_reservations_kind_key_window_status")
                    .table(Reservations::Table)
                    .col(Reservations::ResourceKind)
                    .col(Reservations::ResourceKey)
                    .col(Reservations::Window)
                    .col(Reservations::Status)
                    .to_owned(),
            )
            .await?;

        // idx_reservations_status_expires — sweeper scan path (D-40)
        manager
            .create_index(
                Index::create()
                    .name("idx_reservations_status_expires")
                    .table(Reservations::Table)
                    .col(Reservations::Status)
                    .col(Reservations::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reservations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Reservations {
    Table,
    Id,
    ResourceKind,
    ResourceKey,
    Window,
    Quantity,
    Status,
    ExpiresAt,
    HeldAt,
    CommittedAt,
    ReleasedAt,
    ReleaseReason,
    TenantId,
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

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate up");
        conn
    }

    async fn name_exists(conn: &sea_orm::DatabaseConnection, name: &str, obj_type: &str) -> bool {
        let row = conn
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                format!("SELECT name FROM sqlite_master WHERE type='{obj_type}' AND name='{name}'"),
            ))
            .await
            .expect("query sqlite_master");
        row.is_some()
    }

    #[tokio::test]
    async fn migration_creates_reservations_table_and_indexes() {
        let conn = fresh_db().await;
        assert!(
            name_exists(&conn, "reservations", "table").await,
            "reservations table should exist after up()"
        );
        assert!(
            name_exists(&conn, "idx_reservations_kind_key_window_status", "index").await,
            "idx_reservations_kind_key_window_status should exist"
        );
        assert!(
            name_exists(&conn, "idx_reservations_status_expires", "index").await,
            "idx_reservations_status_expires should exist"
        );
    }

    #[tokio::test]
    async fn migration_down_drops_table() {
        let conn = fresh_db().await;
        assert!(name_exists(&conn, "reservations", "table").await);
        TestMigrator::down(&conn, Some(1))
            .await
            .expect("migrate down");
        assert!(
            !name_exists(&conn, "reservations", "table").await,
            "reservations table should be gone after down()"
        );
    }
}
