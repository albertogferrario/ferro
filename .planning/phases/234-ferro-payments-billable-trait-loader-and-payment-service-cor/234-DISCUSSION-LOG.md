# Phase 234: Billable trait + Loader + PaymentService core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-17
**Phase:** 234-ferro-payments-billable-trait-loader-and-payment-service-cor
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** Stripe injection seam, Billable trait, BillableLoader trait, PaymentService fields, start_checkout flow, request_refund flow, PaymentError set, manifest/publish wiring

---

## Stripe injection seam (DISCREPANCY-1)

| Option | Description | Selected |
|--------|-------------|----------|
| `StripeGateway` trait local to ferro-payments | Mockable seam; prod impl wraps free functions; `Arc<dyn StripeGateway>` in service | ✓ |
| Hold `Arc<ferro_stripe::Client>` per spec text | Not implementable — no such type; Stripe is a global static facade | |
| Add injectable client to ferro-stripe | Heavier cross-crate change; no new Stripe primitive needed | |
| Skip unit tests, integration-only (236) | Contradicts the phase goal "unit tests use mocked Stripe" | |

**Selection:** StripeGateway trait seam (D-01/02/03).
**Notes:** Spec's `Arc<ferro_stripe::Client>` flagged as a literal impossibility — `ferro_stripe::Stripe` is `OnceLock`-backed and checkout/refund are free functions. The seam is the load-bearing decision that makes the orchestrator unit-testable.

## Billable trait

| Decision | Choice | Selected |
|----------|--------|----------|
| Async mechanism | `#[async_trait]`, `on_*` take `&DatabaseTransaction` | ✓ |
| Connect account | Add defaulted `connect_account_id() -> Option<String> { None }` (closes spec gap) | ✓ |
| `Clone` bound | Not Clone; pass `&dyn Billable` (spec open-Q4) | ✓ |

**Notes:** Spec's `Billable` had no Connect accessor yet `start_checkout` snapshots `application_fee_cents` — defaulted method resolves it without burdening non-Connect billables.

## BillableLoader trait

| Decision | Choice | Selected |
|----------|--------|----------|
| Signature | `load(kind, id) -> Result<Option<Box<dyn Billable>>>` | ✓ |
| tenant_id arg | Loader-extracts (no separate arg) — spec open-Q1 | ✓ |

## PaymentService fields & constructor

| Option | Description | Selected |
|--------|-------------|----------|
| Store only 234-used fields | `db`, `stripe`, `loader (#[allow(dead_code)])`, `return_url_builder`; add `processed_log` in 235 | ✓ |
| Store full spec field set now | `processed_log` unused in 234 → `dead_code` → clippy `-D warnings` fail | |

**Notes:** `new()` reshaping across 234→235 accepted (unpublished crate). `loader` field/generic kept per phase goal but `#[allow(dead_code)]` to pass the gate (D-10).

## start_checkout flow

**Selected:** reserved-row-first → build session (line item + return urls + optional destination/fee) → deterministic idempotency key → `create()` → attach `stripe_session_id` + `application_fee_cents` → return `CheckoutUrl` (D-12). `expires_at = now + ttl` (consumer window, distinct from Stripe session expiry, D-12). `payment_intent_id`/`charge_id` stay NULL (D-13). Stripe failure leaves the reserved row for the 236 reaper (D-14).

## request_refund flow (DISCREPANCY-2)

**Selected:** load by id → require `paid` + `charge_id` → snapshot `refund_amount_cents` via `GuardedUpdate WHERE refund_amount_cents IS NULL` (app-layer dedup; async-stripe 0.41 ignores idempotency keys) → call Stripe refund. Does NOT flip to `refunded` (235 webhook does). "refund_requested" = predicate, not enum variant (D-15/16/17).

**Notes:** ROADMAP 236 "refund_requested state" flagged — resolved as a query predicate against the locked 5-variant enum; no migration churn.

## PaymentError set

**Selected:** extend with `Stripe(#[from])`, `Loader(Box<dyn Error + Send + Sync>)`, `AutoRefundTriggered { reason: AutoRefundReason }` (D-18). `AutoRefundTriggered` defined now, returned in 235.

## Manifest / publish wiring (DISCREPANCY-3)

**Selected:** add `ferro-stripe` path dep (~0.9); move `ferro-payments` from publish.yml Wave 1b to a new Wave 1c (intra-wave dep 1b→1b otherwise unordered) (D-19/20/21).

## Claude's Discretion

- Module split (`refund.rs` vs folding into `service.rs`); `StripeGateway` method/`CheckoutRequest` shapes; mock placement (`#[cfg(test)]` vs feature); `AutoRefundReason` names; idempotency-key string format.

## Deferred Ideas

- wire_dispatcher + typed webhook handlers + idempotency + auto-refund fallback dispatch → phase 235.
- reapers + workspace test bin + integration test + publish 0.1.0 → phase 236.
- Provider abstraction beyond Stripe → spec non-goal.
