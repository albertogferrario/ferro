//! Shared test fixture: in-memory SQLite + scratch table with a REAL UNIQUE INDEX
//! for `ConstraintMap` integration tests.
//!
//! The `cw` scratch table has a NON-PK UNIQUE INDEX on `slug` so a duplicate INSERT
//! raises `SQLITE_CONSTRAINT_UNIQUE` (error code 2067), whose driver message is:
//! `"UNIQUE constraint failed: cw.slug"` — the `table.column` token the SQLite path
//! of `ConstraintMap::try_map` parses.
//!
//! Tests using this fixture MUST be annotated `#[serial]` (from the `serial_test`
//! crate) because `DB` is a process-global singleton. Concurrent tests would race
//! on initialization and on the shared `cw` table.
//!
//! # Usage
//!
//! ```rust,ignore
//! mod constraint_map_fixture;
//! use constraint_map_fixture::{exec_sql, init_constraint_db};
//!
//! #[tokio::test]
//! #[serial]
//! async fn my_test() {
//!     init_constraint_db().await;
//!     exec_sql("INSERT INTO cw (id, slug) VALUES (1, 'hello')").await.expect("seed");
//!     let err = exec_sql("INSERT INTO cw (id, slug) VALUES (2, 'hello')").await
//!         .expect_err("expected UNIQUE violation");
//!     // feed err to ConstraintMap::try_map ...
//! }
//! ```

use ferro_rs::database::{DatabaseConfig, DB};
use sea_orm::{ConnectionTrait, Statement};

/// Initialize an in-memory SQLite DB singleton and create a `cw` scratch table
/// with a NON-PK UNIQUE INDEX on `slug`. Safe to call repeatedly within a
/// `#[serial]` test suite — `CREATE TABLE IF NOT EXISTS` and `CREATE UNIQUE INDEX
/// IF NOT EXISTS` are idempotent.
///
/// After this returns, `DB::connection()` yields a live SQLite connection and any
/// attempt to insert a duplicate `slug` value will raise a real UNIQUE-constraint
/// violation (`SQLITE_CONSTRAINT_UNIQUE`, code 2067).
pub async fn init_constraint_db() {
    let config = DatabaseConfig::builder().url("sqlite::memory:").build();
    DB::init_with(config).await.expect("init in-memory sqlite");
    let db = DB::connection().expect("connection after init");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS cw (id INTEGER PRIMARY KEY, slug TEXT NOT NULL)".to_owned(),
    ))
    .await
    .expect("create cw table");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE UNIQUE INDEX IF NOT EXISTS cw_slug_unique ON cw (slug)".to_owned(),
    ))
    .await
    .expect("create unique index");
}

/// Execute a raw SQL statement against the singleton connection. Returns
/// `Err(DbErr)` on any DB error — including a UNIQUE-constraint violation — so
/// callers can capture and feed the error to `ConstraintMap::try_map`.
///
/// Callers must have called [`init_constraint_db`] first.
pub async fn exec_sql(sql: &str) -> Result<(), sea_orm::DbErr> {
    let db = DB::connection().expect("connection");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_owned(),
    ))
    .await
    .map(|_| ())
}
