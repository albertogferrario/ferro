//! Queue-based Stripe webhook dispatch (eventual-consistency path).
//!
//! [`ProcessStripeWebhook`] is a [`ferro_queue::Job`] that holds a raw
//! Stripe event body and dispatches it through a shared
//! [`SyncDispatcher`] on the background worker. The same dispatcher
//! instance serves both the synchronous HTTP path and the queue path —
//! register handlers once with [`SyncDispatcher::on`] and enqueue this
//! job for events that tolerate eventual consistency.
//!
//! # Persistence
//!
//! The job persists `event_type` and `raw_body` through serde. The
//! `dispatcher` field is runtime-only (`#[serde(skip)]`) and must be
//! re-injected by the caller at enqueue time via
//! [`ProcessStripeWebhook::new`]. Jobs deserialized without a dispatcher
//! will panic on [`handle`][ferro_queue::Job::handle] — this is a
//! programming error, not a runtime condition.
//!
//! [`SyncDispatcher`]: crate::webhook::sync::SyncDispatcher
//! [`SyncDispatcher::on`]: crate::webhook::sync::SyncDispatcher::on

use std::sync::Arc;

use crate::webhook::sync::SyncDispatcher;

/// Background job that dispatches a verified Stripe webhook event through
/// a shared [`SyncDispatcher`].
///
/// [`SyncDispatcher`]: crate::webhook::sync::SyncDispatcher
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessStripeWebhook {
    /// The Stripe event type string (e.g. `"checkout.session.completed"`).
    pub event_type: String,
    /// The raw JSON body of the already-verified Stripe event.
    pub raw_body: String,
    /// Connect account id for Connect webhooks; `None` for platform events.
    pub connect_account_id: Option<String>,
    /// Runtime-only: not persisted by the queue. Injected via
    /// [`ProcessStripeWebhook::new`] at enqueue time.
    #[serde(skip)]
    pub dispatcher: Option<Arc<SyncDispatcher>>,
}

impl ProcessStripeWebhook {
    /// Constructs a new queue job with the dispatcher attached.
    ///
    /// Use this constructor at enqueue time — the dispatcher is not
    /// persisted, so jobs deserialized from storage without
    /// re-injection cannot execute.
    pub fn new(
        event_type: String,
        raw_body: String,
        connect_account_id: Option<String>,
        dispatcher: Arc<SyncDispatcher>,
    ) -> Self {
        Self {
            event_type,
            raw_body,
            connect_account_id,
            dispatcher: Some(dispatcher),
        }
    }
}

#[ferro_queue::async_trait]
impl ferro_queue::Job for ProcessStripeWebhook {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let dispatcher = self.dispatcher.as_ref().expect(
            "ProcessStripeWebhook requires dispatcher — use ProcessStripeWebhook::new()",
        );
        let event: stripe::Event = serde_json::from_str(&self.raw_body).map_err(|e| {
            ferro_queue::Error::JobFailed {
                job: "ProcessStripeWebhook".to_string(),
                message: format!("parse stripe event: {e}"),
            }
        })?;
        dispatcher.dispatch(event).await.map_err(|e| ferro_queue::Error::JobFailed {
            job: "ProcessStripeWebhook".to_string(),
            message: e.to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "ProcessStripeWebhook"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhook::events::StripeCheckoutCompleted;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn process_stripe_webhook_job_name() {
        let dispatcher = Arc::new(SyncDispatcher::new());
        let job = ProcessStripeWebhook::new(
            "invoice.paid".to_string(),
            "{}".to_string(),
            None,
            dispatcher,
        );
        use ferro_queue::Job;
        assert_eq!(job.name(), "ProcessStripeWebhook");
    }

    #[test]
    fn new_sets_dispatcher_to_some() {
        let dispatcher = Arc::new(SyncDispatcher::new());
        let job = ProcessStripeWebhook::new(
            "checkout.session.completed".to_string(),
            "{}".to_string(),
            None,
            Arc::clone(&dispatcher),
        );
        assert!(job.dispatcher.is_some());
    }

    #[tokio::test]
    async fn handle_dispatches_parsed_event_through_dispatcher() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&flag);
        let dispatcher = Arc::new(
            SyncDispatcher::new().on(move |_: StripeCheckoutCompleted| {
                let flag = Arc::clone(&flag_clone);
                async move {
                    flag.store(true, Ordering::SeqCst);
                    Ok(())
                }
            }),
        );

        // Reuse Plan 03 fixture for a realistic stripe::Event JSON body.
        let raw = include_str!(
            "../../tests/fixtures/stripe_events/checkout_session_completed.json"
        );

        let job = ProcessStripeWebhook::new(
            "checkout.session.completed".to_string(),
            raw.to_string(),
            None,
            dispatcher,
        );

        use ferro_queue::Job;
        let result = job.handle().await;
        assert!(result.is_ok(), "handle should succeed, got {result:?}");
        assert!(flag.load(Ordering::SeqCst), "registered handler should have run");
    }

    #[tokio::test]
    async fn handle_maps_parse_errors_to_job_failed() {
        let dispatcher = Arc::new(SyncDispatcher::new());
        let job = ProcessStripeWebhook::new(
            "invoice.paid".to_string(),
            "not-json".to_string(),
            None,
            dispatcher,
        );

        use ferro_queue::Job;
        let result = job.handle().await;
        match result {
            Err(ferro_queue::Error::JobFailed { message, .. }) => {
                assert!(message.starts_with("parse stripe event:"), "got: {message}");
            }
            other => panic!("expected JobFailed, got {other:?}"),
        }
    }
}
