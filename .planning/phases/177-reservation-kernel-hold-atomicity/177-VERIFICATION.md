---
phase: 177-reservation-kernel-hold-atomicity
verified: 2026-05-21T12:00:00Z
status: human_needed
score: 6/6 success criteria verified
overrides_applied: 0
human_verification:
  - test: "Run the Postgres-gated race test with a live Postgres instance: DATABASE_URL=postgres://user:pass@localhost:5432/ferro_test cargo test -p ferro-reservation --features postgres-tests hold_race_capacity_1_exactly_one_succeeds_postgres"
    expected: "50/50 iterations pass. The conflict-losing task receives ReservationError::Insufficient (translated from SQLSTATE 40001), not a raw DbErr. Zero flakes."
    why_human: "Requires a live Postgres instance. The test binary compiles clean (cargo build --tests --features postgres-tests exits 0) but cannot execute without DATABASE_URL. This is the only path that exercises the is_serialization_failure 40001-translation arm in a real SERIALIZABLE isolation scenario with true concurrent connections."
  - test: "Run the gestiscilo-it consumer regression: in gestiscilo-it repo, point ferro path dep at this local build, run cargo test --workspace, confirm concurrent_double_book_same_staff passes 5/5."
    expected: "The killer-feature acceptance test that triggered this phase (gestiscilo-it phase 152 STBOOK-15, Bug R5) passes deterministically after the hold fix."
    why_human: "Cross-repo regression requiring gestiscilo-it checkout with a patched Cargo.toml local-path dep. Cannot be automated from the ferro repo alone."
---

# Phase 177: reservation-kernel-hold-atomicity Verification Report

**Phase Goal:** Close the `ReservationKernel::hold` check-then-act race condition that allows two concurrent `tokio::spawn` tasks racing identical `(resource_kind, resource_key, window)` to both succeed when at most `capacity` should. Fix the kernel so the `held <= capacity` invariant holds under concurrent INSERTs.

**Verified:** 2026-05-21
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC-1: Two concurrent hold calls on same key with capacity=1 produce exactly 1 Ok + 1 Err(Insufficient), 50/50 iterations, zero flakiness | VERIFIED | `cargo test -p ferro-reservation hold_race_capacity_1_exactly_one_succeeds` exits 0; loop over 50 iterations each spawning 2 bare `tokio::spawn` tasks, no mutex. |
| 2 | SC-2: Two hold calls on non-overlapping keys both succeed (TestResource::Window=(), different keys used per locked discretion in CONTEXT.md) | VERIFIED | `cargo test -p ferro-reservation hold_non_overlapping_keys_both_succeed` exits 0; two sequential holds on "key_a" and "key_b" both succeed. |
| 3 | SC-3: Existing single-writer kernel tests pass byte-identical | VERIFIED | `cargo test -p ferro-reservation` exits 0; 27 unit tests in src/lib.rs + 3 integration + 2 property all pass. Total 36 tests. |
| 4 | SC-4: Fix path is (a) — `conn.begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite))` | VERIFIED | `kernel.rs` lines 73-79: `.begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite))`. D-06 locked path confirmed. |
| 5 | SC-5: Audit log row count == successful hold count; conflict-loser audit row rolls back | VERIFIED | `cargo test -p ferro-reservation hold_race_audit_atomicity_exactly_n_audit_rows` exits 0; asserts exactly 1 reservations row AND 1 audit_entries row with action "reservation.held" after capacity=1 race. |
| 6 | SC-6: docs/src/database/reservations.md no longer claims kernel does not arbitrate concurrent holds | VERIFIED | `grep -cE 'tokio::(sync::)?Mutex' docs/src/database/reservations.md` = 0; SERIALIZABLE count = 2; 40001 count = 1; "atomically arbitrates" present; "no application-layer mutex" appears twice; "on the roadmap as a follow-up" and "SQLite-validated" both absent. |

**Score:** 6/6 truths verified

### Deferred Items

None. All 6 success criteria verified. All 8 locked decisions verified (see table below).

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-reservation/src/kernel.rs` | Atomic `hold` with SERIALIZABLE txn; `is_serialization_failure` helper | VERIFIED | `begin_with_config(Serializable, ReadWrite)` at line 74; `audit.write(&txn)` at line 159; `txn.commit()` at line 164; `ferro_events::dispatch` after commit at line 189; two `is_serialization_failure` cfg arms at lines 446/458. |
| `ferro-reservation/tests/concurrent_hold.rs` | SC-1/SC-2/SC-5 tests, no tokio::Mutex | VERIFIED | 4 test functions present; 0 Mutex references; 50-iteration loops in 2 tests; `ferro_audit::history_for_target` called in SC-5 test. |
| `ferro-reservation/tests/concurrent_hold_postgres.rs` | Postgres cfg-gated mirror | VERIFIED | File exists; `#![cfg(feature = "postgres-tests")]` inner attribute at file top; 2 test functions (SC-1 50-iter race + SC-5 audit atomicity); compiles clean with `--features postgres-tests`. |
| `ferro-reservation/Cargo.toml` | Features section with sqlx-postgres + postgres-tests | VERIFIED | `[features]` section present; `sqlx-postgres = ["sea-orm/sqlx-postgres", "dep:sqlx"]`; `postgres-tests = ["sqlx-postgres"]`; `sqlx = { version = "0.8", optional = true }`. |
| `docs/src/database/reservations.md` | Stale concurrency advice replaced, SERIALIZABLE described | VERIFIED | All 10 SC-6 acceptance criteria pass (see above). |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `kernel.rs::hold` | `sea_orm::TransactionTrait::begin_with_config` | `.begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite))` | WIRED | Lines 73-79 of kernel.rs. Multi-line call confirmed. |
| `kernel.rs::hold` | `audit.write / am.insert / capacity / held` | All called with `&txn` inside transaction | WIRED | Lines 82-83 (capacity/held on &txn), line 137 (am.insert(&txn)), line 159 (audit.write(&txn)). |
| `kernel.rs::hold::txn.commit()` | `is_serialization_failure` | 40001 → Insufficient translation | WIRED | Lines 164-174: `txn.commit().await.map_err(|e| { if is_serialization_failure(&e) { ReservationError::Insufficient { ... } } else { ReservationError::Db(e) } })?` |
| `concurrent_hold.rs` | `kernel.hold` | bare `tokio::spawn` without Mutex | WIRED | 4 test functions use `tokio::spawn`; zero Mutex imports/references in file. |
| `Cargo.toml [features] postgres-tests` | `sqlx-postgres` feature | feature implication chain | WIRED | `postgres-tests = ["sqlx-postgres"]`; `sqlx-postgres = ["sea-orm/sqlx-postgres", "dep:sqlx"]`. |

### Data-Flow Trace (Level 4)

Not applicable — this phase produces test infrastructure and a kernel fix, not a rendering component with UI data flow. The kernel's own data flows are verified by the test suite (Level 3 wiring suffices).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SC-1: 50-iter race, capacity=1 | `cargo test -p ferro-reservation hold_race_capacity_1_exactly_one_succeeds` | 1 passed in 0.14s | PASS |
| SC-2: non-overlapping keys both succeed | `cargo test -p ferro-reservation hold_non_overlapping_keys_both_succeed` | 1 passed | PASS |
| SC-5: audit atomicity after race | `cargo test -p ferro-reservation hold_race_audit_atomicity_exactly_n_audit_rows` | 1 passed | PASS |
| SC-3: full suite regression | `cargo test -p ferro-reservation` | 36 passed; 0 failed | PASS |
| Clippy clean | `cargo clippy -p ferro-reservation --all-targets -- -D warnings` | exit 0 (no warnings) | PASS |
| Postgres feature compiles | `cargo build -p ferro-reservation --features postgres-tests --all-targets` | exit 0 | PASS |
| SC-6: no tokio::Mutex in docs | `grep -cE 'tokio::(sync::)?Mutex' docs/src/database/reservations.md` | 0 | PASS |
| SC-6: SERIALIZABLE in docs | `grep -c 'SERIALIZABLE' docs/src/database/reservations.md` | 2 | PASS |

### Locked Decision Verification

| Decision | Requirement | Status | Evidence |
|----------|-------------|--------|----------|
| D-01: `hold` signature gains only `+ TransactionTrait` bound | Surface stability | VERIFIED | `pub async fn hold<C: ConnectionTrait + TransactionTrait>` at kernel.rs line 57. All other parameters unchanged. |
| D-02: `commit`/`release`/`extend`/`run_sweep_once` byte-identical | No unintended changes | VERIFIED | `commit` at line 213, `release` at line 281, `extend` at line 372 all use `<C: ConnectionTrait>` (not TransactionTrait). `run_sweep_once` in sweeper.rs not touched. |
| D-03: SQLite + Postgres both supported | Backend portability | VERIFIED | `cargo test -p ferro-reservation` (SQLite) exits 0; `cargo build --features postgres-tests` (Postgres dep chain) exits 0. |
| D-04: Audit log atomicity preserved | `reservation.held` audit inside txn, rolls back on conflict | VERIFIED | `audit.write(&txn)` at line 159 is before `txn.commit()` at line 164. SC-5 test confirms exactly 1 audit row after 2-task race. |
| D-05: No new external crates (NOTE: sqlx was added as optional dep) | D-05 exception | VERIFIED (auto-fix) | `sqlx` added as `optional = true` dep. Required because `kernel.rs` directly references `sqlx::Error::Database` in the `#[cfg(feature = "sqlx-postgres")]` arm. SUMMARY-02 documents this as Rule 1 auto-fix (E0433 would fail otherwise). Transitive-via-sea-orm was insufficient for name resolution. |
| D-06: Fix path (a) locked — `begin_with_config(Serializable, ReadWrite)` | Correct isolation | VERIFIED | kernel.rs lines 73-79 confirm exact path (a) implementation. |
| D-07: SQLSTATE 40001 → Insufficient translation at kernel boundary | Error contract uniformity | VERIFIED | `is_serialization_failure` helper present with two cfg arms (lines 446/458). `txn.commit().await.map_err` closure translates 40001 to `ReservationError::Insufficient { requested: quantity, available: 0, capacity }`. |
| D-08: SQLite primary + Postgres cfg-gated | Test scope | VERIFIED | SQLite tests run unconditionally (4 tests in concurrent_hold.rs always execute). Postgres tests gated on `#![cfg(feature = "postgres-tests")]` inner attribute — `cargo test -p ferro-reservation -- --list` shows zero postgres test entries on default build. |

### Requirements Coverage

Phase 177 has no formal REQUIREMENTS.md IDs (`Requirements: TBD`). The 6 Success Criteria from ROADMAP.md and 8 locked decisions from CONTEXT.md serve as the requirement contract. All verified above.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `Cargo.toml` dev-dep `sea-orm` | 39 | `sqlx-postgres` always compiled in dev-dep sea-orm (not just under postgres-tests) | Info | Compiles sqlx-postgres into all test binaries even on default `cargo test` runs. Noted in REVIEW.md (IN-02). No correctness issue — the kernel's SQLSTATE detection arm is correctly cfg-gated on the crate's `sqlx-postgres` feature, not sea-orm's dev-dep feature state. |
| `concurrent_hold.rs` | 90,133,187,220 | `current_thread` flavor for all race tests | Warning | Under-stresses the Postgres SSI path (needs true parallelism to generate 40001). Noted in REVIEW.md (WR-01). For SQLite, `current_thread` is sufficient — the race window is between `.await` points. For Postgres, `multi_thread` would be more rigorous. Does not block SQLite-primary correctness claim. |
| `concurrent_hold_postgres.rs` | 41-50 | `fresh_pg_db()` panic message does not warn about destructive nature | Warning | Developer safety: a contributor with a non-test `DATABASE_URL` in environment could accidentally wipe tables. Noted in REVIEW.md (WR-02). No fix required for SC compliance. |

None of the above are blockers for goal achievement on the SQLite-primary path. All were flagged in the REVIEW.md.

### Human Verification Required

#### 1. Postgres SQLSTATE 40001 Translation Under Real Parallelism

**Test:** Set `DATABASE_URL=postgres://user:pass@localhost:5432/ferro_test` and run:
```bash
cargo test -p ferro-reservation --features postgres-tests hold_race_capacity_1_exactly_one_succeeds_postgres -- --nocapture
```
**Expected:** 50/50 iterations produce exactly 1 Ok + 1 Err(Insufficient). No `Err(Db(Exec(...)))` or `Err(Db(Query(...)))` variants — all serialization failures must be translated to `Insufficient` by `is_serialization_failure`. Zero test failures.

**Why human:** Requires a live Postgres instance. The test binary compiles clean (`cargo build --tests --features postgres-tests` exits 0), but the 40001-translation arm of `is_serialization_failure` cannot be exercised without a real Postgres SERIALIZABLE isolation conflict. This is the only path that validates D-07 end-to-end under actual concurrent database sessions.

Note: REVIEW.md WR-01 flags that `current_thread` flavor in `concurrent_hold_postgres.rs` may not generate true Postgres SSI conflicts. Consider switching to `#[tokio::test(flavor = "multi_thread", worker_threads = 4)]` for Postgres tests before accepting the 50-iteration result.

#### 2. Consumer Regression: gestiscilo-it concurrent_double_book_same_staff

**Test:** In gestiscilo-it repo, update `Cargo.toml` ferro path dep to this local build, then run:
```bash
cargo test --workspace -- concurrent_double_book_same_staff
```
**Expected:** Test passes 5/5 (currently fails 5/5 per CONTEXT.md — this is the killer-feature acceptance test Bug R5 that triggered Phase 177).

**Why human:** Cross-repo regression requiring gestiscilo-it checkout and patched Cargo.toml. Cannot be automated from the ferro repo. This is the load-bearing acceptance criterion for the phase — the fix was motivated by this specific consumer failure.

### Gaps Summary

No gaps in the SQLite-primary goal achievement. The kernel fix is complete, the serializable transaction is correctly implemented, all tests pass, and documentation is accurate. The two human verification items are:

1. **Postgres 40001 path** — the translation arm compiles correctly and is structurally sound, but needs a live Postgres to confirm it produces the right error type under real SSI contention.
2. **Consumer acceptance test** — the downstream gestiscilo-it test that triggered this phase needs to be run against the patched ferro to confirm the fix resolves the original failure.

These are validation steps, not implementation gaps. The code is complete and correct by all automated checks.

---

_Verified: 2026-05-21_
_Verifier: Claude (gsd-verifier)_
