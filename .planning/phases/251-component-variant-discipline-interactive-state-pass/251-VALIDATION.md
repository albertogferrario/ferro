---
phase: 251
slug: component-variant-discipline-interactive-state-pass
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-03
---

# Phase 251 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via cargo (workspace) |
| **Config file** | Cargo.toml (workspace); no separate test config |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo test --all-features` (CI-exact; plus `cargo fmt --all -- --check` and `cargo clippy --all --all-targets --all-features -- -D warnings`) |
| **Estimated runtime** | ~60s crate-scoped; several minutes full suite |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui` (+ `cargo fmt --all -- --check`)
- **After every plan wave:** Run `cargo clippy --all --all-targets --all-features -- -D warnings` + `cargo test --all-features` (check `df` disk headroom first — known ENOSPC risk)
- **Before `/gsd-verify-work`:** Full CI-exact triple green + ferro-base.css regenerated + Chrome MCP visual pass
- **Max feedback latency:** ~120 seconds (crate-scoped runs); serialize CPU-intensive runs — never parallelize cargo invocations

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| (filled by planner) | | | DS-03 | — | old enum values rejected at spec-parse | unit | `cargo test -p ferro-json-ui component::` | ✅ existing modules updated in place | ⬜ pending |
| (filled by planner) | | | DS-03 | — | N/A | unit (new drift guard) | `cargo test -p ferro-json-ui catalog::tests::` | ❌ W0 | ⬜ pending |
| (filled by planner) | | | DS-03 | — | N/A | unit | `cargo test -p ferro-json-ui variant_enums_strum` | ✅ pattern at component.rs:1845 — extend | ⬜ pending |
| (filled by planner) | | | DS-03 | — | N/A | unit | `cargo test -p ferro-json-ui projection::` | ✅ existing tests updated | ⬜ pending |
| (filled by planner) | | | DS-04 | — | N/A | unit (render class-string assertions) | `cargo test -p ferro-json-ui render::` | ✅ existing (layout INT-07, form ring tests) — flip expected strings | ⬜ pending |
| (filled by planner) | | | DS-04 | — | N/A | structural grep | `grep -rn "duration-150\|duration-300\|focus-visible:ring-primary\|motion-reduce:transition-none" ferro-json-ui/src framework/src` → 0 hits | ❌ verification-step gap | ⬜ pending |
| (filled by planner) | | | DS-04 | — | N/A | smoke | `grep -c "ring-ring\|duration-fast" ferro-json-ui/assets/ferro-base.css` ≥ 1 | ✅ trivially checkable | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] D-19 schema-walking drift-guard test in `ferro-json-ui/src/catalog.rs` tests module (new; model: `builtin_types_count_drift_guard` at catalog.rs:1101)
- [ ] OQ-1 scope decision (action-level `variant` normalization) settled before the walker is written — determines ref-resolution boundary

*No framework install needed; all other coverage exists and is updated in place.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual parity + intended deltas, light + dark | DS-03/DS-04 | Rendered visual quality (hover/focus/disabled/motion feel) is not assertable as strings | Chrome MCP screenshots of the sample `app/` before/after, light + dark, per Phase 250 practice |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
