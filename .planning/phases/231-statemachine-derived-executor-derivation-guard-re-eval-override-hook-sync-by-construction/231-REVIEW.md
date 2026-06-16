---
phase: 231-statemachine-derived-executor-derivation-guard-re-eval-override-hook-sync-by-construction
reviewed: 2026-06-16T00:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - ferro-projections/src/executor.rs
  - ferro-projections/src/service.rs
  - ferro-projections/src/error.rs
  - ferro-projections/src/lib.rs
  - framework/src/lib.rs
  - ferro-mcp-server/src/write_dispatch.rs
  - ferro-mcp-server/src/lib.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - app/src/controllers/mcp.rs
  - app/src/tests/mcp_write_dispatch.rs
  - app/src/tests/mcp_tenant_isolation.rs
  - ferro-mcp-server/tests/intent_loop.rs
  - ferro-mcp-server/tests/jsonrpc_integration.rs
  - ferro-mcp-server/tests/mcp_tenant_isolation.rs
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 231: Code Review Report

**Reviewed:** 2026-06-16T00:00:00Z
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 231 derives the write-transition target and guard from the declared `StateMachine`
(`derive_transition_plan`), unions the transition guard with the action preconditions in the
live guard loop (`merged_guards`), and adds a post-persist override seam (`OverrideFn`).

The core security envelope is sound. The authorization-critical claims hold:

- **Guard union is correct.** `merged_guards` (write_dispatch.rs:127) builds `preconditions ∪
  {transition_guard}`, deduped by name, order-preserving. `dispatch_write` (line 338-348)
  evaluates that union through the live `GuardEvaluatorFn`, fail-closed on `Ok(false)` OR `Err`.
  `ctx.evaluated_guards` is never consulted on the call path. Verified by
  `guard_deduped_when_on_both`, `transition_guard_evaluated_at_call_time`,
  `guard_rejects_illegal_transition`.
- **`to_state` is not caller-influenced.** It is sourced only from `Transition.to` via
  `derive_transition_plan` (executor.rs:84); the executor in mcp.rs:108-110 reads
  `plan.to_state`, never request input. Confirmed by `submit_persists_derived_to_state`.
- **Override runs inside the guarded/audited window** strictly after the base persist
  (write_dispatch.rs:398-400) and cannot suppress the base guard or transition; its `Err`
  propagates via `?` without panicking (`override_error_surfaces`).
- **Derivation is total** — every failure mode returns a typed `Err`; no `unwrap`/`expect`/
  `panic!`/indexing-out-of-bounds on the non-test path. `matches[0]` accesses are guarded by
  the prior `is_empty()` check (executor.rs:77).
- **`ferro-projections` stays schema-only** — executor.rs imports only `schemars`, `serde`, and
  `crate::state::Transition`; no async/sea-orm/tokio/closures leak in.
- **Facade re-export is correct** — `derive_transition_plan` and `TransitionPlan` are exported
  from both `ferro_projections::lib` (lib.rs:17) and re-exported through `framework/src/lib.rs`
  (lines 258, 261) under the `projections` feature.

The two warnings are ordering/consistency concerns in the override-and-audit sequence and the
confirmation-path guard set; neither is a guard bypass. Info items are minor robustness and
duplication notes.

## Warnings

### WR-01: Override runs after base audit; a failing override leaves an audit entry that overstates what persisted

**File:** `ferro-mcp-server/src/write_dispatch.rs:392-419`
**Issue:** The pipeline order is: executor (step 4, line 392) → override hook (step 4b, line
398-400) → idempotency store (step 5) → audit (step 6, line 411). The override `Err` short-
circuits with `?` at line 399 *before* idempotency-store and audit run, so on override failure
no audit entry is written for this call at all — yet the base persist (the executor's DB write)
has already committed. There is no surrounding transaction, so the executor's mutation is not
rolled back when the override fails. The result: a committed state transition with **no audit
record and no idempotency record**, which both breaks the SC#4 "every successful execution is
audited" invariant (the base write succeeded) and means a retried call with the same
`idempotency_key` will re-execute (no stored key), double-applying the base transition plus a
fresh override attempt.

The doc comment on `OverrideFn` (line 74-84) and the test `override_error_surfaces`
(line 1216-1250) assert only that the error "surfaces"; neither covers the audit/idempotency
gap. The test comment even states "The base write's audit already happened" — but it does not,
because audit is step 6, after the override at step 4b.

**Fix:** Decide the intended semantics and make the code match. Two coherent options:

```rust
// Option A — audit + store idempotency for the base persist BEFORE the override,
// so a base persist that committed is always recorded regardless of override outcome.
let result = (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?;
if let Some(key) = idempotency_key {
    store_idempotency(tenant_id, key, &result, db).await?;
}
// audit the base persist here (move step 6 up)...
if let Some(hook) = dispatcher.overrides.get(&action.name) {
    (hook)(&action.name, inputs, tenant_id, db, &result).await?; // override failure no longer erases the base audit
}
Ok(result)
```

```rust
// Option B — run executor + override inside a single DB transaction so an override
// failure rolls back the base persist, restoring "nothing happened, nothing audited".
// (Requires threading a txn handle into ExecutorFn/OverrideFn — larger change.)
```

Option A is the smaller change and matches the existing "no transaction" design; document that
an override failure does NOT roll back the base persist but IS still audited.

### WR-02: `handle_request_confirm` token issuance evaluates only `action.preconditions`, not the transition-guard union

**File:** `ferro-mcp-server/src/write_dispatch.rs:639` (and the parallel pre-loop in
`handle_confirm`, line 799)
**Issue:** Both confirmation handlers re-evaluate guards by iterating `action.preconditions`
directly, bypassing `merged_guards`/`derive_transition_plan`, so a transition-level guard that
is NOT also an action precondition is not checked at token-issuance time (and not in the
confirm-time pre-loop). This is **not a security bypass for execution**: `handle_confirm`
subsequently calls `dispatch_write(..., transition_guard, true)` (line 818-826), which evaluates
the full union and fail-closes — so the actual write is still fully guarded. The defect is
narrower fail-fast behavior: a token can be *issued* (and the confirm pre-check can pass) for a
transition whose transition-only guard would later deny in `dispatch_write`, wasting the
round-trip and producing a slightly less precise denial point. It is also an inconsistency with
`handle_write_call`, which derives `transition_guard` from the plan (line 505-506).

**Fix:** Derive the transition guard once and evaluate the union in both confirmation handlers,
mirroring `handle_write_call`:

```rust
let plan = derive_transition_plan(svc, &action.name).ok();
let transition_guard = plan.as_ref().and_then(|p| p.guard.as_deref());
let guards = merged_guards(&action.preconditions, transition_guard);
for guard_name in &guards {
    let passes = (dispatcher.guard_evaluator)(guard_name, tid, &args, db).await /* ... */;
    // fail-closed as today
}
```

Note `handle_request_confirm` currently does not look up `svc` (only `_svc`); it would need the
service ref to derive the plan.

## Info

### IN-01: `TransitionPlan.guard` documents "re-checked LIVE (EXEC-02)" but carries only `matches[0].guard`

**File:** `ferro-projections/src/executor.rs:84-96`
**Issue:** When a multi-source event has different per-transition guards (e.g.
`draft→cancel` unguarded and `submitted→cancel` guard("cancellation_allowed")), the plan keeps
only the first transition's guard (line 96), silently dropping the others. All matches share one
target (enforced by the `AmbiguousTransition` check) but may legitimately differ in guard. The
inline comment acknowledges "all matches share one event/target" but not that guards may differ.
For the current single-source-guarded fixtures this is correct, but the derivation is lossy for
the per-source-guard case and could under-guard a future state machine.
**Fix:** Either (a) document the constraint explicitly ("per-event guards must be uniform across
sources; the first is taken") and add a validation warning in `service.rs::validate` when a
multi-source event has non-uniform guards, or (b) carry `Vec<(from, Option<guard>)>` so the
consumer can pick the guard matching the record's current state. (a) is sufficient for v1.

### IN-02: Override failure error class is not redacted in the NL/confirm response path

**File:** `ferro-mcp-server/src/write_dispatch.rs:838-849`
**Issue:** In `handle_confirm`, an override returning a non-`GuardFailed` error falls into the
`Err(_)` arm and is redacted to "write operation failed" (good). But in `handle_write_call`
(line 569-573), a `Validation`/`ActionNotFound` override error is passed through verbatim under
`error_kind: "execution_error"`. An override hook is app-supplied and may build a `Validation`
message containing internal detail; the pass-through assumes those messages are agent-safe. This
matches the documented contract (the executor/override owns audit-safe output) but is worth an
explicit note since override authors are a new extension point introduced this phase.
**Fix:** Document on `OverrideFn` that `Validation`/`ActionNotFound` messages it returns are
surfaced verbatim to the caller and must not contain internal detail; or redact all override-
originated errors uniformly.

### IN-03: `validate()` runs full derivation per transition-triggering action (registration cost)

**File:** `ferro-projections/src/service.rs:425-431`
**Issue:** Step 5b calls `derive_transition_plan` for every transition-triggering action, and
each call re-scans `svc.actions` and `sm.transitions`. For a service with many actions this is
O(actions × (actions + transitions)). This is registration-time only (not a hot path) and the
sync-by-construction guarantee it buys is worth it; noting for awareness, not action.
**Fix:** None required for v1. If service registration ever shows up in startup profiles, hoist
the action/event lookups.

### IN-04: `dispatch_write` audit `record_id` derives from `inputs["id"]` via `.to_string()` on a JSON value

**File:** `ferro-mcp-server/src/write_dispatch.rs:410`
**Issue:** `inputs.get("id").map(|v| v.to_string())` stringifies the raw JSON value, so a numeric
id audits as `"1"` while a string id audits as `"\"1\""` (with embedded quotes), and an absent id
audits as `""`. The audit `target_id` is thus shape-dependent on the JSON type of the supplied
id, which can complicate `history_for_target` lookups across heterogeneous callers. The
companion denial-audit path (line 545) has the same pattern.
**Fix:** Normalize to the unquoted scalar, e.g.
`inputs.get("id").map(|v| v.as_i64().map(|n| n.to_string()).unwrap_or_else(|| v.as_str().map(str::to_string).unwrap_or_default())).unwrap_or_default()`,
or a small helper, so numeric and string ids both audit as `1`.

---

_Reviewed: 2026-06-16T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
