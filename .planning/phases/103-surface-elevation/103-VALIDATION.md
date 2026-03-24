---
phase: 103
slug: surface-elevation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 103 — Validation Strategy

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
| 103-01-01 | 01 | 1 | SRF-01 | unit | `cargo test -p ferro-json-ui card_renders_title` | existing — update assertion | ⬜ pending |
| 103-01-02 | 01 | 1 | SRF-01 | unit | `cargo test -p ferro-json-ui card_structural` | existing — no change needed | ⬜ pending |
| 103-01-03 | 01 | 1 | SRF-02 | unit | `cargo test -p ferro-json-ui modal_renders_details` | existing — no bg class assertion | ⬜ pending |
| 103-01-04 | 01 | 1 | SRF-03 | unit | `cargo test -p ferro-json-ui stat_card_renders_label` | existing — update assertion | ⬜ pending |
| 103-01-05 | 01 | 1 | SRF-04 | unit | `cargo test -p ferro-json-ui notification_dropdown` | existing — check bg assertion | ⬜ pending |
| 103-01-06 | 01 | 1 | SRF-05 | unit | `cargo test -p ferro-json-ui sidebar_renders` | existing — no change needed | ⬜ pending |
| 103-01-07 | 01 | 1 | SRF-06 | manual | OddContrast tool verification | manual-only | ⬜ pending |
| 103-01-08 | 01 | 1 | SRF-07 | unit | `cargo test -p ferro-json-ui runtime_js` | ❌ W0 — add new test | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] New test for `runtime.rs` VARIANT_CLASSES — assert FERRO_RUNTIME_JS contains `bg-primary` (not `bg-blue-500`)
- [ ] New test for tab switcher — assert FERRO_RUNTIME_JS contains `border-primary` (not `border-blue-600`)

*Existing cosmetic tests at lines 2873 and 3915 must be updated in the same commit as class changes — not new tests, just assertion string updates.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Dark mode 8 pairs >= 4.5:1 contrast | SRF-06 | No automated oklch WCAG test exists in Rust test harness | Verify each pair at oddcontrast.com with oklch values from default.css |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
