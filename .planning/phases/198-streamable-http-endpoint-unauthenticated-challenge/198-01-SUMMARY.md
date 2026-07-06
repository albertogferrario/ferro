---
phase: 198-streamable-http-endpoint-unauthenticated-challenge
plan: "01"
subsystem: ferro-mcp-server
tags: [mcp, jsonrpc, auth-seam, dispatch, projections]
dependency_graph:
  requires: [197-03]
  provides: [jsonrpc-pure-dispatch, bearer-seam, mcp-server-config]
  affects: [ferro-mcp-server]
tech_stack:
  added: []
  patterns: [bearer-seam, env-var-config-from-env, pure-dispatch-functions]
key_files:
  created:
    - ferro-mcp-server/src/config.rs
    - ferro-mcp-server/src/auth.rs
    - ferro-mcp-server/src/jsonrpc.rs
    - ferro-mcp-server/tests/common/mod.rs
    - ferro-mcp-server/tests/jsonrpc_integration.rs
  modified:
    - ferro-mcp-server/src/lib.rs
    - ferro-mcp-server/tests/dispatch_integration.rs
decisions:
  - "protocolVersion literal is \"2025-03-26\" (rmcp 0.12 LATEST) — no rmcp struct pulled in"
  - "handle_tools_call returns { result: { content, total, limit, offset } } — caller in Plan 02 splices jsonrpc/id"
  - "filter-key allowlist and MAX_LIMIT clamp stay exclusively in dispatch.rs; jsonrpc.rs has no re-implementation"
  - "extract_bearer always Unauthenticated in Phase 198 — no Authenticated code path exists"
  - "BearerOutcome::Authenticated variant kept with #[allow(dead_code)] for Phase 199 to fill without enum reshape"
metrics:
  duration: "517s"
  completed_date: "2026-06-10"
  tasks: 3
  files_changed: 7
---

# Phase 198 Plan 01: Pure JSON-RPC Dispatch + Config + Bearer Seam Summary

**One-liner:** Three pure async JSON-RPC handlers (`handle_initialize`, `handle_tools_list`, `handle_tools_call`) with `McpServerConfig::from_env()` config and `BearerOutcome` seam always returning `Unauthenticated`, exercised by 4 integration tests against an in-memory SQLite fixture — no HTTP server, no OAuth.

## What Was Built

### config.rs — McpServerConfig
Reads `APP_NAME` / `APP_URL` from env with `"Ferro"` / `"http://localhost"` fallbacks, mirroring `InertiaConfig::default()` exactly. `version` from `env!("CARGO_PKG_VERSION")`. `from_env()` is an alias of `default()`. No forbidden identity literals.

### auth.rs — BearerOutcome seam
`BearerOutcome` enum with `Unauthenticated` and `Authenticated(serde_json::Value)` (dead_code, reserved for Phase 199). `extract_bearer` accepts `Option<&str>` and always returns `Unauthenticated` — zero code paths construct `Authenticated`. Two inline unit tests assert this invariant for `None` and for a bearer-token header.

### jsonrpc.rs — Pure dispatch functions
Three `pub async fn` returning `serde_json::Value`:

- `handle_initialize(_params, config)` → `{ "result": { protocolVersion, capabilities.tools, serverInfo } }`
- `handle_tools_list(services, config)` → `{ "result": { tools } }` or `{ "error": { code: -32603 } }`
- `handle_tools_call(call_params, services, db)` → strips `list_` prefix, resolves `mcp_exposed` ServiceDef, delegates to `dispatch`. Unknown tool → `-32601`. Pagination keys stripped before handing remainder to `dispatch` as filters. Filter allowlist + MAX_LIMIT clamp stay in `dispatch.rs`.

### tests/common/mod.rs — Shared fixture
`pub async fn setup_db()` creates in-memory SQLite with `items` table (3 rows). `pub fn item_service()` returns a `ServiceDef::new("item").mcp_exposed(true)` with `id`, `status`, `customer_id` fields. Added `.mcp_exposed(true)` absent from the Phase 197 version.

### tests/jsonrpc_integration.rs — Integration coverage
Four async tokio tests, no live server, no OAuth:
1. `initialize_returns_correct_protocol_version` — asserts `"2025-03-26"`, `capabilities.tools` is object, `serverInfo.name == "TestApp"`
2. `tools_list_returns_only_exposed` — 2 services (one exposed, one not) → tools array len 1, name `"list_order"`
3. `tools_call_returns_rows` — fixture db + `item_service()` → content array len 3
4. `tools_call_unknown_tool_is_method_not_found` — unknown tool name → `error.code == -32601`

## Deviations from Plan

None — plan executed exactly as written.

## Test Results

```
cargo test -p ferro-mcp-server

running 12 unit tests   ... ok (auth + renderer + schema)
running 5 dispatch integration tests ... ok
running 4 jsonrpc integration tests  ... ok
21 total tests, 0 failures
```

## Validation Bar

- `cargo fmt --all -- --check` — clean (auto-formatted 4 files after initial write)
- `cargo clippy --all --all-targets -- -D warnings` — clean, 0 warnings
- `cargo test -p ferro-mcp-server` — 21 tests pass

## Self-Check

### Created files exist:

- FOUND: ferro-mcp-server/src/config.rs
- FOUND: ferro-mcp-server/src/auth.rs
- FOUND: ferro-mcp-server/src/jsonrpc.rs
- FOUND: ferro-mcp-server/tests/common/mod.rs
- FOUND: ferro-mcp-server/tests/jsonrpc_integration.rs

### Commits verified:

- 030bc53e — chore(198-01): extract shared test fixture
- 85573a8f — feat(198-01): add McpServerConfig and BearerOutcome seam
- faefa96b — feat(198-01): add jsonrpc.rs dispatch + integration tests

## Self-Check: PASSED
