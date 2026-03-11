# Phase 96: Stripe Integration - Research

**Researched:** 2026-03-11
**Domain:** Stripe payments, billing subscriptions, Stripe Connect, Rust async-stripe crate
**Confidence:** HIGH (core library and Stripe API well-documented; specific 1.x RC API surface requires care)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Billing Scope:**
- Two-tier billing model: platform subscriptions (gestiscilo.it charges tenants) + Stripe Connect (tenants charge their end users)
- Platform subscriptions use fixed plan tiers (Free/Pro/Enterprise) — maps to TenantContext.plan
- End-user payments via tenant's connected Stripe account are one-time charges only (no end-user subscriptions)
- Stripe Checkout Sessions for payment collection (hosted page, zero PCI scope)
- Stripe Billing Portal redirect for tenant self-service subscription management
- Full subscription lifecycle: trial periods, grace periods on cancel, pause/resume
- Optional platform application fee on Connect transactions

**Webhook Handling:**
- Stripe events dispatched through ferro-events (dispatch_event pattern)
- Framework auto-handles core events: subscription.updated/deleted syncs TenantContext.plan automatically
- Two separate webhook endpoints: /stripe/webhook (platform) and /stripe/connect/webhook (connected accounts), each with its own signing secret
- All webhook processing queued via ferro-queue — verify signature inline, ack 200 immediately, process asynchronously

**Tenant-Billing Link:**
- TenantContext enriched with subscription details: subscription_status, trial_ends_at, on_grace_period, plus helper methods (tenant.on_trial(), tenant.subscribed())
- RequiresPlan("pro") middleware for plan-based route access control (like Auth middleware but for billing)
- Immediate restriction on subscription lapse — no grace period after cancellation/past-due, access downgrades instantly
- Phase 95's TenantContext.plan (currently Option<String>) gets replaced with rich subscription struct

**Developer Surface:**
- New `ferro-stripe` crate following ferro-cache/ferro-queue pattern, feature-gated re-export from framework
- `ferro make:stripe` CLI command scaffolds full integration: webhook routes, event listeners, migrations, env config, Connect setup
- Uses stripe-rust (async-stripe) SDK for type-safe Stripe API bindings
- MCP introspection tools: stripe config status, webhook event listing, subscription info
- Test helpers: mock webhook events, verify subscription state, fake Stripe responses
- Full documentation in docs/src/features/stripe.md

### Claude's Discretion
- API facade design: Stripe:: facade vs trait on TenantContext (evaluate which matches existing Ferro patterns better)
- Connect onboarding flow depth: full end-to-end helpers vs API wrappers only
- Storage approach: columns on tenant table vs separate billing table (evaluate trade-offs)
- Connect account ID placement: in TenantContext vs on-demand query (evaluate per-request frequency)
- Cache/TTL strategy for subscription data in TenantContext lookups

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

## Summary

Phase 96 adds a `ferro-stripe` crate that integrates Stripe's payment infrastructure into the Ferro framework. The integration covers two billing dimensions: platform SaaS subscriptions (the platform charges tenants for plan tiers) and Stripe Connect (tenants collect payments from their end users). Both are well-supported by the `async-stripe` ecosystem, which has a modular crate structure reaching `1.0.0-rc.3` as of March 2026.

The core technical challenge is the TenantContext enrichment — `plan: Option<String>` becomes a richer `SubscriptionInfo` struct that carries status, trial state, and helper methods. Webhook processing follows the established ferro-queue pattern (verify signature inline, ack 200, process async), and the `RequiresPlan` middleware mirrors the existing `AuthMiddleware` pattern exactly. The CLI scaffolding command follows the `make:auth` approach, generating migrations, event listeners, and route stubs.

The two most careful design decisions are: (1) storage layout for billing data (columns on the tenant table vs. a separate `tenant_subscriptions` table), and (2) whether Connect account IDs live in TenantContext or are queried on demand. Both are in Claude's discretion scope.

**Primary recommendation:** Use `async-stripe` 0.41.x (the stable release train; `1.0.0-rc.3` is pre-release and has known breaking changes ahead). Add webhook verification via `async-stripe`'s built-in `Webhook::construct_event`. Store subscription state in a separate `tenant_billing` table (clean separation, does not bloat the tenant table). Keep Connect account ID in TenantContext for zero-overhead plan-gate middleware.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| async-stripe | 0.41.x | Stripe API bindings — Checkout, Billing, Connect | Official community Rust SDK; type-safe generated from Stripe OpenAPI; maintained with weekly CI updates |
| async-stripe-webhook | 0.41.x or separate crate | Webhook signature verification (HMAC-SHA256) | Bundled crypto verification; avoids hand-rolling HMAC |
| thiserror | 2.0 | Error types for ferro-stripe | Workspace convention |
| reqwest | 0.12 | Not needed if using async-stripe (it uses hyper internally) | Already in workspace via ferro-notifications |
| serde / serde_json | 1.x | Serialization of billing structs | Workspace dep |
| chrono | 0.4 | Timestamps for trial_ends_at, period_end | Workspace dep |

### Async-stripe Feature Flags

For `0.41.x`, features are on the monorepo crate. Select only what is needed:

```toml
[dependencies]
async-stripe = { version = "0.41", default-features = false, features = [
    "runtime-tokio-hyper",  # tokio + native-tls (matches workspace runtime)
    "billing",              # subscriptions, invoices, billing portal
    "checkout",             # Checkout Sessions
    "connect",              # Connect accounts, account links, transfers
    "webhook-events",       # Webhook::construct_event + event types
] }
```

Note: `1.0.0-rc.3` splits into per-domain crates (`stripe-billing`, `stripe-checkout`, etc.) but is pre-release with known breaking changes. Use `0.41.x` stable.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| async-stripe 0.41.x | async-stripe 1.0.0-rc.3 | RC3 is pre-release with anticipated breaking changes; 0.41.x is stable and feature-complete for this phase |
| separate ferro-stripe crate | inline in framework | Crate separation is established pattern (ferro-cache, ferro-queue); keeps framework dep tree clean |
| columns on tenant table | separate tenant_billing table | Separate table avoids schema coupling — see Architecture Patterns for decision guidance |

**Installation:**
```bash
# ferro-stripe/Cargo.toml (new crate)
# async-stripe with selective features — no npm equivalent, add to Cargo.toml
```

---

## Architecture Patterns

### Recommended Project Structure

```
ferro-stripe/
├── Cargo.toml
└── src/
    ├── lib.rs               # Public API: re-exports, StripeConfig
    ├── client.rs            # Stripe::new(client) facade — wraps async-stripe Client
    ├── config.rs            # StripeConfig (api_key, webhook_secret, connect_webhook_secret)
    ├── error.rs             # Error enum (thiserror)
    ├── subscription/
    │   ├── mod.rs           # SubscriptionInfo struct, helper methods
    │   ├── checkout.rs      # create_checkout_session(), billing_portal_url()
    │   └── sync.rs          # sync_subscription_from_event() — updates DB from webhook
    ├── connect/
    │   ├── mod.rs           # ConnectAccount, create_account_link()
    │   └── checkout.rs      # create_connect_checkout_session() with application_fee_amount
    ├── webhook/
    │   ├── mod.rs           # StripeWebhookHandler — verify + dispatch
    │   └── events.rs        # StripeEvent wrappers implementing ferro-events Event trait
    └── middleware/
        └── requires_plan.rs # RequiresPlan middleware
```

### Pattern 1: SubscriptionInfo in TenantContext

**What:** Replace `plan: Option<String>` with `subscription: Option<SubscriptionInfo>` where `SubscriptionInfo` carries full billing state.
**When to use:** Every request behind TenantMiddleware — data loaded alongside tenant DB lookup.

```rust
// Source: framework/src/tenant/mod.rs pattern extended
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubscriptionInfo {
    /// Stripe subscription ID (sub_xxx)
    pub stripe_subscription_id: String,
    /// Plan identifier: "free" | "pro" | "enterprise"
    pub plan: String,
    /// Stripe subscription status
    pub status: SubscriptionStatus,
    /// When the trial ends (None if not on trial)
    pub trial_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    /// True when subscription is canceled but period hasn't ended yet
    pub cancel_at_period_end: bool,
    /// When the current billing period ends
    pub current_period_end: chrono::DateTime<chrono::Utc>,
    /// Stripe Connect account ID for this tenant (None if not connected)
    pub stripe_connect_account_id: Option<String>,
}

impl SubscriptionInfo {
    pub fn on_trial(&self) -> bool {
        self.status == SubscriptionStatus::Trialing
    }

    pub fn subscribed(&self) -> bool {
        matches!(self.status, SubscriptionStatus::Active | SubscriptionStatus::Trialing)
    }

    pub fn on_grace_period(&self) -> bool {
        // cancel_at_period_end=true + still active = scheduled to cancel
        self.cancel_at_period_end && self.subscribed()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Trialing,
    Active,
    Incomplete,
    IncompleteExpired,
    PastDue,
    Canceled,
    Unpaid,
    Paused,
}
```

### Pattern 2: Webhook Endpoint — Verify Inline, Process Async

**What:** Read raw body before parsing; verify HMAC immediately; return 200; dispatch job.
**When to use:** Both /stripe/webhook and /stripe/connect/webhook handlers.

```rust
// Source: async-stripe Webhook::construct_event pattern
#[handler]
pub async fn stripe_webhook(req: Request) -> Response {
    // Must read raw bytes BEFORE any JSON parsing (signature covers raw body)
    let stripe_sig = req.header("stripe-signature")
        .ok_or_else(|| Err(HttpResponse::text("missing signature").status(400)))?;
    let body = req.body_string().await
        .map_err(|_| Err(HttpResponse::text("body error").status(400)))?;

    let secret = ferro::config::env_required("STRIPE_WEBHOOK_SECRET");

    let event = stripe::Webhook::construct_event(&body, &stripe_sig, &secret)
        .map_err(|_| Err(HttpResponse::text("invalid signature").status(400)))?;

    // Dispatch to ferro-queue — returns immediately
    ProcessStripeWebhook { event_json: body, event_type: event.type_.to_string() }
        .dispatch()
        .await
        .ok();

    Ok(HttpResponse::json(serde_json::json!({"received": true})))
}
```

### Pattern 3: RequiresPlan Middleware

**What:** Reads TenantContext.subscription, checks plan and status, blocks or passes.
**When to use:** Routes requiring a specific plan tier.

```rust
// Source: framework/src/auth/middleware.rs pattern
pub struct RequiresPlan {
    required_plan: &'static str,
}

impl RequiresPlan {
    pub fn new(plan: &'static str) -> Self { Self { required_plan: plan } }
}

#[async_trait]
impl Middleware for RequiresPlan {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let tenant = current_tenant()
            .ok_or_else(|| Err(HttpResponse::json(json!({"error": "No tenant"})).status(400)))?;

        let has_access = tenant.subscription
            .as_ref()
            .map(|s| s.subscribed() && plan_satisfies(s.plan.as_str(), self.required_plan))
            .unwrap_or(false);

        if has_access {
            next(request).await
        } else {
            Err(HttpResponse::json(json!({
                "error": "This feature requires a higher plan.",
                "required_plan": self.required_plan
            })).status(403))
        }
    }
}
```

### Pattern 4: Stripe Checkout Session Creation

**What:** Create hosted Stripe Checkout for tenant subscription upgrades.

```rust
// Source: async-stripe CreateCheckoutSession docs
use stripe::{CheckoutSession, CreateCheckoutSession, CreateCheckoutSessionLineItems,
             CheckoutSessionMode, CheckoutSessionSubscriptionData, Client};

pub async fn create_subscription_checkout(
    client: &Client,
    stripe_customer_id: &str,
    price_id: &str,
    success_url: &str,
    cancel_url: &str,
) -> Result<String, Error> {
    let mut params = CreateCheckoutSession::new(success_url);
    params.cancel_url = Some(cancel_url);
    params.customer = Some(stripe_customer_id.parse()?);
    params.mode = Some(CheckoutSessionMode::Subscription);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        quantity: Some(1),
        price: Some(price_id.to_string()),
        ..Default::default()
    }]);

    let session = CheckoutSession::create(client, params).await?;
    Ok(session.url.unwrap_or_default())
}
```

### Pattern 5: Connect Destination Charge with Application Fee

**What:** Tenant processes payment from end user; platform takes application_fee_amount.

```rust
// Source: Stripe Connect destination charges docs
// When creating Checkout Session for Connect payment:
params.payment_intent_data = Some(CreateCheckoutSessionPaymentIntentData {
    application_fee_amount: Some(platform_fee_cents),
    transfer_data: Some(CreateCheckoutSessionPaymentIntentDataTransferData {
        destination: tenant.subscription
            .as_ref()
            .and_then(|s| s.stripe_connect_account_id.clone())
            .ok_or(Error::NoConnectAccount)?,
        ..Default::default()
    }),
    ..Default::default()
});
```

### Anti-Patterns to Avoid

- **Parse JSON body before webhook verification:** The Stripe signature covers the raw bytes. Any transformation (even pretty-printing) invalidates the HMAC. Always read raw body first.
- **Store Stripe secrets in TenantContext or logs:** Webhook secrets and API keys must come from env only.
- **Synchronous Stripe API calls in webhook handlers:** Always queue via ferro-queue; Stripe expects 200 within 30 seconds.
- **Ignoring idempotency:** Stripe webhooks can arrive multiple times. Use the event ID to deduplicate before processing.
- **Assuming plan in TenantContext is current:** Stripe webhooks may lag DB. Always source-of-truth from DB, not memory cache for billing decisions. Invalidate tenant cache on subscription events.

---

## Claude's Discretion: Recommendations

### API Facade Design

**Recommendation: static facade `Stripe::` with `OnceLock<Client>` pattern** (matches ferro-notifications' global config approach). TenantContext should not carry Stripe client — it is a per-request value struct. The static facade owns the Stripe Client.

```rust
// ferro-stripe/src/client.rs
static STRIPE_CLIENT: OnceLock<stripe::Client> = OnceLock::new();

pub struct Stripe;
impl Stripe {
    pub fn init(api_key: &str) { ... }
    pub fn client() -> &'static stripe::Client { STRIPE_CLIENT.get().unwrap() }
}
```

### Storage Approach: Separate `tenant_billing` Table

**Recommendation: separate `tenant_billing` table.** Reasons:
- Tenant table is schema-stable across multi-tenant setups; adding Stripe columns couples it to billing concerns
- Separate table is optional — tenants without Stripe simply have no row
- Makes JOIN explicit — framework code doesn't accidentally query billing data in all tenant lookups
- Pattern: DbTenantLookup finder closure already does one query; join or second query for billing is minimal overhead

```sql
-- Migration: create_tenant_billing
CREATE TABLE tenant_billing (
    id INTEGER PRIMARY KEY,
    tenant_id INTEGER NOT NULL UNIQUE REFERENCES tenants(id),
    stripe_customer_id TEXT NOT NULL,
    stripe_subscription_id TEXT,
    plan TEXT NOT NULL DEFAULT 'free',
    subscription_status TEXT NOT NULL DEFAULT 'active',
    trial_ends_at TIMESTAMP,
    current_period_end TIMESTAMP,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    stripe_connect_account_id TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_tenant_billing_tenant_id ON tenant_billing(tenant_id);
```

### Connect Account ID Placement

**Recommendation: include `stripe_connect_account_id` in TenantContext** (via `SubscriptionInfo`). The Connect checkout path is per-request (tenant charges end user), so an on-demand query would add latency to every Connect checkout. Including it in the already-loaded `SubscriptionInfo` is zero additional overhead.

### Cache/TTL Strategy

**Recommendation:** Tenant lookup already uses a 5-minute moka cache. When a Stripe subscription webhook is processed by the job, invalidate the tenant's cache entry by evicting the slug and id keys. The job has access to tenant_id from the billing table, so cache eviction is straightforward. Do not TTL subscription data independently — use the tenant cache TTL (300s) and rely on webhook events for immediate updates.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Webhook HMAC-SHA256 verification | Custom HMAC code | `stripe::Webhook::construct_event` | Timing-safe comparison, tolerance window (300s), correct header parsing |
| Checkout Session URL generation | Custom redirect/form | Stripe Checkout hosted page | PCI scope elimination, mobile-ready, 3DS/SCA handled by Stripe |
| Customer Portal | Custom subscription management UI | `BillingPortalSession::create` redirect | Plan changes, cancellation, invoice history handled by Stripe |
| Subscription status tracking | Polling Stripe API | Webhook events + DB sync | Polling is unreliable; webhooks are the authoritative notification channel |
| Idempotency keys | Random string per request | Stripe's `IdempotencyKey` support | Prevents duplicate charges on retried requests |
| Connect onboarding form | Custom KYC/bank forms | Stripe Connect hosted onboarding (AccountLink) | Legal, compliance, and international bank support handled by Stripe |

**Key insight:** Stripe's hosted pages (Checkout, Billing Portal, Connect onboarding) eliminate entire categories of complexity. The correct integration is redirect-based, not form-based.

---

## Common Pitfalls

### Pitfall 1: Raw Body Consumed Before Signature Verification

**What goes wrong:** Ferro's `Request` may provide `.json::<T>()` or `.body_string()` helpers. If the request body is parsed as JSON first, the raw bytes are consumed and the HMAC verification fails or cannot read the body.
**Why it happens:** HTTP body is a stream — once consumed, it's gone.
**How to avoid:** The webhook handler must call `req.body_bytes().await` (raw bytes) BEFORE any structured parsing. Pass the raw string to `Webhook::construct_event`. Then deserialize from that string.
**Warning signs:** `construct_event` returns `WebhookError::BadSignature` even with a correct secret.

### Pitfall 2: async-stripe Client Not Thread-Safe if Re-Created Per Request

**What goes wrong:** Creating a new `stripe::Client` per webhook request causes connection pool churn and adds latency.
**Why it happens:** `Client` contains a connection pool (hyper).
**How to avoid:** Initialize once at app startup via `OnceLock` (matches ferro-notifications' `CONFIG: OnceLock<NotificationConfig>` pattern). The `Client` is `Clone` and cheap to clone after initialization.

### Pitfall 3: TenantContext Cache Serves Stale Subscription Data

**What goes wrong:** Tenant is loaded and cached with `status: active`. Subscription lapses. Webhook processes and DB is updated, but the 5-minute cache still serves the old tenant (with `status: active`).
**Why it happens:** DbTenantLookup's moka cache TTL is 300 seconds, longer than webhook processing.
**How to avoid:** The webhook job (processing `customer.subscription.updated/deleted`) must evict the tenant from the moka cache after updating the DB. This requires the cache to be accessible from the job. Consider moving cache invalidation to a dedicated function in the framework's tenant module.
**Warning signs:** Plan gate middleware grants access after subscription has lapsed.

### Pitfall 4: Webhook Idempotency

**What goes wrong:** Stripe delivers the same webhook event multiple times (network retries). Processing `customer.subscription.deleted` twice may corrupt state.
**Why it happens:** Stripe guarantees at-least-once delivery.
**How to avoid:** Store processed Stripe event IDs in a `stripe_webhook_events` table. Before processing, check if the event ID was already handled. This is a Wave 0 migration.

### Pitfall 5: Missing `on_behalf_of` for Cross-Region Connect

**What goes wrong:** Connected account is in EU, platform is in US. Checkout fails with "can't create payment for account in different region" without `on_behalf_of`.
**Why it happens:** Stripe requires explicit settlement merchant designation for cross-border payments.
**How to avoid:** Set `on_behalf_of: Some(connect_account_id)` on Connect Checkout Sessions. Makes the connected account the settlement merchant.

### Pitfall 6: Plan Hierarchy Not Enforced Correctly

**What goes wrong:** `RequiresPlan("pro")` should allow `enterprise` users too (enterprise > pro). Without a plan hierarchy, enterprise users are denied pro-gated routes.
**Why it happens:** Simple string equality check misses the plan tier ordering.
**How to avoid:** Define `plan_satisfies(tenant_plan: &str, required: &str) -> bool` with explicit ordering: `["enterprise", "pro", "free"]`. Enterprise satisfies pro. Pro satisfies free. Free satisfies free only.

---

## Code Examples

### Webhook::construct_event (verified pattern)

```rust
// Source: https://www.payments.rs/docs/webhooks
// Source: https://docs.rs/async-stripe-webhook/latest/stripe_webhook/
use stripe::Webhook;

let event = Webhook::construct_event(
    &raw_body_string,   // &str — raw request body, not parsed JSON
    &stripe_sig_header, // &str — "stripe-signature" header value
    &webhook_secret,    // &str — "whsec_xxx" from env
)?;

match event.type_ {
    stripe::EventType::CustomerSubscriptionUpdated => { /* sync plan */ }
    stripe::EventType::CustomerSubscriptionDeleted => { /* revoke access */ }
    stripe::EventType::CheckoutSessionCompleted => { /* provision */ }
    _ => {} // ignore others
}
```

### Subscription Status Values

```rust
// Source: Stripe API docs — https://docs.stripe.com/api/subscriptions/object
// 8 possible status values:
// trialing, active, incomplete, incomplete_expired, past_due, canceled, unpaid, paused
//
// For access control, "subscribed" = trialing | active
// For immediate restriction: anything else denies access
```

### Billing Portal Redirect

```rust
// Source: stripe_billing::billing_portal_configuration docs
use stripe::{BillingPortalSession, CreateBillingPortalSession};

let params = CreateBillingPortalSession::new(stripe_customer_id);
// params.return_url = Some("https://app.example.com/billing");
let session = BillingPortalSession::create(client, params).await?;
// Redirect user to session.url
```

### Connect Account Link (Onboarding)

```rust
// Source: Stripe API docs — AccountLinks
use stripe::{AccountLink, CreateAccountLink, AccountLinkType};

let params = CreateAccountLink {
    account: connect_account_id,
    refresh_url: Some("https://app.example.com/stripe/connect/refresh"),
    return_url: Some("https://app.example.com/stripe/connect/return"),
    type_: AccountLinkType::AccountOnboarding,
    ..Default::default()
};
let link = AccountLink::create(client, params).await?;
// Redirect tenant to link.url
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| stripe-rust 0.x monolith | async-stripe 0.41 + modular feature flags | 2023-2024 | Selective features reduce compile time |
| 1.0.0 alpha/RC crates | 1.0.0-rc.3 (March 2026) — still pre-release | 2026-03-10 | 1.x API splits into per-domain crates; migration guide required |
| Manual HMAC webhook verification | `Webhook::construct_event` | 2021+ | Timing-safe built-in; handles tolerance window |
| Custom subscription UI | Stripe Billing Portal | 2020 | Full self-service portal, no custom code |
| OAuth for Connect | Stripe-hosted onboarding (AccountLink) | 2019+ | KYC, bank verification handled by Stripe |

**Deprecated/outdated:**
- `stripe-rust` crate: Different crate from `async-stripe`. Do not use — unmaintained and has different API shape.
- `async-stripe` 1.0.0-rc.x: Pre-release with known breaking changes ahead (per maintainer note). Use 0.41.x stable.

---

## Open Questions

1. **Cache invalidation mechanism for TenantContext**
   - What we know: DbTenantLookup owns a `moka::sync::Cache` internally. The webhook job needs to evict stale entries after DB update.
   - What's unclear: Jobs do not currently have access to the tenant cache instance. The cache is created inside `DbTenantLookup::new()` with no invalidation API.
   - Recommendation: Add a `invalidate(slug: &str, id: i64)` method to `TenantLookup` trait. Alternatively, accept short staleness (300s window) and document it. For billing, staleness is a security concern — recommend explicit invalidation.

2. **async-stripe version stability for 1.0.0-rc.3**
   - What we know: 1.0.0-rc.3 released 2026-03-10, uses split crate architecture.
   - What's unclear: Whether rc.3 is close enough to stable for this integration or whether breaking changes will hit before final release.
   - Recommendation: Use 0.41.x for this phase. Upgrade path to 1.x can be a separate phase after stable release.

3. **Stripe test mode for CI**
   - What we know: Test helpers are a locked decision. Stripe provides test mode API keys and test webhook payloads.
   - What's unclear: Whether ferro-stripe ships a `TestStripeClient` mock or relies on Stripe's CLI webhook forwarding.
   - Recommendation: Provide `StripeTestHelper` in a `#[cfg(test)]` module with `construct_event_from_fixture(fixture: &str, secret: &str)` and `signed_webhook_payload(event_json: &str, secret: &str)` for generating valid HMAC signatures in tests.

---

## Validation Architecture

`workflow.nyquist_validation` is not set to `false` in config.json — validation section included.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework + `tokio::test` |
| Config file | none — uses `cargo test` conventions |
| Quick run command | `cargo test -p ferro-stripe` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STRIPE-01 | SubscriptionInfo::on_trial() returns true when status=trialing | unit | `cargo test -p ferro-stripe subscription::` | Wave 0 |
| STRIPE-02 | SubscriptionInfo::subscribed() returns true for active+trialing, false for others | unit | `cargo test -p ferro-stripe subscription::` | Wave 0 |
| STRIPE-03 | Webhook::construct_event with valid HMAC returns Ok(Event) | unit | `cargo test -p ferro-stripe webhook::` | Wave 0 |
| STRIPE-04 | Webhook::construct_event with invalid HMAC returns Err | unit | `cargo test -p ferro-stripe webhook::` | Wave 0 |
| STRIPE-05 | RequiresPlan middleware passes for sufficient plan (enterprise satisfies pro) | unit | `cargo test -p ferro-stripe middleware::` | Wave 0 |
| STRIPE-06 | RequiresPlan middleware blocks for insufficient plan (free denied on pro route) | unit | `cargo test -p ferro-stripe middleware::` | Wave 0 |
| STRIPE-07 | plan_satisfies hierarchy: enterprise>pro>free | unit | `cargo test -p ferro-stripe` | Wave 0 |
| STRIPE-08 | TenantContext with SubscriptionInfo serializes correctly (JSON output) | unit | `cargo test -p ferro-rs tenant::` | Wave 0 |
| STRIPE-09 | `ferro make:stripe` generates expected files without panicking | unit | `cargo test -p ferro-cli make_stripe::` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-stripe`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `ferro-stripe/src/lib.rs` — crate entry point (does not exist yet)
- [ ] `ferro-stripe/Cargo.toml` — new crate (does not exist yet)
- [ ] Add `ferro-stripe` to `Cargo.toml` workspace members
- [ ] Add `ferro-stripe` to Wave 1 in `.github/workflows/publish.yml`
- [ ] Add `stripe` feature to `framework/Cargo.toml` optional deps

---

## Sources

### Primary (HIGH confidence)
- Stripe API documentation (https://docs.stripe.com/api/subscriptions/object) — subscription status values, webhook events
- Stripe webhook docs (https://docs.stripe.com/billing/subscriptions/webhooks) — lifecycle events
- async-stripe-webhook docs (https://docs.rs/async-stripe-webhook/latest/stripe_webhook/) — Webhook::construct_event API
- Payments.rs async-stripe docs (https://www.payments.rs/docs/webhooks) — verified webhook axum example
- lib.rs async-stripe entry (https://lib.rs/crates/async-stripe) — version 1.0.0-rc.3 confirmed, feature flags

### Secondary (MEDIUM confidence)
- Stripe Connect destination charges (https://docs.stripe.com/connect/destination-charges) — application_fee_amount pattern
- Stripe Connect + Billing integration (https://docs.stripe.com/connect/integrate-billing-connect) — SaaS fee subscription model
- Shuttle.dev Stripe + Rust tutorial (https://www.shuttle.dev/blog/2024/03/07/stripe-payments-rust) — CreateCheckoutSession code pattern (0.34.1 era, API shape similar to 0.41.x)

### Tertiary (LOW confidence — needs verification at implementation time)
- async-stripe 0.41 feature flag "connect" and "billing" — confirmed by name from search results but not verified against Cargo.toml directly
- `BillingPortalSession::create` exact API shape — verified by name from stripe_billing docs, not by reading actual generated code

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — async-stripe is the established Rust Stripe library; version confirmed via lib.rs
- Architecture: HIGH — follows existing ferro-cache/ferro-queue/ferro-notifications patterns precisely
- Webhook pattern: HIGH — Webhook::construct_event confirmed from official docs and async-stripe-webhook crate
- Pitfalls: HIGH — raw body consumption, cache staleness, idempotency are Stripe integration fundamentals
- async-stripe 1.x vs 0.41.x recommendation: MEDIUM — 1.x is pre-release per lib.rs, but exact breaking change timeline is unknown

**Research date:** 2026-03-11
**Valid until:** 2026-04-10 (async-stripe 1.x may reach stable within 30 days; re-verify before implementation if timeline extends)
