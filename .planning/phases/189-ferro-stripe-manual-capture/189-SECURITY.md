---
phase: 189
slug: ferro-stripe-manual-capture
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-07
---

# Phase 189 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| consumer app → CheckoutBuilder | consumer supplies Mode, account_id, fee_cents | Stripe API params (capture_method, transfer_data) |
| consumer app → payment_intent functions | consumer supplies payment_intent_id (string), amount_cents (i64) | Stripe PaymentIntent API calls |
| Stripe → webhook endpoint → events parser | inbound webhook JSON (untrusted until signature-verified) | typed event structs carrying payment intent state |
| documentation → consumer developer | docs code examples | payment flow wiring; incorrect docs propagate to consumer payment code |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-189-01 | Tampering | CheckoutBuilder::create — manual_capture + Subscription | mitigate | Pre-flight guard at `checkout.rs:187–189` returns `Err(Error::ManualCaptureRequiresPaymentMode)` before `Stripe::client()`. Guard ordering: `is_none()` check → mode guard → client. Verified by `checkout_create_manual_capture_subscription_returns_err` test. | closed |
| T-189-02 | Tampering | merged payment_intent_data construction | mitigate | Single-assignment merge via `build_payment_intent_data()` at `checkout.rs:150–170`; `params.payment_intent_data = self.build_payment_intent_data()` appears exactly once (line 249). D-08 composition test `checkout_create_manual_capture_with_destination_sets_both_fields` asserts `capture_method=Manual` AND `transfer_data.is_some()` coexist. | closed |
| T-189-03 | Information Disclosure | Error message text | accept | Error messages contain no secrets — only public method names and usage hints. See Accepted Risks Log. | closed |
| T-189-04 | Tampering | capture amount_cents (negative or oversized) | mitigate | `payment_intent.rs:31–39` — `n <= 0` early return fires before `u64::try_from`, rejecting zero and negatives with `Err(Error::Stripe("amount_to_capture must be positive"))`. Post-review fix (commit a37094f5) added the `n <= 0` guard strengthening the original `u64::try_from`-only path. Stripe bounds the upper side. Tested by `capture_rejects_negative_amount` and `capture_rejects_zero_amount`. | closed |
| T-189-05 | Tampering | payment_intent_id (malformed) | mitigate | `PaymentIntentId::parse()` runs before `Stripe::client()` in all three functions (`payment_intent.rs:26–28`, `52–54`, `63–65`). Malformed ids return structured `Err(Error::Stripe("invalid payment intent id: ..."))`. Tested by three invalid-id unit tests. | closed |
| T-189-06 | Repudiation / Denial of Service | double-capture on retry | accept (documented) | async-stripe 0.41 does not forward per-request idempotency keys to `PaymentIntent::capture`. Application-layer dedup required. Documented in module doc (`payment_intent.rs:18–21`) and in `docs/src/features/stripe.md:287`. See Accepted Risks Log. | closed |
| T-189-07 | Elevation of Privilege | connected-account impersonation | mitigate | `capture` and `cancel` are platform-scoped only — no `Stripe-Account` header parameter exposed in `payment_intent.rs`. Confirmed: neither `capture` nor `cancel` accepts a connected-account argument. | closed |
| T-189-08 | Spoofing | inbound webhook for 2 new events | mitigate (existing) | New events flow through the existing `verify_webhook` path at `webhook/verify.rs:16–23` (HMAC-SHA256, Stripe's 5-minute timestamp window). `from_raw` operates on an already-deserialized `stripe::Event` obtained only after successful verification. No new ingress path introduced. | closed |
| T-189-09 | Tampering | event type confusion | mitigate | `StripePaymentIntentAmountCapturableUpdated::from_raw` guards on `EventType::PaymentIntentAmountCapturableUpdated` (`events.rs:246`); `StripePaymentIntentCanceled::from_raw` guards on `EventType::PaymentIntentCanceled` (`events.rs:277`). Cross-reject tests `payment_intent_amount_capturable_updated_rejects_canceled_event` and `payment_intent_canceled_rejects_amount_capturable_event` prove mutual rejection. | closed |
| T-189-10 | Tampering | fixture type-string drift (EventType::Other) | mitigate | Fixtures use verified serde-rename strings: `"payment_intent.amount_capturable_updated"` and `"payment_intent.canceled"` (confirmed by direct grep). Parse-all-fields tests in `parser_contract.rs` fail loudly if either fixture deserializes as `EventType::Other`. 19 parser-contract tests pass. | closed |
| T-189-11 | Information Disclosure | docs code examples | mitigate | All examples in `docs/src/features/stripe.md` use placeholder ids (`acct_xxx`, `"booking-deposit-42"`) and `app.example.com` URLs. No real keys, accounts, or tenant identifiers present (`docs/src/features/stripe.md:186,220,254–256`). | closed |
| T-189-12 | Tampering (indirect) | idempotency caveat omission | mitigate | `docs/src/features/stripe.md:287` explicitly states application-layer deduplication is required and recommends a DB unique constraint. Reinforces the module-level caveat in `payment_intent.rs:18–21`. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-189-01 | T-189-03 | `Error::ManualCaptureRequiresPaymentMode` message text `"manual capture requires payment mode; use Mode::Payment with manual_capture()"` contains only a public method name and a usage hint. No secrets, no PII. Safe to surface to consumer logs. | gsd-secure-phase | 2026-06-07 |
| AR-189-02 | T-189-06 | async-stripe 0.41 does not expose a per-request idempotency key API for `PaymentIntent::capture`. No framework-layer fix is available without an async-stripe upgrade. Application-layer deduplication (DB unique constraint) is documented as required in both the module doc and the public docs. The risk is bounded to duplicate charges on network-retry scenarios and is a known limitation of the library version in use. | gsd-secure-phase | 2026-06-07 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-07 | 12 | 12 | 0 | gsd-secure-phase (claude-sonnet-4-6) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-07
