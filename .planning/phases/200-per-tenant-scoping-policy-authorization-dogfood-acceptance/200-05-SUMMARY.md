---
phase: 200-per-tenant-scoping-policy-authorization-dogfood-acceptance
plan: "05"
subsystem: app-mcp-controller
tags: [gate-authorization, policy, mcp, tenant-scoping, security, isError, fail-closed]
dependency_graph:
  requires: ["200-01", "200-02", "200-04"]
  provides: ["gate-check-on-tools-call", "fail-closed-mcp-ability", "d09-tool-error-envelope", "tenant-id-forwarding"]
  affects: ["200-06", "200-07"]
tech_stack:
  added: []
  patterns:
    - "Gate::authorize_for(&user, ability, None) — explicit user, not session-based Auth::id()"
    - "req.get::<serde_json::Value>() to read principal inserted by BearerAuthMiddleware"
    - "make_tool_deny_response helper for D-09 no-disclosure tool-error envelope"
    - "Fail-closed: mcp_ability=None → deny before dispatch (T-200-03b)"
    - "current_tenant().map(|t| t.id) forwarded to handle_tools_call"
key_files:
  created: []
  modified:
    - app/src/controllers/mcp.rs
decisions:
  - "challenge_response gated with #[cfg(test)] — bearer challenge is now BearerAuthMiddleware's responsibility; handler only needs it in tests for the existing token-shape test"
  - "make_tool_deny_response extracted as a named helper — enables unit-testing the D-09 envelope shape independently of the async handler"
  - "validate_bearer removed from production handler path — now exclusively in BearerAuthMiddleware (Plan 200-04); test import retained for existing token-validation unit test"
metrics:
  duration: "6m20s"
  completed_date: "2026-06-10"
  tasks_completed: 1
  files_changed: 1
---

# Phase 200 Plan 05: Gate Policy Check + Fail-Closed + D-09 Tool Error Summary

One-liner: MCP tools/call path is now policy-gated via Gate::authorize_for with explicit user load, fail-closed on absent mcp_ability, and D-09-compliant no-disclosure tool error on deny.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Gate check + fail-closed + D-09 tool error + tenant_id forwarding | 42c5ef3c | app/src/controllers/mcp.rs |

## What Was Built

**Inline bearer validation removed.** The Phase 199 `match validate_bearer(...)` block (lines 46–66) is replaced by `req.get::<ferro::serde_json::Value>()` — reading the principal inserted by `BearerAuthMiddleware` upstream. The handler no longer owns JWT validation; it trusts the middleware stack.

**Principal → user_id.** `principal["sub"].as_str().and_then(|s| s.parse().ok())` — non-numeric or absent sub returns 400 (T-200-SUB). This defensive parse prevents any default-user fallback.

**Service resolution before Gate check.** The `tools/call` arm resolves the target `ServiceDef` by stripping the `list_` prefix from `params["name"]` — using the same logic as `handle_tools_call` but earlier, so the Gate check can read `service.mcp_ability` before dispatch.

**Concrete User load.** `crate::models::users::User::find_by_id(user_id)` — DB error → 500 JSON-RPC error; missing user → 401. This gives `Gate::authorize_for` a concrete `&dyn Authenticatable`.

**Fail-closed (D-04/D-06/T-200-03b).** `service.mcp_ability.as_deref()` — `None` → `make_tool_deny_response("Access denied. This resource requires an explicit ability declaration.", &id)`. An `mcp_exposed` projection without a declared ability is never callable.

**Gate::authorize_for — not Gate::authorize.** Uses the explicit-user variant to avoid `Auth::id()` session lookup, which is absent on the MCP bearer path (Pitfall 7, verified from gate.rs lines 151–161 vs 172–189).

**D-09 deny envelope.** `make_tool_deny_response` produces `{"result": {"content": [{"type": "text", "text": "..."}], "isError": true}, "jsonrpc": "2.0", "id": <id>}` — a JSON-RPC **success** envelope, not a transport 401/403. The message contains no table names, column names, filter values, or row counts.

**Tenant forwarding.** On allow: `ferro::current_tenant().map(|t| t.id)` passed to `handle_tools_call(params, &services, db.inner(), tenant_id)`.

**Tests added (3 new, all passing):**
- `policy_deny_no_ability` — ServiceDef with `mcp_ability=None` → `isError:true` envelope, no data disclosure
- `policy_deny_tool_error_shape` — deny envelope shape: `result.isError`, no rows/columns/total, no "orders"/"customer_name"/"tenant_id" in text
- `deny_response_is_jsonrpc_success_not_transport_error` — `result` key present, no top-level `error` key

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Dead Code] challenge_response gated to #[cfg(test)]**
- **Found during:** clippy run after implementation
- **Issue:** `challenge_response` was marked dead-code by clippy (`-D warnings`) because bearer challenge generation moved to `BearerAuthMiddleware` in Plan 200-04. The existing test `challenge_response_has_correct_header` still references it.
- **Fix:** Added `#[cfg(test)]` attribute to `challenge_response` — keeps the existing test intact, eliminates the dead-code warning.
- **Files modified:** `app/src/controllers/mcp.rs`
- **Commit:** 42c5ef3c

**2. [Rule 1 - Style] Gate::test_lock()/flush() not accessible cross-crate**
- **Found during:** First compile attempt
- **Issue:** The plan's behavior description mentioned using `Gate::flush()` / `Gate::test_lock()` in tests, but those are `#[cfg(test)]`-only in the framework crate and not compiled into external test builds.
- **Fix:** Rewrote `policy_deny_tool_error_shape` to test `make_tool_deny_response` directly (the function called from the Gate deny branch), without setting up a global Gate state. The test is a pure shape/no-disclosure test — it doesn't need to invoke Gate at all. The Gate behavior itself is covered by `framework` crate's own tests.
- **Commit:** 42c5ef3c

## Known Stubs

None — all code paths are wired. The `mcp.rs` stub comment from Plan 200-04 ("Full gate check wired in Plan 200-05") is removed and replaced by the complete implementation.

## Self-Check: PASSED

- `app/src/controllers/mcp.rs` exists and contains `Gate::authorize_for`, `mcp_ability`, `"isError"`, `current_tenant().map(|t| t.id)`
- Commit 42c5ef3c present in git log
- Commit 7bd6f4b2 (fmt) present in git log
- `cargo test -p app` green (7/7 tests pass)
- `cargo clippy -p app --all-targets -- -D warnings` clean
- `cargo build -p app` clean
