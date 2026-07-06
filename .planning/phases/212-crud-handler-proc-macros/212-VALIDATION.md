---
phase: 212
slug: crud-handler-proc-macros
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-13
---

# Phase 212 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | trybuild 1.x (already a dev-dep in ferro-macros) + Rust built-in `#[test]` |
| **Config file** | none — trybuild uses ferro-macros `tests/` dev-deps |
| **Quick run command** | `cargo test -p ferro-macros --test resource_macro` |
| **Full suite command** | `cargo test --all-features -p ferro-macros && cargo test --all-features -p ferro-rs validation` |
| **Estimated runtime** | ~60–120 s (trybuild compiles fixtures) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-macros --test resource_macro` (or the validator unit tests for the `validate_or_redirect` task)
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** Full suite green; `cargo expand` of the pass fixtures shows the named inner fn with typed params
- **Max feedback latency:** ~120 seconds (trybuild compile cycle)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 212-01-xx | 01 | 1 | CRUD-03 | — | validate_or_redirect flashes + redirects, no envelope leak | unit | `cargo test -p ferro-rs validation::validator` | ❌ W0 | ⬜ pending |
| 212-01-xx | 01 | 1 | CRUD-04 | — | `TenantScoped` lookup scoped by tenant_id | compile-pass + unit | `cargo test -p ferro-macros --test resource_macro` | ❌ W0 | ⬜ pending |
| 212-02-xx | 02 | 2 | CRUD-01 | T-212-01 | resource_get binds typed params; 404-on-miss | compile-pass | `cargo test -p ferro-macros --test resource_macro` | ❌ W0 | ⬜ pending |
| 212-02-xx | 02 | 2 | CRUD-02 | T-212-01 | resource_post prelude + validation redirect | compile-pass | `cargo test -p ferro-macros --test resource_macro` | ❌ W0 | ⬜ pending |
| 212-02-xx | 02 | 2 | CRUD-05 | — | cargo-expand shows typed inner fn params | manual+compile | `cargo expand --test resource_macro` | ❌ W0 | ⬜ pending |
| 212-03-xx | 03 | 3 | CRUD-06 | — | macros importable as `ferro::resource_get`; fixture app compiles | compile-pass | `cargo test -p ferro-macros --test resource_macro` (pass fixture) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Task IDs are placeholders — the planner assigns final IDs; map each CRUD-* to the task that satisfies it.*

---

## Wave 0 Requirements

- [ ] `ferro-macros/tests/resource_macro.rs` — trybuild harness (mirrors the existing `action_macro.rs`/`handler` test harness)
- [ ] `ferro-macros/tests/ui/resource/pass/minimal_get.rs` + `minimal_post.rs` — compile-pass fixtures
- [ ] `ferro-macros/tests/ui/resource/fail/{resource_post_missing_redirect_to,resource_get_unknown_placeholder,resource_get_not_async}.rs` + `.stderr` snapshots
- [ ] `framework/src/validation/validator.rs` — `validate_or_redirect` unit tests added to the existing `mod tests`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| IDE autocomplete / jump-to-def on `tenant`/`resource` params | CRUD-05 | rust-analyzer behavior isn't unit-testable | After build, `cargo expand --test resource_macro` confirms the user body lives in a named inner fn with the real typed params (the structural proxy for IDE-friendliness); spot-check in an editor optional |

*All other phase behaviors have automated (trybuild/unit) verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
