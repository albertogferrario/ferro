---
phase: 102
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 102 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none (workspace-level) |
| **Quick run command** | `cargo test -p ferro-json-ui -p ferro-theme -p ferro-cli` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui -p ferro-theme -p ferro-cli`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 102-01-01 | 01 | 1 | FND-01 | unit | `cargo test -p ferro-theme` | existing (update assertion) | ⬜ pending |
| 102-01-02 | 01 | 1 | FND-01 | unit | `cargo test -p ferro-cli` | existing (update assertion) | ⬜ pending |
| 102-01-03 | 01 | 1 | FND-01 | unit | `cargo test -p ferro-theme` | ❌ W0 | ⬜ pending |
| 102-01-04 | 01 | 1 | FND-02 | unit | `cargo test -p framework` | ❌ W0 | ⬜ pending |
| 102-01-05 | 01 | 1 | FND-03 | unit | `cargo test -p ferro-json-ui` | existing (update assertion) | ⬜ pending |
| 102-02-01 | 02 | 1 | FND-04 | unit | `cargo test -p ferro-json-ui` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] New test in `ferro-theme` asserting `TOKEN_FONT_SANS` value is `"--font-sans"`
- [ ] New test in `framework/src/json_ui/mod.rs` asserting Bunny Fonts `<link>` in rendered head
- [ ] Updated test in `ferro-cli/src/commands/make_theme.rs` asserting `--font-sans` (not `--font-family-sans`)
- [ ] New `has_class()` helper function in `ferro-json-ui/src/render.rs` test module

*Wave 0 tests are created as part of each plan's first task (test-first approach).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Inter font visually renders in browser | FND-03 | Font rendering is visual | Open any JSON-UI page in Chrome, inspect body computed style — font-family should show Inter |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
