---
phase: 189
slug: ferro-stripe-manual-capture
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-07
validated: 2026-06-07
---

# Phase 189 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `tokio::test` |
| **Config file** | `Cargo.toml` (no separate test config) |
| **Quick run command** | `cargo test -p ferro-stripe` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60 seconds (quick) / several minutes (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-stripe`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~120 seconds
- **Thermal constraint:** one CPU-intensive cargo operation at a time — never parallelize fmt/clippy/test runs

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | Test | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|------|--------|
| 2 | 01 | 1 | STRIPE-MC-01 | T-189-01 | `manual_capture()` sets `capture_method=manual` in params | unit (builder) | `cargo test -p ferro-stripe checkout` | `checkout_create_manual_capture_sets_capture_method` | covered |
| 2 | 01 | 1 | STRIPE-MC-01 | T-189-01 | `manual_capture()` + `Mode::Subscription` → pre-flight structured error before any network call | unit (builder) | `cargo test -p ferro-stripe checkout` | `checkout_create_manual_capture_subscription_returns_err` | covered |
| 1 | 02 | 2 | STRIPE-MC-02 | T-189-04, T-189-05 | `capture(id, None)` / `capture(id, Some(n))` / `cancel(id)` compile, return correct types, invalid id → structured Error | unit | `cargo test -p ferro-stripe payment_intent` | `capture_rejects_invalid_id_before_network`, `capture_rejects_negative_amount`, `cancel_rejects_invalid_id_before_network`, `retrieve_rejects_invalid_id_before_network` | covered |
| 1 | 02 | 2 | STRIPE-MC-02 | WR-01 (review) | `capture(id, Some(0))` → structured error before any network call | unit | `cargo test -p ferro-stripe payment_intent` | `capture_rejects_zero_amount` | covered |
| 2 | 03 | 3 | STRIPE-MC-03 | T-189-10 | `StripePaymentIntentAmountCapturableUpdated::from_raw` parses golden fixture | integration | `cargo test -p ferro-stripe --test parser_contract` | `payment_intent_amount_capturable_updated_parses_all_fields` | covered |
| 2 | 03 | 3 | STRIPE-MC-03 | T-189-10 | `StripePaymentIntentCanceled::from_raw` parses golden fixture | integration | `cargo test -p ferro-stripe --test parser_contract` | `payment_intent_canceled_parses_all_fields` | covered |
| 2 | 03 | 3 | STRIPE-MC-03 | T-189-09 | Cross-type rejection: each new event returns `None` for the other's fixture (pass-through preserved) | integration | `cargo test -p ferro-stripe --test parser_contract` | `payment_intent_amount_capturable_updated_rejects_canceled_event`, `payment_intent_canceled_rejects_amount_capturable_event` | covered |
| 2 | 01 | 1 | STRIPE-MC-04 | T-189-02 | `manual_capture()` + `destination()` combined → ONE `payment_intent_data` carrying BOTH `capture_method` and `transfer_data`/`on_behalf_of` | unit (builder) | `cargo test -p ferro-stripe checkout` | `checkout_create_manual_capture_with_destination_sets_both_fields` | covered |
| 1 | 04 | 4 | STRIPE-MC-05 | T-189-11, T-189-12 | `docs/src/features/stripe.md` contains Manual capture section + hold/commit/release correspondence table | manual review + grep | `grep -c "capture" docs/src/features/stripe.md` | Plan 04 self-check (17 grep hits, all 7 elements verified) | covered |

---

## Wave 0 Gaps

- [x] `ferro-stripe/src/payment_intent.rs` — new file; covers STRIPE-MC-02
- [x] `ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json` — golden fixture for STRIPE-MC-03
- [x] `ferro-stripe/tests/fixtures/stripe_events/payment_intent_canceled.json` — golden fixture for STRIPE-MC-03
- [x] New parser_contract.rs registrations for both events

---

## Manual-Only

None — all requirements have automated verification. STRIPE-MC-05 (docs) is grep-verified with content assertions from Plan 04's self-check.

---

## Validation Audit 2026-06-07

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

Audit evidence: `cargo test -p ferro-stripe` — 34 lib tests + 19 parser-contract tests, 0 failures. All Wave 0 artifacts exist on disk. Post-review fix a37094f5 added `capture_rejects_zero_amount` (WR-01), recorded above.

---

*Source: 189-RESEARCH.md §Validation Architecture*
