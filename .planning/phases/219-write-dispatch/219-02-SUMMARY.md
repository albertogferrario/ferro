---
phase: 219-write-dispatch
plan: "02"
subsystem: app (sample application) + ferro-mcp-server
tags: [write-dispatch, tenant-isolation, audit, idempotency, bola-prevention, end-to-end]
dependency_graph:
  requires: ["219-01"]
  provides: ["SC#2-cross-tenant-denial", "SC#3-e2e-idempotency", "SC#4-audit-recovery", "all-five-SCs-green"]
  affects: ["app/src/models/orders.rs", "app/src/controllers/mcp.rs", "app/src/migrations/mod.rs"]
tech_stack:
  added: ["ferro-audit (app dependency)", "local migration wrapper pattern"]
  patterns: ["find-then-mutate cross-tenant denial", "live guard evaluator via raw SQL COUNT", "SeaORM SQLite in-memory test isolation", "DeriveMigrationName collision avoidance via local wrappers"]
key_files:
  created:
    - app/src/migrations/m20260614_create_audit_log_table.rs
    - app/src/migrations/m20260614_create_mcp_idempotency_keys_table.rs
    - app/src/tests/mcp_write_dispatch.rs
  modified:
    - app/src/models/orders.rs
    - app/src/controllers/mcp.rs
    - app/Cargo.toml
    - app/src/migrations/mod.rs
    - app/src/tests/mcp_tenant_isolation.rs
    - app/src/tests/mod.rs
    - ferro-mcp-server/src/write_dispatch.rs
decisions:
  - "DeriveMigrationName collision resolved via local wrapper files: both external migration structs in files named migration.rs derive the same seaql_migrations version string; local wrapper files with date-prefixed names break the tie"
  - "TenantScoped::find_for_tenant uses ferro::DB::connection() (global pool) in production; test dispatcher captures explicit db clone passed through the executor closure arg — no global state in tests"
  - "Async closure lifetime trap: action_name (&str) and db (&DatabaseConnection) cannot be captured by async move; fix is to convert to owned values (to_string(), clone()) before the Box::pin(async move { ... }) block"
  - "CallToolResult not available directly in app crate (rmcp indirect dep); test assertions use raw JSON checks on result[\"result\"][\"isError\"] rather than typed deserialization"
metrics:
  duration: "~2 sessions"
  completed: "2026-06-14"
  tasks: 3
  files_changed: 11
requirements_covered: [AMCP-04]
---

# Phase 219 Plan 02: Write Dispatch — App Wiring + SC#2/#4/#3 End-to-End Summary

One-liner: Tenant-scoped Order executor + live guard evaluator threaded into handle_tools_call, with cross-tenant denial (SC#2), audit recovery (SC#4), and e2e idempotency (SC#3) proven via SQLite in-memory fixtures.

## What Was Built

This plan wired the `app` sample application to the `dispatch_write` pipeline established in Plan 01, proving the remaining three success criteria (SC#2, SC#3 e2e, SC#4) against a real SeaORM executor running under in-memory SQLite.

### Task 1: TenantScoped on Order + ferro-audit dep + migration registration

- Added `TenantScoped` impl to `app/src/models/orders.rs` (extension file; entities file is auto-generated and was not touched). The impl uses `Entity::find_by_id(id).filter(Column::TenantId.eq(tenant_id))` — the cross-tenant denial primitive (T-219-01). `None` from the query propagates to the executor as the BOLA-prevention signal.
- Added `ferro-audit = { path = "../ferro-audit", version = "0.2" }` to `app/Cargo.toml`.
- Registered idempotency and audit migrations in `app/src/migrations/mod.rs` via local wrapper files (see Deviations).

### Task 2: make_write_dispatcher + call site repair

- Implemented `check_is_manager(tenant_id, db)` — live SQL COUNT against `users` table; Postgres / SQLite backend dispatch via `ConnectionTrait::get_database_backend()`.
- Implemented `make_write_dispatcher() -> WriteDispatcher`:
  - executor: parses `id` from inputs, calls `Entity::find_by_id(id).filter(Column::TenantId.eq(tenant_id))` (inlining the TenantScoped logic for direct db arg plumbing), returns denial error on `None`, otherwise applies submit/approve/ship state transition via SeaORM `ActiveModel`, records ferro-audit entry via `AuditEntry::record(...).write(db)`, stores idempotency key via `store_idempotency(...)`.
  - guard_evaluator: `"is_manager"` → `check_is_manager(tenant_id, db)`; unknown guard → `Err(GuardFailed(...))` (fail-closed, per code-review fix CR-02 commit `127c3ada` — an earlier draft defaulted to `Ok(true)`, the fail-open bug the review caught).
- Repaired the `handle_tools_call` call site (broken by Plan 01's new `dispatcher` param): `let dispatcher = make_write_dispatcher();` + passes `&dispatcher` as 6th arg.
- Fixed `mcp_tenant_isolation.rs` (existing test file also calling `handle_tools_call`): added `noop_dispatcher()` helper returning a no-op `WriteDispatcher` and passed it at both call sites.

### Task 3: SC#2/#4/#3 e2e fixtures + full CI gate

Created `app/src/tests/mcp_write_dispatch.rs` with:
- `setup_db()`: `Database::connect("sqlite::memory:")` + `Migrator::up(&db, None)` for full schema isolation.
- `seed_two_tenants()`: inserts tenants (ids 1, 2), users (ids 901, 902), orders for tenant 1 (ids 1, 2 status "submitted") and tenant 2 (ids 3, 4 status "submitted").
- `make_test_write_dispatcher(db)`: test-local dispatcher capturing `db.clone()`; mirrors the production dispatcher without touching the global connection pool.
- `call_write_tool()`: wraps `handle_tools_call` with `scope: Some("read_write")`.
- `cross_tenant_write_denied` (SC#2): submits to order id=3 as tenant 1, asserts `result["result"]["isError"] == true`, asserts order 3 `status == "submitted"` (unmutated — BOLA prevented).
- `write_call_produces_audit_entry` (SC#4): submits order id=1 as tenant 1, calls `ferro_audit::history_for_target(&AuditTarget::new("submit", "1"), &db)`, asserts non-empty with `entry.action == "mcp.action.submit"`, `entry.tenant_id == Some("1")`, `entry.after.is_some()`.
- `idempotent_write_e2e` (SC#3): `AtomicUsize` mutation counter, two calls with same `idempotency_key`, asserts counter == 1 and both `structuredContent` values equal.

All three tests GREEN. Full CI gate: fmt clean, clippy clean, `cargo test --all-features` exit 0 (confirmed twice — runs `b890720ei` and `babii2z3l`).

## Commits

| Hash | Message | Key files |
|------|---------|-----------|
| `7ee674dc` | feat(219-02): TenantScoped on Order + make_write_dispatcher + call site repair | orders.rs, mcp.rs, Cargo.toml, migrations/mod.rs, mcp_tenant_isolation.rs |
| `843ad782` | feat(219-02): SC#2/#4/#3 e2e fixtures GREEN + migration collision fix | mcp_write_dispatch.rs, tests/mod.rs, migration wrappers, write_dispatch.rs |
| `6d336d0c` | style(219-02): apply cargo fmt formatting | mcp.rs, migration wrapper, mod.rs, mcp_write_dispatch.rs, write_dispatch.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] DeriveMigrationName version collision: local migration wrapper files**
- **Found during:** Task 1 (registration of idempotency and audit migrations)
- **Issue:** Both `ferro_mcp_oauth::MigrationMcpIdempotencyKeys` (in `ferro-mcp-oauth/src/migration.rs`) and `ferro_audit::CreateAuditLogTable` (in `ferro-audit/src/migration.rs`) have `#[derive(DeriveMigrationName)]` with the same source file stem `"migration"`. SeaORM derives the migration version string from the file stem, so both register as version `"migration"` — triggering a UNIQUE constraint violation on `seaql_migrations.version` at runtime.
- **Fix:** Created two local wrapper files: `app/src/migrations/m20260614_create_mcp_idempotency_keys_table.rs` and `app/src/migrations/m20260614_create_audit_log_table.rs`. Each file has its own `#[derive(DeriveMigrationName)]` struct (`Migration`) and delegates `up/down` to the external crate. The date-prefixed file stem becomes the version string, giving each migration a unique key.
- **Files modified:** `app/src/migrations/m20260614_create_mcp_idempotency_keys_table.rs` (new), `app/src/migrations/m20260614_create_audit_log_table.rs` (new), `app/src/migrations/mod.rs` (updated)
- **Commits:** `843ad782`

**2. [Rule 3 - Blocking] Async closure lifetime trap: action_name and db cannot be borrowed in async move**
- **Found during:** Task 2 (make_write_dispatcher implementation)
- **Issue:** `Box::pin(async move { ... })` closures cannot capture `&str` or `&DatabaseConnection` references — they escape the borrow. The compiler rejects the closure type if `action_name: &str` or `db: &DatabaseConnection` are referenced inside the async block.
- **Fix:** Convert to owned values before the async block: `let action_name = action_name.to_string(); let db = db.clone(); Box::pin(async move { ... })`.
- **Files modified:** `app/src/controllers/mcp.rs`
- **Commits:** `7ee674dc`

**3. [Rule 3 - Blocking] mcp_tenant_isolation.rs broken by Plan 01's new dispatcher param**
- **Found during:** Task 2 (compiling after call site repair in mcp.rs)
- **Issue:** Plan 01 added `dispatcher: &WriteDispatcher` as a trailing param to `handle_tools_call`. The existing `mcp_tenant_isolation.rs` tests call `handle_tools_call` with 5 args — compile failure.
- **Fix:** Added `noop_dispatcher()` helper function returning a `WriteDispatcher` with always-pass no-op closures; passed `&noop_dispatcher()` at both call sites in `mcp_tenant_isolation.rs`.
- **Files modified:** `app/src/tests/mcp_tenant_isolation.rs`
- **Commits:** `7ee674dc`

**4. [Rule 1 - Bug] SC#4 rmcp::model::CallToolResult not available in app crate**
- **Found during:** Task 3 (SC#4 fixture compile)
- **Issue:** `app` does not directly depend on `rmcp`; `rmcp::model::CallToolResult` is not importable for typed deserialization of test results.
- **Fix:** Removed the typed deserialization import; used raw JSON assertions on `result["result"]["isError"]` with `assert_ne!(result["result"]["isError"], true, ...)` for success paths and `assert_eq!(result["result"]["isError"], true, ...)` for denial paths.
- **Files modified:** `app/src/tests/mcp_write_dispatch.rs`
- **Commits:** `843ad782`

**5. [Rule 1 - Bug] DB::connection() returns DbConnection wrapping &DatabaseConnection — needs .inner()**
- **Found during:** Task 1 (TenantScoped impl)
- **Issue:** `ferro::DB::connection()` returns `DbConnection` (a wrapper type), not `&DatabaseConnection` directly. SeaORM `.one(...)` expects `&impl ConnectionTrait`. Calling `.one(connection)` without unwrapping fails to compile.
- **Fix:** Called `.inner()` on the `DbConnection` result before passing to `.one()`.
- **Files modified:** `app/src/models/orders.rs`
- **Commits:** `7ee674dc`

## Success Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|---------|
| SC#1: guard re-eval + dispatch_write pipeline | GREEN (Plan 01) | `843ad782` + Plan 01 SUMMARY |
| SC#2: cross-tenant write denied, record unmutated | GREEN | `cross_tenant_write_denied` test; `isError:true`, `status == "submitted"` assertion |
| SC#3 (e2e): one mutation after two identical idempotency_key calls | GREEN | `idempotent_write_e2e` test; `AtomicUsize` counter == 1 |
| SC#4: ferro-audit entry recoverable via history_for_target | GREEN | `write_call_produces_audit_entry` test; `action == "mcp.action.submit"`, `after.is_some()` |
| SC#5: structured result envelope (no raw text injection) | GREEN (Plan 01) | write_tool_error_result + CallToolResult::structured |
| Full CI gate | GREEN | fmt clean; clippy 0 warnings (`-D warnings`); `cargo test --all-features` exit 0 (×2) |

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. All changes are in test helpers and the sample app's migration/controller/model files. The threat mitigations in T-219-01 (BOLA via find-then-mutate), T-219-02 (live guard re-eval), and T-219-03 (e2e idempotency) are all implemented and covered by fixtures.

## Self-Check: PASSED

Files exist:
- `app/src/models/orders.rs` — contains `impl TenantScoped`
- `app/src/controllers/mcp.rs` — contains `make_write_dispatcher` + `find_for_tenant` + `is_manager`
- `app/src/migrations/m20260614_create_audit_log_table.rs` — local wrapper
- `app/src/migrations/m20260614_create_mcp_idempotency_keys_table.rs` — local wrapper
- `app/src/tests/mcp_write_dispatch.rs` — contains `cross_tenant_write_denied`, `write_call_produces_audit_entry`, `idempotent_write_e2e`

Commits verified: `7ee674dc`, `843ad782`, `6d336d0c` all present in `git log --oneline -6`.
