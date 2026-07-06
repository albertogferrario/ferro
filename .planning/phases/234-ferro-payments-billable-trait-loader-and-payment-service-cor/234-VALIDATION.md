---
phase: 234
slug: ferro-payments-billable-trait-loader-and-payment-service-cor
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-17
---

# Phase 234 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | tokio (`#[tokio::test]`) + sea-orm in-memory SQLite + in-crate `MockStripeGateway` / `MockBillableLoader` |
| **Config file** | none — `#[cfg(test)]` blocks in source files (mirrors 233 `lifecycle.rs` harness) |
| **Quick run command** | `cargo test -p ferro-payments` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~15 seconds (`-p ferro-payments`); full suite longer |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-payments`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full gate green — `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Max feedback latency:** ~15 seconds (quick), full gate before phase close

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 234-xx | tbd | 1 | PAY-POLY-SVC-01 | — | `Billable` async side effects take `&DatabaseTransaction` (no autocommit leak) | unit (compile) | `cargo test -p ferro-payments` | ❌ W0 | ⬜ pending |
| 234-xx | tbd | 1 | PAY-POLY-SVC-02 | — | `BillableLoader::load` object-safe, returns `Box<dyn Billable>` | unit | `cargo test -p ferro-payments` | ❌ W0 | ⬜ pending |
| 234-xx | tbd | 2 | PAY-POLY-SVC-03a | — | `start_checkout` inserts reserved row, attaches session_id, snapshots `application_fee_cents` (Connect) | unit (SQLite+mock) | `cargo test -p ferro-payments -- start_checkout` | ❌ W0 | ⬜ pending |
| 234-xx | tbd | 2 | PAY-POLY-SVC-03b | — | non-Connect billable → no fee snapshot | unit | `cargo test -p ferro-payments -- start_checkout_no_connect` | ❌ W0 | ⬜ pending |
| 234-xx | tbd | 2 | PAY-POLY-SVC-03c | — | `request_refund` (paid + charge_id) snapshots `refund_amount_cents`, calls Stripe | unit | `cargo test -p ferro-payments -- request_refund` | ❌ W0 | ⬜ pending |
| 234-xx | tbd | 2 | PAY-POLY-SVC-03d | — | non-paid / missing charge_id → `StatusPrecondition`, Stripe NOT called | unit | `cargo test -p ferro-payments -- request_refund_precondition` | ❌ W0 | ⬜ pending |
| 234-xx | tbd | 2 | PAY-POLY-SVC-03e | — | dedup: 2nd concurrent refund no-ops, Stripe called exactly once | unit | `cargo test -p ferro-payments -- request_refund_dedup` | ❌ W0 | ⬜ pending |
| 234-xx | tbd | 2 | PAY-POLY-SVC-04 | — | `MockStripeGateway` records calls; tests assert counts | unit | `cargo test -p ferro-payments` | ❌ W0 | ⬜ pending |
| 234-xx | tbd | 1 | PAY-POLY-SVC-05 | — | `PaymentError::Stripe(#[from])` + `Loader` + `AutoRefundTriggered` compile | unit (compile) | `cargo test -p ferro-payments` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky. Task IDs finalized by the planner.*

---

## Wave 0 Requirements

- [ ] `ferro-payments/src/billable.rs` — `Billable` trait + `connect_account_id` default (PAY-POLY-SVC-01)
- [ ] `ferro-payments/src/loader.rs` — `BillableLoader` trait (PAY-POLY-SVC-02)
- [ ] `ferro-payments/src/service.rs` — `PaymentService` + `StripeGateway` + `MockStripeGateway` + tests (PAY-POLY-SVC-03/04)
- [ ] `ferro-payments/src/intent/lifecycle.rs` — `attach_session` fn (guarded by `StripeSessionId.is_null()`) (PAY-POLY-SVC-03a)
- [ ] `ferro-payments/src/error.rs` — extended `PaymentError` + `AutoRefundReason` (PAY-POLY-SVC-05)
- [ ] `ferro-payments/Cargo.toml` — `ferro-stripe` dependency (D-19)
- [ ] `.github/workflows/publish.yml` — Wave 1c step, drop ferro-payments from 1b (D-21)
- [ ] `ferro-payments/src/lib.rs` — new re-exports

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| publish.yml Wave 1c ordering | D-21 | Real publish ordering only observable on a crates.io release run (236) | Review YAML diff; confirm 1c step runs after 1b index-wait and `ferro-payments` removed from `WAVE1B_CRATES` |
| Postgres/MySQL Connect fee path | PAY-POLY-SVC-03a | Live Stripe + multi-backend not unit-tested in 234 (236 integration bin) | Correct-by-construction + code review |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s (quick)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
