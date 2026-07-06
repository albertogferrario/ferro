# Phase 153: ferro-audit — Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults applied to every gray area)
**Milestone:** v11.11 Resource Reservation & Live Read-Model Primitives
**Driver:** gestiscilo-it inventory monitoring field test
**Killer feature (milestone):** Race-free reservations as a first-class framework primitive. Phase 153 is the audit-trail primitive that any state-changing operation can attach to — and that Phase 154 (`ferro-reservation`) uses to log every commit / release / expire.

<domain>
## Phase Boundary

Create a new `ferro-audit` crate inside the ferro workspace that ships an **append-only, structured before/after audit log** with replay-ready query helpers.

The crate is intentionally narrow at v0:

- One row in `audit_log` per state-changing operation, with `before` / `after` JSON payloads
- Typed `AuditActor` / `AuditTarget` abstractions that don't bind ferro to a consumer's domain model
- A `SeaORM` migration that consumers add to their `Migrator`
- A small set of query helpers: `history_for_target`, `recent_by_actor`, `prune_older_than`
- A `reconstruct_state` helper that folds the recorded `before → after` diffs back into the current state (the "replay" promised in the phase title)

The crate delivers ONE foundational primitive that downstream phases (154 reservation; gestiscilo-it inventory + checkout) depend on. It does NOT prescribe what consumers log, when they log it, or how they correlate audit entries to requests — those are call-site decisions.

**In scope:** crate scaffold, public API (`AuditEntry::record(...).write(&conn).await`), `AuditActor` / `AuditTarget` enums (stringly-keyed so ferro stays domain-agnostic), `AuditError`, SeaORM migration with the schema + indexes from `INVENTORY-PRIMITIVES.md` §`ferro-audit`, query helpers, `reconstruct_state` helper, in-memory SQLite tests, rustdoc, one user-facing doc page, workspace version bump + auto-publish.

**Out of scope (deferred):** `audit_log!` macro façade, automatic correlation-id pickup from task-local / tracing span, ferro-events emission on write, distributed audit-stream / log shipping, MCP tools to query the audit log from agents, Postgres CI integration tests, retention policy enforcement (caller-driven), audit redaction / PII filtering.
</domain>

<decisions>
## Implementation Decisions

### Crate placement & scope

- **D-01:** Ship as a new top-level workspace crate at `ferro-audit/` — mirrors Phase 152's `ferro-orm/` placement. The roadmap explicitly names `ferro-audit`; downstream apps will import it as `use ferro_audit::AuditEntry;`. Adding it inside `framework` would force every consumer to depend on the full framework crate for a primitive that is independently useful.
- **D-02:** Crate is thin and additive at v0. It owns ONE table (`audit_log`), one entry type, and the matching query helpers. It does NOT subsume request-tracing, structured logging, or observability — those are existing crates / ecosystem choices the consumer makes (`tracing`, `tracing-subscriber`). `ferro-audit` is for *domain* state changes, not application logs.
- **D-03:** No internal ferro-* dependencies. `ferro-audit` depends on `sea-orm` and `sea-orm-migration` directly, NOT on `ferro-orm`. Even though `INVENTORY-PRIMITIVES.md` §`Cross-crate relationships` says `ferro-audit uses ferro-orm (audit_log table)`, that phrasing means "uses the ORM layer" generically. `ferro-orm v0` exposes only `GuardedUpdate`, which `ferro-audit` does not need (the audit log is append-only — every write is an INSERT, never a guarded conditional UPDATE). Depending on `sea-orm` directly keeps `ferro-audit` a Wave 1a leaf crate and lets phases 152 and 153 ship truly in parallel without serialization on `ferro-orm`'s publish.
- **D-04:** Wave 1a publish (zero internal ferro-* deps). External deps: `sea-orm` (1.0, workspace version), `sea-orm-migration` (1.0), `thiserror` (2), `serde` + `serde_json` (workspace versions), `uuid` (workspace version, with `serde` + `v4` features), `chrono` (workspace version, with `serde`). Add to `.github/workflows/publish.yml` Wave 1a alongside `ferro-orm` and `ferro-wallet`. New-crate-first-publish bootstrap from local terminal (CI token has publish-update only — see `project_ferro_publish_token_scoping.md`).

### Actor model

- **D-05:** `AuditActor` is a typed enum with stringly-keyed variants — ferro must not bind to a consumer's user-table primary-key type:
  ```rust
  pub enum AuditActor {
      User(String),         // user_id rendered as a string (consumer choice: i64.to_string(), Uuid, slug, …)
      System,                // background process with no specific identity
      Job(String),           // queued job name, e.g. "stripe.webhook.subscription_updated"
      ApiClient(String),     // API key / OAuth client id
      Anonymous,             // unauthenticated public action (rare but valid)
  }
  ```
  The DB representation is `(actor_kind: String, actor_id: Option<String>)` — `System` and `Anonymous` write `NULL` to `actor_id`. `actor_kind` is the lowercase enum variant: `"user" | "system" | "job" | "api_client" | "anonymous"`. Serde derives use `#[serde(rename_all = "snake_case")]` and tag-on-`kind` so the wire format matches the DB columns exactly.
- **D-06:** No "current-actor pickup from request" abstraction in v0. Consumers pass `AuditActor` explicitly to every `record(...)` call. A future `from_request(&Request)` helper or task-local pickup is plausible but adds dep weight on `framework` and creates a re-export trap. Out of scope for Phase 153.

### Target model

- **D-07:** `AuditTarget` is a struct, not an enum — the target domain is open-ended (any inventory unit, any user record, any document, any subscription) and a closed enum would force every consumer to upstream variants:
  ```rust
  pub struct AuditTarget {
      pub kind: String,       // dotted-or-snake namespace: "inventory.unit", "user", "checkout.session"
      pub id: String,         // consumer-stringified primary key
  }
  ```
  Constructor sugar: `AuditTarget::new(kind, id)` plus a `From<(impl Into<String>, impl ToString)>` for `(kind, id)` tuples. No type-state, no generics — ferro stays domain-agnostic.
- **D-08:** `target_kind` follows the same dotted-namespace convention as the existing `action` field (`"inventory.stock.adjust"`). Documented as a convention; not enforced at compile time. Consumers can use any string they want, but the canonical example and rustdoc lean into dotted-namespace + snake_case.

### Builder / write API

- **D-09:** Builder-style `AuditEntry::record(action)` returning a chainable struct, NOT a macro. Rationale: the design doc shows an `audit_log!` macro syntax, but Ferro convention favors typed builders that are introspectable in MCP (`code_templates`, `generation_context`). A thin macro façade can land later as a v0.x addition; the builder is the canonical surface.
  ```rust
  AuditEntry::record("inventory.stock.adjust")
      .actor(AuditActor::User(user_id.to_string()))
      .target(AuditTarget::new("inventory.unit", unit_id.to_string()))
      .before(json!({ "quantity": old }))   // optional
      .after(json!({ "quantity": new }))    // optional
      .reason("order_committed")            // optional
      .correlation(request_id)              // optional
      .tenant(tenant_id)                    // optional
      .write(&conn)
      .await?;
  ```
- **D-10:** `action` is the only required field. `actor` defaults to `AuditActor::System` if not set (caller intent: "I forgot, the system did it"). `target` is required only if any query helper that filters by target is to find this entry — but it's strongly recommended on every call; the builder logs a `tracing::warn!` if `write()` is called with no target. Missing `target` does NOT error — append-only audit must never refuse a write — but the diagnostic surfaces the call-site bug.
- **D-11:** `before` and `after` are both `Option<serde_json::Value>`. A pure event (e.g. `"user.password_reset_requested"`) has neither; a creation has only `after`; a deletion has only `before`; an update has both. The schema's `before` / `after` columns are nullable JSON.
- **D-12:** `correlation_id` is `Option<Uuid>`. Optional in v0; consumers pass explicitly. No automatic task-local pickup — the framework does not yet plumb a correlation id end-to-end, and adding that to Phase 153 would balloon scope.
- **D-13:** `tenant_id` is `Option<String>`. Stringly-typed for the same reason `AuditActor::User` is — ferro has no first-class tenant primitive (search confirms only consumer-specific `tenant_id` usage in `ferro-cli`'s `make_stripe` template).
- **D-14:** Execution signature: `async fn write<C: ConnectionTrait>(self, conn: &C) -> Result<AuditEntry, AuditError>`. Returns the persisted `AuditEntry` (with generated `id: Uuid` and DB-stamped `created_at: DateTime<Utc>`) so the caller can attach the audit id to a response, surface it in logs, or store it on a related domain row. No global `DB::connection()` shortcut — the caller passes the connection explicitly, identical to `GuardedUpdate::exec_one`'s contract in Phase 152.

### Error model

- **D-15:** `AuditError` is a `thiserror`-derived enum, one error per crate, panics nowhere:
  ```rust
  pub enum AuditError {
      #[error("audit: action is required")]
      MissingAction,                              // builder built with empty action string
      #[error("audit: db error: {0}")]
      Db(#[from] sea_orm::DbErr),
      #[error("audit: json serialization error: {0}")]
      Json(#[from] serde_json::Error),
  }
  ```
  Display prefix is `"audit: …"` for grep-friendliness across the workspace (matches `"guarded: …"`, `"config: …"`, `"apple sign: …"`).
- **D-16:** Missing target is NOT an error (per D-10). Missing action IS an error — an audit entry with no action is uninterpretable.
- **D-17:** JSON serialization errors propagate as `AuditError::Json`. In practice this only fires if a consumer hands the builder a `serde_json::Value` constructed from a malformed `Map` — extremely rare but covered.

### Schema & migration

- **D-18:** Ship a SeaORM migration as a public re-export so consumers add it to their `Migrator`:
  ```rust
  pub use migration::Migration as CreateAuditLogTable;
  ```
  Consumers register it in their migrator alongside their own migrations:
  ```rust
  impl MigratorTrait for Migrator {
      fn migrations() -> Vec<Box<dyn MigrationTrait>> {
          vec![
              Box::new(ferro_audit::CreateAuditLogTable),
              // ... app migrations
          ]
      }
  }
  ```
  Mirrors the pattern downstream apps already use; lets consumers control migration ordering relative to their domain tables.
- **D-19:** Schema columns (matches `INVENTORY-PRIMITIVES.md` §`ferro-audit` verbatim):
  ```
  audit_log
  ├── id             UUID PRIMARY KEY
  ├── tenant_id      VARCHAR NULL
  ├── actor_kind     VARCHAR NOT NULL
  ├── actor_id       VARCHAR NULL                  -- NULL for System / Anonymous
  ├── action         VARCHAR NOT NULL              -- "inventory.stock.adjust"
  ├── target_kind    VARCHAR NULL                  -- nullable per D-10
  ├── target_id      VARCHAR NULL
  ├── before         JSON NULL
  ├── after          JSON NULL
  ├── reason         VARCHAR NULL
  ├── correlation_id UUID NULL
  ├── created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
  ```
- **D-20:** Indexes (verbatim from the design):
  - `idx_audit_target` on `(tenant_id, target_kind, target_id, created_at)`
  - `idx_audit_actor`  on `(tenant_id, actor_kind, actor_id, created_at)`
  No additional indexes in v0; if a consumer's access pattern needs `(action, created_at)` or `(correlation_id)` they can add their own migration.
- **D-21:** `id` is `Uuid` (not auto-increment) — generated client-side at `write()` time. UUIDv4. Lets consumers log the id before the row hits the DB (relevant for emitting an event that references the audit entry).
- **D-22:** `created_at` is set by the DB (`CURRENT_TIMESTAMP` default) so clock-skew between application servers can't break ordering within a single DB.

### Query helpers (replay)

- **D-23:** Three read helpers on `AuditEntry`:
  - `history_for_target(target: &AuditTarget, conn: &C) -> Result<Vec<AuditEntry>, AuditError>` — ordered by `created_at ASC`. Hits the `idx_audit_target` index.
  - `recent_by_actor(actor: &AuditActor, conn: &C, limit: u64) -> Result<Vec<AuditEntry>, AuditError>` — ordered `created_at DESC`. Hits the `idx_audit_actor` index.
  - `recent(conn: &C, limit: u64) -> Result<Vec<AuditEntry>, AuditError>` — no filter, ordered `created_at DESC`. Useful for an admin "recent activity" panel.
  No `find_by_correlation_id` in v0 (no index for it; documented as "add your own migration if you need this").
- **D-24:** `reconstruct_state` helper — the replay primitive:
  ```rust
  pub fn reconstruct_state(entries: &[AuditEntry]) -> Option<serde_json::Value>
  ```
  Folds each entry's `before → after` JSON merge into a running state. Returns the final reconstructed state, or `None` if the entries are empty or none have `after`. Pure function, no DB call — consumer fetches `history_for_target` then calls `reconstruct_state` on the result. Implementation uses a shallow JSON object merge (newer keys overwrite older keys); arrays and nested objects are replaced wholesale, not deep-merged. Documented as the v0 semantics — deep-merge can be a v0.x option flag.
- **D-25:** No streaming / pagination helper in v0 (`limit` is sufficient for the targeted use cases). A consumer needing pagination uses `sea-orm` directly against the `audit_log` table, which is intentionally exposed via a public `Entity` re-export so SeaORM-native queries work.

### Retention

- **D-26:** `prune_older_than(cutoff: DateTime<Utc>, conn: &C) -> Result<u64, AuditError>` — deletes rows with `created_at < cutoff` and returns the count deleted. Caller decides whether and when to run it (typically a `ferro-queue` cron job in the consumer). No automatic retention enforcement; default is "keep forever".
- **D-27:** Document the operational tradeoff in the user-facing doc: GDPR / privacy may force pruning (consumer-side compliance), but audit trails should generally not be aggressively pruned because they are evidence. Recommend a default of 1-3 years if pruning at all.

### Concurrency

- **D-28:** No concurrency contract beyond "atomic single-row INSERT". Audit is append-only; there is no `read-then-write` pattern to make race-free. Tests do not need a concurrent-writer scenario (Phase 154's reservation tests cover the audit-emission-from-many-writers case end-to-end).
- **D-29:** No deduplication. If a caller writes the same audit entry twice (same action, same target, same time) it appears twice. Idempotency is the caller's job — the audit log records what happened, not what was intended.

### Testing

- **D-30:** Unit tests live next to the code (`#[cfg(test)] mod tests`) in `ferro-audit/src/`. Cover:
  1. Builder happy path: required fields set, optional fields set, `write()` returns the persisted entry with non-nil `id` and `created_at`.
  2. Missing `action` → `AuditError::MissingAction` returned from `write()`.
  3. Missing `target` writes successfully; `target_kind` / `target_id` columns are NULL; a `tracing::warn!` is emitted (verified via `tracing-subscriber/test` or equivalent).
  4. `before` / `after` round-trip — write a complex JSON payload, read it back via `history_for_target`, assert equality.
  5. `AuditActor::System` and `AuditActor::Anonymous` persist `actor_id = NULL`; other variants persist the string.
  6. `history_for_target` ordering (`created_at ASC`) — insert three entries with controlled timestamps, assert order.
  7. `recent_by_actor` ordering (`created_at DESC`) + `limit` enforcement.
  8. `prune_older_than` returns the count deleted and removes only rows strictly older than the cutoff.
  9. `reconstruct_state` on an empty slice returns `None`; on a sequence of writes reconstructs the final object correctly.
- **D-31:** ONE integration test (`tests/replay_round_trip.rs`) that proves the design promise: insert a sequence of audit entries simulating a domain object's lifecycle (`created → updated × 3 → status changed`), call `history_for_target` + `reconstruct_state`, assert the reconstructed JSON equals what the real domain object would look like.
- **D-32:** Property-based tests are **not in scope** for Phase 153. The reservation crate (Phase 154) carries the property-test budget for the milestone (per `INVENTORY-PRIMITIVES.md` testing strategy). Phase 153 ships hand-written tests that fully cover the surface.
- **D-33:** Postgres integration tests deferred. SQLite serial-writer is sufficient to validate append-only semantics and `JSON` column behavior; cross-dialect verification rides Phase 154's broader integration suite.
- **D-34:** Test harness: in-memory SQLite via the existing framework testing pattern (`framework/src/database/testing.rs` is the reference). `ferro-audit` re-derives the harness inline (does not depend on `framework`) — the migration is the unit under test.

### Documentation

- **D-35:** Module-level rustdoc on `lib.rs` with the canonical example from `INVENTORY-PRIMITIVES.md` §`ferro-audit`, rewritten to the builder API per D-09. Lead with the *why* (state-change history; regulatory/forensic value; replay), then show the one-call API.
- **D-36:** New user-facing doc page `docs/src/database/audit-log.md` covering: what the audit log is for, the schema, the `AuditActor` / `AuditTarget` shape, the `record(...).write()` API, query helpers, the replay model (`reconstruct_state`), pruning / retention guidance, GDPR considerations, and a worked example (the `INVENTORY-PRIMITIVES.md` inventory-decrement scenario, wired up end-to-end).
- **D-37:** ferro-mcp introspection: no new MCP tools in this phase. `application_info` will auto-include `ferro-audit` in `installed_crates` once it's a workspace member; `generation_context` and `code_templates` will pick up the rustdoc automatically. If a future agent-facing query helper makes sense (e.g. `audit_log_for_target` MCP tool) it can ship in v0.x.

### Release

- **D-38:** Workspace `[workspace.package] version` bumps one patch (from `0.2.25` to `0.2.26`) when Phase 153 verifies. Standard ferro release process.
- **D-39:** Add `ferro-audit` to Wave 1a of `.github/workflows/publish.yml` alongside `ferro-orm` (D-04 confirms no internal ferro-* deps). New-crate bootstrap from local terminal — same operational reality as Phase 151 / Phase 152.
- **D-40:** CHANGELOG entry under `ferro-audit` (new section) summarising: new crate, append-only `audit_log` table, `AuditActor` / `AuditTarget` typed model, builder API with `record(...).write()`, query helpers (`history_for_target`, `recent_by_actor`, `recent`), `reconstruct_state` replay helper, `prune_older_than` retention helper.

### Folded scope from todos

No pending todos matched Phase 153 (cross_reference_todos returned zero matches at gather time).

### Claude's Discretion

Within the boundaries set above, the planner/executor decides:

- Internal module layout of `ferro-audit/src/` (likely `lib.rs` + `actor.rs` + `target.rs` + `entry.rs` + `error.rs` + `migration.rs`, but the planner is free to consolidate)
- Whether the SeaORM `Entity`/`Model`/`ActiveModel` types live in a `entity` submodule or in `entry.rs`
- The exact JSON-merge implementation in `reconstruct_state` (shallow object merge is the documented v0 semantic; the implementation detail is open)
- Whether to expose a `pub use migration::Migration as CreateAuditLogTable` alias (D-18) at the crate root or via `pub mod migration { pub use … as CreateAuditLogTable; }`
- Exact rustdoc prose & code-block formatting
- Test file names within `ferro-audit/tests/`

### Deferred (NOT in this phase)

- `audit_log!` macro façade — can land as a thin wrapper in v0.x once the builder API is in real use and the ergonomic gaps are concrete.
- Automatic `correlation_id` pickup from `tracing` span / task-local — requires framework-level plumbing not currently in place; would derail v11.11.
- `from_request(&Request) -> AuditActor` helper — would create a `framework` dep, breaking the leaf-crate Wave 1a placement.
- Ferro-events emission on every audit write (`AuditEntryRecorded` event) — useful for real-time audit streaming dashboards, but the consumer can wrap `write()` to dispatch their own event. Not in core.
- MCP tools to query the audit log from an agent — interesting v0.x addition; Phase 153 is the substrate, not the introspection layer.
- Distributed audit-stream / log shipping (Loki, Splunk, S3) — orthogonal concern; the consumer can tail the table or subscribe to their own event.
- Postgres CI integration tests — would require docker-Postgres in CI for one primitive; disproportionate.
- Property-based tests — Phase 154 (`ferro-reservation`) carries that budget for the milestone.
- Automatic redaction / PII filtering on `before` / `after` payloads — consumer is responsible for redacting before passing JSON to the builder.
- Schema migration tooling that *rewrites* historic `before` / `after` payloads (e.g. for GDPR right-to-erasure) — caller-driven via raw SQL or `prune_older_than`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design source of truth

- `.planning/research/INVENTORY-PRIMITIVES.md` §`ferro-audit` — original spec (API shape, schema, indexes, replay claim). Authoritative.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Cross-crate relationships` — confirms `ferro-audit` is a near-leaf in the milestone graph; only `ferro-reservation` (Phase 154) is a downstream consumer.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Migration / rollout` — confirms `ferro-audit` ships in parallel with Phase 152 (`ferro-orm`), additive, no breaking changes.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Testing strategy` — per-crate audit unit tests scope (log writes, indexed queries, before/after JSON round-trip, replay reconstruction).

### Project conventions

- `CLAUDE.md` §`Architecture Principles` — project-agnostic crates rule (no hardcoded app identity, no consumer-specific types in the public API). `ferro-audit` must not bind to a `User` model, a tenant primary-key type, or any consumer-specific id shape.
- `CLAUDE.md` §`Testing & Linting` — exact fmt + clippy + test commands required pre-commit. Applies identically to `ferro-audit`.
- `CLAUDE.md` §`Workspace Structure` — `ferro-audit` is added to this table during execution.
- `.planning/PROJECT.md` — vision anchors; the projection/intent abstraction is the killer feature this milestone unblocks (via reservations + live read-models). Audit is the historical-truth substrate that makes both verifiable.
- `.planning/STATE.md` — current workspace version (`0.2.25` post-152), next version is `0.2.26` after Phase 153 verifies.

### Sibling phase context (must read before planning)

- `.planning/phases/152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-/152-CONTEXT.md` — Phase 152 is the structural twin for crate scaffolding, Wave 1a publish, error-naming convention (`"audit: …"` prefix mirrors `"guarded: …"`), testing harness choice, doc-page placement (`docs/src/database/`), and CHANGELOG shape. Mirror its decisions where they map; deviate only where audit semantics require it.

### Patterns to mirror (template ferro-* crates)

- `ferro-orm/Cargo.toml` — Wave 1a leaf-crate Cargo.toml shape (workspace inheritance, package metadata, dep style). Closest sibling.
- `ferro-orm/src/lib.rs` — module-level rustdoc tone for a v0 single-purpose crate.
- `ferro-wallet/Cargo.toml` — second reference for Wave 1a Cargo.toml shape.
- `ferro-events/src/lib.rs` — minimal-crate-with-one-primitive layout; closest structural analog at the public-API level (`pub use` re-exports, single concept).
- `.github/workflows/publish.yml` — Wave 1a crate list; `ferro-audit` is added here next to `ferro-orm`.
- `framework/src/database/testing.rs` — in-memory SQLite testing harness reference; `ferro-audit` derives its own inline (does not import `framework`).
- `framework/src/database/mod.rs` — SeaORM migration registration pattern as consumers use it; `ferro-audit::CreateAuditLogTable` plugs into this shape.

### Cross-phase coordination

- Phase 152 CONTEXT (above) — `ferro-orm` ships in parallel; `ferro-audit` does NOT depend on `ferro-orm` (per D-03) so the two phases can be executed concurrently without serialization.
- Phase 154 CONTEXT (when written) — `ferro-reservation` depends on `ferro-audit`; Phase 154 will exercise this API as its primary consumer (`hold` / `commit` / `release` / `expire` each emit an audit entry). Phase 153 must not assume reservation-specific concerns.

### Conventions repository (operator memory)

- `feedback_ci_clippy_command_match.md` — match CI's exact clippy command (`--all --all-targets -- -D warnings`) in pre-push checks.
- `feedback_validate_scope_premises.md` — `ferro-audit` does not currently exist as a crate; this premise was verified before this CONTEXT was written (`ls ferro-audit` → not found; `grep` for `ferro-audit` / `ferro_audit` returned zero implementation hits).
- `project_ferro_publish_token_scoping.md` — CI publish token has publish-update only; new-crate bootstrap requires personal token from local terminal.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets

- **SeaORM 1.0** is already a workspace dependency. `ferro-audit` reuses it directly via `sea-orm` and `sea-orm-migration`.
- **Workspace inheritance pattern** (`version.workspace = true`, `edition.workspace = true`, `license.workspace = true`) — copy from `ferro-orm/Cargo.toml`.
- **`thiserror` 2** is the project standard error-derive — used by every leaf crate.
- **`serde` + `serde_json` + `uuid` + `chrono`** are already workspace dependencies (used by `framework`, `ferro-stripe`, `ferro-events`, etc.); `ferro-audit` adds them as direct deps with the same versions.
- **In-memory SQLite testing pattern** — `framework/src/database/testing.rs` is the reference; `ferro-audit` re-derives the harness inline (does not depend on `framework`).
- **No existing audit code in the workspace** — `grep` for `audit_log`, `AuditActor`, `AuditTarget`, `ferro_audit`, `ferro-audit` returned only references in this design doc (`INVENTORY-PRIMITIVES.md`) and roadmap. Greenfield.

### Established patterns

- **One Error enum per crate** (`thiserror` derive) — convention across `ferro-orm`, `ferro-wallet`, `ferro-stripe`, `ferro-events`, `ferro-notifications`. `AuditError` follows the same shape.
- **Display prefix on error enum** — `"guarded: …"` (Phase 152), `"config: …"`, `"apple sign: …"`. `AuditError` uses `"audit: …"` for grep-friendliness.
- **Builder pattern: `with_*` / setter methods taking `mut self` → `Self`** — the `AuditEntry::record(...).actor(…).target(…)…` chain follows the consuming-builder shape used framework-wide.
- **Generic over `ConnectionTrait`** — `framework/src/database/query_builder.rs`, `framework/src/database/model.rs`, `GuardedUpdate::exec_*` all accept `impl ConnectionTrait` / `<C: ConnectionTrait>`. `AuditEntry::write` and every query helper follow suit.
- **`#[serde(rename_all = "snake_case")]`** on enums — applies to `AuditActor` (and the implicit `actor_kind` representation matches the snake_case variant name).
- **Wave 1a Cargo.toml metadata fields** — `description`, `keywords`, `categories = ["database"]`, `repository`, `readme = "README.md"`, `homepage = "https://ferro-rs.dev"`. Copy from `ferro-orm/Cargo.toml`.
- **SeaORM migration as public re-export** — pattern matches how the consumer-side `Migrator` collects migrations from multiple sources; `ferro-audit` exposes its migration as `pub use migration::Migration as CreateAuditLogTable;` so the consumer adds it explicitly.

### Integration points

- **Workspace `Cargo.toml`** — add `"ferro-audit"` to `[workspace.members]`.
- **`.github/workflows/publish.yml`** — add `ferro-audit` to `WAVE1A_CRATES` alongside `ferro-orm` and `ferro-wallet`.
- **Workspace version bump** — `[workspace.package] version = "0.2.26"`.
- **`framework/src/lib.rs`** — DO NOT add an automatic re-export of `ferro_audit`. Consumers depend on `ferro-audit` directly so framework users opt in. Same call as Phase 152.
- **`README.md` (workspace root)** — add `ferro-audit` to the workspace crates table (mirror how `ferro-orm` and `ferro-wallet` were added).
- **`CLAUDE.md` "Workspace Structure" table** — add a row for `ferro-audit` so downstream agents see it immediately.
- **ferro-mcp `application_info` / `installed_crates`** — picks up `ferro-audit` automatically once it's a workspace member; no MCP code changes expected.
- **`docs/SUMMARY.md` / nav** — add `audit-log.md` to the `Database` section (mirrors how `atomic-updates.md` was added in Phase 152).

### Constraints surfaced by the scout

- `ferro-audit` is **a new top-level crate** — Phase 153 is the bootstrap. First publish requires manual personal-token bootstrap from local terminal (CI token is publish-update only) — same operational reality as `ferro-wallet` Phase 151 PLAN-09 and `ferro-orm` Phase 152 plan 06.
- The framework has **no first-class tenant primitive** today — `tenant_id` only appears in `ferro-cli/src/commands/make_stripe.rs` and `ferro-mcp/src/tools/stripe.rs` as a consumer-template field. `AuditEntry::tenant(Option<String>)` is correctly stringly-typed and remains forward-compatible if ferro grows a typed tenant later.
- The framework has **no first-class correlation/request-id primitive** today — no `correlation_id`, `request_id`, `RequestId`, or `CorrelationId` types exist in `framework/` or any `ferro-*` crate. `AuditEntry::correlation(Option<Uuid>)` is caller-supplied; future framework-level plumbing can populate it without breaking the API.

</code_context>

<specifics>
## Specific Ideas

- The canonical sample from the design doc, rewritten to the v0 builder API for the rustdoc top example:
  ```rust
  AuditEntry::record("inventory.stock.adjust")
      .actor(AuditActor::User(user_id.to_string()))
      .target(AuditTarget::new("inventory.unit", unit_id.to_string()))
      .before(json!({ "quantity": old }))
      .after(json!({ "quantity": new }))
      .reason("order_committed")
      .write(&conn)
      .await?;
  ```
- The error-naming style across the workspace (`"guarded: …"`, `"config: …"`, `"apple sign: …"`) — `AuditError` follows the same `"audit: …"` Display prefix.
- The framing in the rustdoc: lead with *why* (state-change forensic history; replay; regulatory evidence), then show the one-call API. Then the operational footgun (`reconstruct_state` is shallow-merge, not deep-merge; consumers needing deep-merge run their own fold).
- Dotted-namespace convention for `action` and `target.kind`: `"inventory.stock.adjust"`, `"checkout.session.created"`, `"user.password_reset_requested"`. Documented as a convention so consumer code looks consistent across apps; not enforced at compile time.
- The mental model the rustdoc opens with: "ferro-audit is the *historical* twin of ferro-events. Events are 'something happened, react now'. Audit entries are 'something happened, here's the evidence forever'."

</specifics>

<deferred>
## Deferred Ideas

- **`audit_log!` macro façade** — recommended in the design doc but explicitly out of scope for v0; the builder is the canonical surface and a macro can wrap it later.
- **Automatic `correlation_id` from `tracing` span / task-local** — needs framework-level plumbing not currently in place; future phase.
- **`from_request(&Request) -> AuditActor` helper** — adds a `framework` dep, breaks Wave 1a placement.
- **Ferro-events `AuditEntryRecorded` event emission on every write** — out of scope; consumer can wrap `write()` to dispatch their own event.
- **MCP tools to query the audit log from an agent** — interesting v0.x addition; Phase 153 is the substrate.
- **Distributed audit-stream / log shipping** — orthogonal concern.
- **Postgres CI integration tests** — disproportionate for v0; ride Phase 154's broader suite.
- **Property-based tests** — Phase 154 carries the milestone's property-test budget.
- **PII redaction / GDPR right-to-erasure tooling** — consumer-side concern; `prune_older_than` is the only primitive in v0.
- **Deep-merge `reconstruct_state` variant** — v0 ships shallow-merge; a `reconstruct_state_deep` or option flag is a plausible v0.x addition once a real consumer hits the limit.
- **Pagination helpers on query API** — v0 ships `limit` only; SeaORM-native queries against the public `Entity` re-export cover the gap.

### Reviewed Todos (not folded)

No todos matched this phase (cross_reference_todos returned zero matches).

</deferred>

---

*Phase: 153-ferro-audit-crate-structured-before-after-audit-log-with-rep*
*Context gathered: 2026-05-13*
*Mode: --auto (single-pass, recommended defaults applied)*
