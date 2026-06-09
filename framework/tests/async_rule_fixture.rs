//! Shared test fixture: in-memory SQLite + scratch table for async-rule tests.
//!
//! Tests using this fixture MUST be annotated `#[serial]` (from the `serial_test`
//! crate) because `DB` is a process-global singleton. Concurrent tests would race
//! on initialization and on the shared `widgets` table.
//!
//! # Usage
//!
//! ```rust,ignore
//! mod async_rule_fixture;
//! use async_rule_fixture::{init_test_db, seed_widget};
//!
//! #[tokio::test]
//! #[serial]
//! async fn my_test() {
//!     init_test_db().await;
//!     seed_widget(1, "hello").await;
//!     let db = ferro_rs::database::DB::connection().expect("connection");
//!     // ... assertions ...
//! }
//! ```

use ferro_rs::database::{DatabaseConfig, DB};
use sea_orm::{ConnectionTrait, Statement};

/// Initialize an in-memory SQLite DB singleton and create a `widgets` scratch
/// table with `id` and `slug` columns. Safe to call repeatedly within a
/// `#[serial]` test suite — `CREATE TABLE IF NOT EXISTS` is idempotent.
///
/// After this returns, `DB::connection()` yields a live SQLite connection.
pub async fn init_test_db() {
    let config = DatabaseConfig::builder().url("sqlite::memory:").build();
    DB::init_with(config).await.expect("init in-memory sqlite");
    let db = DB::connection().expect("connection after init");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS widgets (id INTEGER PRIMARY KEY, slug TEXT)".to_owned(),
    ))
    .await
    .expect("create widgets scratch table");
}

/// Insert a widget row with the given id and slug (helper for seeding test data).
///
/// Callers must have called [`init_test_db`] first.
pub async fn seed_widget(id: i64, slug: &str) {
    let db = DB::connection().expect("connection for seed_widget");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("INSERT INTO widgets (id, slug) VALUES ({id}, '{slug}')"),
    ))
    .await
    .expect("seed widget row");
}
