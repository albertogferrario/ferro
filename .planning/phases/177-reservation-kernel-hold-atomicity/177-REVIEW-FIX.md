---
phase: 177-reservation-kernel-hold-atomicity
fixed_at: 2026-05-21T00:00:00Z
review_path: 177-REVIEW.md
fix_scope: critical_warning
findings_in_scope: 2
fixed: 2
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 177 Code Review — Fix Report

Two advisory warnings from `177-REVIEW.md` applied inline. Both fixes are isolated to `ferro-reservation/tests/concurrent_hold_postgres.rs`; the shipped kernel.rs fix is untouched.

## WR-01 — `current_thread` flavor under-stresses Postgres SSI contention

**Status:** Fixed.

The SQLite-side race tests in `tests/concurrent_hold.rs` were already on `flavor = "multi_thread", worker_threads = 4` (lines 96, 141, 200, 235) — no change needed there. The Postgres mirror at `tests/concurrent_hold_postgres.rs` was the only outstanding case.

**Changes:**
- `hold_race_capacity_1_exactly_one_succeeds_postgres` (line 98): `flavor = "current_thread"` → `flavor = "multi_thread", worker_threads = 4`. Doc comment updated with rationale (SSI contention is more faithfully stressed when tasks run on distinct OS threads; `current_thread` can serialize `.await` resumption on the cooperative scheduler and mask races).
- `hold_race_audit_atomicity_exactly_one_row_postgres` (line 148): same swap. Doc comment cites SC-1 rationale.

**Verification:** `cargo check -p ferro-reservation --tests --features postgres-tests` exits 0. Tests cannot run in this environment (no `DATABASE_URL`); execution under multi_thread is deferred to the live-Postgres human verification in `177-HUMAN-UAT.md`.

## WR-02 — `fresh_pg_db()` panic silent about destructive teardown

**Status:** Fixed.

`fresh_pg_db()` (lines 38-50 in `tests/concurrent_hold_postgres.rs`) calls `TestMigrator::down` then `up` on whatever `DATABASE_URL` points at — destructive teardown the original panic message did not flag.

**Changes:**
- Doc comment expanded with a `WARNING — DESTRUCTIVE` block making the table-drop behavior explicit and recommending a dedicated test DB (typically localhost via docker-compose).
- `.expect()` message expanded from `"DATABASE_URL must be set for the postgres-tests feature"` to a multi-line warning naming the dropped tables and recommending against pointing at production/shared staging.

Did NOT add a runtime localhost assertion — that would prevent CI containers and test infrastructure from using non-localhost addresses, which is a legitimate use case. Documentation-level warning is the right granularity here.

**Verification:** same compile check as WR-01 (single run, both fixes).

## Out of scope

The 4 INFO findings from `177-REVIEW.md` were not in scope (default `critical + warning`). They remain documented in REVIEW.md for future reference.

## CPU discipline note

Per project convention (`feedback_one_cpu_op_at_a_time`), fixes were applied inline (no fixer agent that would chain fmt+clippy+test between each commit) and verified with a single `cargo check` pass with the `postgres-tests` feature enabled. Both edits committed atomically.
