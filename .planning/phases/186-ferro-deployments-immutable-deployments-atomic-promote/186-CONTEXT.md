# Phase 186: ferro-deployments — Immutable Deployments + Atomic Promote - Context

**Gathered:** 2026-06-07 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

New leaf crate `ferro-deployments` providing the deployment abstraction: every publish is an immutable, addressable row; going live is one atomic pointer flip; rollback is promoting an older row. The artifact shape is opaque — static HTML sites, compiled JSON-UI spec bundles, and Inertia SSR manifests all fit the same API. Includes a portable `deployments` migration helper (SQLite + Postgres), the `DeploymentStorage` trait with an S3-compatible default delegating to ferro-storage, and a `preview_url` wildcard-subdomain helper.

**Killer feature:** the one-UPDATE atomic promote — deployment-as-immutable-row makes going live, preview, and rollback all collapse into a single pointer flip. Everything else (schema, storage trait, preview URLs) exists to support this.

Requirements: DEPL-F-01, DEPL-F-02, DEPL-F-03. Consumer: gestiscilo Phase 188 (introduces the `deployments` table via this crate's migration helper).

</domain>

<decisions>
## Implementation Decisions

### Pointer ownership & promote mechanics
- **D-01:** The active pointer is **crate-owned**: a pointer table keyed by an opaque `owner_key` string (e.g. `"tenant:42/frontend:7"` — semantics are the consumer's business). The success criterion requires the concurrent-promote race test inside this crate, which is only possible if the crate owns the pointer storage. Consumers may keep their own denormalized FK (gestiscilo `frontends.active_deployment_id`) but the source of truth is the crate's table.
- **D-02:** `promote(owner_key, deployment_id)` is **last-write-wins**, a single atomic UPDATE, returning the previously-active deployment id. Gestiscilo's "newer-deployment-wins" optimistic check (their PITFALLS B-01) stays consumer-side — do not duplicate that control surface here; document that consumers can compose guards (e.g. `ferro-orm` `GuardedUpdate`) around promote.
- **D-03:** Atomic previous-id return: the pointer row carries `deployment_id` + `previous_deployment_id`; one UPDATE sets both from old row values (`SET previous_deployment_id = deployment_id, deployment_id = ?` — both Postgres and SQLite evaluate SET expressions against pre-update values). Exact SQL formulation is planner/executor discretion as long as the race test (two concurrent promotes serialize, no torn state) passes on both backends.
- **D-04:** `rollback(owner_key)` = promote of the pointer row's `previous_deployment_id`. Promoting a deployment whose status is not `ready` is rejected with a structured error. Promoting a deployment with `artifact_deleted_at` set is also rejected.

### Schema & identifier design
- **D-05:** `deployments` table: i64 autoincrement PK (Phase 185 D-05 portability precedent) **plus** a DNS-safe unique `identifier` string column (lowercase, subdomain-label-safe) used for preview URLs and external addressing. Roadmap criterion 1 lists "identifier" as a distinct recorded field.
- **D-06:** Recorded fields per DEPL-F-01: identifier, `source_ref` (e.g. git SHA, nullable), `artifact_location` (opaque string), `byte_size`, `status`, timestamps (`created_at` + terminal-transition timestamp). Planner refines exact column types following the `CreateJobsTable` precedent.
- **D-07:** Status vocabulary: `building` / `ready` / `failed` (roadmap names these exactly). Allowed transitions: `building→ready`, `building→failed`. Rows are never mutated after reaching terminal status — enforce in the API layer (no raw row mutation surface), documented invariant.
- **D-08:** Include nullable `artifact_deleted_at` from day one (gestiscilo PITFALLS B-03: future rollback must be able to refuse promoting a deployment whose artifacts were lifecycle-deleted). Setting it is the one permitted post-terminal metadata write — it marks artifact GC, not deployment mutation.
- **D-09:** Migration helper follows the Phase 185 `CreateJobsTable` pattern exactly: exported struct implementing `MigrationName` + `MigrationTrait`, SchemaManager-only DDL, zero backend-specific SQL, consumers register it in their own `Migrator`.

### DeploymentStorage trait & ferro-storage coupling
- **D-10:** `ferro-deployments` depends directly on `ferro-storage` (both leaf crates; ferro-deployments lands in publish Wave 1b). The S3-compatible default `DeploymentStorage` impl wraps a `ferro_storage::Storage`/`Disk`.
- **D-11:** Trait granularity: prefix-scoped artifact operations — store/get/delete files under a per-deployment prefix; `artifact_location` recorded as an opaque string the storage impl understands. Exact method signatures are planner discretion; keep the trait minimal and artifact-shape agnostic (criterion 5).
- **D-12:** Like ferro-queue (Phase 185 D-03): the crate takes a `sea_orm::DatabaseConnection`, may depend on `sea-orm` directly, must NOT depend on `framework`.

### API surface & lifecycle
- **D-13:** API shape: a `Deployments` handle struct wrapping the `DatabaseConnection`, methods: `create(owner_key, source_ref) -> Deployment` (status `building`), `mark_ready(id, artifact_location, byte_size)`, `mark_failed(id, error)`, `promote(owner_key, deployment_id) -> previous id`, `rollback(owner_key)`, `active(owner_key)`, `get(id)`, `list(owner_key)`. Listing API is named in the locked design (gestiscilo dashboard consumes it).
- **D-14:** `preview_url(deployment)` reads `DEPLOYMENT_PREVIEW_DOMAIN` env var via a `from_env()` config struct (project-agnostic crates rule — no hardcoded domains); returns `Option<String>` of the form `https://{identifier}.{domain}/`; unset env → `None` ("consumers may leave it unwired").
- **D-15:** Criterion 5 proof: a doc-test or example stores a non-HTML artifact bundle (JSON specs) through the same API — zero HTML/gestiscilo-specific assumptions anywhere in the crate.
- **D-16:** New-crate workspace chores: add to `.github/workflows/publish.yml` Wave 1b (depends on ferro-storage from Wave 1a); first publish requires a one-time manual `cargo publish -p ferro-deployments` (CI token is publish-update only); docs page in `docs/src/`; error type via `thiserror`, one Error enum; builder methods consuming `with_*` where applicable; serde enums `snake_case`.

### Claude's Discretion
- Exact claim/flip SQL per backend (as long as the race test passes on SQLite + Postgres)
- Identifier generation scheme (random slug length/alphabet) — must be DNS-label-safe and unique
- Exact `DeploymentStorage` method signatures and whether streaming is supported in v1
- Whether the pointer lives in a dedicated `deployment_pointers` table or another crate-owned structure (table recommended)
- Whether ferro-mcp gains a deployments introspection tool now or when the framework integrates the crate (docs page is mandatory either way)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Locked design (gestiscilo v7.1 — source of this milestone)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-ARCHITECTURE.md` — D-05 (flat deployment list, preview subdomains, promote = single atomic UPDATE, rollback = promote-of-previous), "Ferro-side primitives" table (`ferro-deployments` deliverable definition), "Reuse across ferro consumer profiles" (artifact-shape agnosticism)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-PITFALLS.md` §B (B-01 double-publish race — consumer-side optimistic check, crate-side LWW; B-03 `artifact_deleted_at` column requirement)

### Ferro repo
- `.planning/ROADMAP.md` §"v12.3 Deployment Platform Primitives" — requirements DEPL-F-01..03, phase success criteria, consumer pairing (gestiscilo Phase 188)
- `.planning/phases/185-ferro-queue-db-backed-job-queue/185-CONTEXT.md` — carried-forward conventions (D-03 connection threading, D-05 schema portability, D-06 migration helper)
- `ferro-queue/src/migration.rs` — the migration-helper template to follow
- `.planning/codebase/CONVENTIONS.md`, `.planning/codebase/TESTING.md` — workspace conventions

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-queue/src/migration.rs` — `CreateJobsTable`: the exact portable-migration-helper pattern to replicate (`MigrationName` + `MigrationTrait`, SchemaManager DDL only).
- `ferro-storage/src/facade.rs` — `Storage` facade + `Disk` handle with `put`/`get`/`delete`/`url`/`files`/`delete_directory`; the S3-compatible default `DeploymentStorage` impl delegates here.
- `ferro-orm` `GuardedUpdate` — atomic conditional updates; documented composition point for consumers wanting newer-deployment-wins promote guards.
- Phase 185 race-test approach (two concurrent claimants, SQLite always-on + cfg-gated Postgres) — reuse for the concurrent-promote test.

### Established Patterns
- New-crate setup: `ferro-bundle` (Phase 183) is the most recent new-crate precedent (workspace member, publish.yml wave entry, docs page).
- Error types: `thiserror`, one Error enum per crate. Serde enums `snake_case`. Consuming `with_*` builders.
- Crates take `sea_orm::DatabaseConnection`; never depend on `framework`.
- Config structs expose `from_env()` reading framework-convention env vars (project-agnostic crates rule).

### Integration Points
- `.github/workflows/publish.yml` — add `ferro-deployments` to Wave 1b (depends on ferro-storage, a Wave 1a leaf).
- First publish: one-time manual `cargo publish -p ferro-deployments` from a local terminal (CI token lacks publish-new).
- `docs/src/` — new feature page required; SUMMARY.md entry.
- Consumer: gestiscilo Phase 188 registers `CreateDeploymentsTable` (or equivalent) in its Migrator and wires `Frontend.active_deployment_id` denormalization around the crate-owned pointer.

</code_context>

<specifics>
## Specific Ideas

- The concurrent-promote race test is the phase's proof artifact (criterion 2): two concurrent promotes serialize correctly — last-write-wins, no torn state — on both SQLite and Postgres (Postgres behind the cfg-gated test, mirroring Phase 185).
- The single-UPDATE previous-id flip (`SET previous_deployment_id = deployment_id, deployment_id = ?`) exploits the SQL standard rule that SET expressions read pre-update row values — valid on both backends.
- Depends on Phase 185 for jobs-table *conventions* only (schema style, migration helper shape) — not a compile dependency.

</specifics>

<deferred>
## Deferred Ideas

- Deployment retention/GC (deleting old artifact prefixes, setting `artifact_deleted_at` automatically) — the column ships now; the lifecycle tooling is a future phase.
- Newer-deployment-wins promote variant (`promote_if_newer`) — consumer-side concern for now (gestiscilo PITFALLS B-01); promote it into the crate only if a second consumer needs it.
- ferro-mcp deployments introspection tool — natural once the framework integrates the crate; not required for the leaf-crate phase (Claude discretion if it falls out for free).

</deferred>

---

*Phase: 186-ferro-deployments-immutable-deployments-atomic-promote*
*Context gathered: 2026-06-07 (auto mode)*
