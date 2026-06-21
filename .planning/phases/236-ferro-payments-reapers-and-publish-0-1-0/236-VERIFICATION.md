---
phase: 236-ferro-payments-reapers-and-publish-0-1-0
verified: 2026-06-21T03:00:00Z
status: human_needed
score: 8/8 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run the gated e2e integration test against Stripe test mode"
    expected: "cargo test -p ferro-payments --test integration -- --ignored with STRIPE_TEST_SECRET_KEY set drives start_checkout → release_expired and asserts the row reaches status=released against real ferro-stripe test mode"
    why_human: "Requires a live STRIPE_TEST_SECRET_KEY (external service). The test is #[ignore]-gated and skips cleanly when absent — its end-to-end behavior cannot be verified offline (isolate-before-spending convention)."
known_issues:
  - "CR-01 (code review, post-publish): charge_id is never persisted in the production lifecycle, so the public request_refund manual-refund API is structurally unreachable. Does NOT affect this phase's goal — the reaper auto-refund/reconcile path refunds by payment_intent_id, which IS persisted. Tracked for a 0.1.1 follow-up."
  - "WR-01..05 + IN-01..04 (code review warnings/info): webhook idempotency-before-side-effect lost-update window, opaque unique-violation mapping, reconcile trusts Stripe amount over snapshot, vanished-loader strands refunded row, refunds.first() refund-selection ambiguity, discarded Stripe idempotency key, coarse is_transient, missing charge_id index, hardcoded 1h age anchor. All for 0.1.1."
---

# Phase 236: Reapers + Workspace Test Bin + Publish 0.1.0 Verification Report

**Phase Goal:** Implement `ReleaseExpiredPaymentIntents` (single SQL pass over payment_intents WHERE status='reserved' AND expires_at < now(), dispatches `on_released` per row in a transaction) and `ReconcileRefundsInFlight` (polls Stripe for refund-in-flight intents). Both as ferro-queue-compatible Job structs consumers schedule via cron. Add a tiny example Billable in a workspace test bin to drive end-to-end against ferro-stripe test mode. Version-bump ferro workspace + publish ferro-payments 0.1.0 to crates.io.
**Verified:** 2026-06-21T03:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth | Status | Evidence |
| --- | ----- | ------ | -------- |
| 1   | `find_expired` selects only reserved intents with `expires_at < now`; `find_refunds_in_flight` selects only paid+refund-snapshot+null-refunded_at rows | ✓ VERIFIED | lifecycle.rs:214 `find_expired`, :234 `find_refunds_in_flight`; :219 `Column::Status.eq(PaymentIntentStatus::Reserved)`, :240 `Column::RefundAmountCents.is_not_null()`. Six behavior unit tests per Plan 01. |
| 2   | ferro-stripe exposes a read-only `list_for_payment_intent` poll primitive; gateway exposes `fetch_refund_status_for_payment_intent` returning `RefundStatus` (Succeeded/Pending/Failed); mock records poll calls | ✓ VERIFIED | refund.rs:90 `list_for_payment_intent` → :102 `stripe::Refund::list`; service.rs:48 `pub enum RefundStatus`, :191 client impl → :195 delegates to `ferro_stripe::refund::list_for_payment_intent`, :652 mock `poll_calls`, :719 mock impl. |
| 3   | `release_expired`/`release_expired_at` releases only expired reserved intents, per-intent txn, race no-op (mark_released==false), failure-isolated, loader-vanished benign (no money captured) | ✓ VERIFIED | service.rs:414 `release_expired_at` → :418 `find_expired(now,…)`, :444-449 "no money captured" benign skip, :476 public delegate. Tests at :1416/1459/1488/1600 cover happy/not-expired/race/continue-on-error. |
| 4   | `reconcile_refunds_in_flight`/`_at` resolves Succeeded via mark_refunded+on_refunded, skips Pending, warns-without-retry on Failed (double-refund guard); clock injected | ✓ VERIFIED | service.rs:495 `reconcile_refunds_in_flight_at`, :561-567 "double-refund guard…operator action required", :595 public delegate. Tests :1636/1673/1707; :1714 asserts NO refund-creation call on Failed. |
| 5   | Two ferro-queue Job structs (`ReleaseExpiredPaymentIntents<L>`, `ReconcileRefundsInFlight<L>`) with serde-skipped `Arc<PaymentService<L>>` injected via `::new`, clean JobFailed on missing handle, PaymentError→JobFailed mapping, re-exported from lib.rs | ✓ VERIFIED | reaper.rs:67/133 `impl … ferro_queue::Job for …<L>`, :37/103 `#[serde(bound="")]`, :59/125 `pub fn new`, :72/138 `service.as_ref().ok_or_else(JobFailed)`, :77 `svc.release_expired()`, :142 `svc.reconcile_refunds_in_flight()`. lib.rs:10 `mod reaper;`, :23 `pub use reaper::{…}`. ferro-queue dep Cargo.toml:24. reaper.rs = 404 lines. |
| 6   | A #[ignore]-gated integration test defines a tiny example Billable and drives start_checkout→release_expired end-to-end against ferro-stripe test mode, skipping cleanly (no panic) when STRIPE_TEST_SECRET_KEY absent | ✓ VERIFIED (structure) | integration.rs:144 `#[ignore = "requires STRIPE_TEST_SECRET_KEY …"]`, :147 env read, :150 early-return skip, :67 `impl Billable for ReservationBillable`, :139 release_expired path. 239 lines. End-to-end execution with a live key → see Human Verification. |
| 7   | Consumer-facing docs page leads with the one-call recovery story (migrations, wire_dispatcher, two cron reapers, auto-refund+reconcile model with double-refund guard); wired into SUMMARY + cross-linked from stripe.md; cargo doc -Dwarnings green | ✓ VERIFIED | docs/src/features/payments.md present (7985 bytes, 13 hits of wire_dispatcher/Release.../Reconcile...); SUMMARY.md:50 `- [Payments](features/payments.md)`; stripe.md:5 cross-link. Plan 06 ran `RUSTDOCFLAGS="-D warnings" cargo doc`. |
| 8   | ferro-payments 0.1.0 published to crates.io (local bootstrap); ferro core + ferro-stripe republished at bumped workspace version via CI; workspace bumped monotonically > tag, ferro-payments stays 0.1.0; milestone tagged | ✓ VERIFIED | `cargo search`: ferro-payments=0.1.0, ferro-stripe=0.9.1, ferro-rs=0.2.70. Cargo.toml workspace version=0.2.70; ferro-payments/Cargo.toml:3 version="0.1.0". Tags v0.2.69 + v0.2.70 present. CI Publish run 27887985604 success (operator-confirmed). |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `ferro-payments/src/intent/lifecycle.rs` | find_expired + find_refunds_in_flight | ✓ VERIFIED | Both finders + 6 unit tests; reserved-only / null-refunded_at predicates present |
| `ferro-stripe/src/refund.rs` | list_for_payment_intent poll primitive | ✓ VERIFIED | Read-only `stripe::Refund::list` wrapper, invalid-id parse test |
| `ferro-payments/src/service.rs` | RefundStatus + poll gateway method + 4 reaper methods | ✓ VERIFIED | enum + trait method + StripeClientGateway + MockStripeGateway + release/reconcile pairs, all wired and tested |
| `ferro-payments/src/reaper.rs` | 2 ferro-queue Job structs | ✓ VERIFIED | 404 lines; both Job impls, serde(bound=""), ::new injection, JobFailed mapping, behavior tests |
| `ferro-payments/Cargo.toml` | ferro-queue dep + own 0.1.0 version | ✓ VERIFIED | :24 ferro-queue path+version dep; :3 version="0.1.0" (independent of workspace bump) |
| `ferro-payments/src/lib.rs` | job re-exports | ✓ VERIFIED | mod reaper + pub use reaper::{…} |
| `ferro-payments/tests/integration.rs` | gated e2e + example Billable | ✓ VERIFIED (structure) | 239 lines, #[ignore], env-guard skip, impl Billable, release_expired drive |
| `docs/src/features/payments.md` | consumer + recovery docs | ✓ VERIFIED | present, all required sections, SUMMARY + stripe.md links |
| `Cargo.toml` (workspace) | bumped version | ✓ VERIFIED | 0.2.70, > tag, monotonic |

### Key Link Verification

| From | To | Via | Status |
| ---- | -- | --- | ------ |
| service.rs::release_expired_at | lifecycle::find_expired | per-intent txn loop | ✓ WIRED (service.rs:418 `find_expired(now, &self.db)`) |
| service.rs::reconcile_refunds_in_flight_at | StripeGateway::fetch_refund_status_for_payment_intent | poll then mark_refunded on Succeeded | ✓ WIRED (service.rs:191/195 delegation; reconcile body invokes poll) |
| service.rs::StripeClientGateway | ferro_stripe::refund::list_for_payment_intent | delegated call + status mapping | ✓ WIRED (service.rs:195) |
| reaper.rs::ReleaseExpiredPaymentIntents::handle | PaymentService::release_expired | Arc handle via ::new, called in Job::handle | ✓ WIRED (reaper.rs:77 `svc.release_expired()`) |
| reaper.rs::ReconcileRefundsInFlight::handle | PaymentService::reconcile_refunds_in_flight | Arc handle via ::new, called in Job::handle | ✓ WIRED (reaper.rs:142 `svc.reconcile_refunds_in_flight()`) |
| docs SUMMARY.md | features/payments.md | mdbook nav after Stripe | ✓ WIRED (SUMMARY.md:50) |
| push to master | publish.yml CI auto-publish | version > tag triggers job | ✓ WIRED (CI run 27887985604 success; tag v0.2.70) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| ferro-payments compiles with new ferro-queue dep + reaper module | `cargo build -p ferro-payments` | Compiling cleanly (deps building, no errors surfaced) | ✓ PASS |
| ferro-payments 0.1.0 resolvable on crates.io | `cargo search ferro-payments` | `ferro-payments = "0.1.0"` | ✓ PASS |
| ferro-stripe poll primitive shipped on crates.io | `cargo search ferro-stripe` | `ferro-stripe = "0.9.1"` | ✓ PASS |
| ferro-rs republished at bumped version | `cargo search ferro-rs` | `ferro-rs = "0.2.70"` | ✓ PASS |
| Milestone tag present | `git tag --list v0.2.70` | `v0.2.70` | ✓ PASS |
| e2e integration test exercises real Stripe test mode | `cargo test … --test integration -- --ignored` (key set) | Requires STRIPE_TEST_SECRET_KEY | ? SKIP → Human Verification |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| PAY-POLY-REAP-01 | 236-01, 236-03 | Release reaper: single pass over reserved+expired intents, on_released per row in txn | ✓ SATISFIED | find_expired + release_expired_at (per-intent txn, isolation, race no-op) |
| PAY-POLY-REAP-02 | 236-01, 236-02, 236-03 | Reconcile reaper: poll Stripe for refund-in-flight intents | ✓ SATISFIED | find_refunds_in_flight + list_for_payment_intent + fetch_refund_status + reconcile_refunds_in_flight_at |
| PAY-POLY-REAP-03 | 236-04 | Both reapers as ferro-queue Job structs scheduled via cron | ✓ SATISFIED | reaper.rs two Job impls + lib.rs re-exports + ferro-queue dep |
| PAY-POLY-REAP-04 | 236-05, 236-06, 236-07 | Example Billable test bin (e2e against Stripe test mode), docs, version-bump + publish 0.1.0 | ✓ SATISFIED | integration.rs gated e2e + example Billable; payments.md docs; crates.io 0.1.0 live. (e2e live-key execution → human) |

Requirement IDs PAY-POLY-REAP-01..04 are defined in the authoritative spec `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md` and ROADMAP.md:3259 (not a REQUIREMENTS.md file — confirmed by 236-CONTEXT.md:183). All four declared across plans are accounted for; no orphaned IDs.

### Anti-Patterns Found

No blocking anti-patterns. The reaper bodies, finders, and Job structs contain real logic (GuardedUpdate no-ops, per-intent transactions, status mapping) — no stubs, placeholders, or empty handlers. The `on_*` hooks in the integration-test example Billable are intentional no-ops (the test asserts payment-intent row state, not consumer side effects) — not a stub.

### Human Verification Required

#### 1. End-to-end integration test against Stripe test mode

**Test:** With a valid `STRIPE_TEST_SECRET_KEY` (a `sk_test_…` key) exported, run:
`cargo test -p ferro-payments --test integration -- --ignored`
**Expected:** The test runs `start_checkout` for the example `ReservationBillable`, forces expiry, calls `release_expired()`, and asserts the payment-intent row reaches `status=released`. It exercises the real ferro-stripe test-mode client.
**Why human:** Requires a live external Stripe secret. The test is `#[ignore]`-gated and returns early (skips cleanly, no panic) when the key is absent, so its end-to-end behavior cannot be verified offline. Per the isolate-before-spending convention, this is the free gated path; a funded live run confirms the happy path.

### Gaps Summary

No gaps block goal achievement. The phase goal — ship both reapers as ferro-queue Job structs and publish ferro-payments 0.1.0 — is fully achieved and confirmed against crates.io. All 8 must-haves and all four PAY-POLY-REAP requirements are satisfied in the codebase and the published artifacts.

One item routes to human verification: the gated e2e integration test's behavior under a live Stripe test key (offline it only proves clean-skip).

The 236-REVIEW.md code review surfaced one critical (CR-01: `charge_id` never persisted → `request_refund` manual API unreachable) and five warnings. These are recorded under `known_issues` for a 0.1.1 follow-up. CR-01 does NOT change goal achievement: the phase goal is the reaper recovery model + publish, and the reaper auto-refund/reconcile path operates on `payment_intent_id` (which IS persisted), not `charge_id`. The manual `request_refund` path is out of this phase's scope (it predates 236).

---

_Verified: 2026-06-21T03:00:00Z_
_Verifier: Claude (gsd-verifier)_
