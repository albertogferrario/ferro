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
- Read authorization: `mcp_ability` (policy Gate) + key scope. Existing *action*
  writes deliberately skip the policy Gate because action names do not map 1:1 to
  a service; they rely on scope gate + live guard re-evaluation.

### Shipped foundations Track A builds on (Phases 231 + 232)

Track A does **not** introduce a dispatch architecture — it extends the one already
shipped and verified:

- **Phase 231 (StateMachine-derived executor):** `ferro-projections/src/executor.rs`
  exposes `derive_transition_plan(svc, action)` — the default executor (state-read →
  guard re-eval → transition → persist) derived from the `StateMachine` declaration
  alone. A **post-persist override hook** (`OverrideFn` + `WriteDispatcher.with_override`
  registry) handles the app-specific 20%. Undeclared transitions are rejected at boot
  by `ServiceDef::validate()` (step 5).
- **Phase 232 (single-source write kernel):** the channel-agnostic kernel lives at
  `framework/src/write/mod.rs` — `dispatch_write(action, …, channel)` plus
  `ExecutorFn`, `GuardEvaluatorFn`, `OverrideFn`, `WriteDispatcher`. It backs BOTH the
  MCP and visual/form surfaces; the hand-written `WriteDispatcher` `match` was retired.
  Idempotency, audit (channel-parameterized), confirmation, guard re-eval, and tenant
  isolation are already in this kernel.

**Consequence:** our earlier-stated decisions "derived default + override hook" and
"retire the hand-written `WriteDispatcher`" describe the *shipped* state, but only for
**state-transition actions**. Track A's job is to add the **CRUD analog** to that same
machinery — not to rebuild it (rebuilding would create the duplicate write-control
surface ferro's conventions forbid).

### Reusable assets

- `framework::write::dispatch_write` + `WriteDispatcher`/`ExecutorFn`/`OverrideFn` —
  extend, do not fork.
- `ferro-projections::derive_transition_plan` — the pattern to mirror with a
  `derive_crud_plan`.
- `ServiceDef::validate()` — extend with the write-ability/creatable checks.
- `TenantScoped` + `find_for_tenant(id, tenant_id)` (from Phase 212) — tenant-scoped
  lookup for update/delete targeting.

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

**Startup validation (fail-fast):** extend the existing `ServiceDef::validate()`
(which already rejects undeclared transitions, Phase 231) with a check: if any of
`creatable/updatable/deletable` is true but `mcp_write_ability` is unset, `validate()`
returns `Err` — a config error at boot, never a silent deny at call time.

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

## Dispatch architecture (extend `framework::write`, do not rebuild)

Phases 231/232 already ship the derived-default + override-hook kernel for
transitions. Track A adds the **CRUD analog** inside the same kernel — it does not
introduce a parallel dispatcher.

**The CRUD derivation (new):** add `derive_crud_plan(svc, verb, inputs)` to
`ferro-projections/src/executor.rs`, mirroring `derive_transition_plan`. It produces a
pure, serializable plan the kernel executes:

- *Create* → `INSERT (creatable cols, tenant_id=ctx, status=initial, created_at=now) RETURNING *`
- *Update* → `UPDATE … SET <patch> WHERE id=? AND tenant_id=ctx AND deleted_at IS NULL`
- *Delete* → `UPDATE … SET deleted_at=now WHERE id=? AND tenant_id=ctx`

**Kernel wiring (extend):** `framework::write::dispatch_write` is keyed on `ActionDef`
today. Track A teaches the kernel a CRUD verb alongside the existing transition path —
either by representing `create/update/delete` as derived `ActionDef`s with a CRUD plan,
or by adding a thin verb discriminant. The existing `ExecutorFn` runs the derived plan;
**idempotency, channel-parameterized audit, confirmation, and tenant context are reused
unchanged** (delete sets `destructiveHint` → the existing confirmation path).

**Override hook (reuse):** the existing `WriteDispatcher::with_override(action, hook)`
registry covers CRUD verbs with no new mechanism — register
`with_override("create_order", …)` for the app-specific 20% (validation, computed
fields, side-effects). The generic derived plan is the default when no override is
registered; `make_write_dispatcher` stays "register overrides," not "implement
everything."

**Single-source preserved:** because the kernel is channel-agnostic (Phase 232), the
same derived CRUD plan backs the MCP surface and the visual/form surface — one
declaration, every modality, exactly as transitions already do.

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

## Tool-result envelope (already structured — extend coverage, not a fix)

The `content[]` bare-object bug was already fixed in **Phase 205**: `handle_tools_call`
emits `CallToolResult::structured(payload)` → one `type:text` block + `structuredContent`
+ `isError:false` (`ferro-mcp-server/src/jsonrpc.rs:144`), with a regression guard
(`jsonrpc.rs:215` `tools_call_result_parses_as_valid_mcp_content`). So this is **not** an
open prerequisite. The only Track A obligation is to route the new `create_/update_/delete_`
results through the same `structured` envelope and extend the regression guard to assert a
well-formed `content[]` for each new verb.

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

1. Data model + `deleted_at` migration
2. `ServiceDef` declaration surface + `validate()` extension (write-ability check)
3. Schema derivation (C/U/D inputSchema + query polish)
4. `derive_crud_plan` in `ferro-projections` + wire CRUD verbs into the existing
   `framework::write` kernel (reusing override registry / idempotency / audit /
   confirmation) — **extends 231/232, does not rebuild the dispatcher**
5. Authorization wiring (scope + write-ability Gate + tenant injection + delete confirmation)
6. App integration + e2e (both MCP and visual surfaces, since the kernel is shared) +
   extend the `tools/call` structured-envelope regression guard to the new CRUD verbs +
   `ferro-mcp` catalog/docs

## Non-goals (YAGNI)

- Dedicated `get_<svc>` (covered by `list_` + id filter) — documented, deferred.
- Per-field `immutable()`/`read_only()` overrides — future extension; Track A
  derives field sets from `FieldMeaning`.
- Tracks B/C/D — separate cycles.
