# Phase 243: App integration, e2e, envelope guard & catalog/docs - Context

**Gathered:** 2026-06-24
**Status:** Ready for planning
**Mode:** `--auto` (gray areas auto-selected; recommended defaults chosen, logged in DISCUSSION-LOG)

<domain>
## Phase Boundary

The final Track A phase: prove the whole CRUD data surface (delivered by Phases 239–242)
end-to-end against the sample app, and bring the introspection/docs surface to the same
quality bar as the Rust API. Four deliverables, no new framework capability:

1. **App projection flip** — turn the app's `order` projection into a CRUD projection
   (`.creatable/.updatable/.deletable` + `.mcp_write_ability`) and drive a full
   create → list → update → delete cycle through the MCP path, with the same derived plan
   succeeding on the visual/form surface (shared `framework::write` kernel).
2. **Structured-envelope regression guard extension** — extend the existing Phase 205
   `CallToolResult::structured` `content[]` assertions to each new verb
   (`create_`/`update_`/`delete_`).
3. **Confirmation-flow e2e** — `delete_<svc>` without a valid token → `confirmation_required`
   echoing `request_confirm_delete_<svc>`; with a valid token → soft-delete.
4. **Catalog/docs** — update `ferro-mcp` authoring surface (`code_templates`,
   `generation_context`) and `docs/src/` so an authoring agent learns the CRUD opt-in and the
   derived tool set (create/update/delete + `list_` query polish).

**Not in scope:** any new framework capability (the kernel, schemas, authz, tenant injection,
and non-disclosure all shipped in 239–242 — this phase only exercises and documents them);
the developer-facing `ferro-mcp/src/tools/crud_operations.rs` introspection tool (a SEPARATE
surface — see D-08). No requirement is uniquely owned here; the phase validates CRUD-01..07
end-to-end.
</domain>

<decisions>
## Implementation Decisions

### E2E drive surface (SC#1)
- **D-01:** The **CI regression gate** is the in-process `handle_tools_call` harness — an
  in-memory SQLite DB + full `Migrator::up` + a `read_write`-scoped `McpContext`
  (`write_authorized: Some(true)` for the authorized case) — mirroring the established
  `app/src/tests/mcp_write_dispatch.rs` and `single_source.rs` patterns. It drives
  create → list → update → delete, the confirmation flow, the per-verb envelope assertions,
  and the MCP↔visual parity. This keeps CI free of a live HTTP server + bearer auth and is
  testable without live spend (consistent with the AMCP-06 "CI-testable" principle).
- **D-02:** The **live `:8090/mcp` + seeded `read_write` bearer drive** named in SC#1 is a
  **documented manual UAT smoke** (the reusable `:8090` + chrome-mcp harness), recorded as a
  HUMAN-UAT item — NOT a blocking CI gate. The in-process harness already exercises the same
  kernel and the same `McpContext` scope/authorization path, so the live drive is
  confirmation, not the gate.
- **D-03:** MCP↔visual parity reuses the `single_source.rs` (Phase 232) approach: the same
  derived `CrudPlan` is exercised through the MCP framing and the visual handler, asserting
  identical persisted effects (channel string the only divergence).

### App `order` projection flip (SC#1)
- **D-04:** Add `.creatable(true).updatable(true).deletable(true)` and
  `.mcp_write_ability("manage-orders")` to `app/src/projections/order.rs`, keeping the
  existing read `.mcp_ability("view-orders")` and `.tenant_column("tenant_id")`. The write
  ability `"manage-orders"` matches the existing test fixtures in `ferro-mcp-server`.
- **D-05:** The `order` projection has a StateMachine (`order_lifecycle`, initial `draft`), so
  per the shipped schema-derivation rules: `create_order` sets `status` server-side to `draft`
  (excluded from the create input); `status` is never an `update_order` input (workflow-only,
  driven by the existing `submit`/`approve`/`ship` transition actions); `id`, `created_at`, and
  `tenant_id` are excluded from write inputs. `soft_delete_column` defaults to `deleted_at`
  (the orders table got it in Phase 239 — verify the migration + entity sync include it).
- **D-06:** `validate()` must pass at boot for the flipped projection (CRUD-07: write verbs
  enabled WITH `mcp_write_ability` present). The host `mcp.rs` write-ability path (Phase 242)
  resolves `order` → `manage-orders` → `Gate::authorize_for`.

### Structured-envelope regression guard (SC#2)
- **D-07:** Extend the existing envelope assertion pattern (in `mcp_tenant_isolation.rs`:
  `result["result"]["content"][0]["type"] == "text"` plus a populated `structuredContent`) to
  assert a well-formed `content[]` for each of `create_order`, `update_order`, and
  `delete_order` results, so the Phase 205 envelope shape is regression-pinned for every CRUD
  verb (not just reads/transitions).

### Catalog/docs scope (SC#4)
- **D-08:** Update the **authoring-facing** surface so an agent reading the project learns the
  CRUD opt-in: `ferro-mcp` `code_templates` (add the
  `.creatable/.updatable/.deletable/.mcp_write_ability` projection pattern and the derived
  `create_/update_/delete_/list_`-with-query-polish tool set) and `generation_context` if it
  enumerates capabilities, plus a `docs/src/` projection-CRUD section.
- **D-09:** Do **NOT** conflate the projection-derived consumer-MCP CRUD tools with the
  developer-MCP `ferro-mcp/src/tools/crud_operations.rs` (a separate model-aware SQL
  introspection tool for `ferro mcp`). They are different surfaces; this phase documents the
  projection-CRUD opt-in, not `crud_operations.rs`.
- **D-10:** Verify the json-ui **builtin-component drift-guard count** is NOT falsely tripped:
  the new CRUD tools are MCP tools, not json-ui components, so no component was added and the
  component-count guards must remain unchanged. Only touch tool/catalog/docs counts that
  actually enumerate the CRUD tool surface.

### Claude's Discretion
- Exact test-module layout and fixture naming; whether confirmation-flow assertions are gated
  `#[cfg(feature = "confirmation")]` (follow the `single_source.rs` precedent for destructive
  paths); the precise wording of the docs/code_templates additions.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Spec & requirements
- `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md` — Track A design;
  the end-to-end acceptance + SQL-shape + authorization sections.
- `.planning/ROADMAP.md` §"Phase 243" (lines 3518–3539) — goal + four success criteria + the
  "exercises CRUD-01..07 end-to-end" coverage note.
- `.planning/REQUIREMENTS.md` — CRUD-01..07 (this phase validates all; owns none uniquely).

### The projection to flip
- `app/src/projections/order.rs` — the `order` `ServiceDef` (has `tenant_column`,
  `mcp_ability`, StateMachine, transition actions; lacks the CRUD flags + `mcp_write_ability`).
- `app/src/migrations/mod.rs` — the orders `deleted_at` migration (Phase 239 substrate).

### E2E harness + envelope guard + parity (extend these, do not rebuild)
- `app/src/tests/mcp_write_dispatch.rs` — in-process write-dispatch e2e harness
  (`setup_db()` + `seed_two_tenants()` + `handle_tools_call`); the model for the CRUD e2e.
- `app/src/tests/mcp_tenant_isolation.rs` (≈ lines 278–290, 351–363) — the
  `content[0].type=="text"` + `structuredContent` envelope-assertion pattern (D-07 extends it).
- `app/src/tests/single_source.rs` — MCP↔visual single-source parity (Phase 232 EXEC-05);
  the model for D-03's CRUD parity check.

### Write path (already shipped — exercised, not modified)
- `ferro-mcp-server/src/write_dispatch.rs` — CRUD verb routing + the
  `request_confirm_`/`confirm_` prefix handlers + the `is_crud_write_tool` write-ability gate
  (Phase 242).
- `framework/src/write/mod.rs` — `execute_crud_plan` (tenant-scoped, soft-delete,
  non-disclosure).

### Authoring/docs surface to update (SC#4)
- `ferro-mcp/src/tools/code_templates.rs` — add the CRUD opt-in projection template.
- `ferro-mcp/src/tools/generation_context.rs` — capability enumeration (if applicable).
- `ferro-mcp/src/tools/crud_operations.rs` — **separate developer-MCP tool; do NOT conflate**
  (D-09 — read only to confirm the boundary).
- `docs/src/` — projection/MCP CRUD documentation section.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **In-process MCP e2e harness** (`mcp_write_dispatch.rs`): `setup_db()` (in-memory SQLite +
  `Migrator::up`), `seed_two_tenants()`, test-local `WriteDispatcher`, `handle_tools_call` with
  an explicit `McpContext`. The CRUD e2e is a new test module in the same style.
- **Envelope-assertion pattern** (`mcp_tenant_isolation.rs`): the exact `content[]` +
  `structuredContent` shape locks; copy it per CRUD verb.
- **Single-source parity** (`single_source.rs`): drives one declaration through MCP + visual,
  asserts identical effects; gated `not(feature = "confirmation")` for destructive paths.
- **The shipped CRUD write path**: derive → `dispatch_write` → `execute_crud_plan`, confirmation
  prefix routing, tenant predicate, non-disclosure — all present; this phase only calls it.

### Established Patterns
- Tests use an explicit `db` arg + test-local dispatcher to stay isolated from the global pool.
- Destructive-flow tests are feature-gated on `confirmation` (the two-step confirm path).
- The `order` projection already declares `tenant_column` + read `mcp_ability` — the flip is
  additive (four builder calls).

### Integration Points
- `app/src/projections/order.rs` (the flip) → flows into the host `mcp.rs` write-ability path
  (Phase 242) and the schema-derivation/`derive_crud_plan` path (Phases 240/241).
- New test module under `app/src/tests/` registered in the app test module tree.
- `ferro-mcp` tool outputs + `docs/src/` (authoring surface; quality bar = Rust API).
</code_context>

<specifics>
## Specific Ideas

- The live `:8090` + bearer drive is real and useful, but as a manual UAT smoke, not a CI gate
  (D-02) — CI stays on the in-process harness to avoid live-server/bearer overhead.
- Keep the projection-CRUD consumer tools mentally separate from `crud_operations.rs` (D-09) —
  the docs must not blur the two surfaces.
- The component-count drift guards are about json-ui components; CRUD tools are not components,
  so those counts must stay unchanged (D-10) — a tripped guard here would be a false positive.
</specifics>

<deferred>
## Deferred Ideas

None — this is the integration/closeout phase for Track A; all framework capability shipped in
239–242. Tracks B–D of the broader MCP capability program remain a future milestone, not this
phase.
</deferred>

---

*Phase: 243-app-integration-e2e-envelope-guard-catalog-docs*
*Context gathered: 2026-06-24*
