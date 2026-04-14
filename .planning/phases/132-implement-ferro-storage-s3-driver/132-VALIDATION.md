---
phase: 132
slug: implement-ferro-storage-s3-driver
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-14
---

# Phase 132 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | `ferro-storage/Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-storage --features s3` |
| **Full suite command** | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-storage --features s3`
- **After every plan wave:** Run `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 132-01-01 | 01 | 1 | D-01..D-03 | unit | `cargo test -p ferro-storage --features s3 -- s3_client` | ❌ W0 | ⬜ pending |
| 132-01-02 | 01 | 1 | D-04..D-05 | unit | `cargo test -p ferro-storage --features s3 -- s3_url` | ❌ W0 | ⬜ pending |
| 132-01-03 | 01 | 1 | D-06..D-12 | unit | `cargo test -p ferro-storage --features s3 -- s3_ops` | ❌ W0 | ⬜ pending |
| 132-02-01 | 02 | 2 | D-14 | integration | `cargo test -p ferro-storage --features s3,s3-tests` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-storage/src/drivers/s3.rs` — complete S3Driver implementation (replaces stubs)
- [ ] `ferro-storage/tests/s3_integration.rs` — integration tests behind `s3-tests` feature

*Existing test infrastructure (cargo test) covers all tooling needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| DO Spaces upload/download | D-14 field test | Requires real S3-compatible bucket with credentials | Set `AWS_*` env vars pointing to DO Spaces bucket, run `cargo test -p ferro-storage --features s3,s3-tests` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
