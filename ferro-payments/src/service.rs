//! Payment orchestrator.
//!
//! `PaymentService<L>` composes the lifecycle layer to mint Stripe Checkout
//! sessions (`start_checkout`) and initiate refunds (`request_refund`).
//! All Stripe calls route through the `StripeGateway` seam so the service is
//! fully unit-testable with a mock and no `Stripe::init` (D-02/D-03).

use std::sync::Arc;

use chrono::Utc;
use ferro_orm::{GuardedUpdate, Value};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, TransactionTrait};

use crate::billable::Billable;
use crate::error::PaymentError;
use crate::intent::entity::{Column, Entity};
use crate::intent::lifecycle;
use crate::intent::status::PaymentIntentStatus;
use crate::loader::BillableLoader;
use crate::BillableKind;

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
#[derive(Debug)]
pub struct CheckoutUrl(pub String);

// ---------------------------------------------------------------------------
// RefundStatus
// ---------------------------------------------------------------------------

/// Resolution of a refund-in-flight poll (D-08).
///
/// Returned by `StripeGateway::fetch_refund_status_for_payment_intent`.
/// The reconcile reaper maps this to the appropriate lifecycle action:
/// `Succeeded` → `mark_refunded`; `Pending` → leave for next tick;
/// `Failed` → `tracing::warn!`, no auto-retry (D-09).
#[derive(Debug, Clone, PartialEq)]
pub enum RefundStatus {
    Succeeded { amount_cents: i64 },
    Pending,
    Failed { reason: Option<String> },
}

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

    async fn create_refund_for_payment_intent(
        &self,
        payment_intent_id: &str,
        amount_cents: Option<i64>,
        idempotency_key: &str,
    ) -> Result<(), ferro_stripe::Error>;

    /// Read-only poll of the most recent refund for `payment_intent_id`.
    ///
    /// Used by the reconcile reaper (D-08) to determine whether a
    /// refund-in-flight intent has landed at Stripe. This is a query — it
    /// never issues or retries a refund (D-09).
    async fn fetch_refund_status_for_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> Result<RefundStatus, ferro_stripe::Error>;
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

    async fn create_refund_for_payment_intent(
        &self,
        payment_intent_id: &str,
        amount_cents: Option<i64>,
        idempotency_key: &str,
    ) -> Result<(), ferro_stripe::Error> {
        ferro_stripe::refund::create_for_payment_intent(
            payment_intent_id,
            amount_cents,
            idempotency_key,
            None,
        )
        .await?;
        Ok(())
    }

    async fn fetch_refund_status_for_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> Result<RefundStatus, ferro_stripe::Error> {
        let refunds =
            ferro_stripe::refund::list_for_payment_intent(payment_intent_id).await?;

        // Take the most recent refund. Stripe returns refunds newest-first with
        // limit=10, so `.first()` is the latest. An empty list means Stripe has
        // no record yet — return Pending so the reaper retries next tick
        // (T-236-03c: missing refund never maps to Succeeded).
        let Some(refund) = refunds.first() else {
            return Ok(RefundStatus::Pending);
        };

        Ok(match refund.status.as_deref() {
            Some("succeeded") => RefundStatus::Succeeded {
                amount_cents: refund.amount,
            },
            Some("pending") | Some("requires_action") => RefundStatus::Pending,
            Some("failed") | Some("canceled") => RefundStatus::Failed {
                reason: refund.failure_reason.clone(),
            },
            // Unknown or missing status — treat as Pending (safe default).
            _ => RefundStatus::Pending,
        })
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
    pub(crate) db: DatabaseConnection,
    pub(crate) stripe: Arc<dyn StripeGateway>,
    pub(crate) processed_log: Arc<dyn ferro_stripe::ProcessedEventLog>,
    pub(crate) loader: L,
    #[allow(clippy::type_complexity)]
    return_url_builder: Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>,
}

impl<L: BillableLoader> PaymentService<L> {
    pub fn new(
        db: DatabaseConnection,
        stripe: Arc<dyn StripeGateway>,
        loader: L,
        processed_log: Arc<dyn ferro_stripe::ProcessedEventLog>,
        return_url_builder: impl Fn(&dyn Billable) -> ReturnUrls + Send + Sync + 'static,
    ) -> Self {
        Self {
            db,
            stripe,
            processed_log,
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
        if billable.amount_cents() <= 0 {
            return Err(PaymentError::StatusPrecondition(
                "amount_cents must be positive to start checkout".to_string(),
            ));
        }
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
        let attached = lifecycle::attach_session(
            row.id,
            &resp.intent.session_id,
            resp.application_fee_cents,
            &self.db,
        )
        .await?;
        if !attached {
            tracing::warn!(
                row_id = row.id,
                "attach_session no-op: session already attached"
            );
        }

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
            .map_err(|e| {
                tracing::error!(
                    intent_id,
                    %charge_id,
                    err = %e,
                    "request_refund Stripe call failed; row is refund-in-flight \
                     (refund_amount_cents set, refunded_at NULL) — phase-236 reaper recovers"
                );
                PaymentError::Stripe(e)
            })
    }

    // -----------------------------------------------------------------------
    // release_expired (PAY-POLY-REAP-01)
    // -----------------------------------------------------------------------

    /// Inner implementation of the release reaper with an injected clock (D-04).
    ///
    /// Selects all `reserved` rows whose `expires_at < now`, then for each:
    /// - `mark_released` (GuardedUpdate `reserved → released`); `Ok(false)` = race
    ///   no-op (webhook already took it) → skip, no on_released.
    /// - Loader `Ok(None)` / `Err` = benign (no money was captured, status was
    ///   Reserved) → `tracing::warn!` + skip, NO auto-refund (D-06).
    /// - `on_released` error → logged, loop continues (D-05 failure isolation).
    ///
    /// Returns the count of intents actually released this tick.
    pub(crate) async fn release_expired_at(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize, PaymentError> {
        let expired = lifecycle::find_expired(now, &self.db).await?;
        let mut released = 0usize;

        for intent in expired {
            let result: Result<(), PaymentError> = async {
                let marked = lifecycle::mark_released(intent.id, &self.db).await?;
                if !marked {
                    // Racing webhook already released this row — skip, no on_released.
                    return Ok(());
                }

                let kind = BillableKind::from_string(intent.billable_kind.clone());
                match self.loader.load(kind, intent.billable_id).await {
                    Ok(Some(billable)) => {
                        let txn = self.db.begin().await.map_err(PaymentError::Db)?;
                        match billable.on_released(&txn).await {
                            Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
                            Err(e) => {
                                txn.rollback().await.ok();
                                return Err(e);
                            }
                        }
                        // Count only when on_released completed successfully.
                        return Ok(());
                    }
                    Ok(None) | Err(_) => {
                        // Loader vanished — benign (no money captured; status was
                        // Reserved). Log and skip; no auto-refund (D-06).
                        tracing::warn!(
                            intent_id = intent.id,
                            "release_expired: loader returned None/Err — \
                             skipping (no money captured)"
                        );
                    }
                }
                Ok(())
            }
            .await;

            match result {
                Ok(()) => released += 1,
                Err(e) => {
                    tracing::error!(
                        intent_id = intent.id,
                        err = %e,
                        "release_expired: per-intent error — continuing"
                    );
                    // D-05: do not propagate; let the loop continue.
                }
            }
        }

        Ok(released)
    }

    /// Release reaper entry (PAY-POLY-REAP-01). Releases reserved intents whose
    /// hold has expired; per-intent transaction, failure-isolated, returns the
    /// count released.
    pub async fn release_expired(&self) -> Result<usize, PaymentError> {
        self.release_expired_at(chrono::Utc::now()).await
    }

    // -----------------------------------------------------------------------
    // reconcile_refunds_in_flight (PAY-POLY-REAP-02)
    // -----------------------------------------------------------------------

    /// Inner implementation of the reconcile reaper with an injected clock (D-04).
    ///
    /// Selects `paid` rows with `refund_amount_cents IS NOT NULL`,
    /// `refunded_at IS NULL`, and `paid_at < older_than` (1 h behind `now`),
    /// then for each polls Stripe:
    /// - `Succeeded` → `mark_refunded` + `on_refunded`; `Ok(false)` = race no-op.
    /// - `Pending` → leave for next tick (no count increment).
    /// - `Failed` → `tracing::warn!` only; NO second Stripe call, NO
    ///   `mark_refunded` (double-refund guard, D-09).
    ///
    /// Returns the count of intents actually resolved this tick.
    pub(crate) async fn reconcile_refunds_in_flight_at(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize, PaymentError> {
        // Age anchor: only rows older than 1 h — prevents polling a refund that
        // Stripe has not yet indexed. The cron schedule is the consumer's knob.
        let older_than = now - chrono::Duration::hours(1);
        let in_flight = lifecycle::find_refunds_in_flight(older_than, &self.db).await?;
        let mut resolved = 0usize;

        for intent in in_flight {
            // Returns Ok(true) when the intent was resolved (Succeeded path),
            // Ok(false) when skipped (Pending, Failed, race no-op, loader-vanished),
            // Err when on_refunded itself fails (D-05: logged, loop continues).
            let result: Result<bool, PaymentError> = async {
                let Some(ref pi_id) = intent.payment_intent_id else {
                    tracing::warn!(
                        intent_id = intent.id,
                        "reconcile: payment_intent_id is NULL — cannot poll Stripe; skipping"
                    );
                    return Ok(false);
                };

                let status = self
                    .stripe
                    .fetch_refund_status_for_payment_intent(pi_id)
                    .await
                    .map_err(PaymentError::Stripe)?;

                match status {
                    RefundStatus::Succeeded { amount_cents } => {
                        let marked = lifecycle::mark_refunded(intent.id, &self.db).await?;
                        if !marked {
                            // Race no-op: webhook already refunded this row.
                            return Ok(false);
                        }

                        let kind = BillableKind::from_string(intent.billable_kind.clone());
                        match self.loader.load(kind, intent.billable_id).await {
                            Ok(Some(billable)) => {
                                let txn = self.db.begin().await.map_err(PaymentError::Db)?;
                                match billable.on_refunded(&txn, amount_cents).await {
                                    Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
                                    Err(e) => {
                                        txn.rollback().await.ok();
                                        return Err(e);
                                    }
                                }
                            }
                            Ok(None) | Err(_) => {
                                // Loader vanished on a money path — mirror webhook
                                // handle_charge_refunded `_` arm: rollback + skip.
                                tracing::warn!(
                                    intent_id = intent.id,
                                    "reconcile: loader returned None/Err on succeeded refund — skipping"
                                );
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    RefundStatus::Pending => {
                        // Leave for next tick; do not increment resolved count.
                        Ok(false)
                    }
                    RefundStatus::Failed { reason } => {
                        // D-09 double-refund guard: NEVER auto-retry a failed refund.
                        // Operator must investigate and manually resolve.
                        tracing::warn!(
                            intent_id = intent.id,
                            ?reason,
                            "reconcile: refund failed at Stripe — NOT auto-retrying \
                             (double-refund guard); operator action required"
                        );
                        Ok(false)
                    }
                }
            }
            .await;

            match result {
                Ok(true) => resolved += 1,
                Ok(false) => {} // skip — Pending, Failed, race no-op, or loader-vanished
                Err(e) => {
                    tracing::error!(
                        intent_id = intent.id,
                        err = %e,
                        "reconcile_refunds_in_flight: per-intent error — continuing"
                    );
                    // D-05: do not propagate; let the loop continue.
                }
            }
        }

        Ok(resolved)
    }

    /// Reconcile reaper entry (PAY-POLY-REAP-02). Polls Stripe for refund-in-flight
    /// intents and resolves succeeded refunds. Per-intent transaction; returns the
    /// count resolved.
    pub async fn reconcile_refunds_in_flight(&self) -> Result<usize, PaymentError> {
        self.reconcile_refunds_in_flight_at(chrono::Utc::now()).await
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
        pi_refund_calls: Mutex<Vec<(String, Option<i64>)>>,
        canned_pi_refund: Mutex<Option<Result<(), ferro_stripe::Error>>>,
        /// Records payment_intent_ids passed to fetch_refund_status_for_payment_intent.
        poll_calls: Mutex<Vec<String>>,
        /// Canned result for the next fetch_refund_status_for_payment_intent call.
        /// `None` → default Ok(RefundStatus::Succeeded { amount_cents: 1000 }).
        canned_refund_status: Mutex<Option<Result<RefundStatus, ferro_stripe::Error>>>,
    }

    impl MockStripeGateway {
        fn checkout_call_count(&self) -> usize {
            self.checkout_calls.lock().unwrap().len()
        }
        fn refund_call_count(&self) -> usize {
            self.refund_calls.lock().unwrap().len()
        }
        #[allow(dead_code)] // used by handle_* tests in plan 05
        fn pi_refund_calls(&self) -> Vec<(String, Option<i64>)> {
            self.pi_refund_calls.lock().unwrap().clone()
        }
        /// Program the next fetch_refund_status_for_payment_intent result.
        fn set_refund_status(&self, result: Result<RefundStatus, ferro_stripe::Error>) {
            *self.canned_refund_status.lock().unwrap() = Some(result);
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
            self.canned_refund.lock().unwrap().take().unwrap_or(Ok(()))
        }

        async fn create_refund_for_payment_intent(
            &self,
            payment_intent_id: &str,
            amount_cents: Option<i64>,
            _key: &str,
        ) -> Result<(), ferro_stripe::Error> {
            self.pi_refund_calls
                .lock()
                .unwrap()
                .push((payment_intent_id.to_string(), amount_cents));
            self.canned_pi_refund
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(()))
        }

        async fn fetch_refund_status_for_payment_intent(
            &self,
            payment_intent_id: &str,
        ) -> Result<RefundStatus, ferro_stripe::Error> {
            self.poll_calls
                .lock()
                .unwrap()
                .push(payment_intent_id.to_string());
            self.canned_refund_status
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(RefundStatus::Succeeded { amount_cents: 1000 }))
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
        async fn on_paid(&self, _txn: &sea_orm::DatabaseTransaction) -> Result<(), PaymentError> {
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
        async fn on_paid(&self, _txn: &sea_orm::DatabaseTransaction) -> Result<(), PaymentError> {
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
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
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
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
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
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        );

        svc.request_refund(intent_id, 5000)
            .await
            .expect("request_refund");

        assert_eq!(
            mock.refund_call_count(),
            1,
            "Stripe must be called exactly once"
        );

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
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
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
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
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
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
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
        assert_eq!(captured.connect_account_id.as_deref(), Some("acct_test"));
    }

    // -----------------------------------------------------------------------
    // PAY-POLY-REAP-02: RefundStatus + fetch_refund_status_for_payment_intent
    // -----------------------------------------------------------------------

    /// PAY-POLY-REAP-02a: MockStripeGateway records poll calls and returns
    /// the default canned status (Succeeded { amount_cents: 1000 }).
    #[tokio::test]
    async fn mock_poll_records_call_and_returns_default() {
        let mock = MockStripeGateway::default();
        let result = mock
            .fetch_refund_status_for_payment_intent("pi_test_123")
            .await
            .expect("mock poll");
        assert_eq!(
            result,
            RefundStatus::Succeeded { amount_cents: 1000 },
            "default canned status is Succeeded(1000)"
        );
        let calls = mock.poll_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], "pi_test_123");
    }

    /// PAY-POLY-REAP-02b: set_refund_status programs a canned Pending response.
    #[tokio::test]
    async fn mock_poll_returns_canned_pending() {
        let mock = MockStripeGateway::default();
        mock.set_refund_status(Ok(RefundStatus::Pending));
        let result = mock
            .fetch_refund_status_for_payment_intent("pi_test_456")
            .await
            .expect("mock poll");
        assert_eq!(result, RefundStatus::Pending);
    }

    /// PAY-POLY-REAP-02c: set_refund_status programs a canned Failed response.
    #[tokio::test]
    async fn mock_poll_returns_canned_failed() {
        let mock = MockStripeGateway::default();
        mock.set_refund_status(Ok(RefundStatus::Failed {
            reason: Some("lost".to_string()),
        }));
        let result = mock
            .fetch_refund_status_for_payment_intent("pi_test_789")
            .await
            .expect("mock poll");
        assert!(
            matches!(result, RefundStatus::Failed { reason: Some(_) }),
            "expected Failed with reason"
        );
    }

    /// WR-03 (D-12): start_checkout rejects amount_cents <= 0 before any DB write.
    #[tokio::test]
    async fn start_checkout_rejects_nonpositive_amount() {
        struct ZeroAmountBillable;

        #[async_trait::async_trait]
        impl Billable for ZeroAmountBillable {
            fn kind(&self) -> BillableKind {
                BillableKind::new("booking")
            }
            fn id(&self) -> i64 {
                99
            }
            fn tenant_id(&self) -> i64 {
                1
            }
            fn amount_cents(&self) -> i64 {
                0
            }
            fn currency(&self) -> &str {
                "EUR"
            }
            fn checkout_line_description(&self) -> String {
                "Zero amount".to_string()
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

        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());

        let svc = PaymentService::new(
            db.clone(),
            mock.clone(),
            MockLoader,
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        );

        let err = svc
            .start_checkout(&ZeroAmountBillable, chrono::Duration::hours(24))
            .await
            .expect_err("should reject zero amount");

        assert!(
            matches!(err, PaymentError::StatusPrecondition(_)),
            "expected StatusPrecondition, got: {err:?}"
        );

        // No reserved row inserted.
        use sea_orm::PaginatorTrait;
        let count = Entity::find().count(&db).await.expect("count query");
        assert_eq!(count, 0, "no DB row must be inserted for zero amount");

        // No Stripe call made.
        assert_eq!(
            mock.checkout_call_count(),
            0,
            "Stripe must NOT be called for zero amount"
        );
    }

    // -----------------------------------------------------------------------
    // Reaper test infrastructure
    // -----------------------------------------------------------------------

    /// A `BillableLoader` that returns a concrete `Billable` for any (kind, id).
    /// Used by reaper tests that need `on_released` / `on_refunded` to be called.
    struct ReturningLoader;

    struct SimpleBillable {
        id: i64,
    }

    #[async_trait::async_trait]
    impl Billable for SimpleBillable {
        fn kind(&self) -> BillableKind {
            BillableKind::new("booking")
        }
        fn id(&self) -> i64 {
            self.id
        }
        fn tenant_id(&self) -> i64 {
            1
        }
        fn amount_cents(&self) -> i64 {
            1000
        }
        fn currency(&self) -> &str {
            "EUR"
        }
        fn checkout_line_description(&self) -> String {
            "Test booking".to_string()
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

    #[async_trait::async_trait]
    impl BillableLoader for ReturningLoader {
        async fn load(
            &self,
            _kind: BillableKind,
            id: i64,
        ) -> Result<Option<Box<dyn Billable>>, PaymentError> {
            Ok(Some(Box::new(SimpleBillable { id })))
        }
    }

    /// Seed a `reserved` row with `expires_at` in the past (already expired).
    /// Returns the inserted row id.
    async fn seed_expired_reserved(
        conn: &sea_orm::DatabaseConnection,
        billable_id: i64,
    ) -> i64 {
        conn.execute_unprepared(&format!(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'booking',{billable_id},1000,'EUR','reserved',\
             '2020-01-01T00:00:00Z','2019-12-01T00:00:00Z')"
        ))
        .await
        .expect("seed expired reserved row");

        conn.query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT last_insert_rowid() AS id".to_string(),
        ))
        .await
        .expect("query last id")
        .expect("row")
        .try_get::<i64>("", "id")
        .expect("id")
    }

    /// Seed a `paid` row with `refund_amount_cents` set and `refunded_at` NULL,
    /// with `paid_at` well in the past (older than 1 h). Returns the row id.
    async fn seed_refund_in_flight(
        conn: &sea_orm::DatabaseConnection,
        billable_id: i64,
        pi_id: &str,
    ) -> i64 {
        conn.execute_unprepared(&format!(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              payment_intent_id,expires_at,reserved_at,paid_at,refund_amount_cents) \
             VALUES (1,'booking',{billable_id},1000,'EUR','paid',\
             '{pi_id}',\
             '2030-01-01T00:00:00Z','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z',500)"
        ))
        .await
        .expect("seed refund in flight row");

        conn.query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT last_insert_rowid() AS id".to_string(),
        ))
        .await
        .expect("query last id")
        .expect("row")
        .try_get::<i64>("", "id")
        .expect("id")
    }

    fn make_svc_with_loader<L: BillableLoader>(
        db: sea_orm::DatabaseConnection,
        mock: Arc<MockStripeGateway>,
        loader: L,
    ) -> PaymentService<L> {
        PaymentService::new(
            db,
            mock,
            loader,
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        )
    }

    // -----------------------------------------------------------------------
    // PAY-POLY-REAP-01: release_expired tests
    // -----------------------------------------------------------------------

    /// Expired reserved intent with a returning loader → on_released called,
    /// status flipped to released, count = 1.
    #[tokio::test]
    async fn release_expired() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        let row_id = seed_expired_reserved(&db, 1001).await;

        let svc = make_svc_with_loader(db.clone(), mock.clone(), ReturningLoader);

        // Use a `now` that is after the expires_at (2020-01-01).
        let now = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(chrono::Utc::now().offset());
        let count = svc
            .release_expired_at(now)
            .await
            .expect("release_expired_at");

        assert_eq!(count, 1, "must release exactly one expired row");

        // Verify the row transitioned to released.
        use crate::intent::status::PaymentIntentStatus;
        let row = Entity::find_by_id(row_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row still exists");
        assert_eq!(
            row.status,
            PaymentIntentStatus::Released,
            "status must be released after reaper"
        );
    }

    /// A row not yet expired (expires_at in the future) must be untouched.
    #[tokio::test]
    async fn release_expired_excludes_non_expired_row() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());

        // Seed a reserved row with expires_at in the future.
        db.execute_unprepared(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'booking',2001,1000,'EUR','reserved',\
             '2030-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
        )
        .await
        .expect("seed future row");

        let svc = make_svc_with_loader(db.clone(), mock.clone(), ReturningLoader);

        let now = chrono::DateTime::parse_from_rfc3339("2026-06-01T00:00:00Z")
            .unwrap()
            .with_timezone(chrono::Utc::now().offset());
        let count = svc
            .release_expired_at(now)
            .await
            .expect("release_expired_at");

        assert_eq!(count, 0, "non-expired row must not be counted");
    }

    /// mark_released returns Ok(false) (racing webhook already released) →
    /// no-op skip, no on_released call, count = 0.
    #[tokio::test]
    async fn reaper_skips_already_released() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        let row_id = seed_expired_reserved(&db, 3001).await;

        // Simulate a racing webhook: mark the row released before the reaper runs.
        db.execute_unprepared(&format!(
            "UPDATE payment_intents SET status='released', released_at='2026-01-01T00:00:00Z' \
             WHERE id={row_id}"
        ))
        .await
        .expect("pre-release row");

        let svc = make_svc_with_loader(db.clone(), mock.clone(), ReturningLoader);

        let now = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(chrono::Utc::now().offset());
        let count = svc
            .release_expired_at(now)
            .await
            .expect("release_expired_at");

        // Row was not in expired query (status != reserved) → count = 0.
        assert_eq!(count, 0, "already-released row must be skipped");
    }

    /// One intent's on_released failure → logged, loop continues, other rows
    /// still released (D-05).
    ///
    /// The failure path that exercises D-05 isolation is `on_released` returning Err
    /// (not loader returning Err — that is D-06 benign skip which still counts the row).
    #[tokio::test]
    async fn reaper_continues_on_error() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc as StdArc;

        // A billable that returns Err from on_released for one specific id.
        struct FailingOnReleasedBillable {
            id: i64,
            should_fail: bool,
            did_fail: StdArc<AtomicBool>,
        }

        #[async_trait::async_trait]
        impl Billable for FailingOnReleasedBillable {
            fn kind(&self) -> BillableKind {
                BillableKind::new("booking")
            }
            fn id(&self) -> i64 {
                self.id
            }
            fn tenant_id(&self) -> i64 {
                1
            }
            fn amount_cents(&self) -> i64 {
                1000
            }
            fn currency(&self) -> &str {
                "EUR"
            }
            fn checkout_line_description(&self) -> String {
                "test".to_string()
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
                if self.should_fail {
                    self.did_fail.store(true, Ordering::SeqCst);
                    return Err(PaymentError::StatusPrecondition(
                        "injected on_released error".to_string(),
                    ));
                }
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

        struct PartiallyFailingLoader {
            fail_id: i64,
            did_fail: StdArc<AtomicBool>,
        }

        #[async_trait::async_trait]
        impl BillableLoader for PartiallyFailingLoader {
            async fn load(
                &self,
                _kind: BillableKind,
                id: i64,
            ) -> Result<Option<Box<dyn Billable>>, PaymentError> {
                let should_fail = id == self.fail_id;
                Ok(Some(Box::new(FailingOnReleasedBillable {
                    id,
                    should_fail,
                    did_fail: self.did_fail.clone(),
                })))
            }
        }

        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        let did_fail = StdArc::new(AtomicBool::new(false));

        // Seed two expired rows — on_released fails for billable_id 4001, succeeds for 4002.
        let _fail_row = seed_expired_reserved(&db, 4001).await;
        let _ok_row = seed_expired_reserved(&db, 4002).await;

        let loader = PartiallyFailingLoader {
            fail_id: 4001,
            did_fail: did_fail.clone(),
        };
        let svc = make_svc_with_loader(db.clone(), mock.clone(), loader);

        let now = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(chrono::Utc::now().offset());
        // Must not return Err — failure isolation means the reaper continues (D-05).
        let count = svc
            .release_expired_at(now)
            .await
            .expect("release_expired_at must not propagate per-intent error");

        // The failing on_released error is logged and the loop continues.
        // The ok row (4002) is counted; the fail row (4001) is logged + not counted.
        assert!(did_fail.load(Ordering::SeqCst), "on_released error must have been triggered");
        assert_eq!(count, 1, "only the row with successful on_released should be counted");
    }

    // -----------------------------------------------------------------------
    // PAY-POLY-REAP-02: reconcile_refunds_in_flight tests
    // -----------------------------------------------------------------------

    /// Stripe poll returns Succeeded → mark_refunded + on_refunded called, count = 1.
    #[tokio::test]
    async fn reconcile_succeeded() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        // Default mock returns Succeeded { amount_cents: 1000 }.
        let row_id = seed_refund_in_flight(&db, 5001, "pi_reconcile_ok").await;

        let svc = make_svc_with_loader(db.clone(), mock.clone(), ReturningLoader);

        // now is 2 h after the seeded paid_at (2026-01-01), so older_than = now - 1h
        // is still > paid_at → row is selected.
        let now = chrono::DateTime::parse_from_rfc3339("2026-01-01T02:00:00Z")
            .unwrap()
            .with_timezone(chrono::Utc::now().offset());
        let count = svc
            .reconcile_refunds_in_flight_at(now)
            .await
            .expect("reconcile_refunds_in_flight_at");

        assert_eq!(count, 1, "succeeded refund must be counted as resolved");

        // Verify the poll was made — drop the guard before the await below.
        {
            let polls = mock.poll_calls.lock().unwrap();
            assert_eq!(polls.len(), 1);
            assert_eq!(polls[0], "pi_reconcile_ok");
        }

        // Verify the row transitioned to refunded.
        use crate::intent::status::PaymentIntentStatus;
        let row = Entity::find_by_id(row_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row still exists");
        assert_eq!(row.status, PaymentIntentStatus::Refunded);
    }

    /// Stripe poll returns Pending → row left untouched, count = 0.
    #[tokio::test]
    async fn reconcile_pending_noop() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        mock.set_refund_status(Ok(RefundStatus::Pending));
        let row_id = seed_refund_in_flight(&db, 6001, "pi_reconcile_pending").await;

        let svc = make_svc_with_loader(db.clone(), mock.clone(), ReturningLoader);

        let now = chrono::DateTime::parse_from_rfc3339("2026-01-01T02:00:00Z")
            .unwrap()
            .with_timezone(chrono::Utc::now().offset());
        let count = svc
            .reconcile_refunds_in_flight_at(now)
            .await
            .expect("reconcile_refunds_in_flight_at");

        assert_eq!(count, 0, "pending refund must NOT be counted");

        // Row must still be in paid status.
        use crate::intent::status::PaymentIntentStatus;
        let row = Entity::find_by_id(row_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row still exists");
        assert_eq!(row.status, PaymentIntentStatus::Paid);
        assert!(row.refunded_at.is_none(), "refunded_at must still be NULL");
    }

    /// Stripe poll returns Failed → warn only, NO second Stripe call, NO mark_refunded.
    /// This test asserts the mock recorded zero refund-creation calls (D-09).
    #[tokio::test]
    async fn reconcile_failed_no_retry() {
        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        mock.set_refund_status(Ok(RefundStatus::Failed {
            reason: Some("do_not_honor".to_string()),
        }));
        let row_id = seed_refund_in_flight(&db, 7001, "pi_reconcile_failed").await;

        let svc = make_svc_with_loader(db.clone(), mock.clone(), ReturningLoader);

        let now = chrono::DateTime::parse_from_rfc3339("2026-01-01T02:00:00Z")
            .unwrap()
            .with_timezone(chrono::Utc::now().offset());
        let count = svc
            .reconcile_refunds_in_flight_at(now)
            .await
            .expect("reconcile_refunds_in_flight_at");

        // Failed → warn, no resolution.
        assert_eq!(count, 0, "failed refund must NOT be counted as resolved");

        // Critical: NO refund-creation call must have been made (D-09 double-refund guard).
        // Drop guards before the await below.
        {
            let pi_refund_calls = mock.pi_refund_calls.lock().unwrap();
            assert_eq!(
                pi_refund_calls.len(),
                0,
                "reconcile must NOT call create_refund_for_payment_intent on Failed status (D-09)"
            );
        }
        {
            let refund_calls = mock.refund_calls.lock().unwrap();
            assert_eq!(
                refund_calls.len(),
                0,
                "reconcile must NOT call create_refund on Failed status (D-09)"
            );
        }

        // Row must be untouched (still paid, refunded_at still NULL).
        use crate::intent::status::PaymentIntentStatus;
        let row = Entity::find_by_id(row_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row still exists");
        assert_eq!(row.status, PaymentIntentStatus::Paid);
        assert!(row.refunded_at.is_none(), "refunded_at must still be NULL");
    }
}
