---
phase: 235
slug: ferro-payments-webhook-sync-dispatcher-integration-and-auto
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-17
---

# Phase 235 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | tokio (`#[tokio::test]`) + sea-orm in-memory SQLite + `MockStripeGateway` + `MemoryProcessedLog` (233/234 harness) |
| **Config file** | none — `#[cfg(test)]` blocks; `fresh_db()` helper |
| **Quick run command** | `cargo test -p ferro-payments` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~15s (`-p ferro-payments`) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-payments`
- **After every wave:** full gate (`fmt` + `clippy --all --all-targets -D warnings` + `test --all-features`)
- **Before `/gsd-verify-work`:** full gate green
- **Max feedback latency:** ~15s (quick)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 235-xx | tbd | 1 | PAY-POLY-WH (prereq) | — | `BillableKind::from_string` builds kind from DB string; refund-by-payment_intent primitive | unit | `cargo test -p ferro-payments` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-01 | — | `wire_dispatcher` returns SyncDispatcher with 3 handlers | unit | `cargo test -p ferro-payments handle_session` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-02 | — | session_completed: mark_paid + on_paid + payment_intent_id attached | unit | `cargo test -p ferro-payments handle_session_completed` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-02 | — | session_completed replay → no-op (ProcessedEventLog) | unit | `cargo test -p ferro-payments handle_session_completed_replay` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-02/06 | T-235 dup-honor | side-state conflict (paid-after-released) → auto-refund | unit | `cargo test -p ferro-payments handle_session_completed_side_state_conflict` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-03 | — | session_expired: mark_released + on_released | unit | `cargo test -p ferro-payments handle_session_expired` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-03 | — | session_expired already-released → no-op | unit | `cargo test -p ferro-payments handle_session_expired_noop` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-04 | — | charge_refunded: find_by_payment_intent + mark_refunded + on_refunded | unit | `cargo test -p ferro-payments handle_charge_refunded` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-04 | — | charge_refunded fallback: payment_intent_id None → find_by_charge_id resolves row | unit | `cargo test -p ferro-payments handle_charge_refunded_charge_id_fallback` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-05 | T-235 lost-money | loader None → auto-refund called exactly once | unit | `cargo test -p ferro-payments auto_refund_billable_vanished` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-05 | T-235 lost-money | loader Err → auto-refund called exactly once | unit | `cargo test -p ferro-payments auto_refund_loader_error` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-06 | T-235 double-honor | webhook+reaper interleaved → exactly one side-effect (guarded updates) | unit (race sim) | `cargo test -p ferro-payments webhook_reaper_race` | ❌ W0 | ⬜ pending |
| 235-xx | tbd | 2 | PAY-POLY-WH-06 | — | paid-after-released → mark_paid Ok(false) → auto-refund | unit | `cargo test -p ferro-payments paid_after_released` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky. Task IDs finalized by the planner.*

---

## Wave 0 Requirements

- [ ] `BillableKind` → `Cow<'static, str>` + `from_string` (`ferro-payments/src/lib.rs`) — D-10
- [ ] `ferro-stripe/src/refund.rs` — `create_for_payment_intent` + `ferro-stripe/src/lib.rs` re-export — D-08
- [ ] `ferro-payments/src/intent/lifecycle.rs` — `find_by_payment_intent`, `find_by_charge_id`, `attach_payment_intent`
- [ ] `ferro-payments/src/service.rs` — `StripeGateway::create_refund_for_payment_intent` + MockStripeGateway impl; `PaymentService::new` adds `processed_log` (cascades to 234 test call sites); `amount_cents <= 0` guard in start_checkout (WR-03)
- [ ] `ferro-payments/src/webhook.rs` (or service.rs section) — `wire_dispatcher` + `handle_session_completed`/`_expired`/`_charge_refunded` + auto-refund — WH-01..06
- [ ] `ferro-payments/src/lib.rs` — re-export `wire_dispatcher`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Refund-by-payment_intent against live Stripe | PAY-POLY-WH-05 | Live Stripe not unit-tested in 235 (236 integration bin) | Mock asserts call shape; live exercised in 236 |
| Stuck-refund recovery (WR-01/D-11) | — | ReconcileRefundsInFlight reaper is phase 236 | Documented recovery path; verified in 236 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s (quick)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
