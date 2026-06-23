# Phase 241: `derive_crud_plan` + wire CRUD verbs into `framework::write` - Context

**Gathered:** 2026-06-23
**Status:** Ready for planning
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen and logged below)

<domain>
## Phase Boundary

Add the CRUD analog of `derive_transition_plan` — `derive_crud_plan(svc, verb, inputs)`
in `ferro-projections` producing a **pure, serializable** INSERT / UPDATE / soft-delete
plan — and teach the **existing** `framework::write` kernel (`dispatch_write`) a CRUD verb
**alongside** the transition path, so `create_`/`update_`/`delete_<svc>` execute through the
*same* dispatcher, override registry, idempotency, channel-parameterized audit, and
confirmation that transitions already use. The kernel is **extended, never forked**.

Delivers requirements **CRUD-06** (derive_crud_plan + framework::write wiring, single-source
across MCP + visual surfaces) and **CRUD-03** (`delete_<svc>` soft-deletes via `deleted_at`,
is confirmation-gated, and is filtered out of `list_` and every read/update/delete path).

**Explicitly out of scope (owned by Phase 242):** write authorization (`read_write` scope +
`.mcp_write_ability` Gate), server-side `tenant_id` injection, and cross-tenant /
soft-deleted non-disclosure. Phase 241 wires the verbs through the kernel; Phase 242 closes
the safety envelope around them. The `CrudPlan` is designed so 242 extends it without rework
(see D-09).

</domain>

<decisions>
## Implementation Decisions

The anchor spec (`docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md`)
locks the architecture. These decisions resolve the remaining implementation gray areas.

### `CrudPlan` type (mirror `TransitionPlan`)
- **D-01:** Add a `CrudPlan` type in `ferro-projections/src/executor.rs` (same module as
  `derive_transition_plan` / `TransitionPlan`), modeled as an **enum** with `Create` /
  `Update` / `Delete` variants. It is **pure serializable data** — `#[derive(Debug, Clone,
  PartialEq, Eq, Serialize, Deserialize, JsonSchema)]` — carrying *resolved* facts as data,
  **no closures, no I/O, no async** (exactly like `TransitionPlan`).
  - *Create*: resolved table, write-included columns + values, server-set fields
    (`created_at`, and `Status`=initial when an SM exists), `RETURNING *`.
  - *Update*: resolved table, patch column/value set, predicate components (`id`,
    `deleted_at IS NULL` via `resolved_soft_delete_column()`).
  - *Delete*: resolved table, soft-delete column set to `now`, predicate (`id`).
  - **[auto] recommended default** — an enum mirrors `TransitionPlan`'s "data, not behavior"
    convention and keeps the three verbs' divergent shapes explicit and serializable for the
    structured-envelope path.

### `derive_crud_plan` signature
- **D-02:** `pub fn derive_crud_plan(svc: &ServiceDef, verb: CrudVerb, inputs: &Value) ->
  Result<CrudPlan, Error>` in `executor.rs`, pure and side-effect-free, mirroring
  `derive_transition_plan(svc, action_name)`. Reuses the Phase 239/240 resolver accessors
  (`resolved_table`, `resolved_soft_delete_column`, `is_write_excluded_field`,
  `is_server_injected_field`) to compute the column sets — single source of truth with the
  schema builders. New `Error` variants as needed (e.g. verb-not-enabled), following the
  existing executor `Error` style.
  - **[auto] recommended default** — a typed `CrudVerb` discriminant (Create/Update/Delete)
    keeps the derivation total and testable; the spec's three SQL shapes map 1:1 to it.

### Kernel-wiring representation (one dispatcher, no fork)
- **D-03:** Teach `framework::write::dispatch_write` a **CRUD verb alongside** the transition
  path using a **thin verb discriminant**, NOT by fabricating synthetic `ActionDef`s. The
  CRUD path flows through the *same* `dispatch_write` pipeline with `transition_guard = None`
  (the guard union collapses to any declared preconditions). Steps already in the kernel —
  guard re-eval, idempotency, confirmation seam, execute, idempotency-store, audit,
  post-persist override hook — are reused unchanged.
  - **[auto] recommended default** — Success Criterion #4 requires *exactly one* `dispatch_write`
    kernel with no second CRUD dispatcher and no transition `match` re-encoded on the CRUD
    path. A discriminant extends the existing entry point; faking `ActionDef`s would smuggle
    transition semantics onto a non-transition verb.

### Generic CRUD execution (framework-provided, not app-supplied)
- **D-04:** The generic CRUD SQL is executed by a **framework-provided generic CRUD
  executor** that interprets a `CrudPlan` into parameterized SQL, invoked *through*
  `dispatch_write`. Apps do **not** hand-write a CRUD `ExecutorFn` — that would defeat
  "zero hand-written tool code." State-transition actions keep using their registered
  `ExecutorFn`; the CRUD verb routes to the built-in interpreter. Both run inside the single
  `dispatch_write` pipeline.
  - **[auto] recommended default** — the whole compression win of Track A is that one
    projection declaration yields a working write surface with no per-verb code. A generic
    interpreter of the serializable plan delivers that; the override hook (D-08) covers the 20%.

### Soft-delete (CRUD-03)
- **D-05:** `delete_<svc>` performs `UPDATE … SET <soft_delete_column> = now WHERE id = ?`
  (soft-delete), never a physical `DELETE`. The column is `resolved_soft_delete_column()`
  (default `deleted_at`, explicit override honored — Phase 239 substrate). A soft-deleted
  row is already excluded from `list_<svc>` by the Phase 239 `deleted_at IS NULL` predicate
  (`ferro-mcp-server/src/dispatch.rs`); `update_<svc>` must carry the same predicate so a
  soft-deleted target is unaddressable.
  - **[auto] recommended default** — matches the spec's Delete SQL and SC#2.

### Delete confirmation gating (reuse the existing seam)
- **D-06:** `delete_<svc>` is confirmation-gated by **reusing** the existing confirmation
  machinery, not a new one. The kernel's confirmation seam currently fires on
  `transition_trigger.is_some()`; extend the seam so the **CRUD delete verb is flagged
  destructive** and the same `!is_confirmed → Err(ConfirmationRequired)` path triggers.
  Synthesize `request_confirm_delete_<svc>` / `confirm_delete_<svc>` framing tools in
  `ferro-mcp-server/src/renderer.rs` (mirroring the transition-only synthesis at
  `renderer.rs:115-155`), reusing `ConfirmationStore`, the `generate_confirmation_token`
  CSPRNG, the single-use `confirm()`, and the `{tenant_id, verb/action, record_id}` binding
  check unchanged. `create_`/`update_` are non-destructive (no confirmation).
  - **[auto] recommended default** — CRUD-03 + spec require confirmation reuse, not a parallel
    token system. The `delete_` input schema already advertises `confirmation_token` (Phase 240).

### Override registry (reuse, no new mechanism)
- **D-07:** Per-verb overrides use the existing `WriteDispatcher::with_override(name, hook)`
  registry keyed on the **tool name** — `with_override("create_order", …)`,
  `"update_order"`, `"delete_order"` — consulted at the existing post-persist hook
  (`framework/src/write/mod.rs:431-433`). The generic derived plan is the **default** when no
  override is registered; `make_write_dispatcher` stays "register overrides," not "implement
  everything."
  - **[auto] recommended default** — SC#3 requires the override to replace the generic plan
    "with no new mechanism." Reusing the keyed registry is exactly that.

### Idempotency & audit (reuse unchanged)
- **D-08:** CRUD verbs reuse the kernel's idempotency lookup/store (scoped by `(tenant_id,
  idempotency_key)`) and the channel-parameterized audit (`ferro_audit::AuditEntry::record`)
  **unchanged** — no CRUD-specific idempotency or audit path. The executor's return value
  (the created/updated record, or the soft-delete outcome) is stored verbatim in the audit
  log, so it must be audit-safe (no `Sensitive` field leakage). Exact audit label string
  (e.g. `{channel}.action.{name}` style for the CRUD verb) is **Claude's discretion** during
  planning, provided it reuses the existing `record(...)` mechanism.
  - **[auto] recommended default** — the spec states idempotency + audit are "reused
    unchanged"; only the verb's identity string is new.

### Tenant-predicate boundary (241 ↔ 242)
- **D-09:** Phase 241 wires `id` + `deleted_at IS NULL` predicates and server-set
  `created_at` / initial `Status`; it does **not** implement server-side `tenant_id`
  injection, the `read_write`-scope/`.mcp_write_ability` Gate, or non-disclosure — those are
  **Phase 242**. The `CrudPlan` MUST be shaped to **accommodate a tenant-predicate slot** so
  Phase 242 extends it (adds `tenant_id = ctx` to create-insert and update/delete predicates)
  without reworking the plan or the kernel wiring. Phase 241 SC deliberately omit tenant from
  create/update/delete (see ROADMAP SC#1–#2).
  - **[auto] recommended default** — prevents scope bleed and double-building the tenant path;
    keeps Phase 241 focused on the verb→kernel seam while leaving a clean extension point.

### Structured-envelope routing
- **D-10:** Route `create_`/`update_`/`delete_` results through the existing Phase 205
  `CallToolResult::structured` envelope (`ferro-mcp-server/src/jsonrpc.rs:144`). Replace the
  current `not_yet_implemented` NTI short-circuit at
  `ferro-mcp-server/src/write_dispatch.rs:155-180` with real derive→dispatch. Extending the
  `tools/call` regression guard to assert well-formed `content[]` for the new verbs is owned
  by **Phase 243** (integration), but Phase 241 must emit through `structured` so 243 only
  adds assertions.
  - **[auto] recommended default** — matches the spec ("route the new results through the same
    `structured` envelope"); guard extension is the integration phase's job.

### Claude's Discretion
- Exact `CrudVerb` enum placement (ferro-projections vs framework re-export) and whether
  `CrudPlan` variants embed column/value vectors vs an ordered map — planner picks the shape
  that serializes cleanly and tests well.
- Audit label string for CRUD verbs (D-08).
- Whether the generic CRUD executor lives as a free function or a method on a kernel type,
  and the SQL builder it uses (must match the existing dispatch tests' sqlite-in-memory
  approach).
- New `Error`/`WriteError` variant names for verb-not-enabled / row-not-found, following
  existing enum style.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (authoritative)
- `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md` — Track A design;
  §"Dispatch architecture (extend `framework::write`, do not rebuild)" and §"Decisions" are
  binding for this phase. Lists reusable assets and the derive_crud_plan SQL shapes.

### Requirements & roadmap
- `.planning/REQUIREMENTS.md` — CRUD-03 (soft-delete + confirmation + filtering), CRUD-06
  (derive_crud_plan + kernel wiring, no rebuild).
- `.planning/ROADMAP.md` §"Phase 241" (lines ~3461-3486) — Goal, Success Criteria #1–#4, and
  the milestone architectural constraint (lines ~3390-3395: extend the kernel, do not fork).

### Pattern to mirror (transition path)
- `ferro-projections/src/executor.rs:22-38` — `TransitionPlan` struct (the data-only plan
  convention `CrudPlan` mirrors).
- `ferro-projections/src/executor.rs:56-115` — `derive_transition_plan(svc, action_name)`
  signature and error modes; `derive_crud_plan` lives in this module and mirrors its shape.
- `ferro-projections/src/lib.rs:17` — re-export site for `derive_transition_plan` /
  `TransitionPlan` (add the CRUD analogs here).

### Kernel to extend (the dispatch seam)
- `framework/src/write/mod.rs:313-436` — `dispatch_write(...)` pipeline (guard re-eval →
  idempotency → confirmation seam → execute → idempotency-store → audit → override hook). The
  CRUD verb is added alongside the transition path here.
- `framework/src/write/mod.rs:142-168` — `WriteDispatcher` + `with_override` registry.
- `framework/src/write/mod.rs:79-135` — `ExecutorFn` / `GuardEvaluatorFn` / `OverrideFn` types.
- `framework/src/write/mod.rs:378-381` — confirmation seam (`!is_confirmed → ConfirmationRequired`).
- `framework/src/write/mod.rs:30-54` — `WriteError` enum.

### CRUD framing layer (where the NTI seam is replaced)
- `ferro-mcp-server/src/write_dispatch.rs:155-180` — current `not_yet_implemented` CRUD
  short-circuit to replace with real derive→dispatch.
- `ferro-mcp-server/src/write_dispatch.rs:123-153, 300-566` — confirmation framing handlers
  (`request_confirm_*` / `confirm_*`) to extend to `delete_<svc>`.
- `ferro-mcp-server/src/renderer.rs:90-155` — CRUD tool emission + transition-only
  confirm-tool synthesis to mirror for delete.
- `ferro-mcp-server/src/jsonrpc.rs:144, 215` — Phase 205 `CallToolResult::structured` envelope
  + regression guard (route CRUD results through this).

### Phase 239/240 substrate (reuse, do not re-derive)
- `ferro-projections/src/service.rs:215-276` — `resolved_table`,
  `resolved_soft_delete_column`, `is_server_injected_field`, `is_write_excluded_field`.
- `ferro-mcp-server/src/dispatch.rs:280-290` — `deleted_at IS NULL` list predicate wiring.
- `ferro-mcp-server/src/schema.rs:249-381` — create/update/delete input-schema builders
  (the column sets `derive_crud_plan` must agree with).

### Tenant lookup (used now for targeting, hardened in 242)
- `framework/src/tenant/scoped.rs:27-41` — `TenantScoped` + `find_for_tenant(id, tenant_id)`.

### Confirmation primitive
- `ferro-ai/src/confirmation/mod.rs:41-75` — `ConfirmationStore` trait (`request_confirmation`,
  single-use `confirm`, TTL).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `derive_transition_plan` + `TransitionPlan` (`ferro-projections/src/executor.rs`) — the
  exact pattern `derive_crud_plan` + `CrudPlan` mirror (pure, serializable, `JsonSchema`).
- `framework::write::dispatch_write` + `WriteDispatcher` + `ExecutorFn`/`OverrideFn` — the
  single channel-agnostic kernel; CRUD extends it, transitions already use it.
- Phase 239 resolver accessors (`resolved_table`, `resolved_soft_delete_column`,
  `is_server_injected_field`) and Phase 240 `is_write_excluded_field` + schema builders —
  the column-set single source of truth `derive_crud_plan` reuses.
- `ConfirmationStore` + `generate_confirmation_token` + the `{tenant, action, record}` token
  binding — reused verbatim for `delete_<svc>`.
- Phase 205 `CallToolResult::structured` envelope — reused for CRUD tool results.

### Established Patterns
- "Derive the default, allow an override hook" — the generic CRUD plan is the default;
  `with_override("create_order", …)` is the 20% seam. No second control surface.
- "One `dispatch_write` kernel, channel only divergence" — MCP and visual/form surfaces share
  the same derived CRUD plan (Phase 232 single-source kernel).
- Plans are *data, not behavior* — `CrudPlan` carries resolved table/columns/predicates, the
  kernel interprets them; no closures in the plan.
- sqlite-in-memory dispatch unit tests are the established test style for the kernel.

### Integration Points
- `ferro-mcp-server/src/write_dispatch.rs:155-180` — the `not_yet_implemented` CRUD seam is
  the single integration point that flips from stub to real dispatch.
- `ferro-projections/src/lib.rs` re-exports — add `derive_crud_plan` / `CrudPlan` / `CrudVerb`.
- `ferro-mcp-server/src/renderer.rs` — synthesize `request_confirm_delete_<svc>` /
  `confirm_delete_<svc>` alongside the existing transition confirm tools.

</code_context>

<specifics>
## Specific Ideas

- "Mirror `derive_transition_plan` with a CRUD plan and run it through the existing kernel"
  (ROADMAP phase title) — the phase is deliberately framed as *analogy + reuse*, not new
  architecture. The strongest signal of success is a grep proving one dispatcher (SC#4).
- The spec's three SQL shapes are the contract for `derive_crud_plan`:
  - Create → `INSERT (creatable cols, status=initial, created_at=now) RETURNING *`
  - Update → `UPDATE … SET <patch> WHERE id=? AND deleted_at IS NULL`
  - Delete → `UPDATE … SET deleted_at=now WHERE id=?`
  (tenant predicate added by Phase 242 — D-09).

</specifics>

<deferred>
## Deferred Ideas

- **Write authorization, tenant injection, non-disclosure** — Phase 242 (CRUD-05). Phase 241
  leaves the `CrudPlan` tenant-predicate slot for 242 to fill (D-09).
- **App `order` projection flip to CRUD + create→list→update→delete e2e over `:8090/mcp` +
  regression-guard extension + `ferro-mcp` catalog/docs** — Phase 243 (integration).
- **Dedicated `get_<svc>` tool** — spec non-goal (covered by `list_` + id filter).
- **Per-field `immutable()`/`read_only()` overrides** — future Track A extension; field sets
  derive from `FieldMeaning` for now.

</deferred>

---

*Phase: 241-derive-crud-plan-wire-crud-verbs-into-framework-write*
*Context gathered: 2026-06-23*
