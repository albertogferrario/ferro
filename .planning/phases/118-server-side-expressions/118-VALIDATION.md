---
phase: 118
slug: server-side-expressions
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-19
---

# Phase 118 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none — standard `#[cfg(test)]` blocks |
| **Quick run command** | `cargo test --package ferro-json-ui --all-features` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30-90 seconds (quick) / ~3-5 minutes (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --package ferro-json-ui --all-features`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green (fmt + clippy + test)
- **Max feedback latency:** ~90 seconds per task (quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 118-01-01 | 01 | 1 | EXPR-01 | — | `$data` resolves typed JSON at slash-path; missing → `Value::Null`; never panics | unit | `cargo test --package ferro-json-ui --all-features -- expression::` | ❌ W0 (new file) | ⬜ pending |
| 118-01-02 | 01 | 1 | EXPR-02 | — | `$template` interpolates `{/path}` placeholders; missing → `""`; escape sequences honored; never panics | unit | `cargo test --package ferro-json-ui --all-features -- expression::` | ❌ W0 (new file) | ⬜ pending |
| 118-01-03 | 01 | 1 | EXPR-01, EXPR-02 | — | Malformed expression objects (non-string value, sibling keys) pass through as literal JSON | unit | `cargo test --package ferro-json-ui --all-features -- expression::` | ❌ W0 (new file) | ⬜ pending |
| 118-01-04 | 01 | 1 | EXPR-01, EXPR-02 | — | Single-pass guarantee: expressions inside resolved `$data` output are NOT re-resolved; `Spec.data` never walked | unit | `cargo test --package ferro-json-ui --all-features -- expression::` | ❌ W0 (new file) | ⬜ pending |
| 118-01-05 | 01 | 1 | EXPR-01, EXPR-02 | — | Resolution scope: `Element.children`, `Element.action`, `Element.visible`, `Spec.title`, `Spec.layout` are NOT walked | unit | `cargo test --package ferro-json-ui --all-features -- expression::` | ❌ W0 (new file) | ⬜ pending |
| 118-02-01 | 02 | 2 | EXPR-03 | — | `JsonUi::render` resolves `$data`/`$template` before HTML emission; rendered markup contains concrete values, no expression markers | integration | `cargo test --package framework --all-features -- json_ui::` | ❌ W0 (additions to existing file) | ⬜ pending |
| 118-02-02 | 02 | 2 | EXPR-03 | — | `JsonUi::render_json` returns resolved spec with no `$data`/`$template` markers in output | integration | `cargo test --package framework --all-features -- json_ui::` | ❌ W0 (additions to existing file) | ⬜ pending |
| 118-02-03 | 02 | 2 | EXPR-03 | — | `JsonUi::render_with_errors` pipeline order: actions → expressions → errors applied against resolved props | integration | `cargo test --package framework --all-features -- json_ui::` | ❌ W0 (additions to existing file) | ⬜ pending |
| 118-02-04 | 02 | 2 | EXPR-03 | — | `JsonUi::render_with_config` honors expression resolution end-to-end | integration | `cargo test --package framework --all-features -- json_ui::` | ❌ W0 (additions to existing file) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/expression.rs` — new file containing `resolve_expressions`, private helpers, and inline `#[cfg(test)] mod tests` block covering EXPR-01 and EXPR-02
- [ ] New test block (or additions to existing test block) in `framework/src/json_ui/mod.rs` — covers EXPR-03 through every public `JsonUi::render*` path
- [ ] No new framework installation needed — `cargo test` is already operational; no new crate dependencies per CONTEXT.md D-11

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
