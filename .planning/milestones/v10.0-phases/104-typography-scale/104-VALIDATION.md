---
phase: 104
slug: typography-scale
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 104 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none (workspace-level) |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 104-01-01 | 01 | 1 | TYP-01 | unit | `cargo test -p ferro-json-ui text_h1_variant` | existing — update assertion | ⬜ pending |
| 104-01-02 | 01 | 1 | TYP-02 | unit | `cargo test -p ferro-json-ui text_h2_variant` | existing — update assertion | ⬜ pending |
| 104-01-03 | 01 | 1 | TYP-02 | unit | `cargo test -p ferro-json-ui test_render_page_header_title_only` | existing — update assertion | ⬜ pending |
| 104-01-04 | 01 | 1 | TYP-03 | unit | `cargo test -p ferro-json-ui text_h3_variant` | existing — update assertion | ⬜ pending |
| 104-01-05 | 01 | 1 | TYP-03 | unit | `cargo test -p ferro-json-ui card_renders_title_and_description` | existing — update assertion | ⬜ pending |
| 104-01-06 | 01 | 1 | TYP-03 | unit | `cargo test -p ferro-json-ui modal_renders_details_summary` | existing — update assertion | ⬜ pending |
| 104-01-07 | 01 | 1 | TYP-04 | unit | `cargo test -p ferro-json-ui text_p_variant` | existing — update assertion | ⬜ pending |
| 104-01-08 | 01 | 1 | TYP-05 | unit | `cargo test -p ferro-json-ui` (layout tests) | existing — check/update | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test files needed. Cosmetic test assertions need updating in the same commit as class changes, but the test functions already exist.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
