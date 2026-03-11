# Stripe Integration

ferro-stripe adds Stripe billing to Ferro applications. It covers two dimensions:

- **Platform subscriptions** — the application charges tenants for plan tiers (Free/Pro/Enterprise)
- **Stripe Connect** — tenants collect payments from their end users via connected Stripe accounts

Feature-gated behind the `stripe` feature in `Cargo.toml`.

## Quick Start

Run the scaffold command to generate the boilerplate:

```bash
# Platform subscriptions only
ferro make:stripe

# Platform subscriptions + Connect support
ferro make:stripe --connect
```

This creates:

- `src/stripe/mod.rs` — init function
- `src/stripe/webhook.rs` — platform webhook handler
- `src/stripe/listeners.rs` — subscription sync event listeners
- `src/stripe/connect_webhook.rs` — Connect webhook handler (with `--connect`)

Set the required environment variables:

```bash
STRIPE_SECRET_KEY=sk_live_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx

# Optional — only for Connect
STRIPE_CONNECT_WEBHOOK_SECRET=whsec_connect_xxx
STRIPE_APPLICATION_FEE_PERCENT=2.5
```

Initialize Stripe in `bootstrap.rs`:

```rust
use crate::stripe;

pub fn register() {
    stripe::init();
}
```

Register the webhook routes in `routes.rs`:

```rust
use ferro::Router;
use crate::stripe::webhook::stripe_webhook;

pub fn routes() -> Router {
    Router::new()
        .post("/stripe/webhook", stripe_webhook)
}
```

## Platform Subscriptions

### Creating a Checkout Session

Redirect the tenant to a Stripe-hosted checkout page to select a plan:

```rust
use ferro::{create_subscription_checkout, handler, HttpResponse, Request, Response};

#[handler]
pub async fn upgrade(req: Request) -> Response {
    let customer_id = "cus_xxx"; // from tenant DB record
    let price_id = "price_xxx";  // from Stripe dashboard

    let url = create_subscription_checkout(
        customer_id,
        price_id,
        "https://app.example.com/billing/success",
        "https://app.example.com/billing/cancel",
    )
    .await
    .map_err(|e| HttpResponse::text(e.to_string()).status(500))?;

    Ok(HttpResponse::redirect(&url))
}
```

### Billing Portal

Redirect the tenant to Stripe's hosted portal for self-service plan management:

```rust
use ferro::{billing_portal_url, handler, HttpResponse, Request, Response};

#[handler]
pub async fn manage_billing(req: Request) -> Response {
    let customer_id = "cus_xxx";

    let url = billing_portal_url(
        customer_id,
        "https://app.example.com/settings",
    )
    .await
    .map_err(|e| HttpResponse::text(e.to_string()).status(500))?;

    Ok(HttpResponse::redirect(&url))
}
```

### Subscription Lifecycle

Stripe subscription states map to `SubscriptionStatus`:

| Status | Meaning |
|--------|---------|
| `trialing` | Trial period active |
| `active` | Paid and current |
| `incomplete` | First invoice pending |
| `incomplete_expired` | First invoice expired |
| `past_due` | Renewal invoice failed |
| `canceled` | Subscription ended |
| `unpaid` | Multiple invoice failures |
| `paused` | Collection paused |

`SubscriptionInfo` exposes three helper methods:

```rust
let sub = tenant.subscription.as_ref().unwrap();

sub.on_trial()        // true when status == trialing
sub.subscribed()      // true when active or trialing
sub.on_grace_period() // true when cancel_at_period_end && subscribed()
```

### RequiresPlan Middleware

Gate routes by plan tier. Higher tiers satisfy lower requirements (enterprise > pro > free):

```rust
use ferro::{RequiresPlan, Router};
use crate::handlers::reports;

pub fn routes() -> Router {
    Router::new()
        .group("/reports", |r| {
            r.middleware(RequiresPlan::new("pro"))
             .get("/", reports::index)
        })
}
```

Returns `403 JSON` when the plan requirement is not met:

```json
{"error": "Plan does not meet requirement", "required_plan": "pro"}
```

### Plan Hierarchy

The plan tier comparison is available directly:

```rust
use ferro::plan_satisfies;

plan_satisfies("enterprise", "pro")   // true
plan_satisfies("pro", "free")         // true
plan_satisfies("free", "pro")         // false
plan_satisfies("custom", "custom")    // true — unknown plans match themselves
```

## Stripe Connect

### Connect Onboarding

Create an account link to start the Stripe Connect onboarding flow:

```rust
use ferro::{create_account_link, handler, HttpResponse, Request, Response};

#[handler]
pub async fn connect_onboarding(req: Request) -> Response {
    let account_id = "acct_xxx"; // stored on the tenant record

    let url = create_account_link(
        account_id,
        "https://app.example.com/connect/refresh",
        "https://app.example.com/connect/return",
    )
    .await
    .map_err(|e| HttpResponse::text(e.to_string()).status(500))?;

    Ok(HttpResponse::redirect(&url))
}
```

### Destination Charges

Process a one-time payment on behalf of a connected account:

```rust
use ferro::{create_connect_checkout, handler, HttpResponse, Request, Response};

#[handler]
pub async fn pay(req: Request) -> Response {
    let connect_id = "acct_xxx";
    let fee = Stripe::config().application_fee_percent.map(|p| {
        (price_cents as f64 * p / 100.0) as i64
    });

    let url = create_connect_checkout(
        connect_id,
        2000,          // $20.00 in cents
        "usd",
        "https://app.example.com/pay/success",
        "https://app.example.com/pay/cancel",
        fee,
    )
    .await
    .map_err(|e| HttpResponse::text(e.to_string()).status(500))?;

    Ok(HttpResponse::redirect(&url))
}
```

## Webhook Configuration

### Two Webhook Endpoints

| Route | Purpose | Secret |
|-------|---------|--------|
| `POST /stripe/webhook` | Platform events | `STRIPE_WEBHOOK_SECRET` |
| `POST /stripe/connect/webhook` | Connect events | `STRIPE_CONNECT_WEBHOOK_SECRET` |

Configure both endpoints in the Stripe Dashboard under **Developers → Webhooks**.

### Signature Verification

Webhooks are verified with HMAC-SHA256. Raw body access is required — do not use JSON body parsers on the webhook route. The scaffold generates handlers that read the raw body string:

```rust
let body = req.body_string().await?;
ferro::verify_webhook(&body, &sig, &Stripe::config().webhook_secret)?;
```

### Async Processing via ferro-queue

Webhook handlers return `200 OK` immediately after signature verification. Processing runs in a background job to avoid Stripe's 30-second timeout:

```
HTTP request
  → verify signature
  → dispatch ProcessStripeWebhook job
  → return 200 OK

ferro-queue worker
  → ProcessStripeWebhook::handle()
  → dispatch ferro-events Event
  → your Listener::handle()
  → DB updates, cache invalidation
```

`ProcessStripeWebhook` dispatches the appropriate ferro-events event based on the Stripe event type:

| Stripe event | ferro-events Event |
|---|---|
| `customer.subscription.updated` | `StripeSubscriptionUpdated` |
| `customer.subscription.deleted` | `StripeSubscriptionDeleted` |
| `checkout.session.completed` | `StripeCheckoutCompleted` |
| `invoice.paid` | `StripeInvoicePaid` |
| `payment_intent.succeeded` (Connect) | `StripeConnectPaymentSucceeded` |

### Event Listeners for Subscription Sync

Register listeners in your event service provider to sync subscription state to the database:

```rust
use ferro::{async_trait, EventError, Listener};
use ferro::{StripeSubscriptionUpdated, StripeSubscriptionDeleted};

pub struct SyncSubscriptionPlan;

#[async_trait]
impl Listener<StripeSubscriptionUpdated> for SyncSubscriptionPlan {
    async fn handle(&self, event: &StripeSubscriptionUpdated) -> Result<(), EventError> {
        // Parse event.event_json, update tenant_billing table
        // Invalidate tenant cache so next request loads fresh state
        Ok(())
    }
}

#[async_trait]
impl Listener<StripeSubscriptionDeleted> for SyncSubscriptionPlan {
    async fn handle(&self, event: &StripeSubscriptionDeleted) -> Result<(), EventError> {
        // Mark subscription as canceled in tenant_billing
        Ok(())
    }
}
```

### Idempotency

`ferro::stripe_is_processed(event_id)` is a stub that always returns `false`. Implement idempotency in your event listeners by recording processed event IDs in a DB table and checking before applying changes.

## TenantContext Enrichment

When the tenant billing table is populated, attach subscription state to `TenantContext`:

```rust
use ferro::{SubscriptionInfo, TenantContext};

// In your DbTenantLookup closure:
let subscription = load_billing_from_db(tenant_id).await;
TenantContext {
    id: tenant.id,
    slug: tenant.slug.clone(),
    name: tenant.name.clone(),
    plan: subscription.as_ref().map(|s| s.plan.clone()),
    subscription,
}
```

`SubscriptionInfo` fields:

| Field | Type | Description |
|-------|------|-------------|
| `stripe_subscription_id` | `String` | Stripe subscription ID (`sub_xxx`) |
| `plan` | `String` | Plan name: `free`, `pro`, `enterprise` |
| `status` | `SubscriptionStatus` | Stripe status |
| `trial_ends_at` | `Option<DateTime<Utc>>` | Trial end timestamp |
| `cancel_at_period_end` | `bool` | Scheduled for cancellation |
| `current_period_end` | `DateTime<Utc>` | Billing period end |
| `stripe_connect_account_id` | `Option<String>` | Connected account ID |

## Database Schema

The `tenant_billing` table stores subscription state per tenant:

```sql
CREATE TABLE tenant_billing (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL REFERENCES tenants(id),
    stripe_customer_id        VARCHAR(255) NOT NULL,
    stripe_subscription_id    VARCHAR(255),
    plan                      VARCHAR(64) NOT NULL DEFAULT 'free',
    status                    VARCHAR(32) NOT NULL DEFAULT 'trialing',
    trial_ends_at             TIMESTAMPTZ,
    cancel_at_period_end      BOOLEAN NOT NULL DEFAULT FALSE,
    current_period_end        TIMESTAMPTZ,
    stripe_connect_account_id VARCHAR(255),
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tenant_billing_tenant_id ON tenant_billing(tenant_id);
CREATE INDEX idx_tenant_billing_stripe_customer ON tenant_billing(stripe_customer_id);
```

## Testing

Enable test helpers in `Cargo.toml`:

```toml
[dev-dependencies]
ferro-stripe = { version = "*", features = ["test-helpers"] }
```

### Mock Subscriptions

```rust
use ferro_stripe::testing::*;

let active  = mock_subscription_active("pro");       // Active, 30d period end
let trial   = mock_subscription_trialing("pro");     // Trialing, 14d trial end
let canceled = mock_subscription_canceled("pro");    // Canceled, period ended
let past_due = mock_subscription_past_due("pro");    // PastDue, period active
let grace   = mock_subscription_on_grace("pro");     // Active, cancel_at_period_end=true
let connect = mock_subscription_with_connect("pro", "acct_123"); // Active with Connect

assert!(active.subscribed());
assert!(trial.on_trial());
assert!(grace.on_grace_period());
assert!(!canceled.subscribed());
```

### Webhook Testing

```rust
use ferro_stripe::testing::{signed_webhook_payload, mock_checkout_completed_event};
use ferro_stripe::verify_webhook;

let event = mock_checkout_completed_event("cs_test_123", "cus_test_456");
let (sig, _ts) = signed_webhook_payload(&event, "whsec_test_secret");

let result = verify_webhook(&event, &sig, "whsec_test_secret");
assert!(result.is_ok());
```

Event fixture generators:

```rust
mock_checkout_completed_event(session_id, customer_id)
mock_subscription_updated_event(subscription_id, customer_id, status)
mock_subscription_deleted_event(subscription_id, customer_id)
mock_invoice_paid_event(invoice_id, customer_id)
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `STRIPE_SECRET_KEY` | Yes | Stripe secret API key (`sk_live_xxx` or `sk_test_xxx`) |
| `STRIPE_WEBHOOK_SECRET` | Yes | Platform webhook signing secret (`whsec_xxx`) |
| `STRIPE_CONNECT_WEBHOOK_SECRET` | No | Connect webhook signing secret |
| `STRIPE_APPLICATION_FEE_PERCENT` | No | Platform fee percentage for Connect charges (e.g. `2.5`) |
