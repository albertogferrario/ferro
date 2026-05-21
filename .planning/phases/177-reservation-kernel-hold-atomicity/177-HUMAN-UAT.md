---
status: partial
phase: 177-reservation-kernel-hold-atomicity
source: [177-VERIFICATION.md]
started: 2026-05-21T00:00:00Z
updated: 2026-05-21T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Postgres SQLSTATE 40001 → ReservationError::Insufficient translation
expected:
- Start a Postgres instance reachable via `DATABASE_URL` (e.g., `docker run -p 5432:5432 -e POSTGRES_PASSWORD=test postgres:16`).
- Run: `DATABASE_URL=postgres://postgres:test@localhost:5432/postgres cargo test -p ferro-reservation --features postgres-tests`.
- Both `hold_race_capacity_1_exactly_one_succeeds_postgres` (50 iterations, capacity=1, racing 2 tasks) and `hold_race_audit_atomicity_exactly_one_row_postgres` must exit 0 with zero flakes across the 50 iterations.
- The kernel must translate Postgres SQLSTATE 40001 (serialization failure) into `ReservationError::Insufficient { requested, available: 0, capacity }` per locked decision D-07. If `40001` ever surfaces as `ReservationError::Db`, the translation site is wrong.
- Consider `flavor = "multi_thread"` if the `current_thread` runtime under-stresses real Postgres SSI contention (per code-review WR-01); test results from `current_thread` are still meaningful but may not exhaust the contention space.
result: [pending]

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
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
