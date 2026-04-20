# Phase 140: Core Reshape - Pattern Map

**Mapped:** 2026-04-20
**Files analyzed:** 12 (new/modified files in scope)
**Analogs found:** 12 / 12

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-stripe/src/checkout.rs` | service | request-response | `ferro-stripe/src/connect/checkout.rs` | exact |
| `ferro-stripe/src/refund.rs` | service | request-response | `ferro-stripe/src/connect/checkout.rs` | role-match |
| `ferro-stripe/src/account.rs` | service | request-response | `ferro-stripe/src/connect/checkout.rs` + `subscription/checkout.rs` | exact (merge) |
| `ferro-stripe/src/idempotency.rs` | utility | event-driven | `ferro-stripe/src/webhook/events.rs` (trait pattern) | partial |
| `ferro-stripe/src/client.rs` | utility | request-response | `ferro-stripe/src/client.rs` (extend in place) | self |
| `ferro-stripe/src/error.rs` | utility | — | `ferro-stripe/src/error.rs` (extend in place) | self |
| `ferro-stripe/src/webhook/verify.rs` | utility | request-response | `ferro-stripe/src/webhook/mod.rs` (extraction) | exact |
| `ferro-stripe/src/webhook/events.rs` | model | event-driven | `ferro-stripe/src/webhook/events.rs` (relocate only) | self |
| `ferro-stripe/src/webhook/sync.rs` | stub | — | none | stub |
| `ferro-stripe/src/webhook/queue.rs` | stub | — | none | stub |
| `ferro-stripe/src/lib.rs` | config | — | `ferro-stripe/src/lib.rs` (rewrite re-exports) | self |
| `ferro-stripe/Cargo.toml` | config | — | `ferro-stripe/Cargo.toml` (add dashmap dep) | self |

---

## Pattern Assignments

### `ferro-stripe/src/checkout.rs` (service, request-response)

**Analog:** `ferro-stripe/src/connect/checkout.rs`

**Imports pattern** (lines 1-6):
```rust
use crate::Error;
use stripe::{
    AccountLink, AccountLinkType, CheckoutSession, CheckoutSessionMode, CreateAccountLink,
    CreateCheckoutSession, CreateCheckoutSessionLineItems, CreateCheckoutSessionPaymentIntentData,
    CreateCheckoutSessionPaymentIntentDataTransferData,
};
```

**Core builder pattern** — consuming `with_*` methods returning `Self`, verified against `connect/checkout.rs` shape and CONTEXT.md D-07 to D-10:
```rust
pub enum Mode {
    Payment,
    Subscription,
}

pub struct LineItem {
    pub name: String,
    pub description: Option<String>,
    pub unit_amount_cents: i64,
    pub quantity: u32,
    pub currency: String,
}

pub struct CheckoutBuilder {
    mode: Mode,
    line_items: Vec<LineItem>,
    success_url: Option<String>,
    cancel_url: Option<String>,
    metadata: Vec<(String, String)>,
    customer_email: Option<String>,
    destination: Option<(String, Option<i64>)>,
    idempotency_key: Option<String>,
}

pub struct CheckoutIntent {
    pub session_id: String,
    pub url: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub idempotency_key: String,
}

impl CheckoutBuilder {
    pub fn new(mode: Mode) -> Self { ... }
    pub fn line_item(self, item: LineItem) -> Self { ... }  // consuming, returns Self
    pub fn success_url(self, url: &str) -> Self { ... }
    pub fn cancel_url(self, url: &str) -> Self { ... }
    pub fn metadata(self, key: &str, value: &str) -> Self { ... }
    pub fn customer_email(self, email: &str) -> Self { ... }
    pub fn destination(self, account_id: &str, fee_cents: Option<i64>) -> Self { ... }
    pub fn idempotency_key(self, key: &str) -> Self { ... }
    pub async fn create(self) -> Result<CheckoutIntent, Error> { ... }
}
```

**Stripe API call pattern** (from `connect/checkout.rs` lines 22-53):
```rust
let client = crate::Stripe::client();
let mut params = CreateCheckoutSession::new();
params.success_url = Some(success_url);
params.cancel_url = Some(cancel_url);
params.mode = Some(CheckoutSessionMode::Payment);
// ... set line_items, payment_intent_data ...
let session = CheckoutSession::create(client, params).await?;
```

**Error handling pattern** — runtime guard for missing idempotency key (D-07); return `Err` before any async call:
```rust
let key = self.idempotency_key.ok_or(Error::MissingIdempotencyKey)?;
```

**`expires_at` conversion pattern** (from `subscription/sync.rs` lines 12-13 — the `trial_end` pattern):
```rust
// CheckoutSession.expires_at may be Option<i64>; use .map + .flatten + fallback:
let expires_at = session.expires_at
    .and_then(|ts| chrono::Utc.timestamp_opt(ts, 0).single())
    .unwrap_or_else(chrono::Utc::now);
```

**Currency parse error pattern** (from `connect/checkout.rs` line 31):
```rust
currency.parse()
    .map_err(|_| Error::Stripe(format!("invalid currency: {currency}")))?
```

---

### `ferro-stripe/src/refund.rs` (service, request-response)

**Analog:** `ferro-stripe/src/connect/checkout.rs`

**Imports pattern** — mirror connect/checkout.rs structure, replace checkout-specific types with refund types:
```rust
use crate::Error;
use stripe::{CreateRefund, Refund, RefundReason};
```

**Core async fn pattern** (shape from `connect/checkout.rs` lines 12-53):
```rust
pub async fn create(
    charge_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReason>,
) -> Result<stripe::Refund, Error> {
    let client = crate::Stripe::client();
    // build CreateRefund params, call Refund::create(client, params).await?
}

pub async fn retrieve(refund_id: &str) -> Result<stripe::Refund, Error> {
    let client = crate::Stripe::client();
    // call Refund::retrieve(client, &refund_id.parse()?, &Default::default()).await?
}
```

**Error conversion** — `?` on `stripe::StripeError` via the `From` impl in `error.rs` (lines 25-29 of error.rs):
```rust
impl From<stripe::StripeError> for Error {
    fn from(e: stripe::StripeError) -> Self {
        Error::Stripe(e.to_string())
    }
}
```
All async-stripe calls can use `?` directly.

---

### `ferro-stripe/src/account.rs` (service, request-response)

**Analog:** `ferro-stripe/src/connect/checkout.rs` (for `create_link`) + `ferro-stripe/src/subscription/checkout.rs` (for `billing_portal_url`)

**`create_link` — copy verbatim from `connect/checkout.rs` lines 60-77:**
```rust
pub async fn create_link(
    account_id: &str,
    refresh_url: &str,
    return_url: &str,
) -> Result<String, Error> {
    let client = crate::Stripe::client();
    let account: stripe::AccountId = account_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid account id: {account_id}")))?;
    let mut params = CreateAccountLink::new(account, AccountLinkType::AccountOnboarding);
    params.refresh_url = Some(refresh_url);
    params.return_url = Some(return_url);
    let link = AccountLink::create(client, params).await?;
    Ok(link.url)
}
```

**`billing_portal_url` — copy verbatim from `subscription/checkout.rs` lines 40-52:**
```rust
pub async fn billing_portal_url(customer_id: &str, return_url: &str) -> Result<String, Error> {
    let client = crate::Stripe::client();
    let customer_id_parsed: stripe::CustomerId = customer_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid customer id: {customer_id}")))?;
    let mut params = CreateBillingPortalSession::new(customer_id_parsed);
    params.return_url = Some(return_url);
    let session = BillingPortalSession::create(client, params).await?;
    Ok(session.url)
}
```

**`create_account` and `retrieve_account`** — new fns; follow the same shape (`client`, `params`, `Type::create/retrieve(client, params).await?`, map errors via `?`). Exact async-stripe 0.41 call sites are ASSUMED (A4); verify against stripe crate before writing.

---

### `ferro-stripe/src/idempotency.rs` (utility, event-driven)

**No direct analog in codebase** — closest pattern is the `ferro_queue::Job` trait impl in `webhook/events.rs` (lines 113-156) for the `#[async_trait]` macro usage shape. That usage is `#[ferro_queue::async_trait]` (re-exported from `async-trait`). For `ProcessedEventLog`, use `async_trait::async_trait` directly since this module does not go through ferro-queue.

**Module doc pattern** — module-level doc with embedded SQL (D-06). Use `//!` doc comments at top of file:
```rust
//! Idempotency primitives for Stripe webhook processing.
//!
//! ## Recommended SQL schema
//!
//! ```sql
//! CREATE TABLE stripe_processed_events (
//!   event_id TEXT PRIMARY KEY,
//!   event_type TEXT NOT NULL,
//!   received_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
//! );
//! ```
//!
//! Ferro does not ship this migration. Applications own the table.
//! The `PRIMARY KEY` on `event_id` is the idempotency fence.
```

**Trait definition pattern** (D-04):
```rust
use async_trait::async_trait;
use crate::Error;

#[async_trait]
pub trait ProcessedEventLog: Send + Sync {
    async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error>;
}
```

**`MemoryProcessedLog` pattern** (D-05 — DashMap insert semantics verified in RESEARCH.md Pitfall 5):
```rust
pub struct MemoryProcessedLog {
    seen: dashmap::DashMap<String, ()>,
}

impl MemoryProcessedLog {
    pub fn new() -> Self {
        Self { seen: dashmap::DashMap::new() }
    }
}

#[async_trait]
impl ProcessedEventLog for MemoryProcessedLog {
    async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error> {
        // insert returns None (absent → first time) → Ok(true)
        // insert returns Some(()) (present → already seen) → Ok(false)
        Ok(self.seen.insert(event_id.to_string(), ()).is_none())
    }
}
```

**Test pattern** — inline `#[cfg(test)]` module at bottom of file, `#[tokio::test]` for async tests (mirrors `config.rs` sync tests and `events.rs` async test shape):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_log_true_then_false() { ... }

    #[tokio::test]
    async fn memory_log_concurrent_insert_applies_once() {
        use std::sync::Arc;
        let log = Arc::new(MemoryProcessedLog::new());
        let log2 = Arc::clone(&log);
        let t1 = tokio::spawn(async move { log.try_mark_processed("evt_race_001").await });
        let t2 = tokio::spawn(async move { log2.try_mark_processed("evt_race_001").await });
        let (r1, r2) = tokio::join!(t1, t2);
        let v1 = r1.unwrap().unwrap();
        let v2 = r2.unwrap().unwrap();
        assert_ne!(v1, v2, "concurrent inserts must apply exactly once");
    }
}
```

---

### `ferro-stripe/src/client.rs` (utility, request-response — extend in place)

**Analog:** `ferro-stripe/src/client.rs` (self — add method to existing `impl Stripe` block)

**Existing `impl Stripe` block** (lines 22-53 of client.rs) — add `with` as a new `pub fn` below `config()`:
```rust
/// Returns a scoped Stripe client for the given API key.
///
/// Use for per-tenant direct-charges scenarios where a different
/// Stripe account key is needed per request.
/// Does not affect the global static client initialized by [`Stripe::init`].
pub fn with(api_key: &str) -> stripe::Client {
    stripe::Client::new(api_key)
}
```

**Doc comment convention** — single-sentence summary, then blank line, then `Use for ...` paragraph, then `Does not affect ...` clarification. Matches the doc style in existing `init`, `client`, `config` methods.

---

### `ferro-stripe/src/error.rs` (utility — extend in place)

**Analog:** `ferro-stripe/src/error.rs` (self — add variant to existing `Error` enum)

**Existing enum pattern** (lines 3-23 of error.rs) — add after `EventAlreadyProcessed`:
```rust
/// Idempotency key not set on CheckoutBuilder before calling create().
#[error("idempotency key required: call .idempotency_key() before .create()")]
MissingIdempotencyKey,
```

**Convention:** doc comment describes the condition, not the variant name. `#[error("...")]` message uses imperative instruction to fix. Pattern matches `WebhookVerification` and `EventAlreadyProcessed` variants.

---

### `ferro-stripe/src/webhook/verify.rs` (utility, request-response — extracted from mod.rs)

**Analog:** `ferro-stripe/src/webhook/mod.rs` lines 17-24 (the `verify_webhook` fn is the entire content)

**Complete file content** — fn extracted verbatim, tests moved with it:
```rust
use crate::Error;

/// Verifies a Stripe webhook signature and parses the event payload.
///
/// Uses HMAC-SHA256 as per the Stripe webhook verification protocol.
/// Returns the parsed [`stripe::Event`] on success.
///
/// # Errors
///
/// Returns [`Error::WebhookVerification`] when:
/// - The signature header is malformed
/// - The HMAC does not match
/// - The timestamp is more than 5 minutes old
pub fn verify_webhook(
    raw_body: &str,
    signature: &str,
    secret: &str,
) -> Result<stripe::Event, Error> {
    stripe::Webhook::construct_event(raw_body, signature, secret)
        .map_err(|e| Error::WebhookVerification(e.to_string()))
}
```

**Test migration:** The three passing tests from `webhook/mod.rs` (lines 83-112) move to `verify.rs`. The `is_processed_returns_false_for_unseen_ids` test (lines 114-119) is **deleted** (fn removed per D-13). The `signed_webhook_payload` import in tests stays at `crate::webhook::events::signed_webhook_payload` (function stays in `events.rs`).

---

### `ferro-stripe/src/webhook/events.rs` (model, event-driven — relocate only)

**No changes to content** (D-12). File moves from `webhook/events.rs` to `webhook/events.rs` (same path — already in the correct location under the `webhook/` directory). Only change: `webhook/mod.rs`'s `pub mod events;` declaration is replaced by a `pub mod events;` declaration in the new `webhook/mod.rs` shim or in `lib.rs` directly.

**Existing import path consumers:** `use crate::webhook::events::signed_webhook_payload` in `webhook/mod.rs` tests — this path resolves correctly as long as `webhook/events.rs` stays at the same path.

---

### `ferro-stripe/src/webhook/sync.rs` (stub)

**Pattern:** one-line comment, no imports, no pub items:
```rust
// Phase 141: SyncDispatcher implementation.
```

---

### `ferro-stripe/src/webhook/queue.rs` (stub)

**Pattern:** one-line comment, no imports, no pub items:
```rust
// Phase 141: ProcessStripeWebhook job relocated here.
```

---

### `ferro-stripe/src/lib.rs` (config — rewrite re-exports)

**Analog:** `ferro-stripe/src/lib.rs` (self — full rewrite, existing file lines 1-48)

**Target state** (replaces current content entirely):
```rust
//! # ferro-stripe
//!
//! Stripe payment and subscription integration for the Ferro framework.
//!
//! Provides capability-axis modules: checkout, refund, account, idempotency, webhook.

pub mod account;
pub mod checkout;
pub mod client;
pub mod config;
pub mod error;
pub mod idempotency;
pub mod refund;
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;
pub mod webhook;

pub use account::{billing_portal_url, create_account, create_link, retrieve_account};
pub use checkout::{CheckoutBuilder, CheckoutIntent, LineItem, Mode};
pub use client::Stripe;
pub use config::StripeConfig;
pub use error::Error;
pub use idempotency::{MemoryProcessedLog, ProcessedEventLog};
pub use webhook::events::{
    ProcessStripeWebhook, StripeCheckoutCompleted, StripeConnectPaymentSucceeded,
    StripeInvoicePaid, StripeSubscriptionDeleted, StripeSubscriptionUpdated,
};
pub use webhook::verify::verify_webhook;
```

**Removed from current lib.rs (breaking symbols):** `is_processed`, `create_account_link`, `create_connect_checkout`, `billing_portal_url` (old path), `create_subscription_checkout`, `plan_from_subscription`, `subscription_info_from_stripe`, `plan_satisfies`, `SubscriptionInfo`, `SubscriptionStatus`, `ConnectAccount`. These must each have a CHANGELOG entry.

**`pub mod connect` and `pub mod subscription`** — deleted, not present in target.

---

### `ferro-stripe/Cargo.toml` (config — add dashmap dep)

**Analog:** `ferro-stripe/Cargo.toml` (self — single line addition)

**Addition to `[dependencies]`** (after existing deps, following alphabetical convention visible in file):
```toml
dashmap = "6"
```

**Version pinning note (A3 / Pitfall 4):** The `version.workspace = true` in the `[package]` section means workspace root controls the version. Since workspace root is `0.2.2` and D-15 targets `0.4.0`, override ferro-stripe's version locally:
```toml
[package]
name = "ferro-stripe"
version = "0.4.0"   # overrides version.workspace = true
```
This prevents force-bumping all other workspace crates. Confirm with user before committing.

---

## Shared Patterns

### Error Conversion via `From<stripe::StripeError>`
**Source:** `ferro-stripe/src/error.rs` lines 25-29
**Apply to:** All fns in `checkout.rs`, `refund.rs`, `account.rs` that call `async-stripe` APIs
```rust
impl From<stripe::StripeError> for Error {
    fn from(e: stripe::StripeError) -> Self {
        Error::Stripe(e.to_string())
    }
}
```
All async-stripe calls terminate with `?`. No explicit `.map_err` needed for `StripeError`.

### String ID Parsing Pattern
**Source:** `ferro-stripe/src/connect/checkout.rs` lines 67-69 and `subscription/checkout.rs` lines 22-25
**Apply to:** Any fn in `account.rs`, `refund.rs`, `checkout.rs` that parses a Stripe ID from `&str`
```rust
let parsed_id: stripe::SomeId = raw_id
    .parse()
    .map_err(|_| Error::Stripe(format!("invalid id: {raw_id}")))?;
```

### Global Client Access
**Source:** `ferro-stripe/src/connect/checkout.rs` line 20; `subscription/checkout.rs` line 16
**Apply to:** All async fns in `checkout.rs`, `refund.rs`, `account.rs`
```rust
let client = crate::Stripe::client();
```
First line of every fn that calls the Stripe API. No dependency injection; global static.

### Inline Test Module Pattern
**Source:** `ferro-stripe/src/config.rs` lines 49-78; `webhook/mod.rs` lines 44-120
**Apply to:** All new files that have testable behavior
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]  // or #[tokio::test] for async
    fn descriptive_snake_case_name_describes_behavior() {
        // assert the behavior, not the implementation
    }
}
```
Test names follow the pattern: `{subject}_{condition}_{expected_result}` (e.g., `memory_log_true_then_false`, `checkout_create_missing_key_returns_err`).

### Consuming Builder Method Convention
**Source:** CONTEXT.md Established Patterns + CLAUDE.md Key Patterns
**Apply to:** All `with_*` / setter methods on `CheckoutBuilder`
```rust
// consuming self, return Self — no &mut self
pub fn line_item(self, item: LineItem) -> Self {
    let mut this = self;
    this.line_items.push(item);
    this
}
```

---

## No Analog Found

All files have analogs or are self-modifications. The following have only partial analogs:

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `ferro-stripe/src/idempotency.rs` | utility | event-driven | No existing trait-backed idempotency primitive in codebase; closest structural analog is `ferro_queue::Job` trait impl in `events.rs` (lines 113-156), but that is a different crate's re-exported macro |

---

## Metadata

**Analog search scope:** `ferro-stripe/src/` (all files read), `ferro-queue/src/` (async_trait pattern search)
**Files scanned:** 11 source files read directly
**Pattern extraction date:** 2026-04-20
