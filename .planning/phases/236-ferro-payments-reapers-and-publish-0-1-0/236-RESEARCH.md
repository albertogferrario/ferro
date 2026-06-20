# Phase 236: ferro-payments Reapers + Publish 0.1.0 — Research

**Researched:** 2026-06-17
**Domain:** Rust async — ferro-payments reaper jobs, async-stripe 0.41 Refund poll API, ferro-queue Job pattern, crates.io publish
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Reaper logic lives as methods on `PaymentService<L>` — `pub async fn release_expired(&self) -> Result<usize, PaymentError>` and `pub async fn reconcile_refunds_in_flight(&self) -> Result<usize, PaymentError>`. Both return count for observability.

**D-02:** Job structs are thin wrappers mirroring `ferro_stripe::ProcessStripeWebhook` — serializable identity fields + `#[serde(skip)] service: Option<Arc<PaymentService<L>>>` injected via `::new(service)`; `handle()` maps `PaymentError` → `ferro_queue::Error::JobFailed`.

**D-03:** Job structs generic over `L: BillableLoader + 'static`: `ReleaseExpiredPaymentIntents<L>` and `ReconcileRefundsInFlight<L>`.

**D-04:** Clock injection via optional `now: DateTime<Utc>` parameter on the internal query path (e.g. `release_expired_at(&self, now)` taking a cutoff; public `release_expired()` calls it with `Utc::now()`). A private `Clock` field is acceptable if cleaner.

**D-05:** Per-intent transaction granularity. One row's failure must NOT roll back others — log the failing intent id and continue. Return the count actually released.

**D-06:** Loader-vanished during release is benign (status was `reserved`, no money captured) — log and skip, no auto-refund.

**D-07:** Add `find_refunds_in_flight(conn, older_than)` helper to `intent/lifecycle.rs` (no such finder exists today — confirmed by grep).

**D-08:** Add a read-only Stripe poll method to `StripeGateway` seam — `async fn fetch_refund_status_for_payment_intent(...)`. Underlying primitive in `ferro-stripe`; gateway method + `MockStripeGateway` extension in `ferro-payments`.

**D-09:** Resolution semantics: succeeded → `mark_refunded` path (mirrors `handle_charge_refunded`); still pending → leave for next tick; never-landed/failed → `tracing::warn!`, do NOT auto-retry (double-refund hazard). 1h cadence, cron-configurable by consumer.

**D-10:** End-to-end integration is a `#[ignore]`-gated integration test in `ferro-payments/tests/` reading `STRIPE_TEST_SECRET_KEY`; skips cleanly when absent.

**D-11:** Add `ferro-queue = { path = "../ferro-queue", version = "0.2" }` to `ferro-payments/Cargo.toml`. New module `ferro-payments/src/reaper.rs`. Re-export both job structs from `lib.rs`.

**D-12:** Reconcile local/remote git divergence first (`git pull --rebase` via HTTPS gh credential helper).

**D-13:** Bump workspace version (currently `0.2.69` local); `ferro-payments` ships at `0.1.0` (already set in its `Cargo.toml`). One-shot publish.

**D-14:** Push → CI auto-publish chain. `ferro-payments 0.1.0` is a new crate — first publish must be bootstrapped locally (CI token is publish-update only, not publish-new). Tag per milestone convention.

**D-15:** Add `docs/src/features/payments.md` (+ SUMMARY link). Cross-link from `docs/src/features/stripe.md`. Run `cargo doc -Dwarnings` before publish push.

**D-16:** No ferro-mcp tool change.

### Claude's Discretion
- Clock shape (param vs private `Clock` field) — D-04.
- `RefundStatus` enum shape returned by the poll gateway method (D-08).
- Whether to mirror the example `Billable` under `examples/` in addition to the test.
- `reaper.rs` internal organization; per-reaper `#[cfg(test)]` module split.
- Exact next patch version after the rebase reveals the published tip (D-13 baseline 0.2.70).

### Deferred Ideas (OUT OF SCOPE)
- gestiscilo Phase 218+ (consumer-repo work).
- Per-connected-account application-fee rates.
- Consumer-facing auto-refund/reconcile observability hook beyond `tracing`.
- Payments-specific ferro-mcp introspection.
</user_constraints>

---

## Summary

Phase 236 closes out the `ferro-payments` milestone (phases 233–235) by adding two time-driven recovery reapers and publishing `ferro-payments 0.1.0` to crates.io. The code surface is straightforward — both reapers are `PaymentService<L>` methods plus thin `ferro-queue` Job wrapper structs that follow an exact template already in the tree (`ProcessStripeWebhook`). The highest-risk research item — the async-stripe 0.41 API for fetching refund status by PaymentIntent — is now fully resolved: `stripe::Refund::list(client, &ListRefunds { payment_intent: Some(pi_id), limit: Some(1), .. })` is available unconditionally (not behind a feature flag) in the version already compiled in the workspace. The `Refund.status` field is `Option<String>` with documented values `"pending"`, `"requires_action"`, `"succeeded"`, `"failed"`, `"canceled"`. The publish is operator-gated for the first-time `cargo publish -p ferro-payments` (CI token is publish-update only), and the local/remote git divergence (`f53ee35e` local WIP above published `5509e7af`) must be rebased before any version work.

**Primary recommendation:** Implement `reaper.rs` and the two `PaymentService<L>` methods first (offline-testable); add the `fetch_refund_status_for_payment_intent` primitive to `ferro-stripe/src/refund.rs` second; wire the Job structs and ferro-queue dep third; write tests; then publish with the explicit operator-gated local bootstrap step last.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Expired-intent recovery | ferro-payments (PaymentService method) | ferro-queue (Job wrapper) | Business logic in service; queue is only the scheduling skin |
| Refund-in-flight reconciliation | ferro-payments (PaymentService method) | ferro-stripe (poll primitive) | Domain lives in payments; Stripe read call is ferro-stripe responsibility |
| Stripe refund status polling | ferro-stripe (refund module) | — | V-95-01: no direct `stripe::` import in ferro-payments consumers |
| Job scheduling (cron) | Consumer (queue registration) | ferro-queue | The crate provides the Job struct; the consumer schedules it |
| Docs | `docs/src/features/payments.md` | SUMMARY link + cross-link from stripe.md | D-15 |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-queue` | `0.2` (workspace) | `Job` trait + `Queueable` blanket impl | Already in workspace; `ProcessStripeWebhook` template uses it |
| `sea-orm` | `1.0` | DB query for `find_expired` / `find_refunds_in_flight` | Already in `ferro-payments` |
| `async-stripe` | `0.41` | `Refund::list` + `ListRefunds` for poll primitive | Already compiled, `stripe::Refund` and `stripe::ListRefunds` available unconditionally |
| `chrono` | `0.4` | `DateTime<Utc>` clock injection | Already in `ferro-payments` |
| `async-trait` | `0.1` | `#[async_trait]` on `StripeGateway` + `Job` | Already in crate |

### Additions

| Dependency | Added To | Change |
|------------|----------|--------|
| `ferro-queue = { path = "../ferro-queue", version = "0.2" }` | `ferro-payments/Cargo.toml [dependencies]` | New (D-11) |

**Dependency cycle check:** `ferro-queue` depends on `rand`, `uuid`, `serde`, `serde_json`, `chrono`, `sea-orm`, `tracing`, `async-trait`. It has NO dependency on `ferro-payments`, `ferro-stripe`, or `ferro-orm`. [VERIFIED: ferro-queue/Cargo.toml grep] Adding `ferro-queue` to `ferro-payments` creates no cycle.

**Publish ordering:** `ferro-queue` is Wave 1a (pure leaf). `ferro-payments` is already Wave 1c (depends on `ferro-stripe` Wave 1b, which depends on `ferro-queue` Wave 1a). Adding the direct `ferro-queue` dep to `ferro-payments` does not change the ordering. [VERIFIED: `.github/workflows/publish.yml`]

---

## Architecture Patterns

### System Architecture Diagram

```
Consumer cron tick
       │
       ▼
ReleaseExpiredPaymentIntents<L>::handle()
       │  calls
       ▼
PaymentService<L>::release_expired()
       │
       ├─► lifecycle::find_expired(conn, now)
       │         SQL: status='reserved' AND expires_at < now
       │
       └─► for each expired row:
               db.begin() → mark_released(id) → Ok(true)?
               │  true  ──► loader.load(kind, id)?
               │                 Ok(Some(b)) ──► b.on_released(&txn) → txn.commit()
               │                 Ok(None)|Err ──► log, txn.rollback, skip (D-06)
               │  false ──► no-op (reaper/webhook race, already released)
               └─► count += 1 on commit

ReconcileRefundsInFlight<L>::handle()
       │  calls
       ▼
PaymentService<L>::reconcile_refunds_in_flight()
       │
       ├─► lifecycle::find_refunds_in_flight(conn, older_than=now-1h)
       │         SQL: status='paid' AND refund_amount_cents IS NOT NULL
       │              AND refunded_at IS NULL AND reserved_at < older_than
       │
       └─► for each in-flight row:
               stripe.fetch_refund_status_for_payment_intent(pi_id)?
               │  RefundStatus::Succeeded ──► mark_refunded(id)→txn→on_refunded
               │  RefundStatus::Pending   ──► no-op, leave for next tick
               │  RefundStatus::Failed    ──► tracing::warn!, no auto-retry (D-09)
               └─► count += 1 on resolved
```

### Recommended Project Structure

```
ferro-payments/src/
├── reaper.rs              # NEW: ReleaseExpiredPaymentIntents<L> + ReconcileRefundsInFlight<L>
├── service.rs             # ADD: release_expired(), reconcile_refunds_in_flight()
├── intent/lifecycle.rs    # ADD: find_expired(), find_refunds_in_flight()
└── lib.rs                 # ADD: pub use reaper::{ReleaseExpiredPaymentIntents, ReconcileRefundsInFlight}

ferro-stripe/src/
└── refund.rs              # ADD: pub async fn list_for_payment_intent()

ferro-payments/tests/
└── integration.rs         # NEW: #[ignore]-gated end-to-end against Stripe test mode

docs/src/features/
└── payments.md            # NEW: consumer-facing one-call story
```

### Pattern 1: find_expired finder (sea-orm)

```rust
// Source: lifecycle.rs, modeled on find_active_for pattern (lifecycle.rs:166)
// Uses Column filter + ColumnTrait — consistent with existing finders

pub async fn find_expired<C: ConnectionTrait>(
    now: chrono::DateTime<Utc>,
    conn: &C,
) -> Result<Vec<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::Status.eq(PaymentIntentStatus::Reserved))
        .filter(Column::ExpiresAt.lt(now))
        .all(conn)
        .await
        .map_err(PaymentError::Db)
}
```

### Pattern 2: find_refunds_in_flight finder (sea-orm)

```rust
// Source: webhook.rs D-11 predicate definition + lifecycle.rs existing pattern
// "refund-in-flight" = status='paid' AND refund_amount_cents IS NOT NULL AND refunded_at IS NULL
// "older_than" parameter filters by age (reserved_at or paid_at — use reserved_at for simplicity,
//  but paid_at is more correct since refunds happen after payment; see Open Questions)

pub async fn find_refunds_in_flight<C: ConnectionTrait>(
    older_than: chrono::DateTime<Utc>,
    conn: &C,
) -> Result<Vec<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::Status.eq(PaymentIntentStatus::Paid))
        .filter(Column::RefundAmountCents.is_not_null())
        .filter(Column::RefundedAt.is_null())
        .filter(Column::PaidAt.lt(older_than))   // rows paid more than cadence ago
        .all(conn)
        .await
        .map_err(PaymentError::Db)
}
```

Note: `PaidAt` is `Option<DateTime<Utc>>`. For `paid` rows `paid_at` is always set (lifecycle invariant). The `.lt(older_than)` filter is the age gate.

### Pattern 3: per-intent transaction for release_expired (sea-orm TransactionTrait)

```rust
// Source: webhook.rs:151 — established txn pattern in PaymentService
// db.begin() → if Err: PaymentError::Db; txn.commit() → if Err: PaymentError::Db
// Same pattern as handle_session_completed

let txn = self.db.begin().await.map_err(PaymentError::Db)?;
// ... use &txn ...
txn.commit().await.map_err(PaymentError::Db)?;
```

The `begin()` call uses `sea_orm::TransactionTrait` (already imported in `webhook.rs` via `use sea_orm::TransactionTrait`). The same import works in `service.rs`.

### Pattern 4: ProcessStripeWebhook job struct (the exact template for D-02)

```rust
// Source: ferro-stripe/src/webhook/queue.rs (VERIFIED by direct read)

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReleaseExpiredPaymentIntents<L: BillableLoader + 'static> {
    // Identity fields (serialized); none needed for a reaper with no per-job state
    // (the job selects rows at execution time, not enqueue time)
    #[serde(skip)]
    pub service: Option<Arc<PaymentService<L>>>,
}

impl<L: BillableLoader + 'static> ReleaseExpiredPaymentIntents<L> {
    pub fn new(service: Arc<PaymentService<L>>) -> Self {
        Self { service: Some(service) }
    }
}

#[ferro_queue::async_trait]
impl<L: BillableLoader + 'static> ferro_queue::Job for ReleaseExpiredPaymentIntents<L> {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let svc = self.service.as_ref()
            .ok_or_else(|| ferro_queue::Error::JobFailed {
                job: "ReleaseExpiredPaymentIntents".to_string(),
                message: "service not injected — use ReleaseExpiredPaymentIntents::new()".to_string(),
            })?;
        svc.release_expired().await
            .map(|_| ())
            .map_err(|e| ferro_queue::Error::JobFailed {
                job: "ReleaseExpiredPaymentIntents".to_string(),
                message: e.to_string(),
            })
    }

    fn name(&self) -> &'static str {
        "ReleaseExpiredPaymentIntents"
    }
}
```

**Serde note:** `#[serde(skip)]` fields default to `None` on deserialization (standard serde behavior for `Option`). A deserialized job without the re-injected service errors cleanly in `handle()`. `Serialize + DeserializeOwned` bounds are satisfied because all serialized fields are `()` if none are added — only the skipped `Option<Arc<...>>` is non-serializable. [VERIFIED: ProcessStripeWebhook same pattern compiles and is tested]

**Generic constraint:** `L: BillableLoader + 'static` required by `Job: Send + Sync + 'static`. `Arc<PaymentService<L>>` is `'static` when `L: 'static`. [ASSUMED — standard Rust lifetime rule, not verified against Job trait bounds explicitly, but consistent with D-03 analysis]

### Pattern 5: async-stripe 0.41 Refund::list API (the poll primitive)

This is the highest-risk item. **VERIFIED by direct inspection of the async-stripe 0.41 source in the Cargo registry.**

```rust
// Source: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-stripe-0.41.0/
//         src/resources/generated/refund.rs (lines 99-101, 387-437)
// NOT behind a feature flag — available in the base crate.

// ferro-stripe/src/refund.rs — new function:
pub async fn list_for_payment_intent(
    payment_intent_id: &str,
) -> Result<Vec<stripe::Refund>, Error> {
    let client = crate::Stripe::client();
    let pi_id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;

    let mut params = stripe::ListRefunds::new();
    params.payment_intent = Some(pi_id);
    params.limit = Some(10); // at most a handful of refunds per PI

    let list = stripe::Refund::list(client, &params).await?;
    Ok(list.data)
}
```

**Key types from the source:**
- `stripe::Refund::list(client: &Client, params: &ListRefunds<'_>) -> Response<List<Refund>>` [VERIFIED]
- `stripe::ListRefunds<'a>` — fields: `charge`, `created`, `ending_before`, `expand`, `limit`, `payment_intent: Option<PaymentIntentId>`, `starting_after` [VERIFIED]
- `stripe::Refund.status: Option<String>` — documented values: `"pending"`, `"requires_action"`, `"succeeded"`, `"failed"`, `"canceled"` [VERIFIED from struct doc comment]
- `stripe::Refund.amount: i64` — always set [VERIFIED]
- `stripe::Refund.id: RefundId` [VERIFIED]

**`RefundStatus` enum for the gateway seam (Claude's discretion — D-08):**

```rust
// ferro-payments/src/service.rs or a small types module
pub enum RefundStatus {
    Succeeded { amount_cents: i64 },
    Pending,
    Failed { reason: Option<String> },
}
```

The production `StripeClientGateway::fetch_refund_status_for_payment_intent` calls `ferro_stripe::refund::list_for_payment_intent`, inspects the most recent refund's `.status`, and maps to `RefundStatus`. The `MockStripeGateway` records calls and returns a canned `RefundStatus`.

**fetch_refund_status_for_payment_intent gateway method signature:**

```rust
// Add to the StripeGateway trait in service.rs:
async fn fetch_refund_status_for_payment_intent(
    &self,
    payment_intent_id: &str,
) -> Result<RefundStatus, ferro_stripe::Error>;
```

The production impl calls `ferro_stripe::refund::list_for_payment_intent(pi_id)`, takes `list.first()`, and maps `.status.as_deref()` to `RefundStatus`.

**Edge case — no refund found:** If `list.data` is empty (race: Stripe processed the refund before the poll, and there is no record yet, or PI has no refund at all), return `RefundStatus::Pending` (safe: next tick will retry). If `list.data` has multiple refunds (partial refund + another), take the most recent by `created` timestamp.

### Pattern 6: #[ignore]-gated integration test pattern

```rust
// Source: ferro-mcp-server/tests/intent_loop.rs (lines 191-194)
// and framework/tests/constraint_map_pg_gate.rs (doc comment)

// ferro-payments/tests/integration.rs
#[tokio::test]
#[ignore] // run with: STRIPE_TEST_SECRET_KEY=sk_test_... cargo test -p ferro-payments -- --ignored
async fn e2e_checkout_and_release() {
    let key = match std::env::var("STRIPE_TEST_SECRET_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("STRIPE_TEST_SECRET_KEY not set — skipping integration test");
            return;
        }
    };
    // ... test body using the key to init ferro_stripe::Stripe ...
}
```

The skip pattern is `return` early (not `panic!`). The test is `#[ignore]` so `cargo test --all-features` skips it by default. Run with `cargo test -p ferro-payments -- --ignored integration`.

### Anti-Patterns to Avoid

- **Batch transaction for release_expired:** A single transaction over all expired rows means one DB error aborts all releases. Use per-intent transactions (D-05).
- **Auto-retry on failed refund in reconcile:** If `RefundStatus::Failed`, do NOT call `create_refund_for_payment_intent` again — the original refund_amount_cents is already set, creating a second Stripe call risks a double refund (D-09 / 235 D-11 rationale).
- **Importing `stripe::` directly in ferro-payments:** All Stripe primitives go through `ferro-stripe` first (V-95-01 / project-agnostic crate rule from CLAUDE.md).
- **Committing without running `cargo doc -Dwarnings`:** Prior publishes have failed on docs build. Run it before the push (D-15, CI Docs build).
- **Using `cargo publish -p ferro-payments` from CI for the first publish:** The CARGO_REGISTRY_TOKEN secret is publish-update only. The first publish must be run locally by the operator.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Guarded status transitions | Custom SQL UPDATE | `ferro_orm::GuardedUpdate` | Already handles the 0-rows-affected no-op semantic; used by all existing lifecycle fns |
| Job serialization with runtime handle | Custom serde impls | `#[serde(skip)] Option<Arc<...>>` + `::new()` pattern | Exact template in `ProcessStripeWebhook`; serde skip gives correct default on deserialize |
| Stripe refund status fetch | Direct `stripe::` call in ferro-payments | `ferro-stripe/src/refund.rs` + gateway seam | Project-agnostic crate rule: no `stripe::` import in consumers |
| DB transaction | Manual SQL | `sea_orm::TransactionTrait::begin` / `txn.commit()` | Established pattern in webhook.rs |

---

## Runtime State Inventory

> This phase is NOT a rename/refactor/migration phase. No runtime state inventory required.

---

## Common Pitfalls

### Pitfall 1: `PaidAt` is `Option` but always set for `paid` rows

**What goes wrong:** The `find_refunds_in_flight` filter uses `.filter(Column::PaidAt.lt(older_than))`. If a row somehow has `paid_at = NULL` with `status = 'paid'` (data anomaly), it is excluded by the `IS NOT NULL` semantics of `lt`. This is actually the correct behavior — a row with NULL `paid_at` cannot have a reliable age.

**How to avoid:** Trust the lifecycle invariant (`mark_paid` always sets `paid_at`). Document it in the finder's doc comment.

### Pitfall 2: Generic `L` on serde-derived Job structs

**What goes wrong:** `#[derive(Serialize, Deserialize)]` on `ReleaseExpiredPaymentIntents<L>` implicitly requires `L: Serialize + DeserializeOwned`. Since `L` is in `#[serde(skip)]`, serde does NOT add those bounds automatically — but Rust may still add them via the derive macro's default bound generation.

**How to avoid:** Add `#[serde(bound = "")]` to suppress the automatic `L: Serialize + DeserializeOwned` bound injection:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]  // suppress L: Serialize + DeserializeOwned
pub struct ReleaseExpiredPaymentIntents<L: BillableLoader + 'static> { ... }
```

This is the correct pattern for generic types with a `#[serde(skip)]` field. [ASSUMED — standard serde generic bound issue; verify by compiling]

### Pitfall 3: ferro-queue registration requires concrete type

**What goes wrong:** Consumers call `job.dispatch().await?` on a concrete `ReleaseExpiredPaymentIntents<MyLoader>`. The `Queueable` blanket impl (`impl<T> Queueable for T where T: Job + Serialize + DeserializeOwned`) only applies to concrete types at the call site. Since `L` is consumer-defined, this is correct — the consumer instantiates with their concrete loader type.

**How to avoid:** Document in the docs page that the consumer must provide a concrete loader type when scheduling the job.

### Pitfall 4: Workspace version vs. ferro-payments version

**What goes wrong:** The workspace version bump (`0.2.70`) and the `ferro-payments` version (`0.1.0`) are independent. The publish workflow reads the workspace version from root `Cargo.toml` for the auto-bump logic, but `ferro-payments` has its own `version = "0.1.0"` in `ferro-payments/Cargo.toml`. The CI publish step does `cargo publish -p ferro-payments` which uses the crate's own version, not the workspace version.

**How to avoid:** Verify `ferro-payments/Cargo.toml` still has `version = "0.1.0"` (not `version.workspace = true`) before the publish push. [VERIFIED: ferro-payments/Cargo.toml explicitly declares `version = "0.1.0"`]

### Pitfall 5: async-stripe `Refund::list` is synchronous (returns `Response<List<Refund>>`)

**What goes wrong:** `stripe::Refund::list` signature is `-> Response<List<Refund>>` where `Response<T>` is `Pin<Box<dyn Future<Output = Result<T, StripeError>>>>`. It IS async (awaitable), but the signature does not look like `async fn`. The `.await` on the return value is correct.

**How to avoid:** Treat `Response<T>` as an awaitable future — `let list = stripe::Refund::list(client, &params).await?;` works correctly.

---

## Code Examples

### find_refunds_in_flight (lifecycle.rs, modeled on existing finders)

```rust
// Source: lifecycle.rs:166-178 (find_active_for pattern), lifecycle.rs:1-10 (imports)
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

pub async fn find_refunds_in_flight<C: ConnectionTrait>(
    older_than: chrono::DateTime<Utc>,
    conn: &C,
) -> Result<Vec<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::Status.eq(PaymentIntentStatus::Paid))
        .filter(Column::RefundAmountCents.is_not_null())
        .filter(Column::RefundedAt.is_null())
        .filter(Column::PaidAt.lt(older_than))
        .all(conn)
        .await
        .map_err(PaymentError::Db)
}
```

### MockStripeGateway extension for fetch_refund_status_for_payment_intent

```rust
// Source: service.rs MockStripeGateway pattern (lines 388-454)
// Add to MockStripeGateway in tests (service.rs or reaper.rs cfg(test) block):

canned_refund_status: Mutex<Option<Result<RefundStatus, ferro_stripe::Error>>>,
poll_calls: Mutex<Vec<String>>, // payment_intent_ids polled

// In impl StripeGateway for MockStripeGateway:
async fn fetch_refund_status_for_payment_intent(
    &self,
    pi_id: &str,
) -> Result<RefundStatus, ferro_stripe::Error> {
    self.poll_calls.lock().unwrap().push(pi_id.to_string());
    self.canned_refund_status
        .lock()
        .unwrap()
        .take()
        .unwrap_or(Ok(RefundStatus::Succeeded { amount_cents: 1000 }))
}
```

### release_expired_at (testable inner method)

```rust
// Source: service.rs (D-04 clock injection pattern)
impl<L: BillableLoader> PaymentService<L> {
    /// Inner method accepting an explicit `now` for deterministic tests.
    pub(crate) async fn release_expired_at(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize, PaymentError> {
        use sea_orm::TransactionTrait;
        let expired = crate::intent::lifecycle::find_expired(now, &self.db).await?;
        let mut released = 0usize;
        for intent in expired {
            let result: Result<(), PaymentError> = async {
                let marked = crate::intent::lifecycle::mark_released(intent.id, &self.db).await?;
                if !marked {
                    return Ok(()); // racing webhook already released — no-op
                }
                let kind = crate::BillableKind::from_string(intent.billable_kind.clone());
                match self.loader.load(kind, intent.billable_id).await {
                    Ok(Some(billable)) => {
                        let txn = self.db.begin().await.map_err(PaymentError::Db)?;
                        match billable.on_released(&txn).await {
                            Ok(()) => { txn.commit().await.map_err(PaymentError::Db)?; }
                            Err(e) => { txn.rollback().await.ok(); return Err(e); }
                        }
                    }
                    Ok(None) | Err(_) => {
                        // D-06: loader-vanished is benign for release — log and skip
                        tracing::warn!(intent_id = intent.id, "release_expired: loader returned None/Err — skipping (no money captured)");
                    }
                }
                Ok(())
            }.await;
            match result {
                Ok(()) => released += 1,
                Err(e) => {
                    tracing::error!(intent_id = intent.id, err = %e, "release_expired: per-intent error — continuing");
                }
            }
        }
        Ok(released)
    }

    pub async fn release_expired(&self) -> Result<usize, PaymentError> {
        self.release_expired_at(chrono::Utc::now()).await
    }
}
```

---

## Open Questions

1. **`find_refunds_in_flight` age anchor: `paid_at` vs. `reserved_at`**
   - What we know: D-07 says "older than the cadence window (default 1h)". The refund-in-flight state is set when `request_refund` or `trigger_auto_refund` is called, which happens after payment. `paid_at` is the timestamp of payment. `refund_amount_cents` could be set minutes to hours after `paid_at`.
   - What's unclear: the spec says filter rows "older than the cadence window" but doesn't specify which timestamp. A row where `refund_amount_cents` was just set is not yet "in flight > 1h" even if `paid_at` was hours ago.
   - **Recommendation:** Use `paid_at` as a conservative proxy. It is always set for paid rows and is simpler than adding a `refund_requested_at` column. A row where `paid_at` is < 1h ago cannot have a Stripe refund that's been in flight > 1h, so the filter is safe (slightly over-eager for very fresh refunds, but `RefundStatus::Pending` will correctly leave them for next tick). Alternatively, accept any row matching the predicate without the age filter and let the `Pending` → leave logic handle freshness. This simpler approach means the reconcile reaper checks all in-flight rows on every tick, which is correct behavior.
   - **Simpler alternative (recommended):** Remove the `older_than` parameter and select all rows matching the in-flight predicate unconditionally. The `older_than` filter is an optimization, not a correctness requirement. The planner should choose.

2. **`RefundStatus` location: service.rs vs. a dedicated types module**
   - What we know: `RefundStatus` is returned by the `StripeGateway` method and used in `reconcile_refunds_in_flight`. It belongs in the crate's type surface.
   - **Recommendation:** Declare `RefundStatus` in `service.rs` alongside `ReturnUrls`, `CheckoutUrl`, `CheckoutRequest`, `CheckoutResponse`. This is consistent with the existing pattern. Export from `lib.rs` if consumers need it (probably not).

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build/test | ✓ | 1.88.0 (workspace) | — |
| async-stripe 0.41 | Refund::list | ✓ | In Cargo.lock | — |
| Stripe test-mode key | `#[ignore]` integration test | Not checked in CI | — | Test skips cleanly on absent env var |
| crates.io publish token | First publish of ferro-payments | ✗ for CI (publish-update only) | — | Local terminal: `cargo publish -p ferro-payments` |
| HTTPS git remote | `git pull --rebase` (D-12) | ✓ | gh credential helper | — |

**Missing dependencies with no fallback:**
- Operator with local terminal + crates.io token: required for `cargo publish -p ferro-payments 0.1.0` (first publish, CI token is publish-update only).

**Missing dependencies with fallback:**
- Stripe test-mode secret key: integration test skips cleanly, CI stays green.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `tokio::test` (async, via `tokio = { version = "1", features = ["full"] }` in dev-deps) |
| Config file | None (inline `#[tokio::test]` attributes) |
| Quick run command | `cargo test -p ferro-payments` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PAY-POLY-REAP-01 | `release_expired_at` with injected clock releases only expired `reserved` rows, calls `on_released` per row | unit (in-memory SQLite) | `cargo test -p ferro-payments release_expired` | ❌ Wave 0 |
| PAY-POLY-REAP-01 | `release_expired` skips already-released rows (mark_released returns Ok(false)) | unit | `cargo test -p ferro-payments reaper_skips_already_released` | ❌ Wave 0 |
| PAY-POLY-REAP-01 | `release_expired` continues on per-intent error (isolation D-05) | unit | `cargo test -p ferro-payments reaper_continues_on_error` | ❌ Wave 0 |
| PAY-POLY-REAP-01 | Webhook + reaper race: reaper wins → webhook no-ops (existing `webhook_reaper_race` in webhook.rs covers one direction) | unit | `cargo test -p ferro-payments webhook_reaper_race` | ✅ |
| PAY-POLY-REAP-02 | `ReleaseExpiredPaymentIntents::handle()` calls `release_expired`; errors map to `JobFailed` | unit | `cargo test -p ferro-payments job_struct_release` | ❌ Wave 0 |
| PAY-POLY-REAP-02 | Job with no service injected errors cleanly with `JobFailed` | unit | `cargo test -p ferro-payments job_no_service_injected` | ❌ Wave 0 |
| PAY-POLY-REAP-03 | `reconcile_refunds_in_flight`: succeeded refund → `mark_refunded` + `on_refunded(amount)` | unit | `cargo test -p ferro-payments reconcile_succeeded` | ❌ Wave 0 |
| PAY-POLY-REAP-03 | `reconcile_refunds_in_flight`: pending refund → leave row for next tick | unit | `cargo test -p ferro-payments reconcile_pending_noop` | ❌ Wave 0 |
| PAY-POLY-REAP-03 | `reconcile_refunds_in_flight`: failed refund → `tracing::warn!`, no Stripe call | unit | `cargo test -p ferro-payments reconcile_failed_no_retry` | ❌ Wave 0 |
| PAY-POLY-REAP-04 | `#[ignore]` integration test skips cleanly when `STRIPE_TEST_SECRET_KEY` absent | integration | `cargo test -p ferro-payments -- --ignored` (auto-skip) | ❌ Wave 0 |
| PAY-POLY-REAP-04 | End-to-end: start_checkout → release_expired → verify row status=released (test-mode Stripe) | integration (opt-in) | `STRIPE_TEST_SECRET_KEY=... cargo test -p ferro-payments -- --ignored e2e` | ❌ Wave 0 |

### Wave 0 Gaps

- [ ] `ferro-payments/tests/integration.rs` — covers PAY-POLY-REAP-04; define tiny example `Billable` here
- [ ] Reaper unit tests in `ferro-payments/src/reaper.rs` `#[cfg(test)]` module — covers PAY-POLY-REAP-01/02/03
- [ ] `ferro-payments/src/service.rs` reaper method tests — clock injection, count assertions

*(Existing tests in `lifecycle.rs` and `webhook.rs` are green and require no changes)*

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-payments`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green + `cargo doc -Dwarnings` green before publish push

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | No | — |
| V5 Input Validation | Partial | `payment_intent_id` parsed via `stripe::PaymentIntentId::parse()` before API call |
| V6 Cryptography | No | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Double refund via concurrent reaper ticks | Tampering | `mark_refunded` GuardedUpdate no-op on second call; `reconcile` returns Ok(false) → skip |
| Refund auto-retry on failure | Tampering | D-09: explicit prohibition on auto-retry; `tracing::warn!` only |
| PI ID injection in poll call | Tampering | `stripe::PaymentIntentId::parse()` validates format before calling Stripe API |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `#[serde(bound = "")]` required to suppress `L: Serialize + DeserializeOwned` bounds on generic Job structs | Code Examples / Pattern 4 | Compiler error during implementation; easy to detect and fix |
| A2 | `L: BillableLoader + 'static` satisfies `Job: Send + Sync + 'static` via `Arc<PaymentService<L>>` | Standard Stack | Compiler error; fix by adding `+ Sync` to `L` bound if needed |
| A3 | `paid_at` is always non-NULL for rows with `status='paid'` (lifecycle invariant) | Pattern 2 / find_refunds_in_flight | Silent exclusion of in-flight rows if invariant broken; log-monitor mitigation |

**All other claims in this research are verified by direct file inspection or Cargo registry source inspection.**

---

## Sources

### Primary (HIGH confidence)

- `ferro-payments/src/service.rs` — `PaymentService<L>`, `StripeGateway` trait, `MockStripeGateway`, test harness patterns [VERIFIED: direct read]
- `ferro-payments/src/intent/lifecycle.rs` — `mark_released`, `mark_refunded`, `find_active_for`, existing finder patterns [VERIFIED: direct read]
- `ferro-payments/src/webhook.rs` — `wire_dispatcher`, per-intent txn pattern, `db.begin()` usage, refund-in-flight predicate [VERIFIED: direct read]
- `ferro-stripe/src/webhook/queue.rs` — `ProcessStripeWebhook` exact template for D-02 [VERIFIED: direct read]
- `ferro-queue/src/job.rs` — `Job` trait full signature, `async fn handle(&self) -> Result<(), Error>`, all optional methods [VERIFIED: direct read]
- `ferro-queue/src/lib.rs` — `Queueable` blanket impl requires `Job + Serialize + DeserializeOwned` [VERIFIED: direct read]
- `~/.cargo/registry/src/.../async-stripe-0.41.0/src/resources/generated/refund.rs` — `Refund::list`, `ListRefunds.payment_intent`, `Refund.status: Option<String>` [VERIFIED: direct read of Cargo registry source]
- `ferro-payments/Cargo.toml` — `version = "0.1.0"`, no `ferro-queue` dep today [VERIFIED: direct read]
- `ferro-stripe/Cargo.toml` — `async-stripe = { version = "0.41", features = ["billing", "checkout", ...] }` [VERIFIED: direct read]
- `ferro-queue/Cargo.toml` — no `ferro-payments` or `ferro-stripe` dep (no cycle) [VERIFIED: direct read/grep]
- `.github/workflows/publish.yml` — Wave 1b (ferro-stripe), Wave 1c (ferro-payments); CI token is `CARGO_REGISTRY_TOKEN` [VERIFIED: direct read]
- `ferro-payments/src/lib.rs` — current re-exports (no `reaper` module yet) [VERIFIED: direct read]
- `ferro-payments/src/intent/status.rs` — `PaymentIntentStatus` variants, NO `RefundRequested` [VERIFIED: direct read]
- `ferro-mcp-server/tests/intent_loop.rs` — `#[ignore]` + env-var guard pattern [VERIFIED: direct read]
- Workspace `Cargo.toml` — `version = "0.2.69"` [VERIFIED: direct read]

### Secondary (MEDIUM confidence)

- Context7 async-stripe docs — confirmed `stripe::Refund` API shapes; `Refund::list` not explicitly shown but type confirmed via registry source
- Prior phases 233/234/235 code — established test harness (in-memory SQLite + MockStripeGateway + MemoryProcessedLog) [VERIFIED]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified from Cargo.toml and Cargo.lock
- Architecture: HIGH — all patterns verified from existing source in the crate
- Async-stripe poll API: HIGH — verified from registry source at `~/.cargo/registry/src/.../async-stripe-0.41.0/src/resources/generated/refund.rs`
- Pitfalls: MEDIUM/HIGH — serde generic bound issue is ASSUMED (A1), all others verified

**Research date:** 2026-06-17
**Valid until:** 2026-07-17 (stable domain — ferro-payments is under active development but async-stripe 0.41 API is stable)
