---
phase: 217-tenant-context-per-tenant-api-key-auth
plan: "00"
subsystem: ferro-mcp-oauth, ferro-mcp-server
tags: [auth, mcp, api-key, tenant-context, tdd-red, skeleton]
dependency_graph:
  requires: []
  provides:
    - ferro-mcp-oauth::validate_api_key (skeleton)
    - ferro-mcp-oauth::generate_mcp_api_key (skeleton)
    - ferro-mcp-oauth::hash_mcp_api_key (real)
    - ferro-mcp-server::resolve_tenant (async unifier)
    - ferro-mcp-server::McpContext (extended with tenant_id, evaluated_guards, scope)
    - ferro-mcp-server::Error::Auth variant
    - ferro-mcp-server::handle_tools_list ctx param
    - ferro-mcp-server scope gate in handle_tools_call
  affects:
    - ferro-mcp-server/tests/jsonrpc_integration.rs (call sites updated)
    - ferro-mcp-server/tests/mcp_tenant_isolation.rs (new RED integration tests)
tech_stack:
  added: []
  patterns:
    - async unifier branching on token shape (ferro_ prefix → DB path, else → JWT path)
    - SHA-256 hash_mcp_api_key mirroring framework/src/api/api_key.rs pattern
    - RED TDD: skeleton stubs return Invalid/STUB so tests fail on assertion not compile
    - scope gate before service lookup (fires even for unknown write tool names)
key_files:
  created:
    - ferro-mcp-server/tests/mcp_tenant_isolation.rs
  modified:
    - ferro-mcp-server/Cargo.toml
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/error.rs
    - ferro-mcp-server/src/auth.rs
    - ferro-mcp-server/src/lib.rs
    - ferro-mcp-server/src/jsonrpc.rs
    - ferro-mcp-server/tests/jsonrpc_integration.rs
    - ferro-mcp-server/tests/common/mod.rs
    - ferro-mcp-oauth/src/validate.rs
    - ferro-mcp-oauth/src/lib.rs
decisions:
  - scope gate moved before service lookup so write-tool rejection fires before -32601 method-not-found (required for SC#3 test to pass)
  - validate_api_key skeleton does DB query structurally but always returns Invalid — fails closed; no auth bypass possible
  - generate_mcp_api_key stub returns ("STUB","STUB") — ensures prefix/length/hash tests are RED
metrics:
  duration_minutes: ~35
  completed_date: "2026-06-13"
  tasks_completed: 3
  tasks_total: 3
  files_created: 1
  files_modified: 10
---

# Phase 217 Plan 00: Compiling Skeleton + RED Test Suite Summary

Wave 0 gate established: workspace compiles with all structural changes, RED tests exist and
fail at assertion (not compile error), scope gate is real and wired.

## What Was Built

**Structural changes (all compile, workspace builds clean):**
- `ferro-mcp-server/Cargo.toml`: added `ferro-mcp-oauth` path dependency (Option A from RESEARCH)
- `ferro-mcp-server/src/renderer.rs`: `McpContext` unit struct → `{ tenant_id: Option<i64>, evaluated_guards: HashMap<String, bool>, scope: Option<String> }`; all unit-struct call sites converted to `McpContext::default()`
- `ferro-mcp-server/src/error.rs`: `Auth(String)` variant added before `Database` (maps to -32603)
- `ferro-mcp-server/src/auth.rs`: `BearerOutcome` stub entirely replaced by `resolve_tenant` async unifier
- `ferro-mcp-server/src/lib.rs`: `BearerOutcome` export replaced by `resolve_tenant` + `BearerCheck` re-exports
- `ferro-mcp-server/src/jsonrpc.rs`: `handle_tools_list` gains `ctx: &McpContext` 2nd parameter; `handle_tools_call` gains `ctx: &McpContext` 5th parameter + scope gate (D-06/SC#3); scope gate placed BEFORE service lookup so write-tool rejection fires for unknown tool names too
- `ferro-mcp-server/tests/jsonrpc_integration.rs`: all `handle_tools_list` and `handle_tools_call` call sites updated with `McpContext::default()` arguments
- `ferro-mcp-server/tests/common/mod.rs`: `#[allow(dead_code)]` on shared helpers to suppress clippy errors from `mcp_tenant_isolation.rs`

**Skeleton functions in `ferro-mcp-oauth/src/validate.rs`:**
- `hash_mcp_api_key(raw_key: &str) -> String`: real SHA-256 hex (not a stub)
- `generate_mcp_api_key() -> (String, String)`: STUB returning ("STUB","STUB") — RED
- `validate_api_key(header, db, expected_tenant) -> BearerCheck`: STUB; does header/prefix check and DB query structurally but always returns `Invalid` — RED

**RED test suite:**
- `ferro-mcp-oauth/src/validate.rs` tests: `generate_mcp_api_key_is_prefixed_and_hash_matches` (RED), `valid_api_key_returns_authenticated` (RED), `unknown_api_key_returns_invalid` (trivially passes), `revoked_api_key_returns_invalid` (trivially passes), `wrong_expected_tenant_returns_forbidden` (RED)
- `ferro-mcp-server/tests/mcp_tenant_isolation.rs` (279 lines): `api_key_and_jwt_produce_same_tenant_id` (RED — SC#2), `read_scope_key_rejected_on_write_tool_name` (PASSES — scope gate is real), `read_scope_key_allowed_on_read_tool` (PASSES), `api_key_cross_tenant_isolation` (RED — SC#5)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Scope gate moved before service lookup**
- **Found during:** Task 3 — `read_scope_key_rejected_on_write_tool_name` returned -32601 instead of -32603
- **Issue:** Plan placed scope gate after service lookup; for synthetic write-tool names not in the service list, the -32601 "Method not found" fired before the scope gate
- **Fix:** Moved scope gate to execute before the service lookup, so any non-`list_`-prefixed call from a `read`-scoped key is rejected with -32603 regardless of whether the tool exists
- **Files modified:** `ferro-mcp-server/src/jsonrpc.rs`
- **Commit:** 97a50be0 (amended scope gate position in the same Task 3 commit)

**2. [Rule 2 - Clippy] `#[allow(dead_code)]` on common/mod.rs helpers**
- **Found during:** Task 3 clippy run
- **Issue:** `mcp_tenant_isolation.rs` includes `mod common;` but does not call `setup_db` or `item_service`; clippy -D warnings treats them as dead code in that test binary
- **Fix:** Added `#[allow(dead_code)]` to both helpers in `common/mod.rs`
- **Files modified:** `ferro-mcp-server/tests/common/mod.rs`
- **Commit:** 97a50be0

## Known Stubs

| Stub | File | Reason |
|------|------|--------|
| `generate_mcp_api_key()` returns `("STUB","STUB")` | `ferro-mcp-oauth/src/validate.rs:115-118` | Intentional RED placeholder; real CSPRNG generation in Plan 01 |
| `validate_api_key()` always returns `BearerCheck::Invalid` | `ferro-mcp-oauth/src/validate.rs:126-161` | Intentional RED placeholder; real SHA-256 DB lookup in Plan 01 |

These stubs prevent `valid_api_key_returns_authenticated`, `wrong_expected_tenant_returns_forbidden`, `api_key_and_jwt_produce_same_tenant_id`, and `api_key_cross_tenant_isolation` from passing. Implementation in Plan 01.

## Threat Flags

No new network endpoints or trust boundaries introduced. The `validate_api_key` skeleton fails closed (`BearerCheck::Invalid`) — consistent with T-217-03 disposition in the plan threat model. No new threat surface beyond what the plan's threat register already covers.

## Self-Check

All files created/modified verified by acceptance criteria grep (all 14 checks passed).

Commits:
- `786c0878`: feat(217-00): add ferro-mcp-oauth dep, extend McpContext, add Auth error variant
- `a079c61b`: feat(217-00): skeleton signatures — validate_api_key, generate_mcp_api_key, resolve_tenant, scope gate, fix call sites
- `97a50be0`: test(217-00): RED test suite — validate_api_key unit tests + mcp_tenant_isolation integration tests
- `b50e510e`: style(217-00): rustfmt all changed files

## Self-Check: PASSED

- `ferro-mcp-server/Cargo.toml` — contains ferro-mcp-oauth dep: FOUND
- `ferro-mcp-server/src/renderer.rs` — contains tenant_id/evaluated_guards/scope: FOUND
- `ferro-mcp-server/src/error.rs` — contains Auth(String): FOUND
- `ferro-mcp-server/src/auth.rs` — contains resolve_tenant, no BearerOutcome: FOUND
- `ferro-mcp-server/tests/mcp_tenant_isolation.rs` — exists, 279 lines: FOUND
- `cargo build -p ferro-mcp-oauth -p ferro-mcp-server --tests` — exits 0: VERIFIED
- RED tests fail on assertion: VERIFIED (`generate_mcp_api_key_is_prefixed_and_hash_matches` panics at "raw_key must start with ferro_")
