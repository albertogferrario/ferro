---
phase: 140
plan: 04
subsystem: ferro-stripe
tags: [stripe, module-restructure, deletion, lib-rewrite]
requirements: [SC-1, SC-10, SC-11]

dependency_graph:
  requires: [01, 02, 03]
  provides: [capability-axis-lib, webhook/verify.rs, webhook/sync.rs, webhook/queue.rs]
  affects:
    - ferro-stripe/src/lib.rs
    - ferro-stripe/src/webhook/mod.rs
    - ferro-stripe/src/webhook/verify.rs
    - ferro-stripe/src/webhook/sync.rs
    - ferro-stripe/src/webhook/queue.rs
    - ferro-stripe/src/testing.rs
    - ferro-stripe/src/idempotency.rs

tech_stack:
  added: []
  patterns: [capability-axis lib.rs, webhook extraction, product-axis deletion]

key_files:
  created:
    - ferro-stripe/src/webhook/verify.rs
    - ferro-stripe/src/webhook/sync.rs
    - ferro-stripe/src/webhook/queue.rs
  modified:
    - ferro-stripe/src/lib.rs
    - ferro-stripe/src/webhook/mod.rs
    - ferro-stripe/src/testing.rs
    - ferro-stripe/src/idempotency.rs
  deleted:
    - ferro-stripe/src/connect/checkout.rs
    - ferro-stripe/src/connect/mod.rs
    - ferro-stripe/src/subscription/checkout.rs
    - ferro-stripe/src/subscription/mod.rs
    - ferro-stripe/src/subscription/sync.rs
    - ferro-stripe/src/webhook/handler.rs

decisions:
  - "testing.rs subscription mocks removed entirely — SubscriptionInfo/SubscriptionStatus no longer exist in ferro-stripe"
  - "idempotency.rs bool-assert-comparison clippy lint fixed (assert! not assert_eq! with literal bool)"
  - "workspace build --all fails only in framework/src/lib.rs lines 94-100 (plan 05 scope)"
  - "verify_webhook re-exported via webhook::verify::verify_webhook in lib.rs (not webhook:: directly)"

metrics:
  duration: ~25min
  completed: 2026-04-20
  tasks: 3
  files: 10
---

# Phase 140 Plan 04: Module Restructure (lib.rs rewrite + deletion + webhook extraction) Summary

Module restructure completing the capability-axis pivot for ferro-stripe: product-axis directories (`connect/`, `subscription/`) and orphaned `webhook/handler.rs` deleted; `verify_webhook` extracted to `webhook/verify.rs`; `webhook/sync.rs` and `webhook/queue.rs` stubs created; `webhook/mod.rs` rewritten as a thin shim; and `ferro-stripe/src/lib.rs` rewritten to expose the capability-axis public API.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extract webhook/verify.rs and add sync.rs, queue.rs stubs | b0934433 | ferro-stripe/src/webhook/verify.rs, sync.rs, queue.rs |
| 2 | Delete product-axis dirs and rewrite webhook/mod.rs | 5cde4f53 | webhook/mod.rs; delete connect/, subscription/, handler.rs |
| 3 | Rewrite ferro-stripe/src/lib.rs with capability-axis API | 5c5452a0 | lib.rs, testing.rs, idempotency.rs |

## Final lib.rs Diff Summary

### Symbols removed

| Symbol | Old path | Replacement |
|--------|----------|-------------|
| `create_connect_checkout` | `connect::checkout` | deleted (no replacement in ferro-stripe) |
| `create_account_link` | `connect::checkout` | `account::create_link` (same fn, new path) |
| `ConnectAccount` | `connect` | deleted |
| `create_subscription_checkout` | `subscription::checkout` | deleted |
| `billing_portal_url` | `subscription::checkout` | `account::billing_portal_url` (same fn, new path) |
| `plan_from_subscription` | `subscription::sync` | deleted |
| `subscription_info_from_stripe` | `subscription::sync` | deleted |
| `plan_satisfies` | `subscription` | deleted |
| `SubscriptionInfo` | `subscription` | deleted |
| `SubscriptionStatus` | `subscription` | deleted |
| `is_processed` | `webhook` | deleted (replaced by `ProcessedEventLog` trait in `idempotency`) |

### Symbols added

| Symbol | Path |
|--------|------|
| `CheckoutBuilder` | `pub use checkout::CheckoutBuilder` |
| `CheckoutIntent` | `pub use checkout::CheckoutIntent` |
| `LineItem` | `pub use checkout::LineItem` |
| `Mode` | `pub use checkout::Mode` |
| `MemoryProcessedLog` | `pub use idempotency::MemoryProcessedLog` |
| `ProcessedEventLog` | `pub use idempotency::ProcessedEventLog` |
| `create_account` | `pub use account::create_account` |
| `create_link` | `pub use account::create_link` |
| `retrieve_account` | `pub use account::retrieve_account` |
| `billing_portal_url` | `pub use account::billing_portal_url` |
| `verify_webhook` | `pub use webhook::verify::verify_webhook` |

### Symbols retained (same re-export, same path)

- `Stripe` (client)
- `StripeConfig` (config)
- `Error` (error)
- All 5 event structs + `ProcessStripeWebhook` (webhook::events)

## Directory Structure After Plan 04

```
ferro-stripe/src/
  account.rs
  checkout.rs
  client.rs
  config.rs
  error.rs
  idempotency.rs
  lib.rs
  refund.rs
  testing.rs
  webhook/
    events.rs
    mod.rs
    queue.rs
    sync.rs
    verify.rs
```

`connect/` and `subscription/` directories are gone. `webhook/handler.rs` is gone.

## Test Results

```
cargo test -p ferro-stripe (before disk-full event):
  25 passed; 0 failed; 0 ignored
  (idempotency: 2, events: 8, verify: 3, testing: 4, client: 2, checkout: 3, config: 3)

cargo fmt -p ferro-stripe -- --check  → clean (exit 0)
cargo clippy -p ferro-stripe --all-targets -- -D warnings → clean (exit 0)
cargo build -p ferro-stripe → Finished (exit 0)
```

Note: A disk-full condition (/dev/disk3s5 at 100%) prevented running `cargo build --all` for the workspace failure confirmation step. The ferro-stripe crate itself builds, tests, and lints clean. Framework breakage in `framework/src/lib.rs` lines 94-100 is confirmed by source inspection — those lines still re-export deleted symbols (`is_processed`, `create_connect_checkout`, `SubscriptionInfo`, etc.) which plan 05 removes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] testing.rs imported deleted SubscriptionInfo/SubscriptionStatus**
- **Found during:** Task 3 — `cargo test -p ferro-stripe` failed with `unresolved import crate::subscription`
- **Issue:** `ferro-stripe/src/testing.rs` used `use crate::subscription::{SubscriptionInfo, SubscriptionStatus}` and 6 mock factory functions that construct `SubscriptionInfo` structs. The `subscription` module was deleted in Task 2.
- **Fix:** Rewrote `testing.rs` removing all 6 subscription mock fns (`mock_subscription_active`, `mock_subscription_trialing`, `mock_subscription_canceled`, `mock_subscription_past_due`, `mock_subscription_on_grace`, `mock_subscription_with_connect`) and their 6 corresponding tests. Retained the 4 event mock fns and 4 tests, plus the `signed_webhook_payload` re-export.
- **Files modified:** ferro-stripe/src/testing.rs
- **Commit:** 5c5452a0

**2. [Rule 1 - Bug] idempotency.rs had clippy bool-assert-comparison lint**
- **Found during:** Task 3 — `cargo clippy -p ferro-stripe --all-targets -- -D warnings` failed with 3 `bool_assert_comparison` errors in idempotency.rs tests
- **Issue:** Three `assert_eq!(expr, true/false, ...)` calls — clippy with `-D warnings` treats this as an error.
- **Fix:** Replaced with `assert!(expr, ...)` and `assert!(!expr, ...)` patterns.
- **Files modified:** ferro-stripe/src/idempotency.rs
- **Commit:** 5c5452a0

## Known Stubs

- `ferro-stripe/src/webhook/sync.rs`: 7-line stub — Phase 141 lands `SyncDispatcher` here
- `ferro-stripe/src/webhook/queue.rs`: 7-line stub — Phase 141 relocates `ProcessStripeWebhook` here

## Threat Flags

None. No new network endpoints, auth paths, or trust boundaries introduced. This plan only restructures existing modules.

## Self-Check

- [x] `ferro-stripe/src/webhook/verify.rs` exists: FOUND (Glob confirmed)
- [x] `ferro-stripe/src/webhook/sync.rs` exists: FOUND (Glob confirmed)
- [x] `ferro-stripe/src/webhook/queue.rs` exists: FOUND (Glob confirmed)
- [x] `connect/` and `subscription/` directories deleted: confirmed via Glob (absent from listing)
- [x] `webhook/handler.rs` deleted: confirmed via Glob (absent from listing)
- [x] lib.rs has no `pub mod connect`, `pub mod subscription`, `is_processed`: grep returned 0
- [x] lib.rs has all required `pub mod` declarations and re-exports: grep returned 1 for each
- [x] All 6 event structs present in lib.rs: confirmed via grep -oE count = 6
- [x] Commit b0934433 (Task 1) present
- [x] Commit 5cde4f53 (Task 2) present
- [x] Commit 5c5452a0 (Task 3) present
- [x] `cargo build -p ferro-stripe` exits 0
- [x] `cargo test -p ferro-stripe` — 25 passed (before disk-full event)
- [x] `cargo fmt -p ferro-stripe -- --check` exits 0
- [x] `cargo clippy -p ferro-stripe --all-targets -- -D warnings` exits 0

## Self-Check: PASSED
