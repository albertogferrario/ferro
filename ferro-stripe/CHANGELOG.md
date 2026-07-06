# Changelog

All notable changes to `ferro-stripe` are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.9.0] - 2026-06-10

### Added

- `StripeConfig::application_fee_for(amount_cents) -> Option<i64>` — computes the
  platform application fee for a Connect destination charge from
  `application_fee_percent`. Returns `None` when the percentage is unset or
  non-positive; otherwise `round(amount_cents * percent / 100)`, clamped to
  `[0, amount_cents]`. Feed the result directly into
  `CheckoutBuilder::destination(account_id, fee)`.
- ferro-mcp `stripe_config_status` reports two new Connect fields:
  `connect_webhook_secret_present` (a boolean — the secret value is never
  returned) and `application_fee_percent` (the parsed number, or null when
  unset), bringing the tool to parity with the config struct.

### Docs

- `docs/src/features/stripe.md` gains a "Connect destination charges with a
  platform fee" section: account create → onboarding link → `account.updated`
  capability persistence → `CheckoutBuilder::destination` fed by
  `application_fee_for`, with a note on the correspondence with the
  manual-capture flow.

### Notes

- Additive and non-breaking: no existing signatures changed; `from_env` is
  unchanged.

## [0.8.0] - 2026-06-10

### Changed

- `verify_webhook` no longer delegates to async-stripe's versioned event
  structs. It verifies the HMAC signature and carries `data.object` as untyped
  JSON in a ferro-owned `WebhookEvent`; each `StripeEvent::from_raw` reads JSON
  fields directly. This makes the webhook path forward-compatible across Stripe
  API versions (events rendered at a newer API version previously failed
  deserialization and were rejected as "invalid signature"). Typed-event public
  fields are unchanged.

## [0.7.0] - 2026-06-09

### Added

- `StripeChargeRefunded::refund_id: Option<String>` — the refund identifier
  from a `charge.refunded` event, parsed from the charge's refunds list
  (`charge.refunds.data[0].id`). Lets a consumer look up a local refund row
  without importing `stripe::` types directly. `None` when the event carries
  no refund.
- `CheckoutBuilder::manual_capture()` — authorize a payment at checkout for
  later capture (`Mode::Payment` only; returns
  `Error::ManualCaptureRequiresPaymentMode` for `Mode::Subscription`).
- `ferro_stripe::payment_intent` module — `capture`, `cancel`, and `retrieve`
  operations for manually captured PaymentIntents.
- Typed webhook events `StripePaymentIntentAmountCapturableUpdated`
  (`payment_intent.amount_capturable_updated`) and `StripePaymentIntentCanceled`
  (`payment_intent.canceled`) for the manual-capture lifecycle.

### Notes

- No 0.6.x release: the Phase 189 manual-capture additions (built but not
  previously published) and the `refund_id` field are released together as a
  single 0.7.0 label rather than as separate intermediate versions.
