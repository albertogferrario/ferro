---
status: passed
phase: 191-constraintmap-portable-unique-violation-detection
source: [191-VERIFICATION.md]
started: 2026-06-09T17:00:00Z
updated: 2026-06-09T18:30:00Z
---

## Current Test

[all tests passed]

## Tests

### 1. Postgres constraint-name identity match (`DatabaseError::constraint()`)
expected: Against a live Postgres instance, a duplicate insert into a table with a NAMED UNIQUE constraint, fed to `ConstraintMap::try_map` with a registration that has NO `.sqlite()` discriminator (Postgres-name match only), returns `Ok(ValidationError)` with the registered field (`ve.has("slug")` true) — proving the `PgDatabaseError::constraint()` runtime-dispatch branch matches on the structured constraint name (not message parsing). The shared `sql_err()` type gate and the entry-match loop are already fully SQLite-tested; only this Postgres-specific branch is unexercised by `cargo test`.
result: passed — `framework/tests/constraint_map_pg_gate.rs::pg_constraint_name_identity_match` ran green against live Postgres (postgres@localhost:5432) on 2026-06-09. Named constraint `cw_pg_slug_key` matched via `constraint()` dispatch with no `.sqlite()` hint. Converted from a manual gate into an `#[ignore]`d, runnable-on-demand automated test (`DATABASE_URL=… cargo test -p ferro-rs --test constraint_map_pg_gate -- --ignored`).

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
