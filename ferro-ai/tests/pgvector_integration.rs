#![cfg(feature = "postgres-tests")]
//! SC-AISDK-05: pgvector-gated integration tests.
//!
//! Run with:
//!   DATABASE_URL=postgres://user:pass@localhost/ferro_test \
//!     cargo test -p ferro-ai --features pgvector,postgres-tests
//!
//! Without the `postgres-tests` feature this file compiles to an empty module.
//! Without `DATABASE_URL` set, each test prints a skip message and returns early.

use ferro_ai::pgvector::PgVectorStore;
use sqlx::PgPool;

/// Returns a connected pool when `DATABASE_URL` is set, `None` otherwise.
async fn fresh_pg_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect to postgres");
    Some(pool)
}

/// Verifies the store+nearest roundtrip: upsert two vectors, query for the
/// nearest neighbor to a point close to id=1's vector, and assert id=1 ranks
/// first with a score in `[-1.0, 1.0]` that exceeds the second neighbor's score.
#[tokio::test]
async fn store_and_nearest_roundtrip() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("DATABASE_URL not set — skipping pgvector integration test");
        return;
    }
    let pool = fresh_pg_pool().await.expect("DATABASE_URL checked above");

    // The integration test owns its own schema (D-09: PgVectorStore is query-only).
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&pool)
        .await
        .expect("create extension vector");

    let table = "ferro_ai_test_embeddings";
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {table} (id BIGINT PRIMARY KEY, vec vector(3))"
    ))
    .execute(&pool)
    .await
    .expect("create test table");

    // Truncate to ensure a clean state for this test run.
    sqlx::query(&format!("TRUNCATE {table}"))
        .execute(&pool)
        .await
        .expect("truncate test table");

    let store = PgVectorStore::new(table, "vec");

    // id=1: [1.0, 0.0, 0.0]  id=2: [0.0, 1.0, 0.0]
    store
        .store(&pool, 1, &[1.0_f32, 0.0, 0.0])
        .await
        .expect("store id=1");
    store
        .store(&pool, 2, &[0.0_f32, 1.0, 0.0])
        .await
        .expect("store id=2");

    // Query close to id=1: [0.9, 0.1, 0.0]
    let neighbors = store
        .nearest(&pool, &[0.9_f32, 0.1, 0.0], 2)
        .await
        .expect("nearest query");

    assert_eq!(neighbors.len(), 2, "expected 2 neighbors");
    assert_eq!(neighbors[0].id, 1, "nearest neighbor should be id=1");

    let top_score = neighbors[0].score;
    let second_score = neighbors[1].score;

    assert!(
        (-1.0..=1.0).contains(&top_score),
        "score must be in [-1, 1], got {top_score}"
    );
    assert!(
        top_score > second_score,
        "id=1 score ({top_score}) should exceed id=2 score ({second_score})"
    );

    // Clean up.
    sqlx::query(&format!("DROP TABLE IF EXISTS {table}"))
        .execute(&pool)
        .await
        .expect("drop test table");
}
