# Payments

ferro-payments adds polymorphic payment intent tracking and self-healing recovery to Ferro applications. It is built on top of ferro-stripe and covers two concerns:

- **Payment intent lifecycle** — typed database rows that track each billable entity through `reserved → paid → refunded` (or `released` on expiry), portable across Postgres, SQLite, and MySQL.
- **Recovery reapers** — two background jobs that self-heal the two money-stuck edge cases (unpaid expiry, refund-in-flight) without operator intervention.

## Quick Start

Three registrations at application startup wire the full payment stack:

**1. Register the migration**

```rust,ignore
use ferro_payments::migration::CreatePaymentIntentsTable;

// In your consumer migrator:
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // ... your existing migrations ...
            Box::new(CreatePaymentIntentsTable),
        ]
    }
}
```

**2. Build the service and wire the webhook dispatcher**

```rust,ignore
use std::sync::Arc;
use ferro_payments::{PaymentService, wire_dispatcher};
use ferro_payments::service::StripeClientGateway;

// In bootstrap.rs or application startup:
let service = Arc::new(PaymentService::new(
    db.clone(),
    Arc::new(StripeClientGateway),
    MyBillableLoader,           // your BillableLoader impl
    Arc::new(processed_log),    // ferro_stripe::ProcessedEventLog impl
    |billable| ferro_payments::service::ReturnUrls {
        success_url: format!("{}/pay/success/{}", base_url, billable.id()),
        cancel_url:  format!("{}/pay/cancel/{}", base_url, billable.id()),
    },
));

// Register the three webhook handlers on your existing SyncDispatcher:
wire_dispatcher(&mut dispatcher, service.clone());
```

**3. Schedule the two reaper jobs via cron**

```rust,ignore
use ferro_payments::{ReleaseExpiredPaymentIntents, ReconcileRefundsInFlight};

// In your queue bootstrap — cadences are controlled by the cron expression:
queue.schedule_cron("0 * * * *",  ReleaseExpiredPaymentIntents::new(Arc::clone(&service)));
queue.schedule_cron("30 * * * *", ReconcileRefundsInFlight::new(Arc::clone(&service)));
```

## Starting a Checkout

Call `start_checkout` to reserve a payment intent and obtain a Stripe Checkout URL for the consumer:

```rust,ignore
use ferro_payments::service::CheckoutUrl;

let CheckoutUrl(url) = service
    .start_checkout(&my_billable, chrono::Duration::hours(24))
    .await?;

// Redirect the user to `url`.
```

`start_checkout` inserts a `reserved` row, creates a Stripe Checkout session, and snapshots the session ID and fee. The `payment_intent_id` and `charge_id` arrive via webhook once the consumer completes checkout — `wire_dispatcher` handles both.

## Recovery Model

Two independent edge cases can leave money in an unresolved state. Both self-heal via the scheduled reapers.

### Unpaid Expiry — `ReleaseExpiredPaymentIntents`

A consumer starts checkout but never completes it. The Checkout session expires; the reserved hold should be released so the billable entity becomes available again.

`ReleaseExpiredPaymentIntents` runs at the configured cron interval (default: hourly), selects all `reserved` rows whose `expires_at` is in the past, and calls `mark_released` on each one inside an individual transaction. A failure on one row is isolated: the reaper logs the error and continues to the next row.

```rust,ignore
// Registered in bootstrap (see Quick Start step 3):
queue.schedule_cron("0 * * * *", ReleaseExpiredPaymentIntents::new(Arc::clone(&service)));
```

### Refund In-Flight — `ReconcileRefundsInFlight`

A `charge.refunded` webhook arrives, auto-refund is initiated, and the row transitions to `refunding` (snapshot set). In rare cases the refund confirmation event is never received — the row stays stuck at `refunding`.

`ReconcileRefundsInFlight` runs at the configured cron interval (default: every 30 minutes past the hour), selects all `refunding` rows, and for each one polls Stripe via `fetch_refund_status_for_payment_intent` — a read-only query, never a new refund call:

| Stripe status | Action |
|---------------|--------|
| `succeeded`   | `mark_refunded` — resolves the row |
| `pending` / `requires_action` | Leave for next tick |
| `failed` / `canceled` | Log warning for operator action — no auto-retry |

**Double-refund guard:** `ReconcileRefundsInFlight` is an idempotent query. It only calls `mark_refunded` when Stripe reports a refund already succeeded. It never issues a new refund call. A failed refund is logged for operator action and is never auto-retried by the reaper.

```rust,ignore
// Registered in bootstrap (see Quick Start step 3):
queue.schedule_cron("30 * * * *", ReconcileRefundsInFlight::new(Arc::clone(&service)));
```

## Implementing `Billable` and `BillableLoader`

`PaymentService` is generic over a `BillableLoader` — the consumer's bridge between a `payment_intents` row and the domain entity being paid for.

```rust,ignore
use ferro_payments::{Billable, BillableLoader, BillableKind, PaymentError};

// The domain entity implements Billable:
impl Billable for MyOrder {
    fn id(&self) -> i64                         { self.id }
    fn kind(&self) -> BillableKind              { BillableKind::new("order") }
    fn tenant_id(&self) -> i64                  { self.tenant_id }
    fn amount_cents(&self) -> i64               { self.amount_cents }
    fn currency(&self) -> &str                  { "eur" }
    fn checkout_line_description(&self) -> String {
        format!("Order #{}", self.id)
    }
    fn connect_account_id(&self) -> Option<&str> { None }
}

// The loader fetches the entity from the database by (kind, id):
struct MyLoader { db: DatabaseConnection }

#[async_trait::async_trait]
impl BillableLoader for MyLoader {
    async fn load(
        &self,
        kind: BillableKind,
        id: i64,
    ) -> Result<Option<Box<dyn Billable>>, PaymentError> {
        if kind.as_str() == "order" {
            let order = MyOrder::find_by_id(id, &self.db).await?;
            Ok(order.map(|o| Box::new(o) as Box<dyn Billable>))
        } else {
            Ok(None)
        }
    }
}
```

`BillableKind` is an open-set discriminator — the crate never enumerates kinds. Consumers define their own string constants and the webhook handlers route to the correct loader by kind.

## Database Schema

The migration `CreatePaymentIntentsTable` creates a single table:

```sql
CREATE TABLE payment_intents (
    id               BIGSERIAL PRIMARY KEY,
    tenant_id        BIGINT NOT NULL,
    billable_kind    VARCHAR(64) NOT NULL,
    billable_id      BIGINT NOT NULL,
    status           VARCHAR(32) NOT NULL DEFAULT 'reserved',
    amount_cents     BIGINT NOT NULL,
    currency         VARCHAR(8) NOT NULL DEFAULT 'eur',
    stripe_session_id         VARCHAR(255),
    stripe_payment_intent_id  VARCHAR(255),
    stripe_charge_id          VARCHAR(255),
    application_fee_cents     BIGINT,
    refund_initiated_at       TIMESTAMPTZ,
    expires_at                TIMESTAMPTZ NOT NULL,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

A partial unique index prevents double-reservations for the same billable entity:

```sql
CREATE UNIQUE INDEX uidx_payment_intents_active
    ON payment_intents (billable_kind, billable_id)
    WHERE status IN ('reserved', 'paid');
```

## Environment Variables

ferro-payments delegates all Stripe calls to ferro-stripe. The required environment variables are the same:

| Variable | Required | Description |
|----------|----------|-------------|
| `STRIPE_SECRET_KEY` | Yes | Stripe secret API key (`sk_live_xxx` or `sk_test_xxx`) |
| `STRIPE_WEBHOOK_SECRET` | Yes | Platform webhook signing secret (`whsec_xxx`) |

See also [Stripe Integration](stripe.md) for the full webhook setup, Connect support, and subscription management.
