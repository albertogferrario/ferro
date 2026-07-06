# Phase 197: McpRenderer & ferro-mcp-server - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning
**Mode:** --auto (recommended defaults selected; review decisions below)

<domain>
## Phase Boundary

Build the projection→MCP-tool rendering substance of milestone v12.6 — the part
that is genuinely differentiated (the transport and OAuth in later phases are
spec-standard infrastructure). Deliverables:

1. **`ferro-mcp-server`** — a new output crate holding `McpRenderer`, an
   implementation of the `Renderer` trait (`ferro-projections/src/render/mod.rs`),
   mirroring how `JsonUiRenderer` lives in `ferro-json-ui`. `ferro-projections`
   gains no dependency on it (SC-4).
2. **Tool definition from `ServiceDef`** — `McpRenderer::render` produces one MCP
   tool definition for a projection: name + description from the projection,
   `inputSchema` derived solely from the projection's filter + pagination fields
   (SC-2). No separately declared schema.
3. **Opt-in exposure** — a projection marked `mcp_exposed` appears in an
   in-process `tools/list`; an unmarked one does not (SC-1).
4. **Dispatch over the read path** — a dispatch function executes the projection's
   existing read path and returns its rows as MCP structured content, output shape
   derived from the projection (SC-3).
5. **Crate registered** in workspace members + `.github/workflows/publish.yml` at
   the correct wave (SC-5).

**In scope:** the crate scaffold; `McpRenderer` (tool definition); the
`mcp_exposed` marker on `ServiceDef`; input-schema derivation from fields;
dispatch executing a read-only list; an in-process `tools/list` + `tools/call`
exercise (no HTTP server, no OAuth).

**Out of scope:** HTTP transport / `/mcp` endpoint (Phase 198); OAuth browser
login (Phase 199); per-tenant scoping + policy enforcement (Phase 200); write
intents, multi-projection auto-exposure, MCP App UI (later milestones).
</domain>

<decisions>
## Implementation Decisions

### Crate placement & dependency direction (D-01) — AMCP-04, SC-4
- **D-01:** `McpRenderer` lives in a NEW output crate `ferro-mcp-server`, mirroring
  `JsonUiRenderer` in `ferro-json-ui`. Dependency direction is
  `ferro-mcp-server` → `ferro-projections` only; `ferro-projections` gains NO
  dependency on the new crate. Manifest mirrors `ferro-json-ui/Cargo.toml`
  (workspace version/edition/license; `serde`, `serde_json`, `schemars` v1,
  `thiserror` 1.0, `tracing`, `ferro-projections` path dep version "0.2"). Pure
  Rust (no nasm/system libs — CLAUDE.md codec rule).
  - **[auto] recommended default.**

### Opt-in exposure marker (D-02) — AMCP-01, SC-1
- **D-02:** Add `mcp_exposed: bool` (default `false`, `#[serde(default)]`) to
  `ServiceDef` (`ferro-projections/src/service.rs`) plus a builder method
  `.mcp_exposed(true)`. It is plain metadata, not rendering logic — so
  `ferro-projections` gains no renderer dependency (SC-4 holds). `McpRenderer` /
  the in-process `tools/list` includes a projection only when this flag is true.
  - **[auto] recommended default** — chosen over (b) a registration list in
    `ferro-mcp-server`, (c) an `intent_hint`. A bool on `ServiceDef` matches SC-1's
    literal wording and keeps exposure co-located with the projection definition.

### MCP protocol types (D-03)
- **D-03:** Reuse the workspace's existing `rmcp` 0.12 dependency (already used by
  `ferro-mcp` and `ferro-api-mcp`) for the MCP tool-definition representation
  rather than reinventing protocol types — this aligns with the Phase 198
  transport, which will use `rmcp`. Where `rmcp`'s `Tool` type is awkward for pure
  emission, emit a `serde_json::Value` shaped to the MCP tool schema.
  - **[auto] recommended default.**
  - **RESEARCH FLAG:** confirm `rmcp` 0.12's `Tool`/tool-definition ergonomics for
    *producing* (not serving) a tool. If `rmcp`'s Tool is coupled to its server
    runtime in a way that bloats a pure renderer crate, fall back to a minimal
    local tool-definition struct serializing to the MCP tool JSON, and record why.

### inputSchema derivation (D-04) — AMCP-02, SC-2
- **D-04:** The tool `inputSchema` is built from the projection's `ServiceDef`
  fields as a `serde_json::Value` JSON Schema — the single source, with no
  separately declared/validated schema. Composition:
  - **Pagination:** `limit` (integer, default 25, max 100) + `offset` (integer,
    default 0).
  - **Filters:** equality filters derived from a conservative subset of fields —
    readable, non-sensitive fields whose `FieldMeaning` is `Identifier` or a
    foreign-key/reference meaning (and enum-like fields if cheaply detectable).
  - **Type mapping:** `DataType` → JSON Schema type (`Integer`→`integer`,
    `Float`→`number`, `Boolean`→`boolean`, `DateTime`/`Date`→`string` w/ format,
    else `string`).
  Sensitive-meaning fields (password/secret/token/api_key) are never emitted as
  filters.
  - **[auto] recommended default.**
  - **RESEARCH FLAG:** confirm the exact "filter field" predicate against
    `FieldMeaning` variants in `ferro-projections/src/field.rs`; keep it
    conservative for the skeleton (a too-broad filter surface is worse than too
    narrow).

### Dispatch over the read path (D-05) — AMCP-03, SC-3
- **D-05:** A dispatch function — distinct from `Renderer::render` (which only
  produces the static tool definition) — executes the projection's existing
  read path and returns rows as MCP structured content, output shape derived from
  the projection. For Phase 197 (in-process, no HTTP/auth) it resolves the
  projection's source model and runs a read-only list, REUSING an existing read
  mechanism rather than reimplementing query logic. The dispatch signature takes a
  DB connection / read context parameter so Phase 200 can wire tenant scoping +
  policy without changing the signature.
  - **[auto] recommended default.**
  - **RESEARCH FLAG (load-bearing):** identify the exact reusable read path. Candidates:
    `crud_list`-style execution, the `render_projection` data path, or the same
    source-model resolution `projection_coverage` / the checkpoint's
    `field_to_column` seam uses (`src/projections/` ↔ `src/models/` name match +
    `list_models`). Determine how rows are obtained without a running server and
    whether a live DB connection is required for the Phase 197 in-process test
    (vs a fixture). This choice shapes the dispatch API the later phases build on.

### Publish wave (D-06) — SC-5
- **D-06:** Register `ferro-mcp-server` in `Cargo.toml` workspace members and in
  `.github/workflows/publish.yml`. Because it depends on `ferro-projections`
  (currently published in Wave 1B), it must publish AFTER 1B — add it to **Wave 2**
  (alongside `ferro-rs`, `ferro-mcp`) unless research finds it can be a later-1B
  entry. CI publish token is publish-update only; a brand-new crate needs a
  one-time manual bootstrap publish from a local terminal (see
  `project_ferro_publish_token_scoping`).
  - **[auto] recommended default.**

### Claude's Discretion
- Exact module layout within `ferro-mcp-server` (e.g. `renderer.rs`, `tool.rs`,
  `dispatch.rs`, `schema.rs`).
- `McpRenderer`'s associated `Output` and `Context` types (within the `Renderer`
  trait contract) — likely `Output = ` an MCP tool definition, `Context = ` a small
  MCP context struct implementing `Default`.
- Naming of the `tools/list` / `tools/call` in-process exercise (a test or a thin
  public function) used to satisfy SC-1/SC-3 before transport exists.
- Whether `limit` max is 100 vs another conservative cap.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design + requirements (authoritative)
- `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` —
  §"Design properties that shape the approach" (one source of truth; MCP as a
  `Renderer` of the same `ServiceDef`), §Architecture (`ferro-mcp-server` /
  `McpRenderer`), §Non-goals.
- `.planning/REQUIREMENTS.md` §AMCP-01..04.
- `.planning/ROADMAP.md` §"Phase 197" (goal + 5 success criteria).

### Code to read / mirror / extend
- `ferro-projections/src/render/mod.rs` — the `Renderer` trait (`type Output`,
  `type Context: Default`, `fn render(service, intents, ctx)`), `BaseContext`.
  The contract `McpRenderer` implements.
- `ferro-projections/src/service.rs:63` — `ServiceDef` (add `mcp_exposed`).
- `ferro-projections/src/field.rs:60` — `FieldDef`, `DataType`, `FieldMeaning`,
  `infer_meaning` (input-schema derivation + filter-field predicate).
- `ferro-json-ui/` — `JsonUiRenderer` (`src/projection.rs`, exported in
  `src/lib.rs:95`) and `ferro-json-ui/Cargo.toml` — the structural template to
  mirror for the new crate.
- `ferro-mcp/Cargo.toml` + `ferro-mcp/src/service.rs` — existing `rmcp` 0.12 usage,
  `schemars`/`JsonSchema` result-type patterns (tool/result conventions).
- `ferro-api-mcp/src/` — existing `rmcp` server patterns (reference for tool shape).
- `.github/workflows/publish.yml` (lines ~211/246/274) — publish waves
  (1A/1B/2/3); register the new crate.
- `Cargo.toml` (workspace root) — add the crate to `members`.

### Read-path candidates (D-05 research)
- `ferro-mcp/src/tools/crud_operations.rs` (crud_list), `render_projection.rs`,
  `projection_coverage.rs`, and the checkpoint's `field_to_column` source-model
  resolution in `ferro-mcp/src/tools/checkpoint_projection.rs`.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The `Renderer` trait is already modality-agnostic with associated `Output`/
  `Context` — `McpRenderer` plugs in exactly as `JsonUiRenderer` does; no trait
  changes needed.
- `rmcp` 0.12 is already a workspace dependency (ferro-mcp, ferro-api-mcp) — MCP
  protocol types are available without adding a new external dependency.
- `schemars` v1 + `serde_json` are the established schema/JSON tooling; `DataType`
  already has a column-type inference helper to mirror for JSON-Schema mapping.
- `ferro-json-ui` is the exact precedent for an output crate that optionally/does
  depend on `ferro-projections` and implements a renderer — copy its manifest and
  module conventions.

### Established Patterns
- Renderers live in their output crate; `ferro-projections` stays renderer-free
  (CLAUDE.md rendering-architecture rule).
- New workspace crate → add to `publish.yml` in the correct wave (memory:
  workspace-crate convention); new crate needs manual bootstrap publish (token is
  publish-update only).
- One Error enum per crate (`thiserror`), serde enums `rename_all = "snake_case"`.

### Integration Points
- `ServiceDef` gains one bool field + builder (the only `ferro-projections` edit).
- The dispatch read path connects to whatever existing read mechanism research
  selects — this is the seam later phases (transport in 198, tenant scoping in 200)
  build on, so its signature should anticipate a connection/context parameter.
</code_context>

<specifics>
## Specific Ideas

- The differentiator this phase delivers (vs hand-written MCP tools elsewhere in
  the ecosystem) is **one source of truth**: the same `ServiceDef` that renders to
  JSON-UI yields the MCP tool's input schema and output shape. SC-2 is the guard —
  there must be NO second, separately-declared schema. Tests should assert the
  inputSchema is derived (e.g. adding a field changes the schema with no other
  edit).
- Keep the filter surface conservative; per-tenant scoping and policy gating are
  Phase 200 — Phase 197 dispatch must not bake in any ad-hoc ownership filter that
  would later compete with the policy layer (no duplicate control surface).
- Annotation taxonomy to adopt when shaping the tool definition (borrowed from the
  MCP ecosystem, protocol-standard): `readOnly` is the correct hint for this
  read-only skeleton tool.
</specifics>

<deferred>
## Deferred Ideas

- HTTP transport, OAuth, per-tenant scoping, write intents, multi-projection
  auto-exposure, MCP App UI — all later phases/milestones (see ROADMAP).
- `rmcp` adoption for the serving layer is decided here only insofar as the tool
  *definition* type; the transport runtime choice is Phase 198.

None blocking — discussion stayed within phase scope.
</deferred>

---

*Phase: 197-mcprenderer-ferro-mcp-server*
*Context gathered: 2026-06-10*
