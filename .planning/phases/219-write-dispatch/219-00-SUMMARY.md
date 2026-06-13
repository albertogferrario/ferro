---
phase: 219-write-dispatch
plan: "00"
subsystem: ferro-mcp-server, ferro-mcp-oauth
tags: [write-dispatch, mcp, guard-reevaluation, idempotency, tdd-red, security]
dependency_graph:
  requires: [218-write-tool-rendering-from-actiondef, 217-tenant-context-per-tenant-api-key-auth]
  provides: [WriteDispatcher type contract, ExecutorFn/GuardEvaluatorFn boxed-future types, MigrationMcpIdempotencyKeys, RED tests for SC#1/SC#3/SC#5]
  affects: [ferro-mcp-server, ferro-mcp-oauth]
tech_stack:
  added: [ferro-audit dep in ferro-mcp-server]
  patterns: [boxed-future async callback without async-trait, composite UNIQUE migration index, TDD RED phase]
key_files:
  created:
    - ferro-mcp-server/src/write_dispatch.rs
  modified:
    - ferro-mcp-server/src/error.rs
    - ferro-mcp-server/src/lib.rs
    - ferro-mcp-server/Cargo.toml
    - ferro-mcp-oauth/src/migration.rs
    - ferro-mcp-oauth/src/lib.rs
decisions:
  - Boxed-future pattern (Pin<Box<dyn Future>>) used for ExecutorFn/GuardEvaluatorFn to avoid adding async-trait dep to ferro-mcp-server
  - Wave 0 stubs return Err(Validation("not implemented")) not unimplemented!() so RED tests fail on assertion not panic
  - Test DB setup uses raw SQL (not MigratorTrait) to avoid async-trait/sea-orm-migration in ferro-mcp-server dev-deps
  - MigrationMcpIdempotencyKeys placed in ferro-mcp-oauth (same pattern as MigrationMcpApiKeys) not ferro-mcp-server (no migration infra)
  - Composite UNIQUE on (tenant_id, idempotency_key) not single-column to prevent cross-tenant replay (T-219-01)
metrics:
  duration: ~15 min
  completed: "2026-06-13"
  tasks: 3
  files: 6
---

# Phase 219 Plan 00: Write Dispatch Skeleton Summary

Wave 0 compiling skeleton + RED tests for the write dispatch surface. Defines every new type, error variant, migration, and module export so the workspace compiles; encodes the security spec in three failing unit tests before any implementation.

## What Was Built

**WriteDispatcher surface** (`ferro-mcp-server/src/write_dispatch.rs` — new):
- `ExecutorFn` and `GuardEvaluatorFn` boxed-future type aliases — no `async-trait` dep
- `WriteDispatcher` struct with public `executor` and `guard_evaluator` fields
- `dispatch_write` stub (returns `Err(Validation("not implemented"))`)
- `handle_write_call` stub (returns `-32601` error envelope)
- Wave 1 helper stubs with `#[allow(dead_code)]`: `find_action`, `validate_action_inputs`, `write_tool_error_result`, `lookup_idempotency`, `store_idempotency`

**Error variants** (`ferro-mcp-server/src/error.rs`):
- `ActionNotFound(String)` — maps to JSON-RPC -32601
- `GuardFailed(String)` — maps to structured isError:true result
- `Validation(String)` — maps to structured isError:true result

**Module export** (`ferro-mcp-server/src/lib.rs`):
- `pub mod write_dispatch` + `pub use write_dispatch::{dispatch_write, handle_write_call, WriteDispatcher}`

**Dependency** (`ferro-mcp-server/Cargo.toml`):
- `ferro-audit = { path = "../ferro-audit", version = "0.2" }` added

**Migration** (`ferro-mcp-oauth/src/migration.rs`):
- `MigrationMcpIdempotencyKeys` struct — creates `mcp_idempotency_keys` table
- Columns: `id` (PK), `tenant_id`, `idempotency_key`, `result` (TEXT/JSON), `created_at`
- COMPOSITE UNIQUE index `idx_mcp_idempotency_keys_tenant_key` on `(tenant_id, idempotency_key)`
- Non-unique index `idx_mcp_idempotency_keys_tenant_id` on `tenant_id` alone
- Exported as `CreateMcpIdempotencyKeysTable` from `ferro-mcp-oauth/src/lib.rs`
- Companion test `mcp_idempotency_keys_migration_creates_table_and_indexes` passes (GREEN)

**RED unit tests** (in `ferro-mcp-server/src/write_dispatch.rs`):
- `guard_denied_at_call_time` (SC#1/T-219-02): guard evaluator returns `Ok(false)` → asserts `Err(GuardFailed(_))` with executor never invoked; FAILS against stub (stub returns `Validation`)
- `idempotent_replay_does_not_re_execute` (SC#3/T-219-03): `Arc<AtomicUsize>` exec_count; two calls with same idempotency_key → asserts count==1 and equal results; FAILS against stub (stub always returns `Err`)
- `write_tool_result_parses_as_valid_mcp_content` (SC#5): drives `handle_write_call` for success and guard-denied cases → asserts `serde_json::from_value::<CallToolResult>` succeeds with correct `is_error` flag; FAILS against stub (stub returns `error` envelope, not `result`)

## Verification Results

| Check | Result |
|-------|--------|
| `cargo build -p ferro-mcp-server -p ferro-mcp-oauth --tests` | PASS (0 warnings) |
| `cargo clippy -p ferro-mcp-server --all-targets -- -D warnings` | PASS |
| `cargo clippy -p ferro-mcp-oauth --all-targets -- -D warnings` | PASS |
| `cargo test -p ferro-mcp-oauth mcp_idempotency_keys_migration` | PASS (GREEN — migration test) |
| `cargo test -p ferro-mcp-server --no-run` | PASS (RED tests compile) |
| `guard_denied_at_call_time` | FAILS (RED — expected) |
| `idempotent_replay_does_not_re_execute` | FAILS (RED — expected) |
| `write_tool_result_parses_as_valid_mcp_content` | FAILS (RED — expected) |
| `grep -qv "async-trait" ferro-mcp-server/Cargo.toml` | PASS (not added) |
| publish.yml wave order: ferro-audit Wave 1A < ferro-mcp-server Wave 2 | CONFIRMED (lines 211, 275) |

## Commits

| Hash | Description |
|------|-------------|
| 3d8393a1 | feat(219-00): WriteDispatcher/ExecutorFn/GuardEvaluatorFn skeleton + error variants + RED tests |
| 481ddb6f | feat(219-00): MigrationMcpIdempotencyKeys with composite UNIQUE (tenant_id, idempotency_key) |

## Deviations from Plan

**[Rule 2 - Missing critical functionality] Test DB setup uses raw SQL instead of MigrationMcpIdempotencyKeys**

- **Found during:** Task 3 (RED test implementation)
- **Issue:** The test module in `ferro-mcp-server` needs an idempotency table for SC#3. Using `MigratorTrait` (the pattern from `ferro-mcp-oauth` migration tests) requires `async_trait::async_trait` in `ferro-mcp-server`'s dev-deps, violating the acceptance criterion `grep -qv "async-trait" ferro-mcp-server/Cargo.toml`.
- **Fix:** Test `setup_db()` creates the `mcp_idempotency_keys` table via raw SQL matching the `MigrationMcpIdempotencyKeys` schema, avoiding any `async-trait` or `sea-orm-migration` dep in `ferro-mcp-server`.
- **Impact:** Wave 1 implementation tests can switch to the real migration if desired; the schema contract is identical.

## Threat Flags

None. No new network endpoints, auth paths, or file access patterns introduced. The migration and type definitions are schema/contract only.

## Self-Check: PASSED

Files created/modified:
- [x] `ferro-mcp-server/src/write_dispatch.rs` — exists, confirmed
- [x] `ferro-mcp-server/src/error.rs` — GuardFailed/ActionNotFound/Validation present
- [x] `ferro-mcp-server/src/lib.rs` — pub mod write_dispatch + pub use confirmed
- [x] `ferro-mcp-server/Cargo.toml` — ferro-audit dep present, no async-trait
- [x] `ferro-mcp-oauth/src/migration.rs` — MigrationMcpIdempotencyKeys present
- [x] `ferro-mcp-oauth/src/lib.rs` — CreateMcpIdempotencyKeysTable exported

Commits:
- [x] 3d8393a1 — confirmed in git log
- [x] 481ddb6f — confirmed in git log
