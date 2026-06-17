//! Payment orchestrator.
//!
//! `PaymentService<L>` composes the lifecycle layer to mint Stripe Checkout
//! sessions (`start_checkout`) and initiate refunds (`request_refund`).
//! All Stripe calls route through the `StripeGateway` seam so the service is
//! fully unit-testable with a mock and no `Stripe::init` (D-02/D-03).

use std::sync::Arc;

use chrono::Utc;
use ferro_orm::{GuardedUpdate, Value};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};

use crate::billable::Billable;
use crate::error::PaymentError;
use crate::intent::entity::{Column, Entity};
use crate::intent::lifecycle;
use crate::intent::status::PaymentIntentStatus;
use crate::loader::BillableLoader;

// ---------------------------------------------------------------------------
// URL types
// ---------------------------------------------------------------------------

/// Success/cancel URLs for a Stripe Checkout session, supplied by the
/// consumer's `return_url_builder` (keeps app identity out of the crate).
pub struct ReturnUrls {
    pub success_url: String,
    pub cancel_url: String,
}

/// The hosted Stripe Checkout URL returned by `start_checkout`.
pub struct CheckoutUrl(pub String);

// ---------------------------------------------------------------------------
// StripeGateway request / response
// ---------------------------------------------------------------------------

/// Parameters for creating a Stripe Checkout session.
pub struct CheckoutRequest {
    pub amount_cents: i64,
    pub currency: String,
    pub line_description: String,
    pub success_url: String,
    pub cancel_url: String,
    pub idempotency_key: String,
    /// `Some` = Connect destination charge; `None` = direct charge.
    pub connect_account_id: Option<String>,
}

/// The gateway return value: the Stripe-minted session plus the fee the
/// production gateway computed internally, so `PaymentService` can snapshot
/// `application_fee_cents` without ever calling `Stripe::config()` in tests.
pub struct CheckoutResponse {
    pub intent: ferro_stripe::CheckoutIntent,
    pub application_fee_cents: Option<i64>,
}

// ---------------------------------------------------------------------------
// StripeGateway trait (the testability seam — D-02)
// ---------------------------------------------------------------------------

/// Abstraction over the two Stripe operations `PaymentService` requires.
///
/// Production code injects `StripeClientGateway`; tests inject a
/// `MockStripeGateway` that records calls and returns canned results — no
/// network, no `Stripe::init()`.
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

// ---------------------------------------------------------------------------
// Production gateway (StripeClientGateway)
// ---------------------------------------------------------------------------

/// Production `StripeGateway` impl wrapping `ferro_stripe::CheckoutBuilder`
/// and `ferro_stripe::refund::create`.
///
/// Fee computation lives here — the only place in the crate that calls
/// `Stripe::config()` (which panics without `Stripe::init`; safe in
/// production, unsafe in tests).
pub struct StripeClientGateway;

#[async_trait::async_trait]
impl StripeGateway for StripeClientGateway {
    async fn create_checkout_session(
        &self,
        req: CheckoutRequest,
    ) -> Result<CheckoutResponse, ferro_stripe::Error> {
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
        Ok(CheckoutResponse {
            intent,
            application_fee_cents,
        })
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

// ---------------------------------------------------------------------------
// PaymentService
// ---------------------------------------------------------------------------

/// Polymorphic payment orchestrator.
///
/// `L` is the consumer's `BillableLoader` implementation. All Stripe calls
/// route through the injected `Arc<dyn StripeGateway>` seam.
pub struct PaymentService<L: BillableLoader> {
    db: DatabaseConnection,
    stripe: Arc<dyn StripeGateway>,
    #[allow(dead_code)] // wired by handle_* in phase 235
    loader: L,
    return_url_builder: Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>,
}

impl<L: BillableLoader> PaymentService<L> {
    pub fn new(
        db: DatabaseConnection,
        stripe: Arc<dyn StripeGateway>,
        loader: L,
        return_url_builder: impl Fn(&dyn Billable) -> ReturnUrls + Send + Sync + 'static,
    ) -> Self {
        Self {
            db,
            stripe,
            loader,
            return_url_builder: Arc::new(return_url_builder),
        }
    }

    // -----------------------------------------------------------------------
    // start_checkout
    // -----------------------------------------------------------------------

    /// Mint a Stripe Checkout session for `billable`.
    ///
    /// Flow (D-12):
    /// 1. INSERT a `reserved` `payment_intents` row with `expires_at = now + ttl`.
    /// 2. Build a `CheckoutRequest` from the billable + `return_url_builder`.
    ///    Idempotency key: `checkout-{intent_id}` (deterministic per row).
    /// 3. Call `self.stripe.create_checkout_session` — fee computation lives in
    ///    the production gateway, never here.
    /// 4. Attach `stripe_session_id` + snapshot `application_fee_cents` via
    ///    `lifecycle::attach_session`.
    /// 5. Return `CheckoutUrl`.
    ///
    /// On gateway failure the reserved row is left for the phase-236 reaper
    /// (no compensating delete — D-14).
    /// `payment_intent_id` / `charge_id` are NOT set here — they arrive on the
    /// webhook (D-13).
    pub async fn start_checkout(
        &self,
        billable: &dyn Billable,
        ttl: chrono::Duration,
    ) -> Result<CheckoutUrl, PaymentError> {
        let expires_at = Utc::now() + ttl;
        let row = lifecycle::create_reserved(
            billable.tenant_id(),
            billable.kind().as_str(),
            billable.id(),
            billable.amount_cents(),
            billable.currency(),
            expires_at,
            &self.db,
        )
        .await?;

        let urls = (self.return_url_builder)(billable);
        let req = CheckoutRequest {
            amount_cents: billable.amount_cents(),
            currency: billable.currency().to_string(),
            line_description: billable.checkout_line_description(),
            success_url: urls.success_url,
            cancel_url: urls.cancel_url,
            idempotency_key: format!("checkout-{}", row.id),
            connect_account_id: billable.connect_account_id(),
        };

        let resp = self
            .stripe
            .create_checkout_session(req)
            .await
            .map_err(PaymentError::Stripe)?;

        // On attach failure the reserved row persists — phase-236 reaper cleans it.
        lifecycle::attach_session(
            row.id,
            &resp.intent.session_id,
            resp.application_fee_cents,
            &self.db,
        )
        .await?;

        Ok(CheckoutUrl(resp.intent.url))
    }

    // -----------------------------------------------------------------------
    // request_refund
    // -----------------------------------------------------------------------

    /// Initiate a refund of `amount_cents` for the payment intent `intent_id`.
    ///
    /// Flow (D-15):
    /// 1. Load the intent row (`NotFound` if absent).
    /// 2. Require `status = paid` AND `charge_id IS NOT NULL`
    ///    (`StatusPrecondition` otherwise).
    /// 3. Atomically snapshot `refund_amount_cents` via
    ///    `GuardedUpdate WHERE refund_amount_cents IS NULL`.
    ///    `Ok(false)` = already in flight — no-op, never call Stripe twice.
    /// 4. Call `self.stripe.create_refund`.
    ///
    /// Does NOT flip `status` to `refunded` — that is the phase-235 webhook's
    /// job (`mark_refunded`, D-16).
    pub async fn request_refund(
        &self,
        intent_id: i64,
        amount_cents: i64,
    ) -> Result<(), PaymentError> {
        let row = Entity::find_by_id(intent_id)
            .one(&self.db)
            .await
            .map_err(PaymentError::Db)?
            .ok_or(PaymentError::NotFound)?;

        if row.status != PaymentIntentStatus::Paid {
            return Err(PaymentError::StatusPrecondition(
                "request_refund requires status = paid".to_string(),
            ));
        }

        let charge_id = row.charge_id.ok_or_else(|| {
            PaymentError::StatusPrecondition(
                "charge_id must be set to request a refund".to_string(),
            )
        })?;

        // Atomic dedup: exactly one concurrent caller wins the WHERE IS NULL guard.
        let snapshot_ok = GuardedUpdate::new(Entity)
            .filter(Column::Id.eq(intent_id))
            .filter(Column::RefundAmountCents.is_null())
            .set_value(Column::RefundAmountCents, Value::BigInt(Some(amount_cents)))
            .exec_at_most_one(&self.db)
            .await
            .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))?;

        if !snapshot_ok {
            // Already in flight — do not call Stripe twice (D-15).
            return Ok(());
        }

        let idempotency_key = format!("refund-{intent_id}");
        self.stripe
            .create_refund(&charge_id, Some(amount_cents), &idempotency_key)
            .await
            .map_err(PaymentError::Stripe)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database};
    use sea_orm_migration::MigratorTrait;
    use std::sync::Mutex;

    use crate::billable::Billable;
    use crate::error::PaymentError;
    use crate::intent::entity::Entity;
    use crate::migration::m20260617_create_payment_intents::Migration as CreateTable;
    use crate::{BillableKind, BillableLoader};

    // -----------------------------------------------------------------------
    // Test infrastructure
    // -----------------------------------------------------------------------

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(CreateTable)]
        }
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");
        TestMigrator::up(&conn, None).await.expect("migrate up");
        conn
    }

    // -----------------------------------------------------------------------
    // MockStripeGateway
    // -----------------------------------------------------------------------

    #[derive(Default)]
    struct MockStripeGateway {
        checkout_calls: Mutex<Vec<CheckoutRequest>>,
        canned_checkout: Mutex<Option<Result<CheckoutResponse, ferro_stripe::Error>>>,
        refund_calls: Mutex<Vec<(String, Option<i64>)>>,
        canned_refund: Mutex<Option<Result<(), ferro_stripe::Error>>>,
    }

    impl MockStripeGateway {
        fn checkout_call_count(&self) -> usize {
            self.checkout_calls.lock().unwrap().len()
        }
        fn refund_call_count(&self) -> usize {
            self.refund_calls.lock().unwrap().len()
        }
    }

    #[async_trait::async_trait]
    impl StripeGateway for MockStripeGateway {
        async fn create_checkout_session(
            &self,
            req: CheckoutRequest,
        ) -> Result<CheckoutResponse, ferro_stripe::Error> {
            self.checkout_calls.lock().unwrap().push(req);
            self.canned_checkout
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Ok(fake_checkout_response(None)))
        }

        async fn create_refund(
            &self,
            charge_id: &str,
            amount_cents: Option<i64>,
            _key: &str,
        ) -> Result<(), ferro_stripe::Error> {
            self.refund_calls
                .lock()
                .unwrap()
                .push((charge_id.to_string(), amount_cents));
            self.canned_refund
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(()))
        }
    }

    fn fake_checkout_response(fee: Option<i64>) -> CheckoutResponse {
        CheckoutResponse {
            intent: ferro_stripe::CheckoutIntent {
                session_id: "cs_test_mock".to_string(),
                url: "https://checkout.stripe.com/mock".to_string(),
                expires_at: Utc::now() + chrono::Duration::hours(1),
                idempotency_key: "checkout-1".to_string(),
            },
            application_fee_cents: fee,
        }
    }

    // -----------------------------------------------------------------------
    // MockLoader
    // -----------------------------------------------------------------------

    struct MockLoader;

    #[async_trait::async_trait]
    impl BillableLoader for MockLoader {
        async fn load(
            &self,
            _kind: BillableKind,
            _id: i64,
        ) -> Result<Option<Box<dyn Billable>>, PaymentError> {
            Ok(None)
        }
    }

    // -----------------------------------------------------------------------
    // Test billable structs
    // -----------------------------------------------------------------------

    struct ConnectBillable;

    #[async_trait::async_trait]
    impl Billable for ConnectBillable {
        fn kind(&self) -> BillableKind {
            BillableKind::new("booking")
        }
        fn id(&self) -> i64 {
            1
        }
        fn tenant_id(&self) -> i64 {
            1
        }
        fn amount_cents(&self) -> i64 {
            5000
        }
        fn currency(&self) -> &str {
            "EUR"
        }
        fn checkout_line_description(&self) -> String {
            "Test booking".to_string()
        }
        fn connect_account_id(&self) -> Option<String> {
            Some("acct_test".to_string())
        }
        async fn on_paid(
            &self,
            _txn: &sea_orm::DatabaseTransaction,
        ) -> Result<(), PaymentError> {
            Ok(())
        }
        async fn on_released(
            &self,
            _txn: &sea_orm::DatabaseTransaction,
        ) -> Result<(), PaymentError> {
            Ok(())
        }
        async fn on_refunded(
            &self,
            _txn: &sea_orm::DatabaseTransaction,
            _amount_cents: i64,
        ) -> Result<(), PaymentError> {
            Ok(())
        }
    }

    struct DirectBillable;

    #[async_trait::async_trait]
    impl Billable for DirectBillable {
        fn kind(&self) -> BillableKind {
            BillableKind::new("booking")
        }
        fn id(&self) -> i64 {
            2
        }
        fn tenant_id(&self) -> i64 {
            1
        }
        fn amount_cents(&self) -> i64 {
            3000
        }
        fn currency(&self) -> &str {
            "EUR"
        }
        fn checkout_line_description(&self) -> String {
            "Test direct booking".to_string()
        }
        // connect_account_id defaults to None
        async fn on_paid(
            &self,
            _txn: &sea_orm::DatabaseTransaction,
        ) -> Result<(), PaymentError> {
            Ok(())
        }
        async fn on_released(
            &self,
            _txn: &sea_orm::DatabaseTransaction,
        ) -> Result<(), PaymentError> {
            Ok(())
        }
        async fn on_refunded(
            &self,
            _txn: &sea_orm::DatabaseTransaction,
            _amount_cents: i64,
        ) -> Result<(), PaymentError> {
            Ok(())
        }
    }

    // -----------------------------------------------------------------------
    // Seed helpers
    // -----------------------------------------------------------------------

    /// Seed a `paid` row with an optional `charge_id` via raw SQL (bypasses
    /// lifecycle guards). `billable_id` is varied per call to avoid the partial
    /// unique index on `(billable_kind, billable_id) WHERE status IN (...)`.
    /// Returns the inserted row id.
    async fn seed_paid_with_charge(
        conn: &sea_orm::DatabaseConnection,
        billable_id: i64,
        charge: Option<&str>,
    ) -> i64 {
        let charge_sql = match charge {
            Some(c) => format!("'{c}'"),
            None => "NULL".to_string(),
        };
        conn.execute_unprepared(&format!(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              charge_id,expires_at,reserved_at) \
             VALUES (1,'booking',{billable_id},5000,'EUR','paid',\
             {charge_sql},\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')"
        ))
        .await
        .expect("seed paid row");

        let row = conn
            .query_one(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT last_insert_rowid() AS id".to_string(),
            ))
            .await
            .expect("query last id")
            .expect("row");
        row.try_get::<i64>("", "id").expect("id")
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// PAY-POLY-SVC-03a: Connect billable → reserved row exists, session_id
    /// attached, application_fee_cents snapshotted.
    #[tokio::test]
    async fn start_checkout() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        // Preset a fee response for the Connect case.
        *mock.canned_checkout.lock().unwrap() = Some(Ok(fake_checkout_response(Some(250))));

        let svc = PaymentService::new(
            db.clone(),
            mock.clone(),
            MockLoader,
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        );

        let url = svc
            .start_checkout(&ConnectBillable, chrono::Duration::hours(24))
            .await
            .expect("start_checkout");

        assert_eq!(url.0, "https://checkout.stripe.com/mock");
        assert_eq!(mock.checkout_call_count(), 1);

        // Verify the reserved row was created and the session was attached.
        let row = Entity::find()
            .one(&db)
            .await
            .unwrap()
            .expect("row must exist");
        assert_eq!(row.stripe_session_id, Some("cs_test_mock".to_string()));
        assert_eq!(row.application_fee_cents, Some(250));
    }

    /// PAY-POLY-SVC-03b: non-Connect billable → application_fee_cents stays NULL.
    #[tokio::test]
    async fn start_checkout_no_connect() {
        let db = fresh_db().await;
        // Default mock returns fee = None for non-Connect.
        let mock = Arc::new(MockStripeGateway::default());

        let svc = PaymentService::new(
            db.clone(),
            mock.clone(),
            MockLoader,
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        );

        svc.start_checkout(&DirectBillable, chrono::Duration::hours(24))
            .await
            .expect("start_checkout_no_connect");

        let row = Entity::find()
            .one(&db)
            .await
            .unwrap()
            .expect("row must exist");
        assert!(
            row.application_fee_cents.is_none(),
            "non-Connect should have no application_fee_cents"
        );
    }

    /// PAY-POLY-SVC-03c: paid row with charge_id → refund_amount_cents
    /// snapshotted and Stripe called exactly once.
    #[tokio::test]
    async fn request_refund() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        let intent_id = seed_paid_with_charge(&db, 101, Some("ch_test_abc")).await;

        let svc = PaymentService::new(
            db.clone(),
            mock.clone(),
            MockLoader,
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        );

        svc.request_refund(intent_id, 5000)
            .await
            .expect("request_refund");

        assert_eq!(mock.refund_call_count(), 1, "Stripe must be called exactly once");

        let row = Entity::find_by_id(intent_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row still exists");
        assert_eq!(row.refund_amount_cents, Some(5000));
    }

    /// PAY-POLY-SVC-03d: non-paid status OR missing charge_id → StatusPrecondition,
    /// Stripe NOT called.
    #[tokio::test]
    async fn request_refund_precondition() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());

        // Seed a reserved row (not paid) — use unique billable_id to avoid unique index.
        let reserved_id = seed_paid_with_charge(&db, 201, None).await;
        // Patch the status to reserved via raw SQL.
        db.execute_unprepared(&format!(
            "UPDATE payment_intents SET status='reserved' WHERE id={reserved_id}"
        ))
        .await
        .expect("patch to reserved");

        // Seed a paid row without charge_id — different billable_id.
        let no_charge_id = seed_paid_with_charge(&db, 202, None).await;

        let svc = PaymentService::new(
            db.clone(),
            mock.clone(),
            MockLoader,
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        );

        // Non-paid → StatusPrecondition
        let err = svc
            .request_refund(reserved_id, 100)
            .await
            .expect_err("should fail on non-paid");
        assert!(
            matches!(err, PaymentError::StatusPrecondition(_)),
            "expected StatusPrecondition, got: {err:?}"
        );

        // Paid but no charge_id → StatusPrecondition
        let err2 = svc
            .request_refund(no_charge_id, 100)
            .await
            .expect_err("should fail without charge_id");
        assert!(
            matches!(err2, PaymentError::StatusPrecondition(_)),
            "expected StatusPrecondition, got: {err2:?}"
        );

        assert_eq!(
            mock.refund_call_count(),
            0,
            "Stripe must NOT be called on precondition failure"
        );
    }

    /// PAY-POLY-SVC-03e: second call to request_refund no-ops; Stripe called
    /// exactly once across both calls.
    #[tokio::test]
    async fn request_refund_dedup() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        let intent_id = seed_paid_with_charge(&db, 301, Some("ch_dedup_test")).await;

        let svc = PaymentService::new(
            db.clone(),
            mock.clone(),
            MockLoader,
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        );

        // First call succeeds.
        svc.request_refund(intent_id, 5000)
            .await
            .expect("first request_refund");

        // Second call must no-op (refund_amount_cents already set → GuardedUpdate
        // returns Ok(false)).
        svc.request_refund(intent_id, 5000)
            .await
            .expect("second request_refund must not error");

        assert_eq!(
            mock.refund_call_count(),
            1,
            "Stripe must be called exactly once even on duplicate request"
        );
    }

    /// PAY-POLY-SVC-04: MockStripeGateway records CheckoutRequest fields so
    /// tests can assert captured values.
    #[tokio::test]
    async fn mock_gateway_records_calls() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());

        let svc = PaymentService::new(
            db.clone(),
            mock.clone(),
            MockLoader,
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        );

        svc.start_checkout(&ConnectBillable, chrono::Duration::hours(24))
            .await
            .expect("start_checkout");

        let calls = mock.checkout_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        let captured = &calls[0];
        assert_eq!(captured.amount_cents, 5000);
        assert_eq!(captured.currency, "EUR");
        assert!(
            captured.idempotency_key.starts_with("checkout-"),
            "idempotency key must be deterministic"
        );
        assert_eq!(
            captured.connect_account_id.as_deref(),
            Some("acct_test")
        );
    }
}
