---
phase: 189
slug: ferro-stripe-manual-capture
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-07
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

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | STRIPE-MC-01 | — | `manual_capture()` sets `capture_method=manual` in params | unit (builder) | `cargo test -p ferro-stripe checkout` | ✅ `src/checkout.rs` (mod tests) | pending |
| TBD | TBD | TBD | STRIPE-MC-01 | — | `manual_capture()` + `Mode::Subscription` → pre-flight structured error before any network call | unit (builder) | `cargo test -p ferro-stripe checkout` | ✅ (new test in mod tests) | pending |
| TBD | TBD | TBD | STRIPE-MC-02 | — | `capture(id, None)` / `capture(id, Some(n))` / `cancel(id)` compile, return correct types, invalid id → structured Error | unit | `cargo test -p ferro-stripe payment_intent` | ❌ Wave 0: `src/payment_intent.rs` (new file) | pending |
| TBD | TBD | TBD | STRIPE-MC-03 | — | `StripePaymentIntentAmountCapturableUpdated::from_raw` parses golden fixture | integration | `cargo test -p ferro-stripe --test parser_contract` | ❌ Wave 0: new fixture + test | pending |
| TBD | TBD | TBD | STRIPE-MC-03 | — | `StripePaymentIntentCanceled::from_raw` parses golden fixture | integration | `cargo test -p ferro-stripe --test parser_contract` | ❌ Wave 0: new fixture + test | pending |
| TBD | TBD | TBD | STRIPE-MC-03 | — | Cross-type rejection: each new event returns `None` for the other's fixture (pass-through preserved) | integration | `cargo test -p ferro-stripe --test parser_contract` | ❌ Wave 0 | pending |
| TBD | TBD | TBD | STRIPE-MC-04 | — | `manual_capture()` + `destination()` combined → ONE `payment_intent_data` carrying BOTH `capture_method` and `transfer_data`/`on_behalf_of` | unit (builder) | `cargo test -p ferro-stripe checkout` | ✅ (new test in mod tests) | pending |
| TBD | TBD | TBD | STRIPE-MC-05 | — | `docs/src/features/stripe.md` contains Manual capture section + hold/commit/release correspondence table | manual review + grep | `grep -c "capture" docs/src/features/stripe.md` | ✅ existing file | pending |

---

## Wave 0 Gaps

- [ ] `ferro-stripe/src/payment_intent.rs` — new file; covers STRIPE-MC-02
- [ ] `ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json` — golden fixture for STRIPE-MC-03
- [ ] `ferro-stripe/tests/fixtures/stripe_events/payment_intent_canceled.json` — golden fixture for STRIPE-MC-03
- [ ] New parser_contract.rs registrations for both events

---

*Source: 189-RESEARCH.md §Validation Architecture*
