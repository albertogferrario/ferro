---
phase: 221-inbound-nl-intent-loop
reviewed: 2026-06-14T02:21:21Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - ferro-mcp-server/src/intent.rs
  - ferro-mcp-server/src/lib.rs
  - ferro-mcp-server/Cargo.toml
  - ferro-mcp-server/tests/intent_loop.rs
  - ferro-ai/src/classifier/mod.rs
  - ferro-ai/src/lib.rs
  - ferro-ai/Cargo.toml
  - app/src/controllers/mcp_chat.rs
  - app/src/controllers/mcp.rs
  - app/src/controllers/mod.rs
  - app/src/routes.rs
  - app/Cargo.toml
  - app/src/tests/mcp_tenant_isolation.rs
  - app/src/tests/mcp_write_dispatch.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 221: Code Review Report

**Reviewed:** 2026-06-14T02:21:21Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

The phase implements an NL intent loop with sound structural design. The primary security objectives are met: `process_nl_turn` routes through `handle_write_call`/`handle_tools_call` without re-implementing dispatch, guard re-evaluation, or tenant isolation; `tenant_id` is derived from `ferro::current_tenant()` (set by `TenantMiddleware` via `JwtClaimResolver`) never from the request body; classifier output is treated as untrusted input entering the same pipeline as direct MCP calls. Feature gating is correctly wired — the `ai` feature pulls in `ferro-ai` with `classifier-trait` only (no `reqwest`), and `ai-live` adds `AnthropicProvider`+`reqwest` transitively. Fixture files contain no secrets.

Three warnings require attention. The most significant is an authorization gap: the `/mcp/chat` endpoint does not call `Gate::authorize_for` before routing read tools via NL, whereas the direct `/mcp` path does. This means a user without the declared `mcp_ability` (e.g. `"view-orders"`) can read that service's data by asking in natural language. The second warning is that the `/mcp/chat` route is registered unconditionally in `routes.rs` even when the `ai-live` feature is disabled — the fallback handler returns HTTP 200 with `isError:true` JSON, which is reachable by any authenticated user. The third warning is a misleading test: `replay_deterministic` exercises the approve write path twice without an idempotency key, executes the (mock) executor twice, yet asserts only that the mock outputs are equal — it does not verify single-execution and silently passes due to the constant-returning mock.

---

## Warnings

### WR-01: `/mcp/chat` bypasses `Gate::authorize_for` for NL-routed read tools

**File:** `app/src/controllers/mcp_chat.rs:72-106`

**Issue:** The direct `/mcp` handler calls `Gate::authorize_for(&user, ability, None)` before dispatching any `list_*` read tool (see `mcp.rs:292`). A projection with no `mcp_ability` declared is denied fail-closed (`mcp.rs:280-288`). The `/mcp/chat` handler delegates directly to `process_nl_turn`, which routes `list_*` tool names to `handle_tools_call`. `handle_tools_call` is a library function — it does not call `Gate::authorize_for`. The result is that a user who lacks the `"view-orders"` ability (or whose service has `mcp_ability = None`) is denied on direct `tools/call` but can succeed on the NL path by phrasing the same request in natural language.

This is an authorization bypass on the `/mcp/chat` surface for all read tools. Write tools are unaffected (scope gate + live guard re-eval covers those on both paths).

**Fix:** Mirror the `mcp.rs` Gate check inside `handle_chat` before passing control to `process_nl_turn`, or extract the Gate check into a shared function callable from both handlers. Because `process_nl_turn` classifies first and then routes, the Gate check needs to happen after classification (the tool name is known only after `classifier.classify()` returns). One workable pattern is to pass a gate-callback into `process_nl_turn` that fires after classification and before dispatch:

```rust
// In mcp_chat.rs, after process_nl_turn returns but before the result:
// Alternatively, add a pre-dispatch hook param to process_nl_turn.

// Simplest safe path until a hook is added: load the user once before calling
// process_nl_turn, then inside process_nl_turn (or a wrapper) apply:
if tool_name.starts_with("list_") {
    let service = services.iter().find(|s| {
        format!("list_{}", s.name) == tool_name && s.mcp_exposed
    });
    if let Some(svc) = service {
        let ability = svc.mcp_ability.as_deref()
            .ok_or_else(|| /* deny */)?;
        Gate::authorize_for(&user, ability, None)
            .map_err(|_| /* deny */)?;
    }
}
```

---

### WR-02: `/mcp/chat` route registered unconditionally; non-`ai-live` builds expose a live endpoint that returns HTTP 200 with `isError:true`

**File:** `app/src/routes.rs:60`, `app/src/controllers/mcp_chat.rs:112-118`

**Issue:** The `/mcp/chat` POST route is registered in `routes!` without a `#[cfg(feature = "ai-live")]` guard. When the app is built without `ai-live`, `handle_chat` compiles to the fallback branch (`cfg(not(feature = "ai-live"))`) which returns:

```json
{
  "result": {
    "content": [{"type": "text", "text": "NL intent loop requires the ai-live feature"}],
    "isError": true
  }
}
```

with HTTP status **200 OK**. The route is fully reachable by any authenticated user. This leaks information about the feature set in use. Comments in `mcp_chat.rs:110-111` say "the route is not exposed" but it is — the route is registered and the middleware runs.

**Fix:** Either gate the route registration behind the feature, or change the fallback to HTTP 404/501:

```rust
// Option A: gate route registration
#[cfg(feature = "ai-live")]
post!("/mcp/chat", controllers::mcp_chat::handle_chat).name("mcp.chat"),

// Option B: return 501 Not Implemented in the non-ai-live branch
#[cfg(not(feature = "ai-live"))]
return Err(HttpResponse::new().status(501));
```

Option A is cleaner and avoids the authenticated dead-route surface entirely.

---

### WR-03: `replay_deterministic` test misleadingly names a non-idempotent write path

**File:** `ferro-mcp-server/tests/intent_loop.rs:766-838`

**Issue:** The `replay_deterministic` test iterates over `[&f_list, &f_approve]`. For the `approve` fixture (a non-destructive write), `process_nl_turn` is called twice against the same DB without an idempotency key in the arguments. The executor (a constant-returning mock `|_, _, _, _| Ok(json!({"status": "approved"}))`) fires twice — the `mcp_idempotency_keys` table in the test DB is created but the key is never set in the fixture's arguments, so the idempotency layer is bypassed. The assertion `assert_eq!(sc1, sc2)` trivially passes because both calls return the same constant mock value, not because execution was deduplicated.

The test name implies it verifies replay determinism (single-execution, same output), but it only verifies that two independent executions of the same constant mock produce the same output. A real executor that mutates DB state would execute twice without protest.

**Fix:** Either add an `idempotency_key` to the approve fixture and assert `exec_count == 1`, or rename the test to reflect what it actually verifies (`output_shape_stable_across_two_calls`). The existing `write_turn` test (which asserts `exec_count == 1` for a single call) does not cover the two-call path:

```json
// approve-order.json — add idempotency_key to fixture arguments
{
  "turn_id": "approve-order",
  "nl_message": "approve the order from Alice",
  "expected_tool": "approve",
  "recorded_selection": {
    "tool_name": "approve",
    "arguments": { "id": 42, "idempotency_key": "replay-det-001" },
    "confidence": 0.92
  }
}
```

Then add an `exec_count` assertion in `replay_deterministic` matching the idempotent-write contract.

---

## Info

### IN-01: `tool_name` routing decision uses `starts_with("list_")` — tolerates injected tool names like `"list_"` (bare prefix)

**File:** `ferro-mcp-server/src/intent.rs:174`

**Issue:** A prompt-injected `tool_name` of exactly `"list_"` (no service suffix) routes to the read path (`handle_tools_call`). `handle_tools_call` calls `tool_name.strip_prefix("list_")` (returning `""`) then looks for a service with `name == ""` and `mcp_exposed == true`. Because no such service exists, it returns a `-32601 Method not found` error. The behavior is correct but slightly opaque.

This is not exploitable — the downstream service lookup denies it — but the routing condition could be made explicit:

```rust
// More precise: require at least one character after "list_"
if sel.tool_name.starts_with("list_") && sel.tool_name.len() > 5 {
```

---

### IN-02: `ambiguous.json` fixture `expected_tool` value is misleading

**File:** `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/ambiguous.json:5`

**Issue:** The fixture sets `"expected_tool": "list_order"` for the ambiguous case, but the test's purpose is to trigger `LowConfidence` (confidence 0.3 < threshold 0.7) — the tool is never dispatched. The `expected_tool` field is checked only in the `fixtures_parse_and_replay_returns_recorded` test (line 125) and the live-eval test (line 264), where `list_order` happens to be the recorded `tool_name`. This is accurate for the replay (the provider returns the recorded value), but semantically confusing: the "expected" tool in an ambiguous scenario is never actually called.

A comment or a separate `expected_outcome: "needs_clarification"` field would reduce future confusion about what this fixture is asserting.

---

_Reviewed: 2026-06-14T02:21:21Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
