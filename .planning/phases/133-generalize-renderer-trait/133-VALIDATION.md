---
phase: 133
slug: generalize-renderer-trait
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-14
---

# Phase 133 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | `ferro-projections/Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-projections --all-features` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-projections --all-features`
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 133-01-01 | 01 | 1 | D-01..D-09 | unit | `cargo test -p ferro-projections --all-features` | ✅ existing | ⬜ pending |
| 133-01-02 | 01 | 1 | D-10..D-11 | compile | `cargo check -p ferro-projections` | ✅ existing | ⬜ pending |
| 133-01-03 | 01 | 1 | D-12..D-14 | integration | `cargo test --all-features` | ✅ existing | ⬜ pending |

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
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
