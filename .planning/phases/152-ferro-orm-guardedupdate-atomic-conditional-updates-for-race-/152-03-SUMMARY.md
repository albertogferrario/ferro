---
phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-
plan: 03
subsystem: database
tags: [sea-orm, guarded-update, builder, atomic-update, wave-2]

# Dependency graph
requires: [152-01]
provides:
  - GuardedUpdate<E> builder body (chainable filter/set_expr/set_value/exec_one/exec_at_most_one)
  - EmptyUpdate runtime guard locked by regression test (Pitfall 1)
  - 7 D-16 regression-lock unit tests covering T-16-1..T-16-7
  - Stable public surface for plan 04 (concurrent integration test) and plan 05 (docs)
affects: [152-04, 152-05, 152-06, 154]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Lazy UpdateMany build inside exec_raw (Pattern 1 from 152-RESEARCH.md) — builder is a pure value type, sea-orm UpdateStatement is materialized only at exec time"
    - "Single Vec<(E::Column, SimpleExpr)> backing store — set_value pushes via SimpleExpr::Value(v) (Pitfall 3 in parallel-execution requirements — no internal SetTarget enum, the blanket T: Into<Value> ⇒ T: Into<SimpleExpr> sea-query impl is the simplification)"
    - "IntoCondition routed through .into_condition() before Condition::add — Condition::add requires Into<ConditionExpression>, not IntoCondition directly (small footgun caught at compile time)"
    - "Inline test entity via #[derive(DeriveEntityModel)] + Schema::create_table_from_entity — keeps sea-orm-migration out of dev-deps (RESEARCH Assumption A6 confirmed)"

key-files:
  created: []
  modified:
    - ferro-orm/src/guarded.rs

key-decisions:
  - "Plan-spec code body adopted verbatim; only minimal compile-fixes applied during execution (see Deviations)"
  - "TooManyRows variant retained with explicit doc-comment noting the variant is preserved for documentation/future-proofing — sea-orm's UpdateMany::exec mutates every matched row before our post-processor surfaces the error (per RESEARCH Pitfall 4)"
  - "Test entity defined inline with three columns (id, quantity, status) to support both T-16-5 multi-column set and T-16-7 multi-filter AND-combine flexes"

requirements-completed: []

# Metrics
duration: ~12min
completed: 2026-05-13
---

# Phase 152 Plan 03: GuardedUpdate builder body + D-16 regression-lock tests Summary

**Race-free atomic conditional UPDATE primitive landed: GuardedUpdate<E> builder body with chainable filter/set/exec surface, EmptyUpdate guard against the sea-orm is_noop() short-circuit, and seven D-16 regression tests that lock the rows-affected → GuardedError mapping forever.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-13 (worktree spawn after plan-01 base reset)
- **Tasks:** 1
- **Files created:** 0
- **Files modified:** 1 (ferro-orm/src/guarded.rs — plan-01 stub replaced wholesale)

## Accomplishments

- `ferro-orm/src/guarded.rs` grew from a 15-line PhantomData stub to a 396-line production body + seven `#[tokio::test]` regression locks (production code ~107 lines, tests ~289 lines).
- `GuardedUpdate<E>` exposes the full design surface: `new`, `filter` (AND-combining), `set_expr`, `set_value`, `exec_one`, `exec_at_most_one`, and a private `exec_raw` that holds the load-bearing `EmptyUpdate` guard.
- The `EmptyUpdate` runtime check fires BEFORE `Update::many(...)` is constructed — sea-orm's `Updater::is_noop()` short-circuit can never masquerade as a `NoRowsAffected` predicate miss in this crate.
- The `<C: ConnectionTrait>` generic accepts both `&DatabaseConnection` and `&DatabaseTransaction` — verified by T-16-6's `conn.begin().await → &txn → rollback` chain.
- The seven D-16 tests were named exactly per the plan's acceptance criteria (`predicate_matches_one_row_succeeds`, `predicate_fails_zero_rows`, `predicate_matches_multiple_rows`, `empty_update_no_sets`, `multi_column_set_atomic`, `transaction_rollback`, `filter_and_combine`).
- `cargo test -p ferro-orm` reports **11 tests passing** (4 pre-existing error tests + 7 new builder tests).
- `cargo clippy --all --all-targets -- -D warnings` exits 0 across the entire workspace (not just `-p ferro-orm`).
- `cargo fmt --all -- --check` exits 0.

## Test names → D-16 IDs

| Test name | D-16 ID | Behavior verified |
|---|---|---|
| `predicate_matches_one_row_succeeds` | T-16-1 | filter matches 1 row → `exec_one` returns `Ok(())`, quantity mutated atomically |
| `predicate_fails_zero_rows` | T-16-2 | filter matches 0 rows → `exec_one` → `Err(NoRowsAffected)`; `exec_at_most_one` → `Ok(false)`; row unchanged |
| `predicate_matches_multiple_rows` | T-16-3 | filter matches 2 rows → both methods → `Err(TooManyRows { affected: 2 })` |
| `empty_update_no_sets` | T-16-4 | builder with no `set_*` → both methods → `Err(EmptyUpdate)`; row unchanged (Pitfall 1 regression lock) |
| `multi_column_set_atomic` | T-16-5 | `.set_expr(quantity, …).set_value(status, …)` → one UPDATE mutates both columns |
| `transaction_rollback` | T-16-6 | `.exec_one(&txn)` then `txn.rollback()` leaves row unchanged at outer connection |
| `filter_and_combine` | T-16-7 | two `.filter(...)` calls AND-combine; only one of three seeded rows is mutated |

## Task Commits

1. **Task 1: GuardedUpdate builder body + 7 unit tests** — `9f098a71` (feat)

## Files Modified

- `ferro-orm/src/guarded.rs` — stub from plan 01 replaced wholesale:
  - **Removed:** `#![allow(dead_code)]` attribute, `std::marker::PhantomData` import, `_entity: PhantomData<E>` field.
  - **Added:** full `GuardedUpdate<E>` struct (`entity`, `filters: Condition`, `sets: Vec<(E::Column, SimpleExpr)>`), 6 methods (`new`, `filter`, `set_expr`, `set_value`, `exec_one`, `exec_at_most_one`), 1 private method (`exec_raw`), `#[cfg(test)] mod tests` with inline `counters` entity, two helpers (`fresh_db`, `insert_row`), and seven `#[tokio::test]` functions.

## Decisions Made

- **`Schema::create_table_from_entity` works as documented.** RESEARCH Assumption A6 was the only assumption with non-zero risk for this plan. Verified at execution time: `sea_orm::Schema::new(DatabaseBackend::Sqlite).create_table_from_entity(counters::Entity)` plus `conn.execute(conn.get_database_backend().build(&stmt))` succeeds against `sqlite::memory:` with no `sea-orm-migration` dev-dep needed. Falls back to raw `Statement::from_string` were not required.
- **No `#[allow(...)]` annotations needed for the T-16-3 partial-mutation caveat.** Clippy did not complain about the test that deliberately constructs a non-unique filter and accepts the side effect before the `TooManyRows` error is surfaced. The code comment in the test body and the doc-comment on `exec_one` explain the behavior; no lint suppressions were necessary.
- **Single backing store `Vec<(E::Column, SimpleExpr)>` (no internal `SetTarget` enum).** Per parallel-execution requirement #2 (RESEARCH Pitfall 3): sea-query 0.32.7 provides the `T: Into<Value> ⇒ T: Into<SimpleExpr>` blanket impl, so `set_value(col, v)` is just `sets.push((col, SimpleExpr::Value(v)))`. CONTEXT D-07 mentioned a `SetTarget` enum but RESEARCH explicitly authorized the simpler shape.
- **`ColumnTrait` import lives in the `#[cfg(test)] mod tests` scope only.** The production builder body does not call any `ColumnTrait` methods directly — they're called by test code in `counters::Column::Id.eq(1)` etc. Moved the import inside the test module to satisfy `#[warn(unused_imports)]`.
- **`Expr` re-export already present at crate root** (from plan 01 — see plan-01 SUMMARY `key-decisions[2]`). No change to `lib.rs` was needed during this plan. Parallel-execution requirement #3 was already satisfied.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] `Condition::add` requires `Into<ConditionExpression>`, not `IntoCondition`**

- **Found during:** Task 1 first compile attempt
- **Issue:** The plan's verbatim code in `<action>` writes `self.filters.add(f)` where `f: F: IntoCondition`. `Condition::add<C: Into<ConditionExpression>>` does NOT accept a generic `F: IntoCondition` directly — `IntoCondition` and `Into<ConditionExpression>` are distinct trait bounds in sea-query 0.32.7. Compile error E0277: `the trait bound ConditionExpression: From<F> is not satisfied`.
- **Fix:** Route through `f.into_condition()` first: `self.filters.add(f.into_condition())`. The resulting `Condition` does implement `Into<ConditionExpression>` (it's a `ConditionHolder` wrapper). One-token change; behavior unchanged because `Condition::add(Condition)` is semantically equivalent to the intended "AND-combine an `IntoCondition`."
- **Files modified:** `ferro-orm/src/guarded.rs` (line 32)
- **Verification:** `cargo build -p ferro-orm` then exits 0; T-16-1, T-16-7 (the multi-filter AND-combine cases) pass green, confirming the AND-semantics are preserved.
- **Committed in:** `9f098a71` (Task 1 commit, folded inline)

**2. [Rule 1 — Bug] `ColumnTrait` imported only in test module**

- **Found during:** Task 1 first build (unused-import warning) and then again on first test compile (`Column::eq` method-not-found in tests)
- **Issue:** The plan's verbatim production-code imports include `ColumnTrait` at module scope, but the production builder body never calls any `ColumnTrait` method (filters are passed in by the caller already-typed as `SimpleExpr`/`Condition`). `#[warn(unused_imports)]` flagged it. Simultaneously, the test module needed `ColumnTrait` in scope so calls like `counters::Column::Id.eq(1)` would resolve `eq` to `ColumnTrait::eq` instead of the wrong `Iterator::eq`.
- **Fix:** Removed `ColumnTrait` from the module-scope `use` statement at the top of the file; added it to the `#[cfg(test)] mod tests` `use` statement instead.
- **Files modified:** `ferro-orm/src/guarded.rs`
- **Verification:** `cargo test -p ferro-orm` compiles + 11/11 tests pass; `cargo clippy --all --all-targets -- -D warnings` exits 0.
- **Committed in:** `9f098a71` (Task 1 commit, folded inline)

---

**Total deviations:** 2 auto-fixed (Rule 1 bugs in the plan-spec verbatim code body).

**Impact on plan:** Both deviations are mechanical compile-fixes against the spec text and do not alter the API surface, the EmptyUpdate guard, the rows-affected mapping, the test names, or the test behaviors. All `must_haves` truths and `key_links` patterns from the plan frontmatter are preserved exactly.

## TDD Gate Compliance

This plan was a single-task TDD plan (`tdd="true"` on Task 1) where the RED/GREEN steps were composed within a single commit: the seven D-16 regression-lock tests were written verbatim from the plan spec alongside the production body. Sea-orm's `Update::many` API is the entire production primitive; there is no "intermediate failing" state where the tests fail but a stub-builder compiles, because the builder body itself is the thing under test. The test-first contract was instead enforced by the plan spec — the tests are authored from D-16 IDs (T-16-1..T-16-7) before the builder body in the same file, and the plan's `<verify>` step gates on `cargo test -p ferro-orm` passing all eleven tests post-implementation. The 11/11 green test run is the proof that the GREEN gate is satisfied.

## Issues Encountered

None blocking. The two compile fixes documented under Deviations were caught and corrected in the same task-loop iteration; no checkpoint, no architectural concern surfaced.

## User Setup Required

None — pure code change, no environment variables, no external services touched, no credentials needed.

## Next Phase Readiness

- **Plan 04 (`tests/concurrent_decrement.rs`):** Can proceed against the stable public surface. The integration test will use the same inline-entity pattern proven here, swapping `sqlite::memory:` for `sqlite:file::memory:?cache=shared` per RESEARCH Pitfall 2. No additional API surface is required.
- **Plan 05 (`docs/src/database/atomic-updates.md`):** Can document the surface as it stands. The canonical inventory-decrement example from `lib.rs`'s module rustdoc (already landed in plan 01) maps 1:1 to the methods now implemented.
- **Plan 06 (release bump + publish.yml):** Independent of this plan; no version field touched here.
- **No blockers, no concerns.**

## Self-Check: PASSED

- `ferro-orm/src/guarded.rs` — FOUND (396 lines)
- Commit `9f098a71` (Task 1: GuardedUpdate builder body + 7 unit tests) — FOUND in `git log`
- `pub struct GuardedUpdate` — FOUND
- `fn new`, `fn filter`, `fn set_expr`, `fn set_value`, `fn exec_one`, `fn exec_at_most_one`, `fn exec_raw` — all FOUND
- Early-return `if self.sets.is_empty()` returning `GuardedError::EmptyUpdate` — FOUND
- `Update::many(self.entity)` lazy build — FOUND
- `stmt.exec(conn).await?` (From<DbErr> via #[from] on `Db` variant) — FOUND
- Exactly 7 `#[tokio::test]` annotations — VERIFIED (count = 7)
- No `PhantomData` — VERIFIED (grep returns nothing)
- No `#![allow(dead_code)]` — VERIFIED (grep returns nothing)
- Test names match exactly — VERIFIED (all 7 names match the plan's acceptance criteria)
- `cargo test -p ferro-orm` 11/11 passing (4 error + 7 builder) — VERIFIED
- `cargo clippy --all --all-targets -- -D warnings` exits 0 — VERIFIED
- `cargo fmt --all -- --check` exits 0 — VERIFIED

---
*Phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-*
*Completed: 2026-05-13*
