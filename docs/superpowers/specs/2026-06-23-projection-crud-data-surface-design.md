# Projection CRUD Data Surface — Design

Date: 2026-06-23
Status: Approved (design); pending implementation plan
Scope: Track A of a four-track "new MCP tools & capabilities" roadmap

## Context: four-track roadmap

"New tools and capabilities" decomposes into four independent capability tracks,
each its own spec → plan → build cycle. Ordered by ferro's substance-first
investment ordering (compressive → operational → conceptual) and killer-feature
density:

1. **Track A — Complete the data surface** *(this spec; foundational, highest
   compression).* Auto-derive full CRUD from a projection.
2. **Track B — Richer write semantics.** Bulk actions, multi-step intents,
   undo/compensation, confirmation polish. Builds on A's write path.
3. **Track C — New capability classes.** Aggregation/analytics tools,
   cross-projection/relational queries, live subscriptions. Overlaps the existing
   `ferro-projection` live read-model runtime — must not duplicate it.
4. **Track D — Agent-experience meta-tools.** Discovery, `generation_context`,
   "what can I do / what changed" introspection. Amplifies A–C; comes last.

**Killer feature of the batch (one sentence):** a single projection declaration
derives a complete, safe, tenant-scoped agent interface — read, query, write, and
workflow — with zero hand-written tool code. Today that is half-true: `list_<svc>`
(with equality filters) and per-`ActionDef` write tools derive; create, richer
query, update, and delete do not. Track A closes that gap.

## Current state (verified)

- `handle_tools_list` derives one read tool per `mcp_exposed` projection, named
  `list_<svc>`, with **equality-filter params already derived** from
  `is_filter_field` (Identifier, ForeignKey, Status, Category, Boolean, Custom).
  Get-by-id already works as `list_` + id equality filter.
- Write tools derive one per `ActionDef`; `build_action_input_schema` injects the
  identifier field plus declared inputs.
- The app's `WriteDispatcher` is a "find-then-mutate" closure — **no insert or
  delete path**.
- Read authorization: `mcp_ability` (policy Gate) + key scope. Existing *action*
  writes deliberately skip the policy Gate because action names do not map 1:1 to
  a service; they rely on scope gate + live guard re-evaluation.

## Track A scope

Full CRUD: **create, read (+query polish), update, soft-delete.**

### Decisions

- **Create model:** opt-in `.creatable(true)`; creatable fields auto-derived from
  existing `field()` declarations (single source of truth). Maximally compressive
  while keeping explicit per-projection control.
- **Update:** data fields only. **State-machine-controlled fields stay
  workflow-only** — when a projection has a `StateMachine`, the `Status` field is
  never a create or update input; state changes happen exclusively through action
  tools. (When no SM exists, a `Status` field is an ordinary writable field.)
- **Delete:** soft-delete (a `deleted_at` column), confirmation-token gated,
  filtered out of `list_`.
- **Query polish:** add range/comparison filters, sort, and pagination to `list_`
  (equality filters already derive and are unchanged).
- **Authorization:** one `.mcp_write_ability` gates create/update/delete via the
  policy Gate and requires `read_write` key scope; delete additionally requires a
  confirmation token.
- **Dispatch:** derived generic default + per-verb override hook (ferro's
  "derive the default, allow an override hook" pattern).

## Declaration surface

New consuming builder methods on `ServiceDef` (mirroring the existing `with_*`
`mut self -> Self` idiom):

```rust
ServiceDef::new("order")
    .table("orders")                    // field->column binding; default pluralize(name), explicit wins
    .creatable(true)                    // derive create_order
    .updatable(true)                    // derive update_order (data fields only)
    .deletable(true)                    // derive delete_order (soft, confirmation-gated)
    .mcp_write_ability("manage-orders") // Gate ability for C/U/D — REQUIRED if any are true
    .soft_delete_column("deleted_at")   // default "deleted_at"
    // existing fields / state_machine / actions unchanged
```

**Startup validation (fail-fast):** if any of `creatable/updatable/deletable` is
true but `mcp_write_ability` is unset, the projection fails to load — a config
error at boot, never a silent deny at call time.

## Derived tool surface

Per opted-in projection, with no further code:

| Tool | Input schema (auto-derived) | Notes |
|------|------|------|
| `create_<svc>` | creatable fields | excludes Identifier, CreatedAt, tenant column (server-injected), and Status when an SM exists (set to initial state); `Sensitive` excluded |
| `update_<svc>` | identifier (required) + updatable fields (all optional → patch) | Status excluded when an SM exists |
| `delete_<svc>` | identifier (required) + confirmation token | soft-delete; `destructiveHint=true` |
| `list_<svc>` (enhanced) | existing equality filters + range ops `<field>__{gt,gte,lt,lte,ne,in}`, `sort` (`field`/`-field`), `limit`/`offset` | equality params unchanged for back-compat |

Field sets derive from the existing `field()` declarations and `FieldMeaning`, so
a projection authored for reads yields correct write schemas for free.

## Dispatch architecture (derived default + override hook)

Unify writes behind a `WriteOp`:

```rust
enum WriteOp {
    Create { fields: Map },
    Update { id: Value, fields: Map },    // patch
    Delete { id: Value },                 // soft
    Action { name: String, inputs: Map }, // existing transitions
}
```

- **Generic default (framework):** derives sea-orm statements from `ServiceDef` +
  `.table()`:
  - *Create* → `INSERT (creatable cols, tenant_id=ctx, status=initial, created_at=now) RETURNING *`
  - *Update* → `UPDATE … SET <patch> WHERE id=? AND tenant_id=ctx AND deleted_at IS NULL`
  - *Delete* → `UPDATE … SET deleted_at=now WHERE id=? AND tenant_id=ctx`
- **Override hook:** a registry keyed by `(service, verb)`; a registered custom
  handler runs instead of the generic path (validation, computed fields,
  side-effects), with the same closure shape as today's dispatcher. The existing
  `make_write_dispatcher` becomes "register overrides," not "implement
  everything."

## Data-model requirements

- Add nullable `deleted_at` to every soft-deletable table (migration). All
  read/update/delete paths filter `deleted_at IS NULL`.
- `created_at` set on create; `tenant_id` always injected from context, never
  accepted as input — the tenant column is excluded from every write schema, so an
  agent cannot set it.

## Authorization & safety

| Tool class | Scope gate | Policy Gate | Extra |
|---|---|---|---|
| `list_` / read | read or read_write | `mcp_ability` (existing) | — |
| `create_` / `update_` | read_write only (read rejected) | `mcp_write_ability` via `Gate::authorize_for(user, ability)` | tenant injected/predicated |
| `delete_` | read_write only | `mcp_write_ability` | + confirmation token |

- The policy Gate applies cleanly because `create_/update_/delete_<svc>` strip 1:1
  to a service name (the reason actions skip the Gate does not apply).
- Tenant safety: create injects `tenant_id` from `ctx`; update/delete carry
  `AND tenant_id = ctx`. Cross-tenant or soft-deleted targets are indistinguishable
  from "not found" (existing D-09 non-disclosure envelope — no row/column/filter
  leakage).

## Error handling (tool-result envelope, not transport errors)

- Missing required create field / type mismatch → validation error naming the
  field(s).
- Update/delete of a row absent in tenant (or soft-deleted) → non-disclosing
  "not found / denied."
- `delete_` without a valid confirmation token → `confirmation_required`,
  echoing a `request_confirm_delete_<svc>` affordance.
- Read scope calling a write tool → scope-denied.

## `tools/call` `content[]` fix — in-scope prerequisite

Every new tool returns through `tools/call`. The current response builds a
bare-object `content[]` (missing `type`) that MCP clients Zod-reject. Track A is
worthless if its tool results cannot be consumed, so fixing the `CallToolResult`
content shape is an in-scope prerequisite task, with a regression test asserting
well-formed `content[]` on every verb. This also unblocks the existing read path.

## Testing strategy

- **Unit (table tests):** creatable/updatable field-set derivation; Status
  inclusion/exclusion with vs without an SM; range/sort/pagination param
  derivation; write-ability startup validation.
- **Unit (sqlite in-memory, matching existing dispatch tests):** each verb's
  generic SQL; tenant injection (create ignores any attempt to set tenant;
  update/delete cross-tenant denied); soft-delete hides from `list_`; override hook
  replaces the generic path.
- **Authz:** read scope rejects writes; write-ability Gate deny; delete without
  confirmation.
- **Integration / e2e:** flip the app's `order` projection to
  `.creatable/.updatable/.deletable`; drive create → list → update → delete through
  `:8090/mcp` with a seeded bearer key.
- **Surface parity:** update `ferro-mcp` `json_ui_catalog`/`code_templates` and
  docs to reflect the new tools (same quality bar as the Rust API).

## Within-Track sequencing (each a plan/phase)

1. `tools/call` `content[]` fix + regression (unblocks everything)
2. Data model + `deleted_at` migration
3. `ServiceDef` declaration surface + startup validation
4. Schema derivation (C/U/D inputSchema + query polish)
5. `WriteOp` unification + generic CRUD dispatch + override hook
6. Authorization wiring (scope + write-ability Gate + tenant injection + delete confirmation)
7. App integration + e2e + `ferro-mcp` catalog/docs

## Non-goals (YAGNI)

- Dedicated `get_<svc>` (covered by `list_` + id filter) — documented, deferred.
- Per-field `immutable()`/`read_only()` overrides — future extension; Track A
  derives field sets from `FieldMeaning`.
- Tracks B/C/D — separate cycles.
