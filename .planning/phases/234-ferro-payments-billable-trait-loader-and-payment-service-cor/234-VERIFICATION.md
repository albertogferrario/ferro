---
phase: 234-ferro-payments-billable-trait-loader-and-payment-service-cor
verified: 2026-06-17T04:30:00Z
status: passed
score: 12/12
overrides_applied: 0
---

# Phase 234: Billable Trait + Loader + PaymentService Core — Verification Report

**Phase Goal:** Implement the `Billable` trait and `BillableLoader` trait per the spec. Implement `PaymentService<L: BillableLoader>` with `start_checkout` (mints a Stripe Checkout session via `ferro_stripe::CheckoutBuilder`, snapshots `application_fee_cents`, attaches the session id) and `request_refund` (calls Stripe refund API, snapshots `refund_amount_cents`). Implement `PaymentError` enum. Unit tests use mocked Stripe, mocked `BillableLoader`, and mocked `ProcessedEventLog`. No webhook integration yet — that's Phase 235.
**Verified:** 2026-06-17T04:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `Billable` is an object-safe `#[async_trait]` trait with sync accessors and async `on_*` side effects | VERIFIED | `ferro-payments/src/billable.rs` line 17: `pub trait Billable: Send + Sync`; async methods `on_paid`, `on_released`, `on_refunded` take `&DatabaseTransaction`; tests confirm `Box<dyn Billable>` is constructible |
| 2 | `Billable::connect_account_id` has a default returning `None` | VERIFIED | `billable.rs` line 34: `fn connect_account_id(&self) -> Option<String> { None }`; test `connect_account_id_defaults_to_none` asserts it |
| 3 | `Billable` is NOT `Clone` | VERIFIED | No `Clone` bound on the trait definition; trait is `Billable: Send + Sync` only |
| 4 | `BillableLoader::load` returns `Result<Option<Box<dyn Billable>>, PaymentError>` | VERIFIED | `loader.rs` line 21-25: `async fn load(&self, kind: BillableKind, id: i64) -> Result<Option<Box<dyn Billable>>, PaymentError>`; no `tenant_id` parameter (D-08) |
| 5 | `lifecycle::attach_session` sets `stripe_session_id` + `application_fee_cents` guarded by `StripeSessionId IS NULL` | VERIFIED | `lifecycle.rs` line 135-158: `GuardedUpdate` with `.filter(Column::StripeSessionId.is_null())`; `Value::BigInt(None)` for SQL NULL; tests `attach_session_sets_session_and_fee` and `attach_session_idempotent_second_call_noops` both pass |
| 6 | `StripeGateway` trait abstracts checkout-session + refund so `PaymentService` is unit-testable without `Stripe::init` | VERIFIED | `service.rs` line 69-81: `pub trait StripeGateway: Send + Sync` with `create_checkout_session` and `create_refund`; `MockStripeGateway` in `#[cfg(test)]` with `Mutex<Vec<CheckoutRequest>>`; all 6 service tests pass with no live Stripe |
| 7 | `StripeClientGateway` production impl wraps `ferro_stripe::CheckoutBuilder` + `refund::create`; `Stripe::config()` confined to this impl only | VERIFIED | `service.rs` line 93-138: `StripeClientGateway` calls `ferro_stripe::CheckoutBuilder`, `ferro_stripe::Stripe::config().application_fee_for(...)`, and `ferro_stripe::refund::create`; `PaymentService` methods never call these directly |
| 8 | `start_checkout` inserts a reserved row, calls the gateway, and attaches `session_id` + `application_fee_cents` | VERIFIED | `service.rs` line 192-236: `lifecycle::create_reserved` → `self.stripe.create_checkout_session` → `lifecycle::attach_session`; test `start_checkout` asserts row exists, `stripe_session_id == "cs_test_mock"`, `application_fee_cents == Some(250)`, `checkout_call_count() == 1` |
| 9 | `request_refund` refuses non-paid / no-charge_id intents and dedups via `GuardedUpdate WHERE refund_amount_cents IS NULL` | VERIFIED | `service.rs` line 255-297: status check + `charge_id.ok_or_else(...)` + `GuardedUpdate WHERE Column::RefundAmountCents.is_null()`; `if !snapshot_ok { return Ok(()) }`; tests `request_refund_precondition` and `request_refund_dedup` pass |
| 10 | `PaymentError` carries `Stripe(#[from] ferro_stripe::Error)`, `Loader(Box<dyn...>)`, and `AutoRefundTriggered { reason }` variants; `AutoRefundReason` defined | VERIFIED | `error.rs`: 6-variant enum; `Stripe(#[from] ferro_stripe::Error)` on line 21; `Loader(Box<dyn std::error::Error + Send + Sync>)` without `#[from]` on line 26; `AutoRefundTriggered { reason: AutoRefundReason }` on line 32; `AutoRefundReason` enum with `LoaderError`, `BillableVanished`, `SideStateConflict` |
| 11 | `ferro-payments/Cargo.toml` depends on `ferro-stripe = { path = "../ferro-stripe", version = "0.9" }` | VERIFIED | `Cargo.toml` line 22: `ferro-stripe = { path = "../ferro-stripe", version = "0.9" }` |
| 12 | `publish.yml` publishes `ferro-payments` in a Wave 1c step after the Wave 1b index-wait; `ferro-payments` absent from `WAVE1B_CRATES` | VERIFIED | Lines 271-293: `Publish Wave 1c (depends on Wave 1b only)` step with `WAVE1C_CRATES="ferro-payments"`; line 248: `WAVE1B_CRATES=` does not contain `ferro-payments`; `Wait for crates.io index update (Wave 1c)` step present after Wave 1c publish |

**Score:** 12/12 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-payments/src/billable.rs` | `Billable` trait + `connect_account_id` default | VERIFIED | 106 lines; trait body matches spec verbatim; test module confirms object-safety and default |
| `ferro-payments/src/loader.rs` | `BillableLoader` trait | VERIFIED | 65 lines; single `load` method, no `tenant_id` param; `MockLoader` test confirms `&dyn BillableLoader` usable |
| `ferro-payments/src/service.rs` | `PaymentService<L>` + `StripeGateway` + types + unit tests | VERIFIED | 747 lines; all 6 named tests present (`start_checkout`, `start_checkout_no_connect`, `request_refund`, `request_refund_precondition`, `request_refund_dedup`, `mock_gateway_records_calls`) |
| `ferro-payments/src/intent/lifecycle.rs` | `attach_session` lifecycle fn | VERIFIED | Lines 129-158; `pub async fn attach_session` present; guarded by `Column::StripeSessionId.is_null()` |
| `ferro-payments/src/error.rs` | Extended `PaymentError` + `AutoRefundReason` | VERIFIED | 6-variant `PaymentError`; `AutoRefundReason` with 3 variants; `Stripe` uses `#[from]`, `Loader` does not |
| `ferro-payments/Cargo.toml` | `ferro-stripe` dependency | VERIFIED | Line 22: `ferro-stripe = { path = "../ferro-stripe", version = "0.9" }` |
| `ferro-payments/src/lib.rs` | Public re-exports for the orchestration layer | VERIFIED | `pub use billable::Billable`, `pub use loader::BillableLoader`, `pub use service::{CheckoutRequest, CheckoutResponse, CheckoutUrl, PaymentService, ReturnUrls, StripeClientGateway, StripeGateway}`, `pub use error::AutoRefundReason`, `pub use intent::lifecycle::attach_session` |
| `.github/workflows/publish.yml` | Wave 1c publish step | VERIFIED | Lines 271-293 |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `loader.rs` | `Billable` | `Box<dyn Billable>` return type | VERIFIED | `loader.rs` line 22: `Result<Option<Box<dyn Billable>>, PaymentError>` |
| `lifecycle.rs` | `payment_intents.stripe_session_id` | `GuardedUpdate WHERE StripeSessionId.is_null()` | VERIFIED | `.filter(Column::StripeSessionId.is_null())` confirmed at line 143 |
| `service.rs` | `lifecycle::create_reserved + lifecycle::attach_session` | `start_checkout` composes the lifecycle layer | VERIFIED | `lifecycle::create_reserved` at line 198; `lifecycle::attach_session` at line 227 |
| `service.rs` | `self.stripe (Arc<dyn StripeGateway>)` | all Stripe calls route through gateway seam | VERIFIED | `self.stripe.create_checkout_session` line 221; `self.stripe.create_refund` line 293; no direct `CheckoutBuilder` or `Stripe::config()` in `PaymentService` |
| `service.rs` | `payment_intents.refund_amount_cents` | `GuardedUpdate WHERE RefundAmountCents.is_null()` | VERIFIED | `Column::RefundAmountCents.is_null()` at line 281 |
| `error.rs` | `ferro_stripe::Error` | `#[from]` in `Stripe` variant | VERIFIED | `Stripe(#[from] ferro_stripe::Error)` at line 21 |
| `publish.yml` | crates.io index (Wave 1b) | index-wait before Wave 1c publish | VERIFIED | `Wait for crates.io index update (Wave 1b)` at line 266; Wave 1c publish at line 271 |
| `lib.rs` | `service::*` | `pub use service::` | VERIFIED | `lib.rs` lines 21-24: full re-export block |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 23 ferro-payments tests pass | `cargo test -p ferro-payments` | 23 passed, 0 failed | PASS |
| `start_checkout` (PAY-POLY-SVC-03a) | filtered by `cargo test -p ferro-payments -- start_checkout` | included in 23 passed | PASS |
| `start_checkout_no_connect` (PAY-POLY-SVC-03b) | filtered name match | included in 23 passed | PASS |
| `request_refund` (PAY-POLY-SVC-03c) | filtered name match | included in 23 passed | PASS |
| `request_refund_precondition` (PAY-POLY-SVC-03d) | filtered name match | included in 23 passed | PASS |
| `request_refund_dedup` (PAY-POLY-SVC-03e) | filtered name match | included in 23 passed | PASS |
| `mock_gateway_records_calls` (PAY-POLY-SVC-04) | included in 23 passed | included in 23 passed | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| PAY-POLY-SVC-01 | 234-02-PLAN | `Billable` object-safe async trait with defaulted `connect_account_id` | SATISFIED | `billable.rs` trait definition; tests confirm object-safety and `None` default |
| PAY-POLY-SVC-02 | 234-02-PLAN | `BillableLoader` object-safe trait returning `Box<dyn Billable>` | SATISFIED | `loader.rs` trait definition; `MockLoader` confirms `&dyn BillableLoader` usable |
| PAY-POLY-SVC-03a | 234-03-PLAN | `start_checkout` Connect billable: reserved row + session_id + fee snapshotted | SATISFIED | `service::tests::start_checkout` green; asserts `stripe_session_id`, `application_fee_cents == Some(250)`, `checkout_call_count == 1` |
| PAY-POLY-SVC-03b | 234-03-PLAN | `start_checkout` non-Connect: `application_fee_cents` stays NULL | SATISFIED | `service::tests::start_checkout_no_connect` green; asserts `application_fee_cents.is_none()` |
| PAY-POLY-SVC-03c | 234-03-PLAN | `request_refund` paid+charge_id: snapshots `refund_amount_cents`, calls Stripe once | SATISFIED | `service::tests::request_refund` green; asserts `refund_amount_cents == Some(5000)`, `refund_call_count == 1` |
| PAY-POLY-SVC-03d | 234-03-PLAN | `request_refund` non-paid / missing charge_id: `StatusPrecondition`, Stripe NOT called | SATISFIED | `service::tests::request_refund_precondition` green; asserts `matches!(err, StatusPrecondition(_))` and `refund_call_count == 0` |
| PAY-POLY-SVC-03e | 234-03-PLAN | dedup: 2nd `request_refund` no-ops; Stripe called exactly once across both calls | SATISFIED | `service::tests::request_refund_dedup` green; asserts `refund_call_count == 1` after two calls |
| PAY-POLY-SVC-04 | 234-03-PLAN | `MockStripeGateway` records calls; tests assert counts and captured fields | SATISFIED | `service::tests::mock_gateway_records_calls` green; asserts `amount_cents`, `currency`, `idempotency_key` prefix, `connect_account_id` |
| PAY-POLY-SVC-05 | 234-01-PLAN | `PaymentError::Stripe(#[from])` + `Loader` + `AutoRefundTriggered` compile | SATISFIED | `error.rs` 6-variant enum compiles clean under `-D warnings` |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `service.rs` | 151 | `#[allow(dead_code)] // wired by handle_* in phase 235` on `loader` field | Info | Intentional forward declaration per D-09; phase-235 webhook handlers will wire this field |

No blocking anti-patterns. The single `dead_code` allow is documented by design.

---

### Human Verification Required

None. All observable behaviors are covered by the unit test suite (`cargo test -p ferro-payments`, 23 tests green) and static code inspection. The one behavior listed as manual-only in the VALIDATION.md — publish.yml Wave 1c ordering — is verifiable via YAML inspection: Wave 1c step (line 271) appears after the Wave 1b index-wait (line 266) and `ferro-payments` is absent from `WAVE1B_CRATES` (line 248). No runtime publish was needed.

---

### Gaps Summary

No gaps. All 12 must-have truths verified. All 6 named unit tests pass. All public re-exports present. Cargo.toml dependency wired. Publish wave ordering fixed.

---

_Verified: 2026-06-17T04:30:00Z_
_Verifier: Claude (gsd-verifier)_
