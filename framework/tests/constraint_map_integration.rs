//! Integration tests for `ConstraintMap::try_map` against a real in-memory SQLite DB.
//!
//! Exercises the full defensive-layer contract:
//! - SC4 TOCTOU simulation: a duplicate INSERT raises a real UNIQUE DbErr that
//!   `try_map` maps to a field-level `ValidationError`.
//! - SC3 SQLite: the identity match comes from the `table.column` message parse
//!   (not the Postgres constraint name).
//! - SC2 end-to-end: a non-UNIQUE `DbErr` passes through `try_map` unchanged.
//! - SC2 unregistered: a real UNIQUE DbErr with no matching entry passes through
//!   unchanged — the error is never swallowed.
//!
//! All tests are `#[serial]` because `DB` is a process-global singleton.

mod constraint_map_fixture;
use constraint_map_fixture::{exec_sql, init_constraint_db};
use ferro_rs::validation::ConstraintMap;
use serial_test::serial;

/// SC4 — TOCTOU concurrent-insert simulation.
///
/// Models the race: both logical inserts pass the proactive `unique` pre-check,
/// then the "losing" insert hits the DB UNIQUE constraint. `try_map` must convert
/// the resulting `DbErr` into the same field-level `ValidationError` that the
/// proactive rule would produce.
#[tokio::test]
#[serial]
async fn toctou_simulation_maps_to_field_error() {
    init_constraint_db().await;

    // "Winning" insert — seeds the row that takes the slug.
    exec_sql("INSERT INTO cw (id, slug) VALUES (1, 'taken')")
        .await
        .expect("seed winning insert");

    // "Losing" insert — hits the UNIQUE constraint.
    let err = exec_sql("INSERT INTO cw (id, slug) VALUES (2, 'taken')")
        .await
        .expect_err("expected UNIQUE violation from losing insert");

    let map = ConstraintMap::new()
        .on("cw_slug_unique", "slug", "has already been taken")
        .sqlite("cw.slug");

    let ve = map
        .try_map(err)
        .expect("try_map should convert UNIQUE violation to ValidationError");

    assert!(
        ve.has("slug"),
        "expected a field error on 'slug', got: {ve:?}"
    );
}

/// SC3 SQLite — identity match via message parse, not Postgres constraint name.
///
/// Registers the entry with a deliberately wrong Postgres name
/// (`"never_matches_pg_name"`) so that only the `.sqlite("cw.slug")` discriminator
/// can match. Verifies that `try_map` resolves via the SQLite message-parse path.
#[tokio::test]
#[serial]
async fn sqlite_identity_match_via_message() {
    init_constraint_db().await;

    exec_sql("INSERT INTO cw (id, slug) VALUES (1, 'taken')")
        .await
        .expect("seed");

    let err = exec_sql("INSERT INTO cw (id, slug) VALUES (2, 'taken')")
        .await
        .expect_err("expected UNIQUE violation");

    // Intentionally wrong Postgres constraint name — match MUST come from SQLite.
    let map = ConstraintMap::new()
        .on("never_matches_pg_name", "slug", "has already been taken")
        .sqlite("cw.slug");

    let ve = map
        .try_map(err)
        .expect("try_map should match via SQLite message parse");

    assert!(
        ve.has("slug"),
        "expected 'slug' field error from SQLite identity match, got: {ve:?}"
    );
}

/// SC2 — non-UNIQUE `DbErr` passes through `try_map` UNCHANGED.
///
/// A custom (non-UNIQUE) error must not be swallowed or converted: the original
/// `DbErr::Custom` must be returned as-is so the caller's `?` reaches the existing
/// `From<DbErr> for ActionError` passthrough at action.rs:196.
#[tokio::test]
#[serial]
async fn non_unique_error_passes_through_unchanged() {
    init_constraint_db().await;

    let err = sea_orm::DbErr::Custom("some other error".into());

    let map = ConstraintMap::new()
        .on("cw_slug_unique", "slug", "has already been taken")
        .sqlite("cw.slug");

    match map.try_map(err) {
        Err(sea_orm::DbErr::Custom(msg)) => {
            assert_eq!(
                msg, "some other error",
                "non-UNIQUE error must pass through with original message"
            );
        }
        other => panic!("expected Err(DbErr::Custom(\"some other error\")), got {other:?}"),
    }
}

/// SC2 — real UNIQUE DbErr with NO matching entry passes through UNCHANGED.
///
/// A genuine UNIQUE-constraint violation where the `ConstraintMap` has no entry
/// matching the `cw.slug` token must be returned as `Err(DbErr)` — never swallowed,
/// never converted to `Ok`.
#[tokio::test]
#[serial]
async fn unregistered_unique_passes_through() {
    init_constraint_db().await;

    exec_sql("INSERT INTO cw (id, slug) VALUES (1, 'taken')")
        .await
        .expect("seed");

    let err = exec_sql("INSERT INTO cw (id, slug) VALUES (2, 'taken')")
        .await
        .expect_err("expected UNIQUE violation");

    // Register a non-matching SQLite key — this entry should NOT match cw.slug.
    let map = ConstraintMap::new()
        .on("cw_slug_unique", "slug", "has already been taken")
        .sqlite("cw.other_col");

    assert!(
        map.try_map(err).is_err(),
        "unregistered UNIQUE violation must pass through unchanged (Err), not be swallowed"
    );
}
