---
status: testing
phase: 177-reservation-kernel-hold-atomicity
source: [177-VERIFICATION.md]
started: 2026-05-21T00:00:00Z
updated: 2026-05-21T00:30:00Z
---

## Current Test

number: 2
name: gestiscilo-it consumer regression — `concurrent_double_book_same_staff`
expected: cross-repo test now passes 5/5
awaiting: user response (Test 1 surfaced a pre-existing Postgres migration gap — see Gaps section)

## Tests

### 1. Postgres SQLSTATE 40001 → ReservationError::Insufficient translation
expected:
- Start a Postgres instance reachable via `DATABASE_URL` (e.g., `docker run -p 5432:5432 -e POSTGRES_PASSWORD=test postgres:16`).
- Run: `DATABASE_URL=postgres://postgres:test@localhost:5432/postgres cargo test -p ferro-reservation --features postgres-tests`.
- Both `hold_race_capacity_1_exactly_one_succeeds_postgres` (50 iterations, capacity=1, racing 2 tasks) and `hold_race_audit_atomicity_exactly_one_row_postgres` must exit 0 with zero flakes across the 50 iterations.
- The kernel must translate Postgres SQLSTATE 40001 (serialization failure) into `ReservationError::Insufficient { requested, available: 0, capacity }` per locked decision D-07. If `40001` ever surfaces as `ReservationError::Db`, the translation site is wrong.
- Consider `flavor = "multi_thread"` if the `current_thread` runtime under-stresses real Postgres SSI contention (per code-review WR-01); test results from `current_thread` are still meaningful but may not exhaust the contention space.
result: pass
resolved_by:
  - "ferro-reservation/src/migration.rs: ResourceKey + Window columns switched from `.json()` to `.json_binary()` (sea-orm jsonb mapping); btree index on these columns now valid on Postgres."
  - "ferro-reservation/src/kernel.rs: D-07 SQLSTATE 40001 translation extended to INSERT (`am.insert`) and audit-write sites (not just `txn.commit`). Postgres SSI can fire 40001 mid-write during pivot detection — the original commit-only translation missed that path. Added `audit_error_db()` helper to probe through AuditError for inner DbErr."
  - "ferro-reservation/tests/concurrent_hold_postgres.rs: module doc updated to require `-- --test-threads=1` (Postgres tests share a live DB and must serialize TestMigrator up/down at the harness level; cargo's default parallelism races on pg_catalog.pg_type)."
verification:
  command: "DATABASE_URL=postgres://postgres:test@localhost:5432/postgres cargo test -p ferro-reservation --features postgres-tests --test concurrent_hold_postgres -- --test-threads=1"
  outcome: "2 passed; 0 failed (1.52s) — 50-iteration race + audit-atomicity both clean against real Postgres SERIALIZABLE isolation."
sqlite_regression: "cargo test -p ferro-reservation still 36/36 pass byte-identical."
diagnosis_journey:
  - "Round 1: migration failed with SQLSTATE 42704 (json + btree). Pre-existing latent bug, not phase-177 regression."
  - "Round 2: after .json_binary() fix, race test iteration 6 surfaced 40001 as Db(Query(...)) instead of Insufficient. is_serialization_failure already matched both Exec and Query DbErr variants — the bug was that translation only applied at txn.commit(), not at the INSERT/audit ? propagation sites. Fixed by adding translation closures at each ? site that can produce a 40001-bearing DbErr."
  - "Round 3 also hit parallel-test race on pg_catalog.pg_type (SQLSTATE 23505 audit_log duplicate). Resolved with --test-threads=1 documentation; deferred a serial_test dep until/unless it's actually needed."

### 2. gestiscilo-it consumer regression — `concurrent_double_book_same_staff`
expected:
- Point gestiscilo-it's `Cargo.toml` at the local patched ferro path (e.g. `ferro-reservation = { path = "../ferro/ferro-reservation" }`).
- From gestiscilo-it repo root: `cargo test --workspace concurrent_double_book_same_staff` (or whichever filter targets the test).
- The test must now PASS 5/5 deterministically — it previously failed 5/5 against the unpatched ferro per the consumer field test (gestiscilo-it phase 152 STBOOK-15 Bug R5).
- Also run gestiscilo-it's Phase 130/131/132 inventory test suite — must remain green (SC-3 byte-identical single-writer behavior).
- If `concurrent_double_book_same_staff` still fails, the kernel fix has a bug not caught by the in-workspace tests — surface immediately as a gap.
result: [pending]

## Summary

total: 2
passed: 1
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps

- truth: "ferro-reservation migrations run cleanly on Postgres so that the kernel atomicity fix can be exercised against real SERIALIZABLE isolation and SQLSTATE 40001 translation can be verified end-to-end."
  status: resolved
  reason: "Postgres SQLSTATE 42704 at TestMigrator::up(): 'data type json has no default operator class for access method btree'. The idx_reservations_kind_key_window_status index (migration.rs:84-94) is a btree index over ResourceKey + Window columns. Both columns are declared via sea-orm `.json()` which maps to Postgres `json` (NOT `jsonb`). Postgres can only btree-index `jsonb`, not `json`. SQLite is permissive about this and never exposed the issue."
  severity: blocker
  test: 1
  artifacts:
    - ferro-reservation/src/migration.rs:50  # ResourceKey column declared `.json()`
    - ferro-reservation/src/migration.rs:52  # Window column declared `.json()`
    - ferro-reservation/src/migration.rs:84-94  # btree index over those columns
    - ferro-reservation/src/entity.rs:24  # ResourceKey: JsonValue
    - ferro-reservation/src/entity.rs:27  # Window: Option<JsonValue>
  missing:
    - Migration column type that's Postgres-indexable: switch `.json()` to `.json_binary()` (sea-orm's jsonb mapping) for ResourceKey and Window, OR drop ResourceKey + Window from the btree index and add a separate non-btree access path
    - A migration-level Postgres CI step so future migrations are tested against real Postgres before merge (not just SQLite)
  scope_decision: |
    This is a PRE-EXISTING latent bug, not a phase-177 regression. The phase-177 kernel atomicity fix
    (CONTEXT.md D-06/D-07/SC-1/SC-5) is structurally correct and SQLite-verified (36/36 tests pass).
    The Postgres path however cannot be exercised end-to-end until the migration is repairable.

    Recommended path: open a follow-up phase (e.g., 177.1 "ferro-reservation Postgres migration
    repair") that:
      1. Changes ResourceKey/Window from `.json()` to `.json_binary()` (sea-orm jsonb).
      2. Adds a CI lane (or local docker-compose target) that runs cargo test --features
         postgres-tests against a real Postgres so this never reaches a consumer again.
      3. Confirms the phase-177 race + audit-atomicity tests pass on Postgres (closing this gap
         and the D-07 verification debt simultaneously).

    Phase 177 itself can be marked accepted-with-caveats: SQLite path proven, Postgres path
    blocked on the migration repair downstream phase.
