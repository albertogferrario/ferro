# Phase 236: ferro-payments reapers + publish 0.1.0 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-17
**Phase:** 236-ferro-payments-reapers-and-publish-0-1-0
**Mode:** `--auto` (all gray areas auto-selected; recommended option chosen per area)
**Areas discussed:** Reaper placement & job shape, Clock injection, release_expired txn
granularity, ReconcileRefundsInFlight Stripe-poll path, workspace test-bin form,
ferro-queue dependency, publish/version/git-reconcile, docs/MCP scope.

---

## Reaper code placement + job-struct shape

| Option | Description | Selected |
|--------|-------------|----------|
| Logic on `PaymentService` methods; thin Job wrappers (serde-skip Arc injection) | Testable offline with existing mocks; matches `ProcessStripeWebhook` | ✓ |
| Logic inside the Job `handle()` directly | Fewer types, but jobs need DB/Stripe context and aren't serializable cleanly | |
| Separate non-queue reaper runner abstraction | New surface, no precedent in workspace | |

**Choice:** `PaymentService::release_expired` + `reconcile_refunds_in_flight`; generic
Job structs in `reaper.rs` wrapping them via `#[serde(skip)] Arc<service>`. (D-01/02/03)

---

## Clock injection

| Option | Description | Selected |
|--------|-------------|----------|
| Optional `now` cutoff param on internal query path | Minimal surface, deterministic tests | ✓ |
| `Clock` trait field on PaymentService | Cleaner but heavier for two call sites (left to discretion) | |
| `Utc::now()` inline, no injection | Untestable reaper timing | |

**Choice:** cutoff param; private Clock field allowed at planner discretion. (D-04)

---

## release_expired transaction granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Per-intent transaction, continue-on-error | One row's failure doesn't roll back the batch; matches webhook semantics | ✓ |
| Single batch transaction | One failure rolls back all released intents | |

**Choice:** per-intent txn; guarded `mark_released` no-op handles the webhook race;
loader-vanished is benign (no money captured, no auto-refund). (D-05/06)

---

## ReconcileRefundsInFlight Stripe poll + resolution

| Option | Description | Selected |
|--------|-------------|----------|
| Poll Stripe (idempotent read) via new StripeGateway method; resolve via mark_refunded path | Safe; avoids double-refund hazard 235 D-11 names | ✓ |
| Reset snapshot + re-issue refund | Double-refund risk (async-stripe 0.41 no idempotency-key forwarding) | |

**Choice:** read-only poll primitive in ferro-stripe + gateway/mock method; succeeded →
resolve like `handle_charge_refunded`; pending → wait; failed → warn (no auto-retry).
Cadence 1h, cron-configurable by consumer. (D-07/08/09)

---

## Workspace "test bin" form

| Option | Description | Selected |
|--------|-------------|----------|
| `#[ignore]`-gated integration test in `tests/`, skips on absent secret | Satisfies spec; CI stays green secret-free | ✓ |
| New publishable workspace member bin crate | Pollutes publish set, needs publish.yml wave | |

**Choice:** gated integration test with tiny example Billable; optional `examples/`
mirror. (D-10)

---

## ferro-queue dependency

| Option | Description | Selected |
|--------|-------------|----------|
| Add `ferro-queue` dep (Wave-1 leaf, ordering OK); reaper.rs module | Required for Job structs | ✓ |

**Choice:** add dep, new `reaper.rs`, re-export jobs from lib.rs. (D-11)

---

## Publish + version + git reconciliation

| Option | Description | Selected |
|--------|-------------|----------|
| Rebase divergence first → bump workspace → CI auto-publish; bootstrap new crate locally | Matches token scoping + monotonic-version reality | ✓ |
| Push current local state and bump blindly | Non-monotonic version → CI publish failure | |

**Choice:** `git pull --rebase` first (D-12); bump 0.2.69→0.2.70 baseline, ferro-stripe
republished, ferro-payments 0.1.0 (D-13); push→CI for updates, **local bootstrap for the
new ferro-payments crate** (publish-update-only token) (D-14).

---

## Docs + MCP scope

| Option | Description | Selected |
|--------|-------------|----------|
| Add `docs/src/features/payments.md` + SUMMARY; no MCP change | Payments is a consumer lib, not MCP-introspected | ✓ |

**Choice:** docs page + cross-link from stripe.md; run Docs build pre-push; explicitly no
ferro-mcp work. (D-15/16)

---

## Claude's Discretion

- Clock shape (param vs field); `RefundStatus` enum shape; examples/ mirror;
  reaper.rs internal layout; exact patch version after rebase.

## Deferred Ideas

- gestiscilo Phase 218+ (consumer repo); per-account fee rates; consumer observability
  hook; payments-specific MCP introspection.
