---
phase: 258
slug: mcp-surface-docs-publish
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-06
updated: 2026-07-06
---

# Phase 258 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + mdBook build |
| **Config file** | Cargo.toml (workspace) / docs/book.toml — none to install (mdbook may need `cargo install mdbook --locked`) |
| **Quick run command** | `cargo test -p ferro-mcp` (Plan 01) / `mdbook build docs` (Plan 02) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features && cargo doc --no-deps --all-features -D warnings` |
| **Estimated runtime** | quick ~60s; full gate ~15-30 min (serialize — one CPU-heavy run at a time; check disk space first) |

---

## Sampling Rate

- **After every task commit:** Run the per-crate quick command for the crate/file touched
- **After every plan wave:** Run the full CI-exact gate (plus `cargo doc -D warnings` before the publish wave)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** one plan wave

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 258-01 T1 | 01 | 1 | POS-12 | T-258-02 | catalog BUILDER_API + RULE_COMPONENTS additive only; SC-1 recorded pre-satisfied | unit + drift-guard | `cargo test -p ferro-mcp -- test_all_components_present builder_api_mentions_fill_viewport_and_each design_system_component_guidance_drift_guarded` | YES (json_ui_catalog.rs); new builder-api test added in-task | covered |
| 258-01 T2 | 01 | 1 | POS-12 | T-258-01 | generation_context register guidance drift-guarded against registries + FERRO_RUNTIME_JS | unit + drift-guard | `cargo test -p ferro-mcp -- test_generation_context_has_all_sections register_composition_drift_guard` | new drift guard added in-task | covered |
| 258-02 T1 | 02 | 1 | POS-12 | T-258-05 | five component sections match verified props ground truth; neutral voice | build + grep | `grep -c "^### \(TileGrid\|SelectionPanel\|FilterTabs\|QuantityStepper\|Numpad\)$" docs/src/json-ui/components.md` (==5) | components.md exists | covered |
| 258-02 T2 | 02 | 1 | POS-12 | T-258-04/T-258-06 | register projection docs; double-submit pointer accurate; no new SUMMARY pages | build gate | `mdbook build docs` | layouts.md / spec-construction.md exist | covered |
| 258-03 T1 | 03 | 2 | POS-13 | T-258-07/T-258-11 | full CI-exact gate green before push; /cassa flip verified standing; specific files staged | full gate | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features && cargo doc --no-deps --all-features -D warnings` | — | covered |
| 258-03 T2 | 03 | 2 | POS-13 | T-258-08 | operator-gated pre-publish checklist (versions + staged files) | checkpoint | (human-verify — blocking) | — | human-verified |
| 258-03 T3 | 03 | 2 | POS-13 | T-258-08/T-258-09 | dual-crate publish verified via API; ff-only master; brief-only handoff | external verify | `curl -s https://crates.io/api/v1/crates/ferro-rs \| jq -r .crate.max_version` (==0.2.89) + ferro-payments (==0.1.6) + `gh api …/releases/latest --jq .tag_name` (==v0.2.89) | — | covered |

---

## SC → Proof Map (from RESEARCH.md Validation Architecture)

| SC | Proof | Plan/Task |
|----|-------|-----------|
| SC-1 (catalog 52 + names) | `cargo test -p ferro-mcp test_all_components_present` — pre-satisfied, record evidence only | 258-01 T1 |
| SC-2 (register guidance) | `register_composition_drift_guard`: every component name / rule id / data attribute in the register guidance exists in BUILTIN registry / `design::rules()` / `FERRO_RUNTIME_JS`; `cargo test -p ferro-mcp` | 258-01 T2 |
| SC-3 (docs) | `mdbook build docs` exits 0; grep each of the five component names has a `###` section + props table in components.md | 258-02 T1/T2 |
| SC-4 (gate + publish) | Full CI-exact gate exit 0; post-push: crates.io API shows ferro-rs 0.2.89 AND ferro-payments 0.1.6; gh API confirms release tag v0.2.89 | 258-03 T1/T3 |

---

## Validation Audit 2026-07-06

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

All 7 per-task rows verified covered (6 automated + 1 human checkpoint). Evidence:

| Task | Evidence |
|------|----------|
| 258-01 T1 | `builder_api_mentions_fill_viewport_and_each` (json_ui_catalog.rs:595) + drift guards; `cargo test -p ferro-mcp` green post-review-fix (313 lib tests + integration suites, 258-REVIEW-FIX.md) |
| 258-01 T2 | `register_composition_drift_guard` (generation_context.rs:558) green with WR-02 hardened checks — all 13 `REGISTER_DATA_ATTRIBUTES` asserted against `FERRO_RUNTIME_JS` |
| 258-02 T1 | grep re-run this audit: 5 `###` component sections in components.md |
| 258-02 T2 | `mdbook build docs` exits 0 (258-VERIFICATION.md re-run + after each REVIEW-FIX docs edit) |
| 258-03 T1 | CI Publish run 28808914072 Test job green (13m1s) before push |
| 258-03 T2 | Operator checkpoint approved 2026-07-06 (recorded in 258-03-SUMMARY.md) — manual by design |
| 258-03 T3 | Re-verified this audit: crates.io ferro-rs 0.2.89 + ferro-payments 0.1.6; `releases/latest` == v0.2.89 (release.yml manual dispatch completed since summary); tag `refs/tags/v0.2.89` on remote |
