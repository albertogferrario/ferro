# Phase 177: ferro-reservation Kernel Atomicity Hardening — `hold` race fix — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 177-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-21
**Phase:** 177-reservation-kernel-hold-atomicity
**Mode:** `--auto` (Claude selected recommended defaults for every open decision; no interactive questions asked)
**Areas discussed:** Fix path, Postgres isolation strategy, Concurrency test scope, Audit-log atomicity, `40001` error translation

---

## Fix path (Path a / b / c)

| Option | Description | Selected |
|--------|-------------|----------|
| (a) `conn.begin()` transaction with serializable isolation | Wrap `hold` body in a SeaORM transaction. Mirrors existing `commit/release/sweeper` GuardedUpdate discipline. Backend-portable (SQLite serializes writers natively; Postgres uses `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE`). Conflict-losing task receives `Err(Insufficient)` after the second read inside the txn sees the just-inserted row. | ✓ (auto: recommended default) |
| (b) Unique partial index on `(resource_kind, resource_key, window_hash) WHERE status='held'` | Schema-level enforcement — second INSERT fails with a unique-constraint violation. | |
| (c) `INSERT … SELECT … WHERE NOT EXISTS` atomic check-and-insert | One SQL statement, atomic at the DB layer. | |

**Auto-mode rationale:** Path (a) was explicitly flagged as the recommendation in the prior context with "Smallest delta, matches existing GuardedUpdate discipline, no schema migration, works with `capacity > 1`". Path (b) breaks `capacity > 1` resources (multiple legitimate `held` rows for the same `(key, window)`). Path (c) loses the `Resource::capacity()` extension point and forces raw SQL with backend-branched escaping.

**Notes:** Locked as D-06. Rejected paths preserved in CONTEXT.md under "Rejected Paths (kept for plan-time blocker recovery)" so the planner can re-open if research surfaces a blocker — but a re-open must surface as an explicit checkpoint, not a silent path switch.

---

## Postgres isolation strategy

| Option | Description | Selected |
|--------|-------------|----------|
| (i) `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE` inside the txn | Single statement issued at txn start. SQLSTATE `40001` (serialization failure) may surface on the conflict-losing task. | ✓ (auto: simpler per Specifics note) |
| (ii) `SELECT ... FOR UPDATE` row-level locking on a key sentinel row | Pessimistic row lock; first writer blocks the second until commit. Requires a sentinel row to exist (or a `LOCK TABLE` workaround). | |

**Auto-mode rationale:** The CONTEXT.md "Specifics" section flagged Option (i) as "simpler". Option (ii) has subtleties (sentinel row creation/upsert, `LOCK TABLE` blast radius, no clean SQLite analog) that would expand the phase scope.

**Notes:** Locked as D-07. The `40001` translation site is a Claude's Discretion item — default is to translate at the kernel boundary into `ReservationError::Insufficient` so the consumer field test (`concurrent_double_book_same_staff`) sees the documented error variant.

---

## Concurrency test scope

| Option | Description | Selected |
|--------|-------------|----------|
| SQLite primary + Postgres cfg-gated | Unconditional SQLite in-memory tests; Postgres mirror gated on `cfg`/feature flag. CI can run Postgres tests when docker-compose Postgres is available. | ✓ (auto: as documented) |
| SQLite only | Skip Postgres tests entirely; rely on the SeaORM abstraction layer for portability claims. | |
| Postgres primary | Force every contributor to run Postgres locally. | |

**Auto-mode rationale:** The CONTEXT.md "Concurrency test infrastructure" section already specified this shape ("an in-memory SQLite (and ideally one against a docker-compose Postgres too, gated on `cfg(feature = "postgres")` or `cfg(test_postgres)`)"). Auto-mode locks it as D-08.

**Notes:** Exact cfg name (`feature = "postgres-tests"` vs `cfg(test_postgres)` vs `DATABASE_URL`-based) is Claude's Discretion — planner picks whichever matches existing ferro-reservation conventions.

---

## Audit-log atomicity

| Option | Description | Selected |
|--------|-------------|----------|
| Audit write INSIDE the atomic block | Conflict-losing task's audit row is rolled back with its transaction — `audit_entries` count for the `(key, window)` equals successful holds. Preserves D-04 invariant ("audit row written exactly once per successful hold, never for the conflict-losing task"). | ✓ (auto: locked at scope time as D-04) |
| Audit write OUTSIDE / after commit | Conflict-losing task could write an audit row before the conflict is detected — violates D-04. | |

**Auto-mode rationale:** D-04 was already locked in the original 2026-05-20 scope. Auto-mode confirms by adding the explicit test (concurrency.rs case 4 — "audit-log atomicity") that asserts the audit row count matches the successful hold count.

---

## `40001` Postgres error translation

| Option | Description | Selected |
|--------|-------------|----------|
| (i) Translate `40001` → `ReservationError::Insufficient` at the kernel boundary | Preserves the documented error contract. Consumer field test expects `Err(Insufficient)` not a db-error variant. | ✓ (auto: default for plans 01+02) |
| (ii) Surface `40001` raw as `ReservationError::Db`; require caller-retry | More transparent about what happened, but breaks the consumer field test expectation and forces every consumer to handle a new error shape. | |

**Auto-mode rationale:** The consumer killer-feature acceptance test (`concurrent_double_book_same_staff`) explicitly asserts `Err(ReservationError::Insufficient)` for the conflict-losing task. Option (ii) would force a consumer-side change and break the documented error contract.

**Notes:** Locked under D-07. The exact translation site (kernel boundary vs txn-retry helper) is Claude's Discretion — both are equivalent if the planner's chosen structure makes one cleaner.

---

## Claude's Discretion

Captured in CONTEXT.md under "Claude's Discretion":

- Exact Postgres cfg name (`feature = "postgres-tests"` vs `cfg(test_postgres)` vs `DATABASE_URL`-gated)
- Iteration count for race-to-capacity test (≥50, plain loop vs `proptest`/`quickcheck`)
- `40001` translation site (kernel boundary vs txn-retry helper)
- Whether to extract a `hold_inner(&txn, ...)` helper
- Doc-update sweep (kernel.rs module doc, PITFALLS T-69-1.2, any consumer-facing docs claiming "the kernel arbitrates concurrent holds")

---

## Deferred Ideas

None. The phase scope is tightly bounded to the `hold` atomicity invariant. Adjacent improvements (Resource trait surface changes, event-sourced reservations, performance optimization) are explicitly out of scope per CONTEXT.md "Phase Boundary".
