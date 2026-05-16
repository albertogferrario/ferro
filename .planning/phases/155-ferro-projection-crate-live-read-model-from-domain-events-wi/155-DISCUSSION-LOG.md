# Phase 155: ferro-projection — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 155-ferro-projection-crate-live-read-model-from-domain-events-wi
**Mode:** `--auto` (single-pass, recommended defaults applied to every gray area)
**Areas discussed:** Crate naming and placement; Cross-crate dependency surface; Projection trait shape; ProjectionKey type; ProjectionRuntime API; Apply algorithm and concurrency model; Schema and migration; Error model; Broadcast contract; Rebuild semantics; Testing scope; Documentation placement and disambiguation; Release shape

---

## Crate Naming and Placement

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro-projection` (singular) at workspace root | Matches roadmap, design doc, phase directory name. Disambiguation from `ferro-projections` (plural) handled in rustdoc + doc page title + README. | ✓ |
| `ferro-readmodel` rename | Sidesteps singular/plural confusion entirely but contradicts the design doc and roadmap; would require backward edits across spec + roadmap + directory. | |
| `ferro-projection-live` rename | Same problem — diverges from roadmap; "live" framing belongs in the docs, not the crate name. | |
| Module inside `framework` crate | Would force consumers to pull the full framework crate for a primitive that is independently useful for any event-sourced read-model. | |

**Auto-selected:** `ferro-projection` (singular) at workspace root. Disambiguation is load-bearing across rustdoc, doc page, README, CLAUDE.md, and CHANGELOG (D-02, D-51, D-52).

---

## Cross-Crate Dependency Surface

| Option | Description | Selected |
|--------|-------------|----------|
| ferro-events + ferro-broadcast + sea-orm + sea-orm-migration | Minimal Wave 1b dep set. Listener registration via global dispatcher; broadcast fanout via existing Broadcaster API; snapshot persistence via SeaORM upsert. | ✓ |
| Add ferro-orm dep | Snapshot upserts are full row replaces by composite PK — no guarded predicate needed. Adding ferro-orm would couple to a feature we don't use. | |
| Add ferro-audit dep | Projections are derived state, not state-changing operations — adding audit would double-log every event (since the underlying domain event is the audit-worthy moment, not the projection update). Consumers compose ferro-audit + ferro-projection at the application layer if they want both. | |
| Add ferro-queue dep | Apply is synchronous and in-process; queuing is consumer territory. | |

**Auto-selected:** ferro-events + ferro-broadcast + sea-orm + sea-orm-migration. Wave 1b publish (D-04, D-05).

---

## Projection Trait Shape

| Option | Description | Selected |
|--------|-------------|----------|
| `Event: ferro_events::Event` bound | Reuses the existing event taxonomy; no fragmentation. State + Default + Serialize + DeserializeOwned; Delta + Serialize + Clone; sync `apply`. | ✓ |
| Define new `DomainEvent` trait | Design doc names this but the codebase already has `ferro_events::Event` with the right bounds. A new trait would fragment the event story. | |
| Make `apply` async | Would let consumers do IO inside the fold but defeats per-key serialization (lock crosses await boundaries). `apply` is a pure state fold by design. | |
| Omit `Default` bound on State | Forces an `Option<State>` API on first-apply; adds noise for marginal gain. `Default` is cheap to implement (empty/zero variant) and matches the natural "empty dashboard" semantic. | |

**Auto-selected:** `Event: ferro_events::Event + Serialize + DeserializeOwned`; `State: Clone + Default + Serialize + DeserializeOwned`; `Delta: Serialize + Clone`; sync `apply` (D-06, D-07, D-08, D-09, D-10).

---

## ProjectionKey Type

| Option | Description | Selected |
|--------|-------------|----------|
| Stringly-typed newtype `ProjectionKey(String)` | Matches Phase 153's `AuditTarget` and Phase 154's `Resource::Key` JSON-shaped opacity. Consumer stringifies any compound key with a documented dotted-namespace convention. | ✓ |
| Generic `type Key: Hash + Eq + Clone + Serialize + DeserializeOwned` | Would force the runtime to carry a generic key parameter through broadcast channel name (String), DB column (String), and per-key Mutex map (HashMap key). Added noise for no benefit. | |
| Typed enum / domain-specific | Violates project-agnostic crates principle (CLAUDE.md). | |

**Auto-selected:** Stringly-typed newtype with `new`, `as_str`, `Display`, `From<String>`, `From<&str>`. Multi-tenancy lives inside the key string (D-11, D-12).

---

## ProjectionRuntime API

| Option | Description | Selected |
|--------|-------------|----------|
| `new` + auto-register (`register(self: Arc<Self>)`) + manual `apply_event` + `read` + `rebuild` | Two entry points: default auto-registration into the global dispatcher (killer feature: one-line wiring), manual path for tests and explicit-dispatch consumers. Both share the same per-key serialization. | ✓ |
| Manual-only `apply_event` (no auto-registration) | Loses the killer "Arc::new + register" one-liner; consumers would wire listeners themselves. | |
| Auto-registration only (no manual `apply_event`) | Tests need the manual path; replay scripts need it; consumers wanting explicit dispatch control need it. | |
| Push-based subscriber API instead of broadcast | Would require ferro-projection to ship its own pub/sub primitive; duplicates ferro-broadcast. | |

**Auto-selected:** Two entry points — `register` (auto) + `apply_event` (manual). Plus `read`, `rebuild`. `ProjectionListener<P>` is an implementation detail (Claude's Discretion notes "recommended NO public"). (D-13, D-14, D-15, D-16, D-17, D-18).

---

## Apply Algorithm and Concurrency Model

| Option | Description | Selected |
|--------|-------------|----------|
| Per-key in-process Mutex via DashMap<String, Arc<Mutex<()>>> | Concurrent events on different keys parallelize; concurrent events on same key serialize. Single source of correctness. v0 explicitly single-instance. | ✓ |
| Optimistic concurrency control via `version` column | Adds retry / quarantine complexity for marginal benefit in v0. Column is in schema as forward-compat scaffolding (D-25); v0.x flips OCC on. | |
| Single dispatcher thread (actor model) | Serializes ALL applies (including across keys); destroys throughput. | |
| Stateless apply with full transaction per event | Would need `SELECT … FOR UPDATE`; sea-orm dialect-divergent; doesn't compose with the broadcast step. | |

**Auto-selected:** Per-key Mutex registry (D-19, D-20, D-31, D-32, D-33). Single-instance assumption documented as the v0 operational caveat (D-34).

---

## Schema and Migration

| Option | Description | Selected |
|--------|-------------|----------|
| `projection_snapshots(projection_name, key, state JSON, version BIGINT, updated_at TIMESTAMP)` composite PK `(projection_name, key)` | One table. Single composite PK index covers every access path. Version column is forward-compat (D-25). | ✓ |
| Single primary table + separate `projection_events` log table | Out of scope for v0 (D-03 / Deferred). v0.x adds it if a real consumer needs replay-from-event-log. | |
| UUID surrogate PK + UNIQUE (projection_name, key) | Adds a column for no benefit; composite PK is simpler and naturally enforces uniqueness. | |
| Separate table per projection | Multiplies migration count; consumers don't want one-table-per-projection administrative cost. | |

**Auto-selected:** Single `projection_snapshots` table with composite PK on `(projection_name, key)`. JSON state column. Migration shipped as `pub use migration::Migration as CreateProjectionSnapshotsTable;` (D-23, D-24, D-25, D-26, D-27).

---

## Error Model

| Option | Description | Selected |
|--------|-------------|----------|
| `ProjectionError` enum with `Db`/`Json`/`Broadcast`/`Events`/`StateNotFound` variants; `"projection: …"` Display prefix | Mirrors workspace convention (`"guarded: …"`, `"audit: …"`, `"reservation: …"`). String payloads for Broadcast/Events because the source crates' Errors don't compose cleanly through `thiserror::From` (Phase 149 precedent). | ✓ |
| Box<dyn Error> umbrella | Loses static typing; consumers can't match. | |
| Per-method error types | Surface bloat; matches no other ferro-* crate. | |

**Auto-selected:** Single `ProjectionError` enum with five variants. `From<sea_orm::DbErr>` and `From<serde_json::Error>` via `thiserror::From`. Hand-written `From<ferro_broadcast::Error>` and `From<ferro_events::Error>` via `.to_string()` (D-28, D-29, D-30).

---

## Broadcast Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Public channel `projection.{name}.{key}`, event name from `broadcast_event_name()` defaulting to `"delta"`, payload is JSON-serialized `Delta` | Predictable channel naming; opinion-free on payload shape; sensible defaults. | ✓ |
| Private channel `private-projection.{name}.{key}` by default | Requires `ChannelAuthorizer` wiring; many projections are public (dashboards, public metrics); private would be the wrong default. Consumers needing private channels override via future `channel_for(key)` (deferred to v0.x). | |
| Envelope payload `{ type, version, state, delta }` | ferro-projection would pick a wire format; consumers wanting an envelope make `Delta` a struct that carries one. | |
| Multiple events per apply (one per state-change pattern) | Multiplies frontend dispatch complexity; one-event-per-apply is the simpler default. | |

**Auto-selected:** `projection.{name}.{key}` public channel, `"delta"` default event name, raw `Delta` JSON payload. `rebuild` broadcasts on the same channel with event name `"rebuild"` and payload = full state (D-38, D-39, D-40, D-41).

---

## Rebuild Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| `rebuild(key, events)` accepts caller-supplied event iterator; discards snapshot; folds through Default; persists final state; broadcasts ONE `"rebuild"` frame | v0 is event-log-source-agnostic. Consumers feed events from audit log, queue logs, recovery file — whatever they have. | ✓ |
| `rebuild()` walks an in-crate `projection_events` table | Requires shipping the event-log table (D-03 / Deferred). Out of scope for v0. | |
| No rebuild API in v0 | Consumers need replay for schema changes / audit-detected divergence; removing it would force per-app hand-rolling. | |
| `rebuild` transactional (DELETE + folded-upsert in one transaction) | v0 takes the per-key Mutex and does sequential DELETE + apply-loop; crash mid-rebuild loses the snapshot but next apply re-initializes from Default. Documented as the v0 crash semantic (D-44). v0.x can tighten. | |

**Auto-selected:** `rebuild(key, events: I) where I: IntoIterator<Item = P::Event>` — accepts caller's stream; under per-key Mutex; broadcasts single `"rebuild"` frame with final state (D-17, D-41, D-42, D-43, D-44).

---

## Testing Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Unit tests + 3 integration tests + 3 proptest properties | Comprehensive coverage: key/error/trait-defaults unit; event-bus integration; cross-crate showcase with ferro-reservation; concurrency integration; proptest determinism + replay equivalence + cross-key independence. | ✓ |
| Unit tests only | Misses the auto-register path validation, the cross-crate composition, and the concurrency claim. | |
| Add Postgres CI test | Same call as Phases 152 / 153 / 154 — deferred. SQLite serial-writer + property tests are sufficient. | |
| Skip property tests | Phase 155 carries the projection-side property-test budget per INVENTORY-PRIMITIVES.md §`Testing strategy`. Replay equivalence is the proof-of-correctness for the rebuild contract. | |

**Auto-selected:** Full coverage — 10 unit tests + 3 integration tests (event bus, cross-crate showcase, concurrency) + 3 proptest properties (apply determinism, replay equivalence, cross-key independence). In-memory SQLite. (D-45 through D-50).

---

## Documentation Placement and Disambiguation

| Option | Description | Selected |
|--------|-------------|----------|
| User-facing page at `docs/src/features/live-read-models.md` (title "Live Read-Models") | Sibling of existing `features/projections.md` (which covers v9.0 Service Projections). Singular/plural disambiguation lives in the page-level note + the page title itself (avoids singular/plural in the URL slug). | ✓ |
| `docs/src/features/projection-live.md` (title "Live Projections") | Keeps "projection" in the URL slug but adds "live" qualifier; readers searching "projection" still hit both pages. Less clear than the action-focused "live-read-models" framing. | |
| `docs/src/database/projection-snapshots.md` | Misframes the feature as a database concern; it's really an event-fold + broadcast feature that happens to persist a snapshot. | |
| Single combined page covering both ferro-projections + ferro-projection | Would be confusing — they're orthogonal abstractions. Two pages with mutual cross-links is cleaner. | |

**Auto-selected:** `docs/src/features/live-read-models.md` with explicit "Not to be confused with `ferro-projections` (plural)" callout. Module-level rustdoc in `lib.rs` opens with the same disambiguation paragraph. README + CLAUDE.md crate-table rows carry the disambiguation in the table itself. (D-02, D-51, D-52, D-53).

---

## Release Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Workspace version 0.2.32 → 0.2.33; Wave 1b publish; first-publish bootstrap from local terminal | Matches the cadence Phases 152 / 153 / 154 established. CI token has publish-update only; new crate needs personal `publish-new` token from a local terminal. | ✓ |
| Bundle with ferro-reservation release | Phase 154 already shipped at 0.2.32; phase boundaries are independent releases. | |
| Bump minor (0.2 → 0.3) | Pre-1.0 with breaking changes acceptable; no reason to bump minor for an additive Wave 1b crate. | |

**Auto-selected:** Workspace version bump 0.2.32 → 0.2.33, Wave 1b publish slot, manual first-publish bootstrap (D-54, D-55, D-56).

---

## Claude's Discretion

The planner / executor decides without further user input:

- Internal module layout of `ferro-projection/src/` (likely `lib.rs` + `projection.rs` + `key.rs` + `runtime.rs` + `listener.rs` + `entity.rs` + `migration.rs` + `error.rs` — consolidation allowed where the public surface is unchanged).
- Whether to expose SeaORM `Entity` / `Model` / `ActiveModel` types publicly (recommended YES — matches Phase 153 / 154).
- Exact `tracing::warn!` / `tracing::error!` wording on broadcast / DB / events failures.
- Exact `proptest` strategy shape (the three properties are locked in D-49; the generator construction is open).
- Whether to ship `ProjectionRuntime::read_required(key)` helper alongside `read` (recommended YES; uses `StateNotFound` variant from D-30).
- Test file names within `ferro-projection/tests/`.
- Rustdoc prose & code-block formatting.

---

## Deferred Ideas

See CONTEXT.md `<deferred>` section. Key items: in-crate persistent event log (v0.x), event-log-backed snapshot interval enforcement (v0.x), optimistic concurrency control (v0.x), cross-instance coordination, private/presence broadcast channels, MCP `list_projections` tool, listener unregistration, macro façade, deep-merge semantics.
