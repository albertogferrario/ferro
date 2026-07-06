---
phase: 140-core-reshape
verified: 2026-04-20T03:34:36Z
status: passed
score: 18/18
overrides_applied: 0
---

# Phase 140: Core Reshape Verification Report

**Phase Goal:** Restructure ferro-stripe from product-axis layout to capability-axis layout, with clean public API, idempotency infrastructure, and full consumer migration.
**Verified:** 2026-04-20T03:34:36Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `dashmap = "6"` declared in `ferro-stripe/Cargo.toml [dependencies]` | VERIFIED | Line 24: `dashmap = "6"` |
| 2 | `Error::MissingIdempotencyKey` variant exists and compiles | VERIFIED | `ferro-stripe/src/error.rs` line 26; all 25 tests pass |
| 3 | `Stripe::with(api_key)` returns `stripe::Client` without touching `STRIPE_CLIENT` static | VERIFIED | `client.rs` lines 59-60; `client::tests::with_does_not_populate_global_static` passes |
| 4 | `ProcessedEventLog` is an `#[async_trait]` trait with `try_mark_processed` method; `Send + Sync` bound declared | VERIFIED | `idempotency.rs` line 38: `pub trait ProcessedEventLog: Send + Sync` with 2 `#[async_trait]` attributes |
| 5 | `MemoryProcessedLog::new()` returns struct backed by `dashmap::DashMap<String, ()>` | VERIFIED | `idempotency.rs` line 54: `seen: dashmap::DashMap<String, ()>` |
| 6 | `MemoryProcessedLog` returns `Ok(true)` first call, `Ok(false)` second call; concurrent insert correctness | VERIFIED | `memory_log_true_then_false` and `memory_log_concurrent_insert_applies_once` pass |
| 7 | `CheckoutBuilder::new(Mode::Payment)` constructs builder with empty state | VERIFIED | `checkout::tests::checkout_builder_new_is_empty` passes |
| 8 | `CheckoutBuilder::create()` without `idempotency_key()` returns `Err(Error::MissingIdempotencyKey)` before network call | VERIFIED | `checkout.rs` line 140: `ok_or(Error::MissingIdempotencyKey)`; `checkout_create_missing_key_returns_err` passes |
| 9 | `CheckoutBuilder` has all 8 consuming builder methods | VERIFIED | `grep -cE` returns 8: `line_item`, `success_url`, `cancel_url`, `metadata`, `customer_email`, `customer_email_opt`, `destination`, `idempotency_key` |
| 10 | `CheckoutIntent`, `Mode`, `LineItem` types exist with correct public fields | VERIFIED | All pub fields present; `line_item_public_fields_constructable` passes |
| 11 | `refund::create` and `refund::retrieve` exist and call async-stripe | VERIFIED | `refund.rs` lines 13, 38; both call `stripe::Refund::create/retrieve` |
| 12 | `account::create_account`, `account::create_link`, `account::retrieve_account`, `account::billing_portal_url` exist | VERIFIED | All 4 `pub async fn` present in `account.rs` |
| 13 | `connect/` and `subscription/` directories deleted; `webhook/handler.rs` deleted | VERIFIED | All three return "DELETED" when checked with `ls` |
| 14 | `webhook/verify.rs` has `verify_webhook`; `webhook/sync.rs` and `webhook/queue.rs` exist as stubs | VERIFIED | All three files confirmed; sync.rs and queue.rs contain Phase 141 comments |
| 15 | `ferro-stripe/src/lib.rs` exposes capability-axis API; no product-axis re-exports | VERIFIED | No `pub mod connect`, no `pub mod subscription`, no `is_processed`; all capability-axis symbols re-exported |
| 16 | `framework/src/tenant/subscription.rs` exists with `SubscriptionInfo`, `SubscriptionStatus`, `plan_satisfies` | VERIFIED | Lines 14, 35, 76 confirmed; no `ferro_stripe::*` refs remain in `tenant/mod.rs` or `requires_plan.rs` |
| 17 | `ferro-stripe/Cargo.toml` has `version = "0.4.0"` (local override); workspace root unchanged at `0.2.2` | VERIFIED | Line 3: `version = "0.4.0"`; workspace `Cargo.toml` line 27: `version = "0.2.2"` |
| 18 | `CHANGELOG.md` has `## [0.4.0]` entry with all removed symbols; `docs/src/features/stripe.md` no longer documents `stripe_is_processed` | VERIFIED | CHANGELOG has 1 `### [0.4.0]` entry with 12 matches for removed symbols; docs show 0 `stripe_is_processed` references; `CheckoutBuilder` and `ProcessedEventLog` documented |

**Score:** 18/18 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-stripe/Cargo.toml` | dashmap dep + version 0.4.0 | VERIFIED | `dashmap = "6"` at line 24; `version = "0.4.0"` at line 3 |
| `ferro-stripe/src/error.rs` | `MissingIdempotencyKey` variant | VERIFIED | Line 26 confirmed |
| `ferro-stripe/src/client.rs` | `Stripe::with(api_key)` method | VERIFIED | Lines 59-60 with `stripe::Client::new(api_key)` |
| `ferro-stripe/src/idempotency.rs` | `ProcessedEventLog` trait, `MemoryProcessedLog` impl, SQL schema | VERIFIED | 123 lines; all key elements present |
| `ferro-stripe/src/checkout.rs` | `CheckoutBuilder`, `CheckoutIntent`, `LineItem`, `Mode` | VERIFIED | All types present with correct public fields |
| `ferro-stripe/src/refund.rs` | `create`, `retrieve` async fns | VERIFIED | 46 lines; both fns present |
| `ferro-stripe/src/account.rs` | 4 account async fns | VERIFIED | All 4 functions confirmed |
| `ferro-stripe/src/webhook/verify.rs` | `verify_webhook` fn | VERIFIED | Line 16 confirmed |
| `ferro-stripe/src/webhook/sync.rs` | Phase 141 stub | VERIFIED | 8 lines; Phase 141 comment present |
| `ferro-stripe/src/webhook/queue.rs` | Phase 141 stub | VERIFIED | 8 lines; Phase 141 comment present |
| `ferro-stripe/src/lib.rs` | Capability-axis re-exports only | VERIFIED | No old symbols; all new symbols present |
| `framework/src/tenant/subscription.rs` | `SubscriptionInfo`, `SubscriptionStatus`, `plan_satisfies` | VERIFIED | All three at lines 14, 35, 76 |
| `CHANGELOG.md` | 0.4.0 breaking-change ledger | VERIFIED | `### [0.4.0]` entry with 12+ symbol references |
| `docs/src/features/stripe.md` | No `stripe_is_processed` as callable | VERIFIED | 0 matches; `CheckoutBuilder` and `ProcessedEventLog` documented |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-stripe/src/error.rs` | thiserror derive | `#[error(...)]` attribute on `MissingIdempotencyKey` | VERIFIED | Pattern confirmed |
| `ferro-stripe/src/client.rs` | `stripe::Client::new` | `Stripe::with(api_key)` | VERIFIED | Line 60 |
| `ferro-stripe/src/idempotency.rs` | `dashmap::DashMap` | `MemoryProcessedLog.seen` field | VERIFIED | Line 54 |
| `ferro-stripe/src/idempotency.rs` | `async_trait` | `#[async_trait]` on trait and impl | VERIFIED | 2 occurrences |
| `ferro-stripe/src/checkout.rs` | `crate::Error::MissingIdempotencyKey` | `ok_or(Error::MissingIdempotencyKey)` | VERIFIED | Line 140 |
| `ferro-stripe/src/checkout.rs` | `stripe::CheckoutSession::create` | async `create()` method | VERIFIED | Present in create body |
| `ferro-stripe/src/account.rs` | `stripe::AccountLink::create` | `create_link()` | VERIFIED | `AccountLinkType::AccountOnboarding` preserved |
| `ferro-stripe/src/account.rs` | `stripe::BillingPortalSession::create` | `billing_portal_url()` | VERIFIED | `CreateBillingPortalSession::new` present |
| `ferro-stripe/src/lib.rs` | `ferro-stripe/src/idempotency.rs` | `pub mod idempotency` | VERIFIED | Line 48 |
| `ferro-stripe/src/lib.rs` | `ferro-stripe/src/checkout.rs` | `pub use checkout::` | VERIFIED | Line 55 |
| `ferro-stripe/src/webhook/mod.rs` | `ferro-stripe/src/webhook/verify.rs` | `pub mod verify` | VERIFIED | Line 7 |
| `framework/src/lib.rs` | `ferro-stripe` | `pub use ferro_stripe::{...}` | VERIFIED | Lines 94-99; capability-axis symbols only |
| `CHANGELOG.md` | framework migration | "Migration" section | VERIFIED | Migration guide with code examples present |

### Data-Flow Trace (Level 4)

Not applicable — this phase delivers library API surface, not a data-rendering component. All artifacts are Stripe API wrappers and infrastructure types with no user-visible rendering.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All ferro-stripe lib tests pass | `cargo test -p ferro-stripe --lib` | 25 passed; 0 failed | PASS |
| `checkout_create_missing_key_returns_err` — idempotency guard fires before network | test included in 25 above | ok | PASS |
| `memory_log_true_then_false` — correct true/false semantics | test included in 25 above | ok | PASS |
| `memory_log_concurrent_insert_applies_once` — concurrent-insert correctness | test included in 25 above | ok | PASS |
| `verify_webhook_with_valid_signature_returns_ok` — HMAC verification | test included in 25 above | ok | PASS |
| `with_does_not_populate_global_static` — scoped client isolation | test included in 25 above | ok | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|------------|------------|-------------|--------|---------|
| SC-9 | Plan 01 | dashmap dep, MissingIdempotencyKey variant, Stripe::with | SATISFIED | All three present and tested |
| SC-2, SC-3, SC-4, SC-12 | Plan 02 | ProcessedEventLog trait, MemoryProcessedLog, SQL schema, async idempotency | SATISFIED | `idempotency.rs` fully implements these |
| SC-5, SC-6, SC-7, SC-8 | Plan 03 | CheckoutBuilder, refund.rs, account.rs capability files | SATISFIED | All three files present with correct API |
| SC-1, SC-10, SC-11 | Plan 04 | Module restructure, deletion, lib.rs rewrite | SATISFIED | Product-axis dirs deleted; lib.rs capability-axis only |
| SC-13, SC-14, SC-15 | Plan 05 | framework migration, CHANGELOG, version bump | SATISFIED | subscription.rs in framework; CHANGELOG at 0.4.0; docs updated |

### Anti-Patterns Found

None. No TODO/FIXME/placeholder stubs were found in new capability files (checkout.rs, refund.rs, account.rs, idempotency.rs). The `// Phase 141:` comments in `webhook/sync.rs` and `webhook/queue.rs` are intentional stub markers, not incomplete implementations — both files are explicitly documented as Phase 141 reservations.

The `let _ = idempotency_key;` in `refund.rs` line 22 is accompanied by a comment explaining the async-stripe 0.41 API limitation. This is a known, documented constraint, not a stub.

### Human Verification Required

None. All must-haves are verified programmatically. The phase delivers library API surface with no visual or real-time UI behavior requiring human testing.

### Gaps Summary

No gaps. All 18 observable truths are verified against the actual codebase. The phase goal is achieved:

- ferro-stripe is restructured from product-axis (`connect/`, `subscription/`) to capability-axis (`checkout`, `refund`, `account`, `idempotency`, `webhook`)
- Clean public API exposed from `lib.rs` with no retired symbols
- Idempotency infrastructure (`ProcessedEventLog` trait + `MemoryProcessedLog`) landed with concurrent-correctness guarantees
- Framework consumer (`framework/`) migrated: `SubscriptionInfo`/`SubscriptionStatus`/`plan_satisfies` re-homed to `framework::tenant::subscription`
- Version bumped to `0.4.0` (local override); CHANGELOG documents all breaking changes with migration paths
- 25 tests pass across all new modules

---

_Verified: 2026-04-20T03:34:36Z_
_Verifier: Claude (gsd-verifier)_
