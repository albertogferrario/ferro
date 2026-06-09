//! Postgres live gate for the async `unique` rule (Phase 190, VALID-01 Postgres path).
//!
//! Exercises the one path the SQLite-only `cargo test` default cannot: the
//! `unique` rule's parameterized COUNT executed against a real Postgres backend —
//! `$1`/`$2` placeholders and double-quoted identifiers (`"widgets"`, `"slug"`,
//! `"id"`). The generated SQL string is already unit-asserted (`build_sql`); this
//! gate confirms it actually executes and returns correct results on Postgres.
//!
//! `#[ignore]`d so the normal suite skips it (no Postgres in CI). Run with:
//!
//! ```bash
//! DATABASE_URL=postgres://USER@localhost:5432/DBNAME \
//!   cargo test -p ferro-rs --test async_validation_pg_gate -- --ignored --nocapture
//! ```

use ferro_rs::database::{DatabaseConfig, DB};
use ferro_rs::{required, rules, string, unique, AsyncValidationError, AsyncValidator};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::json;
use serial_test::serial;

fn pg_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost:5432/postgres".to_owned())
}

async fn exec(sql: &str) {
    let db = DB::connection().expect("pg connection");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_owned(),
    ))
    .await
    .unwrap_or_else(|e| panic!("exec `{sql}`: {e}"));
}

/// Init the DB singleton against Postgres and (re)create a clean `widgets` table
/// with a UNIQUE-relevant `slug` column. Seeds id=1, slug='taken'.
async fn init_pg() {
    let config = DatabaseConfig::builder().url(&pg_url()).build();
    DB::init_with(config)
        .await
        .unwrap_or_else(|e| panic!("init Postgres singleton at {}: {e}", pg_url()));
    exec("DROP TABLE IF EXISTS widgets").await;
    exec("CREATE TABLE widgets (id BIGINT PRIMARY KEY, slug TEXT NOT NULL)").await;
    exec("INSERT INTO widgets (id, slug) VALUES (1, 'taken')").await;
}

/// VALID-01 (Postgres) — duplicate, free, and exclude-self all behave correctly,
/// proving the `$1`/`$2` parameterized COUNT with quoted identifiers executes.
#[tokio::test]
#[serial]
#[ignore = "requires a live Postgres (set DATABASE_URL); run with -- --ignored"]
async fn pg_unique_rule_placeholder_and_quoting_path() {
    init_pg().await;

    // Duplicate → field-level Validation error (the COUNT found the existing row).
    let data = json!({"slug": "taken"});
    let dup = AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("widgets", "slug"))
        .validate_async()
        .await;
    match dup {
        Err(AsyncValidationError::Validation(ref e)) => {
            assert!(
                e.has("slug"),
                "duplicate must yield a 'slug' field error: {e:?}"
            );
        }
        other => panic!("expected Err(Validation) for duplicate, got: {other:?}"),
    }

    // Free value → Ok (COUNT returned 0).
    let data = json!({"slug": "free"});
    let free = AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("widgets", "slug"))
        .validate_async()
        .await;
    assert!(
        free.is_ok(),
        "free value must pass on Postgres, got: {free:?}"
    );

    // Exclude-self → Ok (exercises the `AND "id" <> $2` branch with the second param).
    let data = json!({"slug": "taken"});
    let edit = AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("widgets", "slug").ignore(1_i64))
        .validate_async()
        .await;
    assert!(
        edit.is_ok(),
        "exclude-self (.ignore) must pass own row on Postgres, got: {edit:?}"
    );

    exec("DROP TABLE IF EXISTS widgets").await;
}
