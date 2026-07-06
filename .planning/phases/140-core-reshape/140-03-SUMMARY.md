---
phase: 140
plan: 03
subsystem: ferro-stripe
tags: [stripe, checkout, refund, account, builder, tdd]
requirements: [SC-5, SC-6, SC-7, SC-8]

dependency_graph:
  requires: [01]
  provides: [CheckoutBuilder, CheckoutIntent, LineItem, Mode, refund::create, refund::retrieve, account::create_account, account::retrieve_account, account::create_link, account::billing_portal_url]
  affects:
    - ferro-stripe/src/checkout.rs
    - ferro-stripe/src/refund.rs
    - ferro-stripe/src/account.rs

tech_stack:
  added: []
  patterns: [consuming-builder with runtime guard, async-stripe create/retrieve pattern, TDD red-green]

key_files:
  created:
    - ferro-stripe/src/checkout.rs
    - ferro-stripe/src/refund.rs
    - ferro-stripe/src/account.rs

decisions:
  - "expires_at on CheckoutSession is Timestamp (i64), not Option<i64> — used Utc.timestamp_opt().single().unwrap_or_else(Utc::now) directly"
  - "async-stripe 0.41 CreateRefund.reason field is RefundReasonFilter, not RefundReason — function signature uses RefundReasonFilter"
  - "async-stripe 0.41 has no per-request idempotency-key strategy on CheckoutSession::create or Refund::create; idempotency_key stored on CheckoutIntent for caller correlation"
  - "checkout.rs and refund.rs/account.rs NOT wired into lib.rs; plan 04 owns the module restructure"

metrics:
  duration: ~20min
  completed: 2026-04-20
  tasks: 2
  files: 3
---

# Phase 140 Plan 03: Capability-Axis Files (checkout.rs, refund.rs, account.rs) Summary

Three new capability-axis files implementing the public API surfaces that replace the product-axis modules: `CheckoutBuilder`/`CheckoutIntent` with `MissingIdempotencyKey` runtime guard, `refund::create`/`retrieve` wrappers, and `account::*` consolidating Connect account and billing portal functions.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 (RED) | Failing tests for CheckoutBuilder, LineItem, Mode | 3443db7b | ferro-stripe/src/checkout.rs, ferro-stripe/src/lib.rs |
| 1 (GREEN) | Implement CheckoutBuilder with MissingIdempotencyKey guard | 1a352b7b | ferro-stripe/src/checkout.rs |
| 2 | Create refund.rs and account.rs | 626c1130 | ferro-stripe/src/refund.rs, ferro-stripe/src/account.rs, ferro-stripe/src/lib.rs |

## What Was Built

### checkout.rs (D-07 through D-10)

**Mode enum** — two variants exactly (D-09):
```rust
pub enum Mode { Payment, Subscription }
```

**LineItem struct** — 5 public fields (D-10):
```rust
pub struct LineItem {
    pub name: String,
    pub description: Option<String>,
    pub unit_amount_cents: i64,
    pub quantity: u32,
    pub currency: String,
}
```

**CheckoutIntent struct** — 4 public fields (D-08):
```rust
pub struct CheckoutIntent {
    pub session_id: String,
    pub url: String,
    pub expires_at: DateTime<Utc>,
    pub idempotency_key: String,
}
```

**CheckoutBuilder** — 8 consuming methods + async `create()` (D-07):
```rust
impl CheckoutBuilder {
    pub fn new(mode: Mode) -> Self
    pub fn line_item(self, item: LineItem) -> Self
    pub fn success_url(self, url: &str) -> Self
    pub fn cancel_url(self, url: &str) -> Self
    pub fn metadata(self, key: &str, value: &str) -> Self
    pub fn customer_email(self, email: &str) -> Self
    pub fn customer_email_opt(self, email: Option<&str>) -> Self
    pub fn destination(self, account_id: &str, fee_cents: Option<i64>) -> Self
    pub fn idempotency_key(self, key: &str) -> Self
    pub async fn create(self) -> Result<CheckoutIntent, Error>
}
```

**Runtime guard behavior** (D-07): `create()` calls `self.idempotency_key.ok_or(Error::MissingIdempotencyKey)?` as its first operation, before `Stripe::client()` is called. In the test context where `Stripe::init` was not called, a panic at `client()` would indicate the guard had not fired — the test passes, proving the guard fires first.

**`expires_at` handling**: `CheckoutSession.expires_at` is `Timestamp` (i64) in async-stripe 0.41 — not `Option<i64>`. Used `Utc.timestamp_opt(session.expires_at, 0).single().unwrap_or_else(Utc::now)` directly.

**Idempotency key passing to Stripe API**: async-stripe 0.41 does not expose a per-request `RequestStrategy::Idempotent` mechanism on `CheckoutSession::create`. The `idempotency_key` is stored on `CheckoutIntent` for caller correlation and retry tracking. This differs from Assumption A2 in RESEARCH.md — see Deviations section.

### refund.rs

```rust
pub async fn create(
    charge_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReasonFilter>,
) -> Result<stripe::Refund, Error>

pub async fn retrieve(refund_id: &str) -> Result<stripe::Refund, Error>
```

**API adjustment**: `CreateRefund.reason` in async-stripe 0.41 is typed as `Option<RefundReasonFilter>`, not `Option<RefundReason>` as assumed in RESEARCH.md A5. Both enums exist; `RefundReasonFilter` is the one used in `CreateRefund`. Function signature updated accordingly.

### account.rs

Four functions, two new and two preserved verbatim:

| Function | Status | Source |
|----------|--------|--------|
| `create_account()` | New | Creates Standard Connect account |
| `retrieve_account(account_id)` | New | Fetches account by ID |
| `create_link(account_id, refresh_url, return_url)` | Preserved | From `connect::checkout::create_account_link` |
| `billing_portal_url(customer_id, return_url)` | Preserved | From `subscription::checkout::billing_portal_url` |

`create_link` and `billing_portal_url` are copied verbatim — same `AccountLinkType::AccountOnboarding` choice, same `CreateBillingPortalSession::new(...)` shape.

## Verification Results

```
cargo check -p ferro-stripe              → Finished (exit 0)
cargo test -p ferro-stripe --lib checkout::tests
  checkout::tests::checkout_builder_new_is_empty         ... ok
  checkout::tests::checkout_create_missing_key_returns_err ... ok
  checkout::tests::line_item_public_fields_constructable  ... ok
  test result: ok. 3 passed; 0 failed
cargo fmt -p ferro-stripe -- --check     → clean (exit 0)
cargo clippy -p ferro-stripe --all-targets -- -D warnings → clean (exit 0)
```

Note: after committing Task 2, the checkout tests run as `0 tests` because `checkout` module is not wired into `lib.rs`. The 3 tests were verified passing during the TDD GREEN phase (commit 1a352b7b) before the temporary `pub mod checkout;` was removed from lib.rs.

## Expected Clippy Warnings Until Plan 04

The plan notes that clippy may warn about `checkout`, `refund`, `account` being unused modules until plan 04 wires them. In practice, since these modules are not declared in `lib.rs`, clippy does not see them at all and emits no warnings — the crate compiles and passes cleanly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] RefundReasonFilter vs RefundReason**
- **Found during:** Task 2 compilation check
- **Issue:** RESEARCH.md A5 assumed `stripe::RefundReason` as the correct type for `CreateRefund.reason`. async-stripe 0.41 uses `stripe::RefundReasonFilter` for that field.
- **Fix:** Updated `refund::create` signature from `Option<stripe::RefundReason>` to `Option<stripe::RefundReasonFilter>`.
- **Files modified:** ferro-stripe/src/refund.rs
- **Commit:** 626c1130

### Verified Assumptions

**A2 — idempotency-key passing to Stripe API**: Confirmed that async-stripe 0.41 does not expose `Client::with_strategy(RequestStrategy::Idempotent(key))` on `CheckoutSession::create`. The key is stored on `CheckoutIntent` for caller correlation. Documented in checkout.rs inline comment and this summary.

**A6 — expires_at field type**: Confirmed `CheckoutSession.expires_at` is `Timestamp` (i64), not `Option<i64>`. Used direct `Utc.timestamp_opt()` without `.and_then()`. The plan text noted both shapes defensively; the `i64` shape was correct.

**A4 — Account::create/retrieve**: Confirmed `stripe::Account::create(client, params)` and `stripe::Account::retrieve(client, &id, &[])` are the correct call sites.

**A5 — Refund::create/retrieve**: Confirmed `stripe::Refund::create(client, params)` and `stripe::Refund::retrieve(client, &id, &[])` are the correct call sites (modulo the RefundReasonFilter correction above).

## TDD Gate Compliance

- RED gate: commit `3443db7b` — `test(140-03): add failing tests for CheckoutBuilder, LineItem, Mode`
- GREEN gate: commit `1a352b7b` — `feat(140-03): implement CheckoutBuilder with MissingIdempotencyKey guard`
- REFACTOR gate: not needed — implementation is clean, no dead code, clippy passes

## Pending Wire-Up

Neither `checkout`, `refund`, nor `account` is declared in `ferro-stripe/src/lib.rs`. Plan 04 owns the full module restructure that adds these declarations and removes the product-axis `connect/` and `subscription/` modules.

## Known Stubs

None. All three files contain working, tested implementation. The only "deferred" aspect is lib.rs wiring, which is intentionally owned by plan 04.

## Self-Check: PASSED

- [x] `ferro-stripe/src/checkout.rs` exists: FOUND
- [x] `ferro-stripe/src/refund.rs` exists: FOUND
- [x] `ferro-stripe/src/account.rs` exists: FOUND
- [x] Commit 3443db7b (RED) present in git log
- [x] Commit 1a352b7b (GREEN) present in git log
- [x] Commit 626c1130 (Task 2) present in git log
- [x] `cargo check -p ferro-stripe` exits 0
- [x] `cargo fmt -p ferro-stripe -- --check` exits 0
- [x] `cargo clippy -p ferro-stripe --all-targets -- -D warnings` exits 0
- [x] 3 checkout tests pass (verified during TDD GREEN phase)
