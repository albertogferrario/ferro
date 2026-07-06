//! End-to-end integration test for `ferro-payments`.
//!
//! This test drives the full consumer path (define a `Billable` → `start_checkout`
//! → `release_expired`) against the public crate API only (`ferro_payments::*`
//! paths, no crate-internal access).
//!
//! The test is `#[ignore]`d so `cargo test --all-features` skips it by default —
//! CI stays green without any Stripe secret. When run with `-- --ignored` and no
//! `STRIPE_TEST_SECRET_KEY` set, the test early-returns cleanly (no panic).
//!
//! To run against real Stripe test mode:
//!
//! ```bash
//! STRIPE_TEST_SECRET_KEY=sk_test_... \
//!   cargo test -p ferro-payments --test integration -- --ignored --nocapture
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

use ferro_payments::{
    Billable, BillableKind, BillableLoader, CheckoutUrl, PaymentError, PaymentService, ReturnUrls,
    StripeClientGateway,
};

// ---------------------------------------------------------------------------
// Test migration harness (mirrors the in-memory harness from service.rs tests)
// ---------------------------------------------------------------------------

use ferro_payments::migration::CreatePaymentIntentsTable;

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreatePaymentIntentsTable)]
    }
}

async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory sqlite");
    TestMigrator::up(&conn, None).await.expect("migrate up");
    conn
}

// ---------------------------------------------------------------------------
// Example Billable — a minimal "reservation" domain entity
// ---------------------------------------------------------------------------

/// A simple reservation billable used as the example consumer in this test.
///
/// The kind, amount, currency, and description are stable (no randomness) so the
/// test is deterministic against Stripe test mode — Stripe does not auto-decline
/// small EUR amounts in test mode.
struct ReservationBillable {
    id: i64,
    tenant_id: i64,
}

#[async_trait]
impl Billable for ReservationBillable {
    fn kind(&self) -> BillableKind {
        BillableKind::new("reservation")
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn tenant_id(&self) -> i64 {
        self.tenant_id
    }

    /// 5 EUR in the smallest unit.
    fn amount_cents(&self) -> i64 {
        500
    }

    fn currency(&self) -> &str {
        "EUR"
    }

    fn checkout_line_description(&self) -> String {
        "Reservation deposit (integration test)".to_string()
    }

    // connect_account_id defaults to None (direct charge).

    /// No-op: the integration test asserts payment-intent row state, not consumer
    /// side effects.
    async fn on_paid(&self, _txn: &sea_orm::DatabaseTransaction) -> Result<(), PaymentError> {
        Ok(())
    }

    /// No-op: ditto.
    async fn on_released(&self, _txn: &sea_orm::DatabaseTransaction) -> Result<(), PaymentError> {
        Ok(())
    }

    /// No-op: ditto.
    async fn on_refunded(
        &self,
        _txn: &sea_orm::DatabaseTransaction,
        _amount_cents: i64,
    ) -> Result<(), PaymentError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Example BillableLoader
// ---------------------------------------------------------------------------

/// Loader that resolves the fixed `(kind="reservation", id=1)` pair back to
/// the example billable.
struct ReservationLoader;

#[async_trait]
impl BillableLoader for ReservationLoader {
    async fn load(
        &self,
        _kind: BillableKind,
        id: i64,
    ) -> Result<Option<Box<dyn Billable>>, PaymentError> {
        Ok(Some(Box::new(ReservationBillable { id, tenant_id: 1 })))
    }
}

// ---------------------------------------------------------------------------
// Gated integration test
// ---------------------------------------------------------------------------

/// End-to-end checkout → release_expired path against Stripe test mode.
///
/// Requires `STRIPE_TEST_SECRET_KEY` (a `sk_test_...` key). When the key is
/// absent the test early-returns cleanly — no panic, no failure.
#[tokio::test]
#[ignore = "requires STRIPE_TEST_SECRET_KEY (Stripe test mode); run with -- --ignored"]
async fn e2e_checkout_and_release() {
    // Key guard — early return if absent, NEVER panic.
    let key = match std::env::var("STRIPE_TEST_SECRET_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("STRIPE_TEST_SECRET_KEY not set — skipping integration test");
            return; // early return, not panic!
        }
    };

    // Initialise ferro-stripe in test mode with the caller-supplied key.
    // webhook_secret is required by StripeConfig but unused by start_checkout /
    // release_expired — use a placeholder so the struct is well-formed.
    ferro_stripe::Stripe::init(ferro_stripe::StripeConfig {
        api_key: key,
        webhook_secret: "whsec_integration_test_placeholder".to_string(),
        connect_webhook_secret: None,
        application_fee_percent: None,
    });

    // Build an in-memory SQLite DB with the payment_intents table.
    let db = fresh_db().await;

    // Wire the PaymentService against the production StripeClientGateway (test
    // mode key) and the example loader.
    let svc = PaymentService::new(
        db.clone(),
        Arc::new(StripeClientGateway),
        ReservationLoader,
        Arc::new(ferro_stripe::MemoryProcessedLog::new()),
        |_b| ReturnUrls {
            success_url: "https://example.com/success".to_string(),
            cancel_url: "https://example.com/cancel".to_string(),
        },
    );

    let billable = ReservationBillable {
        id: 1,
        tenant_id: 1,
    };

    // --- start_checkout -------------------------------------------------------
    //
    // Use a very short TTL so the row is immediately expired. This lets
    // release_expired() select and process it without sleeping.
    let ttl = chrono::Duration::seconds(-1); // already expired at creation time
    let url: CheckoutUrl = svc
        .start_checkout(&billable, ttl)
        .await
        .expect("start_checkout must succeed against Stripe test mode");

    // Verify we received a real Stripe Checkout URL.
    assert!(
        url.0.starts_with("https://checkout.stripe.com/"),
        "expected a Stripe Checkout URL, got: {}",
        url.0
    );

    // --- release_expired -------------------------------------------------------
    //
    // The row's expires_at is in the past (negative TTL), so release_expired()
    // must find it and flip it to `released`. The reaper uses `Utc::now()`
    // internally — no clock injection needed here since the row is already expired.
    let released = svc
        .release_expired()
        .await
        .expect("release_expired must succeed");

    assert_eq!(
        released, 1,
        "exactly one reserved row should have been released"
    );

    // --- Verify the row transitioned to `released` in the DB ------------------
    use ferro_payments::PaymentIntentEntity;
    use ferro_payments::PaymentIntentStatus;
    use sea_orm::EntityTrait;

    let row = PaymentIntentEntity::find()
        .one(&db)
        .await
        .expect("DB query must succeed")
        .expect("the payment_intent row must exist");

    assert_eq!(
        row.status,
        PaymentIntentStatus::Released,
        "status must be `released` after the reaper ran; got: {:?}",
        row.status
    );
    assert!(
        row.released_at.is_some(),
        "released_at must be set after release_expired"
    );
}
