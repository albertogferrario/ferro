//! Integration tests for `SyncDispatcher` — covers D-23 success criteria:
//! Err bubbles, Ok passes, unknown events no-op, `Arc<SyncDispatcher>` is
//! thread-safe across tokio tasks.
//!
//! Run with: `cargo test -p ferro-stripe --all-features --test dispatcher`
//! The `--all-features` flag enables the `test-helpers` feature which exposes
//! the `ferro_stripe::testing` module used for event construction.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ferro_stripe::testing::{mock_checkout_completed_event, mock_invoice_paid_event};
use ferro_stripe::{Error, StripeCheckoutCompleted, StripeInvoicePaid, SyncDispatcher};

fn parse_event(raw: &str) -> stripe::Event {
    serde_json::from_str::<stripe::Event>(raw)
        .expect("mock event JSON should deserialize as stripe::Event")
}

#[tokio::test]
async fn dispatch_ok_path_completes_and_invokes_handler() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = Arc::clone(&calls);
    let dispatcher = SyncDispatcher::new().on(move |_: StripeInvoicePaid| {
        let calls = Arc::clone(&calls_clone);
        async move {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    });

    let event = parse_event(&mock_invoice_paid_event("in_test_001", "cus_test_001"));
    let result = dispatcher.dispatch(event).await;
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn dispatch_bubbles_handler_error() {
    let dispatcher = SyncDispatcher::new().on(|_: StripeInvoicePaid| async {
        Err::<(), Error>(Error::Stripe("boom".into()))
    });

    let event = parse_event(&mock_invoice_paid_event("in_test_002", "cus_test_002"));
    let result = dispatcher.dispatch(event).await;
    match result {
        Err(Error::Stripe(msg)) => assert_eq!(msg, "boom"),
        other => panic!("expected Err(Error::Stripe(\"boom\")), got {other:?}"),
    }
}

#[tokio::test]
async fn dispatch_unknown_event_returns_ok_and_handler_is_not_invoked() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = Arc::clone(&calls);
    let dispatcher = SyncDispatcher::new().on(move |_: StripeInvoicePaid| {
        let calls = Arc::clone(&calls_clone);
        async move {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    });

    // Dispatch a checkout.session.completed event — no handler registered for it.
    let event = parse_event(&mock_checkout_completed_event("cs_test_003", "cus_test_003"));
    let result = dispatcher.dispatch(event).await;
    assert!(result.is_ok(), "unknown event should return Ok, got {result:?}");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "handler must NOT be invoked for non-matching event"
    );
}

#[tokio::test]
async fn dispatch_only_invokes_matching_handler_when_multiple_registered() {
    let invoice_calls = Arc::new(AtomicUsize::new(0));
    let checkout_calls = Arc::new(AtomicUsize::new(0));
    let ic = Arc::clone(&invoice_calls);
    let cc = Arc::clone(&checkout_calls);
    let dispatcher = SyncDispatcher::new()
        .on(move |_: StripeInvoicePaid| {
            let ic = Arc::clone(&ic);
            async move {
                ic.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .on(move |_: StripeCheckoutCompleted| {
            let cc = Arc::clone(&cc);
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });

    let event = parse_event(&mock_checkout_completed_event("cs_multi", "cus_multi"));
    dispatcher.dispatch(event).await.expect("ok");
    assert_eq!(invoice_calls.load(Ordering::SeqCst), 0);
    assert_eq!(checkout_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn dispatcher_is_thread_safe_across_arc() {
    let dispatcher = Arc::new(
        SyncDispatcher::new()
            .on(|_: StripeInvoicePaid| async { Ok(()) })
            .on(|_: StripeCheckoutCompleted| async { Ok(()) }),
    );

    let d1 = Arc::clone(&dispatcher);
    let d2 = Arc::clone(&dispatcher);

    let t1 = tokio::spawn(async move {
        let event = parse_event(&mock_invoice_paid_event("in_par_1", "cus_par_1"));
        d1.dispatch(event).await
    });
    let t2 = tokio::spawn(async move {
        let event = parse_event(&mock_checkout_completed_event("cs_par_2", "cus_par_2"));
        d2.dispatch(event).await
    });

    let (r1, r2) = tokio::join!(t1, t2);
    assert!(r1.expect("task 1 join").is_ok());
    assert!(r2.expect("task 2 join").is_ok());
}
