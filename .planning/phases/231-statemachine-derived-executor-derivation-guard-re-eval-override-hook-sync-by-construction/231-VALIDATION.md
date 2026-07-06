---
phase: 231
slug: statemachine-derived-executor-derivation-guard-re-eval-override-hook-sync-by-construction
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-16
---

# Phase 231 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Detailed per-EXEC validation architecture (test files, commands, fixtures) lives in
> `231-RESEARCH.md` § "Validation Architecture" — this file is the sampling contract.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` (+ `trybuild` exists but is N/A here — see EXEC-04) |
| **Config file** | workspace `Cargo.toml` (`profile.test` already disk-tuned) |
| **Quick run command** | `cargo test -p ferro-projections` (the pure derivation; fast, no DB) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~quick <30s; full gate minutes (watch disk — see `project_ferro_disk_full_test_gate`) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p <touched crate>` (e.g. `ferro-projections` for derivation; the write-dispatch crate for guard/override/exec).
- **After every plan wave:** `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
- **Before `/gsd-verify-work`:** full gate green (fmt + clippy + test).
- **Max feedback latency:** quick path < 30s.

---

## Per-Requirement Verification Map

| Requirement | Secure Behavior | Test Type | Automated Command | Status |
|-------------|-----------------|-----------|-------------------|--------|
| EXEC-01 | A `transition_trigger`-only action derives a `TransitionPlan` (`from→to`) from `Transition.to`; no `match action_name` remains in the write path | unit (pure) | `cargo test -p ferro-projections derive_transition_plan` | ⬜ pending |
| EXEC-02 | The runtime re-evaluates the plan's guard at execution; a guard-false transition is refused and state is unchanged (not advisory list-time) | integration | `cargo test -p <write-dispatch crate> guard_rejects_illegal_transition` | ⬜ pending |
| EXEC-03 | A post-persist override side effect runs in addition to the base dispatch; the no-override path stays declaration-only | integration | `cargo test -p <write-dispatch crate> override_hook_runs_post_persist` | ⬜ pending |
| EXEC-04 | An `ActionDef.transition_trigger` naming a transition the `StateMachine` does not declare fails `ServiceDef::validate()` at registration/boot — not at runtime | unit | `cargo test -p ferro-projections validate_rejects_undeclared_trigger` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Synthetic state-machine fixture in `ferro-projections` tests (a `ServiceDef` with a small `StateMachine` + `ActionDef` transition triggers) for the pure derivation + `validate()` tests (EXEC-01, EXEC-04).
- [ ] An end-to-end transition test on the write-dispatch path exercising guard re-eval (EXEC-02) and the override hook (EXEC-03) against the existing synthetic order/approval anchor.

*Reuse the COMP-05 `approval_workflow` / `order` anchors where they already exist rather than authoring new app models.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| "No hand-written `match` remains" is partly a deletion proof | EXEC-01 | A grep-absence assertion is automatable but the *semantic* equivalence (derived `to` == old hardcoded `to`) is best eyeballed once | After EXEC-01, `grep -rn 'match action_name' app/src` returns nothing; diff the old hardcoded targets against derived `Transition.to` |
