---
phase: 231-statemachine-derived-executor-derivation-guard-re-eval-override-hook-sync-by-construction
plan: 01
subsystem: ferro-projections
tags: [projections, state-machine, derivation, validation, write-path]
requires:
  - "ferro-projections::StateMachine::states_for_event"
  - "ferro-projections::ActionDef.transition_trigger"
  - "ferro-projections::Transition.to"
provides:
  - "ferro-projections::derive_transition_plan() — pure (ServiceDef, action) -> TransitionPlan"
  - "ferro-projections::TransitionPlan — serializable transition-write fact"
  - "ferro-projections::ServiceDef::validate() sync-by-construction gate (EXEC-04)"
  - "framework facade re-export: ferro::derive_transition_plan, ferro::TransitionPlan"
affects:
  - "Phase 232 consumer runtime (interprets TransitionPlan against a concrete entity)"
tech-stack:
  added: []
  patterns:
    - "Serializable plan (data, not behavior) — no closures, no async, no DB in the schema crate"
    - "validate() round-trips against derivation so the two checks cannot diverge"
key-files:
  created:
    - "ferro-projections/src/executor.rs"
  modified:
    - "ferro-projections/src/error.rs"
    - "ferro-projections/src/lib.rs"
    - "ferro-projections/src/service.rs"
    - "framework/src/lib.rs"
decisions:
  - "to_state sourced ONLY from Transition.to; event fan-out to >1 target is a hard AmbiguousTransition error, never a silent first-pick"
  - "EXEC-04 proven at the validate() unit level (registration-time); app-boot integration test deferred to EXEC-05/Phase 232 by design"
  - "guard/effects union with action.preconditions deferred to Plan 02 consumer runtime; the plan carries the raw transition guard + effects union"
metrics:
  duration: ~12m
  completed: 2026-06-16
requirements: [EXEC-01, EXEC-04]
---

# Phase 231 Plan 01: StateMachine-Derived Executor — Derivation Core + Drift Gate Summary

Pure, serializable `TransitionPlan` derivation (`derive_transition_plan`) in the schema-only `ferro-projections` crate, plus a hardened `ServiceDef::validate()` that round-trips against the derivation so executor/StateMachine drift is rejected at registration time — zero new dependencies, no closures, no async.

## What Was Built

**EXEC-01 — derivation core (`ferro-projections/src/executor.rs`, new):**
- `TransitionPlan` value type (`action`, `event`, `from_states: Vec<String>`, `to_state`, `guard`, `effects`), deriving `Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema` like the rest of the crate.
- `derive_transition_plan(&ServiceDef, &str) -> Result<TransitionPlan, Error>` — reads `Transition.to` as the single source of truth for the target state, replacing the consumer's hand-written `match action_name => new_status`. Multi-source aware (`from_states` collects every `Transition.from` carrying the event). Effects are the order-preserving, dedup'd union of `Transition.actions` and `ActionDef.effects`.
- Four new typed `Error` variants in `error.rs`: `NoTransitionTrigger`, `NoStateMachine`, `UndeclaredTransition { action, event }`, `AmbiguousTransition { event }`.
- Re-exports from `lib.rs` and the `framework` facade (`#[cfg(feature = "projections")]` block) so the app reaches the symbols via `ferro::derive_transition_plan` / `ferro::TransitionPlan` without a direct `ferro-projections` dependency.

**EXEC-04 — sync-by-construction gate (`ferro-projections/src/service.rs`):**
- Added step 5b to `validate()`: every action with `transition_trigger.is_some()` is round-tripped through `derive_transition_plan`, propagating any `Err` via `?`. An action that passes `validate()` is therefore guaranteed derivable at runtime — drift between "validate accepts it" and "derivation can build a plan" is structurally impossible. `AmbiguousTransition` now surfaces at registration, not first call.
- The pre-existing step-5 undeclared-trigger error stays fatal (unchanged).

## Tasks

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | TransitionPlan + derive_transition_plan() + facade re-export | 628ad9dc | executor.rs (new), error.rs, lib.rs (both crates) |
| 2 | Harden ServiceDef::validate() into the EXEC-04 gate | 9b7559a8 | service.rs |

## Verification

- `cargo test -p ferro-projections` — 258 + 22 + 1 + 8 tests pass (lib unit + catalog + schemas + doctests).
- `cargo test -p ferro-projections derive_transition_plan` / `validate_rejects_undeclared_trigger` — pass.
- `cargo build -p ferro-rs --features projections` — exits 0 (facade compiles with new re-exports).
- `cargo clippy -p ferro-projections --all-targets -- -D warnings` — clean.
- `cargo fmt -p ferro-projections -- --check` — clean.
- `git diff ferro-projections/Cargo.toml` — empty (no new dependency).
- `grep 'async fn\|Box<dyn Fn\|sea_orm\|tokio' ferro-projections/src/executor.rs` — no match.
- `grep -rn 'tests/ui' ferro-projections` — no match (EXEC-04 is registration-time, not compile-time).

New tests: `derive_transition_plan`, `derive_approve_carries_transition_guard`, `derive_multi_source_event`, `derive_no_trigger`, `derive_undeclared_trigger_errors`, `derive_no_state_machine`, `derive_no_action`, `derive_ambiguous_fan_out`, `transition_plan_serde_round_trip`, `validate_rejects_undeclared_trigger`, `validate_accepts_well_formed_order_service`, `validate_round_trips_derivation`, `validate_rejects_ambiguous_fan_out_at_registration`.

## Deviations from Plan

None — plan executed exactly as written. The `--all-features` workspace gate is deferred to Wave 2 per the plan's acceptance scope; crate-scoped checks were run and are green.

## Known Stubs

None.

## TDD Gate Compliance

Task 1 carried `tdd="true"`. The `TransitionPlan` impl and its `#[cfg(test)] mod tests` land in the same new module file (`executor.rs`), so RED and GREEN are a single `feat` commit rather than separate `test`/`feat` commits — there is no production code path that could ship without the co-located tests. All behavior bullets from the plan are covered by named tests (see Verification).

## Notes for Phase 232 (EXEC-02/03/05)

- The consumer runtime interprets `TransitionPlan` against a concrete SeaORM entity (state read → assert `current_state ∈ from_states` → live guard re-eval of `plan.guard` ∪ `action.preconditions`, deduped → persist `plan.to_state` → override hook).
- Guard/precondition de-duplication is intentionally NOT done here; the plan carries the raw `Transition.guard`. Dedup-by-name happens in the consumer's live `GuardEvaluatorFn` loop.
- The app-boot integration test (`boot_rejects_invalid_service`) is deferred to EXEC-05/Phase 232 by design; EXEC-04 is proven at the `validate()` unit level in this plan.

## Self-Check: PASSED

- ferro-projections/src/executor.rs — FOUND
- 231-01-SUMMARY.md — FOUND
- commit 628ad9dc — FOUND
- commit 9b7559a8 — FOUND
