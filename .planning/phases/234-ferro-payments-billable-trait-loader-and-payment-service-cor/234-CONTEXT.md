# Phase 234: Billable trait + Loader + PaymentService core - Context

**Gathered:** 2026-06-17
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults auto-selected; review decisions below)

<domain>
## Phase Boundary

Add the **orchestration layer** to `ferro-payments` on top of the phase-233 data layer:

- The `Billable` trait — domain entities expose amount, currency, line description,
  and per-status side effects (`on_paid` / `on_released` / `on_refunded`) to the
  payment layer without coupling it to any concrete table.
- The `BillableLoader` trait — the consumer registers a polymorphic loader so the
  payment layer can resolve a `(kind, id)` to a `Box<dyn Billable>`.
- `PaymentService<L: BillableLoader>` with **only two methods** in this phase:
  - `start_checkout(billable, ttl)` — mint a Stripe Checkout session via
    `ferro_stripe::CheckoutBuilder`, snapshot `application_fee_cents`, attach the
    session id to a fresh `reserved` `payment_intents` row, return the hosted URL.
  - `request_refund(intent_id, amount_cents)` — call the Stripe refund API and
    snapshot `refund_amount_cents` (status flip to `refunded` is the webhook's job,
    phase 235).
- The **full `PaymentError`** variant set (adds `Stripe`, `Loader`,
  `AutoRefundTriggered` to the 233 minimal set).
- Add the **`ferro-stripe` dependency** to the crate manifest (deferred from 233).
- Unit tests with a **mocked Stripe seam**, **mocked `BillableLoader`**, and the
  existing `MemoryProcessedLog`.

**Out of scope (later phases):**
- `wire_dispatcher` + `handle_session_completed` / `handle_session_expired` /
  `handle_charge_refunded` typed webhook handlers, idempotency via
  `ProcessedEventLog`, auto-refund *fallback* dispatch — **phase 235**.
- `release_expired` reaper, `ReconcileRefundsInFlight`, workspace test bin,
  publish `0.1.0` — **phase 236**.

</domain>

<decisions>
## Implementation Decisions

### Stripe injection seam (the architectural keystone — resolves DISCREPANCY-1)
- **D-01:** The spec's `PaymentService { stripe: Arc<ferro_stripe::Client> }` and
  "unit tests use mocked `ferro_stripe::Client`" are **not literally implementable**:
  ferro-stripe exposes no `Client` type. `ferro_stripe::Stripe` is a global static
  facade (`OnceLock<stripe::Client>`), and `checkout`/`refund` are **free functions**
  that call `Stripe::client()` internally — nothing injectable, nothing mockable.
- **D-02:** Resolve by defining a small **`StripeGateway` trait local to
  ferro-payments** — the seam that makes the orchestrator unit-testable (this is the
  phase's killer property: a *trustworthy, mock-tested* polymorphic payment
  orchestrator). Minimum surface:
  - `async fn create_checkout_session(&self, req: CheckoutRequest) -> Result<ferro_stripe::CheckoutIntent, ferro_stripe::Error>`
  - `async fn create_refund(&self, charge_id: &str, amount_cents: Option<i64>, idempotency_key: &str) -> Result<(), ferro_stripe::Error>`
  - The production impl (`StripeClientGateway` or similar) wraps
    `ferro_stripe::CheckoutBuilder` + `ferro_stripe::refund::create`. A test mock
    records calls and returns canned results.
- **D-03:** `PaymentService` holds `stripe: Arc<dyn StripeGateway>`. `new()` takes the
  gateway as a parameter so production wires the real one and tests inject the mock.
  Do **not** modify `ferro-stripe` to add an injectable client — keep the seam in
  ferro-payments (the consumer of the static facade), per the "ferro-stripe-first only
  when a new Stripe primitive is needed" rule; no new Stripe primitive is needed here.

### `Billable` trait
- **D-04:** `#[async_trait]` (crate already depends on `async-trait`). Sync accessors
  `kind() -> BillableKind`, `id() -> i64`, `tenant_id() -> i64`, `amount_cents() -> i64`,
  `currency() -> &str`, `checkout_line_description() -> String`. Async side effects
  `on_paid` / `on_released` / `on_refunded(.., amount_cents)` all take
  `&sea_orm::DatabaseTransaction` and return `Result<(), PaymentError>`.
- **D-05:** Add a **defaulted** `fn connect_account_id(&self) -> Option<String> { None }`
  to `Billable`. This closes a spec gap: the design snapshots `application_fee_cents`
  for Connect destination charges in `start_checkout`, but the spec's `Billable` trait
  exposed no Connect account. The default `None` keeps non-Connect billables trivial;
  Connect billables override it so `start_checkout` can call
  `CheckoutBuilder::destination(account_id, fee)`.
- **D-06:** `Billable` is **not** `Clone` (spec open-Q4, leaning no). Everything passes
  `&dyn Billable`.

### `BillableLoader` trait
- **D-07:** `#[async_trait] async fn load(&self, kind: BillableKind, id: i64)
  -> Result<Option<Box<dyn Billable>>, PaymentError>`. `Ok(None)` = billable vanished;
  `Err(PaymentError::Loader(..))` = consumer-side failure. Both feed the auto-refund
  fallback **in phase 235**, not here.
- **D-08:** **No separate `tenant_id` argument** (spec open-Q1, leaning loader-extracts).
  Tenant scoping is the loader's concern; the loaded `Billable` exposes `tenant_id()`.

### `PaymentService` fields & constructor
- **D-09:** Store **only fields used in this phase** to stay clippy-clean under
  `-D warnings`: `db: DatabaseConnection`, `stripe: Arc<dyn StripeGateway>`,
  `loader: L`, `return_url_builder: Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>`.
  `processed_log` is **not** stored in 234 (it is only needed by the webhook handlers)
  — it is added to `new()` in phase 235. The `new()` signature changing across 234→235
  is acceptable (crate unpublished, no compat constraint).
- **D-10:** `loader: L` is required by the phase goal (`PaymentService<L: BillableLoader>`)
  but is **not exercised by `start_checkout`/`request_refund`** — both work on a
  `&dyn Billable` the caller already holds, or on a loaded intent row. Keep the field
  with `#[allow(dead_code)]` and a `// wired by handle_* in phase 235` comment to pass
  the clippy gate. The generic param + trait bound stay so 235 adds methods, not a
  reshaped type.
- **D-11:** Introduce `ReturnUrls { success_url: String, cancel_url: String }` and a
  `CheckoutUrl(String)` newtype (matches spec naming) in ferro-payments.

### `start_checkout(billable, ttl)`
- **D-12:** Order of operations:
  1. `create_reserved(...)` — INSERT a `reserved` row with
     `expires_at = Utc::now() + ttl` (consumer's reservation window; **independent of**
     Stripe's own server-side session expiry in `CheckoutIntent.expires_at`).
  2. Build `CheckoutBuilder::new(Mode::Payment)` with one line item from the billable
     (`amount_cents`, `currency`, `checkout_line_description`), `success_url`/`cancel_url`
     from `return_url_builder(billable)`, and — when `connect_account_id()` is `Some` —
     `.destination(account_id, Stripe::config().application_fee_for(amount_cents))`.
  3. Set a **deterministic idempotency key** derived from the intent id
     (e.g. `format!("checkout-{intent_id}")`) before `create()`.
  4. On success, attach `stripe_session_id` and snapshot `application_fee_cents` onto
     the row.
  5. Return `CheckoutUrl`.
- **D-13:** `payment_intent_id` is **not** known at checkout-session creation
  (`CheckoutIntent` carries only `session_id` + `url` + `expires_at`); it arrives later
  on the webhook. Leave `payment_intent_id` / `charge_id` NULL here — matches the 233
  data model.
- **D-14:** If `create_checkout_session` fails after the row was inserted, the
  `reserved` row remains and is swept by the phase-236 reaper. `start_checkout` returns
  `PaymentError::Stripe(..)`. (Do not attempt compensating delete — the reaper is the
  single cleanup path.)

### `request_refund(intent_id, amount_cents)` (resolves DISCREPANCY-2)
- **D-15:** Flow: load the intent by id (`NotFound` if absent) → require `status = paid`
  **and** `charge_id` present (else `StatusPrecondition`) → snapshot
  `refund_amount_cents` via a **`GuardedUpdate` with `WHERE refund_amount_cents IS NULL`**
  (app-layer dedup, since async-stripe 0.41 does not forward idempotency keys to the
  Stripe API — a `0`-rows result means a refund is already in flight, so **no-op /
  do not call Stripe twice**) → call `stripe.create_refund(charge_id, Some(amount_cents),
  key)`.
- **D-16:** `request_refund` does **not** flip status to `refunded`. The `charge.refunded`
  webhook does that in phase 235 (`mark_refunded`). Between `request_refund` and the
  webhook, the "refund-in-flight" state is the **predicate**
  `status = 'paid' AND refund_amount_cents IS NOT NULL AND refunded_at IS NULL` —
  **not** a new enum variant.
- **D-17:** **DISCREPANCY-2 flagged:** ROADMAP phase 236 describes
  "intents in `refund_requested` state". The 233-locked `PaymentIntentStatus` has only
  `reserved|paid|released|failed|refunded` (no `refund_requested`). Resolution: keep the
  5-variant enum and treat "refund_requested" as the D-16 predicate. The phase-236
  `ReconcileRefundsInFlight` reaper queries that predicate; it does **not** require an
  enum change. (If the planner finds a strong reason to prefer an explicit variant, that
  is a 233-migration change to escalate — default is the predicate, no migration churn.)

### `PaymentError` (full set)
- **D-18:** Extend the 233 enum (`NotFound`, `StatusPrecondition(String)`,
  `Db(#[from] sea_orm::DbErr)`) with:
  - `Stripe(#[from] ferro_stripe::Error)`
  - `Loader(Box<dyn std::error::Error + Send + Sync>)`
  - `AutoRefundTriggered { reason: AutoRefundReason }`
  Define `AutoRefundReason` (e.g. `LoaderError`, `BillableVanished`, `SideStateConflict`).
  Note: `AutoRefundTriggered` is **defined** in 234 (it is part of the error model) but
  is only **returned** by the webhook handlers in 235; defining it now keeps the error
  type stable across the two phases.

### Manifest / dependency wiring (resolves DISCREPANCY-3)
- **D-19:** Add `ferro-stripe = { path = "../ferro-stripe", version = "0.9" }` to
  `[dependencies]`. Confirm the exact published minor against `ferro-stripe/Cargo.toml`
  at plan time (currently `0.9.x`).
- **D-20:** Unit tests use the **D-02 StripeGateway mock** and do **not** need
  `ferro_stripe::Stripe::init(...)`. Only add ferro-stripe's `test-helpers` dev-feature
  if a test needs real event JSON (most webhook-event needs are phase 235).
- **D-21:** **DISCREPANCY-3 flagged:** `ferro-payments` currently sits in
  `publish.yml` **Wave 1b** — the same wave as `ferro-stripe`. Adding the ferro-stripe
  dependency creates an **intra-wave dependency** (1b → 1b), which the wave model does
  not order. Move `ferro-payments` to a **new Wave 1c** step that runs after the Wave 1b
  crates-io index wait (mirrors the existing 1a→1b wait pattern). Update
  `WAVE1B_CRATES` to drop `ferro-payments` and add the new `WAVE1C_CRATES="ferro-payments"`
  step + its index-wait before Wave 2.

### Claude's Discretion
- Exact module split: spec proposes `billable.rs`, `loader.rs`, `service.rs`,
  `refund.rs`. Planner may fold `request_refund` into `service.rs` or keep `refund.rs`;
  keep `webhook.rs`/`reaper.rs` absent until 235/236.
- Exact `StripeGateway` method names/`CheckoutRequest` shape, and whether the mock lives
  in `#[cfg(test)]` or behind a `test-helpers` feature.
- `AutoRefundReason` variant names and whether `CheckoutUrl`/`ReturnUrls` get builder
  ergonomics.
- Idempotency-key format string (any deterministic per-intent value is acceptable).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Authoritative spec
- `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md` — full crate
  design: `Billable` / `BillableKind` / `BillableLoader` (§ "Public API"),
  `PaymentService` (signatures, `start_checkout` / `request_refund` semantics),
  error model (§ "Error model"), Stripe integration table (§ "Integration with
  ferro-stripe"), webhook race semantics, **Open questions §1/§2/§4** (resolved here:
  D-08, helper deferred to 235, D-06). **Source of truth — there are no PAY-POLY-SVC
  entries in a REQUIREMENTS file; read this doc.**
- Companion (external repo, reference only):
  `gestiscilo-it/app:docs/superpowers/specs/2026-06-17-tenant-booking-upfront-payment-design.md`

### Prior phase (the data layer this builds on)
- `.planning/phases/233-ferro-payments-crate-polymorphic-billable/233-CONTEXT.md` —
  locked data-layer decisions (status enum is 5-variant — D-04 there; GuardedUpdate
  no-op semantics — D-09 there; column layout).
- `ferro-payments/src/intent/lifecycle.rs` — `create_reserved`, `mark_*`,
  `find_active_for`, `find_by_stripe_session` (the functions `PaymentService` composes).
- `ferro-payments/src/intent/status.rs`, `ferro-payments/src/error.rs`,
  `ferro-payments/src/lib.rs` (`BillableKind`), `ferro-payments/Cargo.toml`.

### ferro-stripe surface to reuse (verify signatures here — spec text is approximate)
- `ferro-stripe/src/lib.rs` — public exports. **`ferro_stripe::Stripe`** (static facade,
  NOT `Client`), `CheckoutBuilder`, `CheckoutIntent`, `LineItem`, `Mode`, `Error`,
  `ProcessedEventLog`, `MemoryProcessedLog`, `SyncDispatcher`, `StripeEvent`.
- `ferro-stripe/src/checkout.rs` — `CheckoutBuilder` (consuming combinators,
  `idempotency_key` required before `create()`; `.destination(account_id, fee_cents)`,
  `.manual_capture()`), `CheckoutIntent { session_id, url, expires_at, idempotency_key }`
  (no `payment_intent_id` — D-13).
- `ferro-stripe/src/refund.rs` — `refund::create(charge_id, amount_cents,
  idempotency_key, reason)`; **idempotency key is NOT forwarded by async-stripe 0.41**
  (drives the D-15 app-layer dedup).
- `ferro-stripe/src/client.rs` — `Stripe::client()` / `Stripe::config()` global static
  (the reason for the D-02 gateway seam).
- `ferro-stripe/src/config.rs` — `StripeConfig::application_fee_for(amount_cents)
  -> Option<i64>` (the D-12 fee snapshot source).
- `ferro-stripe/src/idempotency.rs` — `ProcessedEventLog` trait + `MemoryProcessedLog`
  (used in 235; `MemoryProcessedLog` available for any 234 test that wants it).

### Reusable primitives
- `ferro-orm/src/lib.rs` — `GuardedUpdate` (D-15 refund-snapshot dedup).

### Workspace conventions
- `CLAUDE.md` (project) — pre-commit gate (`fmt` + `clippy --all --all-targets
  -D warnings` + `test --all-features`), project-agnostic crate rule (Connect /
  return-url base reads `APP_URL` conventions — relevant if `return_url_builder`
  defaults are added; here the builder is consumer-supplied so no app-identity in-crate),
  publish.yml wave rule (D-21).
- `.github/workflows/publish.yml` — Wave 1a/1b/2/3 structure (D-21 adds 1c).
- `Cargo.toml` (workspace root) — members already include `ferro-payments`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase-233 lifecycle functions are the exact building blocks `start_checkout`
  (`create_reserved` + a session-id/fee attach update) and `request_refund` compose.
- `ferro_orm::GuardedUpdate` — atomic conditional update for the refund-in-flight
  dedup (D-15) and the session-id/fee attach.
- `ferro_stripe::CheckoutBuilder` / `refund::create` / `StripeConfig::application_fee_for`
  — the production `StripeGateway` impl is a thin wrapper over these.
- `MemoryProcessedLog` — ready-made in-memory `ProcessedEventLog` for tests.

### Established Patterns
- One `thiserror` error enum per crate (extend, don't add a second).
- `#[async_trait]` for object-safe async traits (`Billable`, `BillableLoader`,
  `StripeGateway`).
- Builder combinators consume `self -> Self` (ferro convention) — `ReturnUrls`/any
  new builder should match.

### Integration Points
- `ferro-payments/Cargo.toml` `[dependencies]` (add ferro-stripe) + `lib.rs`
  re-exports (`Billable`, `BillableLoader`, `PaymentService`, `StripeGateway`,
  `ReturnUrls`, `CheckoutUrl`, `AutoRefundReason`, extended `PaymentError`).
- `.github/workflows/publish.yml` wave move (D-21).

### Constraints / Net-New Risk
- **Mockability of a static-facade dependency** is the central design risk. The D-02
  `StripeGateway` seam is what makes the phase-goal "unit tests use mocked Stripe"
  achievable at all — without it, the orchestrator can only be integration-tested
  (236). Treat the seam as load-bearing, not incidental.
- Clippy `-D warnings` on the unused `loader` field / generic in 234 (D-10) — handle
  deliberately, don't let it block the gate.

</code_context>

<specifics>
## Specific Ideas

- The phase's value is a **trustworthy** polymorphic payment orchestrator: the
  `StripeGateway` mock + `BillableLoader` mock must let a unit test assert, with no
  network, that `start_checkout` (a) inserts a reserved row, (b) snapshots
  `application_fee_cents` when the billable is Connect-routed, and (c) attaches the
  returned session id; and that `request_refund` (d) refuses a non-`paid`/no-`charge_id`
  intent, (e) snapshots `refund_amount_cents`, and (f) **no-ops the second concurrent
  call** rather than double-refunding.
- Implement the spec's "Testing" table rows that fall in this phase
  (`PaymentService::start_checkout unit`, `request_refund` precondition + dedup);
  webhook-race / reaper / auto-refund-fallback rows are 235/236.

</specifics>

<deferred>
## Deferred Ideas

Surfaced from the spec but out of scope for 234:

- `wire_dispatcher` + `OnCheckoutCompleted/Expired/ChargeRefunded` handlers,
  `handle_session_completed/expired/charge_refunded`, idempotency via
  `ProcessedEventLog`, **auto-refund fallback dispatch** (loader-None / side-state
  conflict) — **phase 235**. (`AutoRefundTriggered` / `AutoRefundReason` are *defined*
  in 234 per D-18 but *returned* in 235.)
- `release_expired` / `ReleaseExpiredPaymentIntents` / `ReconcileRefundsInFlight`
  reapers, workspace example-`Billable` test bin, real ferro-stripe-test-mode
  integration test, version bump + publish `0.1.0` — **phase 236**.
- Spec open-Q3 (`ReconcileRefundsInFlight` cadence) — phase 236.
- Provider abstraction beyond Stripe (spec non-goal).

None of these were requested for 234 — discussion stayed within the
trait+loader+service-core boundary.

</deferred>

---

*Phase: 234-ferro-payments-billable-trait-loader-and-payment-service-cor*
*Context gathered: 2026-06-17*
