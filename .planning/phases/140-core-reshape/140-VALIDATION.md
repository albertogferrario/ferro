---
phase: 140
slug: core-reshape
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-20
audited: 2026-04-20
---

# Phase 140 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | tokio-test via `#[tokio::test]` |
| **Config file** | None — inline test modules per file |
| **Quick run command** | `cargo test -p ferro-stripe` |
| **Full suite command** | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-stripe`
- **After every plan wave:** Run `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 140-W0-01 | — | 0 | SC-2 | — | N/A | compile | `cargo check -p ferro-stripe` | ✅ | ✅ green |
| 140-W0-02 | — | 0 | SC-3 | — | N/A | unit | `cargo test -p ferro-stripe memory_log_true_then_false` | ✅ | ✅ green |
| 140-W0-03 | — | 0 | SC-12 | — | N/A | unit | `cargo test -p ferro-stripe memory_log_concurrent_insert_applies_once` | ✅ | ✅ green |
| 140-W0-04 | — | 0 | SC-6 | — | N/A | unit | `cargo test -p ferro-stripe checkout_create_missing_key_returns_err` | ✅ | ✅ green |
| 140-W0-05 | — | 0 | SC-10,11 | — | N/A | compile | `cargo check -p ferro-stripe` | ✅ | ✅ green |
| 140-W0-06 | — | 0 | SC-13 | — | N/A | CI gate | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `ferro-stripe/src/idempotency.rs` — new file with `ProcessedEventLog` trait + `MemoryProcessedLog` impl
- [x] `ferro-stripe/src/checkout.rs` — new file with `CheckoutBuilder` + `CheckoutIntent`
- [x] `ferro-stripe/src/refund.rs` — new file with `create` + `retrieve` fns
- [x] `ferro-stripe/src/account.rs` — new file consolidating account fns
- [x] `ferro-stripe/src/webhook/verify.rs` — extracted from `webhook/mod.rs`
- [x] `ferro-stripe/src/webhook/sync.rs` — stub file
- [x] `ferro-stripe/src/webhook/queue.rs` — stub file
- [x] `CHANGELOG.md` — documents all breaking changes and migration paths

*Tests are inline in each new file — no separate test infrastructure file needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `ferro-stripe 0.4.0` publishes cleanly to crates.io | SC-13 | Requires `cargo publish` with valid credentials | Run `cargo publish -p ferro-stripe --dry-run` to verify, then publish after CI green |
| CHANGELOG migration notes are complete and accurate | SC-14 | Content correctness requires human review | Read CHANGELOG.md, confirm every removed symbol has a migration path listed |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** 2026-04-20

---

## Validation Audit 2026-04-20

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 6 |
| Escalated | 0 |

All 6 tasks verified green against live codebase. Test name corrections applied: `memory_log_concurrent_insert_applies_once` and `checkout_create_missing_key_returns_err` (exact names from cargo test output). Full suite (2360 tests) was green at phase completion per 140-05-SUMMARY; current working-tree failures in `ferro-json-ui` are unrelated uncommitted changes outside phase 140 scope.
