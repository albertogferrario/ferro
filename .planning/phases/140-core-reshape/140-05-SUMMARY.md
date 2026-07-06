---
phase: 140
plan: 05
subsystem: framework, ferro-stripe, docs
tags: [stripe, framework, changelog, version-bump, migration]
requirements: [SC-13, SC-14, SC-15]

dependency_graph:
  requires: [01, 02, 03, 04]
  provides: [workspace-green, ferro-stripe-0.4.0, subscription-types-in-framework, changelog]
  affects:
    - framework/src/lib.rs
    - framework/src/tenant/mod.rs
    - framework/src/tenant/requires_plan.rs
    - framework/src/tenant/subscription.rs
    - ferro-stripe/Cargo.toml
    - framework/Cargo.toml
    - CHANGELOG.md
    - docs/src/features/stripe.md

tech_stack:
  added: []
  patterns: [framework-local subscription types, capability-axis re-exports]

key_files:
  created:
    - framework/src/tenant/subscription.rs
    - CHANGELOG.md
  modified:
    - framework/src/lib.rs
    - framework/src/tenant/mod.rs
    - framework/src/tenant/requires_plan.rs
    - ferro-stripe/Cargo.toml
    - framework/Cargo.toml
    - docs/src/features/stripe.md

decisions:
  - "Option B chosen: SubscriptionInfo/SubscriptionStatus/plan_satisfies moved to framework::tenant::subscription (not re-added to ferro-stripe, not deleted)"
  - "framework/Cargo.toml ferro-stripe dep bumped from 0.2 to 0.4 (required by version conflict)"
  - "ferry fmt import reordering applied to requires_plan.rs tests block"

metrics:
  duration: ~40min
  completed: 2026-04-20
  tasks: 2
  files: 8
---

# Phase 140 Plan 05: Consumer Migration, Version Bump, Changelog Summary

Closes the loop on Phase 140: migrated all in-workspace consumers of ferro-stripe off retired product-axis symbols, re-homed subscription types to `framework::tenant::subscription`, bumped ferro-stripe to 0.4.0, wrote the CHANGELOG breaking-change ledger, and updated `docs/src/features/stripe.md` to the capability-axis API. `cargo build --all --all-features` is green.

## Design Decision: Option B Selected

The objective presented three options for handling `SubscriptionInfo`, `SubscriptionStatus`, and `plan_satisfies` after their removal from ferro-stripe:

- **(A)** Re-add them to ferro-stripe under a new file — contradicts capability-axis spirit
- **(B) Move to `framework::tenant::subscription`** — preserves clean-up, keeps plan-hierarchy logic framework-side
- **(C)** Delete entirely — too aggressive, breaks framework tests

**Option B was chosen** as recommended. These are app-state data shapes (tenant subscription state), not Stripe API wrappers. Moving them to `framework::tenant::subscription` correctly reflects their domain: they describe what a tenant's billing state looks like, not what the Stripe API returns.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Move SubscriptionInfo/Status/plan_satisfies to framework; fix tenant mod and requires_plan | 37909486 | framework/src/tenant/subscription.rs (new), mod.rs, requires_plan.rs, lib.rs |
| 2 | Bump ferro-stripe version, write CHANGELOG, update docs/stripe.md | 4d568b79 | ferro-stripe/Cargo.toml, framework/Cargo.toml, CHANGELOG.md, docs/src/features/stripe.md |

## Final `framework/src/lib.rs` ferro-stripe re-export block

```rust
#[cfg(feature = "stripe")]
pub use ferro_stripe::{
    account, checkout, refund, verify_webhook, CheckoutBuilder, CheckoutIntent,
    Error as StripeError, LineItem, MemoryProcessedLog, Mode, ProcessStripeWebhook,
    ProcessedEventLog, Stripe, StripeCheckoutCompleted, StripeConfig,
    StripeConnectPaymentSucceeded, StripeInvoicePaid, StripeSubscriptionDeleted,
    StripeSubscriptionUpdated,
};
```

- `account`, `checkout`, `refund` re-exported as modules so downstream can write `ferro::account::billing_portal_url(...)`, `ferro::refund::create(...)`.
- All retired product-axis symbols absent: `create_connect_checkout`, `create_subscription_checkout`, `create_account_link`, `is_processed`/`stripe_is_processed`, `plan_from_subscription`, `subscription_info_from_stripe`, `plan_satisfies`, `SubscriptionInfo`, `SubscriptionStatus`, `ConnectAccount`.

## docs/src/features/stripe.md Changes

Lines updated (approximate, after edits):
- **Lines ~68-87**: Subscription checkout example replaced with `CheckoutBuilder::new(Mode::Subscription)` pattern
- **Lines ~93-109**: Billing portal example replaced with `account::billing_portal_url`
- **Lines ~163-170**: Plan hierarchy example updated to `ferro::tenant::subscription::plan_satisfies`
- **Lines ~179-195**: Connect onboarding example replaced with `account::create_link`
- **Lines ~202-224**: Destination charges example replaced with `CheckoutBuilder::new(Mode::Payment).destination(...)`
- **Lines ~303**: Idempotency section: removed `stripe_is_processed` as callable, replaced with `ProcessedEventLog::try_mark_processed` guidance and code example
- **Lines ~310-321**: TenantContext enrichment: updated import to `ferro::tenant::subscription::SubscriptionInfo`
- **Lines ~370-384**: Mock subscriptions section: removed references to deleted `mock_subscription_*` helpers, replaced with direct `SubscriptionInfo` construction

## Test Results

```
cargo test --all-features: 2360 passed; 0 failed; 0 ignored
cargo fmt --all -- --check: exit 0
cargo clippy --all --all-targets -- -D warnings: exit 0
cargo build --all --all-features: exit 0
```

## cargo publish --dry-run

```
cargo publish -p ferro-stripe --dry-run --allow-dirty
   Compiling ferro-stripe v0.4.0 (...)
    Finished `dev` profile
   Uploading ferro-stripe v0.4.0 (...)
warning: aborting upload due to dry run
```

Exit 0. Manifest is valid for publish.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] framework/Cargo.toml had `ferro-stripe = "^0.2"` dependency constraint**
- **Found during:** Task 2 — `cargo clippy` failed with "failed to select a version for the requirement `ferro-stripe = "^0.2"`, candidate versions found which didn't match: 0.4.0"
- **Issue:** When `ferro-stripe/Cargo.toml` was bumped to `0.4.0`, the `framework/Cargo.toml` dependency constraint `version = "0.2"` no longer matched. This is a workspace-internal path dependency where version constraints still apply.
- **Fix:** Updated `framework/Cargo.toml` line 46 from `version = "0.2"` to `version = "0.4"`.
- **Files modified:** `framework/Cargo.toml`
- **Commit:** 4d568b79

**2. [Rule 1 - Bug] cargo fmt reordered test imports in requires_plan.rs**
- **Found during:** Task 2 CI gate — `cargo fmt --all -- --check` flagged import ordering in the test block
- **Issue:** The new `use crate::tenant::subscription::{...}` import was inserted before `use crate::tenant::TenantContext;` but fmt expects alphabetical ordering
- **Fix:** `cargo fmt --all` applied automatically; the change was included in the Task 2 commit.
- **Files modified:** `framework/src/tenant/requires_plan.rs`
- **Commit:** 4d568b79

## Known Stubs

None introduced by this plan. The `webhook/sync.rs` and `webhook/queue.rs` stubs from plan 04 are documented in 140-04-SUMMARY.md.

## Threat Flags

None. No new network endpoints, auth paths, or trust boundaries introduced. This plan migrates types and updates documentation.

## Self-Check

- [x] `framework/src/tenant/subscription.rs` exists: FOUND
- [x] `pub enum SubscriptionStatus` in subscription.rs: count=1
- [x] `pub struct SubscriptionInfo` in subscription.rs: count=1
- [x] `pub fn plan_satisfies` in subscription.rs: count=1
- [x] No `ferro_stripe::SubscriptionInfo` in tenant/mod.rs: count=0
- [x] No `ferro_stripe::SubscriptionStatus` in tenant/mod.rs: count=0
- [x] No `ferro_stripe::plan_satisfies` in requires_plan.rs: count=0
- [x] No `is_processed as stripe_is_processed` in lib.rs: count=0
- [x] No `create_connect_checkout` etc. in lib.rs: count=0
- [x] `CheckoutBuilder` in lib.rs: count=1
- [x] `MemoryProcessedLog`+`ProcessedEventLog` in lib.rs: count=2
- [x] `ferro-stripe/Cargo.toml` has `version = "0.4.0"`: count=1
- [x] `Cargo.toml` (workspace) still has `version = "0.2.2"`: count=1
- [x] `CHANGELOG.md` exists with `### [0.4.0]` and all removed symbols: FOUND
- [x] No `stripe_is_processed` in docs/stripe.md: count=0
- [x] Commit 37909486 (Task 1) present: FOUND
- [x] Commit 4d568b79 (Task 2) present: FOUND
- [x] `cargo build --all --all-features` exits 0
- [x] `cargo test --all-features` exits 0 (2360 passed)
- [x] `cargo fmt --all -- --check` exits 0
- [x] `cargo clippy --all --all-targets -- -D warnings` exits 0
- [x] `cargo publish -p ferro-stripe --dry-run --allow-dirty` exits 0

## Self-Check: PASSED
