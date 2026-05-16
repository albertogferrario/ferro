---
phase: 153
plan: 04
subsystem: database
tags: [rust, sea-orm, audit-log, builder-pattern, uuid, sqlite]

requires:
  - phase: 153-03
    provides: entity.rs (DeriveEntityModel for audit_log) and migration.rs (CreateAuditLogTable)

provides:
  - AuditEntry type alias (entity::Model) with record() entry point
  - AuditEntryBuilder chainable builder with 7 setters
  - write<C: ConnectionTrait>(self, conn) — validates, INSERTs, re-fetches, returns AuditEntry
  - 5 happy-path unit tests (D-30-1..D-30-5)

affects: [153-05, 154-ferro-reservation]

tech-stack:
  added: []
  patterns:
    - "Type alias for SeaORM Model (pub type AuditEntry = entity::Model) for zero-cost query interop"
    - "Post-INSERT find_by_id re-fetch to populate DB-stamped created_at on UUID PK entities (RESEARCH Pitfall 1/F-12)"
    - "ActiveValue::NotSet for DB-defaulted columns so CURRENT_TIMESTAMP fires at INSERT time"
    - "ActiveValue::Set(None::<JsonValue>) for absent JSON columns — avoids SQL null pitfall (Pitfall 3)"
    - "sea_orm_migration::prelude::* in test module to make async_trait::async_trait available"

key-files:
  created: []
  modified:
    - ferro-audit/src/entry.rs

key-decisions:
  - "AuditEntry is a type alias (not a duplicate struct) for entity::Model — query helpers return Vec<AuditEntry> directly without conversion"
  - "Post-INSERT find_by_id(new_id).one(conn) re-fetch is mandatory — SQLite with UUID PK + DEFAULT CURRENT_TIMESTAMP does not return the server-stamped value in the INSERT response"
  - "ActiveValue::NotSet used for created_at so the DB CURRENT_TIMESTAMP default fires, not an application clock value"
  - "Missing action returns Err(AuditError::MissingAction) before any DB call — validated eagerly"
  - "Missing target writes successfully with tracing::warn! diagnostic — audit log must never refuse a write"
  - "Used sea_orm_migration::prelude::* (not just MigratorTrait) in test module to pull in async_trait re-export"

requirements-completed: [D-09, D-10, D-11, D-12, D-13, D-14, D-28, D-29, D-30]

duration: 2min
completed: 2026-05-13
---

# Phase 153 Plan 04: AuditEntryBuilder + write() + 5 Happy-Path Tests — Summary

**Chainable `AuditEntry::record(action).actor(…).target(…).write(&conn)` builder with mandatory post-INSERT UUID re-fetch for DB-stamped `created_at`, and 5 unit tests proving the write path end-to-end.**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-05-13T07:57:25Z
- **Completed:** 2026-05-13T07:59:38Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Overwrote `entry.rs` stub (33 lines) with full builder implementation (338 lines)
- `AuditEntry` is now a type alias for `entity::Model` — zero-cost interop with SeaORM queries
- `AuditEntry::record(action)` returns `AuditEntryBuilder` with `actor: AuditActor::System` default (D-10)
- Seven chainable setters all consuming `mut self -> Self` (workspace convention): `actor`, `target`, `before`, `after`, `reason`, `correlation`, `tenant`
- `write<C: ConnectionTrait>` validates non-empty action, warns on missing target via `tracing::warn!`, generates UUIDv4, inserts via SeaORM `ActiveModel`, re-fetches by id to populate DB-stamped `created_at`
- All 5 happy-path tests pass: `happy_path`, `missing_action`, `missing_target_writes`, `json_roundtrip`, `actor_null_id`
- Total crate tests: 18 (13 baseline + 5 new)

## Critical Implementation Notes

### Post-INSERT re-fetch (RESEARCH Pitfall 1 / F-12)

The `write()` method performs a mandatory `entity::Entity::find_by_id(new_id).one(conn).await?` re-fetch after INSERT. SeaORM's SQLite driver with UUID primary keys (`auto_increment = false`) does not return the `DEFAULT CURRENT_TIMESTAMP` value in the INSERT response. Without the re-fetch, the returned `AuditEntry.created_at` would be `NaiveDateTime::default()` (zero value). The `happy_path` test asserts `entry.created_at != NaiveDateTime::default()` to lock this behavior.

### ActiveValue::NotSet for created_at (D-22)

`created_at` is intentionally `ActiveValue::NotSet` in the `ActiveModel`. This lets the DB `CURRENT_TIMESTAMP` default fire at INSERT time, ensuring ordering correctness across multiple application servers.

### ActiveValue::Set(None::<JsonValue>) for absent JSON (RESEARCH Pitfall 3)

Absent `before` / `after` fields are set via `ActiveValue::Set(None::<serde_json::Value>)`. Using `ActiveValue::Set(Value::Null)` would store the string `"null"` instead of SQL NULL — a subtle bug caught by the research.

## Task Commits

1. **Task 1: AuditEntryBuilder + write() + 5 tests** — `4489d07b` (feat)

## Files Modified

- `ferro-audit/src/entry.rs` — overwritten: stub (33 lines) → full builder + tests (338 lines)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `async_trait` not resolved in test module**
- **Found during:** Task 1 (first test run)
- **Issue:** The test module used `#[async_trait::async_trait]` on `TestMigrator` but `async_trait` is not a direct dependency of `ferro-audit`. The `migration.rs` tests work because they use `use super::*` which pulls in `use sea_orm_migration::prelude::*` (which re-exports `async_trait`). The `entry.rs` test module only had `use sea_orm_migration::MigratorTrait` — too narrow.
- **Fix:** Changed to `use sea_orm_migration::prelude::*` in the test module — this re-exports `async_trait` as a proc-macro path, making `#[async_trait::async_trait]` resolve correctly.
- **Files modified:** `ferro-audit/src/entry.rs`
- **Verification:** `cargo test -p ferro-audit` exits 0 with 18 tests passing

**2. [Rule 1 - Style] rustfmt required `write()` signature on one line**
- **Found during:** Post-task `cargo fmt --all -- --check`
- **Issue:** `write<C: ConnectionTrait>(self, conn: &C) -> Result<AuditEntry, AuditError>` was formatted across 4 lines; `rustfmt` collapses it to one line.
- **Fix:** Applied rustfmt's preferred formatting.
- **Files modified:** `ferro-audit/src/entry.rs`
- **Verification:** `cargo fmt --all -- --check` exits 0

---

**Total deviations:** 2 auto-fixed (1 bug — missing import path, 1 style — rustfmt)
**Impact on plan:** Both fixes are mechanical. No scope creep. The first fix resolves a compile error; the second resolves a formatting gate.

## Issues Encountered

None — both deviations were caught immediately by the build/fmt gates and fixed in the same task.

## Stub Tracking

No stubs remain in `entry.rs`. The file previously had `#![allow(dead_code)]` and a bare struct definition. The full builder body is now in place. The `AuditEntryBuilder` type is not re-exported from `lib.rs` (callers only ever see it as the return value of `AuditEntry::record()` — they don't name the type directly). This is correct per D-09.

## Threat Surface Scan

No new network endpoints or auth paths introduced. The `write()` method constructs its INSERT via SeaORM's `ActiveValue::Set(value)` parameterized API — no raw SQL string concatenation (T-153-02 mitigated). The `created_at` column is `NotSet` so only the DB clock controls it (T-153-04-CLOCK mitigated). The post-INSERT re-fetch ensures the returned `AuditEntry.created_at` matches the actual DB value (T-153-04-REFETCH mitigated, verified by `happy_path` test).

## Next Phase Readiness

Plan 153-04 closes the write path gate. Plan 153-05 adds the query helpers (`history_for_target`, `recent_by_actor`, `recent`), the `reconstruct_state` replay helper, and `prune_older_than` with their 4 unit tests. The `entry.rs` type alias (`pub type AuditEntry = entity::Model`) means query results from plan 05 materialize directly as `AuditEntry` without conversion.

## Self-Check: PASSED

- [x] `ferro-audit/src/entry.rs` exists (338 lines, full body)
- [x] Contains `pub type AuditEntry = entity::Model;`
- [x] Contains `pub fn record(action: impl Into<String>) -> AuditEntryBuilder`
- [x] Contains `pub async fn write<C: ConnectionTrait>(self, conn: &C) -> Result<AuditEntry, AuditError>`
- [x] Contains `Uuid::new_v4()` — client-generated UUID
- [x] Contains `find_by_id(new_id)` — post-INSERT re-fetch
- [x] Contains `AuditError::MissingAction` — early validation
- [x] Contains `tracing::warn!` — missing-target diagnostic
- [x] Contains `ActiveValue::NotSet` — DB-default for created_at
- [x] 5 test functions: `happy_path`, `missing_action`, `missing_target_writes`, `json_roundtrip`, `actor_null_id`
- [x] Commit `4489d07b` present in git log
- [x] `cargo test -p ferro-audit` — 18/18 pass
- [x] `cargo clippy -p ferro-audit --all-targets -- -D warnings` — exit 0
- [x] `cargo fmt --all -- --check` — exit 0
