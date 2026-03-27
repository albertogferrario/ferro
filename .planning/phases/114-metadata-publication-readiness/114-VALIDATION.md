---
phase: 114
slug: metadata-publication-readiness
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 114 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) |
| **Config file** | None — workspace Cargo.toml |
| **Quick run command** | `cargo clippy --all --all-targets -- -D warnings` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo clippy --all --all-targets -- -D warnings`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 114-01-01 | 01 | 1 | META-01 | manual-only | `grep -E "homepage\|readme\|categories" ferro-broadcast/Cargo.toml ferro-theme/Cargo.toml ferro-projections/Cargo.toml` | ✅ | ⬜ pending |
| 114-01-02 | 01 | 1 | META-02 | smoke | `cargo rustc --package ferro-rs --lib -- -W missing-docs 2>&1 \| grep "^warning:" \| wc -l` (must be 0) | ✅ | ⬜ pending |
| 114-02-01 | 02 | 1 | META-03 | manual-only | `wc -l ferro-json-ui/README.md ferro-lang/README.md ferro-whatsapp/README.md` (all > 9) | ✅ | ⬜ pending |
| 114-02-02 | 02 | 1 | META-04 | manual-only | `grep -c "^//!" ferro-json-ui/src/lib.rs ferro-lang/src/lib.rs` (both > 0) | ✅ Already satisfied | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. This phase has no test files to create; validation is compilation and line-count checks.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cargo.toml fields present | META-01 | Metadata presence, not runtime behavior | grep for homepage/readme/categories in target Cargo.toml files |
| README content beyond 9 lines | META-03 | Line count check, not testable in cargo test | `wc -l` on each README |
| Crate-level //! comments | META-04 | Already satisfied; verification only | `grep -c "^//!"` on lib.rs files |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
