# Phase 235: webhook SyncDispatcher integration + auto-refund fallback - Context

**Gathered:** 2026-06-17
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults auto-selected; review decisions below)

<domain>
## Phase Boundary

Wire `ferro-payments` into the live Stripe webhook flow:

- `wire_dispatcher(dispatcher, service)` — registers three typed-event handlers on the
  caller's `ferro_stripe::SyncDispatcher`: `StripeCheckoutCompleted`,
  `StripeCheckoutExpired`, `StripeChargeRefunded`.
- `PaymentService::handle_session_completed` / `handle_session_expired` /
  `handle_charge_refunded` — each idempotency-guarded via `ProcessedEventLog`, each
  dispatching transactionally to the loaded `Billable`'s `on_paid` / `on_released` /
  `on_refunded`.
- **Auto-refund fallback** for the cases where the money was captured but we cannot
  honor it: loader returns `Err`, loader returns `Ok(None)` (billable vanished), or the
  intent is already in a terminal/side state (paid-after-released).
- Race-condition tests: webhook + reaper interleaved, webhook replay, loader-not-found.

**Out of scope (later phases):**
- `ReleaseExpiredPaymentIntents` / `ReconcileRefundsInFlight` reapers, workspace test
  bin, publish `0.1.0` — **phase 236**.

</domain>

<decisions>
## Implementation Decisions

### PaymentService gains the webhook fields (completes the 234 deferral)
- **D-01:** Add `processed_log: Arc<dyn ferro_stripe::ProcessedEventLog>` to
  `PaymentService` (deferred from 234 per 234-D-09) and reshape `PaymentService::new(...)`
  to take it. The `loader` field loses its 234 `#[allow(dead_code)]` — it is now read by
  the `handle_*` methods. (Crate unpublished — `new()` signature change is free.)
- **D-02:** `StripeGateway` (the 234 seam) gains the methods the auto-refund path needs
  (see D-08). The 234 `MockStripeGateway` is extended in lockstep so the new handlers
  remain unit-testable offline.

### `wire_dispatcher` shape
- **D-03:** `pub fn wire_dispatcher<L: BillableLoader + 'static>(dispatcher: SyncDispatcher,
  service: Arc<PaymentService<L>>) -> SyncDispatcher`. Consuming builder (because
  `SyncDispatcher::on` consumes `self`). Registers exactly three handlers via
  `.on::<StripeCheckoutCompleted,_,_>(...)`, `.on::<StripeCheckoutExpired,_,_>(...)`,
  `.on::<StripeChargeRefunded,_,_>(...)`. Each closure clones `Arc<service>` and awaits
  the matching `handle_*`.
- **D-04 (error-boundary bridge):** `SyncDispatcher::on` handlers must return
  `Result<(), ferro_stripe::Error>`, but `handle_*` return `Result<(), PaymentError>`.
  The orphan rule forbids `impl From<PaymentError> for ferro_stripe::Error` in either
  crate, so the bridge is done **inline in the `wire_dispatcher` closures** with a private
  helper. Mapping policy:
  - **Transient** errors (`PaymentError::Db`, `PaymentError::Stripe`) → return
    `Err(ferro_stripe::Error::Stripe(msg))` so the HTTP layer returns non-2xx and Stripe
    **retries** the webhook (the guarded updates make retries idempotent).
  - **Terminal** outcomes (idempotent replay skip, auto-refund triggered, loader
    vanished after refund) → log via `tracing` and return `Ok(())` so Stripe does **not**
    retry. `AutoRefundTriggered` is logged, not propagated to the dispatcher.

### Idempotency + transactional dispatch
- **D-05:** Each `handle_*` calls `processed_log.try_mark_processed(event.event_id())`
  **first**; on `Ok(false)` (replay) return `Ok(())` immediately (fast path). True
  idempotency does not rely on this alone — the `mark_*` guarded updates are idempotent by
  construction (235 builds on the 234/233 `GuardedUpdate` no-op semantics). Consumer
  `on_*` side-effect idempotency is the consumer's responsibility (document it).
- **D-06 (handle_session_completed):** lookup intent by `session_id`
  (`find_by_stripe_session`) → `mark_paid` (guarded `reserved→paid`):
  - `Ok(false)` (not reserved — e.g. reaper already released it) → **side-state conflict**
    → auto-refund (D-08, reason `SideStateConflict`) → `Ok(())`.
  - `Ok(true)` → open a DB transaction → `loader.load(kind, id)`:
    - `Err(_)` → auto-refund (reason `LoaderError`), roll back, `Ok(())`.
    - `Ok(None)` → auto-refund (reason `BillableVanished`), roll back, `Ok(())`.
    - `Ok(Some(billable))` → `billable.on_paid(&txn)` → commit. Also persist
      `payment_intent_id` (now known from the event) onto the row.
- **D-07 (handle_session_expired / handle_charge_refunded):**
  - expired: lookup by `session_id` → `mark_released` (guarded `reserved→released`) → if
    `true`, txn `billable.on_released(&txn)` → commit. `Ok(false)` → no-op `Ok(())`.
  - charge_refunded: lookup by `payment_intent_id` (fallback `charge_id`) →
    `mark_refunded` (guarded `paid→refunded`) → if `true`, txn
    `billable.on_refunded(&txn, amount_refunded_cents)` → commit. This is the event that
    **resolves** the refund-in-flight predicate (sets `refunded_at`). `Ok(false)` → no-op.

### Auto-refund fallback (resolves the charge_id-absent gap — DISCREPANCY-1)
- **D-08:** **`StripeCheckoutCompleted` carries NO `charge_id`** (only `session_id`,
  `payment_intent_id: Option<String>`, `amount_total_cents`). The existing
  `ferro_stripe::refund::create` takes a `charge_id`, so the session.completed auto-refund
  path cannot use it. Resolution: **refund by payment_intent**. Add a ferro-stripe
  primitive `refund::create_for_payment_intent(payment_intent_id, amount_cents, key, reason)`
  (async-stripe's `CreateRefund` supports a `payment_intent` field — research MUST confirm
  the exact field/version), and add a matching `StripeGateway::create_refund_for_payment_intent`
  method (mock + prod). Per the "no direct `stripe::` import in consumers" gate (V-95-01)
  and the spec rule "new Stripe primitive → ferro-stripe first", the primitive lives in
  ferro-stripe. The auto-refund uses `payment_intent_id` (always present on
  session.completed when `payment_status=paid`); if `payment_intent_id` is `None`, log and
  return `Ok(())` (nothing to refund — free/setup-mode session). The 236 publish bumps
  ferro-stripe + ferro-payments together, so the ferro-stripe addition is absorbed there.
- **D-09:** Auto-refund snapshots `refund_amount_cents` (full `amount_total_cents`) via the
  same `GuardedUpdate WHERE refund_amount_cents IS NULL` dedup as 234, then calls the
  payment-intent refund. After triggering, `tracing::warn!` with the `AutoRefundReason`
  and return `Ok(())` (do not make Stripe retry).

### Carry-forward from 234 code review (`234-REVIEW.md`)
- **D-10 (WR-04 — BLOCKS the loader, fix FIRST):** `BillableKind(&'static str)` cannot be
  built from the runtime `String` the webhook reads from `payment_intents.billable_kind`.
  Change `BillableKind` to hold `Cow<'static, str>`: keep `pub const fn new(s: &'static str)
  -> Self { Self(Cow::Borrowed(s)) }`, add `pub fn from_string(s: String) -> Self
  { Self(Cow::Owned(s)) }`, `as_str()` unchanged. `handle_*` build the kind via
  `BillableKind::from_string(intent.billable_kind.clone())` before `loader.load`.
- **D-11 (WR-01 — stuck refund, resolved by design + documented):** When a refund's Stripe
  call fails *after* the `refund_amount_cents` snapshot, the row stays "refund-in-flight"
  (`status='paid' AND refund_amount_cents IS NOT NULL AND refunded_at IS NULL`). Do **NOT**
  compensate-reset the snapshot — async-stripe 0.41 does not forward idempotency keys, so a
  reset+retry risks a double refund on a lost-response. Instead this stuck state is
  **recovered by the phase-236 `ReconcileRefundsInFlight` reaper** (polls Stripe — an
  idempotent query — for in-flight refunds and reconciles). Document this recovery path in
  the refund code and in `docs/src/features`. `handle_charge_refunded` (D-07) is the happy-path
  resolver. This is the spec's intended design (236 exists for exactly this).
- **D-12 (WR-03):** Add the `amount_cents <= 0` early-return guard to `start_checkout`
  (cheap, prevents an orphan reserved row on a Stripe 400) while touching the service in
  this phase. (WR-02 is doc-only — note the global application-fee rate; no code change.)

### Claude's Discretion
- Whether `handle_*` live in `service.rs` or a new `webhook.rs` module (spec proposes
  `webhook.rs`); the private `PaymentError→ferro_stripe::Error` bridge helper location.
- Exact `AutoRefundReason` selection per branch (already enumerated in 234).
- Whether to expose an auto-refund observability hook to consumers (default: `tracing`
  only, no new consumer callback in 235 — keep scope).
- Test module organization for the race tests (one `#[cfg(test)]` per concern).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Authoritative spec
- `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md` §"PaymentService"
  (`handle_*`, `wire_dispatcher`), §"Webhook race semantics" (the 5-row race table — the
  source of the auto-refund cases), §"Integration with ferro-stripe", §"Error model"
  (`AutoRefundTriggered`/`AutoRefundReason`). **Source of truth — PAY-POLY-WH-01..06 are
  defined here, not in a REQUIREMENTS.md file.**

### Prior phase (what 235 builds on)
- `.planning/phases/234-ferro-payments-billable-trait-loader-and-payment-service-cor/234-CONTEXT.md`
  — locked 234 decisions (StripeGateway seam D-01/02/03, refund-in-flight predicate
  D-15/16/17, PaymentService fields D-09/10, error model D-18).
- `.planning/phases/234-ferro-payments-billable-trait-loader-and-payment-service-cor/234-REVIEW.md`
  — WR-01/WR-03/WR-04 carry-forwards (D-10/11/12 above).
- `ferro-payments/src/service.rs` — `PaymentService`, `StripeGateway`, `MockStripeGateway`,
  `start_checkout`, `request_refund` (the file 235 extends with `handle_*` + new gateway
  method).
- `ferro-payments/src/billable.rs` (`Billable::on_*`), `ferro-payments/src/loader.rs`
  (`BillableLoader::load`), `ferro-payments/src/lib.rs` (`BillableKind` — D-10 changes it),
  `ferro-payments/src/intent/lifecycle.rs` (`find_by_stripe_session`, `mark_*`,
  `find_active_for`).

### ferro-stripe surface to reuse (verify signatures — spec text is approximate)
- `ferro-stripe/src/webhook/sync.rs` — `SyncDispatcher::on<E,H,Fut>` (consuming builder;
  handler `Fn(E) -> Future<Output=Result<(), ferro_stripe::Error>>`; first error
  short-circuits `dispatch`).
- `ferro-stripe/src/webhook/events.rs` — typed-event field shapes (VERIFIED):
  - `StripeCheckoutCompleted { event_id, session_id, payment_intent_id: Option<String>,
    amount_total_cents, currency, metadata, customer_email }` — **NO charge_id** (D-08).
  - `StripeCheckoutExpired { event_id, session_id, metadata }`.
  - `StripeChargeRefunded { event_id, charge_id, payment_intent_id: Option<String>,
    refund_id: Option<String>, amount_refunded_cents, metadata }`.
  - `StripeEvent` trait + `WebhookEvent`.
- `ferro-stripe/src/idempotency.rs` — `ProcessedEventLog::try_mark_processed(id)
  -> Result<bool, Error>` (true=first, false=replay) + `MemoryProcessedLog` (test impl).
- `ferro-stripe/src/refund.rs` — `refund::create(charge_id, amount, key, reason)`
  (charge-based, used by `request_refund`); **D-08 adds a payment_intent-based variant
  here**. Note the async-stripe 0.41 idempotency-key-not-forwarded caveat.

### Workspace conventions
- `CLAUDE.md` — pre-commit gate (`fmt` + `clippy --all --all-targets -D warnings` +
  `test --all-features`); V-95-01 "no direct `stripe::` import in consumers" (drives D-08
  going to ferro-stripe); project-agnostic crate rule.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The 234 `StripeGateway` + `MockStripeGateway` — extend with the auto-refund method; the
  mock's call-recording makes "auto-refund called exactly once with the right amount"
  unit-testable offline.
- 234/233 `mark_paid`/`mark_released`/`mark_refunded` guarded updates — the `bool` return
  is the side-state-conflict signal (D-06), no extra read needed.
- `find_by_stripe_session` (233) for session lookups; add/confirm a `find_by_payment_intent`
  helper for `handle_charge_refunded` (lifecycle.rs) if absent.
- `MemoryProcessedLog` (ferro-stripe) — the test `ProcessedEventLog`.
- `ferro_stripe::testing::*` — signed/typed webhook event JSON builders for tests.

### Established Patterns
- `#[async_trait]` traits; `GuardedUpdate` no-op idempotency; one `thiserror` enum;
  `#[cfg(test)]` in-memory SQLite harness (233/234 template).

### Integration Points
- `ferro-payments/src/lib.rs` re-exports (`wire_dispatcher`, changed `BillableKind`).
- `ferro-stripe/src/refund.rs` + `ferro-stripe/src/lib.rs` (new payment-intent refund fn —
  D-08; ferro-stripe republished alongside ferro-payments in 236).

### Constraints / Net-New Risk
- **DISCREPANCY-1 (D-08):** auto-refund at session.completed has no charge_id → needs the
  ferro-stripe payment-intent refund primitive. Highest-risk item; research must confirm
  async-stripe `CreateRefund.payment_intent` support before the planner commits.
- Error-boundary mapping (D-04) is subtle: wrong transient/terminal classification either
  loses events (false-Ok) or hammers retries (false-Err). Race tests must cover both.

</code_context>

<specifics>
## Specific Ideas

- Implement the spec "Testing" rows that belong to 235: webhook-race (reaper + handler
  interleaved against the guarded updates), webhook replay (`ProcessedEventLog` →
  second dispatch is a no-op), loader-not-found (→ auto-refund called once), and the
  paid-after-released side-state conflict (→ auto-refund). All offline via
  `MockStripeGateway` + `MemoryProcessedLog` + in-memory SQLite.
- A consumer wires it in one call:
  `let dispatcher = ferro_payments::wire_dispatcher(SyncDispatcher::new(), service.clone());`

</specifics>

<deferred>
## Deferred Ideas

- `ReleaseExpiredPaymentIntents` reaper, `ReconcileRefundsInFlight` reaper (the D-11
  recovery mechanism), workspace example-`Billable` test bin, real ferro-stripe-test-mode
  integration test, version bump + publish ferro core + ferro-stripe + ferro-payments
  `0.1.0` — **phase 236**.
- WR-02 (per-connected-account fee rates) — doc-only note in 235; a real per-account fee
  API would be a future ferro-stripe change, not now.
- Consumer-facing auto-refund observability hook (beyond `tracing`) — only if a consumer
  asks.

</deferred>

---

*Phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-*
*Context gathered: 2026-06-17*
