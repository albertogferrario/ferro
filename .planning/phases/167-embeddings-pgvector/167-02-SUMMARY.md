---
phase: 167-embeddings-pgvector
plan: "02"
subsystem: ferro-ai
tags: [pgvector, sqlx, semantic-search, feature-gate, aisdk-05]
dependency_graph:
  requires: [167-01, 165-llmclient-trait-provider-implementations]
  provides: [ferro_ai::pgvector::PgVectorStore, ferro_ai::pgvector::Neighbor, pgvector feature gate]
  affects:
    - ferro-ai/Cargo.toml
    - ferro-ai/src/pgvector/mod.rs
    - ferro-ai/src/lib.rs
    - ferro-ai/tests/pgvector_integration.rs
tech_stack:
  added:
    - "pgvector 0.4 (optional dep, pgvector feature only)"
    - "sqlx 0.8 (optional dep, pgvector feature only; matches workspace Cargo.lock pin)"
  patterns:
    - optional-dep-behind-feature-gate
    - manual-FromRow-for-computed-column
    - two-layer-database-url-guard
    - parameterized-sqlx-runtime-query
key_files:
  created:
    - ferro-ai/src/pgvector/mod.rs
    - ferro-ai/tests/pgvector_integration.rs
  modified:
    - ferro-ai/Cargo.toml
    - ferro-ai/src/lib.rs
decisions:
  - "PgVectorStore takes &sqlx::PgPool (not impl PgExecutor) per plan lock — PgExecutor is not dyn-compatible in sqlx 0.8"
  - "Manual FromRow for Neighbor: score is a computed column (1-distance), not a direct table column; derive would fail"
  - "sqlx optional dep is unavoidable alongside pgvector 0.4 — pgvector does not re-export PgPool; documented in [features] comment per D-12"
  - "SC#4 verified: cargo tree --no-default-features shows neither pgvector nor sqlx in the dep graph"
metrics:
  duration_minutes: 25
  completed_date: "2026-06-08"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 4
---

# Phase 167 Plan 02: PgVectorStore feature-gated module (AISDK-05) Summary

Feature-gated `ferro_ai::pgvector` module with `PgVectorStore::store` (upsert) and `PgVectorStore::nearest` (cosine similarity query) over raw `sqlx::PgPool`; default builds pull neither `pgvector` nor `sqlx` (SC#4 verified).

## Tasks Completed

| Task | Name | Commit | Key files |
|------|------|--------|-----------|
| 1 | Cargo.toml feature + optional deps + PgVectorStore module | bf069c9a | ferro-ai/Cargo.toml, ferro-ai/src/pgvector/mod.rs |
| 2 | lib.rs feature-gated wiring + gated integration test + dep-graph verification | 4fcae858 | ferro-ai/src/lib.rs, ferro-ai/tests/pgvector_integration.rs |
| - | Cargo.lock update (new optional deps) | 580d9786 | Cargo.lock |

## What Was Built

**`ferro_ai::pgvector::PgVectorStore`** — thin query primitive over a caller-supplied `&sqlx::PgPool`. Constructor `new(table, column)` takes trusted application-code names; `store(pool, id, embedding)` upserts via `ON CONFLICT (id) DO UPDATE`; `nearest(pool, query, k)` returns `Vec<Neighbor>` ordered by cosine similarity using pgvector's `<=>` operator. Score is `1 - cosine_distance`, range `[-1, 1]`, matching the `cosine_similarity` convention from Plan 01.

**`ferro_ai::pgvector::Neighbor`** — `{ id: i64, score: f32 }`. Uses manual `impl sqlx::FromRow` because `score` is a computed SQL expression, not a direct table column; `#[derive(FromRow)]` would not map it correctly.

**Feature gating (`pgvector` / `postgres-tests`)** — both optional deps (`pgvector 0.4`, `sqlx 0.8`) are listed with `optional = true` and referenced via `dep:` syntax so the crate name does not create an implicit feature. The `#[cfg(feature = "pgvector")] pub mod pgvector;` gate in `lib.rs` excludes the entire module tree when the feature is off; items inside `pgvector/mod.rs` require no per-item `#[cfg]`.

**Integration test (`ferro-ai/tests/pgvector_integration.rs`)** — `#![cfg(feature = "postgres-tests")]` at the top makes the file a no-op without the feature. The `store_and_nearest_roundtrip` test owns its schema (D-09): creates a temp table, stores two 3-dim vectors, queries nearest to a point close to id=1, asserts id=1 ranks first with score in `[-1, 1]`, then drops the table. Two-layer guard: compile-time feature gate + runtime `DATABASE_URL` env-var early-return.

## SC#4 Dep-Graph Assertion

`cargo tree -p ferro-ai --no-default-features` output contains **no `pgvector` line** and **no `sqlx` line** — confirmed by running the command during Task 2 verification. Under `--features pgvector` both appear as expected.

## Verification Results

- `cargo clippy -p ferro-ai --features pgvector -- -D warnings` — clean (Task 1)
- `cargo clippy -p ferro-ai --all-features --all-targets -- -D warnings` — clean (Task 2)
- `cargo test -p ferro-ai` — 91 unit tests passed, 0 failed (all Plan 01 tests preserved)
- `cargo test -p ferro-ai --features pgvector,postgres-tests` — compiles; `store_and_nearest_roundtrip` skips cleanly (no `DATABASE_URL`)
- `cargo fmt -p ferro-ai -- --check` — clean
- SC#4: `cargo tree -p ferro-ai --no-default-features` — no `pgvector`, no `sqlx` in output

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. `PgVectorStore::store` and `::nearest` are fully implemented. The integration test is fully implemented and skips gracefully without a live database.

## Threat Flags

No new network endpoints or auth paths introduced. `PgVectorStore` operates over a caller-supplied pool — trust boundary is already inside the caller's application. T-167-04 (SQL injection via table/column interpolation) is mitigated by design: `table` and `column` come from the `new()` constructor (trusted app code), never from request input; documented in the struct rustdoc. T-167-05 and T-167-06 accepted per plan threat register.

## Self-Check: PASSED

- ferro-ai/Cargo.toml contains `pgvector = ["dep:pgvector", "dep:sqlx"]`: FOUND
- ferro-ai/Cargo.toml contains `postgres-tests = ["pgvector"]`: FOUND
- ferro-ai/src/pgvector/mod.rs contains `pub struct PgVectorStore`: FOUND
- ferro-ai/src/pgvector/mod.rs contains `ON CONFLICT (id) DO UPDATE`: FOUND
- ferro-ai/src/pgvector/mod.rs contains `(1.0 - (`: FOUND
- ferro-ai/src/pgvector/mod.rs contains `Error::Sqlx(e.to_string())`: FOUND
- ferro-ai/src/lib.rs contains `#[cfg(feature = "pgvector")]`: FOUND
- ferro-ai/src/lib.rs contains `pub use pgvector::{Neighbor, PgVectorStore}`: FOUND
- ferro-ai/tests/pgvector_integration.rs contains `#![cfg(feature = "postgres-tests")]`: FOUND
- ferro-ai/tests/pgvector_integration.rs contains `DATABASE_URL`: FOUND
- Commits bf069c9a, 4fcae858, 580d9786: FOUND in git log
