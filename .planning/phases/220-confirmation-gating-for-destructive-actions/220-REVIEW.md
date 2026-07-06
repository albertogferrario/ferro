---
phase: 220-confirmation-gating-for-destructive-actions
reviewed: 2026-06-14T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - ferro-mcp-server/src/write_dispatch.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/config.rs
  - ferro-mcp-server/src/error.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - ferro-ai/Cargo.toml
  - ferro-ai/src/lib.rs
  - ferro-mcp-server/Cargo.toml
  - app/src/controllers/mcp.rs
  - app/Cargo.toml
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 220: Code Review Report

**Reviewed:** 2026-06-14
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Phase 220 implements a two-step confirmation gate (`request_confirm_<action>` → `confirm_<action>`) for destructive MCP actions, backed by `ferro-ai::InMemoryConfirmationStore`. The core security invariants hold: the token is server-generated (CSPRNG, BASE62, ~256-bit entropy), never read from agent input, consumed atomically on `confirm()` (single-use), and bound to `(tenant_id, action_name, record_id)` — all three dimensions are verified before execution. Guards are re-evaluated at confirm time with live DB state. The D-08 seam is correctly gated by `is_confirmed: bool` which is only ever set to `true` by `handle_confirm` after token validation; no agent-controlled code path can reach the executor with `is_confirmed=true`. Feature-gating compiles correctly in both on and off configurations. The `OnceLock<InMemoryConfirmationStore>` singleton is sound for a process-scoped store.

Four warnings and three info items are noted, primarily around error string leakage in the confirmation flow (counter to the 219 CR-01 redaction discipline established for the same pipeline), a TOCTOU window in token storage timing, a missing TTL cap comment, and minor quality issues.

## Warnings

### WR-01: Internal error strings from `ConfirmationStore` leaked to agent in `handle_request_confirm`

**File:** `ferro-mcp-server/src/write_dispatch.rs:598-601`
**Issue:** When `store.request_confirmation()` returns `Err`, the error string `{e}` is formatted directly into the MCP response `message` field. The `ConfirmationStore` implementation is app-pluggable; an implementation returning a DB error, lock error, or internal state detail would expose that string to the agent caller. This contradicts the 219 CR-01 discipline already applied to `Database`, `Serialization`, and `Auth` variants in the same file (line 499), where all non-agent-safe errors are redacted to `"write operation failed"`.

```rust
// Current (line 598-601):
return json!({ "result": write_tool_error_result(json!({
    "error_kind": "execution_error",
    "message": format!("failed to store confirmation: {e}")
})) });

// Fix — redact internal detail:
return json!({ "result": write_tool_error_result(json!({
    "error_kind": "execution_error",
    "message": "failed to store confirmation token"
})) });
```

### WR-02: Internal error strings from `ConfirmationStore` leaked to agent in `handle_confirm`

**File:** `ferro-mcp-server/src/write_dispatch.rs:663-667`
**Issue:** Same class as WR-01. When `store.confirm()` returns `Err`, the raw error string is passed directly into the agent-visible `message` field. This is the hot path (called on every confirm attempt) and the only place in `handle_confirm` where internal store state can escape.

```rust
// Current (line 663-667):
Err(e) => {
    return json!({ "result": write_tool_error_result(json!({
        "error_kind": "execution_error",
        "message": format!("confirmation store error: {e}")
    })) });
}

// Fix:
Err(_) => {
    return json!({ "result": write_tool_error_result(json!({
        "error_kind": "execution_error",
        "message": "confirmation store error"
    })) });
}
```

### WR-03: Guard error detail at confirm time leaks internal guard name and error string

**File:** `ferro-mcp-server/src/write_dispatch.rs:711-714`
**Issue:** At confirm time, when a guard evaluator returns `Err(e)`, both the guard name and the full error string are forwarded to the agent (`format!("precondition '{guard_name}' error at confirm time: {e}")`). The guard evaluator in `app/src/controllers/mcp.rs` (line 127) can produce `ferro_mcp_server::Error::GuardFailed` strings containing internal details. Additionally, this is asymmetric with the request-time guard error path (line 565, which also leaks but is at least consistent). The pattern for guard errors already established elsewhere in `dispatch_write` (line 279) uses `crate::Error::GuardFailed` which is then redacted at the handler boundary — the same redaction should apply here.

```rust
// Current (line 711-714):
"message": format!("precondition '{guard_name}' error at confirm time: {e}")

// Fix — guard name and error detail are internal; redact:
"message": "precondition not met at confirm time"
```

Note: the same issue exists in `handle_request_confirm` at line 565 (`format!("precondition '{guard_name}' error: {e}")`), though its impact is lower since request_confirm is a read-like operation.

### WR-04: TOCTOU window between `store.request_confirmation()` success and response delivery

**File:** `ferro-mcp-server/src/write_dispatch.rs:590-608`
**Issue:** In `handle_request_confirm`, the token is generated and stored first (line 590-602), then the `confirmation_token` is included in the response. If the response fails to deliver (network error, handler panic after this point), the token is already live in the store and will remain until TTL expiry. A replay attacker who intercepts the network failure and retries the request will receive a second fresh token for the same action — the old token also remains valid until it expires.

This is not exploitable in isolation (an attacker would need a valid session plus a network-level interception), but it means a single `request_confirm` invocation can leave multiple valid tokens for the same action alive simultaneously if the caller retries. The `request_confirmation` implementation correctly aborts the old TTL task and replaces the entry when the same key is re-inserted, but the key here is the token itself (generated fresh each call), not the action — so retried calls create independent entries.

**Fix:** The reliable mitigation is to key the store entry on `(tenant_id, action_name, record_id)` instead of the token, storing the token inside the payload. This makes `request_confirmation` idempotent per logical action — a retry replaces the previous pending entry. The token is then retrieved from the stored payload at confirm time. This requires a schema change to the confirmation flow but eliminates the multiple-live-token window. As a lower-effort stopgap, document the behavior and note it as accepted risk.

## Info

### IN-01: `confirmation_ttl_seconds` minimum of 300s is not validated in tests or documented as a design constraint

**File:** `ferro-mcp-server/src/config.rs:43-47`
**Issue:** The TTL is clamped to `300..=600` seconds (5–10 minutes). The comment says "Range: 300–600 (5–10 min)" but neither the rationale for the minimum (prevents accidentally short TTLs that expire before the user can act) nor the maximum (limits the attack window for a stolen token) is documented. The SC#3 test in `write_dispatch.rs:1314` uses a 5-second TTL, which would be clamped to 300 in production — the test bypasses config and passes `ttl_secs=5` directly to `handle_request_confirm`, so this is test-only and correct, but the discrepancy between test TTL and production minimum TTL is not commented.

**Fix:** Add a brief inline comment on the clamp explaining the security rationale (min: user has time to act; max: limits stolen-token window). No code change required.

### IN-02: `app/Cargo.toml` declares `ferro-ai` as both a direct optional dependency and pulls it transitively through `ferro-mcp-server/confirmation`

**File:** `app/Cargo.toml:24,33`
**Issue:** The `app` crate declares `ferro-ai` as a direct optional dependency (line 24) with `default-features = false, features = ["confirmation"]`, and also activates it via `ferro-mcp-server/confirmation` (line 33). This means when the `confirmation` feature is enabled, `ferro-ai` appears twice in the resolution — once as a direct dep and once as transitive — which Cargo unifies correctly but makes the dependency intent harder to read. The direct `dep:ferro-ai` could be dropped if the app only needs `ferro_ai::InMemoryConfirmationStore`, since that type is already re-exported via `ferro-mcp-server` when the `confirmation` feature is on.

**Fix:** Verify whether any `ferro-ai` symbol is imported directly in `app/src/controllers/mcp.rs` (line 27 imports `ferro_ai::InMemoryConfirmationStore` directly). If so, the direct dependency is needed. If the type is accessible via `ferro_mcp_server::ferro_ai::...` or a re-export, the direct dep can be dropped. Current state is not wrong, just redundant — document or clean up.

### IN-03: `handle_confirm` reads `record_id` from agent-supplied `args` for the binding check

**File:** `ferro-mcp-server/src/write_dispatch.rs:687-693`
**Issue:** The record mismatch check compares `args.get("id")` (from the `confirm_<action>` call, agent-supplied) against `binding.get("record_id")` (from the stored payload, server-set at request time). This is architecturally correct — the token is already consumed from the store before this check runs (line 654), so an attacker who fails the mismatch check loses the token and cannot retry. However, the `confirm_<name>` tool schema declares `id` as a required field (renderer.rs:319), yet `handle_confirm` does not validate the `id` field is present using `validate_action_inputs` before performing the binding check. A missing `id` produces `call_record_id = None`, which will only match a stored `record_id = null` (i.e., an action confirmed without a record ID at request time). The path is safe by coincidence for non-null record IDs, but should be explicit.

**Fix:** Call `validate_action_inputs` on the confirm-step action (or at minimum check `args.get("id").is_some()`) before the binding comparison to make the validation explicit rather than implicit.

---

_Reviewed: 2026-06-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
