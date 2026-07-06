# Phase 140: Core Reshape - Research

**Researched:** 2026-04-20
**Domain:** ferro-stripe crate refactor — capability-axis module restructure, idempotency primitive, CheckoutBuilder, Stripe::with
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Target layout matches design doc §3.1 exactly: `checkout.rs`, `refund.rs`, `account.rs`, `webhook/{verify,events,sync,queue}`, `idempotency.rs`, `client.rs`. `connect/` and `subscription/` directories deleted in full.
- **D-02:** `webhook/sync.rs` and `webhook/queue.rs` created as stubs in this phase — directory structure exists, dispatch logic ships in Phase 141.
- **D-03:** `webhook/verify.rs` extracts the existing `verify_webhook` fn from `webhook/mod.rs`. `webhook/mod.rs` becomes a thin re-export shim or is removed.
- **D-04:** `#[async_trait] pub trait ProcessedEventLog: Send + Sync { async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error>; }`
- **D-05:** `MemoryProcessedLog` backed by `DashMap<String, ()>`. `dashmap` added explicitly to `ferro-stripe/Cargo.toml`.
- **D-06:** Module doc comment on `idempotency.rs` ships the recommended SQL schema verbatim.
- **D-07:** `idempotency_key` required before `create()`. Runtime check, not typestate — returns `Err(Error::MissingIdempotencyKey)`.
- **D-08:** `CheckoutIntent` carries `session_id: String`, `url: String`, `expires_at: DateTime<Utc>`, `idempotency_key: String`.
- **D-09:** `Mode` enum: `pub enum Mode { Payment, Subscription }`.
- **D-10:** `LineItem` struct: `name: String`, `description: Option<String>`, `unit_amount_cents: i64`, `quantity: u32`, `currency: String`.
- **D-11:** Stripe event structs do NOT implement `ferro_events::Event`. `SyncDispatcher` (Phase 141) is the sole handler registry. No `ferro_events::Event` impls written in Phase 140.
- **D-12:** `webhook/events.rs` substance is NOT reshaped in this phase. File moves location only if needed; content untouched beyond location change.
- **D-13:** `webhook::is_processed` free fn removed. `lib.rs` re-export removed. No callers remain (verify before removing).
- **D-14:** `Stripe::with(key: &str) -> stripe::Client` — returns scoped client; existing `Stripe::init` + global static unchanged.
- **D-15:** `ferro-stripe` version bumped to `0.4.0` in workspace root `Cargo.toml`.
- **D-16:** CHANGELOG.md entry documents every removed symbol and its replacement.

### Claude's Discretion

- Internal error type for `MissingIdempotencyKey` — add to existing `Error` enum in `error.rs`, name and message at implementer's discretion.
- `MemoryProcessedLog` concurrent test strategy — `tokio::spawn` + `tokio::join!` or similar; exact structure left to implementer.
- Whether `webhook/mod.rs` becomes a re-export shim or is deleted and replaced by explicit `pub mod` declarations in `lib.rs`.

### Deferred Ideas (OUT OF SCOPE)

- Webhook secret rotation support (second-secret variant for `verify_webhook`)
- Typestate builder for `CheckoutBuilder` to enforce `idempotency_key` at compile time
- `stripe_subscription_info` MCP tool update (Phase 142)
</user_constraints>

---

## Summary

Phase 140 is a structural reset of `ferro-stripe`: replace the product-axis module tree (`connect/`, `subscription/`) with the capability-axis tree, land `CheckoutBuilder`/`CheckoutIntent`, `ProcessedEventLog`/`MemoryProcessedLog`, and `Stripe::with(key)` in one coherent release, and remove the stubbed `is_processed` free fn. No dispatch changes; no event struct changes beyond file relocation.

The current codebase is fully understood. All source files have been read. Every symbol that must be deleted, moved, or added is catalogued below. The design doc (v11.6-FERRO-STRIPE-REFACTOR.md §3.1–3.5) specifies the exact API contracts; no design ambiguity remains.

The main execution risk is the `CheckoutBuilder::create()` implementation against the `async-stripe` API — specifically how `CreateCheckoutSession` accepts metadata, `payment_intent_data.transfer_data`, and `expires_at`. These are verified below against existing code in `connect/checkout.rs`.

**Primary recommendation:** Execute as three logical waves: (1) module skeleton + deletions, (2) new API surfaces (idempotency, builder, client), (3) lib.rs re-export update + CHANGELOG + version bump + CI gate.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Module layout restructure | ferro-stripe crate | — | Pure file/module reorganization within one crate |
| ProcessedEventLog trait | ferro-stripe/idempotency.rs | App DB layer | Trait in ferro; impl (SQL) in consumer app |
| MemoryProcessedLog | ferro-stripe/idempotency.rs | — | In-process DashMap; test/dev use only |
| CheckoutBuilder/Intent | ferro-stripe/checkout.rs | Stripe API | Builder assembles params; Stripe API creates session |
| Refund API | ferro-stripe/refund.rs | Stripe API | Thin wrapper over stripe::Refund CRUD |
| Account consolidation | ferro-stripe/account.rs | Stripe API | Merges create_account, create_link, billing_portal_url |
| Stripe::with(key) | ferro-stripe/client.rs | — | Returns ephemeral Client, no global state change |
| webhook/verify.rs | ferro-stripe/webhook/verify.rs | — | Pure fn extraction, no logic change |
| webhook/events.rs | ferro-stripe/webhook/events.rs | — | File relocated; content frozen until Phase 141 |
| webhook/sync.rs stub | ferro-stripe/webhook/sync.rs | — | Empty stub; Phase 141 fills it |
| webhook/queue.rs stub | ferro-stripe/webhook/queue.rs | — | Empty stub; Phase 141 moves ProcessStripeWebhook here |

---

## Standard Stack

### Core (already in Cargo.toml — verified)

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| async-stripe | 0.41 | Stripe API client | Features: billing, checkout, connect, webhook-events |
| thiserror | 2 | Error derive | Used by existing `Error` enum |
| serde / serde_json | 1 | Serialization | CheckoutIntent, LineItem derive |
| chrono | 0.4 + serde | DateTime<Utc> for CheckoutIntent.expires_at | Already a dep |
| async-trait | 0.1 | ProcessedEventLog trait | Already a dep |
| tokio | 1 (dev) | Test runtime | tokio::test, tokio::spawn |

### New Dependency

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| dashmap | 6.x | MemoryProcessedLog backing store | Concurrent HashMap; transitive in workspace, needs explicit dep |

[ASSUMED] `dashmap` version — workspace Cargo.lock has it as transitive dep; the current latest is 6.x. Verify with `cargo tree -i dashmap` before pinning.

**Installation addition to ferro-stripe/Cargo.toml:**
```toml
dashmap = "6"
```

---

## Architecture Patterns

### System Architecture: Module Transformation

```
BEFORE (product axis)                    AFTER (capability axis)
─────────────────────────────────────    ─────────────────────────────────────
ferro-stripe/src/                        ferro-stripe/src/
  client.rs          → EXTEND             client.rs         (+ Stripe::with)
  config.rs          → KEEP               config.rs         (unchanged)
  connect/           → DELETE             checkout.rs       (NEW: Builder/Intent)
    mod.rs                                refund.rs         (NEW: create/retrieve)
    checkout.rs                           account.rs        (NEW: consolidation)
  subscription/      → DELETE             idempotency.rs    (NEW: trait + MemLog)
    mod.rs                                webhook/
    checkout.rs                             verify.rs       (EXTRACTED from mod.rs)
    sync.rs                                 events.rs       (MOVED, content frozen)
  webhook/           → RESHAPE              sync.rs         (STUB)
    mod.rs                                  queue.rs        (STUB)
    events.rs                           error.rs            (+ MissingIdempotencyKey)
    handler.rs       → DELETE           testing.rs          (unchanged)
  error.rs           → EXTEND
  testing.rs         → KEEP
```

### Deletion Inventory (verified by source read)

| File / Symbol | Action | Replacement |
|---|---|---|
| `connect/` directory (mod.rs, checkout.rs) | Delete | `checkout.rs`, `account.rs` |
| `subscription/` directory (mod.rs, checkout.rs, sync.rs) | Delete | `checkout.rs`, `account.rs` |
| `webhook/handler.rs` | Delete | App-side webhook handler (not ferro's responsibility) |
| `webhook/mod.rs` | Delete or convert to re-export shim | `webhook/verify.rs` + `lib.rs` pub mods |
| `webhook::is_processed` fn | Remove | `ProcessedEventLog::try_mark_processed` |
| `create_connect_checkout` | Remove from lib.rs re-export | `CheckoutBuilder::new(Mode::Payment).destination(...)` |
| `create_subscription_checkout` | Remove from lib.rs re-export | `CheckoutBuilder::new(Mode::Subscription)...` |
| `billing_portal_url` (subscription::checkout) | Remove from lib.rs re-export | `account::billing_portal_url` |
| `plan_from_subscription`, `subscription_info_from_stripe` | Remove from lib.rs re-export | No replacement in Phase 140 (subscription helpers; CHANGELOG note) |
| `ConnectAccount` struct | Remove from lib.rs re-export | No direct replacement (Connect account ID is just `String` in account.rs) |
| `SubscriptionInfo`, `SubscriptionStatus`, `plan_satisfies` | Remove from lib.rs re-export | No replacement in Phase 140 (CHANGELOG note) |

### Pattern: CheckoutBuilder (consuming builder)

[VERIFIED: existing connect/checkout.rs and CONTEXT.md D-07 through D-10]

```rust
// checkout.rs
use crate::{Error, client::Stripe};
use chrono::{DateTime, Utc};
use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCheckoutSessionPaymentIntentData,
    CreateCheckoutSessionPaymentIntentDataTransferData,
};

pub enum Mode { Payment, Subscription }

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
    pub expires_at: DateTime<Utc>,
    pub idempotency_key: String,
}

impl CheckoutBuilder {
    pub fn new(mode: Mode) -> Self { ... }
    pub fn line_item(self, item: LineItem) -> Self { ... }
    pub fn success_url(self, url: &str) -> Self { ... }
    pub fn cancel_url(self, url: &str) -> Self { ... }
    pub fn metadata(self, key: &str, value: &str) -> Self { ... }
    pub fn customer_email(self, email: &str) -> Self { ... }
    pub fn customer_email_opt(self, email: Option<&str>) -> Self { ... }
    pub fn destination(self, account_id: &str, fee_cents: Option<i64>) -> Self { ... }
    pub fn idempotency_key(self, key: &str) -> Self { ... }
    pub async fn create(self) -> Result<CheckoutIntent, Error> { ... }
}
```

**Note on `expires_at`:** `stripe::CheckoutSession` returns `expires_at: i64` (Unix timestamp). Convert with `chrono::Utc.timestamp_opt(session.expires_at, 0).single().unwrap_or_else(Utc::now)`. [VERIFIED: existing code uses this pattern in subscription/sync.rs]

**Note on `idempotency_key` in Stripe API:** The async-stripe client passes idempotency keys via a `stripe::RequestStrategy`. [ASSUMED: exact API call is `Client::with_strategy(RequestStrategy::Idempotent(key))`] — verify against async-stripe 0.41 docs before implementing. The `idempotency_key` field on `CheckoutIntent` is the key that was used, stored for caller correlation.

### Pattern: ProcessedEventLog + MemoryProcessedLog

[VERIFIED: CONTEXT.md D-04, D-05; design doc §3.5]

```rust
// idempotency.rs
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

use async_trait::async_trait;
use crate::Error;

#[async_trait]
pub trait ProcessedEventLog: Send + Sync {
    async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error>;
}

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
        // insert returns None if key was absent (first time) → Ok(true)
        // insert returns Some(()) if key was present → Ok(false)
        Ok(self.seen.insert(event_id.to_string(), ()).is_none())
    }
}
```

### Pattern: Stripe::with(key)

[VERIFIED: client.rs; CONTEXT.md D-14]

```rust
// Extend client.rs
impl Stripe {
    /// Returns a scoped Stripe client for the given API key.
    ///
    /// Use for per-tenant direct-charges scenarios where a different
    /// Stripe account key is needed per request.
    /// Does not affect the global static client initialized by [`Stripe::init`].
    pub fn with(api_key: &str) -> stripe::Client {
        stripe::Client::new(api_key)
    }
}
```

### Pattern: webhook/verify.rs extraction

[VERIFIED: webhook/mod.rs — `verify_webhook` fn is self-contained, no module state]

```rust
// webhook/verify.rs
use crate::Error;

pub fn verify_webhook(raw_body: &str, signature: &str, secret: &str) -> Result<stripe::Event, Error> {
    stripe::Webhook::construct_event(raw_body, signature, secret)
        .map_err(|e| Error::WebhookVerification(e.to_string()))
}
```

The existing tests in `webhook/mod.rs` that test `verify_webhook` move with the fn to `webhook/verify.rs`. The `is_processed` test (`is_processed_returns_false_for_unseen_ids`) is deleted — its fn is removed.

### Pattern: error.rs extension

[VERIFIED: error.rs; CONTEXT.md Claude's Discretion]

Add to existing `Error` enum:

```rust
/// Idempotency key not set on CheckoutBuilder before calling create().
#[error("idempotency key required: call .idempotency_key() before .create()")]
MissingIdempotencyKey,
```

### Pattern: lib.rs re-exports (target state)

```rust
pub mod checkout;
pub mod client;
pub mod config;
pub mod error;
pub mod idempotency;
pub mod refund;
pub mod account;
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;
pub mod webhook;

pub use checkout::{CheckoutBuilder, CheckoutIntent, LineItem, Mode};
pub use client::Stripe;
pub use config::StripeConfig;
pub use error::Error;
pub use idempotency::{MemoryProcessedLog, ProcessedEventLog};
pub use refund::Refund;  // or just the fns if no struct
pub use webhook::verify::verify_webhook;
// webhook::events re-exports (existing event structs survive frozen)
pub use webhook::events::{
    ProcessStripeWebhook, StripeCheckoutCompleted, StripeConnectPaymentSucceeded,
    StripeInvoicePaid, StripeSubscriptionDeleted, StripeSubscriptionUpdated,
};
```

**Removed from lib.rs (breaking):** `is_processed`, `create_account_link`, `create_connect_checkout`, `billing_portal_url`, `create_subscription_checkout`, `plan_from_subscription`, `subscription_info_from_stripe`, `plan_satisfies`, `SubscriptionInfo`, `SubscriptionStatus`, `ConnectAccount`.

**Added to lib.rs:** `account::*` fns (`create_account`, `create_link`, `retrieve_account`, `billing_portal_url`), `refund` fns/structs, `CheckoutBuilder`, `CheckoutIntent`, `LineItem`, `Mode`, `ProcessedEventLog`, `MemoryProcessedLog`.

### Pattern: refund.rs

[VERIFIED: design doc §3.1; existing code in connect/checkout.rs for async-stripe call shape]

```rust
// refund.rs
pub async fn create(
    charge_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReason>,
) -> Result<stripe::Refund, Error> { ... }

pub async fn retrieve(refund_id: &str) -> Result<stripe::Refund, Error> { ... }
```

[ASSUMED: `stripe::Refund::create` and `stripe::Refund::retrieve` are the correct async-stripe 0.41 call sites. The existing code pattern (`CheckoutSession::create(client, params).await?`) confirms the general shape; refund-specific params need verification against async-stripe docs.]

### Pattern: account.rs

[VERIFIED: all three source functions read from connect/checkout.rs and subscription/checkout.rs]

Consolidates from existing code:
- `create_account` — new fn (creates a Connect Express/Standard account)
- `create_link` — moved verbatim from `connect::checkout::create_account_link`
- `retrieve_account` — new fn (fetches account details)
- `billing_portal_url` — moved verbatim from `subscription::checkout::billing_portal_url`

`create_account` and `retrieve_account` are new. [ASSUMED: `stripe::Account::create` and `stripe::Account::retrieve` are the correct async-stripe 0.41 call sites.]

### Stub files (D-02)

```rust
// webhook/sync.rs
// Phase 141: SyncDispatcher implementation.

// webhook/queue.rs
// Phase 141: ProcessStripeWebhook job relocated here.
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Concurrent hash map for MemoryProcessedLog | Custom Mutex<HashMap> | `dashmap::DashMap` — lock-free concurrent HashMap |
| HMAC-SHA256 webhook verification | Custom crypto | `stripe::Webhook::construct_event` (already used) |
| Idempotency key passing to Stripe API | Custom header injection | `stripe::RequestStrategy::Idempotent(key)` |
| Error derive | Manual Display/Error impls | `thiserror` (already used) |
| Async trait | Async fn in trait (Rust < 1.75 stable AFIT) | `async-trait = "0.1"` (already a dep) |

---

## Runtime State Inventory

This is a refactor phase — no stored data, no live service config, no OS-registered state, no secrets, no build artifacts affected beyond the crate itself.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — ferro-stripe is a library crate, no persistent state | None |
| Live service config | None | None |
| OS-registered state | None | None |
| Secrets/env vars | `STRIPE_API_KEY`, `STRIPE_WEBHOOK_SECRET`, `STRIPE_CONNECT_WEBHOOK_SECRET` — read by `StripeConfig::from_env()`, unchanged | None |
| Build artifacts | `ferro-stripe` on crates.io (published) — 0.4.0 is a new version, no artifact conflict | Publish 0.4.0 after CI green |

The only cross-repo concern: gestiscilo uses `ferro-stripe` via crates.io or `[patch.crates-io]`. It needs a CHANGELOG to migrate off the removed symbols. That is the CHANGELOG.md deliverable, not a code task.

---

## Common Pitfalls

### Pitfall 1: Leaving `handler.rs` callers of `is_processed`

**What goes wrong:** `webhook/handler.rs` re-exports or calls nothing from `is_processed` (confirmed by read — it uses `verify_webhook` and `ProcessStripeWebhook`, not `is_processed`). But `webhook/mod.rs` test `is_processed_returns_false_for_unseen_ids` references `is_processed`. That test must be deleted along with the fn.

**How to avoid:** After deleting `is_processed`, run `cargo check` immediately. The compiler will surface any remaining callers.

### Pitfall 2: `webhook/events.rs` tests reference `signed_webhook_payload`

**What goes wrong:** `signed_webhook_payload` is in `webhook/events.rs` behind `#[cfg(any(test, feature = "test-helpers"))]`. The tests in `webhook/mod.rs` import it via `use crate::webhook::events::signed_webhook_payload`. When `webhook/mod.rs` is restructured, that import path must be updated to `webhook::verify` tests or kept in events.rs (its natural home since it has HMAC deps).

**How to avoid:** Keep `signed_webhook_payload` in `testing.rs` (it's already the correct location per `lib.rs` — it's pub in events.rs but conceptually belongs in testing). Alternatively leave it in `events.rs` and update the import in `verify.rs` tests. Either is fine; just be consistent.

**Note:** `testing.rs` currently re-exports nothing; the actual `signed_webhook_payload` fn lives in `webhook/events.rs`. If moving to `testing.rs`, delete from `events.rs` and update all callers.

### Pitfall 3: `CheckoutSession.expires_at` field type

**What goes wrong:** `stripe::CheckoutSession.expires_at` may be `Option<i64>` not `i64` in async-stripe 0.41. Unwrapping without checking gives a panic.

**How to avoid:** Use `.map(|ts| Utc.timestamp_opt(ts, 0).single()).flatten().unwrap_or_else(Utc::now)`. Pattern already used in subscription/sync.rs for `trial_end`.

### Pitfall 4: Workspace version bump affects all published crates

**What goes wrong:** `version.workspace = true` in ferro-stripe means bumping workspace root `version` from `0.2.2` to `0.4.0` would bump ALL crates in the workspace to 0.4.0. That is a major version leap for unrelated crates.

**How to avoid:** Per CONTEXT.md D-15, `ferro-stripe` version is bumped via workspace root. BUT this is correct only if the workspace root's version is the ferro-stripe-specific version, or if `ferro-stripe` overrides the version locally. Check whether `ferro-stripe/Cargo.toml` should override with `version = "0.4.0"` directly instead of `version.workspace = true`. [ASSUMED: the design doc's §4 says "0.3.x → 0.4", implying ferro-stripe was already versioned independently. The workspace root is `0.2.2`. Overriding locally in ferro-stripe/Cargo.toml is likely the correct move.]

**Resolution:** Set `version = "0.4.0"` directly in `ferro-stripe/Cargo.toml` (override workspace) rather than bumping the workspace root. Confirm this is the intent before implementation.

### Pitfall 5: DashMap `insert` semantics

**What goes wrong:** `DashMap::insert(k, v)` returns `Option<V>` — `None` if key was absent, `Some(old_value)` if key was present. This is the correct semantic for `try_mark_processed`: `is_none()` → first time → `Ok(true)`, `is_some()` → already seen → `Ok(false)`.

**How to avoid:** Use `self.seen.insert(event_id.to_string(), ()).is_none()`. This is safe under concurrent access — DashMap's shard locking makes the insert atomic per key.

### Pitfall 6: `webhook/mod.rs` pub re-exports must stay compilable during transition

**What goes wrong:** If `webhook/mod.rs` is deleted before `webhook/verify.rs` exists, the crate breaks. The module restructure must add files before deleting the old ones, or be done atomically in a single commit.

**How to avoid:** Add new files first, update `lib.rs` pub mods, then delete old files.

---

## Code Examples

### Concurrent MemoryProcessedLog test

```rust
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

    // Exactly one must be true, one must be false
    assert_ne!(v1, v2, "concurrent inserts must apply exactly once");
}
```

### MemoryProcessedLog true-then-false contract test

```rust
#[tokio::test]
async fn memory_log_true_then_false() {
    let log = MemoryProcessedLog::new();
    assert_eq!(log.try_mark_processed("evt_001").await.unwrap(), true);
    assert_eq!(log.try_mark_processed("evt_001").await.unwrap(), false);
    assert_eq!(log.try_mark_processed("evt_002").await.unwrap(), true);
}
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | tokio-test via `#[tokio::test]` (dev-dependency: tokio 1 full+test-util) |
| Config file | None — inline test modules per file |
| Quick run command | `cargo test -p ferro-stripe` |
| Full suite command | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` |

### Phase Requirements → Test Map

| SC | Behavior | Test Type | Automated Command | Notes |
|----|----------|-----------|-------------------|-------|
| SC-2 | `ProcessedEventLog` trait exists and compiles | compile | `cargo check -p ferro-stripe` | Wave 0 |
| SC-3 | `MemoryProcessedLog` true-then-false | unit | `cargo test -p ferro-stripe memory_log_true_then_false` | New test |
| SC-12 | Concurrent `try_mark_processed` applies once | unit | `cargo test -p ferro-stripe memory_log_concurrent_insert` | New test |
| SC-6 | `create()` without key returns `Err(MissingIdempotencyKey)` | unit | `cargo test -p ferro-stripe checkout_create_missing_key` | New test |
| SC-10 | `is_processed` removed, no callers | compile | `cargo check -p ferro-stripe` | Absence verified by build |
| SC-11 | lib.rs re-exports clean | compile | `cargo check -p ferro-stripe` | Absence verified by build |
| SC-13 | `cargo test --all-features` + clippy pass | CI gate | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` | Phase gate |

### Wave 0 Gaps

- [ ] `ferro-stripe/src/idempotency.rs` — new file, creates ProcessedEventLog + MemoryProcessedLog
- [ ] `ferro-stripe/src/checkout.rs` — new file, CheckoutBuilder + CheckoutIntent
- [ ] `ferro-stripe/src/refund.rs` — new file, create + retrieve
- [ ] `ferro-stripe/src/account.rs` — new file, consolidated account fns
- [ ] `ferro-stripe/src/webhook/verify.rs` — extracted from mod.rs
- [ ] `ferro-stripe/src/webhook/sync.rs` — stub
- [ ] `ferro-stripe/src/webhook/queue.rs` — stub
- [ ] `CHANGELOG.md` — must document all breaking changes with migration paths

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `dashmap` is version 6.x in Cargo.lock | Standard Stack | Wrong major version → compile error; fix: `cargo tree -i dashmap` |
| A2 | `stripe::RequestStrategy::Idempotent(key)` is the correct async-stripe 0.41 API for passing idempotency keys | Architecture Patterns (CheckoutBuilder) | Wrong API → compile error; fix: check async-stripe 0.41 docs |
| A3 | `ferro-stripe/Cargo.toml` should override workspace version with `version = "0.4.0"` directly rather than via workspace | Common Pitfalls (Pitfall 4) | Bumping workspace root to 0.4.0 would version-bump all crates | 
| A4 | `stripe::Account::create` and `stripe::Account::retrieve` are the correct async-stripe call sites for account.rs | Architecture Patterns (account.rs) | Wrong fn name → compile error; fix: check async-stripe docs |
| A5 | `stripe::Refund::create` and `stripe::Refund::retrieve` are correct for refund.rs | Don't Hand-Roll | Wrong fn name → compile error; fix: check async-stripe docs |
| A6 | `stripe::CheckoutSession.expires_at` is `Option<i64>` | Code Examples | If `i64` not `Option<i64>`, .map() is wrong; check stripe-rust type def |

---

## Open Questions

1. **Workspace version override (A3)**
   - What we know: `ferro-stripe/Cargo.toml` currently uses `version.workspace = true`; workspace root is `0.2.2`; design doc says bump to `0.4.0`
   - What's unclear: Should ferro-stripe break from workspace versioning and use its own `version = "0.4.0"`, or does the workspace root need to reach `0.4.0`?
   - Recommendation: Override locally in ferro-stripe/Cargo.toml — other crates should not be force-bumped to 0.4.0. Confirm with user before committing the version bump.

2. **`handler.rs` fate**
   - What we know: `webhook/handler.rs` provides `handle_platform_webhook` and `handle_connect_webhook` which are helper fns that verify + enqueue. These are currently pub but not re-exported in lib.rs.
   - What's unclear: Should handler.rs be deleted (dispatch pattern changes to app-side) or kept as a convenience until Phase 141?
   - Recommendation: Delete — it queues `ProcessStripeWebhook` which moves to `webhook/queue.rs` in Phase 141; keeping it creates a dangling dependency on queue.rs content that doesn't exist yet in 140.

3. **`SubscriptionInfo`/`SubscriptionStatus`/`plan_satisfies` removal**
   - What we know: These are re-exported from lib.rs and come from `subscription/` which is deleted.
   - What's unclear: Do any consumers (gestiscilo) use these? CHANGELOG needs a migration note.
   - Recommendation: Remove with CHANGELOG entry. Gestiscilo migration is deferred to its Phase 95.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code-only changes within a single Rust crate. No external tools or services required beyond the existing Rust toolchain.

---

## Sources

### Primary (HIGH confidence)

- `ferro-stripe/src/lib.rs` — verified all current pub re-exports
- `ferro-stripe/src/webhook/mod.rs` — verified `is_processed` fn and `verify_webhook` fn
- `ferro-stripe/src/webhook/events.rs` — verified all event structs carry `event_json`, implement `ferro_events::Event`
- `ferro-stripe/src/webhook/handler.rs` — verified `handle_platform_webhook` / `handle_connect_webhook` shape
- `ferro-stripe/src/connect/checkout.rs` — verified `create_connect_checkout`, `create_account_link` fns
- `ferro-stripe/src/connect/mod.rs` — verified `ConnectAccount` struct
- `ferro-stripe/src/subscription/checkout.rs` — verified `create_subscription_checkout`, `billing_portal_url` fns
- `ferro-stripe/src/subscription/sync.rs` — verified `plan_from_subscription`, `subscription_info_from_stripe` fns
- `ferro-stripe/src/client.rs` — verified `Stripe::init`, `Stripe::client`, `Stripe::config`
- `ferro-stripe/src/error.rs` — verified Error enum; `EventAlreadyProcessed` already exists
- `ferro-stripe/Cargo.toml` — verified deps: async-trait ✓, chrono ✓, dashmap absent (needs addition)
- `.planning/phases/140-core-reshape/140-CONTEXT.md` — all decisions
- `.planning/research/v11.6-FERRO-STRIPE-REFACTOR.md` — full design spec

### Secondary (MEDIUM confidence)

- async-stripe 0.41 API shape: inferred from existing usage patterns in connect/checkout.rs — idempotency key passing and refund/account fns unverified [ASSUMED A2, A4, A5]

---

## Metadata

**Confidence breakdown:**
- Module layout and deletions: HIGH — all source files read, exact symbol inventory complete
- New API surfaces (ProcessedEventLog, MemoryProcessedLog, Stripe::with): HIGH — design doc is prescriptive, patterns are standard
- CheckoutBuilder async-stripe integration: MEDIUM — builder shape is HIGH confidence; idempotency key passing mechanism is ASSUMED
- refund.rs / account.rs new fns: MEDIUM — fn signatures are clear; async-stripe 0.41 call sites unverified
- Version bump strategy: MEDIUM — workspace override decision needs confirmation

**Research date:** 2026-04-20
**Valid until:** 2026-05-20 (stable domain; async-stripe 0.41 API unlikely to change)
