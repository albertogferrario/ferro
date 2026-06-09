# Phase 191: ConstraintMap + Portable UNIQUE-Violation Detection - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-09
**Phase:** 191-constraintmap-portable-unique-violation-detection
**Mode:** `--auto` (recommended defaults auto-selected, grounded in codebase scout)
**Areas discussed:** API shape, Module home, Detection mechanism, Backend bifurcation, Match-key portability, Verification strategy

---

## API shape

| Option | Description | Selected |
|--------|-------------|----------|
| ROADMAP-literal `.on(constraint, field, message)` + `try_map → Result<ValidationError, DbErr>` | Matches success-criterion 1 verbatim; PG constraint name as primary key | ✓ |
| Closure-based `.map(|err| ...)` registration | More flexible but heavier call site; diverges from SC1 | |
| Attribute macro on handler | Out of scope; no runtime opt-in granularity | |

**Selected:** ROADMAP-literal builder (D-01, D-02).
**Notes:** SC1 specifies the exact signature; honoring it keeps the contract checkable.

## Module home

| Option | Description | Selected |
|--------|-------------|----------|
| `framework/src/validation/constraint_map.rs` | Coherent with the `ValidationError` target; satisfies ROADMAP key constraint | ✓ |
| `ferro-orm` (next to `GuardedUpdate`) | DB-adjacent but maps to a validation type that lives in `framework` | |

**Selected:** validation/ module (D-04).
**Notes:** ROADMAP: all Phase 191 impl in validation/; only Phase 192 mcp template is carved out.

## Detection mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| `DbErr::sql_err()` → `SqlErr::UniqueConstraintViolation` | Portable, sea-orm-native violation-type detection; confirmed in 1.1.20 | ✓ |
| Raw message-string matching on both backends | Postgres messages are unstable/localizable; rejected by VALID-05 | |

**Selected:** `sql_err()` for type detection (D-05).
**Notes:** Scout confirmed `sea-orm 1.1.20` exposes `sql_err()` and `SqlErr::UniqueConstraintViolation`.

## Backend bifurcation (identity)

| Option | Description | Selected |
|--------|-------------|----------|
| PG: `PgDatabaseError::constraint()`; SQLite: parse `table.column` from message | Matches ROADMAP SC3; structured on PG, message-token on SQLite (only option there) | ✓ |
| Parse constraint name from PG message string | Fragile, localizable; explicitly rejected by VALID-05 | |

**Selected:** structured PG name + SQLite message token (D-06).

## Match-key portability (genuine gray area)

| Option | Description | Selected |
|--------|-------------|----------|
| One entry stores BOTH ids: `.on(pg_name, field, msg).sqlite("table.column")` | Single registration portable across backends; explicit | ✓ |
| Auto-derive `table.column` from constraint name | Magic; constraint naming conventions vary — fragile | |
| Separate per-backend maps | Doubles consumer bookkeeping; error-prone | |

**Selected:** both identifiers per entry, optional `.sqlite(...)` (D-07).
**Notes:** Postgres-only deployments need only `.on(...)`; CI-on-SQLite needs the `.sqlite()` hint. Planner may refine exact spelling provided the behavioral contract holds.

## Verification strategy

| Option | Description | Selected |
|--------|-------------|----------|
| SQLite unit/serial tests + documented Postgres manual gate | Full SQLite coverage in `cargo test`; PG constraint-name signed off in VERIFICATION.md | ✓ |
| Require live Postgres in CI | Not provisioned in `cargo test` default; would block the suite | |

**Selected:** SQLite-automated + Postgres manual gate (D-10, D-11, D-12).
**Notes:** Mirrors the Phase 190 Postgres manual-gate precedent.

## Claude's Discretion

- Exact spelling of the SQLite discriminator API (`.sqlite(...)` vs `.on_sqlite(...)` vs `ConstraintId`).
- `try_map` internals (type-check then identity-match ordering; inner sqlx downcast for Postgres).
- `ConstraintMap` reuse model (per-handler construction recommended; no global state).

## Deferred Ideas

- Foreign-key / check / not-null constraint mapping (`SqlErr` also exposes FK violations) — UNIQUE-only this phase.
- ferro-mcp template + validation docs (VALID-06) — Phase 192.
