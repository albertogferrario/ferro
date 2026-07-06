---
phase: 258
slug: mcp-surface-docs-publish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-06
---

# Phase 258 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) — none to install |
| **Quick run command** | `cargo test -p ferro-mcp` / `cargo test -p ferro-json-ui` (crate under edit) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick ~60s; full gate ~15-30 min (serialize — one CPU-heavy run at a time; check disk space first) |

---

## Sampling Rate

- **After every task commit:** Run the per-crate quick command for the crate touched
- **After every plan wave:** Run the full CI-exact gate (plus `cargo doc` `-D warnings` before the publish wave)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** one plan wave

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| (filled by planner) | | | POS-12 | — | catalog/generation_context additive only | unit + drift-guard | `cargo test -p ferro-mcp` | — | pending |
| (filled by planner) | | | POS-12 | — | docs build clean | build gate | `mdbook build docs` | — | pending |
| (filled by planner) | | | POS-13 | — | gate green before publish push | full gate + publish verify | CI-exact gate; crates.io API check | — | pending |

---

## SC → Proof Map (from RESEARCH.md Validation Architecture)

| SC | Proof |
|----|-------|
| SC-1 (catalog 52 + names) | `cargo test -p ferro-mcp test_all_components_present` — pre-satisfied, record evidence only |
| SC-2 (register guidance) | New drift-guard test: every component name / rule id / data attribute in the register guidance exists in BUILTIN registry / `design::rules()` / runtime attribute constants; `cargo test -p ferro-mcp` |
| SC-3 (docs) | `mdbook build docs` exits 0; grep each of the five component names has a `###` section + props table in components.md |
| SC-4 (gate + publish) | Full CI-exact gate exit 0; post-push: crates.io API shows ferro-rs 0.2.89 AND ferro-payments 0.1.6; gh API confirms master advanced |
