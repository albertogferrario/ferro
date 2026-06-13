# Changelog

All notable changes to Ferro crates are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased] — ferro-macros / framework (Phase 212 — CRUD Handler Proc Macros)

### Added

- `#[resource_get]` / `#[resource_post]` route-attribute proc macros: fold the
  tenant-resolve + typed-param + tenant-scoped-lookup + 404-on-miss prelude into a
  single attribute while `tenant` and the resource stay real typed parameters; the
  user body moves to a named inner fn (IDE jump-to-def preserved).
- `TenantScoped` trait (`type Id: FromStr`, async `find_for_tenant(id, tenant_id)`):
  the lookup contract the macros call; tenant-scoped by construction.
- `Validator::validate_or_redirect(url)`: composes the existing
  `with_old_input` + `into_action_error` chain into the validator's `?` flow
  (uses the data already held by the validator — no separate `&data` argument).

---

## [Unreleased] — ferro-json-ui / ferro-mcp / ferro-cli (Phases 162–163)

Changes landed on `v12.0/json-ui-v2`. Accumulated through Phases 162–164; single publish at Phase 161 (merge to master).

### Added

- `CheckboxList` component for multi-select checkbox groups with static or data-driven options (162-01, D-01/D-02). Props: `field`, `options`, `options_path`, `selected_path`, `label`, `description`, `disabled`, `error`. Renders one `<input type="checkbox">` per option; all strings HTML-escaped.
- `SwitchProps.compact: Option<bool>` for compact inline switch display via `scale-75 origin-left` CSS toggle (162-03, D-16).
- `ImageProps.inline_svg: Option<String>` and `ImageProps::inline_svg(svg, alt)` factory for server-constructed inline SVG; emits an aria-labelled wrapper div in place of `<img>` (162-03, D-17).
- `RichTextEditor` plugin backed by Quill 2.0.3, registered via the `JsonUiPlugin` surface (`ferro-json-ui/src/plugins/rich_text_editor.rs`). Two consumer sites in gestiscilo documenti templates unblocked (162-04, D-18). Note: Quill CDN SRI hashes are TODO — tracked in `162-DEFERRED.md`.
- `SpecError::FooterMissing { element_id, footer_id }` variant; spec validator (`validate_footer_ids`) now rejects footer references to unknown element IDs at parse time via `SpecBuilder::build()` (162-07, D-07).
- `json_ui_verify_action` MCP tool — accepts `{ handler, method? }`, returns `RouteInfo` on exact match or the closest Levenshtein candidate on miss. Reads route names from the existing registry; no second source of truth (162-09, D-09).
- `strum::AsRefStr` derive on `AlertVariant`, `BadgeVariant`, `ButtonVariant`, `ToastVariant` (`component.rs`) and `DialogVariant`, `NotifyVariant` (`action.rs`) — typed enums round-trip to their snake_case wire string via `.as_ref()` without changing the JSON wire format (162-08, D-11).
- `docs/src/json-ui/migration-v1-to-v2.md` — 493-line v1→v2 migration guide with 7 worked-example sections covering `render_file`, Card+Form+Alert depth-flattening, DataTable interpolation, read/edit detail pattern, CheckboxList, variant strum round-trip, and `json_ui_verify_action` (162-10, D-20).
- `docs/src/json-ui/plugins.md` — JsonUiPlugin authoring guide extended with a RichTextEditor built-in plugin section and catalog discoverability notes (162-10, D-19).
- 7 `migration_v1_to_v2` `code_templates` entries in `ferro-mcp/src/tools/code_templates.rs`, surfaced via the `code_templates` MCP tool — one template per migration guide section (162-10, D-22).

### Changed

- DataTable `row_actions[i].action.url` now interpolates any column key from the row at render time (`{label}`, `{slug_path}`, `{status}`, …) in addition to the legacy `{row_key}` and `{id}` (162-02, D-03/D-04). Missing-key placeholders pass through unchanged. Wire format unchanged.
- Spec validator emits a stderr warning when an element ID appears in both `props.footer` and `children` of the same parent (162-07, D-08). Non-fatal; no new crate dependency.
- `ferro-json-ui/src/catalog.rs` and `ferro-mcp/src/tools/json_ui_catalog.rs` updated to reflect the post-Wave-1 catalog shape: 40 built-in components (including CheckboxList) and 2 plugin components (Map + RichTextEditor) (162-01, 162-04, 162-05, D-21).
- `docs/src/json-ui/components.md` gains a v1→v2 migration banner linking to the new guide, a worked `Card` + flat `children` example, and an "Inline view/edit" section (162-10, D-13, D-14, D-15).

### Removed

- Auth layout (`ferro-json-ui/src/layout.rs`) no longer wraps content in `bg-card rounded-lg shadow-md p-8`. Specs using `layout: "auth"` must declare their own `Card` root for card chrome (162-06, D-05). Breaking for any spec that relied on the implicit wrapper; gestiscilo audit confirmed all auth specs already declare Card roots.

### Notes

- D-06 (`Fragment`/`Group` borderless container): NOT added. D-05 resolves the double-card friction.
- D-10 (`#[handler(name)]` attribute): NOT added. Route names are registered at `route!`/`get!`/`post!` macro call sites; the new MCP tool reads from that single source of truth.
- D-23, D-24: Phase 162 does not publish to crates.io and does not bump the workspace version (still 0.2.35). The single publish for v12.0 happens at Phase 161 (merge `v12.0/json-ui-v2` → master).

---

## [Unreleased] — ferro-json-ui / ferro-mcp / ferro-cli (Phase 163)

Changes landed on `v12.0/json-ui-v2`. No version bump; single publish at Phase 161.

### Added

- `ferro-json-ui`: `$each` element-level iteration directive. Templated elements expand at resolve time into N clones with auto-suffixed IDs `{id}-0`..`{id}-(N-1)`. Loop variable bound by `as` scopes `$data` paths starting with `/{as}/...` to the current row. Sibling templates sharing the same `{path, as}` produce correlated indexes; mismatched-each siblings are rejected at validation.
- `ferro-json-ui`: `$if` element-level conditional emission directive. Falsy predicates remove the element from the spec at resolve time (distinct from `visible` which renders hidden DOM). Reuses the existing `Visibility` enum, including `and`/`or`/`not` composition.
- `ferro-json-ui`: `expand_directives` public function in `ferro_json_ui::resolve`. Runs before `resolve_actions` in the resolve pipeline.
- `ferro-json-ui`: Five new `SpecError` variants for directive validation — `EachPathNotArray`, `IfPathMissing`, `EachAsReservedName`, `NestedEach`, `MismatchedEach`. Validation fires at `Spec::from_json` time (best-effort against `spec.data` when non-null).
- `ferro-json-ui`: `SpecBuilder::element_nested` and `NestedElement` builder type. Ergonomic nested-tree construction for cases where neither static JSON nor `$each` nor `$if` express the element graph. Child IDs auto-generated by structural position; the runtime `Spec` shape is unchanged.
- `ferro-cli`: `json-ui:migrate-v1` subcommand. AST-based codemod that converts v1 controller files (`make_node` + `JsonUiView::new`) to v2 (flat JSON spec + `JsonUi::render_file`). Single file per invocation; idempotent; `--dry-run` flag for preview; cases that cannot be auto-translated produce a `// TODO: ferro json-ui:migrate-v1 could not auto-translate this handler` marker.
- `ferro-mcp`: `json_ui_catalog` tool output now includes a `directives` field listing the `$each` and `$if` directives with name, description, syntax example, and validation-error references.
- `docs`: New `docs/src/json-ui/spec-construction.md` with the four-quadrant decision rubric (static / `$each` / `$if` / `SpecBuilder`). `docs/src/json-ui/expressions.md` extended with `$each` and `$if` sections.

### Changed

- `ferro-json-ui`: `Element` struct gains two optional fields — `each: Option<EachDirective>` (serde-renamed `$each`) and `if_: Option<Visibility>` (serde-renamed `$if`). Both fields use `skip_serializing_if = Option::is_none`; existing specs without directives serialize identically.
- `framework`: `JsonUi::resolve` pipeline runs `expand_directives` before `resolve_actions` and `resolve_expressions`. Specs without directives produce identical output to pre-163 behavior.

## ferro-projection

### [0.2.33] — 2026-05-14

Initial release. Phase 155 — `ferro-projection` crate (live read-model
runtime: subscribe to domain events, persist per-key snapshots,
broadcast deltas).

**Not the same as `ferro-projections` (plural).** That crate is the
Service Projection abstraction (`ServiceDef → IntentGraph →
JsonUiRenderer`). `ferro-projection` (singular) is the live read-model
runtime described above. The two abstractions are orthogonal.

#### Added

- New crate `ferro-projection` exposing the `Projection` trait for
  consumer-implemented live read-models. Associated types `Event`
  (`ferro_events::Event + Serialize + DeserializeOwned`), `State`
  (`Clone + Default + Serialize + DeserializeOwned + Send + Sync +
  'static`), `Delta` (`Serialize + Clone + Send + Sync + 'static`).
  Const `NAME: &'static str` for the projection's dotted-namespace
  identifier. Sync `apply(&self, state: &mut State, event: &Event)
  -> Delta` (pure fold; runs inside per-key Mutex). Defaulted
  `snapshot_interval()` (returns 100) and `broadcast_event_name()`
  (returns `"delta"`).
- `ProjectionRuntime<P: Projection>` orchestrator owning the
  database connection, the broadcaster handle, the projection impl,
  and the per-key Mutex registry. Two entry points: `register(self:
  Arc<Self>)` wires a `ProjectionListener<P>` into
  `ferro_events::global_dispatcher` (one-line wiring), and
  `apply_event(&self, event: &P::Event)` is the manual entry point
  for tests, replay scripts, or custom dispatchers.
- `read(&self, key) -> Result<Option<State>, ProjectionError>` and
  `read_required(&self, key) -> Result<State, ProjectionError>`
  (returns `StateNotFound` on miss). Read path does NOT acquire the
  per-key Mutex.
- `rebuild(&self, key, events: impl IntoIterator<Item = P::Event>)
  -> Result<State, ProjectionError>` discards the persisted snapshot,
  folds the supplied event sequence through `State::default()`,
  persists the final state, and broadcasts ONE `"rebuild"` frame
  carrying the full final state. Empty iterator wipes the row.
- Per-key in-process serialization via
  `DashMap<String, Arc<tokio::sync::Mutex<()>>>` — each key gets its
  own Mutex; same-key applies serialize, different-key applies run
  in parallel. The shard-lock-drop-before-await pattern is the
  correctness mechanism.
- Snapshot persistence via SeaORM `OnConflict::columns([projection_name,
  key]).update_columns([state, version, updated_at])` upsert on the
  composite primary key. Schema: `projection_snapshots` table with 5
  columns + composite PK on `(projection_name, key)`.
- Delta broadcast on `projection.{NAME}.{key}` channels via
  `ferro_broadcast::Broadcast::new(...).channel(...).event(...).data(delta).send()`.
  Event name defaults to `"delta"` (consumer can override). Broadcast
  failure does NOT roll back the persisted state — log at
  `tracing::warn!` and surface `ProjectionError::Broadcast`.
- `ProjectionError` — `Db(#[from] sea_orm::DbErr) | Json(#[from]
  serde_json::Error) | Broadcast(String) | Events(String) |
  StateNotFound { name, key }`. Display prefix `"projection: …"`.
  Hand-rolled `From<ferro_broadcast::Error>` and
  `From<ferro_events::Error>` impls (Phase 149 precedent for
  stringly-typed variants).
- `ProjectionKey` opaque newtype around `String` with `new`, `as_str`,
  `Display`, `From<String>`, `From<&str>`, serde Serialize +
  Deserialize. Multi-tenancy lives inside the key string by convention.
- `CreateProjectionSnapshotsTable` migration — consumers register it
  in their `Migrator` alongside other ferro-* crates' migrations.
- Public SeaORM re-exports: `ProjectionSnapshotEntity`,
  `ProjectionSnapshotModel`, `ProjectionSnapshotActiveModel` —
  consumer-side queries against `projection_snapshots` use these.
- Workspace member registered in `Cargo.toml`; auto-publish Wave 1b
  slot reserved in `.github/workflows/publish.yml`. First publish
  bootstrapped from a local terminal (CI publish token has
  `publish-update` scope only); subsequent versions auto-publish.
- New documentation page `docs/src/features/live-read-models.md`
  covering the disambiguation from `ferro-projections` plural, the
  anti-pattern, the typed-runtime replacement, the trait surface, the
  two entry points (register + apply_event), the read + rebuild
  paths, the broadcast channel contract, operational footguns (3),
  and a worked example folding `ferro_reservation::ReservationEvent`
  into per-`resource_kind` counters.

v11.11 Resource Reservation & Live Read-Model Primitives complete — ferro-orm GuardedUpdate (Phase 152), ferro-audit (Phase 153), ferro-reservation (Phase 154), ferro-projection (Phase 155) now all shipped.

## ferro-reservation

### [0.2.32] — 2026-05-13

Initial release. Phase 154 — `ferro-reservation` crate (generic
hold/commit/release resource reservation kernel with TTL and event
broadcast).

#### Added

- New crate `ferro-reservation` exposing `ReservationKernel<R: Resource>`
  with `hold` / `commit` / `release` / `extend` / `run_sweep_once` — a
  typed, race-free state-transition pipeline for any capacity-constrained
  resource. Consumers implement the `Resource` trait against their own
  domain model.
- `Resource` trait: consumer-implemented capacity model. Associated
  types `Key` (Hash + Eq + Clone + Send + Sync + Serialize + DeserializeOwned)
  and `Window` (PartialEq + Clone + Send + Sync + Serialize +
  DeserializeOwned; use `()` for non-windowed resources). Const
  `KIND: &'static str` for dotted-namespace identification
  (`"inventory.unit"`, `"checkout.slot"`, `"api.quota"`). Two async
  methods `capacity` and `held` generic over `<C: ConnectionTrait>`.
- `ReservationKernel<R>` with `new(db, resource)` constructor and four
  state-transition methods. State machine: `held → committed | released
  | expired`. Terminal states have no outgoing transitions; any attempt
  surfaces as `ReservationError::ConflictingState`.
- `ReservationContext` per-call audit metadata bundle: `actor`,
  `correlation_id`, `tenant_id`, `reason`. Four constructors
  (`system`, `user`, `job`, `anonymous`) and three consuming
  builder methods (`with_correlation`, `with_tenant`, `with_reason`).
- `ReservationHandle` opaque token — full snapshot of hold-time fields
  with `Serialize + Deserialize` for embedding in Stripe payment intent
  metadata, queued-job payloads, or other side channels.
- `ReservationEvent` enum (`Held | Committed | Released | Expired`)
  implementing `ferro_events::Event` — dispatched via
  `ferro_events::dispatch` AFTER every successful state transition.
  Event dispatch is best-effort; failure logs at `tracing::warn!` and
  does NOT roll back the DB state.
- `ReleaseReason` enum (`UserCancelled | PaymentFailed | AdminOverride
  | Other(String)`) — typed reason recorded on the audit log and emitted
  with `ReservationEvent::Released`.
- `SweepReport` returned from `run_sweep_once` (`expired_count`,
  `scanned_at`) for sweep observability.
- `ReservationError` — `Insufficient { requested, available, capacity }
  | ConflictingState { id, expected } | NotFound { id } | Db(#[from] DbErr)
  | Guarded(#[from] GuardedError) | Audit(#[from] AuditError) | Json(#[from]
  serde_json::Error)`. Display prefix `"reservation: …"`.
- Unconditional audit emission via `ferro-audit`: every successful
  state transition writes one `AuditEntry` with
  `action = "reservation.{held|committed|released|expired|extended}"`.
  Audit failure surfaces as `ReservationError::Audit` but does NOT
  roll back the DB state.
- Race-free state transitions via `ferro-orm::GuardedUpdate`. Every
  transition predicate includes `Status.eq("held")`; concurrent
  callers surface as `ConflictingState`. The sweeper uses
  `exec_at_most_one` so concurrent sweepers tolerate 0-rows-affected
  as a normal outcome.
- `run_sweep_once()` — sweeper primitive. Scans for held rows with
  `expires_at < now`, transitions to expired (LIMIT 500 per call), emits
  one `ReservationEvent::Expired` + one `AuditEntry` per row with
  `AuditActor::System`. Consumers schedule sweeps themselves (no
  `ferro-queue` runtime dependency); three idiomatic patterns
  documented (`ferro-queue` Job, `tokio::time::interval`, cron CLI).
- `CreateReservationsTable` migration — consumers register it in their
  `Migrator` alongside `ferro_audit::CreateAuditLogTable`. Schema: 12
  columns + 2 composite indexes (`idx_reservations_kind_key_window_status`,
  `idx_reservations_status_expires`).
- Targeted re-exports of the SeaORM symbols required by the public API
  (no blanket `pub use sea_orm::*`). The `ReservationEntity` /
  `ReservationModel` / `ReservationActiveModel` re-exports enable
  consumer-side sea-orm-native queries. The `AuditActor` re-export
  from `ferro-audit` lets consumers build `ReservationContext` without
  a direct ferro-audit dependency for the common case.
- Workspace member registered in `Cargo.toml`; auto-publish Wave 1b
  slot reserved in `.github/workflows/publish.yml`. First publish
  bootstrapped from a local terminal (CI publish token has
  `publish-update` scope only); subsequent versions auto-publish.
- New documentation page `docs/src/database/reservations.md` covering
  the resource/window abstraction, defining a `Resource` impl, kernel
  construction, the four lifecycle methods, TTL + sweeper (three
  scheduling idioms), event subscription pattern, audit log inspection,
  common patterns (slot hold during checkout, ticket reservations, API
  rate-limit buckets), consistency model (per-statement atomicity,
  SQLite-validated; Postgres correctness for hold() deferred), and
  operational footguns.

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

### [0.2.34] — 2026-05-14

**Phase 156 — frontend/src/types/ generator-owned convention cleanup.**

- Reconciled the `frontend/src/types/` generator-owned convention end-to-end.
- Untracked `app/frontend/src/types/{inertia-props,routes}.ts` in the reference app; gitignore template now carries a load-bearing comment.
- Fixed the `generate_types.rs` emitted header comment that pointed to `frontend/src/types/` (should be `frontend/src/lib/types/` for hand-written types).
- Added `ferro doctor` check `frontend_types_convention` (advisory; flags hand-written files under the generator-owned directory).
- Dockerfile renderer now emits a `types-gen` Rust-toolchain stage before the frontend-builder stage so `frontend/src/types/` is regenerated inside the Docker build context. The frontend stage now `COPY --from=types-gen` the generated files in before `npm run build`.
- `DockerContext` gained a `ferro_version: String` field; new `resolve_ferro_version` helper parses the project's `Cargo.lock` for the `ferro-rs` package version with `env!("CARGO_PKG_VERSION")` as a fallback.
- New docs page `docs/src/cli/frontend-types.md` documents the convention end-to-end, including the `ferro docker:init --force` upgrade path for existing scaffolded projects.
- `ferro doctor` check count: 10 -> 11.

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
