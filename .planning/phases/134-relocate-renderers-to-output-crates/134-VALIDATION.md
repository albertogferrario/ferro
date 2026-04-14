---
phase: 134
slug: relocate-renderers-to-output-crates
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-15
---

# Phase 134 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test --all-features -p ferro-projections -p ferro-json-ui` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --all-features -p ferro-projections -p ferro-json-ui`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 134-01-01 | 01 | 1 | N/A | compilation | `cargo check -p ferro-json-ui --features projections` | ✅ | ⬜ pending |
| 134-01-02 | 01 | 1 | N/A | unit | `cargo test -p ferro-json-ui --features projections` | ✅ | ⬜ pending |
| 134-02-01 | 02 | 2 | N/A | compilation | `cargo check -p ferro-projections` | ✅ | ⬜ pending |
| 134-02-02 | 02 | 2 | N/A | unit | `cargo test -p ferro-projections` | ✅ | ⬜ pending |
| 134-03-01 | 03 | 2 | N/A | compilation | `cargo check -p ferro-mcp` | ✅ | ⬜ pending |
| 134-03-02 | 03 | 2 | N/A | full | `cargo test --all-features` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
