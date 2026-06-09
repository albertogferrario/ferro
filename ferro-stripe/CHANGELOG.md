# Changelog

All notable changes to `ferro-stripe` are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

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
