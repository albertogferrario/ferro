---
status: complete
phase: 239-soft-delete-data-model-deleted-at-migration
source: [239-VERIFICATION.md]
started: 2026-06-23T00:00:00.000Z
updated: 2026-07-28T00:00:00.000Z
---

## Current Test

Postgres path verified 2026-07-28 via Docker (postgres:16-alpine, port 5433).

## Tests

### 1. Postgres migration path for `deleted_at`
expected: Running the app's `db:migrate` against a live Postgres `DATABASE_URL` applies `m20260623_add_deleted_at_to_orders` cleanly; the `orders.deleted_at` column exists with `is_nullable = YES` and `column_default = NULL`.
result: PASS — `DATABASE_URL=postgres://postgres:test@localhost:5433/ferro_test cargo run -p app -- db:migrate` exited 0 ("Migrations completed successfully!"). `information_schema.columns` confirmed: `data_type = timestamp without time zone`, `is_nullable = YES`, `column_default = NULL`.

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
