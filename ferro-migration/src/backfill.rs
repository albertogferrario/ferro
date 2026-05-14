//! Backend-portable backfill helpers for SeaORM migrations.

use crate::error::Error;
use sea_orm::{DbBackend, DbErr, Statement};
use sea_orm_migration::prelude::*;

/// Backfill a column with random hex strings of `hex_len` characters.
///
/// SQLite uses `lower(hex(randomblob(N/2)))`; Postgres uses
/// `encode(gen_random_bytes(N/2), 'hex')` (requires `pgcrypto`; see README).
/// MySQL is not supported.
///
/// Only rows where the column IS NULL or empty string are updated.
pub async fn backfill_random_hex(
    manager: &SchemaManager<'_>,
    table: &str,
    column: &str,
    hex_len: u32,
) -> Result<(), DbErr> {
    let backend = manager.get_database_backend();
    let sql = sql_for_random_hex(backend, table, column, hex_len)?;
    manager
        .get_connection()
        .execute(Statement::from_string(backend, sql))
        .await
        .map(|_| ())
}

pub(crate) fn sql_for_random_hex(
    backend: DbBackend,
    table: &str,
    column: &str,
    hex_len: u32,
) -> Result<String, Error> {
    let byte_len = hex_len / 2;
    match backend {
        DbBackend::Sqlite => Ok(format!(
            "UPDATE \"{table}\" SET \"{column}\" = lower(hex(randomblob({byte_len}))) \
             WHERE \"{column}\" IS NULL OR \"{column}\" = ''"
        )),
        DbBackend::Postgres => Ok(format!(
            "UPDATE \"{table}\" SET \"{column}\" = encode(gen_random_bytes({byte_len}), 'hex') \
             WHERE \"{column}\" IS NULL OR \"{column}\" = ''"
        )),
        DbBackend::MySql => Err(Error::UnsupportedBackend(
            "backfill_random_hex: MySQL not supported".into(),
        )),
    }
}

/// Backfill a column with random UUID v4 strings.
///
/// SQLite emits a UUID-shaped hex string assembled from `randomblob`.
/// Postgres uses `gen_random_uuid()::text` (Postgres 13+, core distribution).
///
/// Only rows where the column IS NULL or empty string are updated.
pub async fn backfill_random_uuid(
    manager: &SchemaManager<'_>,
    table: &str,
    column: &str,
) -> Result<(), DbErr> {
    let backend = manager.get_database_backend();
    let sql = sql_for_random_uuid(backend, table, column)?;
    manager
        .get_connection()
        .execute(Statement::from_string(backend, sql))
        .await
        .map(|_| ())
}

pub(crate) fn sql_for_random_uuid(
    backend: DbBackend,
    table: &str,
    column: &str,
) -> Result<String, Error> {
    match backend {
        DbBackend::Sqlite => Ok(format!(
            "UPDATE \"{table}\" SET \"{column}\" = \
             lower(hex(randomblob(4))) || '-' || \
             lower(hex(randomblob(2))) || '-4' || \
             substr(lower(hex(randomblob(2))), 2) || '-' || \
             substr('89ab', abs(random()) % 4 + 1, 1) || \
             substr(lower(hex(randomblob(2))), 2) || '-' || \
             lower(hex(randomblob(6))) \
             WHERE \"{column}\" IS NULL OR \"{column}\" = ''"
        )),
        DbBackend::Postgres => Ok(format!(
            "UPDATE \"{table}\" SET \"{column}\" = gen_random_uuid()::text \
             WHERE \"{column}\" IS NULL OR \"{column}\" = ''"
        )),
        DbBackend::MySql => Err(Error::UnsupportedBackend(
            "backfill_random_uuid: MySQL not supported".into(),
        )),
    }
}

/// Backfill a column with the current timestamp.
///
/// SQLite uses `CURRENT_TIMESTAMP`; Postgres uses `now()`.
/// MySQL is not supported.
///
/// Only rows where the column IS NULL are updated.
pub async fn backfill_current_timestamp(
    manager: &SchemaManager<'_>,
    table: &str,
    column: &str,
) -> Result<(), DbErr> {
    let backend = manager.get_database_backend();
    let sql = sql_for_current_timestamp(backend, table, column)?;
    manager
        .get_connection()
        .execute(Statement::from_string(backend, sql))
        .await
        .map(|_| ())
}

pub(crate) fn sql_for_current_timestamp(
    backend: DbBackend,
    table: &str,
    column: &str,
) -> Result<String, Error> {
    match backend {
        DbBackend::Sqlite => Ok(format!(
            "UPDATE \"{table}\" SET \"{column}\" = CURRENT_TIMESTAMP \
             WHERE \"{column}\" IS NULL"
        )),
        DbBackend::Postgres => Ok(format!(
            "UPDATE \"{table}\" SET \"{column}\" = now() \
             WHERE \"{column}\" IS NULL"
        )),
        DbBackend::MySql => Err(Error::UnsupportedBackend(
            "backfill_current_timestamp: MySQL not supported".into(),
        )),
    }
}

/// General-purpose backfill escape hatch: caller provides a closure that
/// produces backend-specific SQL.
///
/// The closure receives the current `DbBackend` and must return the SQL string
/// to execute, or `Err(DbErr)` if the backend is unsupported.
pub async fn backfill<F>(manager: &SchemaManager<'_>, sql_fn: F) -> Result<(), DbErr>
where
    F: FnOnce(DbBackend) -> Result<String, DbErr>,
{
    let backend = manager.get_database_backend();
    let sql = sql_fn(backend)?;
    manager
        .get_connection()
        .execute(Statement::from_string(backend, sql))
        .await
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_hex_sqlite_emits_randomblob() {
        let sql = sql_for_random_hex(DbBackend::Sqlite, "bookings", "checkin_token", 16).unwrap();
        assert!(sql.contains("lower(hex(randomblob(8)))"));
        assert!(sql.contains("\"bookings\""));
        assert!(sql.contains("\"checkin_token\""));
        assert!(sql.contains("IS NULL OR \"checkin_token\" = ''"));
    }

    #[test]
    fn random_hex_postgres_emits_gen_random_bytes() {
        let sql = sql_for_random_hex(DbBackend::Postgres, "bookings", "checkin_token", 16).unwrap();
        assert!(sql.contains("encode(gen_random_bytes(8), 'hex')"));
        assert!(sql.contains("\"bookings\""));
    }

    #[test]
    fn random_hex_mysql_returns_unsupported() {
        let err = sql_for_random_hex(DbBackend::MySql, "t", "c", 16).unwrap_err();
        assert!(matches!(err, Error::UnsupportedBackend(ref msg) if msg.contains("MySQL")));
    }

    #[test]
    fn random_uuid_sqlite_uses_randomblob_segments() {
        let sql = sql_for_random_uuid(DbBackend::Sqlite, "users", "external_id").unwrap();
        assert!(sql.contains("randomblob(4)"));
        assert!(sql.contains("'-4'"));
        assert!(sql.contains("substr('89ab'"));
    }

    #[test]
    fn random_uuid_postgres_uses_gen_random_uuid() {
        let sql = sql_for_random_uuid(DbBackend::Postgres, "users", "external_id").unwrap();
        assert!(sql.contains("gen_random_uuid()::text"));
    }

    #[test]
    fn current_timestamp_sqlite_uses_constant() {
        let sql = sql_for_current_timestamp(DbBackend::Sqlite, "t", "created_at").unwrap();
        assert!(sql.contains("CURRENT_TIMESTAMP"));
    }

    #[test]
    fn current_timestamp_postgres_uses_now() {
        let sql = sql_for_current_timestamp(DbBackend::Postgres, "t", "created_at").unwrap();
        assert!(sql.contains("now()"));
    }
}
