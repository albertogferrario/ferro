---
phase: 252
slug: design-module-lint-cli
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-03
---

# Phase 252 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | none — workspace Cargo.toml drives test discovery |
| **Quick run command** | `cargo test -p ferro-json-ui design` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~30s · full suite several minutes |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui design` (plus `cargo test -p ferro-cli design_lint` for CLI tasks)
- **After every plan wave:** Run `cargo test --all-features` (serialize — one CPU-intensive op at a time)
- **Before `/gsd-verify-work`:** Full CI-exact gate green: `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features`
- **Max feedback latency:** ~120 seconds for the quick loop

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 252-01-* | 01 | 1 | DS-05 | — | invalid `design` content never fails Spec parse | unit | `cargo test -p ferro-json-ui design` | ❌ W0 | ⬜ pending |
| 252-01-* | 01 | 1 | DS-05 | — | per-rule violating+conforming pairs (10 rules + inference) | unit | `cargo test -p ferro-json-ui design` | ❌ W0 | ⬜ pending |
| 252-02-* | 02 | 2 | DS-06 | T-252-01 | CLI path walk stays inside given root; non-spec JSON skipped | integration | `cargo test -p ferro-cli design_lint` | ❌ W0 | ⬜ pending |
| 252-02-* | 02 | 2 | DS-06 | — | `--deny` exit non-zero only on warning-level findings | integration | `cargo test -p ferro-cli design_lint` | ❌ W0 | ⬜ pending |
| 252-03-* | 03 | 2 | DS-05 | — | sample app views lint clean | integration | `cargo test -p app design_lint_clean` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

(Exact task IDs filled by the planner; rows above define the coverage contract.)

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — cargo test needs no install. Test files are created alongside implementation per plan (design module unit tests in `ferro-json-ui/src/design/`, CLI tests in `ferro-cli`, app gate test in `app`).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Human-readable CLI output formatting | DS-06 | Formatting quality is visual judgment | Run `ferro design:lint app/src/views` from repo root; confirm findings grouped by file, readable severity/rule/suggestion layout |
