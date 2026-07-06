# Phase 218: Write-Tool Rendering from ActionDef - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults grounded in the v15.0 research docs + phase success criteria; logged in `218-DISCUSSION-LOG.md`)

<domain>
## Phase Boundary

Project each `ServiceDef`'s `ActionDef`s into MCP **write tools that appear in `tools/list`**, derived purely from `ActionDef` — no hand-authored per-tool definitions. Rendering only.

In scope (AMCP-03):
- One write tool per `ActionDef` in each `mcp_exposed` `ServiceDef`; tool name derived from `action.name`.
- Input schema derived from `ActionDef.inputs` via a new `build_action_input_schema(action, service)` (parallel to the read-path `build_input_schema`).
- Guard-filtered: a tool whose precondition evaluates `false` for the calling tenant (`ctx.evaluated_guards`) is omitted from `tools/list`; `true`/absent → present.
- `ToolAnnotations` carry `readOnlyHint: false` + `destructiveHint` derived from `ActionDef` attributes.
- Extend the Phase 205 `CallToolResult`/Tool strict-deserialization regression test to cover every new write tool.

**Out of scope (later phases):** write **dispatch/execution**, server-side guard **re-evaluation at call time**, idempotency, audit log — all Phase 219 (AMCP-04). Confirmation gating + `confirm_<action>` tools — Phase 220. This phase makes write tools **visible and well-formed**, not callable. Calling a write tool still has no executor behind it after 218 (219 adds `dispatch_write`); the 217 scope gate already rejects write tools for `read`-scoped keys before any dispatch would occur.

</domain>

<decisions>
## Implementation Decisions

### Tool naming (D-01)
- **D-01:** Write tool name = `action.name` verbatim (SC#1: derived from `action.name`, no hand-authored overrides in `McpRenderer`). On a name collision across services within one `tools/list`, disambiguate the colliding ones as `<action.name>_on_<service.name>` (ARCHITECTURE Decision (b)). Read tools keep `list_<service.name>`. Write tool names must **not** start with `list_` (action verbs never do) — this keeps the 217 scope gate's `!name.starts_with("list_")` write-detection correct.

### Input schema derivation (D-02)
- **D-02:** Add `build_action_input_schema(action: &ActionDef, service: &ServiceDef) -> Result<Value>` to `ferro-mcp-server/src/schema.rs`. Each `InputDef` → one JSON Schema property: `data_type` via the existing `data_type_to_json_schema()` (currently private — promote to `pub(crate)` or factor a shared helper so both read and write schema builders use the single mapping), `description` → property `description`, `required: true` → name lands in `required[]`. **Inject the parent `ServiceDef`'s first `FieldMeaning::Identifier` field as a required integer param** (the record to act on) per ARCHITECTURE Decision (b). `ActionDef.preconditions` are NOT in the schema — they drive the list-time guard filter only. `ActionDef.effects` are not rendered.
- Field scoping (PITFALLS §3): do not emit inputs/identifiers whose `FieldMeaning` is sensitive (Password/Secret/Token/Sensitive). Researcher to confirm the exact `FieldMeaning` exclusion set already used by the read path's `is_filter_field`.

### Guard filtering (D-03)
- **D-03:** For each `action.precondition` name, check `ctx.evaluated_guards.get(precondition)`. If **any** precondition is explicitly `Some(false)`, omit that action's tool from `tools/list`. Absent key = offer the tool (same semantics as `BaseContext.evaluated_guards` in the v14.0 visual path and the read path). This filter is a **visibility** mechanism, not an authorization gate (PITFALLS §2 — enforcement is the server-side re-check in 219; do not let 218's filter be mistaken for the auth boundary). Tests set `ctx.evaluated_guards` explicitly. The runtime *population* of `evaluated_guards` for the calling tenant (evaluating named guards against tenant/DB state) is a 219 concern — flag for researcher whether a minimal population hook is needed in 218 for an end-to-end list test, or whether explicit-map tests suffice for AMCP-03.

### Annotations (D-04)
- **D-04:** `ToolAnnotations::new().read_only(false).destructive(action.transition_trigger.is_some())` (SC#4: derived from `ActionDef` attributes, not per-tool inference). `transition_trigger.is_some()` is the existing attribute signalling a state transition. **No new `ActionDef` field in 218.** `idempotentHint` is NOT set — there is no `ActionDef` attribute for it and idempotency is a 219 dispatch concern (`idempotency_key`); the roadmap overview's mention of `idempotentHint` is descriptive, not required by SC#4. A dedicated `destructive`/`irreversible`/`requires_confirmation` `ActionDef` flag may be added in Phase 220 (confirmation gating) — 218 deliberately uses only `transition_trigger`.

### Tool list assembly (D-05)
- **D-05:** Extend `render_exposed_tools(services, ctx)`: for each `mcp_exposed` service, emit the existing `list_<service>` read tool, then one guard-passing write tool per `ActionDef`. Order: read tool first, then write tools in `ActionDef` declaration order. The description for a write tool comes from `action.description`, falling back to `action.display_name`, then a generated `"<action.name> <service>"` string. The renderer stays the single source — no per-tool hand-authoring.

### Renderer placement (D-06)
- **D-06:** All rendering logic lives in `ferro-mcp-server` (`renderer.rs` + `schema.rs`) — the existing output-crate home for `McpRenderer` (v11.5 boundary rule). `ferro-projections` is not modified (it already owns `ActionDef`). No new crate.

### SC#5 regression coverage (D-07)
- **D-07:** Extend the Phase 205 strict-deserialization regression test (in `ferro-mcp-server/tests/`, the one asserting `tools`/`CallToolResult` round-trip cleanly through `rmcp`'s strict model types — researcher to pin exact file/test name) so every write tool's definition deserializes strictly via the `rmcp` `Tool` type, catching any malformed `inputSchema`/annotation shape. This is the Phase 205 content-block-bug guard applied to the new write tools.

### Claude's Discretion
- Exact signature/return shape of `build_action_input_schema` (mirror `build_input_schema`).
- Whether `data_type_to_json_schema` is promoted to `pub(crate)` or wrapped in a shared helper.
- Test fixture shape (a service with 1 read + ≥2 actions, one guarded, one with `transition_trigger`).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### v15.0 design
- `.planning/research/ARCHITECTURE.md` §"Decision (b): ServiceDef → Tool Mapping" → "Write tools (actions)" + "Action route/precondition → tool input schema mapping" — the primary design for this phase.
- `.planning/research/PITFALLS.md` §2 (server-side guard bypass — 218's list filter is NOT the auth gate; that's 219), §3 (prompt injection / structured content / sensitive-field scoping — omit sensitive `FieldMeaning` inputs), §5 (destructive confirmation — `destructiveHint` here feeds the 220 confirm flow).
- `.planning/research/FEATURES.md` — "Tool input schema derived from ServiceDef" / "Auto-exposure" (`mcp_exposed` opt-in) rows; "MCP-specific handler code per action" anti-pattern (derive from `ActionDef`, never hand-author).
- `.planning/REQUIREMENTS.md` — AMCP-03 (the requirement this phase closes); AMCP-04/05 (what is deferred to 219/220).

### Phase 217 foundation (just shipped)
- `.planning/phases/217-tenant-context-per-tenant-api-key-auth/217-CONTEXT.md` + `217-SECURITY.md` — `McpContext { tenant_id, evaluated_guards, scope }` and the scope gate that write tools must remain consistent with.

### Code touch-points (read before editing)
- `ferro-projections/src/action.rs` — `ActionDef { name, display_name, description, inputs, preconditions, effects, transition_trigger }`, `InputDef { name, data_type, meaning, required, description }`. Do NOT modify (218 reads these).
- `ferro-projections/src/service.rs` — `ServiceDef.actions: Vec<ActionDef>`, `mcp_exposed`, the `FieldMeaning::Identifier` field.
- `ferro-mcp-server/src/renderer.rs` — `McpRenderer::render` (read tool, `list_<name>`, `read_only(true)`) + `render_exposed_tools` (the extension point for write tools).
- `ferro-mcp-server/src/schema.rs` — `build_input_schema(service)`, private `data_type_to_json_schema(dt)`, `is_filter_field` (sensitive-field exclusion precedent).
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_list` calls `render_exposed_tools(services, ctx)`; the 217 scope gate at `handle_tools_call`.
- `ferro-mcp-server/tests/jsonrpc_integration.rs` — `tools/list` + `structuredContent` assertions; the Phase 205 strict-deser regression test to extend.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `McpRenderer::render` + `render_exposed_tools` (`renderer.rs`): the read-tool path to extend — same `Tool::new(name, description, Arc::new(schema_map)).annotate(annotations)` shape, just `read_only(false)` + per-action iteration.
- `schema::build_input_schema` + `data_type_to_json_schema` (`schema.rs`): the read-path schema builder; `build_action_input_schema` mirrors it over `ActionDef.inputs`. `is_filter_field` shows the sensitive-`FieldMeaning` exclusion pattern to reuse.
- `rmcp::model::{Tool, ToolAnnotations}` — `ToolAnnotations::new().read_only(false).destructive(bool)` is the exact annotation API.
- `ActionDef`/`InputDef` builders and fields are complete and serde-stable — no projection-layer change needed.
- `McpContext.evaluated_guards` (Phase 217) — the guard-filter source, already threaded through `handle_tools_list`.

### Established Patterns
- Opt-in exposure via `ServiceDef.mcp_exposed` (read path already filters on it — write tools inherit the same gate).
- `evaluated_guards` semantics: absent = show, explicit `false` = hide (v14.0/217).
- Single-source rendering: tools derived from the projection, never hand-authored (FEATURES anti-pattern).

### Integration Points
- `handle_tools_list` in `jsonrpc.rs` — where `render_exposed_tools` is called and the (read+write) tool list is assembled.
- The 217 scope gate at `handle_tools_call` already treats any non-`list_` tool as a write tool — 218's action-named tools slot into that gate with no change.

</code_context>

<specifics>
## Specific Ideas

- "Tool definitions are derived purely from `ServiceDef` — no hand-authored per-tool surface" (AMCP-03) is the hard constraint: every write tool's name, schema, and annotations come from `ActionDef`, mechanically.
- 218 produces *visible, well-formed, non-callable* write tools. A reviewer should confirm there is no execution path wired here (that's 219) and that the list filter is not described as authorization.

</specifics>

<deferred>
## Deferred Ideas

- `dispatch_write()` + server-side guard re-evaluation at call time + idempotency key + audit log + typed `CallToolResult::structured` write result — Phase 219 (AMCP-04).
- `ferro-ai` confirmation gating + synthesized `confirm_<action>` tools + TTL — Phase 220 (AMCP-05).
- `idempotentHint` annotation and any explicit `destructive`/`irreversible`/`requires_confirmation` flag on `ActionDef` — revisit in 219/220 where execution/confirmation semantics make them meaningful.
- Inbound NL classification to tool+args — Phase 221 (AMCP-06).

</deferred>

---

*Phase: 218-write-tool-rendering-from-actiondef*
*Context gathered: 2026-06-13*
