# Phase 236: ferro-payments reapers + workspace test bin + publish 0.1.0 - Context

**Gathered:** 2026-06-17
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults auto-selected; review decisions below)

<domain>
## Phase Boundary

Close out the `ferro-payments` crate (built across 233–235) and ship it. Three concerns:

1. **Reapers** — two time-driven recovery jobs:
   - `ReleaseExpiredPaymentIntents` — single SQL pass over
     `payment_intents WHERE status = 'reserved' AND expires_at < now()`, dispatching
     `on_released` per row. The release path for slots the customer never paid for.
   - `ReconcileRefundsInFlight` — polls Stripe for intents stuck in the
     refund-in-flight predicate (235 D-11: `status='paid' AND refund_amount_cents IS
     NOT NULL AND refunded_at IS NULL`) older than ~1 hour, and resolves them. This is
     the recovery mechanism 235 deferred here by design.
   Both are **ferro-queue–compatible job structs** consumers schedule via cron.
2. **Workspace test bin** — a tiny example `Billable` driving the crate end-to-end
   against ferro-stripe test mode (the integration row in the spec Testing table).
3. **Publish** — version-bump the ferro workspace and publish `ferro-payments 0.1.0`
   (plus the ferro-stripe republish that 235's payment-intent refund primitive
   requires) to crates.io. After publication, gestiscilo Phase 218 is unblocked.

**Out of scope:**
- Any consumer (gestiscilo) code — handed off via the publish, written in the
  consumer repo (cross-repo phase split convention).
- New ferro-stripe event types or dispatcher contract changes.
- Per-connected-account fee rates (235 deferred; future ferro-stripe change).

</domain>

<decisions>
## Implementation Decisions

### Reaper code placement + job-struct shape
- **D-01:** Reaper **logic lives as methods on `PaymentService<L>`** —
  `pub async fn release_expired(&self) -> Result<usize, PaymentError>` (already named
  in the spec Public API) and a new
  `pub async fn reconcile_refunds_in_flight(&self) -> Result<usize, PaymentError>`.
  This keeps them unit-testable with the existing in-memory SQLite + `MockStripeGateway`
  + mocked `BillableLoader` harness (233/234/235 template) — no queue runtime needed
  to test the core behavior. Both return a count for observability/test assertions.
- **D-02:** The **Job structs are thin wrappers** that follow the established
  `ferro_stripe::ProcessStripeWebhook` pattern (`ferro-stripe/src/webhook/queue.rs`):
  serializable identity fields + a `#[serde(skip)] service: Option<Arc<PaymentService<L>>>`
  runtime handle injected via a `::new(service)` constructor at consumer-registration
  time; `handle()` errors clearly if the handle was not re-injected after deserialize.
  `handle()` calls the matching `PaymentService` method and maps `PaymentError` →
  `ferro_queue::Error::JobFailed` so the queue's retry/backoff applies.
- **D-03:** Job structs are generic over `L: BillableLoader + 'static` (mirrors
  `PaymentService<L>` / `wire_dispatcher`). `ReleaseExpiredPaymentIntents<L>` and
  `ReconcileRefundsInFlight<L>`. Consumers instantiate with their concrete loader.
  `Job: Send + Sync + 'static` is satisfied because `Arc<PaymentService<L>>` is `'static`
  when `L: 'static`.

### Clock injection (time-driven testability)
- **D-04:** Both reapers compare against "now". Inject the clock so the spec's
  "Injected clock" reaper tests are offline and deterministic. **Default: an optional
  `now: DateTime<Utc>` parameter on the internal query path** (e.g.
  `release_expired_at(&self, now)` taking the cutoff; public `release_expired()` calls
  it with `Utc::now()`). Avoids adding a `Clock` trait to the public surface for two
  call sites. (Claude's discretion: a private `Clock` field is acceptable if the
  planner finds it cleaner — the requirement is deterministic tests, not a specific
  shape.)

### `release_expired` transaction granularity + failure isolation
- **D-05:** **Per-intent transaction**, not one transaction over the whole batch. For
  each expired row: open a txn → `mark_released` (guarded `reserved→released`; `false`
  = a racing webhook already took it → skip, no-op) → on `true`, `billable.on_released(&txn)`
  → commit. One row's failure must **not** roll back the others — log the failing
  intent id and continue; the reaper re-runs on the next cron tick and picks up stragglers.
  Return the count actually released. This matches the 235 webhook handler's per-intent
  txn semantics (235 D-06/07) and the partial-unique race handling (spec race table).
- **D-06:** Loader-vanished during release (`load` → `Ok(None)`/`Err`) is **benign for
  release** (no money was captured — status was `reserved`, nothing to refund): log and
  skip that row. This differs from the webhook auto-refund path, where money *was*
  captured. No auto-refund in the release reaper.

### `ReconcileRefundsInFlight` — Stripe poll + resolution
- **D-07:** The reaper selects rows matching the 235 refund-in-flight predicate
  (`status='paid' AND refund_amount_cents IS NOT NULL AND refunded_at IS NULL`) with the
  in-flight marker older than the cadence window (default 1h; D-09). Add a
  `find_refunds_in_flight(conn, older_than)` helper to `intent/lifecycle.rs` if absent
  (234 established the predicate; confirm whether a finder already exists).
- **D-08:** Add a **read-only Stripe poll method to the `StripeGateway` seam** —
  `async fn fetch_refund_status_for_payment_intent(&self, payment_intent_id) ->
  Result<RefundStatus, ferro_stripe::Error>` (or list-refunds-for-PI). Polling is an
  **idempotent query** — this is why 235 D-11 chose reconcile over a non-idempotent
  reset+retry (async-stripe 0.41 does not forward idempotency keys; a blind retry risks
  a double refund). The underlying Stripe read primitive lives in **ferro-stripe** per
  V-95-01 ("no direct `stripe::` import in consumers" / "new Stripe primitive →
  ferro-stripe first"); the gateway method + `MockStripeGateway` extension live in
  ferro-payments. The 236 publish bumps ferro-stripe + ferro-payments together so the
  ferro-stripe addition is absorbed here.
- **D-09:** Resolution: when the poll reports the refund **succeeded**, resolve via the
  same path as `handle_charge_refunded` (235 D-07): txn → `mark_refunded` (guarded
  `paid→refunded`) → `billable.on_refunded(&txn, amount)` → commit (sets `refunded_at`,
  clearing the in-flight predicate). When the poll reports the refund **still pending**,
  leave the row for the next tick. When the poll reports the original refund **never
  landed / failed**, log a `tracing::warn!` (operator-actionable; do NOT auto-retry the
  refund here — that is the double-refund hazard 235 D-11 names). The 1h cadence is the
  default; **configurable by the consumer via the ferro-queue cron expression** at
  registration (spec open-question 3 — resolved: cron-configurable, no crate-side knob).

### Workspace "test bin" form
- **D-10:** Do **not** add a new publishable workspace member crate just for a test
  binary (would pollute the publish set + require a publish.yml wave entry). Instead the
  end-to-end integration is a **`#[ignore]`-gated integration test** in
  `ferro-payments/tests/` that reads a Stripe test-mode secret from env (e.g.
  `STRIPE_TEST_SECRET_KEY`) and **skips cleanly when the key is absent** (so CI
  `--all-features` stays green without secrets — isolate-before-spending convention).
  The "tiny example `Billable`" is defined in that test (and may additionally be mirrored
  under `ferro-payments/examples/` for discoverability — Claude's discretion). This
  satisfies the spec's "Integration (workspace test bin) — real ferro-stripe test mode
  against a tiny example Billable" without a new crate or a hard secret dependency in CI.

### ferro-queue dependency + workspace wiring
- **D-11:** Add `ferro-queue = { path = "../ferro-queue", version = "0.2" }` to
  `ferro-payments/Cargo.toml` (the crate has no queue dep today — the reapers introduce
  it). ferro-queue is a Wave-1 leaf; ferro-payments is already Wave 1c in publish.yml, so
  publish ordering needs no change. Re-export the two job structs from
  `ferro-payments/src/lib.rs`. New module `ferro-payments/src/reaper.rs` (per spec crate
  layout) holding the job structs; the `PaymentService` methods (D-01) stay in
  `service.rs`.

### Publish + version + git reconciliation (operator-critical)
- **D-12:** **Reconcile the local/remote git divergence BEFORE any version work.** Per
  project memory, local `master` carries a WIP commit (`f53ee35e`, "SegmentedControl +
  SidebarLayout primitives") above the published `5509e7af`, diverged from the remote
  `0.2.68` bump. Run `git pull --rebase` (HTTPS via the gh credential helper — SSH is
  denied) and verify the resulting workspace version is strictly greater than what's on
  crates.io BEFORE bumping. Pushing a non-monotonic version fails CI auto-publish.
- **D-13:** Bump the **workspace version** (currently `0.2.69` local) to the next patch
  (`0.2.70` baseline, adjust after the rebase reveals the true published tip) — this
  republishes ferro core **and ferro-stripe** (235's payment-intent refund primitive +
  236's poll primitive). `ferro-payments` ships at **`0.1.0`** (independent crate version,
  already set in its Cargo.toml). One-shot publish so the consumer can pin both ferro and
  ferro-payments together (spec Versioning section).
- **D-14:** Publish via the established **push → CI auto-publish** chain (publish.yml
  Wave 1b ferro-stripe, Wave 1c ferro-payments). After a verified push, fix the recurrent
  stale `origin/master` ref locally (`git update-ref refs/remotes/origin/master HEAD`).
  Tag per existing milestone convention. New-crate caveat: the CI publish token is
  **publish-update only, not publish-new** — `ferro-payments 0.1.0` is a brand-new crate,
  so its **first publish must be bootstrapped from a local terminal** (`cargo publish -p
  ferro-payments`); subsequent versions go through CI. Flag this to the operator.

### Docs + MCP scope
- **D-15:** Add a `docs/src/features/payments.md` page (and SUMMARY link) — the
  consumer-facing one-call story: register migrations, `wire_dispatcher(...)`, schedule
  the two reapers via cron, the auto-refund + reconcile recovery model. Cross-link from
  `docs/src/features/stripe.md`. Run the CI Docs build (`cargo doc -Dwarnings`) before the
  publish push (it has bitten prior publishes).
- **D-16:** **No ferro-mcp tool change** — ferro-payments is a consumer library, not a
  framework route/model/handler the MCP introspects. Note this explicitly so the planner
  does not scope phantom MCP work. (If a future phase adds payments-specific MCP
  introspection, that is its own scope.)

### Claude's Discretion
- Clock shape (param vs private `Clock` field) — D-04, requirement is deterministic tests.
- `RefundStatus` enum shape returned by the poll gateway method (D-08).
- Whether to mirror the example `Billable` under `examples/` in addition to the test.
- reaper.rs internal organization; per-reaper `#[cfg(test)]` module split.
- Exact next patch version after the rebase reveals the published tip (D-13 baseline 0.2.70).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Authoritative spec
- `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md` — §"Public API"
  (`release_expired` on `PaymentService`), §"Crate layout" (`reaper.rs` =
  `ReleaseExpiredPaymentIntents` + `ReconcileRefundsInFlight`), §"Webhook race semantics"
  (partial-unique + status-precondition no-op on the reaper/webhook race), §"Testing"
  (reaper = injected clock; integration = workspace test bin against ferro-stripe test
  mode), §"Versioning + publication" (0.1.0 + one-shot ferro workspace bump),
  §"Open questions" (Q3 reconcile cadence = 1h, cron-configurable — resolved D-09).
  **Source of truth — PAY-POLY-REAP-01..04 are defined here, not in a REQUIREMENTS.md
  file.**

### Prior phases (what 236 builds on)
- `.planning/phases/235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-/235-CONTEXT.md`
  — D-11 (refund-in-flight stuck-state predicate; reconcile reaper is the named recovery
  here), D-07 (`handle_charge_refunded` happy-path resolver the reconcile reaper mirrors),
  the StripeGateway seam + MockStripeGateway extension pattern.
- `.planning/phases/234-ferro-payments-billable-trait-loader-and-payment-service-cor/234-CONTEXT.md`
  — D-15/16/17 refund-in-flight predicate definition, StripeGateway seam (D-01/02/03),
  PaymentService field shape, error model.

### Code surface 236 extends
- `ferro-payments/src/service.rs` — `PaymentService<L>`, `StripeGateway` trait (+ prod
  `StripeClientGateway` + test `MockStripeGateway`), `start_checkout`, `request_refund`,
  the 235 handlers; add `release_expired` + `reconcile_refunds_in_flight` + the poll
  gateway method here.
- `ferro-payments/src/intent/lifecycle.rs` — `mark_released` / `mark_refunded` guarded
  updates, `find_by_*` finders; add `find_expired` / `find_refunds_in_flight` finders.
- `ferro-payments/src/lib.rs` — re-exports (add the two job structs).
- `ferro-payments/src/webhook.rs` — `wire_dispatcher` (235) for the handler/reaper race.
- `ferro-payments/Cargo.toml` — add `ferro-queue`; `ferro-payments/README.md` exists.

### Queue job pattern to copy
- `ferro-stripe/src/webhook/queue.rs` — `ProcessStripeWebhook`: serializable fields +
  `#[serde(skip)]` runtime `Arc` handle injected via `::new(...)`, `handle()` →
  `ferro_queue::Error::JobFailed` mapping. The exact template for D-02.
- `ferro-queue/src/job.rs` — `Job` trait (`handle(&self)`, `name`, `max_retries`,
  `retry_delay`, `idempotency_key`, `timeout`); `Queueable` blanket impl
  (`ferro-queue/src/lib.rs`).

### Publish + workspace conventions
- `.github/workflows/publish.yml` — Wave 1b (`ferro-stripe`), Wave 1c (`ferro-payments`)
  ordering; auto-publish on push.
- `CLAUDE.md` — pre-commit gate (`fmt` + `clippy --all --all-targets -D warnings` +
  `test --all-features`); CI Docs build (`cargo doc -Dwarnings`) + cargo-deny; "add new
  crate to publish.yml" rule (already done); project-agnostic crate rule.
- Project memory: publish-token publish-new vs publish-update scoping (D-14), stale
  `origin/master` ref fix, local/remote divergence to rebase (D-12).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `MockStripeGateway` (235) records calls — extend with the poll method so
  reconcile-resolves-refund is asserted offline.
- `mark_released` / `mark_refunded` guarded updates — the `bool` return is the
  webhook/reaper race signal (no extra read), drives D-05/D-09 no-op handling.
- In-memory SQLite test harness (233/234/235) + `MemoryProcessedLog` (ferro-stripe).
- `ProcessStripeWebhook` (ferro-stripe) — the serialize-skip + Arc-injection job template.

### Established Patterns
- `#[async_trait]`; `GuardedUpdate` no-op idempotency; per-intent DB transaction;
  one `thiserror` enum per crate; `#[cfg(test)]` in-memory SQLite; `#[ignore]`-gated
  network integration tests skipping on absent secrets.

### Integration Points
- New `ferro-payments/src/reaper.rs`; `ferro-payments/src/lib.rs` re-exports;
  `ferro-payments/Cargo.toml` (+ ferro-queue); `ferro-stripe/src/refund.rs` or a new
  read module (poll primitive) + ferro-stripe republish; `docs/src/features/payments.md`
  + SUMMARY.

### Constraints / Net-New Risk
- **Highest risk: the Stripe poll primitive (D-08)** — research MUST confirm the
  async-stripe 0.41 API for fetching refund/PI status before the planner commits a
  signature, same diligence 235 applied to `CreateRefund.payment_intent`.
- **Publish is operator-gated, not a code task:** new-crate first-publish needs a local
  terminal (token is publish-update only — D-14); git divergence must be rebased first
  (D-12). The planner should isolate the publish into a final, clearly operator-flagged
  step, not bury it in a code plan.

</code_context>

<specifics>
## Specific Ideas

- Killer artifact for this phase: a consumer schedules recovery in three lines — register
  migrations, `wire_dispatcher`, and two `dispatch` cron entries for the reaper jobs — and
  every money-stuck edge case (unpaid expiry, refund-in-flight) self-heals. The docs page
  (D-15) should lead with that.
- Implement the spec Testing rows owned by 236: reaper-with-injected-clock (assert
  `on_released` per expired intent), webhook+reaper race (guarded-update no-op), and the
  gated end-to-end (D-10).

</specifics>

<deferred>
## Deferred Ideas

- gestiscilo Phase 218+ (tenant booking upfront payment) — consumer-repo work, unblocked
  by this publish; written there per cross-repo phase split.
- Per-connected-account application-fee rates (235 WR-02) — future ferro-stripe change.
- Consumer-facing auto-refund/reconcile observability hook beyond `tracing` — only if a
  consumer asks.
- Payments-specific ferro-mcp introspection — out of scope (D-16); its own phase if ever.

</deferred>

---

*Phase: 236-ferro-payments-reapers-and-publish-0-1-0*
*Context gathered: 2026-06-17*
