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
use ferro::{CheckoutBuilder, LineItem, Mode, handler, HttpResponse, Request, Response};

#[handler]
pub async fn upgrade(req: Request) -> Response {
    let intent = CheckoutBuilder::new(Mode::Subscription)
        .line_item(LineItem {
            name: "Pro Plan".into(),
            description: None,
            unit_amount_cents: 1500,
            quantity: 1,
            currency: "usd".into(),
        })
        .success_url("https://app.example.com/billing/success")
        .cancel_url("https://app.example.com/billing/cancel")
        .idempotency_key(&format!("upgrade-{}", chrono::Utc::now().timestamp()))
        .create()
        .await
        .map_err(|e| HttpResponse::text(e.to_string()).status(500))?;

    Ok(HttpResponse::redirect(&intent.url))
}
```

### Billing Portal

Redirect the tenant to Stripe's hosted portal for self-service plan management:

```rust
use ferro::{account, handler, HttpResponse, Request, Response};

#[handler]
pub async fn manage_billing(req: Request) -> Response {
    let customer_id = "cus_xxx";

    let url = account::billing_portal_url(
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
let sub = tenant.subscription.as_ref().expect("subscription not loaded");

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

The plan tier comparison is available in the framework's tenant module:

```rust
use ferro::tenant::subscription::plan_satisfies;

plan_satisfies("enterprise", "pro")   // true
plan_satisfies("pro", "free")         // true
plan_satisfies("free", "pro")         // false
plan_satisfies("custom", "custom")    // true — unknown plans match themselves
```

## Stripe Connect

### Connect Onboarding

Create an account link to start the Stripe Connect onboarding flow:

```rust
use ferro::{account, handler, HttpResponse, Request, Response};

#[handler]
pub async fn connect_onboarding(req: Request) -> Response {
    let account_id = "acct_xxx"; // stored on the tenant record

    let url = account::create_link(
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
use ferro::{CheckoutBuilder, LineItem, Mode, handler, HttpResponse, Request, Response};

#[handler]
pub async fn pay(req: Request) -> Response {
    let connect_id = "acct_xxx";
    // <!-- TODO(140): reword narrative for capability-axis -->
    let intent = CheckoutBuilder::new(Mode::Payment)
        .destination(connect_id, Some(100)) // 100 cents application fee
        .line_item(LineItem {
            name: "Payment".into(),
            description: None,
            unit_amount_cents: 2000, // $20.00
            quantity: 1,
            currency: "usd".into(),
        })
        .success_url("https://app.example.com/pay/success")
        .cancel_url("https://app.example.com/pay/cancel")
        .idempotency_key(&format!("pay-{}", chrono::Utc::now().timestamp()))
        .create()
        .await
        .map_err(|e| HttpResponse::text(e.to_string()).status(500))?;

    Ok(HttpResponse::redirect(&intent.url))
}
```

## Manual Capture

Manual capture authorizes card funds at checkout without charging them, then captures (charges) some or all of the authorized amount later, or cancels (releases) the hold. Useful for booking deposits where the final charge amount may differ from the initially authorized amount.

### Authorize at checkout

Set `manual_capture()` on the builder to authorize funds without an immediate charge:

```rust
use ferro::{CheckoutBuilder, LineItem, Mode, handler, HttpResponse, Request, Response};
use ferro_stripe::payment_intent;

#[handler]
pub async fn book(req: Request) -> Response {
    let intent = CheckoutBuilder::new(Mode::Payment)
        .manual_capture()
        .line_item(LineItem {
            name: "Booking deposit".into(),
            description: None,
            unit_amount_cents: 5000, // $50.00 authorized
            quantity: 1,
            currency: "usd".into(),
        })
        .success_url("https://app.example.com/bookings/success")
        .cancel_url("https://app.example.com/bookings/cancel")
        .idempotency_key("booking-deposit-42")
        .create()
        .await
        .map_err(|e| HttpResponse::text(e.to_string()).status(500))?;

    Ok(HttpResponse::redirect(&intent.url))
}
```

`manual_capture()` is only valid with `Mode::Payment`. Calling it with `Mode::Subscription` returns `Error::ManualCaptureRequiresPaymentMode` from `create()` before any network call is made.

### Capture and cancel

After the customer completes checkout, use the `payment_intent` module to resolve the hold:

```rust
use ferro_stripe::payment_intent;

// Full capture — charge the entire authorized amount.
payment_intent::capture(&payment_intent_id, None).await?;

// Partial capture — charge 2000 cents; Stripe auto-releases the remainder.
payment_intent::capture(&payment_intent_id, Some(2000)).await?;

// Cancel — release the hold without charging.
payment_intent::cancel(&payment_intent_id).await?;

// Retrieve — poll current authorization state.
payment_intent::retrieve(&payment_intent_id).await?;
```

Application-layer deduplication is required for capture retries. async-stripe 0.41 does not forward per-request idempotency keys to `PaymentIntent::capture`; a database unique constraint on the operation is the recommended guard against double-captures.

### Webhook lifecycle

Two typed events track the manual capture lifecycle:

| Stripe event | Typed event | Meaning |
|---|---|---|
| `payment_intent.amount_capturable_updated` | `StripePaymentIntentAmountCapturableUpdated` | Funds authorized and capturable (hold is live) |
| `payment_intent.canceled` | `StripePaymentIntentCanceled` | Hold released (manually or by Stripe auto-expiry) |

These events parse and dispatch the same way as all other typed ferro-stripe events — register handlers against them using `SyncDispatcher` or the queue-based path, identically to `StripeCheckoutCompleted` or `StripeSubscriptionUpdated`.

### Operational realities

- Stripe holds a card authorization for approximately 7 days. Uncaptured PaymentIntents are auto-cancelled after this window expires, surfacing as a `payment_intent.canceled` event. The `cancellation_reason` field on `StripePaymentIntentCanceled` indicates automatic expiry when Stripe triggers the cancellation.
- A partial capture charges the specified amount and Stripe automatically releases the remainder of the authorization.

### Connect composition

`manual_capture()` composes with `destination()`. The authorization is created on the platform account; on capture, Stripe performs the transfer to the connected account per the destination-charge pattern. `payment_intent::capture` and `payment_intent::cancel` are platform-scoped — no connected-account header is required.

### Correspondence with ferro-reservation

The authorize/capture/cancel triple maps directly to the `ferro-reservation` hold/commit/release vocabulary:

| ferro-reservation | Stripe PaymentIntent |
|---|---|
| `hold()` | Authorize at checkout (`manual_capture()` sets `capture_method=manual`) |
| `commit()` | `payment_intent::capture(id, amount)` |
| `release()` | `payment_intent::cancel(id)` |

This is a documented semantic correspondence — a convention for pairing a reservation hold with a payment authorization. There is no compile-time dependency between the two crates.

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

Implement `ferro::ProcessedEventLog` against your database and call `try_mark_processed(&event.id)` at the top of each webhook handler — returns `Ok(false)` when the event was already processed. Use `ferro::MemoryProcessedLog` in tests and single-process development only.

```rust
use ferro::{ProcessedEventLog, MemoryProcessedLog};
use std::sync::Arc;

// In tests:
let log = Arc::new(MemoryProcessedLog::default());

// In your webhook job (inject Arc<dyn ProcessedEventLog> via your DI container):
if !self.log.try_mark_processed(&event.id).await? {
    // Already processed — skip side effects.
    return Ok(());
}
```

For production, implement the trait against a `processed_stripe_events` table — see `ferro_stripe::idempotency` module docs for the recommended SQL schema.

## TenantContext Enrichment

When the tenant billing table is populated, attach subscription state to `TenantContext`:

```rust
use ferro::tenant::subscription::SubscriptionInfo;
use ferro::TenantContext;

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

`SubscriptionInfo` is now defined in `ferro::tenant::subscription` (framework-local type, not a Stripe-API wrapper). Fields:

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

<!-- TODO(140): reword narrative for capability-axis — subscription mock helpers were removed in 0.4.0; construct SubscriptionInfo directly from ferro::tenant::subscription -->

Construct `SubscriptionInfo` directly from `ferro::tenant::subscription` in tests:

```rust
use ferro::tenant::subscription::{SubscriptionInfo, SubscriptionStatus};

let active = SubscriptionInfo {
    stripe_subscription_id: "sub_test".into(),
    plan: "pro".into(),
    status: SubscriptionStatus::Active,
    trial_ends_at: None,
    cancel_at_period_end: false,
    current_period_end: chrono::Utc::now() + chrono::Duration::days(30),
    stripe_connect_account_id: None,
};

assert!(active.subscribed());
assert!(!active.on_trial());
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

## MCP Tools

Three MCP tools support Stripe integration development: configuration status, webhook listener discovery, and subscription schema inspection.

### `stripe_config_status`

Reports which Stripe environment variables are present or missing, and whether the scaffold directory (`src/stripe/`) exists along with which files it contains. Use this to diagnose missing configuration before debugging webhook failures.

### `stripe_webhook_events`

Scans `src/stripe/listeners.rs` for `Listener<T>` implementations and returns the Stripe event type and listener struct name for each. Use this to audit which Stripe events have listeners and which are unhandled.

### `stripe_subscription_info`

Reads the `tenant_billing` migration file and returns the table schema: column names, types, nullability, defaults, and indexes. Use this to understand the subscription data shape without connecting to the database.
