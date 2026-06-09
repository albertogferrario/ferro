//! Postgres identity-path gate for `ConstraintMap::try_map` (Phase 191, SC3 Postgres).
//!
//! This test exercises the one `try_map` branch that the SQLite-only `cargo test`
//! default cannot reach: matching a UNIQUE violation on the **structured Postgres
//! constraint name** via `sqlx::postgres::PgDatabaseError::constraint()` (runtime
//! dispatch on `dyn DatabaseError`), NOT message parsing.
//!
//! It is `#[ignore]`d so the normal suite skips it (no Postgres in CI). Run it
//! against a live Postgres with:
//!
//! ```bash
//! DATABASE_URL=postgres://USER@localhost:5432/DBNAME \
//!   cargo test -p ferro-rs --test constraint_map_pg_gate -- --ignored --nocapture
//! ```
//!
//! The registered entry has NO `.sqlite()` discriminator, so a successful match
//! can ONLY come from the Postgres constraint-name path — isolating SC3 Postgres.

use ferro_rs::validation::ConstraintMap;
use sea_orm::{ConnectionTrait, Database, Statement};

fn pg_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://alberto@localhost:5432/postgres".to_owned())
}

async fn exec(db: &sea_orm::DatabaseConnection, sql: &str) -> Result<(), sea_orm::DbErr> {
    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_owned(),
    ))
    .await
    .map(|_| ())
}

/// SC3 (Postgres) — `try_map` matches on the structured constraint name.
///
/// Seeds a row, triggers a duplicate INSERT against a table with a NAMED UNIQUE
/// constraint (`cw_pg_slug_key`), and feeds the resulting `DbErr` to a
/// `ConstraintMap` registered with that constraint name and NO `.sqlite()` hint.
/// A field-level `ValidationError` on `slug` proves the `constraint()` dispatch.
#[tokio::test]
#[ignore = "requires a live Postgres (set DATABASE_URL); run with -- --ignored"]
async fn pg_constraint_name_identity_match() {
    let url = pg_url();
    let db = Database::connect(&url)
        .await
        .unwrap_or_else(|e| panic!("connect to Postgres at {url}: {e}"));

    // Clean slate (idempotent across reruns).
    exec(&db, "DROP TABLE IF EXISTS cw_pg_gate")
        .await
        .expect("drop");
    exec(
        &db,
        "CREATE TABLE cw_pg_gate (id INT PRIMARY KEY, slug TEXT NOT NULL, \
         CONSTRAINT cw_pg_slug_key UNIQUE (slug))",
    )
    .await
    .expect("create table with named unique constraint");

    exec(&db, "INSERT INTO cw_pg_gate (id, slug) VALUES (1, 'taken')")
        .await
        .expect("seed winning insert");

    let err = exec(&db, "INSERT INTO cw_pg_gate (id, slug) VALUES (2, 'taken')")
        .await
        .expect_err("expected a UNIQUE violation from the losing insert");

    // Postgres-name-only registration — no `.sqlite()`. The match MUST come from
    // the structured constraint name returned by `DatabaseError::constraint()`.
    let map = ConstraintMap::new().on("cw_pg_slug_key", "slug", "has already been taken");

    let ve = map
        .try_map(err)
        .expect("try_map should match the Postgres constraint name and return ValidationError");

    assert!(
        ve.has("slug"),
        "expected a field error on 'slug' from the Postgres constraint() path, got: {ve:?}"
    );

    // Best-effort cleanup.
    let _ = exec(&db, "DROP TABLE IF EXISTS cw_pg_gate").await;
}
