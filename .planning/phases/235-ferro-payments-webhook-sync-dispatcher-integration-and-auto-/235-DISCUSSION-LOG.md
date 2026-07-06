# Phase 235: webhook SyncDispatcher integration + auto-refund fallback - Discussion Log

> **Audit trail only.** Not consumed by downstream agents. Decisions live in CONTEXT.md.

**Date:** 2026-06-17
**Phase:** 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-
**Mode:** `--auto` (recommended defaults; no interactive prompts)
**Areas discussed:** PaymentService webhook fields, wire_dispatcher shape, error-boundary bridge, idempotency/transactional dispatch, auto-refund fallback (charge_id gap), 234 carry-forwards

---

## PaymentService webhook fields
**Selected:** add `processed_log: Arc<dyn ProcessedEventLog>` (234 deferral), reshape `new()`, loader field loses `#[allow(dead_code)]` (D-01/02).

## wire_dispatcher shape
**Selected:** `fn wire_dispatcher<L>(SyncDispatcher, Arc<PaymentService<L>>) -> SyncDispatcher` consuming builder, 3 `.on::<E,_,_>` registrations (D-03).

## Error-boundary bridge (PaymentError → ferro_stripe::Error)
| Option | Selected |
|--------|----------|
| Inline map in wire_dispatcher closures (orphan rule blocks a From impl); transient→Err (Stripe retries), terminal→log+Ok | ✓ |
| `impl From<PaymentError> for ferro_stripe::Error` | rejected (orphan rule) |

**Selected:** D-04 inline bridge with transient/terminal policy.

## Idempotency + transactional dispatch
**Selected:** `try_mark_processed(event_id)` fast-path first; guarded `mark_*` provide true idempotency; per-event handler flows D-05/06/07 (mark→load→on_*→commit; bool return = side-state signal).

## Auto-refund fallback (charge_id gap — DISCREPANCY-1)
| Option | Description | Selected |
|--------|-------------|----------|
| Refund by payment_intent (new ferro-stripe `refund::create_for_payment_intent` + gateway method) | session.completed has no charge_id, only payment_intent_id | ✓ |
| Refund by charge_id | impossible — StripeCheckoutCompleted has no charge_id | |
| Retrieve charge via payment_intent then refund-by-charge | extra Stripe round-trip + still needs a ferro-stripe addition | |

**Selected:** D-08 refund-by-payment_intent (ferro-stripe primitive, absorbed in 236 publish). Snapshot via `refund_amount_cents IS NULL` dedup, log reason, return Ok (D-09).

## 234 carry-forwards
- **WR-04 (D-10):** `BillableKind` → `Cow<'static,str>` with `const new(&'static str)` + `from_string(String)`. Blocks the webhook loader → fix first. ✓
- **WR-01 (D-11):** stuck refund recovered by the 236 `ReconcileRefundsInFlight` reaper (NOT compensate-reset — would risk double-refund without Stripe idempotency). Documented. ✓
- **WR-03 (D-12):** add `amount_cents <= 0` guard to start_checkout. ✓
- **WR-02:** doc-only (global application-fee rate). No code change.

## Claude's Discretion
- `handle_*` in service.rs vs new webhook.rs; bridge helper location; AutoRefundReason per branch; auto-refund observability hook (default tracing-only); race-test module layout.

## Deferred
- Reapers + test bin + publish 0.1.0 → 236.
