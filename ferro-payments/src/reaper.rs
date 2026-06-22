//! Queue-compatible reaper jobs wrapping [`PaymentService`] recovery methods.
//!
//! Both jobs follow the [`ferro_stripe::ProcessStripeWebhook`] template exactly:
//! a serde-skipped `Arc<PaymentService<L>>` handle injected at enqueue time via
//! `::new()`. A job deserialized without a handle returns
//! [`ferro_queue::Error::JobFailed`] on [`handle`][ferro_queue::Job::handle]
//! so the queue can surface the failure without panicking or silently no-oping.
//!
//! # Consumer registration
//!
//! ```rust,ignore
//! // At application startup (once):
//! let svc = Arc::new(payment_service);
//! // Schedule via the cron facility — two entries suffice:
//! queue.schedule_cron("0 * * * *", ReleaseExpiredPaymentIntents::new(Arc::clone(&svc)));
//! queue.schedule_cron("30 * * * *", ReconcileRefundsInFlight::new(Arc::clone(&svc)));
//! ```

use std::sync::Arc;

use crate::loader::BillableLoader;
use crate::service::PaymentService;

// ---------------------------------------------------------------------------
// ReleaseExpiredPaymentIntents
// ---------------------------------------------------------------------------

/// ferro-queue job: release reserved intents whose hold has expired
/// (PAY-POLY-REAP-01). Schedule via cron; thin wrapper over
/// [`PaymentService::release_expired`]. Inject the service via `::new` at
/// consumer-registration time.
///
/// The struct has no serialized identity fields — rows are selected at
/// execution time from the database, so a forged payload cannot smuggle a
/// billable id or amount (T-236-06).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct ReleaseExpiredPaymentIntents<L: BillableLoader + 'static> {
    /// Runtime-only: not persisted by the queue. Injected via `::new` at
    /// enqueue / registration time.
    #[serde(skip)]
    pub service: Option<Arc<PaymentService<L>>>,
}

impl<L: BillableLoader + 'static> std::fmt::Debug for ReleaseExpiredPaymentIntents<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReleaseExpiredPaymentIntents")
            .field("service", &self.service.as_ref().map(|_| "<injected>"))
            .finish()
    }
}

impl<L: BillableLoader + 'static> ReleaseExpiredPaymentIntents<L> {
    /// Constructs a new job with the service handle attached.
    ///
    /// Use this constructor at registration time — the handle is not
    /// persisted, so a deserialized job without re-injection cannot execute
    /// (T-236-05: errors loudly rather than panicking or no-oping).
    pub fn new(service: Arc<PaymentService<L>>) -> Self {
        Self {
            service: Some(service),
        }
    }
}

#[ferro_queue::async_trait]
impl<L: BillableLoader + 'static> ferro_queue::Job for ReleaseExpiredPaymentIntents<L> {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let svc = self
            .service
            .as_ref()
            .ok_or_else(|| ferro_queue::Error::JobFailed {
                job: "ReleaseExpiredPaymentIntents".to_string(),
                message: "service not injected — use ReleaseExpiredPaymentIntents::new()"
                    .to_string(),
            })?;
        svc.release_expired()
            .await
            .map(|_| ())
            .map_err(|e| ferro_queue::Error::JobFailed {
                job: "ReleaseExpiredPaymentIntents".to_string(),
                message: e.to_string(),
            })
    }

    fn name(&self) -> &'static str {
        "ReleaseExpiredPaymentIntents"
    }
}

// ---------------------------------------------------------------------------
// ReconcileRefundsInFlight
// ---------------------------------------------------------------------------

/// ferro-queue job: poll Stripe for refund-in-flight intents and resolve
/// succeeded refunds (PAY-POLY-REAP-02). Schedule via cron; thin wrapper over
/// [`PaymentService::reconcile_refunds_in_flight`]. Inject the service via
/// `::new` at consumer-registration time.
///
/// Like [`ReleaseExpiredPaymentIntents`], the struct carries no serialized
/// identity fields — row selection is done at execution time (T-236-06).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct ReconcileRefundsInFlight<L: BillableLoader + 'static> {
    /// Runtime-only: not persisted by the queue. Injected via `::new` at
    /// enqueue / registration time.
    #[serde(skip)]
    pub service: Option<Arc<PaymentService<L>>>,
}

impl<L: BillableLoader + 'static> std::fmt::Debug for ReconcileRefundsInFlight<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconcileRefundsInFlight")
            .field("service", &self.service.as_ref().map(|_| "<injected>"))
            .finish()
    }
}

impl<L: BillableLoader + 'static> ReconcileRefundsInFlight<L> {
    /// Constructs a new job with the service handle attached.
    ///
    /// Use this constructor at registration time — the handle is not
    /// persisted, so a deserialized job without re-injection cannot execute
    /// (T-236-05: errors loudly rather than panicking or no-oping).
    pub fn new(service: Arc<PaymentService<L>>) -> Self {
        Self {
            service: Some(service),
        }
    }
}

#[ferro_queue::async_trait]
impl<L: BillableLoader + 'static> ferro_queue::Job for ReconcileRefundsInFlight<L> {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let svc = self
            .service
            .as_ref()
            .ok_or_else(|| ferro_queue::Error::JobFailed {
                job: "ReconcileRefundsInFlight".to_string(),
                message: "service not injected — use ReconcileRefundsInFlight::new()".to_string(),
            })?;
        svc.reconcile_refunds_in_flight()
            .await
            .map(|_| ())
            .map_err(|e| ferro_queue::Error::JobFailed {
                job: "ReconcileRefundsInFlight".to_string(),
                message: e.to_string(),
            })
    }

    fn name(&self) -> &'static str {
        "ReconcileRefundsInFlight"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use sea_orm::{Database, DatabaseConnection};
    use sea_orm_migration::MigratorTrait;

    use crate::billable::Billable;
    use crate::error::PaymentError;
    use crate::migration::m20260617_create_payment_intents::Migration as CreateTable;
    use crate::service::{CheckoutRequest, CheckoutResponse, ReturnUrls, StripeGateway};
    use crate::{BillableKind, BillableLoader};

    // -----------------------------------------------------------------------
    // Test DB infrastructure (mirrors service.rs tests)
    // -----------------------------------------------------------------------

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(CreateTable)]
        }
    }

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");
        TestMigrator::up(&conn, None).await.expect("migrate up");
        conn
    }

    // -----------------------------------------------------------------------
    // Minimal MockStripeGateway (no recording needed for reaper job tests)
    // -----------------------------------------------------------------------

    struct MockStripeGateway;

    #[async_trait::async_trait]
    impl StripeGateway for MockStripeGateway {
        async fn create_checkout_session(
            &self,
            _req: CheckoutRequest,
        ) -> Result<CheckoutResponse, ferro_stripe::Error> {
            Ok(CheckoutResponse {
                intent: ferro_stripe::CheckoutIntent {
                    session_id: "cs_test".to_string(),
                    url: "https://checkout.stripe.com/mock".to_string(),
                    expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                    idempotency_key: "checkout-1".to_string(),
                },
                application_fee_cents: None,
            })
        }

        async fn create_refund(
            &self,
            _charge_id: &str,
            _amount_cents: Option<i64>,
            _idempotency_key: &str,
        ) -> Result<(), ferro_stripe::Error> {
            Ok(())
        }

        async fn create_refund_for_payment_intent(
            &self,
            _payment_intent_id: &str,
            _amount_cents: Option<i64>,
            _idempotency_key: &str,
        ) -> Result<String, ferro_stripe::Error> {
            Ok("re_mock".to_string())
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
            Ok(None)
        }
    }

    // -----------------------------------------------------------------------
    // Minimal MockLoader (always returns None — no money on reserved rows)
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

    fn make_svc(db: DatabaseConnection) -> Arc<PaymentService<MockLoader>> {
        Arc::new(PaymentService::new(
            db,
            Arc::new(MockStripeGateway),
            MockLoader,
            Arc::new(ferro_stripe::MemoryProcessedLog::new()),
            |_b| ReturnUrls {
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
            },
        ))
    }

    // -----------------------------------------------------------------------
    // Behavior tests
    // -----------------------------------------------------------------------

    /// ReleaseExpiredPaymentIntents::new sets service to Some.
    #[tokio::test]
    async fn job_struct_release_new_sets_service_to_some() {
        let db = fresh_db().await;
        let svc = make_svc(db);
        let job = ReleaseExpiredPaymentIntents::new(Arc::clone(&svc));
        assert!(job.service.is_some());
    }

    /// job.name() returns the correct static string for ReleaseExpiredPaymentIntents.
    #[tokio::test]
    async fn job_struct_release_name() {
        let db = fresh_db().await;
        let svc = make_svc(db);
        let job = ReleaseExpiredPaymentIntents::new(Arc::clone(&svc));
        use ferro_queue::Job;
        assert_eq!(job.name(), "ReleaseExpiredPaymentIntents");
    }

    /// ReconcileRefundsInFlight::new sets service to Some.
    #[tokio::test]
    async fn job_struct_reconcile_new_sets_service_to_some() {
        let db = fresh_db().await;
        let svc = make_svc(db);
        let job = ReconcileRefundsInFlight::new(Arc::clone(&svc));
        assert!(job.service.is_some());
    }

    /// job.name() returns the correct static string for ReconcileRefundsInFlight.
    #[tokio::test]
    async fn job_struct_reconcile_name() {
        let db = fresh_db().await;
        let svc = make_svc(db);
        let job = ReconcileRefundsInFlight::new(Arc::clone(&svc));
        use ferro_queue::Job;
        assert_eq!(job.name(), "ReconcileRefundsInFlight");
    }

    /// handle() with no service injected (service == None) returns JobFailed
    /// for ReleaseExpiredPaymentIntents (T-236-05).
    #[tokio::test]
    async fn job_no_service_injected_release() {
        let job: ReleaseExpiredPaymentIntents<MockLoader> =
            ReleaseExpiredPaymentIntents { service: None };
        use ferro_queue::Job;
        let result = job.handle().await;
        match result {
            Err(ferro_queue::Error::JobFailed { job, message }) => {
                assert_eq!(job, "ReleaseExpiredPaymentIntents");
                assert!(
                    message.contains("ReleaseExpiredPaymentIntents::new()"),
                    "message should name the constructor: {message}"
                );
            }
            other => panic!("expected JobFailed, got {other:?}"),
        }
    }

    /// handle() with no service injected returns JobFailed for ReconcileRefundsInFlight
    /// (T-236-05).
    #[tokio::test]
    async fn job_no_service_injected() {
        let job: ReconcileRefundsInFlight<MockLoader> = ReconcileRefundsInFlight { service: None };
        use ferro_queue::Job;
        let result = job.handle().await;
        match result {
            Err(ferro_queue::Error::JobFailed { job, message }) => {
                assert_eq!(job, "ReconcileRefundsInFlight");
                assert!(
                    message.contains("ReconcileRefundsInFlight::new()"),
                    "message should name the constructor: {message}"
                );
            }
            other => panic!("expected JobFailed, got {other:?}"),
        }
    }

    /// A default-deserialized job (from empty JSON "{}") has service == None
    /// (serde skip default — T-236-06).
    #[test]
    fn deserialized_job_has_service_none() {
        let job: ReleaseExpiredPaymentIntents<MockLoader> =
            serde_json::from_str("{}").expect("deserialize empty JSON");
        assert!(
            job.service.is_none(),
            "deserialized job must have service == None (serde skip default)"
        );

        let job2: ReconcileRefundsInFlight<MockLoader> =
            serde_json::from_str("{}").expect("deserialize empty JSON");
        assert!(
            job2.service.is_none(),
            "deserialized job must have service == None (serde skip default)"
        );
    }

    /// handle() with an injected service calls release_expired and maps success to Ok(()).
    #[tokio::test]
    async fn job_struct_release() {
        let db = fresh_db().await;
        let svc = make_svc(db);
        let job = ReleaseExpiredPaymentIntents::new(Arc::clone(&svc));
        use ferro_queue::Job;
        // No expired rows in the DB — expects Ok(()) with count = 0.
        let result = job.handle().await;
        assert!(
            result.is_ok(),
            "handle should succeed on empty DB: {result:?}"
        );
    }

    /// handle() with an injected service calls reconcile_refunds_in_flight
    /// and maps success to Ok(()).
    #[tokio::test]
    async fn job_struct_reconcile() {
        let db = fresh_db().await;
        let svc = make_svc(db);
        let job = ReconcileRefundsInFlight::new(Arc::clone(&svc));
        use ferro_queue::Job;
        // No refund-in-flight rows — expects Ok(()) with count = 0.
        let result = job.handle().await;
        assert!(
            result.is_ok(),
            "handle should succeed on empty DB: {result:?}"
        );
    }
}
