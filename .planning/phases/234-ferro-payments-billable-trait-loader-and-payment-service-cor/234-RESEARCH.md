# Phase 234: Billable trait + Loader + PaymentService core — Research

**Researched:** 2026-06-17
**Domain:** Rust async traits, Stripe abstraction seam, SeaORM GuardedUpdate, ferro-payments crate extension
**Confidence:** HIGH — all critical claims verified against live source files

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Stripe injection seam:**
- D-01: `ferro_stripe::Stripe` is a global static facade (OnceLock). No injectable `Client` type.
- D-02: Define `StripeGateway` trait local to ferro-payments. Minimum surface: `create_checkout_session` + `create_refund`.
- D-03: `PaymentService` holds `stripe: Arc<dyn StripeGateway>`. Production wraps ferro-stripe; tests inject mock.

**`Billable` trait:**
- D-04: `#[async_trait]`. Sync accessors: `kind`, `id`, `tenant_id`, `amount_cents`, `currency`, `checkout_line_description`. Async side effects: `on_paid`, `on_released`, `on_refunded` take `&DatabaseTransaction`.
- D-05: Default `fn connect_account_id(&self) -> Option<String> { None }` on `Billable`.
- D-06: `Billable` is NOT `Clone`. Pass `&dyn Billable` everywhere.

**`BillableLoader` trait:**
- D-07: `#[async_trait] async fn load(&self, kind: BillableKind, id: i64) -> Result<Option<Box<dyn Billable>>, PaymentError>`.
- D-08: No separate `tenant_id` argument — loader extracts it.

**`PaymentService` fields and constructor:**
- D-09: Store only `db: DatabaseConnection`, `stripe: Arc<dyn StripeGateway>`, `loader: L`, `return_url_builder: Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>`. `processed_log` deferred to phase 235.
- D-10: `loader: L` kept with `#[allow(dead_code)]` + comment. Generic param stays so 235 adds methods without reshaping the type.
- D-11: Introduce `ReturnUrls { success_url: String, cancel_url: String }` and `CheckoutUrl(String)` newtype.

**`start_checkout(billable, ttl)` flow (D-12..D-14):**
1. `create_reserved(...)` with `expires_at = Utc::now() + ttl`.
2. Build `CheckoutBuilder::new(Mode::Payment)` with line item + URLs + optional `.destination(account_id, fee)`.
3. Set deterministic idempotency key derived from intent id before `.create()`.
4. On success, attach `stripe_session_id` + snapshot `application_fee_cents` via `GuardedUpdate`.
5. Return `CheckoutUrl`.
- D-13: `payment_intent_id` not available at checkout creation. Leave NULL.
- D-14: On Stripe failure after INSERT, leave reserved row for phase-236 reaper. Return `PaymentError::Stripe(..)`.

**`request_refund(intent_id, amount_cents)` (D-15..D-17):**
- D-15: Load by id → require `status = paid` AND `charge_id` present → snapshot `refund_amount_cents` via `GuardedUpdate WHERE refund_amount_cents IS NULL` → call `stripe.create_refund(...)`.
- D-16: Does NOT flip status to `refunded` — that is the webhook's job (phase 235).
- D-17: No `refund_requested` enum variant. "Refund-in-flight" = predicate `status='paid' AND refund_amount_cents IS NOT NULL AND refunded_at IS NULL`.

**`PaymentError` (D-18):** Extend 233 enum with `Stripe(#[from] ferro_stripe::Error)`, `Loader(Box<dyn std::error::Error + Send + Sync>)`, `AutoRefundTriggered { reason: AutoRefundReason }`. Define `AutoRefundReason` enum.

**Manifest / publish wave (D-19..D-21):**
- D-19: Add `ferro-stripe = { path = "../ferro-stripe", version = "0.9" }` to ferro-payments dependencies.
- D-20: Unit tests use `StripeGateway` mock only. No `Stripe::init(...)` in tests.
- D-21: Move `ferro-payments` from Wave 1b to new Wave 1c (after 1b index-wait). Update `WAVE1B_CRATES` to drop ferro-payments; add `WAVE1C_CRATES="ferro-payments"` step + index-wait before Wave 2.

### Claude's Discretion

- Exact module split: planner may fold `request_refund` into `service.rs` or keep `refund.rs`.
- Exact `StripeGateway` method names/`CheckoutRequest` shape.
- Whether the mock lives in `#[cfg(test)]` or a `test-helpers` feature.
- `AutoRefundReason` variant names.
- Whether `CheckoutUrl`/`ReturnUrls` get builder ergonomics.
- Idempotency key format string (any deterministic per-intent value).

### Deferred Ideas (OUT OF SCOPE)

- `wire_dispatcher` + typed webhook handlers + idempotency via `ProcessedEventLog` + auto-refund fallback — phase 235.
- `release_expired` / `ReconcileRefundsInFlight` reapers + workspace test bin + publish — phase 236.
- Provider abstraction beyond Stripe.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PAY-POLY-SVC-01 | `Billable` trait + `BillableKind` — object-safe, `#[async_trait]` | Verified: `async-trait = "0.1"` already in ferro-payments Cargo.toml; `#[async_trait]` pattern confirmed in lifecycle.rs test harness |
| PAY-POLY-SVC-02 | `BillableLoader` trait — `async fn load(kind, id) -> Result<Option<Box<dyn Billable>>, PaymentError>` | Verified: object-safe via `#[async_trait]`; `Box<dyn Billable>` return requires Billable: `Send + Sync` (confirmed by D-04) |
| PAY-POLY-SVC-03 | `PaymentService<L>` with `start_checkout` and `request_refund` | Verified: lifecycle building blocks exist; `attach_session` update must be a new lifecycle fn (see Pattern section) |
| PAY-POLY-SVC-04 | `StripeGateway` trait seam + production impl + test mock | Verified: ferro-stripe has no injectable Client; seam is mandatory (confirmed from `client.rs`, `checkout.rs`, `refund.rs`) |
| PAY-POLY-SVC-05 | Extended `PaymentError` + `AutoRefundReason` + publish.yml Wave 1c | Verified: `ferro_stripe::Error` is a public `thiserror` enum suitable for `#[from]`; Wave 1b currently includes ferro-payments alongside ferro-stripe (intra-wave dep confirmed) |

</phase_requirements>

---

## Summary

Phase 234 adds the orchestration layer to ferro-payments on top of the phase-233 data layer. The central design challenge is that `ferro_stripe::Stripe` is a global static facade — the spec's proposed `Arc<ferro_stripe::Client>` does not correspond to any real type. The `StripeGateway` trait seam defined in D-02/D-03 is the solution: a small local trait in ferro-payments that the production impl delegates to ferro-stripe free functions, and that tests mock without touching the global static.

All three code verification objectives from the CONTEXT.md are confirmed. The `CheckoutIntent` struct contains `{ session_id, url, expires_at, idempotency_key }` — no `payment_intent_id` field (D-13 confirmed). The `refund::create` function accepts an `idempotency_key` parameter but explicitly does not forward it to async-stripe 0.41 (`let _ = idempotency_key;`) — the D-15 app-layer dedup via `GuardedUpdate WHERE refund_amount_cents IS NULL` is mandatory. The publish.yml `WAVE1B_CRATES` currently includes both `ferro-stripe` and `ferro-payments` in the same wave — the D-21 intra-wave dependency violation is confirmed, requiring a Wave 1c step.

The lifecycle layer from phase 233 provides `create_reserved`, `mark_paid/released/refunded`, `find_active_for`, and `find_by_stripe_session`. There is no existing lifecycle function to attach `stripe_session_id` + `application_fee_cents` to a reserved row after the Stripe API call; that function must be added in this phase (named `attach_session` or inlined in the service — discretion of planner).

**Primary recommendation:** Implement in module order: `billable.rs` + `loader.rs` first (pure traits), then `StripeGateway` trait in `gateway.rs` or at the top of `service.rs`, then `service.rs` with `PaymentService::new/start_checkout/request_refund`, then extend `error.rs`. All tests go in `service.rs` `#[cfg(test)]` using the in-memory SQLite harness from lifecycle.rs as template.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `Billable`/`BillableLoader` trait definitions | ferro-payments crate | — | Domain abstraction owned by the payment layer; consumer implements |
| `StripeGateway` trait + production impl | ferro-payments crate | ferro-stripe (delegated) | Seam lives in the consumer of the static facade, not in ferro-stripe itself |
| `PaymentService` orchestration | ferro-payments crate | ferro-orm (GuardedUpdate) | All DB state mutations go through lifecycle fns + GuardedUpdate |
| Stripe API calls (checkout/refund) | ferro-stripe (static facade) | StripeGateway mock in tests | Tests never call Stripe::init; production delegates to ferro-stripe free fns |
| Idempotency dedup for refund | ferro-payments (GuardedUpdate filter) | — | async-stripe 0.41 does not forward idempotency keys; app-layer guard is mandatory |
| publish.yml wave ordering | `.github/workflows/publish.yml` | — | Wave 1c for ferro-payments after ferro-stripe is indexed |

---

## Standard Stack

### Core (verified against live files)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `async-trait` | `0.1` | Object-safe async trait impls | Already in ferro-payments Cargo.toml [VERIFIED: ferro-payments/Cargo.toml] |
| `sea-orm` | `1.0` | DB access + `DatabaseTransaction` | Required for `on_paid`/`on_released`/`on_refunded` signatures [VERIFIED] |
| `ferro-orm` | `0.2` | `GuardedUpdate` for atomic conditional UPDATE | Already a dependency; `exec_at_most_one` is the exact method for refund dedup [VERIFIED: ferro-orm/src/guarded.rs] |
| `ferro-stripe` | `0.9.0` | Stripe API wrapping | New dependency for phase 234; confirmed version `0.9.0` [VERIFIED: ferro-stripe/Cargo.toml] |
| `thiserror` | `2` | Error derive | Already in ferro-payments; `ferro_stripe::Error` uses thiserror [VERIFIED] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `chrono` | `0.4` | `Duration` + `Utc::now()` | `start_checkout` TTL arithmetic |
| `std::sync::Arc` | stdlib | `Arc<dyn StripeGateway>` + `Arc<dyn Fn(...)>` | Shared ownership in `PaymentService` |

**Installation (new dependency only):**
```bash
# In ferro-payments/Cargo.toml [dependencies]:
# ferro-stripe = { path = "../ferro-stripe", version = "0.9" }
```

---

## Architecture Patterns

### System Architecture Diagram

```
Consumer code
    │
    │  start_checkout(&dyn Billable, ttl)
    ▼
PaymentService<L: BillableLoader>
    │
    ├─► lifecycle::create_reserved(...)          ─► payment_intents row (status=reserved)
    │
    ├─► CheckoutRequest { amount, currency, ... }
    │       │
    │       ▼
    │   Arc<dyn StripeGateway>
    │       │  (production: StripeClientGateway)     (test: MockStripeGateway)
    │       │          │                                      │
    │       │          ▼                                      ▼
    │       │   CheckoutBuilder::new(Mode::Payment)    records call, returns canned CheckoutIntent
    │       │   .line_item(...).success_url(...).idempotency_key(...)
    │       │   .destination(acct, fee)  [if connect_account_id is Some]
    │       │   .create().await
    │       │
    │       ▼
    │   CheckoutIntent { session_id, url, expires_at, idempotency_key }
    │
    ├─► GuardedUpdate (attach_session: set stripe_session_id + application_fee_cents)
    │
    └─► Ok(CheckoutUrl("https://..."))

Consumer code
    │
    │  request_refund(intent_id, amount_cents)
    ▼
PaymentService<L>
    │
    ├─► Entity::find_by_id(intent_id)   → NotFound if absent
    ├─► assert status=paid AND charge_id IS NOT NULL   → StatusPrecondition if not
    ├─► GuardedUpdate WHERE refund_amount_cents IS NULL  → Ok(false) = already in flight, no-op
    │       SET refund_amount_cents = amount_cents
    │
    ├─► stripe.create_refund(charge_id, Some(amount_cents), idempotency_key)
    │
    └─► Ok(())
```

### Recommended Project Structure

```
ferro-payments/src/
├── lib.rs                   # re-exports: Billable, BillableLoader, PaymentService, StripeGateway,
│                            #             ReturnUrls, CheckoutUrl, AutoRefundReason, PaymentError
├── billable.rs              # Billable trait + connect_account_id defaulted method
├── loader.rs                # BillableLoader trait
├── service.rs               # PaymentService<L>, StripeGateway, CheckoutRequest, ReturnUrls, CheckoutUrl
│                            # (or gateway.rs for StripeGateway if planner prefers)
├── error.rs                 # PaymentError (extended) + AutoRefundReason
├── intent/
│   ├── mod.rs
│   ├── entity.rs            # (unchanged from 233)
│   ├── status.rs            # (unchanged from 233)
│   └── lifecycle.rs         # + attach_session() new fn
└── migration/               # (unchanged from 233)
```

### Pattern 1: `StripeGateway` trait + `CheckoutRequest`

**What:** A local trait in ferro-payments that wraps the two ferro-stripe operations `PaymentService` needs. Makes the orchestrator unit-testable without touching ferro-stripe.

**When to use:** All Stripe calls inside `PaymentService` go through this trait — never call `ferro_stripe::CheckoutBuilder` or `ferro_stripe::refund::create` directly from service code.

> **Pattern updated per Open Question 1 resolution — the fee is returned via
> `CheckoutResponse`, NOT carried on `CheckoutRequest`. Authoritative shape:
> `234-PATTERNS.md` service.rs section.** The production gateway computes the fee
> internally (`Stripe::config().application_fee_for`) so `PaymentService` never calls
> `Stripe::config()` (which panics in tests), and returns it for snapshotting.

```rust
// Source: derived from ferro-stripe/src/checkout.rs + refund.rs (verified)

/// Parameters for creating a Stripe Checkout session.
pub struct CheckoutRequest {
    pub amount_cents: i64,
    pub currency: String,
    pub line_description: String,
    pub success_url: String,
    pub cancel_url: String,
    pub idempotency_key: String,
    /// Some = Connect destination charge; None = direct charge.
    pub connect_account_id: Option<String>,
}

/// Gateway return: the Stripe-minted session plus the fee the gateway computed,
/// so `PaymentService` can snapshot `application_fee_cents` without touching
/// `Stripe::config()`.
pub struct CheckoutResponse {
    pub intent: ferro_stripe::CheckoutIntent,
    pub application_fee_cents: Option<i64>,
}

#[async_trait::async_trait]
pub trait StripeGateway: Send + Sync {
    async fn create_checkout_session(
        &self,
        req: CheckoutRequest,
    ) -> Result<CheckoutResponse, ferro_stripe::Error>;

    async fn create_refund(
        &self,
        charge_id: &str,
        amount_cents: Option<i64>,
        idempotency_key: &str,
    ) -> Result<(), ferro_stripe::Error>;
}
```

**Production impl shape:**

```rust
// Source: [VERIFIED: ferro-stripe/src/checkout.rs, refund.rs]
pub struct StripeClientGateway;

#[async_trait::async_trait]
impl StripeGateway for StripeClientGateway {
    async fn create_checkout_session(
        &self,
        req: CheckoutRequest,
    ) -> Result<CheckoutResponse, ferro_stripe::Error> {
        // Fee computed internally — PaymentService never calls Stripe::config().
        let application_fee_cents = req
            .connect_account_id
            .as_ref()
            .and_then(|_| ferro_stripe::Stripe::config().application_fee_for(req.amount_cents));
        let mut builder = ferro_stripe::CheckoutBuilder::new(ferro_stripe::Mode::Payment)
            .line_item(ferro_stripe::LineItem {
                name: req.line_description.clone(),
                description: None,
                unit_amount_cents: req.amount_cents,
                quantity: 1,
                currency: req.currency.clone(),
            })
            .success_url(&req.success_url)
            .cancel_url(&req.cancel_url)
            .idempotency_key(&req.idempotency_key);
        if let Some(account_id) = &req.connect_account_id {
            builder = builder.destination(account_id, application_fee_cents);
        }
        let intent = builder.create().await?;
        Ok(CheckoutResponse { intent, application_fee_cents })
    }

    async fn create_refund(
        &self,
        charge_id: &str,
        amount_cents: Option<i64>,
        idempotency_key: &str,
    ) -> Result<(), ferro_stripe::Error> {
        ferro_stripe::refund::create(charge_id, amount_cents, idempotency_key, None).await?;
        Ok(())
    }
}
```

Note: `ferro_stripe::refund::create` returns `Result<stripe::Refund, ferro_stripe::Error>`. The production impl discards the `Refund` value — `PaymentService` does not need it.

### Pattern 2: `attach_session` lifecycle function

**What:** A new lifecycle function (addition to lifecycle.rs) that sets `stripe_session_id` and `application_fee_cents` on a reserved row after a successful Stripe checkout session creation.

**When to use:** Called immediately after `stripe.create_checkout_session(...)` succeeds in `start_checkout`. Uses `GuardedUpdate` to atomically set both columns, guarded by `Column::Id.eq(id)` (no status guard needed — the row was just created in the same call and cannot have changed status in a single thread).

```rust
// Source: pattern derived from lifecycle.rs mark_paid (verified)
pub async fn attach_session<C: ConnectionTrait>(
    id: i64,
    stripe_session_id: &str,
    application_fee_cents: Option<i64>,
    conn: &C,
) -> Result<bool, PaymentError> {
    // Guard: stripe_session_id IS NULL (idempotent for retries)
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::StripeSessionId.is_null())
        .set_value(
            Column::StripeSessionId,
            Value::String(Some(Box::new(stripe_session_id.to_string()))),
        )
        .set_value(
            Column::ApplicationFeeCents,
            match application_fee_cents {
                Some(f) => Value::BigInt(Some(f)),
                None => Value::BigInt(None),
            },
        )
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}
```

### Pattern 3: `request_refund` GuardedUpdate for dedup

**What:** Use `GuardedUpdate WHERE refund_amount_cents IS NULL` to atomically snapshot `refund_amount_cents` and prevent calling Stripe twice if concurrent calls race.

**Why it works:** If two concurrent callers both read the intent (status=paid, refund_amount_cents=NULL) and both attempt the GuardedUpdate, exactly one will set the value (the `WHERE IS NULL` guard excludes the second). The second caller's `exec_at_most_one` returns `Ok(false)` — a no-op, never reaches the Stripe API.

```rust
// Source: pattern derived from ferro-orm/src/guarded.rs exec_at_most_one (verified)
let snapshot_ok = GuardedUpdate::new(Entity)
    .filter(Column::Id.eq(intent_id))
    .filter(Column::RefundAmountCents.is_null())
    .set_value(
        Column::RefundAmountCents,
        Value::BigInt(Some(amount_cents)),
    )
    .exec_at_most_one(conn)
    .await
    .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))?;

if !snapshot_ok {
    // Already in flight — do not call Stripe twice.
    return Ok(());
}
// Proceed to stripe.create_refund(...)
```

### Pattern 4: `Billable` trait with `#[async_trait]` and object-safe design

**What:** `#[async_trait]` makes async methods object-safe by wrapping futures in `Box<dyn Future + Send>`. `Box<dyn Billable>` can then be returned from `BillableLoader::load`.

**Object-safety requirement:** `Billable: Send + Sync`. All methods take `&self` (not `self`). No associated types. The `connect_account_id` default method is a sync fn returning `Option<String>` — fully object-safe.

```rust
// Source: design spec + async-trait crate behavior [ASSUMED for exact macro expansion,
//          VERIFIED for pattern — used throughout ferro-stripe/src/idempotency.rs]

#[async_trait::async_trait]
pub trait Billable: Send + Sync {
    fn kind(&self) -> BillableKind;
    fn id(&self) -> i64;
    fn tenant_id(&self) -> i64;
    fn amount_cents(&self) -> i64;
    fn currency(&self) -> &str;
    fn checkout_line_description(&self) -> String;

    // Default: non-Connect billables return None; Connect billables override.
    fn connect_account_id(&self) -> Option<String> { None }

    async fn on_paid(
        &self,
        txn: &sea_orm::DatabaseTransaction,
    ) -> Result<(), crate::PaymentError>;

    async fn on_released(
        &self,
        txn: &sea_orm::DatabaseTransaction,
    ) -> Result<(), crate::PaymentError>;

    async fn on_refunded(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        amount_cents: i64,
    ) -> Result<(), crate::PaymentError>;
}
```

### Anti-Patterns to Avoid

- **Calling `ferro_stripe::CheckoutBuilder` directly inside `PaymentService`:** Bypasses the `StripeGateway` seam and makes the service untestable without `Stripe::init()`. Always go through `self.stripe.create_checkout_session(...)`.
- **Reading the intent row before `GuardedUpdate` for refund dedup:** A read-then-write introduces a race window. The `WHERE refund_amount_cents IS NULL` guard in `GuardedUpdate` is the correct single-statement dedup path.
- **Adding `payment_intent_id` to `start_checkout` return or snapshot:** `CheckoutIntent` does not carry `payment_intent_id` — confirmed field list is `{ session_id, url, expires_at, idempotency_key }`. The PI id arrives on the webhook (phase 235).
- **Storing `processed_log` in `PaymentService` in phase 234:** Causes an unused-field clippy error and pulls in lifecycle that is phase 235 only. Keep it out of `new()` until phase 235.
- **Using `#[allow(dead_code)]` on the entire struct:** Apply only to the `loader` field — the rest of the fields are used.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic conditional UPDATE for refund dedup | `SELECT … UPDATE …` with OCC retry | `GuardedUpdate::exec_at_most_one` | Read-then-write has a race window; GuardedUpdate is a single SQL statement [VERIFIED: ferro-orm/src/guarded.rs] |
| Stripe checkout session | Custom HTTP client | `StripeClientGateway` wrapping `CheckoutBuilder` | `CheckoutBuilder` handles idempotency guard, currency parsing, Connect wiring |
| Stripe API error type | Custom error enum | `ferro_stripe::Error` via `#[from]` | `ferro_stripe::Error` is a public `thiserror` enum; `#[from]` works correctly [VERIFIED: ferro-stripe/src/error.rs] |
| Async trait object safety | Manual boxing | `async-trait = "0.1"` | Already a dependency; produces correct `Box<dyn Future + Send>` boxing |
| In-memory SQLite for tests | Spinning up Postgres | `Database::connect("sqlite::memory:")` + `TestMigrator` | Exact pattern from lifecycle.rs [VERIFIED: ferro-payments/src/intent/lifecycle.rs] |

---

## Verification Results: Three Critical Discrepancy Points

### DISCREPANCY-1: Stripe is a global static, not an injectable client

**Claim (CONTEXT.md D-01):** `ferro_stripe::Stripe` is a global `OnceLock<stripe::Client>`. No injectable `Client` type.

**VERIFIED.** From `ferro-stripe/src/client.rs`:
```rust
static STRIPE_CLIENT: OnceLock<stripe::Client> = OnceLock::new();
static STRIPE_CONFIG: OnceLock<StripeConfig> = OnceLock::new();

pub struct Stripe;  // Zero-size type, static facade only.
```
`Stripe::client()` returns `&'static stripe::Client` — a reference to the global, not an owned injectable value. There is no `ferro_stripe::Client` type. The public exports in `ferro-stripe/src/lib.rs` confirm: `pub use client::Stripe` (the zero-size facade), no `Client` export.

**`CheckoutBuilder::create()` calls `crate::Stripe::client()` internally** — confirmed at line 193 of checkout.rs. This is why the seam must live in ferro-payments, not in ferro-stripe.

**`CheckoutIntent` field list VERIFIED:** `{ session_id: String, url: String, expires_at: DateTime<Utc>, idempotency_key: String }` — no `payment_intent_id` field. D-13 is correct.

**idempotency key NOT forwarded by async-stripe 0.41 VERIFIED.** From `ferro-stripe/src/checkout.rs` line 251:
> "Note: async-stripe 0.41 does not expose a per-request idempotency-key strategy on CheckoutSession::create."

From `ferro-stripe/src/refund.rs` line 27:
```rust
let _ = idempotency_key;  // parameter accepted but explicitly discarded
```

### DISCREPANCY-2: Refund-in-flight is a predicate, not an enum variant

**Claim (CONTEXT.md D-17):** `PaymentIntentStatus` has 5 variants, no `refund_requested`. "Refund-in-flight" is the predicate `status='paid' AND refund_amount_cents IS NOT NULL AND refunded_at IS NULL`.

**VERIFIED.** From `ferro-payments/src/intent/status.rs`:
```rust
pub enum PaymentIntentStatus {
    Reserved, Paid, Released, Failed, Refunded  // exactly 5 variants
}
```
No `RefundRequested`. No migration needed to implement D-16.

**`GuardedUpdate::exec_at_most_one` API VERIFIED** from `ferro-orm/src/guarded.rs`:
- `.filter()` — AND-combines conditions
- `.set_value(col, Value::...)` — sets a column to a literal value
- `.exec_at_most_one(conn)` — returns `Ok(true)` on 1 row, `Ok(false)` on 0 rows
- Existing usage pattern in lifecycle.rs confirmed: `map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))`

**`Column::RefundAmountCents.is_null()` filter:** SeaORM `ColumnTrait` provides `.is_null()` on column expressions. Verified via usage in lifecycle.rs (`Column::Status.is_in([...])` and `.filter(...)` chains).

### DISCREPANCY-3: Publish wave ordering

**Claim (CONTEXT.md D-21):** `ferro-payments` currently in `WAVE1B_CRATES` alongside `ferro-stripe`. Adding the ferro-stripe dependency creates an intra-wave ordering violation.

**VERIFIED.** From `.github/workflows/publish.yml`:
```
WAVE1B_CRATES="ferro-projections ferro-text ferro-ai ferro-stripe ferro-whatsapp
               ferro-notifications ferro-reservation ferro-payments ferro-projection
               ferro-deployments"
```
Both `ferro-stripe` and `ferro-payments` are in the same loop with no ordering guarantee. After adding `ferro-stripe` as a dependency of `ferro-payments`, crates.io may not have indexed `ferro-stripe` before `ferro-payments` publishes.

**Exact YAML edit required:**
1. Remove `ferro-payments` from `WAVE1B_CRATES`.
2. Add a new step after "Wait for crates.io index update (Wave 1b)":
   ```yaml
   - name: Publish Wave 1c (depends on Wave 1b only)
     run: |
       echo "Publishing Wave 1c crates..."
       WAVE1C_CRATES="ferro-payments"
       for crate in $WAVE1C_CRATES; do
         # ... same pattern as 1a/1b ...
       done
   - name: Wait for crates.io index update (Wave 1c)
     run: |
       echo "Waiting for crates.io to index Wave 1c crates..."
       sleep 30
   ```

**ferro-stripe version pin confirmed:** `version = "0.9.0"` in `ferro-stripe/Cargo.toml`. Dependency pin in ferro-payments: `ferro-stripe = { path = "../ferro-stripe", version = "0.9" }` (D-19 is correct).

---

## Common Pitfalls

### Pitfall 1: `#[allow(dead_code)]` scope

**What goes wrong:** Applying `#[allow(dead_code)]` to the struct itself suppresses warnings on ALL fields. Under `-D warnings`, clippy will flag any legitimately dead field that should have been removed.

**How to avoid:** Apply the attribute to the specific field only:
```rust
pub struct PaymentService<L: BillableLoader> {
    db: DatabaseConnection,
    stripe: Arc<dyn StripeGateway>,
    #[allow(dead_code)] // wired by handle_* in phase 235
    loader: L,
    return_url_builder: Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>,
}
```

**Warning signs:** Clippy passes but other unused fields are silently suppressed.

### Pitfall 2: `Arc<dyn Fn(...) + Send + Sync>` closure object safety

**What goes wrong:** `Fn(&dyn Billable) -> ReturnUrls` is object-safe when wrapped in `Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>`. However, `&dyn Billable` as a parameter to a trait-object `Fn` requires the closure's captured lifetimes to be compatible.

**How to avoid:** Use `Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>` exactly (no `'static` bound on the fn pointer; the `Arc` provides shared ownership). If the closure needs captures, ensure they are `Send + Sync`.

**Warning signs:** `the trait Fn(&dyn Billable) -> ReturnUrls cannot be made into an object` compiler error.

### Pitfall 3: `GuardedUpdate` with `Value::BigInt(None)` for NULL

**What goes wrong:** Setting an optional `i64` column to NULL via `set_value` requires `Value::BigInt(None)`. Using `Value::String(None)` or the wrong variant causes a runtime type mismatch in SeaORM.

**How to avoid:** Check the column type from entity.rs first. `application_fee_cents: Option<i64>` → `Value::BigInt(Some(f))` / `Value::BigInt(None)`. `stripe_session_id: Option<String>` → `Value::String(Some(Box::new(s)))` / `Value::String(None)`.

**Warning signs:** SeaORM type coercion error or silent no-op insert.

### Pitfall 4: `ferro_stripe::Error` does not implement `std::error::Error` directly as `Loader` variant

**What goes wrong:** `PaymentError::Loader(Box<dyn std::error::Error + Send + Sync>)` cannot use `#[from] ferro_stripe::Error` — that would be `Stripe(#[from] ferro_stripe::Error)`. The `Loader` variant is for consumer-side errors from `BillableLoader`, not Stripe errors.

**How to avoid:** The two variants are distinct:
- `Stripe(#[from] ferro_stripe::Error)` — Stripe API failures
- `Loader(Box<dyn std::error::Error + Send + Sync>)` — no `#[from]`, consumer sets manually via `PaymentError::Loader(Box::new(err))`

**Warning signs:** `#[from]` compile error: "conflicting implementations of From".

### Pitfall 5: `start_checkout` failing to handle `attach_session` failure

**What goes wrong:** If `attach_session` fails after a successful Stripe API call, the intent row is in state `reserved` with no session attached, but a real Stripe session exists. The next call to `start_checkout` for the same billable will hit the partial unique index.

**How to avoid:** Per D-14, the phase-236 reaper handles this case by expiring reserved rows with `stripe_session_id IS NULL AND expires_at < now()`. `start_checkout` returns `PaymentError::Db(..)` on `attach_session` failure; do not attempt compensating delete.

---

## Code Examples

### Extending `PaymentError` with new variants

```rust
// Source: [VERIFIED: ferro-payments/src/error.rs current state + ferro-stripe/src/error.rs]
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("payment: not found")]
    NotFound,

    #[error("payment: status precondition not met: {0}")]
    StatusPrecondition(String),

    #[error("payment: db error: {0}")]
    Db(#[from] sea_orm::DbErr),

    // --- Phase 234 additions ---
    #[error("payment: stripe error: {0}")]
    Stripe(#[from] ferro_stripe::Error),

    #[error("payment: loader error: {0}")]
    Loader(Box<dyn std::error::Error + Send + Sync>),

    #[error("payment: auto-refund triggered: {reason:?}")]
    AutoRefundTriggered { reason: AutoRefundReason },
}

#[derive(Debug)]
pub enum AutoRefundReason {
    LoaderError,
    BillableVanished,
    SideStateConflict,
}
```

Note: `ferro_stripe::Error` is a `thiserror`-derived enum (`Config`, `Stripe(String)`, `NoConnectAccount`, `WebhookVerification`, `EventAlreadyProcessed`, `MissingIdempotencyKey`, `ManualCaptureRequiresPaymentMode`). It implements `std::error::Error` + `Display` — `#[from]` works correctly. [VERIFIED: ferro-stripe/src/error.rs]

### `PaymentService::new` constructor shape

```rust
// Source: derived from D-09/D-11 decisions
pub struct PaymentService<L: BillableLoader> {
    db: sea_orm::DatabaseConnection,
    stripe: std::sync::Arc<dyn StripeGateway>,
    #[allow(dead_code)] // wired by handle_* in phase 235
    loader: L,
    return_url_builder: std::sync::Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>,
}

impl<L: BillableLoader> PaymentService<L> {
    pub fn new(
        db: sea_orm::DatabaseConnection,
        stripe: std::sync::Arc<dyn StripeGateway>,
        loader: L,
        return_url_builder: impl Fn(&dyn Billable) -> ReturnUrls + Send + Sync + 'static,
    ) -> Self {
        Self {
            db,
            stripe,
            loader,
            return_url_builder: std::sync::Arc::new(return_url_builder),
        }
    }
}
```

### Test harness shape (mirrors lifecycle.rs `#[cfg(test)]`)

```rust
// Source: [VERIFIED: ferro-payments/src/intent/lifecycle.rs test harness pattern]
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;
    use crate::migration::m20260617_create_payment_intents::Migration as CreateTable;
    use std::sync::{Arc, Mutex};

    struct TestMigrator;
    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(CreateTable)]
        }
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        TestMigrator::up(&conn, None).await.unwrap();
        conn
    }

    // Mock StripeGateway: records calls, returns canned results
    #[derive(Default)]
    struct MockStripeGateway {
        checkout_calls: Mutex<Vec<CheckoutRequest>>,
        checkout_result: Mutex<Option<Result<CheckoutResponse, ferro_stripe::Error>>>,
        refund_calls: Mutex<Vec<(String, Option<i64>)>>,
        refund_result: Mutex<Option<Result<(), ferro_stripe::Error>>>,
    }

    #[async_trait::async_trait]
    impl StripeGateway for MockStripeGateway {
        async fn create_checkout_session(
            &self,
            req: CheckoutRequest,
        ) -> Result<CheckoutResponse, ferro_stripe::Error> {
            self.checkout_calls.lock().unwrap().push(req);
            self.checkout_result.lock().unwrap().take()
                // Test controls application_fee_cents here — fully offline (no Stripe::config()).
                .unwrap_or_else(|| Ok(CheckoutResponse {
                    intent: fake_checkout_intent(),
                    application_fee_cents: None,
                }))
        }
        async fn create_refund(
            &self,
            charge_id: &str,
            amount_cents: Option<i64>,
            _key: &str,
        ) -> Result<(), ferro_stripe::Error> {
            self.refund_calls.lock().unwrap().push((charge_id.to_string(), amount_cents));
            self.refund_result.lock().unwrap().take().unwrap_or(Ok(()))
        }
    }

    fn fake_checkout_intent() -> ferro_stripe::CheckoutIntent {
        ferro_stripe::CheckoutIntent {
            session_id: "cs_test_mock".to_string(),
            url: "https://checkout.stripe.com/mock".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            idempotency_key: "checkout-1".to_string(),
        }
    }

    // Mock BillableLoader
    struct MockLoader;
    #[async_trait::async_trait]
    impl BillableLoader for MockLoader {
        async fn load(&self, _kind: BillableKind, _id: i64)
            -> Result<Option<Box<dyn Billable>>, PaymentError> { Ok(None) }
    }
}
```

---

## `StripeConfig::application_fee_for` usage in `start_checkout`

**Verified signature** from `ferro-stripe/src/config.rs`:
```rust
pub fn application_fee_for(&self, amount_cents: i64) -> Option<i64>
```
Returns `Some(round(amount_cents * percent / 100))` when `application_fee_percent` is set and positive; `None` otherwise. Clamped to `[0, amount_cents]`.

**Call site in `start_checkout`:**
```rust
// Must call Stripe::config() — requires Stripe::init() to have been called.
// In production this is fine. In tests, use the StripeGateway mock and
// pre-compute fee in CheckoutRequest (or pass None for non-Connect tests).
let fee = if let Some(acct) = billable.connect_account_id() {
    let fee = ferro_stripe::Stripe::config().application_fee_for(billable.amount_cents());
    Some((acct, fee))
} else {
    None
};
```

**Test concern:** In unit tests, `Stripe::config()` panics if `Stripe::init()` was not called. D-20 says tests use the StripeGateway mock and do not call `Stripe::init()`. Therefore: `start_checkout` must NOT call `Stripe::config()` directly. The production `StripeClientGateway::create_checkout_session` calls it internally. The `PaymentService::start_checkout` passes fee computation to the gateway via `CheckoutRequest.application_fee_cents`.

**Solution for fee computation in `PaymentService`:** Either:
1. Pass fee as `None` if `connect_account_id()` is `None`; call `Stripe::config().application_fee_for(...)` only inside `StripeClientGateway` (production-only code path). The `CheckoutRequest` carries the pre-computed `application_fee_cents` passed in by the production gateway constructor.
2. Or: make `application_fee_cents` a field on `CheckoutRequest` computed by the **caller** of `start_checkout`, not by `PaymentService` itself.

**Recommended (cleanest for testability):** Put fee computation inside `StripeClientGateway::create_checkout_session`. `PaymentService` passes `connect_account_id` + `amount_cents` to the gateway via `CheckoutRequest`; the production gateway calls `Stripe::config().application_fee_for(...)` and uses the result. `PaymentService` **still snapshots `application_fee_cents`** — but it reads it from the `CheckoutRequest` that was passed to the gateway (or from the gateway's return, if CheckoutIntent carried it). Since `CheckoutIntent` does not carry `application_fee_cents`, the service needs to compute or carry the value itself.

**Simplest correct solution:** Add `application_fee_cents: Option<i64>` to `CheckoutRequest`. `PaymentService::start_checkout` sets it by calling a method on `StripeGateway` (or by asking the billable's `connect_account_id` + computing the fee). But to avoid calling `Stripe::config()` directly in `PaymentService`, pass the fee-computation responsibility entirely to the production gateway and have `start_checkout` snapshot whatever the gateway computed.

**Concrete resolution:** The `StripeGateway` trait returns `ferro_stripe::CheckoutIntent` which does not carry fee. Therefore, `PaymentService` cannot derive `application_fee_cents` from the result. Options:
- Compute fee in `PaymentService` via a separate injection (e.g. a `fee_calculator: Arc<dyn FeeCalculator>`) — adds complexity.
- Pass `connect_account_id` + `amount_cents` to the gateway; the gateway computes fee internally and returns it via an extended `CheckoutResponse { intent: CheckoutIntent, application_fee_cents: Option<i64> }` — simplest.
- Or: `CheckoutRequest` includes a pre-computed `application_fee_cents: Option<i64>` field; `PaymentService` computes it using a closure/function injected at construction (e.g. the same `return_url_builder` pattern).

**Planner note (discretion):** The cleanest solution per D-02 is to extend the `StripeGateway` return to carry `application_fee_cents`. Define:
```rust
pub struct CheckoutResponse {
    pub intent: ferro_stripe::CheckoutIntent,
    pub application_fee_cents: Option<i64>,
}
```
and have `StripeGateway::create_checkout_session` return `Result<CheckoutResponse, ferro_stripe::Error>`. The production impl calls `Stripe::config().application_fee_for(...)` internally; the mock returns whatever fee the test needs. `PaymentService` snapshots `resp.application_fee_cents` directly. This keeps all `Stripe::config()` calls inside the production gateway. [ASSUMED for exact return type — planner chooses between this and alternative approaches]

---

## Runtime State Inventory

This is a greenfield phase (new code added to an existing crate, no renames or data migrations). No runtime state inventory is required.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | tokio (via `#[tokio::test]`) + sea-orm in-memory SQLite |
| Config file | no separate config — `#[cfg(test)]` blocks in source files |
| Quick run command | `cargo test -p ferro-payments` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PAY-POLY-SVC-01 | `Billable` trait: sync accessors + async side effects compile and implement correctly | unit (compilation) | `cargo test -p ferro-payments` | ❌ Wave 0 |
| PAY-POLY-SVC-02 | `BillableLoader::load` returns `Box<dyn Billable>` and object-safe | unit | `cargo test -p ferro-payments` | ❌ Wave 0 |
| PAY-POLY-SVC-03a | `start_checkout`: inserts reserved row, attaches session_id, snapshots application_fee_cents (Connect case) | unit (in-memory SQLite + MockStripeGateway) | `cargo test -p ferro-payments -- start_checkout` | ❌ Wave 0 |
| PAY-POLY-SVC-03b | `start_checkout`: no fee snapshot on non-Connect billable | unit | `cargo test -p ferro-payments -- start_checkout_no_connect` | ❌ Wave 0 |
| PAY-POLY-SVC-03c | `request_refund`: status=paid + charge_id present → snapshots refund_amount_cents → calls Stripe | unit | `cargo test -p ferro-payments -- request_refund` | ❌ Wave 0 |
| PAY-POLY-SVC-03d | `request_refund`: status != paid → `StatusPrecondition` error, Stripe not called | unit | `cargo test -p ferro-payments -- request_refund_precondition` | ❌ Wave 0 |
| PAY-POLY-SVC-03e | `request_refund` dedup: second concurrent call no-ops, Stripe called exactly once | unit | `cargo test -p ferro-payments -- request_refund_dedup` | ❌ Wave 0 |
| PAY-POLY-SVC-04 | `StripeGateway` mock: records calls, tests can assert call counts | unit | `cargo test -p ferro-payments` | ❌ Wave 0 |
| PAY-POLY-SVC-05 | `PaymentError::Stripe(#[from])` + `Loader` + `AutoRefundTriggered` compile | unit (compilation) | `cargo test -p ferro-payments` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-payments`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`

### Wave 0 Gaps

- [ ] `ferro-payments/src/billable.rs` — `Billable` trait (PAY-POLY-SVC-01)
- [ ] `ferro-payments/src/loader.rs` — `BillableLoader` trait (PAY-POLY-SVC-02)
- [ ] `ferro-payments/src/service.rs` — `PaymentService` + `StripeGateway` + tests (PAY-POLY-SVC-03/04)
- [ ] `ferro-payments/src/intent/lifecycle.rs` — `attach_session` fn (PAY-POLY-SVC-03a)
- [ ] `ferro-payments/src/error.rs` — extended `PaymentError` + `AutoRefundReason` (PAY-POLY-SVC-05)
- [ ] `ferro-payments/Cargo.toml` — `ferro-stripe` dependency (D-19)
- [ ] `.github/workflows/publish.yml` — Wave 1c step (D-21)
- [ ] `ferro-payments/src/lib.rs` — new re-exports

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | partial | `tenant_id` propagated from `Billable::tenant_id()` — callers must validate ownership before calling `start_checkout` |
| V5 Input Validation | yes | `amount_cents > 0`, `currency` non-empty validated at billable impl; `intent_id` existence checked before `request_refund` |
| V6 Cryptography | no | Stripe handles payment cryptography; this crate does no crypto |

**Known threat patterns:**

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Double-refund on retry | Tampering | `GuardedUpdate WHERE refund_amount_cents IS NULL` (app-layer dedup) |
| Cross-tenant access to `request_refund` | Elevation of privilege | Callers must verify `intent.tenant_id` matches the request context; not enforced in this crate |
| Stripe session replay | Spoofing | `stripe_session_id UNIQUE` index + phase-235 webhook idempotency via `ProcessedEventLog` |

---

## Open Questions (RESOLVED)

1. **Fee computation in `PaymentService` without calling `Stripe::config()` directly**
   - What we know: `CheckoutIntent` does not carry `application_fee_cents`. `Stripe::config()` panics in tests. Fee must be snapshotted.
   - What's unclear: Whether `StripeGateway::create_checkout_session` should return `CheckoutResponse { intent, application_fee_cents }` or whether `PaymentService` should accept a fee-computation injection separately.
   - **RESOLVED:** `StripeGateway::create_checkout_session` returns
     `CheckoutResponse { intent, application_fee_cents }`; the production
     `StripeClientGateway` computes the fee internally via `Stripe::config().application_fee_for`
     and returns it, so `PaymentService` never calls `Stripe::config()`. `application_fee_cents`
     is NOT a field on `CheckoutRequest`. See Plan 03 Task 1 and `234-PATTERNS.md` service.rs
     (authoritative shape).

2. **`attach_session` as lifecycle fn vs. inline `ActiveModel` update**
   - What we know: No `attach_session` function exists in lifecycle.rs. The pattern for `GuardedUpdate` on a single row is established.
   - What's unclear: Whether to add `attach_session` to `lifecycle.rs` (consistent with the layer) or inline the update in `service.rs` (simpler, fewer files).
   - **RESOLVED:** Add `attach_session` to `lifecycle.rs` (Plan 02 Task 3), guarded by
     `Column::StripeSessionId.is_null()` — keeps the layer boundary clean and makes the
     operation independently testable.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `CheckoutResponse` wrapper (carrying `application_fee_cents`) is the cleanest solution for fee snapshot | Code Examples / Open Questions | Planner may choose a different approach (inline fee computation via Stripe::config inside StripeClientGateway + snapshot from CheckoutRequest). Any approach satisfying D-12 step 4 is valid. |
| A2 | `MockStripeGateway` lives in `#[cfg(test)]` rather than `test-helpers` feature | Architecture Patterns / Test pattern | If planner adds `test-helpers` feature to ferro-payments, the mock can be feature-gated instead. Functional outcome identical. |

---

## Sources

### Primary (HIGH confidence)

- `ferro-payments/Cargo.toml` — verified dependency versions, `async-trait = "0.1"` present
- `ferro-payments/src/intent/lifecycle.rs` — verified test harness pattern, `GuardedUpdate` usage, `map_err` idiom
- `ferro-payments/src/intent/status.rs` — verified 5-variant enum, no `refund_requested`
- `ferro-payments/src/intent/entity.rs` — verified column layout, `Option<i64>` for `refund_amount_cents`
- `ferro-payments/src/error.rs` — verified current 3-variant state
- `ferro-payments/src/lib.rs` — verified current public exports
- `ferro-stripe/src/client.rs` — verified `OnceLock` static facade, no injectable Client
- `ferro-stripe/src/checkout.rs` — verified `CheckoutIntent` field list, idempotency key not forwarded, `Stripe::client()` internal call
- `ferro-stripe/src/refund.rs` — verified `let _ = idempotency_key` explicit discard
- `ferro-stripe/src/config.rs` — verified `application_fee_for` signature + semantics
- `ferro-stripe/src/error.rs` — verified `Error` enum variants + `#[from] stripe::StripeError`
- `ferro-stripe/src/lib.rs` — verified public exports, no `Client` type exported
- `ferro-stripe/src/idempotency.rs` — verified `MemoryProcessedLog` + `ProcessedEventLog` trait
- `ferro-stripe/Cargo.toml` — verified `version = "0.9.0"`
- `ferro-orm/src/guarded.rs` — verified `exec_at_most_one` signature + behavior
- `ferro-orm/src/lib.rs` — verified `GuardedUpdate`, `Value`, `ColumnTrait` re-exports
- `.github/workflows/publish.yml` — verified `WAVE1B_CRATES` contains both ferro-stripe and ferro-payments

### Secondary (MEDIUM confidence)

- `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md` — authoritative spec for public API shapes and test table

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps verified against live Cargo.toml files
- Architecture: HIGH — all three discrepancy verifications confirmed from live source
- Pitfalls: HIGH — derived from verified GuardedUpdate behavior and ferro-stripe static facade
- Validation architecture: HIGH — test harness pattern confirmed from lifecycle.rs

**Research date:** 2026-06-17
**Valid until:** 2026-08-17 (stable — ferro-stripe and ferro-orm are internal crates, no external version drift)
