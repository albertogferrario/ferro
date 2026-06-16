---
phase: 231-statemachine-derived-executor-derivation-guard-re-eval-override-hook-sync-by-construction
plan: 02
subsystem: ferro-mcp-server / app write path
tags: [write-dispatch, state-machine, guard-reeval, override-hook, projections]
requires:
  - "ferro_projections::derive_transition_plan() (Plan 01)"
  - "ferro_projections::TransitionPlan (Plan 01)"
  - "ferro::derive_transition_plan / ferro::TransitionPlan facade re-export (Plan 01)"
  - "ferro-mcp-server::dispatch_write live guard loop (Phase 219)"
provides:
  - "ferro-mcp-server::dispatch_write transition_guard param — preconditions ∪ transition-guard, deduped, live (EXEC-02)"
  - "ferro-mcp-server::OverrideFn + WriteDispatcher.overrides post-persist registry (EXEC-03)"
  - "WriteDispatcher::new()/with_override() consuming builder"
  - "app executor derives to_state via ferro::derive_transition_plan — no match action_name (EXEC-01 end-to-end)"
affects:
  - "Phase 232 — cross-surface wiring (MCP + visual/form), retire hand-written WriteDispatcher (EXEC-05)"
tech-stack:
  added: []
  patterns:
    - "Boxed-future callback (no async-trait) for OverrideFn, mirroring ExecutorFn"
    - "Union+dedup-by-name of guard surfaces in the single live GuardEvaluatorFn loop"
    - "Consuming builder (with_override: mut self -> Self) per ferro convention"
key-files:
  created: []
  modified:
    - "ferro-mcp-server/src/write_dispatch.rs"
    - "ferro-mcp-server/src/lib.rs"
    - "ferro-mcp-server/src/jsonrpc.rs"
    - "ferro-mcp-server/tests/intent_loop.rs"
    - "ferro-mcp-server/tests/jsonrpc_integration.rs"
    - "ferro-mcp-server/tests/mcp_tenant_isolation.rs"
    - "app/src/controllers/mcp.rs"
    - "app/src/tests/mcp_write_dispatch.rs"
    - "app/src/tests/mcp_tenant_isolation.rs"
decisions:
  - "Transition guard threaded into dispatch_write as an explicit Option<&str> param (not re-derived inside) — keeps the signature stable for Phase 232 and the caller owns the ferro_projections dependency"
  - "Guard union built in dispatch_write via merged_guards(); the no-transition path returns action.preconditions unchanged (back-compatible)"
  - "Override registry lives on WriteDispatcher (consumer runtime), keyed by action name; runs post-persist so it cannot suppress the base guard/transition (T-231-05)"
  - "App test fixtures (mcp_write_dispatch.rs) also switched off the hand-written match to derive_transition_plan, so `grep -rn 'match action_name' app/src` is empty as required"
metrics:
  duration: ~28m
  completed: 2026-06-16
requirements: [EXEC-01, EXEC-02, EXEC-03]
---

# Phase 231 Plan 02: StateMachine-Derived Executor — Consumer Write-Path Wiring Summary

Wired the Plan 01 derivation into the existing write-dispatch path: `dispatch_write` now re-evaluates the union of `action.preconditions` and the transition-level `TransitionPlan.guard` (deduped by name) through the single live `GuardEvaluatorFn` loop (EXEC-02); a post-persist `OverrideFn` registry on `WriteDispatcher` lets the app attach side effects without replacing the base dispatch (EXEC-03); and the app executor derives `to_state` from `ferro::derive_transition_plan(...).to_state` — the hand-written `match action_name => new_status` is deleted everywhere in `app/src` (EXEC-01 end-to-end).

## What Was Built

**EXEC-02 — transition-guard union + dedup (`ferro-mcp-server/src/write_dispatch.rs`):**
- `dispatch_write` gains a `transition_guard: Option<&str>` parameter. A new `merged_guards(preconditions, transition_guard)` helper unions the two guard surfaces, deduping by string equality and preserving order; the existing step-1 live guard loop iterates the merged set through the unchanged `(dispatcher.guard_evaluator)(...)` call. `ctx.evaluated_guards` is never consulted (the forbidding comment is preserved and is the only reference).
- `handle_write_call` and `handle_confirm` derive the transition guard via `ferro_projections::derive_transition_plan(svc, &action.name).ok()` and pass `plan.guard.as_deref()`. `.ok()` (not `?`) keeps a non-transition action dispatching with an empty transition guard, so the union equals `action.preconditions` exactly (back-compatible).

**EXEC-03 — post-persist override registry (`write_dispatch.rs`, `lib.rs`):**
- `pub type OverrideFn` — boxed-future hook `(action, inputs, tenant, db, base_result) -> Result<()>`, mirroring `ExecutorFn` (no `async-trait`). Receives the base persist result for chaining related-record writes.
- `WriteDispatcher` gains `overrides: HashMap<String, OverrideFn>`, plus a `new(executor, guard_evaluator)` constructor and a consuming `with_override(action, hook)` builder. In `dispatch_write` the hook runs **after** the executor (step 4b) and **before** idempotency store / audit-of-base — inside the guarded, audited window, so it cannot bypass the base guard or transition (T-231-05). Absent key = declaration-only common path.
- `OverrideFn` re-exported from `lib.rs`.

**EXEC-01 (end-to-end) — derived `to_state` in the app (`app/src/controllers/mcp.rs`):**
- The executor closure resolves the `ServiceDef` from `exposed_services()` and derives `new_status` from `ferro::derive_transition_plan(svc, &action_name).to_state` (facade path only — no `ferro_projections::` in `app/`). The `match action_name { "submit" => "submitted", ... }` block is deleted.
- `make_write_dispatcher` builds the dispatcher via `WriteDispatcher::new(...)` — no override registered (common path stays declaration-only).

## Tasks

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1+2 | Transition-guard union/dedup (EXEC-02) + post-persist override seam (EXEC-03) | 8c87c102 | write_dispatch.rs, lib.rs, jsonrpc.rs, 3 integration tests |
| 3 | Derive to_state in app executor — delete hand-written match (EXEC-01) | dbc35374 | controllers/mcp.rs, tests/mcp_write_dispatch.rs, tests/mcp_tenant_isolation.rs |

## Verification

- `cargo test -p ferro-mcp-server --all-features` — 50 lib + 5 + 9 + 5 + 4 tests pass. New: `guard_rejects_illegal_transition`, `transition_guard_evaluated_at_call_time`, `guard_deduped_when_on_both`, `override_hook_runs_post_persist`, `no_override_is_declaration_only`, `override_error_surfaces`. Regression `guard_denied_at_call_time` still passes.
- `cargo build -p ferro-mcp-server --all-features` — exits 0 (all dispatch_write call sites :735/:866/:906/:916/:1205 + WriteDispatcher literals updated to the new arity).
- `cargo build -p app` — exits 0. `cargo test -p app mcp_write_dispatch` — 4 tests pass, incl. new `submit_persists_derived_to_state` (draft → submit → derived "submitted").
- `grep -rn 'match action_name' app/src` — empty.
- `grep -n 'ferro_projections::' app/src/controllers/mcp.rs` — empty (facade-only).
- `grep -n 'derive_transition_plan' ferro-mcp-server/src/write_dispatch.rs` — matches (call sites + import).
- `grep -n 'evaluated_guards' ferro-mcp-server/src/write_dispatch.rs` — only the forbidding comments; never read for a guard decision.
- `grep -n 'pub type OverrideFn' write_dispatch.rs` / `grep -n 'OverrideFn' lib.rs` — match.
- **Full workspace gate:** `cargo fmt --all -- --check` clean; `cargo clippy --all --all-targets -- -D warnings` clean; `cargo test --all-features` exited 0 (no failures, no ENOSPC). Disk had ~14Gi free.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated additional `WriteDispatcher` struct literals beyond the plan's enumerated list**
- **Found during:** Task 2 (adding the `overrides` field to `WriteDispatcher`).
- **Issue:** Adding `overrides` broke every struct-literal construction, including ones the plan did not enumerate: `ferro-mcp-server/src/jsonrpc.rs`, `ferro-mcp-server/tests/intent_loop.rs` (7 sites), `ferro-mcp-server/tests/jsonrpc_integration.rs`, `ferro-mcp-server/tests/mcp_tenant_isolation.rs`, `app/src/tests/mcp_tenant_isolation.rs`. The crate/workspace would not compile otherwise.
- **Fix:** Converted the no-op test helpers to the new `WriteDispatcher::new(...)` constructor; inserted `overrides: std::collections::HashMap::new()` into the `intent_loop.rs` literals that set custom executors.
- **Files modified:** as listed above.
- **Commit:** 8c87c102.

**2. [Rule 3 - Blocking] App test fixtures in `mcp_write_dispatch.rs` carried their own `match action_name`**
- **Found during:** Task 3 (the `grep -rn 'match action_name' app/src` acceptance check).
- **Issue:** Two test-local executor fixtures duplicated the production `match action_name => new_status`. Leaving them would fail the phase success criterion (`grep` must be empty).
- **Fix:** Both fixtures now derive `new_status` via `ferro::derive_transition_plan(svc, &action_name).to_state`, resolving the `ServiceDef` through `crate::controllers::mcp::exposed_services()`. Added `submit_persists_derived_to_state` to assert the derived target persists end-to-end.
- **Files modified:** app/src/tests/mcp_write_dispatch.rs.
- **Commit:** dbc35374.

**Task commit granularity:** Tasks 1 and 2 both modify `ferro-mcp-server/src/write_dispatch.rs` with tightly interleaved hunks (the `overrides` field, the `transition_guard` param, and their test call-site updates land in the same struct/function). Since the environment cannot split a single file's hunks non-interactively, they were committed as one atomic commit (8c87c102) covering EXEC-02 + EXEC-03. Task 3 (the app, a separate crate) is its own commit (dbc35374).

## Known Stubs

None.

## TDD Gate Compliance

Tasks 1 and 2 carried `tdd="true"`. The new tests and their implementations land in the same `write_dispatch.rs` (tests are a co-located `#[cfg(test)] mod tests`), so RED and GREEN are a single `feat` commit rather than separate `test`/`feat` commits — there is no production path that could ship without the co-located tests. All `<behavior>` bullets are covered by named tests (see Verification).

## Threat Flags

None — no new network endpoint, auth path, file access, or schema surface was introduced. The change extends the existing audited, guarded write-dispatch path. The threat register's `mitigate` dispositions (T-231-04 transition-guard live re-eval, T-231-05 post-persist override timing, T-231-06 to_state source) are all implemented as specified.

## Self-Check: PASSED
</content>
</invoke>
