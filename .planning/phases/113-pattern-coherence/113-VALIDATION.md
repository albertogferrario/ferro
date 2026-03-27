---
phase: 113
slug: pattern-coherence
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 113 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + grep verification |
| **Config file** | Cargo.toml workspace |
| **Quick run command** | `cargo build --all-features` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo build --all-features`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 113-01-01 | 01 | 1 | COH-01 | grep | `grep -rn "use ferro::\*" docs/src/` | N/A | ⬜ pending |
| 113-01-02 | 01 | 1 | COH-01 | grep | `grep -rn "use ferro::.*::" docs/src/` | N/A | ⬜ pending |
| 113-01-03 | 01 | 1 | COH-02 | grep | `grep -B2 "pub async fn" docs/src/ -r` | N/A | ⬜ pending |
| 113-01-04 | 01 | 1 | COH-03 | grep | `grep -rn "\.unwrap()" docs/src/` | N/A | ⬜ pending |
| 113-02-01 | 02 | 1 | COH-04 | compilation | `cargo build --all-features` | ✅ | ⬜ pending |
| 113-02-02 | 02 | 1 | COH-04 | grep | `grep -rn "COMPONENT_CATALOG" ferro-cli/src/ ferro-mcp/src/` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Import style consistency | COH-01 | Docs have no runnable tests | Grep for glob/sub-module imports, verify zero results |
| Handler macro presence | COH-02 | Docs have no runnable tests | Grep for handler fns without `#[handler]`, verify zero results |
| No unwrap in examples | COH-03 | Docs have no runnable tests | Grep for `.unwrap()` in docs, verify zero results |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
