---
phase: 96-stripe-integration
plan: 01
subsystem: payments
tags: [stripe, async-stripe, billing, subscriptions, connect, onceLock]

requires: []

provides:
  - ferro-stripe crate with async-stripe 0.41.x dependency
  - SubscriptionInfo struct with on_trial(), subscribed(), on_grace_period() helpers
  - SubscriptionStatus enum covering all 8 Stripe statuses with snake_case serde
  - plan_satisfies() function implementing enterprise > pro > free hierarchy
  - StripeConfig loading from environment variables
  - Stripe client facade using OnceLock<stripe::Client>
  - Error enum with 5 variants for config, API, webhook, and idempotency errors
  - create_subscription_checkout() and billing_portal_url() async functions
  - subscription_info_from_stripe() mapping stripe::Subscription to SubscriptionInfo
  - create_connect_checkout() with destination charges and optional application fee
  - create_account_link() for Stripe Connect onboarding
  - ConnectAccount struct stub for connect module

affects: [96-02, 96-03, 96-04, 96-05, 96-06, framework-tenant, ferro-cli, ferro-mcp]

tech-stack:
  added:
    - async-stripe 0.41.0 (with billing, checkout, connect, webhook-events features)
  patterns:
    - OnceLock<stripe::Client> static facade matching ferro-notifications CONFIG pattern
    - plan_satisfies() index-based tier comparison (enterprise=2 > pro=1 > free=0)
    - Error enum per crate with thiserror 2.x derives

key-files:
  created:
    - ferro-stripe/Cargo.toml
    - ferro-stripe/src/lib.rs
    - ferro-stripe/src/error.rs
    - ferro-stripe/src/config.rs
    - ferro-stripe/src/client.rs
    - ferro-stripe/src/subscription/mod.rs
    - ferro-stripe/src/subscription/checkout.rs
    - ferro-stripe/src/subscription/sync.rs
    - ferro-stripe/src/connect/mod.rs
    - ferro-stripe/src/connect/checkout.rs
  modified:
    - Cargo.toml (added ferro-stripe to workspace members)

key-decisions:
  - "async-stripe 0.41.x (stable) over 1.0.0-rc.3 (pre-release) — 1.x still anticipates breaking changes"
  - "OnceLock<stripe::Client> facade (Stripe::init + Stripe::client()) matches ferro-notifications pattern"
  - "plan_satisfies() uses index-based comparison: enterprise=2, pro=1, free=0; unknown plans only match themselves"
  - "SubscriptionInfo.cancel_at_period_end: bool — on_grace_period() requires this + subscribed()"
  - "CreateCheckoutSession::new() takes no args in async-stripe 0.41 — success_url/cancel_url are fields"

patterns-established:
  - "Stripe facade: Stripe::init(config) at app startup, Stripe::client() anywhere"
  - "plan_satisfies(tenant_plan, required_plan) for all authorization checks"
  - "Checkout functions call crate::Stripe::client() internally — no client param"

requirements-completed: [STRIPE-01, STRIPE-02, STRIPE-07]

duration: 8min
completed: 2026-03-11
---

# Phase 96 Plan 01: ferro-stripe Foundation Summary

**async-stripe 0.41.x crate with SubscriptionInfo, SubscriptionStatus, plan hierarchy, OnceLock client facade, checkout and Connect checkout functions**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-11T02:54:06Z
- **Completed:** 2026-03-11T03:02:12Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- ferro-stripe crate added to workspace with async-stripe 0.41.0 (billing, checkout, connect, webhook-events features)
- SubscriptionInfo with all 8 SubscriptionStatus variants and on_trial/subscribed/on_grace_period helpers
- plan_satisfies() with enterprise > pro > free hierarchy; unknown plans match only themselves
- Static Stripe client facade via OnceLock — initialized once at app startup, available everywhere
- Subscription checkout, billing portal, Connect checkout, and account link functions compile against real async-stripe API
- 11 unit tests pass covering all helper methods, plan hierarchy edge cases, and JSON serialization

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ferro-stripe crate with core types** - `16ab847` (feat)
2. **Task 2: Add subscription checkout and billing portal functions** - `195f927` (feat)

## Files Created/Modified

- `ferro-stripe/Cargo.toml` — Crate definition with async-stripe 0.41.x and supporting dependencies
- `ferro-stripe/src/lib.rs` — Public API re-exports (Stripe, StripeConfig, Error, SubscriptionInfo, SubscriptionStatus, plan_satisfies, ConnectAccount, all async functions)
- `ferro-stripe/src/error.rs` — Error enum: Config, Stripe, NoConnectAccount, WebhookVerification, EventAlreadyProcessed
- `ferro-stripe/src/config.rs` — StripeConfig with from_env() loading STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET, optional STRIPE_CONNECT_WEBHOOK_SECRET and STRIPE_APPLICATION_FEE_PERCENT
- `ferro-stripe/src/client.rs` — Stripe facade with OnceLock<stripe::Client> and OnceLock<StripeConfig>
- `ferro-stripe/src/subscription/mod.rs` — SubscriptionInfo, SubscriptionStatus, plan_satisfies() with 11 tests
- `ferro-stripe/src/subscription/checkout.rs` — create_subscription_checkout() and billing_portal_url()
- `ferro-stripe/src/subscription/sync.rs` — subscription_info_from_stripe() mapping stripe::Subscription fields
- `ferro-stripe/src/connect/mod.rs` — ConnectAccount struct stub
- `ferro-stripe/src/connect/checkout.rs` — create_connect_checkout() with destination charges, create_account_link()
- `Cargo.toml` — Added ferro-stripe to workspace members list

## Decisions Made

- Used async-stripe 0.41.x stable over 1.0.0-rc.3 (pre-release as of March 2026, per GitHub releases)
- OnceLock<stripe::Client> facade (Stripe::init + Stripe::client()) matches ferro-notifications CONFIG pattern exactly
- plan_satisfies() uses index-based comparison with const slice ["free", "pro", "enterprise"]; unknown plan strings only satisfy themselves via exact match
- CreateCheckoutSession::new() in async-stripe 0.41 takes no arguments — success_url and cancel_url are Option<&str> fields set after construction (differs from documentation examples targeting older versions)
- Deviation Rule 1: Fixed CreateAccountLink initialization — 0.41 API has ::new(account, type_) not Default::default() pattern

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed async-stripe 0.41.x API shape mismatches**
- **Found during:** Task 1 and 2 (initial build)
- **Issue:** Plan described CreateCheckoutSession::new(success_url) but 0.41.x has ::new() with no args; CreateAccountLink has no Default impl; Subscription.current_period_end is i64 not Option<i64>; cancel_at_period_end is bool not Option<bool>
- **Fix:** Corrected all API calls to match actual 0.41.x types verified from the crate source
- **Files modified:** ferro-stripe/src/subscription/checkout.rs, ferro-stripe/src/subscription/sync.rs, ferro-stripe/src/connect/checkout.rs
- **Verification:** cargo build -p ferro-stripe succeeded; all 11 tests pass
- **Committed in:** 195f927 (Task 2 commit)

**2. [Rule 1 - Bug] Fixed clippy field_reassign_with_default in connect/checkout.rs**
- **Found during:** Task 2 (clippy run)
- **Issue:** CreateCheckoutSessionPaymentIntentData was constructed via Default::default() then fields assigned — clippy field-reassign-with-default fires
- **Fix:** Initialized struct directly with all fields in one struct literal expression
- **Files modified:** ferro-stripe/src/connect/checkout.rs
- **Verification:** cargo clippy -p ferro-stripe --all-targets -- -D warnings clean
- **Committed in:** 195f927 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 Rule 1 API shape corrections)
**Impact on plan:** Both fixes necessary for compilation and lint compliance. async-stripe 0.41.x API shape verified directly from crate source.

## Issues Encountered

- The research/plan had async-stripe API shapes based on documentation examples; actual 0.41.0 crate has slightly different constructors (CreateCheckoutSession, CreateAccountLink). Corrected by reading the installed crate source directly.

## User Setup Required

None — no external service configuration required for this plan. Stripe credentials will be needed at runtime (STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET) but are documented in StripeConfig::from_env().

## Next Phase Readiness

- ferro-stripe foundation complete; all types and functions for Plans 02-06 are available
- Plan 02 (webhook handlers) can reference Stripe::client(), StripeConfig, and subscription_info_from_stripe()
- Plan 03 (TenantContext enrichment) can replace plan: Option<String> with subscription: Option<SubscriptionInfo>
- Plan 04 (Connect checkout) can use create_connect_checkout() and create_account_link() directly
- No blockers

---
*Phase: 96-stripe-integration*
*Completed: 2026-03-11*
