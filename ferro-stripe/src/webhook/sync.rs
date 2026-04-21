//! Synchronous Stripe webhook dispatch.
//!
//! [`SyncDispatcher`] is the handler registry for typed Stripe webhook
//! events. Register handlers with [`SyncDispatcher::on`] (consuming
//! builder) and call [`SyncDispatcher::dispatch`] from your webhook HTTP
//! endpoint for payment-correctness events that must run synchronously
//! before responding to Stripe. For eventual-consistency events, enqueue
//! `webhook::queue::ProcessStripeWebhook` instead — the same dispatcher
//! instance serves both paths.
//!
//! # Example
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use ferro_stripe::{SyncDispatcher, StripeCheckoutCompleted};
//!
//! let dispatcher = Arc::new(
//!     SyncDispatcher::new()
//!         .on(|event: StripeCheckoutCompleted| async move {
//!             tracing::info!(session = %event.session_id, "checkout completed");
//!             Ok(())
//!         })
//! );
//!
//! // Later, inside the HTTP webhook handler:
//! // let event = verify_webhook(body, sig, secret)?;
//! // dispatcher.dispatch(event).await?;
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::webhook::events::StripeEvent;
use crate::Error;

type BoxedHandler = Box<
    dyn Fn(stripe::Event) -> Pin<Box<dyn Future<Output = (bool, Result<(), Error>)> + Send>>
        + Send
        + Sync,
>;

/// Registry of typed Stripe webhook handlers.
///
/// Use [`SyncDispatcher::new`] + [`SyncDispatcher::on`] to build the
/// registry, then share an `Arc<SyncDispatcher>` across tasks. The
/// dispatcher is `Send + Sync` and safe to call from any executor.
pub struct SyncDispatcher {
    handlers: Vec<BoxedHandler>,
}

impl SyncDispatcher {
    /// Creates an empty dispatcher with no registered handlers.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Registers a handler for a single typed event `E`.
    ///
    /// The handler is invoked when [`StripeEvent::from_raw`] returns
    /// `Some` for the event type at dispatch time. Multiple handlers may
    /// be registered; all matching handlers are invoked in registration
    /// order and the first error short-circuits dispatch.
    pub fn on<E, H, Fut>(mut self, handler: H) -> Self
    where
        E: StripeEvent,
        H: Fn(E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.handlers.push(Box::new(move |event: stripe::Event| {
            let handler = Arc::clone(&handler);
            Box::pin(async move {
                match E::from_raw(&event) {
                    Some(typed) => {
                        let result = handler(typed).await;
                        (true, result)
                    }
                    None => (false, Ok(())),
                }
            })
        }));
        self
    }

    /// Dispatches a verified Stripe event through all registered handlers.
    ///
    /// # Errors
    ///
    /// Returns the first [`Error`] produced by a matching handler. When
    /// no registered handler matches the event type, logs at
    /// `tracing::debug` and returns `Ok(())`.
    pub async fn dispatch(&self, event: stripe::Event) -> Result<(), Error> {
        let mut any_matched = false;
        for handler in &self.handlers {
            let (matched, result) = handler(event.clone()).await;
            if matched {
                any_matched = true;
                result?;
            }
        }
        if !any_matched {
            tracing::debug!(
                event_type = ?event.type_,
                event_id = %event.id,
                "unregistered stripe event type — skipping"
            );
        }
        Ok(())
    }
}

impl Default for SyncDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SyncDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncDispatcher")
            .field("handlers_count", &self.handlers.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn sync_dispatcher_is_send_and_sync() {
        _assert_send_sync::<SyncDispatcher>();
        _assert_send_sync::<Arc<SyncDispatcher>>();
    }

    #[test]
    fn new_dispatcher_has_no_handlers() {
        let d = SyncDispatcher::new();
        assert!(d.handlers.is_empty());
    }

    #[test]
    fn default_dispatcher_has_no_handlers() {
        let d = SyncDispatcher::default();
        assert!(d.handlers.is_empty());
    }
}
