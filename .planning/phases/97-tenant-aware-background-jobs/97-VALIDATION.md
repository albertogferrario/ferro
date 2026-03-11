---
phase: 97
slug: tenant-aware-background-jobs
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 97 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` |
| **Config file** | none — `cargo test --all-features` |
| **Quick run command** | `cargo test -p ferro-queue --all-features` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-queue --all-features`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 97-01-01 | 01 | 1 | JobPayload tenant_id field | unit | `cargo test -p ferro-queue job::tests` | ❌ W0 | ⬜ pending |
| 97-01-02 | 01 | 1 | Old payload backward compat | unit | `cargo test -p ferro-queue job::tests` | ❌ W0 | ⬜ pending |
| 97-01-03 | 01 | 1 | TenantScopeProvider trait | unit | `cargo test -p ferro-queue worker::tests` | ❌ W0 | ⬜ pending |
| 97-02-01 | 02 | 1 | OnceLock capture hook | unit | `cargo test -p ferro-queue dispatcher::tests` | ❌ W0 | ⬜ pending |
| 97-02-02 | 02 | 1 | PendingDispatch::for_tenant | unit | `cargo test -p ferro-queue dispatcher::tests` | ❌ W0 | ⬜ pending |
| 97-02-03 | 02 | 1 | Auto-capture at dispatch | unit | `cargo test -p ferro-queue dispatcher::tests` | ❌ W0 | ⬜ pending |
| 97-03-01 | 03 | 2 | Worker::with_tenant_lookup | unit | `cargo test -p ferro-queue worker::tests` | ❌ W0 | ⬜ pending |
| 97-03-02 | 03 | 2 | process_job scope wrapping | unit | `cargo test -p ferro-queue worker::tests` | ❌ W0 | ⬜ pending |
| 97-03-03 | 03 | 2 | Worker::clone preserves scope | unit | `cargo test -p ferro-queue worker::tests` | ❌ W0 | ⬜ pending |
| 97-03-04 | 03 | 2 | TenantNotFound → job fails | unit | `cargo test -p ferro-queue worker::tests` | ❌ W0 | ⬜ pending |
| 97-03-05 | 03 | 2 | No provider → runs without scope | unit | `cargo test -p ferro-queue worker::tests` | ❌ W0 | ⬜ pending |
| 97-04-01 | 04 | 2 | Framework hook registration | unit | `cargo test --all-features` | ❌ W0 | ⬜ pending |
| 97-04-02 | 04 | 2 | Tracing span includes tenant_id | unit | `cargo test -p ferro-queue worker::tests` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] New test cases in `ferro-queue/src/job.rs` — covers tenant_id serialization / backward compat
- [ ] New test cases in `ferro-queue/src/dispatcher.rs` — covers hook registration + for_tenant + auto-capture
- [ ] New test cases in `ferro-queue/src/worker.rs` — covers TenantScopeProvider injection + scope wrapping + clone

*Existing test infrastructure covers framework-level integration.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Stripe webhook → job → tenant context | End-to-end | Requires running Redis + Stripe webhook | 1. Trigger webhook 2. Check job handler sees correct tenant |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
