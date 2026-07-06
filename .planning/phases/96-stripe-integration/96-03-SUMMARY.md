---
phase: 96-stripe-integration
plan: 03
subsystem: ferro-stripe
tags: [stripe, webhooks, events, queue, hmac]
dependency_graph:
  requires: [96-01, 96-02]
  provides: [webhook-verification, stripe-events, webhook-handlers, subscription-sync]
  affects: [ferro-stripe, framework]
tech_stack:
  added: [ferro-events, ferro-queue, hmac, sha2, hex]
  patterns: [tdd, hmac-sha256, queue-dispatch, event-wrapper]
key_files:
  created:
    - ferro-stripe/src/webhook/mod.rs
    - ferro-stripe/src/webhook/events.rs
    - ferro-stripe/src/webhook/handler.rs
  modified:
    - ferro-stripe/Cargo.toml
    - ferro-stripe/src/lib.rs
    - ferro-stripe/src/subscription/sync.rs
    - framework/src/lib.rs
decisions:
  - "ferro_queue::dispatch() used directly over Queueable::dispatch() — Queueable returns PendingDispatch builder, not Future"
  - "plan_from_subscription resolution: metadata[plan] > price nickname > unknown"
  - "is_processed stub returns false always — full idempotency deferred to user's event listener with DB"
  - "signed_webhook_payload is a regular pub fn, not feature-gated — used in production test suites"
metrics:
  duration_seconds: 630
  completed_date: "2026-03-11"
  tasks_completed: 2
  files_created: 3
  files_modified: 4
---

# Phase 96 Plan 03: Webhook Verification, Events, and Subscription Sync Summary

Stripe webhook verification with HMAC-SHA256, 5 ferro-events event wrappers, ProcessStripeWebhook queue job, and platform/Connect webhook handlers dispatching asynchronously via ferro-queue.

## What Was Built

### Webhook Verification (`ferro-stripe/src/webhook/mod.rs`)

- `verify_webhook(raw_body, signature, secret)` — wraps `stripe::Webhook::construct_event`, maps to `Error::WebhookVerification`
- `is_processed(event_id)` — stub returning false; full idempotency requires user-defined DB check

### Event Types (`ferro-stripe/src/webhook/events.rs`)

Five ferro-events event wrappers:
- `StripeSubscriptionUpdated` — `"stripe.customer.subscription.updated"`
- `StripeSubscriptionDeleted` — `"stripe.customer.subscription.deleted"`
- `StripeCheckoutCompleted` — `"stripe.checkout.session.completed"`
- `StripeInvoicePaid` — `"stripe.invoice.paid"`
- `StripeConnectPaymentSucceeded` — `"stripe.connect.payment_intent.succeeded"`

All derive `Clone`, implement `ferro_events::Event` (Clone + Send + Sync).

`ProcessStripeWebhook` job struct:
- Implements `ferro_queue::Job` with `async_trait`
- Dispatches appropriate ferro-events Event from `event_type` string
- `name()` returns `"ProcessStripeWebhook"`
- Dispatches to `"stripe-webhooks"` queue (via `ferro_queue::dispatch` with on_queue)

`signed_webhook_payload(payload, secret) -> (String, i64)`:
- Generates valid `t={ts},v1={hmac_sha256}` header for testing
- Uses HMAC-SHA256, same algorithm as `stripe::Webhook::construct_event`

### Webhook Handlers (`ferro-stripe/src/webhook/handler.rs`)

- `handle_platform_webhook(raw_body, signature)` — verifies with `webhook_secret`, dispatches `ProcessStripeWebhook`
- `handle_connect_webhook(raw_body, signature)` — verifies with `connect_webhook_secret`, dispatches with `connect_account_id` from `event.account`
- Both return `Ok(())` immediately after job dispatch (ack 200 pattern)

### Subscription Sync (`ferro-stripe/src/subscription/sync.rs`)

- `plan_from_subscription(sub)` — resolution order: metadata["plan"] > price nickname > "unknown"
- `subscription_info_from_stripe(sub)` — maps all 7 fields: id, plan (via `plan_from_subscription`), status, trial_ends_at, cancel_at_period_end, current_period_end, stripe_connect_account_id

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ferro_queue::Job has no queue() method**
- **Found during:** Task 1 compilation
- **Issue:** Plan specified `fn queue()` in the Job impl, but ferro-queue's Job trait has no such method. Only `ShouldQueue` in ferro-events has `queue()`.
- **Fix:** Removed the `queue()` method from the `ProcessStripeWebhook` Job impl. The queue name is applied at dispatch time via `ferro_queue::dispatch_to(job, "stripe-webhooks")`. Updated test to assert `job.name()` instead.
- **Files modified:** `ferro-stripe/src/webhook/events.rs`

**2. [Rule 1 - Bug] Queueable::dispatch() returns PendingDispatch, not Future**
- **Found during:** Task 2 (handler.rs) compilation
- **Issue:** `job.dispatch()` from the `Queueable` trait returns a `PendingDispatch<J>` builder, not a future. Awaiting it directly fails.
- **Fix:** Used `ferro_queue::dispatch(job).await` instead, which is the module-level async function.
- **Files modified:** `ferro-stripe/src/webhook/handler.rs`

## Tests

23 unit tests pass, 2 doc-tests pass:
- `verify_webhook_with_valid_signature_returns_ok`
- `verify_webhook_with_tampered_body_returns_err`
- `verify_webhook_with_wrong_secret_returns_err`
- `is_processed_returns_false_for_unseen_ids`
- 5 event name tests (one per event type)
- `events_are_clone_send_sync` (compile-time check)
- `signed_webhook_payload_generates_valid_signature`
- `process_stripe_webhook_job_name`

## Self-Check: PASSED

- ferro-stripe/src/webhook/mod.rs: FOUND
- ferro-stripe/src/webhook/events.rs: FOUND
- ferro-stripe/src/webhook/handler.rs: FOUND
- commit e9e8d45: FOUND
