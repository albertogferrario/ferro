# Changelog

All notable changes to Ferro crates are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## ferro-audit

### [0.2.31] — 2026-05-13

Initial release. Phase 153 — `ferro-audit` crate (append-only structured
before/after audit log with replay-ready query helpers). Milestone v11.11.

#### Added

- New crate `ferro-audit` exposing the `AuditEntry::record(action).…write(&conn)`
  chainable builder — persists one row per state-changing operation to an
  `audit_log` table with typed actor, target, before/after JSON, reason,
  correlation id, and tenant scoping. The DB-stamped `created_at`
  (`DEFAULT CURRENT_TIMESTAMP`) is the single source of truth for ordering.
- `AuditActor` typed enum: `User(String) | System | Job(String) | ApiClient(String) | Anonymous`
  — stringly-keyed so the crate stays project-agnostic. `System` and `Anonymous`
  persist `actor_id = NULL`.
- `AuditTarget` struct: `kind: String, id: String` with `From<(K, I)>` tuple impl.
  Dotted-namespace convention (`"inventory.unit"`, `"checkout.session"`).
- `AuditError` — `MissingAction | Db(#[from] DbErr) | Json(#[from] serde_json::Error)`.
  Display prefix `"audit: …"`.
- Query helpers `history_for_target` (ASC, indexed), `recent_by_actor` (DESC, limited,
  indexed), `recent` (DESC, limited, global).
- `reconstruct_state(&[AuditEntry])` — pure shallow-merge fold of `after` payloads into
  the final state. The "replay" primitive in the phase title.
- `prune_older_than(cutoff, &conn)` — caller-driven retention helper returning the deleted
  row count. Strict less-than (`created_at < cutoff`); preserves rows at the cutoff.
- `CreateAuditLogTable` migration — consumers register it in their `Migrator`. Schema:
  12 columns + 2 composite indexes (`idx_audit_target`, `idx_audit_actor`).
- Targeted re-exports of the SeaORM symbols required by the public API; no blanket
  `pub use sea_orm::*`. The `AuditLogEntity` re-export enables consumer-side sea-orm-native
  queries (pagination, custom filters).
- Workspace member registered in `Cargo.toml`; auto-publish Wave 1a slot reserved in
  `.github/workflows/publish.yml`. First publish bootstrapped from a local terminal
  (CI publish token has `publish-update` scope only); subsequent versions auto-publish.
- New documentation page `docs/src/database/audit-log.md` covering the anti-pattern,
  the API, AuditActor / AuditTarget shape, schema + indexes, replay semantics (shallow
  merge), retention and GDPR considerations, and the error variants.

## ferro-orm

### [0.2.30] — 2026-05-13

Initial release. Phase 152 — `ferro-orm` crate (atomic conditional UPDATE
primitive for race-free counter mutations and state transitions).
Milestone v11.11.

#### Added

- New crate `ferro-orm` exposing the `GuardedUpdate<E>` builder — compiles
  to a single `UPDATE … WHERE …` SQL statement, replacing the hand-rolled
  `read → check → write` pattern wherever a column's value is conditionally
  mutated. The database engine's per-statement atomicity (SQLite serial
  writer, Postgres `READ COMMITTED`) is the correctness mechanism;
  `GuardedUpdate` adds the chainable surface and the rows-affected →
  `GuardedError` mapping on top.
- `GuardedUpdate::filter(impl IntoCondition)` — AND-combines multiple
  filter calls onto an internal `Condition`. Matches `sea_orm::QueryFilter`
  ergonomics.
- `GuardedUpdate::set_expr(col, SimpleExpr)` and `set_value(col, Value)` —
  chainable per-column set, supports value-derived (`Expr::col(…).sub(…)`)
  and literal (`Value::String(…)`) assignments in the same statement.
- `GuardedUpdate::exec_one(&conn)` — succeeds iff exactly one row matched;
  `0 → Err(NoRowsAffected)`, `>1 → Err(TooManyRows { affected })`. Default
  for race-free counter mutations.
- `GuardedUpdate::exec_at_most_one(&conn)` — `Ok(true)` on 1 row,
  `Ok(false)` on 0 rows (predicate failure is a normal outcome),
  `Err(TooManyRows)` on >1 rows. For optimistic updates.
- `GuardedError` — `NoRowsAffected | TooManyRows { affected } |
  EmptyUpdate | Db(#[from] DbErr)`. Display prefix `"guarded: …"`.
- Targeted re-exports of the SeaORM symbols required by the public API
  (`EntityTrait`, `ColumnTrait`, `ConnectionTrait`, `IntoCondition`,
  `SimpleExpr`, `Value`, `DbErr`, `Expr`); no blanket `pub use sea_orm::*`.
- Workspace member registered in `Cargo.toml`; auto-publish Wave 1a slot
  reserved in `.github/workflows/publish.yml`. First publish is
  bootstrapped from a local terminal (CI publish token has
  `publish-update` scope only); subsequent versions auto-publish.
- New documentation page `docs/src/database/atomic-updates.md` covering
  the anti-pattern, the API, common patterns (counter decrement, status
  transition, optimistic concurrency), and the per-statement atomicity
  contract.

## ferro-wallet

### [0.2.24] — 2026-05-11

Initial release. Phase 151 — `ferro-wallet` crate (Apple `.pkpass` +
Google Wallet save-link issuance). Milestone v11.10.

#### Added

- New crate `ferro-wallet` exposing the `WalletSubject` trait,
  `ApplePassBuilder` (PKCS#7-signed `.pkpass` ZIP via `openssl` + `zip` +
  `sha1`), and `GoogleWalletBuilder` (RS256-signed save JWT via
  `jsonwebtoken`, returning a `pay.google.com/gp/v/save/{jwt}` URL).
- `WalletConfig::from_env` reads `APP_NAME` / `APP_URL` and optional
  Apple / Google clusters; missing wallet env vars never error (D-02).
  Mirrors `ferro-inertia::InertiaConfig::app_name` and
  `ferro-stripe::StripeConfig::from_env` (architecture principle #6 —
  project-agnostic crates).
- `images` module — `fit_to` (resize-preserve-aspect + centre-pad onto
  transparent canvas), `apple_logo_set` (160×50 / 320×100 / 480×150),
  `apple_icon_set` (29×29 / 58×58 / 87×87, derivable from logo when icon
  absent), `google_hero` (1032×336).
- `qr` module — PNG bytes + `data:image/png;base64,…` data-URI helpers
  via `qrcode-generator`.
- End-to-end integration tests mint crypto material at runtime — no real
  Apple WWDR or Google service-account credentials in CI (D-09).
- Workspace member registered in `Cargo.toml`; auto-publish Wave 1a slot
  reserved in `.github/workflows/publish.yml`. First publish is
  bootstrapped from a local terminal (CI publish token has
  `publish-update` scope only); subsequent versions auto-publish.

## ferro-rs

### [0.2.13] — 2026-04-21

Bug fix: `get!("/", ...)` registered inside `group!("/prefix", { ... })`
is now reachable at both `/prefix` and `/prefix/`. Previously only
non-root paths matched; the trailing-slash variant of the root-in-group
case returned 404. Discovered via a production field application that
routes under `/s/{slug}/`.

#### Fixed

- Group path combination in both `GroupDef::register_with_inherited`
  (the macro-based `group!`) and `GroupBuilder::finalize` (the
  builder-based `Router::group`) now registers a leaf `get!("/", ...)`
  under both `/prefix` and `/prefix/`. A trailing slash on the group
  prefix is also correctly stripped, so `group!("/api/", { get!("/x", ...) })`
  produces `/api/x`, not `/api//x`.
- Nested-group prefix accumulation strips a trailing slash on the
  parent prefix before concatenating the child prefix, so
  `group!("/a/", { group!("/b", { get!("/", h) }) })` accumulates to
  `/a/b` rather than `/a//b`.

#### Unchanged

- Top-level (non-grouped) `get!("/", ...)` behavior.
- Route introspection: `get_registered_routes()` and
  `ferro-mcp list_routes` still show one entry per logical handler —
  the canonical path without trailing slash.
- Named-route resolution: `route("foo", &[])` returns the canonical
  path.
- Middleware attached to grouped routes fires for both trailing-slash
  variants.

## ferro-stripe

### [0.4.0] — 2026-04-20

Capability-axis refactor. The crate is reorganized around Stripe capabilities
(checkout, refund, account, idempotency, webhook) rather than Stripe products
(connect, subscription). Consumer-facing symbols change significantly; see
the migration table below.

#### Added

- `checkout::CheckoutBuilder` — consuming builder for Stripe Checkout Sessions
  covering both `Payment` and `Subscription` modes, Connect destination charges,
  metadata, and required idempotency keys.
- `checkout::CheckoutIntent` — typed return from `CheckoutBuilder::create()`
  carrying `session_id`, `url`, `expires_at`, `idempotency_key`.
- `checkout::Mode` — `Payment` | `Subscription`.
- `checkout::LineItem` — typed line-item input for `CheckoutBuilder`.
- `refund::create(charge_id, amount_cents, idempotency_key, reason)` and
  `refund::retrieve(refund_id)` — first-class refund surface.
- `account::create_account`, `account::retrieve_account` — new Connect account
  operations (complementing the existing `create_link` and `billing_portal_url`
  which moved to `account::` unchanged).
- `idempotency::ProcessedEventLog` — async trait for deduplicating Stripe
  webhook events on `event_id`. Apps ship a DB-backed impl; the recommended
  SQL schema is in the module doc.
- `idempotency::MemoryProcessedLog` — in-memory reference implementation
  backed by `DashMap`, intended for tests and single-process development.
- `client::Stripe::with(api_key)` — returns a scoped `stripe::Client` without
  touching the global static. Use for per-tenant direct-charges scenarios.
- `webhook::sync` and `webhook::queue` — empty modules reserving the file
  locations for Phase 141's `SyncDispatcher` and queue-path relocation.
- `webhook::verify::verify_webhook` — the HMAC-verification fn moved out of
  `webhook/mod.rs` into a dedicated submodule. Public behavior unchanged.
- `Error::MissingIdempotencyKey` — returned by `CheckoutBuilder::create()`
  when `.idempotency_key()` was not called before `.create()`.

#### Removed (breaking)

| Removed symbol | Replacement | Notes |
|---|---|---|
| `webhook::is_processed` (and `lib` re-export) | `idempotency::ProcessedEventLog::try_mark_processed` | The stub was never correct; apps must implement the trait against their DB. |
| `connect::checkout::create_connect_checkout` | `CheckoutBuilder::new(Mode::Payment).destination(...).create()` | Destination charge is now explicit on the builder. |
| `subscription::checkout::create_subscription_checkout` | `CheckoutBuilder::new(Mode::Subscription).create()` | Single checkout entry point. |
| `connect::checkout::create_account_link` | `account::create_link` | Same signature; moved path. |
| `subscription::checkout::billing_portal_url` | `account::billing_portal_url` | Same signature; moved path. |
| `subscription::sync::plan_from_subscription` | (app responsibility) | Mapping from `stripe::Subscription` to plan name is app logic. |
| `subscription::sync::subscription_info_from_stripe` | (app responsibility) | Ditto. |
| `subscription::SubscriptionInfo` | `framework::tenant::subscription::SubscriptionInfo` (within this workspace) or app-local type (external consumers) | Type was app state, not a Stripe-API wrapper. |
| `subscription::SubscriptionStatus` | `framework::tenant::subscription::SubscriptionStatus` | See above. |
| `subscription::plan_satisfies` | `framework::tenant::subscription::plan_satisfies` (within workspace) or app-local 5-line fn | Plan-hierarchy logic is app concern, not Stripe. |
| `connect::ConnectAccount` | Use Stripe account ID as `String` directly | The wrapper added nothing. |
| `webhook::handler::handle_platform_webhook` / `handle_connect_webhook` | Phase 141 will provide `SyncDispatcher`-based replacements. For Phase 140, consumers should call `verify_webhook` directly and dispatch `ProcessStripeWebhook` manually via `ferro_queue::dispatch`. | Temporary gap; narrow window since the queue path is being reshaped in Phase 141 anyway. |

#### Changed (breaking)

- Module layout: `connect/` and `subscription/` directories are gone. Modules
  now reflect capabilities: `checkout`, `refund`, `account`, `idempotency`,
  `webhook`. Imports must be updated accordingly.
- `CheckoutBuilder::create()` returns `Err(Error::MissingIdempotencyKey)` when
  `.idempotency_key()` was not set. This is a runtime check, not a typestate
  (chosen for simplicity; typestate may be revisited pre-1.0).

#### Unchanged

- `Stripe::init` static facade and global client pattern.
- `StripeConfig::from_env()` and all environment-variable names.
- `verify_webhook` signature (`raw_body`, `signature`, `secret`) — only the
  path changed (from `webhook::verify_webhook` to `webhook::verify::verify_webhook`;
  `ferro_stripe::verify_webhook` still works via re-export).
- The five webhook event structs (`StripeCheckoutCompleted`, `StripeSubscriptionUpdated`,
  `StripeSubscriptionDeleted`, `StripeInvoicePaid`, `StripeConnectPaymentSucceeded`)
  keep their current shape. Phase 141 drops the `event_json: String` field.
- `testing::signed_webhook_payload` (location unchanged).

#### Migration guide

Replace old call sites mechanically:

```rust
// Before
let url = ferro_stripe::create_connect_checkout(
    &account_id, 1000, "usd", success, cancel, Some(100),
).await?;

// After
let intent = ferro_stripe::CheckoutBuilder::new(ferro_stripe::Mode::Payment)
    .destination(&account_id, Some(100))
    .line_item(ferro_stripe::LineItem {
        name: "Payment".into(),
        description: None,
        unit_amount_cents: 1000,
        quantity: 1,
        currency: "usd".into(),
    })
    .success_url(success)
    .cancel_url(cancel)
    .idempotency_key(&order_idempotency_key)
    .create()
    .await?;
let url = intent.url;
```

```rust
// Before
if ferro_stripe::is_processed(&event.id) { return Ok(()); }

// After
if !self.log.try_mark_processed(&event.id).await? {
    // Already processed — skip side effects.
    return Ok(());
}
// where `self.log: Arc<dyn ProcessedEventLog>` is injected by the app.
```

See the crate-level doc on `ferro-stripe` for full examples.
