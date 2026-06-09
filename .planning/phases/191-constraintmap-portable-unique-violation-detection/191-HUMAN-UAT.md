---
status: partial
phase: 191-constraintmap-portable-unique-violation-detection
source: [191-VERIFICATION.md]
started: 2026-06-09T17:00:00Z
updated: 2026-06-09T17:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Postgres constraint-name identity match (`DatabaseError::constraint()`)
expected: Against a live Postgres instance, a duplicate insert into a table with a NAMED UNIQUE constraint, fed to `ConstraintMap::try_map` with a registration that has NO `.sqlite()` discriminator (Postgres-name match only), returns `Ok(ValidationError)` with the registered field (`ve.has("slug")` true) — proving the `PgDatabaseError::constraint()` runtime-dispatch branch matches on the structured constraint name (not message parsing). The shared `sql_err()` type gate and the entry-match loop are already fully SQLite-tested; only this Postgres-specific branch is unexercised by `cargo test`.
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
