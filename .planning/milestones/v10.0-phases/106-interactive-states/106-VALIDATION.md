---
phase: 106
slug: interactive-states
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 106 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` via `cargo test` |
| **Config file** | none — cargo workspace |
| **Quick run command** | `cargo test -p ferro-json-ui 2>&1 \| tail -5` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui 2>&1 | tail -5`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 106-01-01 | 01 | 0 | INT-01..07 | unit | `cargo test -p ferro-json-ui` | ❌ W0 | ⬜ pending |
| 106-01-02 | 01 | 1 | INT-01 | unit | `cargo test -p ferro-json-ui button_focus_ring` | ❌ W0 | ⬜ pending |
| 106-01-03 | 01 | 1 | INT-02 | unit | `cargo test -p ferro-json-ui tabs_focus_ring` | ❌ W0 | ⬜ pending |
| 106-01-04 | 01 | 1 | INT-03 | unit | `cargo test -p ferro-json-ui pagination_focus_ring` | ❌ W0 | ⬜ pending |
| 106-01-05 | 01 | 1 | INT-04 | unit | `cargo test -p ferro-json-ui breadcrumb_focus_ring` | ❌ W0 | ⬜ pending |
| 106-01-06 | 01 | 1 | INT-05 | unit | `cargo test -p ferro-json-ui sidebar_nav_focus_ring` | ❌ W0 | ⬜ pending |
| 106-01-07 | 01 | 1 | INT-06 | unit | `cargo test -p ferro-json-ui table_row_hover` | ❌ W0 | ⬜ pending |
| 106-01-08 | 01 | 1 | INT-07 | unit | covered by INT-01..05 tests | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/render.rs` test block — 7 new structural tests (INT-01 through INT-07) using `has_class()` helper, placed in existing `mod structural_tests`

*Existing infrastructure covers all phase requirements — no new test files or frameworks needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Focus ring visible only on keyboard tab, not mouse click | INT success criteria #4 | Browser-specific `focus-visible:` behavior | Tab through page in Chrome, verify ring appears; click same elements, verify ring does not appear |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
