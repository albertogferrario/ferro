---
phase: 231-statemachine-derived-executor-derivation-guard-re-eval-override-hook-sync-by-construction
verified: 2026-06-16T00:00:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: none
  previous_score: none
gaps: []
deferred:
  - truth: "Derived executor drives writes across MCP + visual/form surfaces; hand-written WriteDispatcher retired (EXEC-05)"
    addressed_in: "Phase 232"
    evidence: "REQUIREMENTS.md EXEC-05 marked [ ] Pending, mapped to Phase 232; PLAN 02 objective explicitly scopes EXEC-05 out of Phase 231"
---

# Phase 231: StateMachine-Derived Executor Verification Report

**Phase Goal:** A developer declares a state-transition write solely by naming a `StateMachine` transition on the `ActionDef` (`transition_trigger`), and `ferro-projections` derives the default executor (state read → guard re-eval → transition → persist) from the StateMachine declaration alone — with guard-fail rejection, a post-persist override hook for the app-specific 20%, and build/registration-time rejection of any reference to a transition the StateMachine does not declare. No hand-written `match` re-encodes transition facts.

**Verified:** 2026-06-16
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth (Success Criterion) | Status | Evidence |
| - | ------------------------- | ------ | -------- |
| 1 | A transition write declared ONLY by `ActionDef.transition_trigger` dispatches through the derived executor with NO hand-written `match` | VERIFIED | `grep -rn 'match action_name' app/src` → only hit is a doc-comment at `mcp_write_dispatch.rs:292` ("no `match action_name` anywhere"), no code match. App executor `app/src/controllers/mcp.rs:108-110` derives `new_status = ferro::derive_transition_plan(svc,&action_name).to_state`. End-to-end test `submit_persists_derived_to_state` (`mcp_write_dispatch.rs:299`) seeds order→"draft", drives `submit`, asserts DB persists derived `"submitted"` — PASS |
| 2 | The derived executor re-evaluates the transition guard live at execution time and REJECTS a guard-false transition (state unchanged) | VERIFIED | `dispatch_write` (write_dispatch.rs:338-348) iterates `merged_guards(preconditions, transition_guard)` through the live `GuardEvaluatorFn`; `ctx.evaluated_guards` is forbidden by comment only (never read). Test `guard_rejects_illegal_transition` passes a transition-only guard `is_manager` returning `Ok(false)`, executor is `panic!` if reached → asserts `Err(GuardFailed)`. `transition_guard_evaluated_at_call_time` proves the transition guard (absent from preconditions) is still evaluated. All PASS |
| 3 | A post-persist override side effect runs in addition to base dispatch without re-declaring the transition; no-override path stays declaration-only | VERIFIED | `OverrideFn` type (write_dispatch.rs:74), `WriteDispatcher.overrides` registry (:98), `with_override` builder (:112); hook invoked at step 4b (:398) AFTER executor, BEFORE idempotency store. Tests `override_hook_runs_post_persist`, `no_override_is_declaration_only`, `override_error_surfaces` all PASS. App `make_write_dispatcher` uses `WriteDispatcher::new(...)` with NO override → common path declaration-only |
| 4 | An `ActionDef`/override naming a transition the StateMachine does not declare is rejected at registration/boot (`validate()` returns Err), NOT runtime; NOT a trybuild fixture | VERIFIED | `ServiceDef::validate()` step 5 (service.rs:404-417) returns `Err(Validation)` for undeclared trigger; step 5b (:425-431) round-trips every triggering action through `derive_transition_plan` propagating `Err` via `?`. Tests `validate_rejects_undeclared_trigger`, `validate_rejects_ambiguous_fan_out_at_registration`, `validate_round_trips_derivation` PASS. `grep -rln 'trybuild\|tests/ui' ferro-projections` → empty (no compile-time fixture) |
| 5 | Derivation lives entirely in `ferro-projections` — no new crate, no parallel DSL; no async/sea-orm/tokio/`Box<dyn Fn>` in the derivation module | VERIFIED | `ferro-projections/src/executor.rs` is the sole derivation site (pure fn). `grep 'async fn\|Box<dyn Fn\|sea_orm\|tokio' executor.rs` → no match. `git diff ferro-projections/Cargo.toml` → empty (no new deps). No Cargo.toml changes across the phase's commits |

**Score:** 5/5 truths verified

### Deferred Items

| # | Item | Addressed In | Evidence |
| - | ---- | ------------ | -------- |
| 1 | Cross-surface wiring (MCP + visual/form), retire hand-written `WriteDispatcher` (EXEC-05) | Phase 232 | REQUIREMENTS.md EXEC-05 marked `[ ]` Pending → Phase 232; PLAN 02 objective scopes EXEC-05 out. `derive_transition_plan` appears only on the MCP surface (`app/src/...`), NOT in ferro-json-ui or ferro-inertia — visual/form correctly untouched this phase |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `ferro-projections/src/executor.rs` | `TransitionPlan` + pure `derive_transition_plan` + Error variants | VERIFIED | Exists (10216 B); `pub struct TransitionPlan` + `pub fn derive_transition_plan` present; 9 co-located tests pass |
| `ferro-projections/src/service.rs` | hardened `validate()` round-trips derivation | VERIFIED | step 5b at :425-431 calls `derive_transition_plan` with `?`; 4 validate tests pass |
| `ferro-projections/src/lib.rs` | re-export `derive_transition_plan`, `TransitionPlan` | VERIFIED | `mod executor;` + `pub use executor::{derive_transition_plan, TransitionPlan};` (:6,:17) |
| `framework/src/lib.rs` | facade re-export so app reaches via `ferro::` | VERIFIED | `derive_transition_plan` (:258), `TransitionPlan` (:261) in the `pub use ferro_projections::{...}` block |
| `ferro-mcp-server/src/write_dispatch.rs` | guard union/dedup + `OverrideFn` registry | VERIFIED | `merged_guards`, `OverrideFn`, `overrides` registry, post-persist seam all present; 6 new tests pass |
| `ferro-mcp-server/src/lib.rs` | `OverrideFn` re-export | VERIFIED | `pub use write_dispatch::{... OverrideFn, WriteDispatcher}` (:26) |
| `app/src/controllers/mcp.rs` | executor derives `to_state`, match deleted, facade-only | VERIFIED | `ferro::derive_transition_plan` (:108); no `ferro_projections::` (grep empty); no `match action_name` (grep empty) |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `app executor` | `ferro::derive_transition_plan` | `plan.to_state` replaces match | WIRED | mcp.rs:108-110; facade path resolves; end-to-end test persists derived value |
| `dispatch_write guard loop` | `TransitionPlan.guard ∪ preconditions` | `merged_guards` deduped, live evaluator | WIRED | write_dispatch.rs:338 + :505-506 derive transition_guard via `derive_transition_plan(...).ok()` |
| `dispatch_write override seam` | `OverrideFn` registry by action name | post-persist invocation step 4b | WIRED | write_dispatch.rs:398, between executor (:393) and idempotency store |
| `ServiceDef::validate` | derivation round-trip | fatal `Err` on undeclared/ambiguous | WIRED | service.rs:425-431 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| app executor | `new_status` | `derive_transition_plan(svc).to_state` → SeaORM `active.status = Set(new_status)` → DB | Yes — end-to-end test reads back `"submitted"` from DB | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Derivation + validate suite | `cargo test -p ferro-projections --lib` | 258 passed, 0 failed (incl. all `executor::tests::*` + `validate_*`) | PASS |
| Guard/override/dedup suite | `cargo test -p ferro-mcp-server --all-features` | 50 lib + 23 integration passed; all 6 new tests + regression `guard_denied_at_call_time` PASS | PASS |
| End-to-end derived submit | `cargo test -p app mcp_write_dispatch` | 4 passed incl. `submit_persists_derived_to_state` | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| EXEC-01 | 01, 02 | Derive executor from StateMachine alone, no hand-written match | SATISFIED | Truth 1 |
| EXEC-02 | 02 | Live server-side guard re-eval, reject guard-false transition | SATISFIED | Truth 2 |
| EXEC-03 | 02 | Post-persist override hook; common path declaration-only | SATISFIED | Truth 3 |
| EXEC-04 | 01 | Drift rejected at registration (`validate()` Err), not runtime | SATISFIED | Truth 4 |
| EXEC-05 | — | Cross-surface wiring, retire WriteDispatcher | DEFERRED | Phase 232 (Pending) — correctly out of scope |

### Anti-Patterns Found

None. `grep -rn 'TODO|FIXME|unimplemented!|todo!|placeholder|not yet implemented'` across the four modified source files returned no matches. Summaries report no known stubs; verified true.

### Human Verification Required

None. All five success criteria are provable programmatically via tests and grep, all of which were executed and passed.

### Gaps Summary

No gaps. All five ROADMAP success criteria are MET with concrete evidence:

1. **MET** — The `match action_name` duplication is deleted everywhere in `app/src` (only a doc-comment mentions it). The app executor derives `to_state` via the `ferro::` facade and an end-to-end test proves `submit` persists the derived `"submitted"`.
2. **MET** — `dispatch_write` re-evaluates the union of preconditions and transition guard live; a transition-only guard returning false rejects the write before the executor runs (executor panics if reached).
3. **MET** — A post-persist `OverrideFn` registry runs after the base persist; the no-override path is unchanged and used by the app.
4. **MET** — `validate()` returns `Err` for an undeclared trigger AND round-trips against the derivation, surfacing ambiguity at registration. No trybuild/`tests/ui` fixture — it is a runtime `validate()` Err path, as required.
5. **MET** — Derivation is a pure function in `ferro-projections/src/executor.rs` with zero new deps and no async/sea-orm/tokio/`Box<dyn Fn>`.

EXEC-05 (cross-surface wiring) is correctly NOT done — it is explicitly Phase 232, marked Pending in REQUIREMENTS.md, and `derive_transition_plan` appears only on the MCP surface.

---

_Verified: 2026-06-16_
_Verifier: Claude (gsd-verifier)_
