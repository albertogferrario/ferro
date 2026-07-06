# Phase 239: Soft-delete data model + `deleted_at` migration - Context

**Gathered:** 2026-06-23
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults; review before planning)

<domain>
## Phase Boundary

Establish the **soft-delete data substrate** every CRUD read/update/delete path in
v16.3 depends on. Concretely, this phase delivers four things at the data layer:

1. A backend-portable migration adding a nullable `deleted_at` column to the
   soft-deletable table(s) — applies clean under a fresh `db:migrate` on both SQLite
   and Postgres.
2. The `table()`/`soft_delete_column()` field→column binding: a resolver that maps a
   projection to its concrete table and soft-delete column (with defaults), consumed by
   the read path (and by CRUD dispatch in later phases).
3. The `deleted_at IS NULL` predicate enforced **at the data layer** (the read query
   builder), so a soft-deleted row is invisible by construction — not by per-tool
   filtering.
4. The `created_at`-on-create and tenant-column-as-server-injected contract fixed at the
   data/classification boundary (`created_at` defaulted in-DB; tenant + identifier +
   created_at classified as non-agent-input).

**Out of scope (later phases, do not build here):** CRUD input-schema *emission*
(Phase 240), `derive_crud_plan` + the INSERT/UPDATE/soft-delete execution through
`framework::write` (Phase 241), authorization/Gate wiring + non-disclosure envelope
(Phase 242), app projection flip + e2e (Phase 243). This phase lays the substrate those
consume; it does not perform writes.

</domain>

<decisions>
## Implementation Decisions

### Migration scope (target tables)
- **D-01:** Add `deleted_at` to **`orders` only** — the single projection that opts into
  delete in v16.3 (flipped to `.deletable(true)` in Phase 243). Do not speculatively add
  the column to other tables (`todos`, etc.); soft-delete is opt-in per projection and no
  other table is soft-deletable this milestone. The roadmap's "soft-deletable table(s)"
  resolves to one table here.

### Migration form & backend portability
- **D-02:** Author a **new standalone additive migration** (e.g.
  `m20260623_add_deleted_at_to_orders.rs`) using sea-orm's `alter_table` +
  `add_column(ColumnDef::new(Orders::DeletedAt).timestamp().null())`. Never edit the
  shipped `m20260611_create_orders_table.rs` — migrations are append-only.
- **D-03:** Register the new migration in `app/src/migrations/mod.rs` `Migrator::migrations()`
  in chronological order (after the existing `m20260614_*` entries).
- **D-04:** A **nullable timestamp** column is backend-portable across SQLite and Postgres
  via sea-orm's `ColumnDef` — no backend-specific SQL. Verification runs `db:migrate` on
  both backends (mirrors the existing migration CI/test posture). No backfill needed:
  existing rows get `NULL` = "not deleted", which is the correct default.

### `deleted_at IS NULL` enforcement point
- **D-05:** Inject the `deleted_at IS NULL` predicate in the **read query builder**
  (`ferro-mcp-server/src/dispatch.rs`, the `read_dispatch` WHERE-clause assembly),
  **mirroring the existing tenant-predicate injection** (`dispatch.rs:153`). Gate the
  injection on the projection being soft-deletable (its resolved soft-delete column is
  applicable — see D-06/D-07). This satisfies success criterion #3: enforcement is at the
  data layer, by construction, for every read — not added per-tool.
- **D-06:** Use the **resolved** soft-delete column name (D-07) when building the predicate,
  not a hardcoded `deleted_at`, so an explicit `.soft_delete_column(...)` override is honored.

### field→column binding resolver
- **D-07:** Add resolver accessors on `ServiceDef` in `ferro-projections/src/service.rs`:
  - `resolved_table()` → returns `self.table` or the default `format!("{}s", name.to_lowercase())`
    (matches the inline derivation + `TODO: ServiceDef.table` currently at `dispatch.rs:122`).
  - `resolved_soft_delete_column()` → returns `self.soft_delete_column` or `"deleted_at"`.
- **D-08:** **Wire `resolved_table()` into `dispatch.rs`**, replacing the inline
  `format!("{}s", service.name.to_lowercase())` and removing its TODO. This is the
  mechanical consequence of introducing the resolver and is required so the read path
  (and the `deleted_at` predicate, which references the same table) uses the bound table.
- **D-09:** Cover the resolver with table tests asserting **default vs explicit** for both
  table name and soft-delete column (success criterion #2).

### `created_at` + tenant server-injected contract (success criterion #4)
- **D-10:** `created_at` is set **at the data layer via the column's `DEFAULT current_timestamp`**
  (already true on `orders`). The new `deleted_at` is nullable with no default. This phase
  asserts the create-time default exists; the actual INSERT path is Phase 241.
- **D-11:** Add a **field-classification helper** (in `ferro-projections`, near the
  `FieldMeaning` logic) that identifies fields which must be **server-injected / never an
  agent input**: the tenant column (`service.tenant_column`), the identifier
  (`FieldMeaning::Identifier`), and `created_at` (`FieldMeaning::CreatedAt`). This is the
  schema-derivation boundary that Phase 240 consumes to exclude those fields from write
  schemas. Scope here = the classification predicate + tests; **not** the schema emission.

### Claude's Discretion
- Exact migration filename/date stamp and `DeriveIden` enum shape (follow the existing
  `m20260611_create_orders_table.rs` idiom).
- Naming of the resolver accessors and classification helper, provided semantics match
  D-07/D-11.
- Whether the classification helper returns a set of excluded field names or a per-field
  predicate — planner's call, as long as Phase 240 can consume it.

### Considered but NOT adopted (kept minimal)
- A reusable `add_soft_delete_column` helper in the `ferro-migration` crate. Deferred —
  YAGNI for a single table this milestone; revisit if a second soft-deletable table
  appears. A direct app migration is the simple, correct solution now.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Anchor spec (read first)
- `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md` — Track A
  design. Especially "Data-model requirements" (§166–172), "Dispatch architecture"
  (§134–164, the `derive_crud_plan` it will feed), "Within-Track sequencing" item 1
  (§228), and "Non-goals" (§239).

### Roadmap & requirements
- `.planning/ROADMAP.md` §"Phase 239" (lines ~3407–3427) — goal + 4 success criteria.
- `.planning/REQUIREMENTS.md` — CRUD-03 (soft-delete) and CRUD-05 (non-disclosure) are the
  downstream consumers of this substrate; Phase 239 owns no requirement uniquely.

### Code to extend (shipped substrate)
- `ferro-projections/src/service.rs` — `ServiceDef`; shipped `table` / `soft_delete_column`
  / `creatable` / `updatable` / `deletable` / `mcp_write_ability` fields + builders
  (`5cb17d60`) and `validate()` write-ability rule (~line 436). Add resolver accessors
  (D-07) and classification helper (D-11) here.
- `ferro-projections/src/field.rs` — `FieldMeaning` enum + `infer_meaning` (Identifier,
  CreatedAt, ForeignKey, Sensitive, Status). Basis for D-11.
- `ferro-mcp-server/src/dispatch.rs` — `read_dispatch`: table derivation at line ~122
  (`TODO: ServiceDef.table`) and tenant-predicate injection at line ~153 (the pattern to
  mirror for `deleted_at IS NULL`, D-05/D-08).
- `app/src/migrations/m20260611_create_orders_table.rs` — migration idiom to follow.
- `app/src/migrations/mod.rs` — `Migrator::migrations()` registration list (D-03).
- `app/src/models/orders.rs` — `TenantScoped` + `find_for_tenant(id, tenant_id)` (Phase 212),
  the targeting primitive update/delete reuse later.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Tenant-predicate injection** (`dispatch.rs:151–168`): bound-parameter WHERE-clause
  append, fail-closed when context missing. The `deleted_at IS NULL` predicate copies this
  shape (no bound value needed — it's a literal `IS NULL`).
- **`FieldMeaning` / `infer_meaning`** (`field.rs`): already classifies Identifier,
  CreatedAt, ForeignKey, Sensitive, Status — the server-injected contract (D-11) builds on
  this rather than re-deriving.
- **sea-orm migration idiom** (`m20260611_create_orders_table.rs`): `DeriveMigrationName`,
  `MigrationTrait`, `DeriveIden` enum, backend-portable `ColumnDef`. `created_at` already
  uses `.timestamp().not_null().default(Expr::current_timestamp())` — the create-time
  default for D-10.

### Established Patterns
- Migrations are **append-only**; never edit a shipped migration. New behavior = new file
  + register in `mod.rs`.
- SQL in the read path is **parameterized** (`Statement::from_sql_and_values`), table name
  derived in code (not user input). The resolver (D-07) feeds that derivation.
- Predicates that must hold for *every* call (tenant scoping; now soft-delete) live in the
  shared read builder, not in per-tool code.

### Integration Points
- `ServiceDef` (ferro-projections) → consumed by `dispatch.rs` (ferro-mcp-server) for reads.
  Resolver accessors cross this boundary.
- The migration lands in `app/` (the sample app's `orders` table); the framework crates
  stay project-agnostic (resolver defaults + classification are generic).

</code_context>

<specifics>
## Specific Ideas

- Mirror the tenant-predicate code at `dispatch.rs:153` line-for-line in structure when
  adding the soft-delete predicate — same `where_clauses.push(...)` / placement, so the
  two "always-on" predicates read identically.
- Replace, don't leave, the `TODO: ServiceDef.table` comment at `dispatch.rs:122` — D-08
  is its resolution.

</specifics>

<deferred>
## Deferred Ideas

- **Reusable `add_soft_delete_column` migration helper in `ferro-migration`** — out of
  scope; single-table need this milestone. Revisit when a second soft-deletable table
  appears.
- **`deleted_at` on additional tables** — opt-in per projection; add when a projection
  declares `.deletable(true)` and needs it. Not speculative.
- **`get_<svc>` dedicated tool, per-field `immutable()`/`read_only()` overrides** — spec
  non-goals (§239–243), tracked there, not here.

</deferred>

---

*Phase: 239-soft-delete-data-model-deleted-at-migration*
*Context gathered: 2026-06-23 (--auto)*
