# ferro-payments — Polymorphic PaymentIntent Crate Design

**Date:** 2026-06-17
**Crate:** new `ferro-payments` (workspace member)
**Companion spec:** `gestiscilo-it/app:docs/superpowers/specs/2026-06-17-tenant-booking-upfront-payment-design.md`
**Source:** the first consumer app needs to take Stripe payments for three different
domain entities (cart orders, paywalled file shares, time-slot bookings) using the
same Stripe pipeline. Today each consumer rolls its own integration on top of
`ferro-stripe`, and one consumer (gestiscilo) has already shipped a "shadow Order"
workaround that leaks domain semantics into the Stripe layer. A reusable polymorphic
payment layer in the framework removes the workaround and lets future ferro apps
take Stripe payments for any first-class entity without re-implementing the wiring.

## Goals

- A `PaymentIntent` record that owns Stripe Checkout session id, payment intent id,
  charge id, status lifecycle, timestamps, and refund state — for any billable
  entity, not just orders.
- A `Billable` trait so domain entities expose their amount, description, and
  per-status side effects to the payment layer without coupling the payment layer
  to any concrete table.
- A `BillableLoader` trait so the consumer registers per-kind loaders and the payment
  layer can dispatch polymorphically (webhook handler, reaper, refund call site).
- Reuse of the existing `ferro-stripe::SyncDispatcher`, `ProcessedEventLog`, and
  `CheckoutBuilder` — no duplication.
- Migration helpers that respect the cross-backend rules (Postgres + SQLite +
  MySQL).
- A reaper job for expired intents and a reconcile-refunds-in-flight reaper.

## Non-goals

- Provider abstraction beyond Stripe. The `Billable` trait stays generic, but the
  first integration is Stripe via `ferro-stripe`. Other providers can be added in
  a future phase; the design must not preclude them but won't ship them now.
- Customer-facing UI. Consumers render their own.
- Refund policy. Refund-window and percentage configuration is consumer-domain;
  this crate exposes the orchestration only.
- Currency conversion. The crate stores currency as a string column but accepts
  only matching amounts at the call site.

---

## Crate layout

New workspace member `ferro-payments` parallel to `ferro-stripe`.

```
ferro-payments/
├── Cargo.toml                      # depends on ferro, ferro-stripe, sea-orm
├── src/
│   ├── lib.rs                      # public re-exports
│   ├── billable.rs                 # Billable trait + BillableKind enum
│   ├── loader.rs                   # BillableLoader trait
│   ├── intent/
│   │   ├── mod.rs
│   │   ├── entity.rs               # sea-orm Entity for payment_intents
│   │   ├── status.rs               # PaymentIntentStatus enum
│   │   └── lifecycle.rs            # create_reserved / mark_paid / mark_released / mark_refunded
│   ├── service.rs                  # PaymentService — the orchestrator
│   ├── webhook.rs                  # SyncDispatcher integration (typed handlers wrappers)
│   ├── reaper.rs                   # ReleaseExpiredPaymentIntents + ReconcileRefundsInFlight
│   ├── refund.rs                   # request_refund helper
│   ├── error.rs                    # PaymentError enum
│   └── migration/
│       ├── mod.rs
│       └── m20260617_create_payment_intents.rs
└── tests/                          # unit + integration
```

---

## Public API

### `Billable` trait

```rust
pub trait Billable: Send + Sync {
    fn kind(&self) -> BillableKind;
    fn id(&self) -> i64;
    fn tenant_id(&self) -> i64;
    fn amount_cents(&self) -> i64;
    fn currency(&self) -> &str;                       // "EUR" today
    fn checkout_line_description(&self) -> String;

    async fn on_paid(&self, txn: &DatabaseTransaction)
        -> Result<(), PaymentError>;
    async fn on_released(&self, txn: &DatabaseTransaction)
        -> Result<(), PaymentError>;
    async fn on_refunded(&self, txn: &DatabaseTransaction, amount_cents: i64)
        -> Result<(), PaymentError>;
}
```

### `BillableKind`

A string-backed enum that consumers extend by registering their kinds via the
loader. The crate ships no built-in variants beyond `Custom(&'static str)`:

```rust
pub struct BillableKind(&'static str);

impl BillableKind {
    pub const fn new(s: &'static str) -> Self { Self(s) }
    pub fn as_str(&self) -> &'static str { self.0 }
}
```

Consumers declare their own constants:

```rust
const ORDER: BillableKind = BillableKind::new("order");
const BOOKING: BillableKind = BillableKind::new("booking");
const FILE_SHARE: BillableKind = BillableKind::new("file_share");
```

The crate stores the string in `payment_intents.billable_kind` and never enumerates
known kinds — this lets future apps add billables without forking ferro.

### `BillableLoader` trait

```rust
pub trait BillableLoader: Send + Sync {
    async fn load(
        &self,
        kind: BillableKind,
        id: i64,
    ) -> Result<Option<Box<dyn Billable>>, PaymentError>;
}
```

The consumer's loader matches on `kind` and loads the right entity. Loader failures
(DB error, unknown kind) are surfaced via `PaymentError` so webhook handlers can
trigger auto-refund fallback.

### `PaymentService`

```rust
pub struct PaymentService<L: BillableLoader> {
    db: DatabaseConnection,
    stripe: Arc<ferro_stripe::Client>,
    processed_log: Arc<dyn ProcessedEventLog>,
    loader: L,
    return_url_builder: Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>,
}

impl<L: BillableLoader> PaymentService<L> {
    pub fn new(/* ... */) -> Self;

    pub async fn start_checkout(
        &self,
        billable: &dyn Billable,
        ttl: Duration,
    ) -> Result<CheckoutUrl, PaymentError>;

    pub async fn handle_session_completed(&self, event: StripeEvent)
        -> Result<(), PaymentError>;
    pub async fn handle_session_expired(&self, event: StripeEvent)
        -> Result<(), PaymentError>;
    pub async fn handle_charge_refunded(&self, event: StripeEvent)
        -> Result<(), PaymentError>;

    pub async fn release_expired(&self) -> Result<usize, PaymentError>;
    pub async fn request_refund(
        &self,
        intent_id: i64,
        amount_cents: i64,
    ) -> Result<(), PaymentError>;
}
```

`start_checkout` builds a Stripe Checkout session via `ferro_stripe::CheckoutBuilder`,
records the session id on a fresh `payment_intents` row, and returns the URL.

`handle_*` methods are the typed-event entry points. The crate exposes thin
`SyncDispatcher` shim handlers that the consumer registers via:

```rust
let service = Arc::new(PaymentService::new(...));
ferro_payments::wire_dispatcher(&mut dispatcher, service.clone());
```

`wire_dispatcher` registers three handlers: `OnCheckoutCompleted`,
`OnCheckoutExpired`, `OnChargeRefunded`. Each looks up the `payment_intent` by
Stripe identifier, dispatches `on_paid` / `on_released` / `on_refunded` on the
loaded billable, and idempotency-guards via the existing `ProcessedEventLog`.

`release_expired` is the reaper entry: single SQL pass over
`payment_intents WHERE status = 'reserved' AND expires_at < now()`, dispatching
`on_released` for each. Designed to be called from a ferro-queue job (consumers
schedule the queue job; the crate provides the implementation).

`request_refund` calls the Stripe refund API and writes the refund_amount snapshot.
Webhook `charge.refunded` arrives later and flips status to `refunded` via the
typed handler.

### Migrations

The crate ships SeaORM migrations under `ferro_payments::migration::*`:

```rust
pub fn migration_create_payment_intents() -> Box<dyn MigrationTrait>;
```

Consumers register these alongside their own migrations in their migrator. Schema
follows the table layout in the gestiscilo spec.

---

## Data model

`payment_intents` columns (per the gestiscilo spec — repeated here for ferro
self-sufficiency):

| Column                  | Type           | Notes                                              |
|-------------------------|----------------|----------------------------------------------------|
| `id`                    | BIGINT PK      |                                                    |
| `tenant_id`             | BIGINT NOT NULL| Consumer-defined FK target; no FK in this crate.   |
| `billable_kind`         | TEXT NOT NULL  | Consumer-defined kind string                       |
| `billable_id`           | BIGINT NOT NULL| Consumer-defined FK target; no FK in this crate.   |
| `amount_cents`          | BIGINT NOT NULL|                                                    |
| `currency`              | TEXT NOT NULL  | Default `'EUR'`                                    |
| `status`                | TEXT NOT NULL  | `reserved` \| `paid` \| `released` \| `failed` \| `refunded` |
| `stripe_session_id`     | TEXT NULL UNIQUE | Set when Checkout session minted                 |
| `payment_intent_id`     | TEXT NULL      | Stripe's id                                        |
| `charge_id`             | TEXT NULL      | Set on success                                     |
| `application_fee_cents` | BIGINT NULL    | Connect destination charge fee snapshot            |
| `expires_at`            | TIMESTAMPTZ NOT NULL |                                              |
| `reserved_at`           | TIMESTAMPTZ NOT NULL |                                              |
| `paid_at` / `released_at` / `refunded_at` | TIMESTAMPTZ NULL |                                |
| `refund_amount_cents`   | BIGINT NULL    | Partial-refund support                             |
| `metadata`              | JSONB NULL     | Free-form, no PII                                  |

Partial unique index: `(billable_kind, billable_id) WHERE status IN ('reserved',
'paid')`. Indexed on `(tenant_id, status)`, `(stripe_session_id)`,
`(payment_intent_id)`.

`tenant_id` and `billable_id` columns carry **no FK constraint** at the
ferro-payments level — the consumer's tables are unknown to the crate. Consumers
who want referential integrity add FKs in their own migrations.

### Migration rules (per consumer convention)

- `manager.get_database_backend()`, never hardcoded.
- `TRUE` / `FALSE` for booleans.
- No `INSERT OR IGNORE`.
- Portable timestamp defaults.

---

## Integration with `ferro-stripe`

`ferro-payments` depends on `ferro-stripe` and reuses:

| Mechanism                         | Used as                                                              |
|-----------------------------------|----------------------------------------------------------------------|
| `CheckoutBuilder`                 | session creation inside `start_checkout`                             |
| `SyncDispatcher`                  | wired by `wire_dispatcher` helper                                    |
| `ProcessedEventLog`               | idempotency guard for the typed handlers                             |
| Typed events (`StripeCheckoutCompleted`, `StripeCheckoutExpired`, `StripeChargeRefunded`) | input to crate-side handlers                |
| Connect destination charge support| `application_fee_cents` snapshotted on intent at `start_checkout`    |
| Refund API                        | called from `request_refund`                                         |

The crate does **not** add new Stripe event types or modify the dispatcher contract.
If a future polymorphic need surfaces a new event type, that work goes to
`ferro-stripe` first.

---

## Error model

```rust
pub enum PaymentError {
    NotFound,                       // billable or payment_intent
    StatusPrecondition(String),     // mark_paid called on a released intent, etc.
    Stripe(ferro_stripe::Error),
    Db(sea_orm::DbErr),
    Loader(Box<dyn std::error::Error + Send + Sync>),
    AutoRefundTriggered { reason: AutoRefundReason },
}
```

Auto-refund is not an "error" semantically (the customer's card was charged), but
it's a non-happy outcome that the webhook handler must communicate to the calling
context. The handler returns Ok after triggering the refund; `AutoRefundTriggered`
is only returned from `start_checkout` and `request_refund` for cases the consumer
must surface.

---

## Webhook race semantics

Same as the gestiscilo spec (repeated here so the ferro-side spec is
self-sufficient):

| Race                                            | Handling                                                                                  |
|-------------------------------------------------|-------------------------------------------------------------------------------------------|
| Webhook + reaper close together                 | Partial unique + status precondition in `mark_paid` / `mark_released` — second writer no-ops |
| Customer pays after slot released               | `mark_paid` succeeds; loader returns billable; `on_paid` notices the side state already released → auto-refund |
| Webhook replay                                  | `ProcessedEventLog.is_processed(event_id)` — return Ok early                              |
| Loader returns Err                              | Log + auto-refund the intent (the money was charged; we cannot identify what for)         |
| Loader returns Ok(None)                         | Same — billable vanished (DELETE in flight); refund                                       |

---

## Testing

| Layer                                       | Tests                                                                              |
|---------------------------------------------|------------------------------------------------------------------------------------|
| `intent::lifecycle` unit                    | Status transitions; partial-unique enforcement against in-memory SQLite            |
| `PaymentService::handle_*` unit             | Mocked Stripe + mocked loader + mocked `ProcessedEventLog`                         |
| `PaymentService::start_checkout` unit       | Asserts `application_fee_cents` snapshot; asserts session id attached              |
| Webhook race                                | Reaper + handler interleaved against in-memory clock                               |
| Reaper                                      | Injected clock; assert `on_released` called per expired intent                     |
| Auto-refund fallback                        | Loader returns None → crate calls Stripe refund (mocked); status becomes `refunded` |
| Integration (workspace test bin)            | Real ferro-stripe test mode against a tiny example Billable                        |

---

## Versioning + publication

- New crate at version `0.1.0`.
- Ferro workspace version bump alongside (one-shot publish: ferro core +
  ferro-payments together so the consumer can pin both).
- Consumer (gestiscilo) bumps `ferro` in `Cargo.toml`, runs `cargo update`, removes
  any local `[patch.crates-io]` before commit per repo convention.

---

## Open questions

1. Should `BillableLoader::load` take `tenant_id` as a separate argument for the
   common case where loaders are tenant-scoped, or should the loader extract it
   from the loaded billable? Leaning loader-extracts: keeps the trait signature
   small, and tenant-scoping is the loader's concern.
2. `wire_dispatcher` vs direct handler registration: the helper hides the typed
   handler trio behind one call. Tradeoff is discoverability vs ceremony.
   Leaning helper.
3. `ReconcileRefundsInFlight` cadence: 1 hour matches the gestiscilo spec.
   Configurable via ferro-queue cron expression at consumer-registration time.
4. Should `Billable` implementations be required to be `Clone`? Today's design
   passes `&dyn Billable` everywhere — no clone needed. Leaning no clone.
