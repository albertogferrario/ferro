---
phase: 232-single-source-write-surfaces
reviewed: 2026-06-16T00:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - framework/src/write/mod.rs
  - framework/src/lib.rs
  - framework/Cargo.toml
  - ferro-mcp-server/src/write_dispatch.rs
  - ferro-mcp-server/src/error.rs
  - ferro-mcp-server/src/lib.rs
  - ferro-mcp-server/src/intent.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - ferro-mcp-server/Cargo.toml
  - app/src/controllers/visual_action.rs
  - app/src/controllers/mcp.rs
  - app/src/controllers/mod.rs
  - app/src/routes.rs
  - app/src/tests/single_source.rs
  - app/src/tests/visual_action.rs
  - app/src/tests/mcp_write_dispatch.rs
  - app/src/tests/mod.rs
  - ferro-mcp-server/tests/jsonrpc_integration.rs
  - ferro-mcp-server/tests/mcp_tenant_isolation.rs
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 232: Code Review Report

**Reviewed:** 2026-06-16T00:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Phase 232 relocates the write-execution kernel from `ferro-mcp-server` into
`framework::write` and adds a new public visual write endpoint
`POST /{service}/{action}`. The security envelope is solid: the relocated kernel
preserves every reviewed invariant from phase 231, and the new visual surface
derives `to_state` and `tenant_id` from authoritative sources only.

Security verification (all PASS):

- **`to_state` is never request-influenced.** Both channels derive the target
  state exclusively from `derive_transition_plan(svc, action_name).to_state`
  (`executor.rs:84`, sourced from `Transition.to`). The executor ignores any
  `status`/`to_state` in the body; `app/src/tests/visual_action.rs:351`
  (`visual_action_rejects_form_supplied_to_state`) regression-locks this.
- **`tenant_id` comes only from auth.** `visual_action.rs:40` reads
  `ferro::current_tenant()`; the executor's `find_by_id(...).filter(Column::TenantId.eq(tenant_id))`
  is the cross-tenant denial primitive. Cross-tenant write returns the same
  redacted "not found or cross-tenant access denied" message — no resource
  enumeration. Locked by `visual_cross_tenant_denied`.
- **Guards are re-evaluated server-side** inside the shared `dispatch_write`
  (`write/mod.rs:334-344`), fail-closed on both `Ok(false)` and `Err`. The app
  evaluator denies unknown guard names (`mcp.rs:135`). The visibility cache
  (`ctx.evaluated_guards`) is never consulted at call time.
- **WR-01 ordering survived the move.** Idempotency-store (step 5) and audit
  (step 6) seal *before* the post-persist override (step 7) — `write/mod.rs:400-433`.
  `override_error_surfaces` proves the base audit + idempotency key persist even
  when the override fails.
- **Audit channel is parameterized, not spoofable.** Each caller passes a
  literal (`"mcp"` / `"web"`); the channel arrives as a fixed argument, never
  from the payload (`write/mod.rs:414`). `audit_channel_is_parameterized` locks it.
- **Error redaction holds.** `Database`/`Serialization` variants are collapsed
  to `"write operation failed"` on both channels (`visual_action.rs:118`,
  `write_dispatch.rs:252`); no SQL/table/column leaks.
- **No dependency cycle.** `framework/Cargo.toml` has no `ferro-mcp-server` dep;
  `ferro-mcp-server` depends on `ferro-rs` with `default-features = false`. The
  kernel owns its self-contained `WriteError` and never imports a channel type.
- **`From<WriteError> for Error`** (`error.rs:48`) is variant-for-variant with no
  loss and no panic; the `#[cfg(feature="confirmation")]` arm is correctly gated.
- **Route precedence is correct.** matchit prefers literals, so the two-segment
  literals (`/auth/login`, `/mcp/chat`, `/users/{user}`) win; no existing
  all-param two-segment route exists to conflict with `/{service}/{action}`, so
  the silent `.insert(...).ok()` drop (router.rs:240) cannot fire here. The
  route carries `TenantMiddleware(...).on_failure(Forbidden)`.

Two warnings concern correctness of the new endpoint's primary use case (HTML
form bodies) and a latent route-conflict fragility. No critical issues.

## Warnings

### WR-01: Form-urlencoded bodies break id extraction — the endpoint's primary path fails

**File:** `app/src/controllers/visual_action.rs:67`
**Issue:** The endpoint exists to handle the projection renderer's emitted
action-button URL (`ferro-json-ui/.../builder.rs:685`), which a browser submits
as `application/x-www-form-urlencoded`. `req.input::<Value>()` routes form bodies
through `serde_urlencoded::from_bytes::<Value>` (`request.rs:588`, `body.rs:32`).
urlencoded carries no type information, so every field deserializes as a JSON
**string**: a form `id=1` yields `{"id": "1"}`, not `{"id": 1}`.

The reused executor extracts the id with `inputs["id"].as_i64()`
(`mcp.rs:75`), which returns `None` for the string `"1"`, producing
`WriteError::Validation("missing id")` → HTTP 422. A JSON POST works (integer
preserved); a standard HTML form POST does not. The new tests
(`visual_action.rs`, `single_source.rs`) all pass JSON `json!({"id": 1})`
directly to `dispatch_write`, so they never exercise the form-decode path and
miss this gap.

**Fix:** Make id extraction tolerant of string-encoded numerics in the executor
closure, e.g.:
```rust
let id_val = inputs["id"]
    .as_i64()
    .or_else(|| inputs["id"].as_str().and_then(|s| s.parse::<i64>().ok()));
```
Or coerce numeric form fields in the visual handler before dispatch. Add a test
that drives `controllers::visual_action::handle` with an actual
`application/x-www-form-urlencoded` body (not a hand-built JSON `Value`) so the
form path is covered.

### WR-02: Silent `.insert(...).ok()` route registration masks future param-route conflicts

**File:** `framework/src/routing/router.rs:240` (registration of `visual_action` at `app/src/routes.rs:119`)
**Issue:** The router swallows matchit insert errors with `.insert(path, ...).ok()`.
Today `/{service}/{action}` registers cleanly because every other two-segment
route has a literal first segment. But matchit rejects a *second* all-param
two-segment route at the same tree position (e.g. a future
`/{tenant}/{resource}`), and that rejection would be silently dropped — the
route would simply not exist, with no error at boot. For a security-relevant
write surface, a dropped registration is a fail-open-shaped surprise (the route
vanishes rather than denying). The phase's own `visual_route_registered_without_shadowing`
test guards precedence but not the silent-drop failure mode.

**Fix:** This is pre-existing framework behavior, not introduced by this phase,
so the minimal phase-scoped action is to (a) keep the precedence test and (b)
file a framework follow-up to surface insert conflicts (log/panic at boot or
return a registration `Result`) rather than `.ok()`-discarding them. At
minimum, add an assertion in the route test that the visual pattern actually
resolves (already present), and document the single-all-param-route invariant
near the route registration so a future second catch-all is caught in review.

## Info

### IN-01: Cross-tenant denial reported as 422 `invalid_request`, not 403/404

**File:** `app/src/controllers/visual_action.rs:104-109`
**Issue:** The executor returns `WriteError::Validation("not found or cross-tenant
access denied")` for both genuine validation failures and cross-tenant attempts
(`mcp.rs:93`). The visual handler maps `Validation` to HTTP 422
`invalid_request`. This is acceptable and arguably good (no resource
enumeration, uniform response), but conflates "your input was malformed" with
"you may not touch this record." A caller cannot distinguish a 422 caused by a
missing id from one caused by a cross-tenant target.
**Fix:** No change required for security. If clearer semantics are wanted later,
introduce a distinct `WriteError::NotFound` variant mapped to 404 for the
find-miss case, keeping the message redacted. Document the current conflation
intentionally if it stays.

### IN-02: `ctx` parameter is dead in the write path

**File:** `ferro-mcp-server/src/write_dispatch.rs:116`
**Issue:** `handle_write_call` takes `ctx: &crate::McpContext` and immediately
discards it with `let _ = ctx;`. The doc comment says it is retained "for future
extensions (e.g. tracing)." This is dead-parameter plumbing threaded through
several signatures.
**Fix:** Acceptable if the future use is imminent; otherwise drop the parameter
from `handle_write_call` and its callers to reduce signature noise. Low priority.

### IN-03: `derive_transition_plan` recomputed twice per visual write

**File:** `app/src/controllers/visual_action.rs:60` and `app/src/controllers/mcp.rs:108`
**Issue:** The visual handler derives the plan once to get `transition_guard`
(step 4), then the reused executor calls `derive_transition_plan` again to get
`to_state`. Two derivations of the same pure plan per request. This is a clarity/
duplication note, not a correctness issue — the derivation is pure and cheap, and
performance is explicitly out of v1 review scope.
**Fix:** Optionally pass the already-derived `TransitionPlan.to_state` into the
executor rather than re-deriving, consolidating to one derivation. Low priority.

---

_Reviewed: 2026-06-16T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
