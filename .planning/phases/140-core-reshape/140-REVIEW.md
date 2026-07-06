---
phase: 140
status: issues_found
depth: standard
reviewed_at: 2026-04-20T00:00:00Z
files_reviewed: 18
files_reviewed_list:
  - ferro-stripe/Cargo.toml
  - ferro-stripe/src/lib.rs
  - ferro-stripe/src/client.rs
  - ferro-stripe/src/error.rs
  - ferro-stripe/src/config.rs
  - ferro-stripe/src/idempotency.rs
  - ferro-stripe/src/checkout.rs
  - ferro-stripe/src/refund.rs
  - ferro-stripe/src/account.rs
  - ferro-stripe/src/testing.rs
  - ferro-stripe/src/webhook/mod.rs
  - ferro-stripe/src/webhook/events.rs
  - ferro-stripe/src/webhook/verify.rs
  - ferro-stripe/src/webhook/queue.rs
  - ferro-stripe/src/webhook/sync.rs
  - framework/Cargo.toml
  - framework/src/lib.rs
  - framework/src/tenant/mod.rs
  - framework/src/tenant/requires_plan.rs
  - framework/src/tenant/subscription.rs
findings:
  critical: 0
  warning: 4
  info: 4
  total: 8
---

# Phase 140: Code Review Report

**Reviewed:** 2026-04-20
**Depth:** standard
**Files Reviewed:** 18 (ferro-stripe restructure + framework tenant module)
**Status:** issues_found

## Summary

The capability-axis restructure is architecturally sound. The module split
(checkout, refund, account, idempotency, webhook) is clean and well-documented.
Error handling is consistent and every public function has a clear error type.
The `ProcessedEventLog` / `MemoryProcessedLog` idempotency design is correct —
DashMap shard-level atomicity is the right tool for the in-process case and the
docs accurately warn about its production limitations.

Four warnings and four info items were found. No critical issues.

---

## Warnings

### WR-01: Idempotency key accepted but silently ignored in `refund::create`

**File:** `ferro-stripe/src/refund.rs:22`

**Issue:** `idempotency_key` is accepted as a required parameter (enforcing caller discipline), but the value is immediately discarded with `let _ = idempotency_key;`. The comment says async-stripe 0.41 does not expose a per-request mechanism. That may be accurate, but callers have no way to know the key has no effect. If a caller retries a failed refund with the same key expecting deduplication at the Stripe layer, they will silently get a duplicate charge.

The API surface implies safety that does not exist. This is a correctness risk for money-moving code.

**Fix:** One of:
1. Document the no-op explicitly in the parameter's doc comment so callers cannot miss it:
   ```rust
   /// NOTE: async-stripe 0.41 does not forward this key to the Stripe API.
   /// Stripe-layer deduplication is NOT guaranteed until this crate upgrades.
   /// Application-layer deduplication (e.g. a DB unique constraint on charge_id)
   /// is required to prevent duplicate refunds on retry.
   pub idempotency_key: &str,
   ```
2. Alternatively, remove the parameter until it can be forwarded, so callers are not misled.

---

### WR-02: `checkout.rs` `session.url` silently defaults to empty string on None

**File:** `ferro-stripe/src/checkout.rs:222`

**Issue:** `session.url.unwrap_or_default()` maps `None` to `""`. `CheckoutIntent.url` is typed as `String` (not `Option<String>`), so callers cannot distinguish "Stripe returned no URL" from "Stripe returned an empty string". A caller that redirects the user to `intent.url` will redirect to `""` (i.e. the current path), producing a confusing UX failure with no error signal.

Stripe's hosted-page URL should always be present for a successfully created session; `None` here indicates an unexpected API response that should surface as an error.

**Fix:**
```rust
let url = session.url.ok_or_else(|| {
    Error::Stripe("checkout session created but url field was absent".to_string())
})?;

Ok(CheckoutIntent {
    session_id: session.id.to_string(),
    url,
    expires_at,
    idempotency_key,
})
```

---

### WR-03: `TenantFailureMode::Custom` panics on clone — undocumented in public API

**File:** `framework/src/tenant/mod.rs:150`

**Issue:** `TenantFailureMode` derives/implements `Clone`, but the `Custom` variant panics at runtime:
```rust
Self::Custom(_) => panic!("TenantFailureMode::Custom cannot be cloned"),
```
`Clone` is part of the type's public API. Callers reasonably expect `Clone` to be total. The panic is undocumented at the type level or the variant level. Any middleware that stores `TenantMiddleware` (which likely clones `TenantFailureMode` when cloning the middleware) and uses the `Custom` variant will crash at startup or on the first request, with no compile-time warning.

**Fix:** Either:
1. Remove `Clone` from `TenantFailureMode` and propagate that constraint to `TenantMiddleware` (preferred — correct at the type level).
2. Or, if `Clone` must be kept, document the panic prominently on the `Custom` variant with a `#[doc]` note, and add a `#[must_use]` note that `Custom` instances should not be placed in `Clone`-requiring contexts.

---

### WR-04: `config.rs` test mutates process-global env vars without cleanup

**File:** `ferro-stripe/src/config.rs:55-60`

**Issue:** The test calls `std::env::remove_var()` on `STRIPE_SECRET_KEY` and `STRIPE_WEBHOOK_SECRET` but never restores them. Cargo runs unit tests in the same process by default. If another test elsewhere in the test binary reads these vars (or if the integration test suite sets them for a different test), the removal can cause flaky failures depending on test execution order. This is the known env-var mutation hazard in Rust test suites.

**Fix:** Use a scoped guard pattern, or assert the vars are absent before removing (to make the test's precondition explicit). A minimal fix:
```rust
// Save and restore
let old_key = std::env::var("STRIPE_SECRET_KEY").ok();
let old_secret = std::env::var("STRIPE_WEBHOOK_SECRET").ok();
std::env::remove_var("STRIPE_SECRET_KEY");
std::env::remove_var("STRIPE_WEBHOOK_SECRET");

let result = StripeConfig::from_env();
assert!(matches!(result, Err(Error::Config(_))));

// Restore
if let Some(k) = old_key { std::env::set_var("STRIPE_SECRET_KEY", k); }
if let Some(s) = old_secret { std::env::set_var("STRIPE_WEBHOOK_SECRET", s); }
```
Or use `serial_test` (already in `framework`'s dev-dependencies) or `temp-env` crate.

---

## Info

### IN-01: `idempotency.rs` concurrent test does not enforce which caller wins

**File:** `ferro-stripe/src/idempotency.rs:105-121`

**Issue:** The test `memory_log_concurrent_insert_applies_once` asserts `v1 != v2` (exactly one true, one false). This is correct for the happy path but the test is non-deterministic by construction — `tokio::spawn` on a single-threaded test runtime may execute tasks sequentially, not in parallel, making the race condition trivially serialized. The test passes for the wrong reason on `#[tokio::test]` (which defaults to current-thread). Consider adding `#[tokio::test(flavor = "multi_thread")]` to make the concurrent scenario actually concurrent.

**Fix:**
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn memory_log_concurrent_insert_applies_once() { ... }
```

---

### IN-02: `events.rs` parse helpers silently swallow errors

**File:** `ferro-stripe/src/webhook/events.rs:158-221`

**Issue:** All five `parse_*` functions return `Option<T>` and silently discard parse errors. If a Stripe event arrives with an unexpected schema (e.g., `customer` is an object instead of a string after a Stripe API version change), the event is silently ignored and `ProcessStripeWebhook::handle` returns `Ok(())` with no trace. This makes debugging schema drift very hard.

This is in-scope as a code quality issue because silent `Ok(())` from a job that should have processed an event is indistinguishable from legitimate success.

**Fix:** Log a warning (even `eprintln!` or a framework log macro) when a parse returns `None` in the `handle()` match arms, so the failure is observable:
```rust
"customer.subscription.updated" => {
    match parse_subscription_updated(&self.event_json) {
        Some(event) => event.dispatch_sync(),
        None => {
            // Replace with framework logger when available
            eprintln!("[ferro-stripe] failed to parse customer.subscription.updated: {}", &self.event_json[..200.min(self.event_json.len())]);
        }
    }
}
```

---

### IN-03: `checkout.rs` metadata uses `HashMap` (unordered) — acceptable but worth noting

**File:** `ferro-stripe/src/checkout.rs:186-192`

**Issue:** Metadata is stored internally as `Vec<(String, String)>` (ordered, allows duplicates) then converted to `HashMap<String, String>` before sending to Stripe. If a caller adds the same key twice via `.metadata("key", "v1").metadata("key", "v2")`, the second value silently wins (HashMap insert semantics). The builder's internal `Vec` representation suggests the intent was to allow multiple calls, but the HashMap conversion drops duplicates without any error.

**Fix:** Either deduplicate eagerly in the `.metadata()` builder method, or document that duplicate keys have last-write-wins semantics.

---

### IN-04: `webhook/events.rs` — `signed_webhook_payload` is a public function not re-exported from `lib.rs`

**File:** `ferro-stripe/src/webhook/events.rs:235` / `ferro-stripe/src/lib.rs`

**Issue:** `signed_webhook_payload` is a `pub fn` in `ferro_stripe::webhook::events` and is re-exported through `ferro_stripe::testing` (feature-gated). However, `verify.rs` tests access it via `crate::webhook::events::signed_webhook_payload` — that works fine. The concern is that `ferro_stripe::webhook::events` is a fully public module (no feature gate), so `signed_webhook_payload` is unconditionally part of the public API of the crate even though it is only useful as a test helper. Consider moving it behind the `test-helpers` feature or placing it in `testing.rs` only.

**Fix:**
```rust
// In webhook/events.rs, gate the function:
#[cfg(any(test, feature = "test-helpers"))]
pub fn signed_webhook_payload(payload: &str, secret: &str) -> (String, i64) { ... }
```
This prevents the HMAC signing code from being compiled into production binaries unnecessarily.

---

_Reviewed: 2026-04-20_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
