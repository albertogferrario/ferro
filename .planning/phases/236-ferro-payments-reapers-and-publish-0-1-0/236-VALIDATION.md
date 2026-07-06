---
phase: 236
slug: ferro-payments-reapers-and-publish-0-1-0
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-17
---

# Phase 236 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` (in-memory SQLite + `MockStripeGateway` harness from 233/234/235) |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-payments` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~quick: <30s for `-p ferro-payments`; full gate slower (disk-watch per memory) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-payments` (plus `cargo clippy -p ferro-payments --all-targets -- -D warnings` on code tasks)
- **After every plan wave:** Run `cargo test -p ferro-payments` (and `-p ferro-stripe` for the poll-primitive wave)
- **Before publish task:** Full `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` AND `cargo doc -Dwarnings` (CI Docs gate)
- **Max feedback latency:** ~30s for the targeted crate suite

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 236-01-xx | 01 | 0 | PAY-POLY-REAP-01 | T-236-01 | `find_expired` selects only `reserved AND expires_at<now`; `find_refunds_in_flight` only `paid AND refund_amount_cents IS NOT NULL AND refunded_at IS NULL` | unit | `cargo test -p ferro-payments lifecycle` | ❌ W0 | ⬜ pending |
| 236-02-xx | 02 | 1 | PAY-POLY-REAP-01 | T-236-02 | `release_expired` per-intent txn; `mark_released==false` race no-op; one row failure does not abort batch; returns count | unit (injected clock) | `cargo test -p ferro-payments release_expired` | ❌ W0 | ⬜ pending |
| 236-03-xx | 03 | 1 | PAY-POLY-REAP-02 | T-236-03 | ferro-stripe `list_for_payment_intent` + `StripeGateway::refund_status`; `RefundStatus` maps succeeded/pending/failed | unit (mock) | `cargo test -p ferro-payments reconcile && cargo test -p ferro-stripe refund` | ❌ W0 | ⬜ pending |
| 236-04-xx | 03 | 1 | PAY-POLY-REAP-02 | T-236-04 | `reconcile_refunds_in_flight`: succeeded→`mark_refunded`+`on_refunded` (sets refunded_at); pending→skip; failed→warn, NO auto-retry (double-refund guard) | unit (injected clock + mock) | `cargo test -p ferro-payments reconcile` | ❌ W0 | ⬜ pending |
| 236-05-xx | 04 | 2 | PAY-POLY-REAP-03 | T-236-05 | `ReleaseExpiredPaymentIntents<L>` / `ReconcileRefundsInFlight<L>` Job structs: serde-skip Arc handle, `handle()`→`JobFailed`, re-injection error path | unit | `cargo test -p ferro-payments reaper` | ❌ W0 | ⬜ pending |
| 236-06-xx | 05 | 2 | PAY-POLY-REAP-04 | T-236-06 | gated end-to-end skips cleanly when `STRIPE_TEST_SECRET_KEY` absent; tiny example Billable drives start→pay/expire | integration (`#[ignore]`/env-gated) | `cargo test -p ferro-payments -- --ignored` (skips without key) | ❌ W0 | ⬜ pending |
| 236-07-xx | 06 | 3 | — | — | docs page builds; full gate + `cargo doc -Dwarnings` green pre-publish | gate | `cargo doc --no-deps -p ferro-payments` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] No framework install — Rust test harness exists.
- [ ] `MockStripeGateway` extended with the poll method (in `ferro-payments/src/service.rs` `#[cfg(test)]`) — shared test fixture for reconcile tests.
- [ ] `ferro_stripe::testing` signed/typed event builders reused for any handler/reaper race test.

*Wave 0 here = the lifecycle finders (`find_expired`, `find_refunds_in_flight`) that every later test depends on; planned as the first wave.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real Stripe test-mode end-to-end | PAY-POLY-REAP-04 | Needs a live Stripe test secret key not present in CI | Set `STRIPE_TEST_SECRET_KEY`, run `cargo test -p ferro-payments -- --ignored`; observe a real Checkout session minted + reaper release/reconcile |
| First-publish of `ferro-payments 0.1.0` | (publish) | New crate; CI token is publish-update only — must run from a local terminal | After rebase + version bump + green full gate: `cargo publish -p ferro-payments` locally; subsequent versions via CI push |
| ferro workspace + ferro-stripe version bump publish | (publish) | Operator git push triggers CI auto-publish; requires verified monotonic version after rebase | `git pull --rebase`, bump workspace 0.2.69→0.2.70, push; then `git update-ref refs/remotes/origin/master HEAD` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (the two finders)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (targeted crate suite)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
