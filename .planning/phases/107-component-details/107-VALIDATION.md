---
phase: 107
slug: component-details
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 107 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` via `cargo test` |
| **Config file** | none — cargo workspace |
| **Quick run command** | `cargo test -p ferro-json-ui 2>&1 \| tail -5` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui 2>&1 | tail -5`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 107-01-01 | 01 | 1 | CMP-01 | unit | `cargo test -p ferro-json-ui alert_svg_icon` | ❌ W0 | ⬜ pending |
| 107-01-02 | 01 | 1 | CMP-02 | unit | `cargo test -p ferro-json-ui skeleton_shimmer_class` | ❌ W0 | ⬜ pending |
| 107-01-03 | 01 | 1 | CMP-03 | unit | `cargo test -p ferro-json-ui breadcrumb_svg_separator` | ❌ W0 | ⬜ pending |
| 107-01-04 | 01 | 1 | CMP-04 | unit | `cargo test -p ferro-json-ui tab_active_font_semibold` | ❌ W0 | ⬜ pending |
| 107-01-05 | 01 | 1 | CMP-05 | unit | `cargo test -p ferro-json-ui notification_bell_svg` | ❌ W0 | ⬜ pending |
| 107-01-06 | 01 | 1 | CMP-06 | unit | `cargo test -p ferro-json-ui collapsible_svg_chevron` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/render.rs` — 6 new structural tests for CMP-01 through CMP-06 in `mod structural_tests`
- [ ] Update existing exact-string tests that break: `skeleton_default` (asserts `animate-pulse`), and any alert/collapsible tests asserting on emoji/entity content

*All tests live inline in render.rs following the project pattern.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Shimmer gradient renders correctly with theme tokens | CMP-02 | CSS custom property resolution requires browser rendering | Load a page with Skeleton component, verify gradient sweep animation |
| Bell SVG visual consistency with DashboardLayout | CMP-05 | Visual comparison across render paths | Compare notification bell in standalone Header vs DashboardLayout sidebar |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
