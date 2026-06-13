---
phase: 220-confirmation-gating-for-destructive-actions
plan: "01"
subsystem: ferro-mcp-server
tags: [confirmation-gate, write-dispatch, two-step-flow, mcp, security]
dependency_graph:
  requires: ["220-00"]
  provides: ["220-02"]
  affects: ["ferro-mcp-server", "ferro-ai"]
tech_stack:
  added: []
  patterns:
    - "cfg-gated is_confirmed: bool param on dispatch_write to bypass seam"
    - "InMemoryConfirmationStore (ferro-ai) with TTL expiry"
    - "BASE62 CSPRNG token (cfm_ prefix, 43-char, ~256-bit)"
    - "(tenant_id, action_name, record_id) binding stored at request, verified at confirm"
    - "Guard re-evaluation at confirm time on stored inputs with live DB"
    - "Post-disambiguation two-tool synthesis in render_exposed_tools"
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/config.rs
    - ferro-mcp-server/src/write_dispatch.rs
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/jsonrpc.rs
decisions:
  - "is_confirmed bool param on dispatch_write (not a separate code path): seam bypass is a single param, single source of execution truth"
  - "Confirmation tools synthesized post-disambiguation so strip_prefix routing returns valid action names (Pitfall 3)"
  - "tokio::time::pause() called manually after setup_db() — start_paused=true freezes SQLite pool acquire"
metrics:
  duration: "cross-session (~24h wall, <1h compute)"
  completed_date: "2026-06-14"
  tasks_completed: 3
  files_modified: 4
---

# Phase 220 Plan 01: Confirmation Gate Implementation Summary

Server-side two-step confirmation gate for destructive MCP actions: CFG-gated D-08 seam returns `ConfirmationRequired`, `request_confirm_<action>` issues a CSPRNG token bound to (tenant, action, record), `confirm_<action>` verifies binding + re-evaluates guards + executes exactly once, with `request_confirm_` / `confirm_` tool pair synthesized post-disambiguation in the renderer.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | TTL config + D-08 seam + token gen + is_confirmed | 65d476d5 | config.rs, write_dispatch.rs |
| 2 | handle_request_confirm + handle_confirm + routing | 65d476d5 | write_dispatch.rs, jsonrpc.rs |
| 3 | Two-tool synthesis in renderer + test count fixes | d291cb18 | renderer.rs, jsonrpc.rs |

## Verification

All 54 tests pass under `--features confirmation`:
- 40 lib tests (including SC#1–SC#4 + guard-at-confirm)
- 5 dispatch_integration tests
- 5 jsonrpc_integration tests
- 4 mcp_tenant_isolation tests

Feature-off build compiles cleanly. Clippy clean both with and without confirmation feature.

## Success Criteria Check

- SC#1: bare destructive write without token → `confirmation_required`, executor not called. GREEN.
- SC#2: `request_confirm` → `confirm` executes exactly once (single-use token). GREEN.
- SC#3: expired token rejected after TTL (paused-clock test). GREEN.
- SC#4: action/record mismatch rejected; cross-tenant binding rejected. GREEN.
- Guard re-eval at confirm: guard denied on stale live state, action not executed. GREEN.
- Two-tool synthesis: `request_confirm_<action>` + `confirm_<action>` in tools/list, post-disambiguation. GREEN.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SC#3 pool timeout with start_paused=true**
- **Found during:** Task 2 (SC#3 TTL test)
- **Issue:** `#[tokio::test(start_paused = true)]` freezes tokio timers; SQLite pool acquire internally uses `tokio::time::timeout`, so it immediately times out before DB connection is established.
- **Fix:** Changed test to `#[tokio::test]` and called `tokio::time::pause()` manually after `setup_db()` completes. All time-advance assertions still work correctly.
- **Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
- **Commit:** 65d476d5

**2. [Rule 1 - Bug] Tool count assertions fail with confirmation feature**
- **Found during:** Task 3 (post-synthesis test run)
- **Issue:** `test_one_write_tool_per_action` and `write_tools_definitions_parse_as_valid_mcp_tool` both asserted exactly 3 tools; with confirmation feature, `submit_order` (destructive) gains 2 confirmation tools → 5 total.
- **Fix:** Wrapped count assertions in `#[cfg(not(feature = "confirmation"))]` / `#[cfg(feature = "confirmation")]` blocks with correct counts (3 vs 5). Added assertions for presence of `request_confirm_submit_order` and `confirm_submit_order` under `#[cfg(feature = "confirmation")]`.
- **Files modified:** `ferro-mcp-server/src/renderer.rs`, `ferro-mcp-server/src/jsonrpc.rs`
- **Commit:** d291cb18

**3. [Rule 3 - Blocking] rand not a direct dependency**
- **Found during:** Task 1 (generate_confirmation_token)
- **Issue:** `use rand::Rng` failed — rand was only a transitive dep. `#[cfg(feature = "confirmation")]` can't reach transitive crates directly.
- **Fix:** Added `rand = { version = "0.8", optional = true }` to ferro-mcp-server/Cargo.toml and `"dep:rand"` to the `confirmation` feature array.
- **Files modified:** `ferro-mcp-server/Cargo.toml`
- **Commit:** 65d476d5

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced. The confirmation gate is entirely in-process (InMemoryConfirmationStore) behind the existing `tools/call` trust boundary. All threat mitigations from STRIDE register T-220-01 through T-220-CR01 are implemented and tested.

## Known Stubs

None — all confirmation paths produce real structured responses.

## Self-Check: PASSED

- `ferro-mcp-server/src/config.rs`: FOUND — contains `confirmation_ttl_seconds`
- `ferro-mcp-server/src/write_dispatch.rs`: FOUND — contains `handle_request_confirm`, `handle_confirm`, `generate_confirmation_token`, `ConfirmationRequired`
- `ferro-mcp-server/src/renderer.rs`: FOUND — contains `request_confirm_`, synthesis block
- `ferro-mcp-server/src/jsonrpc.rs`: FOUND — contains `strip_prefix("request_confirm_")`
- Commit d291cb18: FOUND
- Commit 65d476d5: FOUND
- All 54 tests: PASSED
