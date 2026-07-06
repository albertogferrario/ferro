# Phase 96: Stripe Integration - Research

**Researched:** 2026-03-11 (re-research refresh)
**Domain:** Stripe payments, billing subscriptions, Stripe Connect, Rust async-stripe crate
**Confidence:** HIGH (core library and Stripe API well-documented; async-stripe 1.x still pre-release — use 0.41.x)

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

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| STRIPE-01 | SubscriptionInfo struct with helper methods (on_trial, subscribed, on_grace_period) | Pattern 1 — SubscriptionInfo struct defined with all methods |
| STRIPE-02 | TenantContext enriched with subscription field replacing plan: Option<String> | Pattern 1 — TenantContext modification specified |
| STRIPE-03 | Webhook endpoint: verify HMAC inline, queue job, ack 200 | Pattern 2 — webhook handler with Webhook::construct_event |
| STRIPE-04 | Stripe events dispatched via ferro-events after queue processing | Confirmed ferro-events API: global_dispatcher().dispatch(event).await |
| STRIPE-05 | RequiresPlan middleware for plan-gate access control | Pattern 3 — mirrors AuthMiddleware exactly |
| STRIPE-06 | Stripe Checkout Session creation for platform subscriptions | Pattern 4 — CreateCheckoutSession with subscription mode |
| STRIPE-07 | Stripe Billing Portal redirect for tenant self-service | Code example — BillingPortalSession::create |
| STRIPE-08 | Stripe Connect: account creation, onboarding link generation | Code example — AccountLink::create with AccountOnboarding type |
| STRIPE-09 | Connect Checkout with application_fee_amount (one-time charges) | Pattern 5 — destination charges with transfer_data |
| STRIPE-10 | tenant_billing migration: separate table with all billing columns | Architecture — separate tenant_billing table recommendation |
| STRIPE-11 | ferro make:stripe CLI command scaffolding | CLI pattern — follows make:auth approach with templates |
| STRIPE-12 | MCP introspection tools for Stripe state | Follows existing MCP tools pattern in ferro-mcp/src/tools/ |
| STRIPE-13 | Test helpers for webhook signature construction in tests | Open Question 3 — StripeTestHelper approach defined |
</phase_requirements>

---

## Summary

Phase 96 adds a `ferro-stripe` crate integrating Stripe's payment infrastructure into the Ferro framework. Two billing dimensions: platform SaaS subscriptions (platform charges tenants for plan tiers) and Stripe Connect (tenants collect one-time payments from end users). Both are well-supported by the `async-stripe` ecosystem.

The async-stripe crate is at `1.0.0-rc.3` as of March 10, 2026 (confirmed via GitHub releases). The 1.x series uses a split multi-crate architecture (`stripe-billing`, `stripe-webhook`, etc.), but is still pre-release with known breaking changes expected before final stable. The stable series is `0.41.x`, which remains the safe choice for this integration. The webhook verification API (`Webhook::construct_event`) is stable across both series.

The current codebase has been verified: `TenantContext` still has `plan: Option<String>` (no changes since Phase 95 that would affect it), `ferro-queue` uses a `Job` trait with `dispatch().await` ergonomics, and `ferro-events` uses `global_dispatcher().dispatch(event).await`. Both are confirmed integration targets. The `ferro-events` dispatcher stores listeners by `TypeId` and calls them in priority order — Stripe webhook events will define types implementing the `Event` trait and be dispatched through this system.

The two most careful design decisions (in Claude's discretion): (1) storage layout for billing data (separate `tenant_billing` table recommended — clean separation), and (2) whether Connect account IDs live in TenantContext (recommended — zero overhead on Connect checkout path). Cache invalidation after webhook processing is the most complex operational concern.

**Primary recommendation:** Use `async-stripe` 0.41.x stable. Store billing state in a separate `tenant_billing` table joined at tenant lookup time. Include `stripe_connect_account_id` in `SubscriptionInfo` for zero-overhead plan-gate middleware. Verify webhook signatures with `Webhook::construct_event` before any body parsing. Queue all webhook processing via ferro-queue.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| async-stripe | 0.41.x | Stripe API bindings — Checkout, Billing, Connect | Official community Rust SDK; type-safe generated from Stripe OpenAPI; maintained with weekly CI updates |
| async-stripe (webhook feature) | 0.41.x | Webhook HMAC-SHA256 signature verification | Built-in timing-safe verification; handles tolerance window; avoids hand-rolling crypto |
| thiserror | 2.0 | Error types for ferro-stripe | Workspace convention (`thiserror = "2"` in ferro-queue/Cargo.toml) |
| serde / serde_json | 1.x | Serialization of billing structs and webhook payloads | Workspace dep |
| chrono | 0.4 | Timestamps: trial_ends_at, current_period_end | Workspace dep — already used in ferro-queue |
| async-trait | 0.1 | Async traits for ferro-events Listener impl | Workspace dep — ferro-events re-exports it |

### Async-stripe Feature Flags (0.41.x)

```toml
# ferro-stripe/Cargo.toml
[dependencies]
async-stripe = { version = "0.41", default-features = false, features = [
    "runtime-tokio-hyper",  # tokio + hyper runtime (matches workspace)
    "billing",              # subscriptions, invoices, billing portal
    "checkout",             # Checkout Sessions
    "connect",              # Connect accounts, account links, transfers
    "webhook-events",       # Webhook::construct_event + EventType enum
] }
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
```

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| async-stripe 0.41.x | async-stripe 1.0.0-rc.3 | RC3 confirmed pre-release (March 10, 2026) with anticipated breaking changes; 0.41.x is stable and feature-complete |
| separate ferro-stripe crate | inline in framework | Crate separation is established pattern (ferro-cache, ferro-queue, ferro-notifications); keeps framework dep tree clean |
| separate tenant_billing table | columns on tenant table | Separate table avoids schema coupling; tenant table remains stable across non-billing deployments |
| static OnceLock Stripe client | per-request client creation | Per-request client creation causes connection pool churn; OnceLock matches ferro-notifications CONFIG pattern |

---

## Architecture Patterns

### Recommended Project Structure

```
ferro-stripe/
├── Cargo.toml
└── src/
    ├── lib.rs               # Public API: re-exports, StripeConfig, Stripe facade
    ├── client.rs            # static STRIPE_CLIENT: OnceLock<stripe::Client>
    ├── config.rs            # StripeConfig (api_key, webhook_secret, connect_webhook_secret)
    ├── error.rs             # Error enum (thiserror) — one Error per crate
    ├── subscription/
    │   ├── mod.rs           # SubscriptionInfo struct, SubscriptionStatus, helper methods
    │   ├── checkout.rs      # create_checkout_session(), billing_portal_url()
    │   └── sync.rs          # sync_subscription_from_event() — DB update from webhook
    ├── connect/
    │   ├── mod.rs           # ConnectAccount helpers, create_account_link()
    │   └── checkout.rs      # create_connect_checkout_session() with application_fee_amount
    ├── webhook/
    │   ├── mod.rs           # verify_and_dispatch() — HMAC verify then ferro-queue dispatch
    │   └── events.rs        # StripeEvent types implementing ferro-events Event trait
    └── middleware/
        └── requires_plan.rs # RequiresPlan — reads TenantContext.subscription, checks plan
```

Framework integration points:
```
framework/src/tenant/mod.rs      # TenantContext: plan: Option<String> → subscription: Option<SubscriptionInfo>
framework/src/tenant/lookup.rs   # DbTenantLookup: extend finder to JOIN tenant_billing
framework/src/lib.rs             # #[cfg(feature = "stripe")] re-export ferro-stripe types
ferro-cli/src/commands/          # make_stripe.rs — new CLI scaffold command
ferro-mcp/src/tools/             # stripe_status.rs, stripe_webhooks.rs, stripe_subscription.rs
```

### Pattern 1: SubscriptionInfo in TenantContext

**What:** Replace `plan: Option<String>` with `subscription: Option<SubscriptionInfo>`. Loaded alongside tenant at lookup time from `tenant_billing` table JOIN.
**When to use:** Every request behind TenantMiddleware — subscription state available in handlers via `TenantContext::from_request`.

```rust
// ferro-stripe/src/subscription/mod.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
        // cancel_at_period_end=true + still active = scheduled to cancel, grace period
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

// framework/src/tenant/mod.rs — updated TenantContext
#[derive(Debug, Clone, serde::Serialize)]
pub struct TenantContext {
    pub id: i64,
    pub slug: String,
    pub name: String,
    /// Rich subscription state replacing plan: Option<String>
    /// None = no Stripe billing configured for this tenant
    pub subscription: Option<ferro_stripe::SubscriptionInfo>,
}
```

### Pattern 2: Webhook Endpoint — Verify Inline, Queue Async

**What:** Read raw body before any parsing; verify HMAC; return 200; dispatch ferro-queue job.
**When to use:** Both `/stripe/webhook` and `/stripe/connect/webhook` handlers.

```rust
// Source: async-stripe Webhook::construct_event pattern
// ferro-stripe/src/webhook/mod.rs
#[handler]
pub async fn stripe_webhook(req: Request) -> Response {
    // CRITICAL: raw bytes BEFORE any JSON parsing — HMAC covers raw body
    let stripe_sig = req.header("stripe-signature")
        .ok_or_else(|| HttpResponse::text("missing stripe-signature").status(400))?;
    let body = req.body_string().await
        .map_err(|_| HttpResponse::text("body read error").status(400))?;

    let secret = std::env::var("STRIPE_WEBHOOK_SECRET")
        .expect("STRIPE_WEBHOOK_SECRET not set");

    // Verify signature — returns Err on invalid HMAC or stale timestamp (>300s)
    let event = stripe::Webhook::construct_event(&body, &stripe_sig, &secret)
        .map_err(|_| HttpResponse::text("invalid signature").status(400))?;

    // Queue for async processing — ack immediately
    ProcessStripeWebhook {
        event_id: event.id.to_string(),
        event_type: event.type_.to_string(),
        payload: body,
    }
    .dispatch()
    .await
    .ok(); // fire and forget — queue failure does not 500

    Ok(HttpResponse::json(serde_json::json!({"received": true})))
}
```

### Pattern 3: RequiresPlan Middleware

**What:** Reads TenantContext.subscription, checks plan hierarchy and subscription status.
**When to use:** Routes requiring a specific plan tier. Mirrors AuthMiddleware pattern exactly.

```rust
// Source: framework/src/auth/middleware.rs pattern
// ferro-stripe/src/middleware/requires_plan.rs
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
            .ok_or_else(|| HttpResponse::json(json!({"error": "No tenant context"})).status(400))?;

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

// Plan hierarchy — enterprise satisfies pro, pro satisfies free
fn plan_satisfies(tenant_plan: &str, required: &str) -> bool {
    let tiers = ["enterprise", "pro", "free"];
    let tenant_rank = tiers.iter().position(|&p| p == tenant_plan).unwrap_or(usize::MAX);
    let required_rank = tiers.iter().position(|&p| p == required).unwrap_or(usize::MAX);
    tenant_rank <= required_rank  // lower index = higher tier
}
```

### Pattern 4: Stripe Checkout Session Creation

**What:** Create hosted Stripe Checkout for tenant subscription upgrades.

```rust
// Source: async-stripe CreateCheckoutSession docs
// https://docs.rs/async-stripe/latest/stripe/generated/checkout/checkout_session/struct.CreateCheckoutSession.html
use stripe::{CheckoutSession, CreateCheckoutSession, CreateCheckoutSessionLineItems,
             CheckoutSessionMode, Client};

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

**What:** Tenant processes payment from end user via their connected Stripe account; platform takes application_fee_amount.

```rust
// Source: Stripe Connect docs — destination charges pattern
// Application fee goes to platform; remainder to connected account
params.payment_intent_data = Some(CreateCheckoutSessionPaymentIntentData {
    application_fee_amount: Some(platform_fee_cents), // optional — 0 if no fee
    transfer_data: Some(CreateCheckoutSessionPaymentIntentDataTransferData {
        destination: tenant.subscription
            .as_ref()
            .and_then(|s| s.stripe_connect_account_id.clone())
            .ok_or(Error::NoConnectAccount)?,
        ..Default::default()
    }),
    ..Default::default()
});
// on_behalf_of required for cross-region Connect (EU tenant, US platform)
params.payment_intent_data.as_mut().unwrap().on_behalf_of =
    Some(connect_account_id.clone());
```

### Pattern 6: ferro-events Integration (confirmed from source)

**What:** Stripe webhook events implement the `Event` trait and are dispatched through the global EventDispatcher.
**Why:** ferro-events stores listeners by TypeId, calls them in priority order. The Job handler constructs the typed StripeEvent and dispatches it.

```rust
// Source: ferro-events/src/dispatcher.rs — confirmed global_dispatcher pattern
// ferro-stripe/src/webhook/events.rs
#[derive(Clone)]
pub struct SubscriptionUpdatedEvent {
    pub tenant_id: i64,
    pub new_status: SubscriptionStatus,
    pub new_plan: String,
}

impl ferro_events::Event for SubscriptionUpdatedEvent {
    fn name(&self) -> &'static str { "SubscriptionUpdated" }
}

// In the webhook job handler:
// Source: ferro-events/src/dispatcher.rs — dispatch() API
ferro_events::dispatch(SubscriptionUpdatedEvent {
    tenant_id,
    new_status,
    new_plan,
}).await.ok();
```

### Pattern 7: ferro-queue Job Definition (confirmed from source)

**What:** Webhook processing job implementing `ferro_queue::Job`.
**How:** ferro-queue provides `Job` trait with `handle()` + `max_retries()` + `retry_delay()`. Job implements `Queueable` blanket trait for `.dispatch().await`.

```rust
// Source: ferro-queue/src/job.rs — Job trait (confirmed by reading source)
use ferro_queue::{Job, Error, async_trait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStripeWebhook {
    pub event_id: String,       // for idempotency check
    pub event_type: String,
    pub payload: String,        // raw body string (already HMAC-verified)
}

#[async_trait]
impl Job for ProcessStripeWebhook {
    async fn handle(&self) -> Result<(), Error> {
        // 1. Check idempotency: event_id already processed? → return Ok
        // 2. Parse payload into stripe::Event
        // 3. Match event type → update tenant_billing in DB
        // 4. Evict tenant from TenantContext cache
        // 5. Dispatch ferro-events event for app-level listeners
        Ok(())
    }

    fn max_retries(&self) -> u32 { 5 }

    fn retry_delay(&self, attempt: u32) -> std::time::Duration {
        // Exponential backoff — matches ferro-queue Job pattern
        std::time::Duration::from_secs(2u64.pow(attempt))
    }
}
```

### Anti-Patterns to Avoid

- **Parse JSON body before webhook verification:** The Stripe signature covers raw bytes. Any transformation (even pretty-printing) invalidates the HMAC. Always read raw body first with `req.body_string().await` before calling `Webhook::construct_event`.
- **Store Stripe secrets in TenantContext or logs:** Webhook secrets and API keys must come from env only, never serialized into logs or response bodies.
- **Synchronous Stripe API calls in webhook handlers:** Always queue via ferro-queue; Stripe expects 200 within 30 seconds. If the Stripe API call in the handler itself is slow, use the queue.
- **Ignoring idempotency:** Stripe webhooks can arrive multiple times (at-least-once delivery). Use the event ID to deduplicate before processing. Store processed event IDs in `stripe_webhook_events` table.
- **Assuming cached TenantContext is current after billing events:** The 5-minute moka cache in `DbTenantLookup` will serve stale subscription state after a webhook updates the DB. The webhook job must evict the tenant cache entry after the DB update.
- **Missing on_behalf_of for cross-region Connect:** Required when connected account (EU) and platform (US) are in different regions. Without it, Stripe returns a region mismatch error on the Checkout Session.

---

## Claude's Discretion: Recommendations

### API Facade Design

**Recommendation: static `Stripe::` facade with `OnceLock<Client>` pattern.** Matches ferro-notifications' `CONFIG: OnceLock<NotificationConfig>` approach. TenantContext is a per-request value struct and must not hold the Stripe client.

```rust
// ferro-stripe/src/client.rs
use std::sync::OnceLock;

static STRIPE_CLIENT: OnceLock<stripe::Client> = OnceLock::new();

pub struct Stripe;

impl Stripe {
    pub fn init(api_key: impl Into<String>) {
        STRIPE_CLIENT.set(stripe::Client::new(api_key)).ok();
    }

    pub fn client() -> &'static stripe::Client {
        STRIPE_CLIENT.get().expect("Stripe::init() not called")
    }
}
```

### Storage Approach: Separate `tenant_billing` Table

**Recommendation: separate `tenant_billing` table.** Reasons:
- Tenant table is schema-stable; billing columns would couple all tenant queries to Stripe concerns
- Tenants without billing simply have no row — no NULL columns polluting schema
- DbTenantLookup finder closure already does one query; LEFT JOIN for billing is minimal overhead
- Isolation makes it easy to add billing to existing Ferro projects without tenant table migrations

```sql
-- Migration: create_tenant_billing
CREATE TABLE tenant_billing (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL UNIQUE REFERENCES tenants(id) ON DELETE CASCADE,
    stripe_customer_id TEXT NOT NULL,
    stripe_subscription_id TEXT,
    plan TEXT NOT NULL DEFAULT 'free',
    subscription_status TEXT NOT NULL DEFAULT 'active',
    trial_ends_at TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    stripe_connect_account_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_tenant_billing_tenant_id ON tenant_billing(tenant_id);
CREATE INDEX idx_tenant_billing_stripe_customer ON tenant_billing(stripe_customer_id);

-- Idempotency table
CREATE TABLE stripe_webhook_events (
    event_id TEXT PRIMARY KEY,           -- Stripe event ID (evt_xxx)
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Connect Account ID Placement

**Recommendation: include `stripe_connect_account_id` in `SubscriptionInfo`** (loaded from `tenant_billing` JOIN at tenant lookup). The Connect checkout path is per-request; an on-demand query would add latency to every Connect payment flow. The tenant cache (5-minute TTL) already amortizes the JOIN cost.

### Cache/TTL Strategy

**Recommendation:** Use the existing 5-minute moka cache TTL for the tenant+subscription JOIN. When the webhook job updates the DB, evict the tenant by both slug and id cache keys. The job needs access to the tenant_id from the billing table to look up the slug for eviction.

Add `invalidate(slug: &str, id: i64)` to `TenantLookup` trait. `DbTenantLookup` implements it by calling `self.cache.invalidate(&slug.to_string())` and `self.cache.invalidate(&id.to_string())`. Pass the invalidator into the webhook job at dispatch time or look it up via a framework-level registry.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Webhook HMAC-SHA256 verification | Custom HMAC code | `stripe::Webhook::construct_event` | Timing-safe comparison; 300-second tolerance window; correct Stripe-Signature header parsing |
| Checkout Session URL generation | Custom redirect/form | Stripe Checkout hosted page | PCI scope elimination; mobile-ready; 3DS/SCA handled by Stripe |
| Subscription management UI | Custom plan change forms | `BillingPortalSession::create` redirect | Plan changes, cancellation, invoice history handled by Stripe |
| Subscription status polling | Cron job polling Stripe API | Webhook events + DB sync | Polling is unreliable and adds cost; webhooks are the authoritative notification channel |
| Idempotency key generation | Random string per request | Stripe's built-in `IdempotencyKey` on API calls | Prevents duplicate charges on client retries; Stripe handles key storage |
| Connect onboarding form | Custom KYC/bank verification forms | Stripe Connect hosted onboarding (`AccountLink`) | Legal compliance, international bank support, KYC all handled by Stripe |
| Application fee calculation | Custom fee splitting logic | Stripe `application_fee_amount` on PaymentIntent | Stripe handles fund splitting and transfers; no custom payout logic needed |

**Key insight:** Every Stripe-hosted page (Checkout, Billing Portal, Connect onboarding) eliminates an entire category of security and compliance complexity. The correct Ferro integration is redirect-based, not form-based.

---

## Common Pitfalls

### Pitfall 1: Raw Body Consumed Before Signature Verification

**What goes wrong:** If Ferro's `Request` JSON parsing or middleware reads and buffers the body before the webhook handler gets it, `Webhook::construct_event` receives an already-consumed or transformed body.
**Why it happens:** HTTP body is a stream — once consumed or parsed, the raw bytes are gone. Any JSON pretty-printing or re-serialization changes bytes and invalidates the HMAC.
**How to avoid:** The webhook handler MUST call `req.body_string().await` (raw bytes as string) first, before any structured parsing. Pass that raw string to `Webhook::construct_event`. Then deserialize the event from the already-parsed stripe::Event returned by construct_event.
**Warning signs:** `construct_event` returns `WebhookError::BadSignature` even with a correct secret in local testing.

### Pitfall 2: TenantContext Cache Serves Stale Subscription Data

**What goes wrong:** Tenant cached with `status: active` at T=0. Subscription lapses at T=1. Webhook processes and DB updated at T=2. Cache evicts at T=300 (5 minutes). During T=2 to T=300, `RequiresPlan` middleware still grants access.
**Why it happens:** `DbTenantLookup`'s moka cache TTL is 300 seconds.
**How to avoid:** Webhook job must evict tenant from cache after DB update. Requires `TenantLookup::invalidate()` method (see Open Question 1). Without it, billing decisions remain unsafe.
**Warning signs:** Cancelled tenant retains plan access for up to 5 minutes after cancellation event.

### Pitfall 3: Webhook Idempotency

**What goes wrong:** Stripe delivers `customer.subscription.deleted` twice. Processing twice may double-cancel, corrupt grace period state, or dispatch double ferro-events notifications.
**Why it happens:** Stripe guarantees at-least-once delivery, not exactly-once.
**How to avoid:** Check `stripe_webhook_events` table for the event ID before processing. Insert on process. Wrap in a transaction. Return Ok from the job without processing if event_id already exists.
**Warning signs:** Billing state corruption, double notification emails, contradictory DB state.

### Pitfall 4: async-stripe Client Created Per-Request

**What goes wrong:** Connection pool churn, latency spikes, resource exhaustion under load.
**Why it happens:** `stripe::Client` contains a hyper connection pool. Recreating it per-request discards pooled connections.
**How to avoid:** Initialize once at app startup via `OnceLock` (see Stripe:: facade pattern). The `Client` is `Clone` and cloning it shares the underlying pool.
**Warning signs:** Increasing latency under load; high number of TCP connections to Stripe API servers.

### Pitfall 5: Missing `on_behalf_of` for Cross-Region Connect

**What goes wrong:** Connected account (EU) + platform (US) → Stripe returns region mismatch error when creating Checkout Session.
**Why it happens:** Stripe requires explicit settlement merchant designation for cross-border payment flows.
**How to avoid:** Set `on_behalf_of: Some(connect_account_id)` on Connect Checkout Sessions. Makes the connected account the settlement merchant for compliance.
**Warning signs:** Checkout Session creation returns a Stripe error about the account region.

### Pitfall 6: Plan Hierarchy Not Enforced

**What goes wrong:** `RequiresPlan("pro")` should allow enterprise users (enterprise > pro > free). Without hierarchy, enterprise users are denied pro-gated routes.
**Why it happens:** Simple `==` check on plan string ignores ordering.
**How to avoid:** `plan_satisfies(tenant_plan, required)` with explicit tier ordering (see Pattern 3). Enterprise satisfies pro. Pro satisfies free. Free satisfies only free.
**Warning signs:** Enterprise tenants can't access pro features; users downgraded from enterprise lose access to pro routes.

### Pitfall 7: Missing moka Cache for SubscriptionInfo Changes

**What goes wrong:** When stripe_connect_account_id is added to a tenant mid-request cycle, the cached TenantContext has None — Connect checkout fails despite the account being connected.
**Why it happens:** moka cache returned the pre-connection TenantContext.
**How to avoid:** Invalidate cache when Connect account is linked (not just on webhook events). Any mutation to tenant_billing rows should be followed by cache eviction.

---

## Code Examples

### Webhook::construct_event (verified pattern)

```rust
// Source: https://docs.rs/async-stripe/latest/stripe/struct.Webhook.html
use stripe::Webhook;

// raw_body: raw request body string — NOT parsed JSON
// stripe_sig: "stripe-signature" header value
// webhook_secret: "whsec_xxx" from env
let event = Webhook::construct_event(&raw_body, &stripe_sig, &webhook_secret)?;

match event.type_ {
    stripe::EventType::CustomerSubscriptionUpdated => { /* sync plan from DB */ }
    stripe::EventType::CustomerSubscriptionDeleted => { /* revoke access */ }
    stripe::EventType::CheckoutSessionCompleted => { /* provision customer/subscription */ }
    stripe::EventType::CustomerSubscriptionTrialWillEnd => { /* notify tenant */ }
    stripe::EventType::InvoicePaymentFailed => { /* notify payment failure */ }
    _ => {} // unknown events: log and ignore
}
```

### Subscription Status Values (from Stripe API)

```rust
// Source: Stripe API docs — https://docs.stripe.com/api/subscriptions/object
// 8 possible status values (matching SubscriptionStatus enum variants):
// trialing, active, incomplete, incomplete_expired, past_due, canceled, unpaid, paused
//
// For access control:
//   "subscribed" = active | trialing (grant access)
//   anything else = deny access immediately (no grace period per locked decisions)
//
// For grace period detection:
//   cancel_at_period_end = true AND subscribed = true → on_grace_period
```

### Billing Portal Redirect

```rust
// Source: stripe_billing::billing_portal docs
use stripe::{BillingPortalSession, CreateBillingPortalSession};

let mut params = CreateBillingPortalSession::new(stripe_customer_id);
params.return_url = Some("https://app.example.com/billing");
let session = BillingPortalSession::create(Stripe::client(), params).await?;
// Redirect tenant to session.url
Ok(HttpResponse::new().status(302).header("Location", session.url))
```

### Connect Account Link (Onboarding)

```rust
// Source: Stripe API docs — Account Links
// https://docs.stripe.com/connect/onboarding/quickstart
use stripe::{AccountLink, CreateAccountLink, AccountLinkType};

let params = CreateAccountLink {
    account: connect_account_id,
    refresh_url: Some("https://app.example.com/stripe/connect/refresh"),
    return_url: Some("https://app.example.com/stripe/connect/return"),
    type_: AccountLinkType::AccountOnboarding,
    ..Default::default()
};
let link = AccountLink::create(Stripe::client(), params).await?;
// Redirect tenant to link.url — time-limited one-use URL
Ok(HttpResponse::new().status(302).header("Location", link.url))
```

### ferro-events dispatch (verified from source)

```rust
// Source: ferro-events/src/dispatcher.rs — dispatch() function
// Dispatches to global dispatcher; no listener registration needed here
// Listeners registered at app boot in service provider

ferro_events::dispatch(SubscriptionUpdatedEvent {
    tenant_id: billing_row.tenant_id,
    new_status: SubscriptionStatus::Active,
    new_plan: billing_row.plan.clone(),
}).await.ok(); // Event listener errors are logged but not propagated to job
```

### ferro-queue dispatch (verified from source)

```rust
// Source: ferro-queue/src/lib.rs — Queueable blanket trait
// Any type implementing Job + Serialize + DeserializeOwned gets .dispatch()

ProcessStripeWebhook {
    event_id: event.id.to_string(),
    event_type: event.type_.to_string(),
    payload: raw_body,
}
.dispatch()
.await?;
// In redis mode: pushed to queue; in sync mode (QUEUE_CONNECTION=sync): executed immediately
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| stripe-rust (wyyerd/stripe-rs) | async-stripe (arlyon/async-stripe) | 2020-2021 | Different crate; stripe-rs is unmaintained |
| async-stripe monolith 0.x | 0.41.x monolith + 1.0.0-rc.3 split crates | 2024-2026 | 1.x splits into per-domain crates; 0.41.x remains monolith with feature flags |
| Manual HMAC webhook verification | `Webhook::construct_event` | 2021+ | Timing-safe, built-in tolerance window, correct header parsing |
| Custom subscription management UI | Stripe Billing Portal | 2020 | Full self-service portal — no custom code needed |
| OAuth-based Connect | Stripe-hosted onboarding (AccountLink) | 2019+ | KYC, bank verification, compliance all handled by Stripe |
| Polling for subscription changes | Webhook events + DB sync | 2019+ | Polling adds latency and cost; webhooks are real-time and authoritative |

**Deprecated/outdated:**
- `stripe-rust` / `wyyerd/stripe-rs`: Different crate from `async-stripe`. Do not use — unmaintained and different API shape.
- `async-stripe` 1.0.0-rc.x: Pre-release as of March 2026. 1.0.0-rc.3 released March 10, 2026. Use 0.41.x for this phase. Upgrade to 1.x after stable release.

---

## Open Questions

1. **Cache invalidation mechanism for TenantContext**
   - What we know: `DbTenantLookup` owns a `moka::sync::Cache` internally with no public invalidation API. The webhook job needs to evict stale entries after DB update. Current `TenantLookup` trait has only `find_by_slug` and `find_by_id` — no invalidation method.
   - What's unclear: How to give the webhook job access to the cache instance. The cache is owned by `DbTenantLookup`, which is registered in the framework at app startup. Jobs don't have dependency injection.
   - Recommendation: Add `invalidate(slug: &str, id: i64)` method to the `TenantLookup` trait in `framework/src/tenant/lookup.rs`. Store the lookup in a global `Arc<dyn TenantLookup>` accessible from jobs (similar to how Queue::connection() works). This is a Wave 0 framework change.

2. **async-stripe version stability**
   - What we know: 1.0.0-rc.3 released March 10, 2026. The maintainer notes "still expecting a few breaking changes before RC" (per GitHub README at time of research). The latest confirmed stable is 0.41.x.
   - What's unclear: Whether stable 1.0 will release before or after this phase completes.
   - Recommendation: Use 0.41.x for this phase. The 0.41.x API is feature-complete for all requirements. Migration to 1.x can be a separate phase.

3. **Test helpers for webhook signature construction**
   - What we know: Tests need to construct valid Stripe-Signature headers with HMAC-SHA256 for unit testing webhook handlers without real Stripe API calls. Stripe's format: `t={timestamp},v1={hex(HMAC-SHA256(secret, "{timestamp}.{payload}"))}`
   - What's unclear: Whether async-stripe 0.41.x exposes a test utility for this or if it must be hand-constructed.
   - Recommendation: Provide `StripeTestHelper` in `ferro-stripe/src/test_helpers.rs` (behind `#[cfg(test)]` or `test-helpers` feature flag) with `signed_payload(payload: &str, secret: &str) -> String` that returns the `Stripe-Signature` header value. This is straightforward to implement with `sha2` + `hmac` crates or the `sha2` already in the workspace.

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
| STRIPE-01 | `SubscriptionInfo::on_trial()` returns true when status=Trialing | unit | `cargo test -p ferro-stripe subscription` | Wave 0 |
| STRIPE-02 | `SubscriptionInfo::subscribed()` true for Active/Trialing, false for others | unit | `cargo test -p ferro-stripe subscription` | Wave 0 |
| STRIPE-03 | `plan_satisfies("enterprise", "pro")` returns true; `plan_satisfies("free", "pro")` false | unit | `cargo test -p ferro-stripe middleware` | Wave 0 |
| STRIPE-04 | `Webhook::construct_event` with valid HMAC returns Ok(Event) | unit | `cargo test -p ferro-stripe webhook` | Wave 0 |
| STRIPE-05 | `Webhook::construct_event` with invalid HMAC returns Err | unit | `cargo test -p ferro-stripe webhook` | Wave 0 |
| STRIPE-06 | `RequiresPlan::handle` passes request for sufficient plan | unit | `cargo test -p ferro-stripe middleware` | Wave 0 |
| STRIPE-07 | `RequiresPlan::handle` returns 403 for insufficient plan | unit | `cargo test -p ferro-stripe middleware` | Wave 0 |
| STRIPE-08 | `TenantContext` with `SubscriptionInfo` serializes to valid JSON | unit | `cargo test -p ferro-rs tenant` | Wave 0 |
| STRIPE-09 | `ProcessStripeWebhook` job skips duplicate event_id (idempotency) | unit | `cargo test -p ferro-stripe webhook` | Wave 0 |
| STRIPE-10 | `SubscriptionInfo::on_grace_period()` true when cancel_at_period_end=true + subscribed | unit | `cargo test -p ferro-stripe subscription` | Wave 0 |
| STRIPE-11 | `ferro make:stripe` generates expected files without panicking | unit | `cargo test -p ferro-cli make_stripe` | Wave 0 |
| STRIPE-12 | `StripeTestHelper::signed_payload` produces valid Stripe-Signature header | unit | `cargo test -p ferro-stripe test_helpers` | Wave 0 |
| STRIPE-13 | MCP stripe_status tool returns structured JSON | unit | `cargo test -p ferro-mcp stripe_status` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-stripe`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `ferro-stripe/` directory and `Cargo.toml` — crate does not exist yet
- [ ] `ferro-stripe/src/lib.rs` — crate entry point
- [ ] Add `ferro-stripe` to workspace `Cargo.toml` members list
- [ ] Add `ferro-stripe` to Wave 1 in `.github/workflows/publish.yml`
- [ ] Add `stripe` feature to `framework/Cargo.toml` optional deps with `ferro-stripe` dep
- [ ] Add `invalidate` method to `TenantLookup` trait in `framework/src/tenant/lookup.rs`

---

## Sources

### Primary (HIGH confidence)
- Stripe subscriptions webhook docs (https://docs.stripe.com/billing/subscriptions/webhooks) — lifecycle events, trial_will_end, payment_failed
- Stripe Connect webhook docs (https://docs.stripe.com/connect/webhooks) — account.updated, Connect event routing
- async-stripe docs.rs (https://docs.rs/async-stripe/latest/stripe/) — CreateCheckoutSession, Webhook types
- async-stripe GitHub releases (https://github.com/arlyon/async-stripe/releases) — 1.0.0-rc.3 confirmed pre-release March 10, 2026
- Stripe Connect onboarding (https://docs.stripe.com/connect/onboarding/quickstart) — AccountLink, account types
- Current codebase (read directly): ferro-queue/src/lib.rs, job.rs, dispatcher.rs — confirmed Job trait + Queueable API
- Current codebase (read directly): ferro-events/src/dispatcher.rs — confirmed global_dispatcher().dispatch() API
- Current codebase (read directly): framework/src/tenant/mod.rs — confirmed plan: Option<String> (not yet changed)
- Current codebase (read directly): framework/src/auth/middleware.rs — RequiresPlan mirror pattern

### Secondary (MEDIUM confidence)
- Shuttle.dev Stripe Rust tutorial (https://www.shuttle.dev/blog/2024/03/07/stripe-payments-rust) — CreateCheckoutSession code shape (0.34.1 era, same general shape as 0.41.x)
- Stripe webhook signature docs (https://docs.stripe.com/webhooks/signature) — HMAC-SHA256 format, 300s tolerance

### Tertiary (LOW confidence — verify at implementation)
- async-stripe 0.41 feature flag exact names ("billing", "checkout", "connect", "webhook-events") — confirmed by name from search results and docs.rs; verify against actual Cargo.toml when adding dependency
- `BillingPortalSession::create` exact parameter struct shape — verify against docs.rs at implementation time
- `CreateAccountLink` exact struct fields — verify against docs.rs at implementation time

---

## Metadata

**Confidence breakdown:**
- Standard stack (async-stripe 0.41.x): HIGH — confirmed as stable; 1.0.x is pre-release per GitHub releases
- Architecture patterns: HIGH — all patterns verified against actual codebase source files
- ferro-queue integration: HIGH — read from ferro-queue/src/lib.rs, job.rs, dispatcher.rs directly
- ferro-events integration: HIGH — read from ferro-events/src/dispatcher.rs directly
- TenantContext current state: HIGH — read from framework/src/tenant/mod.rs directly
- Webhook verification pattern: HIGH — confirmed from Stripe docs + async-stripe API docs
- Pitfalls: HIGH — raw body, idempotency, cache staleness are Stripe fundamentals
- async-stripe feature flag exact names: LOW — need Cargo.toml verification at implementation time

**Research date:** 2026-03-11
**Valid until:** 2026-04-10 (async-stripe 1.x may reach stable within 30 days; re-verify before implementation if timeline extends beyond April 2026)
