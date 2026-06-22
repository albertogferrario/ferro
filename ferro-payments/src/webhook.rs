//! Webhook dispatcher registration and typed handler implementations.
//!
//! [`wire_dispatcher`] registers three typed handlers on a [`SyncDispatcher`]
//! for the payment intent lifecycle. Each handler is idempotent (ProcessedEventLog
//! fast-path), transactional (sea-orm begin/commit/rollback), and auto-refunds on
//! un-honorable captures (D-03..D-09, D-11).
//!
//! # Auto-refund invariant
//!
//! When money is captured but the billable cannot be honored (loader error, billable
//! vanished, or side-state conflict), `trigger_auto_refund` issues an exactly-once
//! refund by payment_intent — snapshotted under `WHERE refund_amount_cents IS NULL`
//! so concurrent callers cannot double-refund. On Stripe failure the row is left as
//! "refund-in-flight" and the phase-236 reaper recovers it.

use std::sync::Arc;

use ferro_orm::{GuardedUpdate, Value};
use sea_orm::{ColumnTrait, TransactionTrait};

use ferro_stripe::{
    StripeChargeRefunded, StripeCheckoutCompleted, StripeCheckoutExpired, SyncDispatcher,
};

use crate::error::{AutoRefundReason, PaymentError};
use crate::intent::entity::{Column, Entity};
use crate::intent::lifecycle;
use crate::loader::BillableLoader;
use crate::service::PaymentService;
use crate::BillableKind;

// ---------------------------------------------------------------------------
// wire_dispatcher
// ---------------------------------------------------------------------------

/// Register three typed Stripe webhook handlers on `dispatcher` for the
/// payment intent lifecycle. Returns the updated dispatcher.
///
/// Call once at app startup:
/// ```rust,ignore
/// let dispatcher = ferro_payments::wire_dispatcher(SyncDispatcher::new(), service.clone());
/// ```
pub fn wire_dispatcher<L: BillableLoader + 'static>(
    dispatcher: SyncDispatcher,
    service: Arc<PaymentService<L>>,
) -> SyncDispatcher {
    // Pre-clone one Arc per handler slot; each invocation re-clones from these.
    let svc1 = Arc::clone(&service);
    let svc2 = Arc::clone(&service);
    let svc3 = Arc::clone(&service);
    dispatcher
        .on(move |event: StripeCheckoutCompleted| {
            let svc = Arc::clone(&svc1);
            async move {
                svc.handle_session_completed(event)
                    .await
                    .map_err(payment_to_stripe_error)
            }
        })
        .on(move |event: StripeCheckoutExpired| {
            let svc = Arc::clone(&svc2);
            async move {
                svc.handle_session_expired(event)
                    .await
                    .map_err(payment_to_stripe_error)
            }
        })
        .on(move |event: StripeChargeRefunded| {
            let svc = Arc::clone(&svc3);
            async move {
                svc.handle_charge_refunded(event)
                    .await
                    .map_err(payment_to_stripe_error)
            }
        })
}

// ---------------------------------------------------------------------------
// Error bridge (D-04)
// ---------------------------------------------------------------------------

/// Bridge a [`PaymentError`] to a [`ferro_stripe::Error`] for the dispatcher.
///
/// Terminal outcomes (NotFound, StatusPrecondition, AutoRefundTriggered) must
/// never reach this bridge — handlers absorb them and return `Ok(())`.
fn payment_to_stripe_error(e: PaymentError) -> ferro_stripe::Error {
    match e {
        PaymentError::Stripe(s) => s,
        other => ferro_stripe::Error::Stripe(format!("payment: {other}")),
    }
}

/// Returns `true` for transient infrastructure errors that warrant Stripe
/// retrying the webhook. Returns `false` for permanent / business-state errors
/// that should be absorbed (return `Ok(())`) to stop the retry loop.
fn is_transient(e: &PaymentError) -> bool {
    matches!(e, PaymentError::Db(_) | PaymentError::Stripe(_))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

impl<L: BillableLoader> PaymentService<L> {
    /// Handle `checkout.session.completed` events.
    ///
    /// Flow: idempotency fast-path → find row → mark_paid (GuardedUpdate) →
    /// attach payment_intent_id → open txn → load billable → on_paid.
    /// Any failure to honor the capture routes to `trigger_auto_refund`.
    pub(crate) async fn handle_session_completed(
        &self,
        event: StripeCheckoutCompleted,
    ) -> Result<(), PaymentError> {
        // D-05: idempotency fast-path
        if !self
            .processed_log
            .try_mark_processed(&event.event_id)
            .await
            .map_err(PaymentError::Stripe)?
        {
            return Ok(());
        }

        // D-06: lookup by session_id
        let Some(intent) = lifecycle::find_by_stripe_session(&event.session_id, &self.db).await?
        else {
            // No row for this session — nothing to do.
            return Ok(());
        };

        // WR-01: flip reserved→paid AND run on_paid in ONE transaction. The
        // status flip committing separately (before on_paid) is what made a
        // transient on_paid failure unrecoverable: the row was left `paid`, the
        // event marked processed, and Stripe's retry short-circuited on the
        // idempotency fast-path → the capture was never honored. With the flip
        // inside the txn, a transient failure rolls the row back to `reserved`
        // and we un-mark the event so the retry redoes mark_paid + on_paid.
        let txn = self.db.begin().await.map_err(PaymentError::Db)?;
        let marked = lifecycle::mark_paid(intent.id, &txn).await?;
        if !marked {
            // Row not reserved — the reaper released it first (the entry
            // idempotency check rules out a plain duplicate delivery). The money
            // was captured, so record the capture and auto-refund.
            txn.rollback().await.ok();
            return self
                .capture_then_auto_refund(&event, intent.id, AutoRefundReason::SideStateConflict)
                .await;
        }

        // Attach payment_intent_id within the same txn (guarded — idempotent).
        if let Some(ref pi_id) = event.payment_intent_id {
            lifecycle::attach_payment_intent(intent.id, pi_id, &txn).await?;
        }

        let kind = BillableKind::from_string(intent.billable_kind.clone());
        match self.loader.load(kind, intent.billable_id).await {
            Err(_) => {
                txn.rollback().await.ok();
                self.capture_then_auto_refund(&event, intent.id, AutoRefundReason::LoaderError)
                    .await
            }
            Ok(None) => {
                txn.rollback().await.ok();
                self.capture_then_auto_refund(&event, intent.id, AutoRefundReason::BillableVanished)
                    .await
            }
            Ok(Some(billable)) => match billable.on_paid(&txn).await {
                Ok(()) => {
                    txn.commit().await.map_err(PaymentError::Db)?;
                    Ok(())
                }
                Err(e) => {
                    txn.rollback().await.ok();
                    if is_transient(&e) {
                        // WR-01: the status flip was rolled back with the txn, so
                        // un-mark the event — Stripe's retry will reprocess from
                        // `reserved` and re-run mark_paid + on_paid cleanly.
                        self.processed_log.unmark(&event.event_id).await.ok();
                        return Err(e);
                    }
                    // Permanent business-state conflict — record capture + auto-refund.
                    self.capture_then_auto_refund(
                        &event,
                        intent.id,
                        AutoRefundReason::SideStateConflict,
                    )
                    .await
                }
            },
        }
    }

    /// Honor-failure branch of [`Self::handle_session_completed`]: the money was
    /// captured at Stripe but the billable cannot be honored. Commit the capture
    /// (mark paid + attach payment_intent_id, both guarded/idempotent) so the row
    /// is `paid` and the reconcile reaper can recover a failed refund, then issue
    /// the auto-refund. Runs outside the (rolled-back) on_paid transaction.
    async fn capture_then_auto_refund(
        &self,
        event: &StripeCheckoutCompleted,
        intent_id: i64,
        reason: AutoRefundReason,
    ) -> Result<(), PaymentError> {
        lifecycle::mark_paid(intent_id, &self.db).await?;
        if let Some(ref pi_id) = event.payment_intent_id {
            lifecycle::attach_payment_intent(intent_id, pi_id, &self.db).await?;
        }
        self.trigger_auto_refund(
            &event.payment_intent_id,
            event.amount_total_cents,
            intent_id,
            reason,
        )
        .await
    }

    /// Handle `checkout.session.expired` events.
    ///
    /// Flow: idempotency fast-path → find row → mark_released → open txn →
    /// load billable → on_released. Already-released rows are no-ops.
    pub(crate) async fn handle_session_expired(
        &self,
        event: StripeCheckoutExpired,
    ) -> Result<(), PaymentError> {
        if !self
            .processed_log
            .try_mark_processed(&event.event_id)
            .await
            .map_err(PaymentError::Stripe)?
        {
            return Ok(());
        }

        let Some(intent) = lifecycle::find_by_stripe_session(&event.session_id, &self.db).await?
        else {
            return Ok(());
        };

        // WR-01: flip reserved→released and run on_released atomically. A
        // transient failure rolls the row back to `reserved` (the release reaper
        // then retries it) and un-marks the event so a Stripe retry reprocesses.
        let txn = self.db.begin().await.map_err(PaymentError::Db)?;
        let marked = lifecycle::mark_released(intent.id, &txn).await?;
        if !marked {
            txn.rollback().await.ok();
            return Ok(()); // no-op: already non-reserved (reaper or paid)
        }

        let kind = BillableKind::from_string(intent.billable_kind.clone());
        match self.loader.load(kind, intent.billable_id).await {
            Ok(Some(billable)) => match billable.on_released(&txn).await {
                Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
                Err(e) => {
                    txn.rollback().await.ok();
                    if is_transient(&e) {
                        // Transient — un-mark + propagate so Stripe retries (the
                        // status flip was rolled back with the txn).
                        self.processed_log.unmark(&event.event_id).await.ok();
                        return Err(e);
                    }
                    // Terminal business-state error — absorb; no retry loop. The
                    // row reverts to `reserved`; the release reaper is the backstop.
                }
            },
            _ => {
                txn.rollback().await.ok();
            }
        }
        Ok(())
    }

    /// Handle `charge.refunded` events.
    ///
    /// Lookup: `payment_intent_id` primary, `charge_id` fallback (D-07).
    /// Flow: idempotency fast-path → find row → mark_refunded → open txn →
    /// load billable → on_refunded(amount_cents).
    pub(crate) async fn handle_charge_refunded(
        &self,
        event: StripeChargeRefunded,
    ) -> Result<(), PaymentError> {
        if !self
            .processed_log
            .try_mark_processed(&event.event_id)
            .await
            .map_err(PaymentError::Stripe)?
        {
            return Ok(());
        }

        // D-07: try payment_intent_id first, fallback to charge_id
        let intent_opt = match &event.payment_intent_id {
            Some(pi_id) => lifecycle::find_by_payment_intent(pi_id, &self.db).await?,
            None => None,
        };
        let intent = match intent_opt {
            Some(i) => i,
            None => match lifecycle::find_by_charge_id(&event.charge_id, &self.db).await? {
                Some(i) => i,
                None => return Ok(()), // no row for this refund
            },
        };

        // WR-03: when this system snapshotted the refund amount (system-initiated
        // refund under the IS-NULL guard), prefer it over the event's reported
        // amount, which could reflect an unrelated refund on the same charge.
        // Fall back to the event amount for refunds with no snapshot (e.g. a
        // dashboard-initiated refund).
        let refund_amount = intent
            .refund_amount_cents
            .unwrap_or(event.amount_refunded_cents);

        // WR-01: flip paid→refunded and run on_refunded atomically; a transient
        // failure rolls the row back to `paid` and un-marks the event so a Stripe
        // retry reprocesses (the reconcile reaper is the secondary backstop).
        let txn = self.db.begin().await.map_err(PaymentError::Db)?;
        let marked = lifecycle::mark_refunded(intent.id, &txn).await?;
        if !marked {
            txn.rollback().await.ok();
            return Ok(()); // already refunded or wrong source state
        }

        // WR-05: persist the Stripe refund id so a later reconcile poll — and the
        // consumer's `on_refunded` sidecar — resolve by the exact refund.
        // Recent Stripe API versions omit the inline `refunds` list from
        // `charge.refunded`, so `event.refund_id` is frequently `None`; backfill
        // it from the PaymentIntent's refund list (query only, never refunds).
        let refund_id = match event.refund_id.clone() {
            Some(id) => Some(id),
            None => match &event.payment_intent_id {
                Some(pi) => self
                    .stripe
                    .latest_refund_id_for_payment_intent(pi)
                    .await
                    .ok()
                    .flatten(),
                None => None,
            },
        };
        if let Some(ref refund_id) = refund_id {
            lifecycle::attach_refund_id(intent.id, refund_id, &txn).await?;
        }

        let kind = BillableKind::from_string(intent.billable_kind.clone());
        match self.loader.load(kind, intent.billable_id).await {
            Ok(Some(billable)) => {
                match billable.on_refunded(&txn, refund_amount).await {
                    Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
                    Err(e) => {
                        txn.rollback().await.ok();
                        if is_transient(&e) {
                            // Transient — un-mark + propagate so Stripe retries
                            // (the status flip was rolled back with the txn).
                            self.processed_log.unmark(&event.event_id).await.ok();
                            return Err(e);
                        }
                        // Terminal business-state error — absorb; no retry loop.
                    }
                }
            }
            _ => {
                txn.rollback().await.ok();
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Auto-refund helper (D-09 / D-11)
    // -----------------------------------------------------------------------

    /// Snapshot `refund_amount_cents` then call Stripe's refund-by-payment_intent.
    ///
    /// The `WHERE refund_amount_cents IS NULL` dedup guard ensures exactly one
    /// concurrent caller calls Stripe. On Stripe failure the row is left as
    /// "refund-in-flight" (status=paid, refund_amount_cents set, refunded_at NULL)
    /// — D-11: do NOT compensate-reset. Phase-236 ReconcileRefundsInFlight recovers it.
    async fn trigger_auto_refund(
        &self,
        payment_intent_id: &Option<String>,
        amount_cents: i64,
        intent_id: i64,
        reason: AutoRefundReason,
    ) -> Result<(), PaymentError> {
        let Some(pi_id) = payment_intent_id else {
            tracing::debug!("auto_refund skipped: payment_intent_id absent (free/setup session)");
            return Ok(());
        };

        // D-09: atomic snapshot (WHERE IS NULL dedup — mirrors request_refund)
        let snapshot_ok = GuardedUpdate::new(Entity)
            .filter(Column::Id.eq(intent_id))
            .filter(Column::RefundAmountCents.is_null())
            .set_value(Column::RefundAmountCents, Value::BigInt(Some(amount_cents)))
            .exec_at_most_one(&self.db)
            .await
            .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))?;

        if !snapshot_ok {
            // Another caller already snapshotted the refund — no-op.
            return Ok(());
        }

        let idempotency_key = format!("auto-refund-{intent_id}");
        match self
            .stripe
            .create_refund_for_payment_intent(pi_id, Some(amount_cents), &idempotency_key)
            .await
        {
            Ok(refund_id) => {
                // WR-05: persist the refund id so the reconcile reaper resolves by
                // it. Best-effort — a failure here just falls back to the PI poll.
                lifecycle::attach_refund_id(intent_id, &refund_id, &self.db)
                    .await
                    .ok();
                tracing::warn!(
                    intent_id,
                    pi_id = %pi_id,
                    reason = ?reason,
                    "auto-refund triggered"
                );
            }
            Err(e) => {
                // D-11: do NOT compensate-reset refund_amount_cents on failure.
                // The row is now "refund-in-flight": status=paid, refund_amount_cents IS NOT NULL,
                // refunded_at IS NULL. Phase-236 ReconcileRefundsInFlight reaper is the recovery.
                tracing::error!(
                    intent_id,
                    pi_id = %pi_id,
                    reason = ?reason,
                    err = %e,
                    "auto-refund Stripe call failed; row is refund-in-flight (phase-236 reaper recovers)"
                );
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use sea_orm::{ConnectionTrait, Database, EntityTrait as _};
    use sea_orm_migration::MigratorTrait;

    use ferro_stripe::{
        MemoryProcessedLog, ProcessedEventLog, StripeCheckoutCompleted, StripeCheckoutExpired,
    };

    use crate::billable::Billable;
    use crate::error::PaymentError;
    use crate::intent::entity::Entity;
    use crate::intent::lifecycle;
    use crate::migration::m20260617_create_payment_intents::Migration as CreateTable;
    use crate::service::{CheckoutRequest, CheckoutResponse, ReturnUrls, StripeGateway};
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
        canned_pi_refund: Mutex<Option<Result<String, ferro_stripe::Error>>>,
        canned_latest_refund_id: Mutex<Option<String>>,
    }

    impl MockStripeGateway {
        fn pi_refund_calls(&self) -> Vec<(String, Option<i64>)> {
            self.pi_refund_calls.lock().unwrap().clone()
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
                .unwrap_or_else(|| {
                    Ok(CheckoutResponse {
                        intent: ferro_stripe::CheckoutIntent {
                            session_id: "cs_test_mock".to_string(),
                            url: "https://checkout.stripe.com/mock".to_string(),
                            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                            idempotency_key: "checkout-1".to_string(),
                        },
                        application_fee_cents: None,
                    })
                })
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
        ) -> Result<String, ferro_stripe::Error> {
            self.pi_refund_calls
                .lock()
                .unwrap()
                .push((payment_intent_id.to_string(), amount_cents));
            self.canned_pi_refund
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Ok("re_mock".to_string()))
        }

        async fn fetch_refund_status_for_payment_intent(
            &self,
            _payment_intent_id: &str,
        ) -> Result<crate::service::RefundStatus, ferro_stripe::Error> {
            Ok(crate::service::RefundStatus::Pending)
        }

        async fn fetch_refund_status_by_id(
            &self,
            _refund_id: &str,
        ) -> Result<crate::service::RefundStatus, ferro_stripe::Error> {
            Ok(crate::service::RefundStatus::Pending)
        }

        async fn latest_refund_id_for_payment_intent(
            &self,
            _payment_intent_id: &str,
        ) -> Result<Option<String>, ferro_stripe::Error> {
            Ok(self.canned_latest_refund_id.lock().unwrap().clone())
        }
    }

    // -----------------------------------------------------------------------
    // MockBillable — records which lifecycle hook was called
    // -----------------------------------------------------------------------

    #[derive(Default, Clone)]
    struct MockBillable {
        on_paid_count: Arc<std::sync::atomic::AtomicUsize>,
        on_released_count: Arc<std::sync::atomic::AtomicUsize>,
        on_refunded_calls: Arc<Mutex<Vec<i64>>>,
        on_paid_error: bool,
    }

    impl MockBillable {
        fn paid_count(&self) -> usize {
            self.on_paid_count.load(std::sync::atomic::Ordering::SeqCst)
        }
        fn released_count(&self) -> usize {
            self.on_released_count
                .load(std::sync::atomic::Ordering::SeqCst)
        }
        fn refunded_amounts(&self) -> Vec<i64> {
            self.on_refunded_calls.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl Billable for MockBillable {
        fn kind(&self) -> BillableKind {
            BillableKind::new("mock")
        }
        fn id(&self) -> i64 {
            1
        }
        fn tenant_id(&self) -> i64 {
            1
        }
        fn amount_cents(&self) -> i64 {
            1000
        }
        fn currency(&self) -> &str {
            "eur"
        }
        fn checkout_line_description(&self) -> String {
            "Mock item".to_string()
        }
        async fn on_paid(&self, _txn: &sea_orm::DatabaseTransaction) -> Result<(), PaymentError> {
            if self.on_paid_error {
                return Err(PaymentError::StatusPrecondition(
                    "mock on_paid error".to_string(),
                ));
            }
            self.on_paid_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        async fn on_released(
            &self,
            _txn: &sea_orm::DatabaseTransaction,
        ) -> Result<(), PaymentError> {
            self.on_released_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        async fn on_refunded(
            &self,
            _txn: &sea_orm::DatabaseTransaction,
            amount_cents: i64,
        ) -> Result<(), PaymentError> {
            self.on_refunded_calls.lock().unwrap().push(amount_cents);
            Ok(())
        }
    }

    // -----------------------------------------------------------------------
    // Configurable MockLoader
    // -----------------------------------------------------------------------

    enum LoaderBehavior {
        Billable(MockBillable),
        None,
        Error,
    }

    struct ConfigurableMockLoader {
        behavior: Mutex<LoaderBehavior>,
    }

    impl ConfigurableMockLoader {
        fn returning_billable(b: MockBillable) -> Self {
            Self {
                behavior: Mutex::new(LoaderBehavior::Billable(b)),
            }
        }
        fn returning_none() -> Self {
            Self {
                behavior: Mutex::new(LoaderBehavior::None),
            }
        }
        fn returning_error() -> Self {
            Self {
                behavior: Mutex::new(LoaderBehavior::Error),
            }
        }
    }

    #[async_trait::async_trait]
    impl BillableLoader for ConfigurableMockLoader {
        async fn load(
            &self,
            _kind: BillableKind,
            _id: i64,
        ) -> Result<Option<Box<dyn Billable>>, PaymentError> {
            match &*self.behavior.lock().unwrap() {
                LoaderBehavior::Billable(b) => Ok(Some(Box::new(b.clone()))),
                LoaderBehavior::None => Ok(Option::None),
                LoaderBehavior::Error => Err(PaymentError::Loader("mock loader error".into())),
            }
        }
    }

    // -----------------------------------------------------------------------
    // Event builders
    // -----------------------------------------------------------------------

    fn make_completed_event(
        session_id: &str,
        pi_id: Option<&str>,
        amount: i64,
    ) -> StripeCheckoutCompleted {
        StripeCheckoutCompleted {
            event_id: "evt_test_1".to_string(),
            session_id: session_id.to_string(),
            payment_intent_id: pi_id.map(str::to_string),
            amount_total_cents: amount,
            currency: "eur".to_string(),
            metadata: HashMap::default(),
            customer_email: None,
        }
    }

    fn make_expired_event(session_id: &str) -> StripeCheckoutExpired {
        StripeCheckoutExpired {
            event_id: "evt_test_2".to_string(),
            session_id: session_id.to_string(),
            metadata: HashMap::default(),
        }
    }

    fn make_charge_refunded_event(
        charge_id: &str,
        pi_id: Option<&str>,
        amount: i64,
    ) -> StripeChargeRefunded {
        StripeChargeRefunded {
            event_id: "evt_test_3".to_string(),
            charge_id: charge_id.to_string(),
            payment_intent_id: pi_id.map(str::to_string),
            refund_id: None,
            amount_refunded_cents: amount,
            metadata: HashMap::default(),
        }
    }

    /// Seed a reserved row with a session_id. Returns the row id.
    async fn seed_reserved_with_session(
        conn: &sea_orm::DatabaseConnection,
        billable_id: i64,
        session_id: &str,
    ) -> i64 {
        conn.execute_unprepared(&format!(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              stripe_session_id,expires_at,reserved_at) \
             VALUES (1,'mock',{billable_id},1000,'eur','reserved',\
             '{session_id}',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')"
        ))
        .await
        .expect("seed reserved row");

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

    /// Seed a paid row with payment_intent_id and optional charge_id. Returns the row id.
    async fn seed_paid_with_pi(
        conn: &sea_orm::DatabaseConnection,
        billable_id: i64,
        pi_id: &str,
        charge_id: Option<&str>,
    ) -> i64 {
        let charge_sql = match charge_id {
            Some(c) => format!("'{c}'"),
            None => "NULL".to_string(),
        };
        conn.execute_unprepared(&format!(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              payment_intent_id,charge_id,expires_at,reserved_at,paid_at) \
             VALUES (1,'mock',{billable_id},1000,'eur','paid',\
             '{pi_id}',{charge_sql},\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z','2026-06-17T00:00:00Z')"
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

    /// PAY-POLY-WH-02: session_completed happy path — mark_paid + on_paid +
    /// payment_intent_id attached; status = paid, no auto-refund.
    #[tokio::test]
    async fn handle_session_completed() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        let intent_db_id = seed_reserved_with_session(&db, 1, "cs_session_completed").await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_completed_event("cs_session_completed", Some("pi_abc123"), 1000);
        svc.handle_session_completed(event)
            .await
            .expect("handle ok");

        // Status must be paid
        let row = Entity::find_by_id(intent_db_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(row.status, crate::intent::status::PaymentIntentStatus::Paid);
        // payment_intent_id must be attached
        assert_eq!(row.payment_intent_id, Some("pi_abc123".to_string()));
        // on_paid must have been called once
        assert_eq!(billable.paid_count(), 1, "on_paid must be called once");
        // no auto-refund
        assert!(
            mock_stripe.pi_refund_calls().is_empty(),
            "no auto-refund on happy path"
        );
    }

    /// PAY-POLY-WH-02: replay — second dispatch of same event_id is a no-op.
    #[tokio::test]
    async fn handle_session_completed_replay() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());
        let log: Arc<dyn ProcessedEventLog> = Arc::new(MemoryProcessedLog::new());

        seed_reserved_with_session(&db, 2, "cs_replay").await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::clone(&log),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_completed_event("cs_replay", Some("pi_replay"), 1000);
        // First call
        svc.handle_session_completed(event.clone())
            .await
            .expect("first call ok");
        // Second call with same event_id — must be no-op
        svc.handle_session_completed(event.clone())
            .await
            .expect("second call ok");

        // on_paid must be called exactly once
        assert_eq!(
            billable.paid_count(),
            1,
            "on_paid must be called exactly once across two dispatches"
        );
    }

    /// PAY-POLY-WH-02/06: side-state conflict — mark_paid returns Ok(false) because
    /// row was already released; trigger_auto_refund fires exactly once.
    #[tokio::test]
    async fn handle_session_completed_side_state_conflict() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        let intent_db_id = seed_reserved_with_session(&db, 3, "cs_conflict").await;

        // Pre-release the row (simulates reaper winning)
        lifecycle::mark_released(intent_db_id, &db)
            .await
            .expect("mark_released");

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_completed_event("cs_conflict", Some("pi_conflict"), 1000);
        svc.handle_session_completed(event)
            .await
            .expect("handle returns Ok");

        // auto-refund must have been called exactly once with the right pi_id and amount
        let calls = mock_stripe.pi_refund_calls();
        assert_eq!(calls.len(), 1, "auto-refund must fire exactly once");
        assert_eq!(calls[0].0, "pi_conflict");
        assert_eq!(calls[0].1, Some(1000));
    }

    /// PAY-POLY-WH-03: session_expired happy path — mark_released + on_released.
    #[tokio::test]
    async fn handle_session_expired() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        let intent_db_id = seed_reserved_with_session(&db, 4, "cs_expired_happy").await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_expired_event("cs_expired_happy");
        svc.handle_session_expired(event).await.expect("handle ok");

        let row = Entity::find_by_id(intent_db_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.status,
            crate::intent::status::PaymentIntentStatus::Released
        );
        assert_eq!(billable.released_count(), 1, "on_released must be called");
    }

    /// PAY-POLY-WH-03: already-released row → mark_released Ok(false) → no-op.
    #[tokio::test]
    async fn handle_session_expired_noop() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        let intent_db_id = seed_reserved_with_session(&db, 5, "cs_expired_noop").await;
        // Pre-release
        lifecycle::mark_released(intent_db_id, &db)
            .await
            .expect("mark_released");

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_expired_event("cs_expired_noop");
        svc.handle_session_expired(event).await.expect("noop ok");

        // on_released must NOT be called (mark_released returned Ok(false))
        assert_eq!(
            billable.released_count(),
            0,
            "on_released must not be called for already-released row"
        );
    }

    /// PAY-POLY-WH-04: charge_refunded — find_by_payment_intent + mark_refunded +
    /// on_refunded(amount_cents).
    #[tokio::test]
    async fn handle_charge_refunded() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        let intent_db_id = seed_paid_with_pi(&db, 6, "pi_charge_refunded", Some("ch_abc")).await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_charge_refunded_event("ch_abc", Some("pi_charge_refunded"), 750);
        svc.handle_charge_refunded(event).await.expect("handle ok");

        let row = Entity::find_by_id(intent_db_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.status,
            crate::intent::status::PaymentIntentStatus::Refunded
        );
        let amounts = billable.refunded_amounts();
        assert_eq!(amounts.len(), 1);
        assert_eq!(
            amounts[0], 750,
            "on_refunded must receive amount_refunded_cents"
        );
    }

    /// PAY-POLY-WH-05: loader returns Ok(None) → auto-refund called exactly once;
    /// status stays paid, refund_amount_cents snapshotted.
    #[tokio::test]
    async fn auto_refund_billable_vanished() {
        let db = fresh_db().await;
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_none();

        seed_reserved_with_session(&db, 7, "cs_vanished").await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_completed_event("cs_vanished", Some("pi_vanished"), 1000);
        svc.handle_session_completed(event)
            .await
            .expect("handle ok");

        // auto-refund called exactly once
        let calls = mock_stripe.pi_refund_calls();
        assert_eq!(calls.len(), 1, "auto-refund must fire exactly once");
        assert_eq!(calls[0].0, "pi_vanished");
        assert_eq!(calls[0].1, Some(1000));

        // row status still paid (auto-refund does not revert status)
        let row = lifecycle::find_by_payment_intent("pi_vanished", &db)
            .await
            .expect("find")
            .expect("row exists");
        assert_eq!(row.status, crate::intent::status::PaymentIntentStatus::Paid);
        assert_eq!(
            row.refund_amount_cents,
            Some(1000),
            "refund_amount_cents must be snapshotted"
        );
    }

    /// PAY-POLY-WH-05: loader returns Err → auto-refund called exactly once.
    #[tokio::test]
    async fn auto_refund_loader_error() {
        let db = fresh_db().await;
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_error();

        seed_reserved_with_session(&db, 8, "cs_loaderr").await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_completed_event("cs_loaderr", Some("pi_loaderr"), 1000);
        svc.handle_session_completed(event)
            .await
            .expect("handle ok");

        let calls = mock_stripe.pi_refund_calls();
        assert_eq!(
            calls.len(),
            1,
            "auto-refund must fire exactly once on loader error"
        );
        assert_eq!(calls[0].0, "pi_loaderr");
    }

    /// PAY-POLY-WH-06: webhook+reaper interleaved → exactly one side-effect.
    ///
    /// Simulates the reaper marking_released before the webhook marks_paid.
    /// GuardedUpdate ensures exactly one wins; the loser produces no side effect
    /// or fires auto-refund.
    #[tokio::test]
    async fn webhook_reaper_race() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        let intent_db_id = seed_reserved_with_session(&db, 9, "cs_race").await;

        // Reaper wins: mark_released before handle_session_completed runs
        let released = lifecycle::mark_released(intent_db_id, &db)
            .await
            .expect("reaper mark_released");
        assert!(released, "reaper must win the race");

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_completed_event("cs_race", Some("pi_race"), 1000);
        svc.handle_session_completed(event)
            .await
            .expect("handle ok");

        // on_paid must NOT have been called (reaper won)
        assert_eq!(
            billable.paid_count(),
            0,
            "on_paid must not fire when reaper won"
        );

        // auto-refund must fire exactly once (mark_paid returned Ok(false))
        let calls = mock_stripe.pi_refund_calls();
        assert_eq!(
            calls.len(),
            1,
            "auto-refund must fire once when webhook lost the race"
        );
    }

    /// PAY-POLY-WH-06: paid-after-released — mark_paid Ok(false) → auto-refund.
    #[tokio::test]
    async fn paid_after_released() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        let intent_db_id = seed_reserved_with_session(&db, 10, "cs_paid_after_released").await;
        // Pre-release
        lifecycle::mark_released(intent_db_id, &db)
            .await
            .expect("mark_released");

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let event = make_completed_event(
            "cs_paid_after_released",
            Some("pi_paid_after_released"),
            1000,
        );
        svc.handle_session_completed(event)
            .await
            .expect("handle ok");

        // auto-refund must fire
        let calls = mock_stripe.pi_refund_calls();
        assert_eq!(calls.len(), 1, "auto-refund fires on paid-after-released");
        assert_eq!(calls[0].0, "pi_paid_after_released");
    }

    /// PAY-POLY-WH-04: charge_refunded fallback — payment_intent_id None →
    /// find_by_charge_id resolves the row.
    #[tokio::test]
    async fn handle_charge_refunded_charge_id_fallback() {
        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        // Seed with charge_id but no payment_intent_id
        let intent_db_id =
            seed_paid_with_pi(&db, 11, "pi_fallback_not_in_event", Some("ch_fallback")).await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        // Event has no payment_intent_id → must fall back to charge_id lookup
        let event = make_charge_refunded_event("ch_fallback", None, 500);
        svc.handle_charge_refunded(event).await.expect("handle ok");

        let row = Entity::find_by_id(intent_db_id)
            .one(&db)
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(
            row.status,
            crate::intent::status::PaymentIntentStatus::Refunded,
            "row must be refunded via charge_id fallback"
        );
        let amounts = billable.refunded_amounts();
        assert_eq!(amounts.len(), 1, "on_refunded must be called");
        assert_eq!(amounts[0], 500);
    }

    /// PAY-POLY-WH-01: wire_dispatcher registers three handlers; dispatching
    /// a checkout.session.completed event routes to handle_session_completed
    /// (observe the side-effect on the DB).
    #[tokio::test]
    async fn wire_dispatcher_registers_three_handlers() {
        use ferro_stripe::{SyncDispatcher, WebhookEvent};

        let db = fresh_db().await;
        let billable = MockBillable::default();
        let mock_stripe = Arc::new(MockStripeGateway::default());
        let loader = ConfigurableMockLoader::returning_billable(billable.clone());

        seed_reserved_with_session(&db, 12, "cs_wire_test").await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock_stripe.clone(),
            loader,
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        let dispatcher = wire_dispatcher(SyncDispatcher::new(), svc.clone());

        // Build a raw WebhookEvent for checkout.session.completed
        let raw_json = serde_json::json!({
            "id": "evt_wire_test",
            "type": "checkout.session.completed",
            "data": {
                "object": {
                    "id": "cs_wire_test",
                    "payment_intent": "pi_wire",
                    "amount_total": 1000,
                    "currency": "eur"
                }
            }
        })
        .to_string();
        let event = WebhookEvent::from_json(&raw_json).expect("parse event");

        // Dispatch — must route to handle_session_completed
        dispatcher.dispatch(event).await.expect("dispatch ok");

        // Verify the side-effect: status must be paid
        let row = lifecycle::find_by_payment_intent("pi_wire", &db)
            .await
            .expect("find")
            .expect("row exists after dispatch");
        assert_eq!(
            row.status,
            crate::intent::status::PaymentIntentStatus::Paid,
            "dispatching checkout.session.completed must mark row paid"
        );
        assert_eq!(
            billable.paid_count(),
            1,
            "on_paid must be called through the dispatcher"
        );
    }

    /// WR-01: a transient on_paid failure must NOT lose the capture. The status
    /// flip rolls back to `reserved` and the event is un-marked, so Stripe's
    /// retry reprocesses and on_paid completes — no auto-refund, no lost update.
    #[tokio::test]
    async fn handle_session_completed_transient_recovers() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc as StdArc;

        #[derive(Clone)]
        struct FlakyBillable {
            attempts: StdArc<AtomicUsize>,
        }

        #[async_trait::async_trait]
        impl Billable for FlakyBillable {
            fn kind(&self) -> BillableKind {
                BillableKind::new("mock")
            }
            fn id(&self) -> i64 {
                1
            }
            fn tenant_id(&self) -> i64 {
                1
            }
            fn amount_cents(&self) -> i64 {
                1000
            }
            fn currency(&self) -> &str {
                "eur"
            }
            fn checkout_line_description(&self) -> String {
                "flaky".to_string()
            }
            async fn on_paid(
                &self,
                _txn: &sea_orm::DatabaseTransaction,
            ) -> Result<(), PaymentError> {
                // First attempt: transient (Stripe) error → must be retried.
                if self.attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                    return Err(PaymentError::Stripe(ferro_stripe::Error::Stripe(
                        "transient".to_string(),
                    )));
                }
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

        struct FlakyLoader {
            attempts: StdArc<AtomicUsize>,
        }

        #[async_trait::async_trait]
        impl BillableLoader for FlakyLoader {
            async fn load(
                &self,
                _kind: BillableKind,
                _id: i64,
            ) -> Result<Option<Box<dyn Billable>>, PaymentError> {
                Ok(Some(Box::new(FlakyBillable {
                    attempts: self.attempts.clone(),
                })))
            }
        }

        let db = fresh_db().await;
        let mock = Arc::new(MockStripeGateway::default());
        let attempts = StdArc::new(AtomicUsize::new(0));
        seed_reserved_with_session(&db, 42, "cs_flaky").await;

        let svc = Arc::new(PaymentService::new(
            db.clone(),
            mock.clone(),
            FlakyLoader {
                attempts: attempts.clone(),
            },
            Arc::new(MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ));

        // First delivery: transient on_paid failure → Err, row rolled back to reserved.
        let r1 = svc
            .handle_session_completed(make_completed_event("cs_flaky", Some("pi_flaky"), 1000))
            .await;
        assert!(r1.is_err(), "transient failure must propagate as Err");
        let row = lifecycle::find_by_stripe_session("cs_flaky", &db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.status,
            crate::intent::status::PaymentIntentStatus::Reserved,
            "WR-01: row must roll back to reserved on transient failure (not stuck paid)"
        );

        // Retry (same event_id): the event was un-marked, so it reprocesses.
        let r2 = svc
            .handle_session_completed(make_completed_event("cs_flaky", Some("pi_flaky"), 1000))
            .await;
        assert!(r2.is_ok(), "retry must succeed after transient failure");
        let row = lifecycle::find_by_stripe_session("cs_flaky", &db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.status,
            crate::intent::status::PaymentIntentStatus::Paid,
            "retry must complete the capture"
        );
        assert_eq!(
            attempts.load(Ordering::SeqCst),
            2,
            "on_paid attempted twice (fail, then succeed)"
        );
        assert_eq!(
            mock.pi_refund_calls().len(),
            0,
            "transient recovery must NOT auto-refund"
        );
    }
}
