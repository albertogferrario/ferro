//! Migration: create `payment_intents` table with supporting indexes and the
//! cross-backend partial unique index enforcing at most one active row per
//! `(billable_kind, billable_id)`.
//!
//! # Security note
//! All `execute_unprepared` strings are static migration literals with no
//! interpolated user-supplied values — SQL injection is impossible by
//! construction (T-233-03).

use sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl sea_orm_migration::MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260617_000001_create_payment_intents"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // --- Table creation ---------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .table(PaymentIntents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PaymentIntents::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::TenantId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::BillableKind)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::BillableId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::AmountCents)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PaymentIntents::Currency).text().not_null())
                    .col(ColumnDef::new(PaymentIntents::Status).text().not_null())
                    .col(
                        ColumnDef::new(PaymentIntents::StripeSessionId)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::PaymentIntentId)
                            .text()
                            .null(),
                    )
                    .col(ColumnDef::new(PaymentIntents::ChargeId).text().null())
                    .col(ColumnDef::new(PaymentIntents::StripeRefundId).text().null())
                    .col(
                        ColumnDef::new(PaymentIntents::ApplicationFeeCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::ReservedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::PaidAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::ReleasedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::RefundedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PaymentIntents::RefundAmountCents)
                            .big_integer()
                            .null(),
                    )
                    .col(ColumnDef::new(PaymentIntents::Metadata).json().null())
                    .to_owned(),
            )
            .await?;

        // --- Supporting indexes -----------------------------------------------

        // Composite index for tenant + status queries (most common filter pattern).
        manager
            .create_index(
                Index::create()
                    .name("idx_payment_intents_tenant_status")
                    .table(PaymentIntents::Table)
                    .col(PaymentIntents::TenantId)
                    .col(PaymentIntents::Status)
                    .to_owned(),
            )
            .await?;

        // Unique index on stripe_session_id — each Stripe session maps to exactly
        // one payment intent.
        manager
            .create_index(
                Index::create()
                    .name("idx_payment_intents_stripe_session_id")
                    .table(PaymentIntents::Table)
                    .col(PaymentIntents::StripeSessionId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Non-unique index on payment_intent_id for webhook lookup.
        manager
            .create_index(
                Index::create()
                    .name("idx_payment_intents_payment_intent_id")
                    .table(PaymentIntents::Table)
                    .col(PaymentIntents::PaymentIntentId)
                    .to_owned(),
            )
            .await?;

        // Non-unique index on stripe_refund_id for the reconcile reaper's
        // poll-by-refund-id lookup (WR-05).
        manager
            .create_index(
                Index::create()
                    .name("idx_payment_intents_stripe_refund_id")
                    .table(PaymentIntents::Table)
                    .col(PaymentIntents::StripeRefundId)
                    .to_owned(),
            )
            .await?;

        // --- Partial unique index (cross-backend) ------------------------------
        //
        // SeaORM's IndexCreateStatement has no WHERE-clause API (D-03). The
        // partial-unique invariant — at most one active (reserved|paid) row per
        // billable — is enforced via raw DDL branched on the database backend:
        //
        //   Postgres / SQLite: true partial unique index (identical syntax).
        //   MySQL: stored generated column (active ⟹ identity string; inactive ⟹ NULL)
        //          plus a plain UNIQUE index; MySQL does not deduplicate NULLs, giving
        //          the correct "only one active row" semantics.
        //
        // All strings below are static literals with NO user-supplied values (T-233-03).
        let db = manager.get_connection();
        match manager.get_database_backend() {
            DatabaseBackend::Postgres | DatabaseBackend::Sqlite => {
                db.execute_unprepared(
                    "CREATE UNIQUE INDEX uq_payment_intents_active \
                     ON payment_intents (billable_kind, billable_id) \
                     WHERE status IN ('reserved','paid')",
                )
                .await?;
            }
            DatabaseBackend::MySql => {
                db.execute_unprepared(
                    "ALTER TABLE payment_intents \
                     ADD COLUMN active_billable_key VARCHAR(600) \
                     AS (CASE WHEN status IN ('reserved','paid') \
                              THEN CONCAT(billable_kind, '|', CAST(billable_id AS CHAR)) \
                              ELSE NULL END) STORED",
                )
                .await?;
                db.execute_unprepared(
                    "CREATE UNIQUE INDEX uq_payment_intents_active_mysql \
                     ON payment_intents (active_billable_key)",
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Dropping the table drops all its indexes, including the partial unique
        // index and the MySQL generated column.
        manager
            .drop_table(Table::drop().table(PaymentIntents::Table).to_owned())
            .await
    }
}

/// Column identifiers for the `payment_intents` table.
#[derive(DeriveIden)]
enum PaymentIntents {
    Table,
    Id,
    TenantId,
    BillableKind,
    BillableId,
    AmountCents,
    Currency,
    Status,
    StripeSessionId,
    PaymentIntentId,
    ChargeId,
    StripeRefundId,
    ApplicationFeeCents,
    ExpiresAt,
    ReservedAt,
    PaidAt,
    ReleasedAt,
    RefundedAt,
    RefundAmountCents,
    Metadata,
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
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

    /// Returns `true` if an object with the given name and type exists in
    /// `sqlite_master`.
    async fn name_exists(conn: &sea_orm::DatabaseConnection, name: &str, obj_type: &str) -> bool {
        let row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("SELECT name FROM sqlite_master WHERE type='{obj_type}' AND name='{name}'"),
            ))
            .await
            .expect("query sqlite_master");
        row.is_some()
    }

    /// Migration creates the table and all four indexes (three supporting +
    /// the partial unique).
    #[tokio::test]
    async fn migration_creates_table_and_indexes() {
        let conn = fresh_db().await;
        assert!(name_exists(&conn, "payment_intents", "table").await);
        assert!(name_exists(&conn, "idx_payment_intents_tenant_status", "index").await);
        assert!(name_exists(&conn, "idx_payment_intents_stripe_session_id", "index").await);
        assert!(name_exists(&conn, "idx_payment_intents_payment_intent_id", "index").await);
        assert!(name_exists(&conn, "idx_payment_intents_stripe_refund_id", "index").await);
        assert!(name_exists(&conn, "uq_payment_intents_active", "index").await);
    }

    /// `down()` removes the table (and all its indexes).
    #[tokio::test]
    async fn migration_down_drops_table() {
        let conn = fresh_db().await;
        TestMigrator::down(&conn, Some(1))
            .await
            .expect("migrate down");
        assert!(!name_exists(&conn, "payment_intents", "table").await);
    }

    /// The partial unique index rejects a second active row (`status =
    /// 'reserved'`) for the same `(billable_kind, billable_id)` pair.
    #[tokio::test]
    async fn partial_unique_rejects_second_active_row() {
        let conn = fresh_db().await;

        // First active row — must succeed.
        conn.execute_unprepared(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'order',42,1000,'EUR','reserved',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')",
        )
        .await
        .expect("first insert");

        // Second active row for the same billable — must fail with a unique
        // constraint violation on the partial index.
        let result = conn
            .execute_unprepared(
                "INSERT INTO payment_intents \
                 (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
                  expires_at,reserved_at) \
                 VALUES (1,'order',42,1000,'EUR','reserved',\
                 '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')",
            )
            .await;

        assert!(
            result.is_err(),
            "second active insert must violate partial unique index"
        );
    }

    /// After the active row transitions to a non-active status (`released`),
    /// a new active row for the same billable is accepted by the index.
    #[tokio::test]
    async fn partial_unique_allows_new_active_after_release() {
        let conn = fresh_db().await;

        // Insert a released row — not covered by the partial index, so allowed.
        conn.execute_unprepared(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'order',42,1000,'EUR','released',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')",
        )
        .await
        .expect("released row");

        // A new reserved row for the same billable must succeed because the
        // previous row has a non-active status excluded from the partial index.
        conn.execute_unprepared(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'order',42,1000,'EUR','reserved',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')",
        )
        .await
        .expect("new reserved after release");
    }
}
