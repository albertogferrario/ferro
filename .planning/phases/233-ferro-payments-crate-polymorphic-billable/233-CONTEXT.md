# Phase 233: ferro-payments crate scaffold + PaymentIntent entity + migration - Context

**Gathered:** 2026-06-17
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults auto-selected; review decisions below)

<domain>
## Phase Boundary

Create a new workspace member `ferro-payments` parallel to `ferro-stripe`, containing
**only the data layer**:

- `BillableKind` (open-set, string-backed) and `PaymentIntentStatus` enums.
- The SeaORM `Entity` for the `payment_intents` table.
- Lifecycle methods on the model: `create_reserved`, `mark_paid`, `mark_released`,
  `mark_refunded`, `find_active_for`, `find_by_stripe_session`.
- Migration `m20260617_create_payment_intents`, portable across Postgres + SQLite +
  MySQL, with the partial unique index `(billable_kind, billable_id) WHERE status IN
  ('reserved','paid')` and supporting indexes `(tenant_id, status)`,
  `(stripe_session_id)`, `(payment_intent_id)`.
- Unit tests covering state transitions and partial-unique enforcement against
  in-memory SQLite.

**Out of scope (later phases):** `PaymentService`, `Billable` trait, `BillableLoader`,
`wire_dispatcher`, webhook handlers (234/235), reapers (236), the `ferro-stripe`
dependency wiring (234), and the full `PaymentError` variant set (234).

</domain>

<decisions>
## Implementation Decisions

### Cross-backend partial unique index (the key technical decision)
- **D-01:** The migration branches on `manager.get_database_backend()`. **Postgres
  and SQLite** get a true partial unique index — identical syntax: `CREATE UNIQUE
  INDEX uq_payment_intents_active ON payment_intents (billable_kind, billable_id)
  WHERE status IN ('reserved','paid')`. (SQLite ≥ 3.8.0 supports partial indexes;
  this is the test target.)
- **D-02:** **MySQL** has no partial/filtered indexes. Emulate with a stored
  generated column (e.g. `active_billable_key` = identity string when `status IN
  ('reserved','paid')`, else `NULL`) plus a plain `UNIQUE` index on it — MySQL does
  not deduplicate `NULL`s, which gives the correct "only one active row per billable"
  semantics. **Research must confirm** the minimum MySQL version (generated columns
  need 5.7+, 8.0 recommended) and the NULL-uniqueness behavior before the planner
  commits this exact column expression.
- **D-03:** No SeaORM-native partial-index API exists and there is **no precedent in
  this workspace** (existing migrations use plain `Index::create()`); the `WHERE`
  clause is emitted via raw SQL through the schema manager connection per backend.

### Column representations (portability-first)
- **D-04:** `status` is a `TEXT` column. Rust `PaymentIntentStatus` derives
  `DeriveActiveEnum` with `rs_type = "String"`, `db_type = "Text"`, and per-variant
  `string_value` (`reserved | paid | released | failed | refunded`). **No native DB
  ENUM type** (non-portable across the three backends).
- **D-05:** `billable_kind` is a raw `TEXT` column with **no enum** at the DB or
  entity layer — matches `BillableKind(&'static str)` as an open set the crate never
  enumerates.
- **D-06:** Timestamps use SeaORM `timestamp_with_time_zone` (entity type
  `chrono::DateTime<Utc>`). NOT-NULL stamps (`reserved_at`, `expires_at`) are set in
  Rust at insert time — **no DB-level `DEFAULT now()`** (non-portable). Nullable
  stamps (`paid_at`, `released_at`, `refunded_at`) are set on transition.
- **D-07:** `metadata` uses SeaORM `ColumnType::Json`, nullable, entity type
  `serde_json::Value` (maps JSONB on PG, JSON on MySQL, TEXT on SQLite). Documented
  free-form, no PII.
- **D-08:** `tenant_id` and `billable_id` carry **no FK constraint** at the
  ferro-payments level (consumer tables unknown to the crate). Consumers add their
  own FKs.

### Lifecycle method semantics
- **D-09:** State-transition methods (`mark_paid`, `mark_released`, `mark_refunded`)
  are implemented as **atomic guarded UPDATEs** via `ferro_orm::GuardedUpdate`, with
  the required source status in the `WHERE` clause. A rows-affected count of `0` means
  the precondition was not met and the call is a **no-op** — this satisfies the
  design's "second writer no-ops" race semantics *by construction* (no read-then-write
  window).
- **D-10:** `create_reserved` is a plain INSERT; a second concurrent active row is
  rejected by the partial unique index (D-01/D-02), not by application logic.
- **D-11:** `find_active_for(kind, id)` filters `status IN ('reserved','paid')`;
  `find_by_stripe_session(session_id)` filters on the unique `stripe_session_id`.

### Crate manifest / dependency surface (scaffold)
- **D-12:** Phase 233 `ferro-payments` depends only on: `sea-orm`, `chrono`, `serde`,
  `serde_json`, `thiserror`, `async-trait`, and `ferro-orm` (for `GuardedUpdate`).
  **`ferro-stripe` is deliberately NOT a dependency yet** — no service/webhook code
  exists in 233, and an unused dependency would trip `clippy -D warnings`. It is added
  in phase 234 when `PaymentService` needs `CheckoutBuilder`/`Client`.
- **D-13:** A **minimal** `PaymentError` ships in 233 with only the variants the data
  layer needs: `Db(sea_orm::DbErr)`, `StatusPrecondition(String)`, `NotFound`. The
  `Stripe`, `Loader`, and `AutoRefundTriggered` variants are deferred to phase 234
  (they pull in `ferro-stripe` / loader types).
- **D-14:** Crate version `0.1.0`; `edition`/`license`/`repository` inherit from the
  workspace. Add `ferro-payments` to the root `Cargo.toml` members **and** to
  `.github/workflows/publish.yml` in a wave after `ferro-orm` (Wave 2+, since it
  depends on an internal crate).

### Claude's Discretion
- Exact module file split inside `src/` (the design doc proposes `billable.rs`,
  `loader.rs`, `intent/{mod,entity,status,lifecycle}.rs`, etc.) — planner may collapse
  the not-yet-needed modules (`service.rs`, `webhook.rs`, `reaper.rs`, `refund.rs`,
  `loader.rs`, `billable.rs`) into stubs or omit them in 233, keeping only what the
  data layer requires. Recommend shipping only `lib.rs`, `intent/` (entity, status,
  lifecycle), `migration/`, and `error.rs` in this phase.
- Exact name and SQL expression of the MySQL generated column (subject to D-02
  research confirmation).
- Whether `BillableKind` lives in a `billable.rs` stub now or is introduced minimally
  alongside the entity — both acceptable as long as the entity stores the raw string.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Authoritative spec
- `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md` — the full
  ferro-payments crate design: crate layout, public API, the `payment_intents` data
  model table (§ "Data model"), migration rules, error model, webhook race semantics,
  versioning/publication. **This is the source of truth — REQUIREMENTS.md has no
  PAY-POLY-DM entries; do not grep for them, read this doc.**
- Companion (external repo, reference only):
  `gestiscilo-it/app:docs/superpowers/specs/2026-06-17-tenant-booking-upfront-payment-design.md`
  — the first consumer's matching spec (column layout + race table originate here).

### Reusable primitives
- `ferro-orm/src/lib.rs` — `GuardedUpdate<E>` (atomic single-statement conditional
  UPDATE); the lifecycle precondition mechanism (D-09).
- `ferro-stripe/` (crate root + `Cargo.toml`, currently `0.9.0`) — sibling crate to
  mirror for layout, manifest conventions, `thiserror` error style, and feature flags.

### Migration / entity patterns (no partial-index precedent — see D-03)
- `app/src/migrations/m20260228_create_api_keys_table.rs` — SeaORM table + index
  (`Index::create()`) + `DeriveIden` enum pattern.
- `app/src/migrations/` (other `m2026*` files) — table-creation conventions.
- `benchmark/apps/ferro-conduit/src/models/*.rs` — `#[derive(DeriveEntityModel)]`
  entity patterns.

### Workspace conventions
- `CLAUDE.md` (project) — publish.yml wave rule (D-14), project-agnostic crate rule
  (no app-identity hardcoding — N/A for a pure data layer here), pre-commit gate
  (`fmt` + `clippy --all --all-targets -D warnings` + `test --all-features`).
- `Cargo.toml` (workspace root) — members list (add `ferro-payments`).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro_orm::GuardedUpdate` — drop-in for atomic status transitions (D-09); avoids
  read-modify-write races entirely.
- `ferro-stripe` crate — structural template for a new payment-domain crate (Cargo
  manifest shape, error module, lib re-exports).
- Existing SeaORM migrations in `app/src/migrations/` — copy the table/index/`DeriveIden`
  scaffolding; adapt for the raw partial-index SQL branch.

### Established Patterns
- Hand-rolled SeaORM migrations via the schema builder (no external migration tooling).
- One `thiserror`-derived error enum per crate (snake_case serde where serialized).
- `#[derive(DeriveEntityModel)]` for entities; `DeriveActiveEnum` for string-mapped
  status enums.

### Integration Points
- Root `Cargo.toml` `members` array.
- `.github/workflows/publish.yml` wave ordering (after `ferro-orm`).
- No `AppConfig`/app-identity surface needed — the crate is a pure data + migration
  layer in 233.

### Constraints / Net-New Risk
- **No partial/filtered unique index exists anywhere in the workspace.** This is
  net-new and the cross-backend MySQL path (D-02) is the highest-risk item — the
  researcher should validate it against the design doc's portability claim before the
  planner finalizes the migration.

</code_context>

<specifics>
## Specific Ideas

- The `payment_intents` column list is fixed by the design doc's "Data model" table —
  implement exactly those columns, types, and nullability.
- Tests run against **in-memory SQLite**; the SQLite partial-index path is the one
  exercised by CI. Postgres/MySQL paths are correct-by-construction + reviewed, not
  unit-tested in 233.

</specifics>

<deferred>
## Deferred Ideas

These came up via the design doc but belong to later phases — do not implement in 233:

- `PaymentService<L>`, `start_checkout`, `request_refund`, `release_expired` — phase 234.
- `Billable` trait, `BillableLoader` trait — phase 234.
- `wire_dispatcher` + `OnCheckoutCompleted/Expired/ChargeRefunded` handlers, idempotency
  via `ProcessedEventLog`, auto-refund fallback — phase 235.
- `ReleaseExpiredPaymentIntents`, `ReconcileRefundsInFlight` reapers + workspace test
  bin + publish `0.1.0` — phase 236.
- `ferro-stripe` dependency, full `PaymentError` variants (`Stripe`, `Loader`,
  `AutoRefundTriggered`) — phase 234.
- Design open questions (loader `tenant_id` signature, `wire_dispatcher` vs direct
  registration, `ReconcileRefundsInFlight` cadence, `Billable: Clone`) — all phase 234+.

None of these were requested for 233 — discussion stayed within the data-layer boundary.

</deferred>

---

*Phase: 233-ferro-payments-crate-polymorphic-billable*
*Context gathered: 2026-06-17*
