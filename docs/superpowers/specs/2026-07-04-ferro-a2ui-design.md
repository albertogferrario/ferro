# ferro-a2ui: A2UI Renderer Design

**Date:** 2026-07-04
**Status:** Phases 1–3 delivered (wire types, archetype builders across both
catalog tiers and emission modes, action-event mapping) as an experimental,
unpublished workspace crate built against the v1.0 RC. MCP endpoint wiring
and crates.io publication (phases 4–5) gated on A2UI v1.0 stable (see
Implementation Trigger).
**Supersedes:** scope sketch in `.planning/backlog/2026-07-04-a2ui-renderer-assessment.md`

## Summary

`ferro-a2ui` is a new output crate implementing the `Renderer` trait to emit
[A2UI](https://a2ui.org/) surfaces — Google's open protocol (Apache 2.0,
`a2ui-project` org) for agent-generated declarative UI. It projects the same
`ServiceDef` + derived intents that ferro-json-ui, ferro-text, and the MCP
renderer already consume, adding A2UI as a sibling Layer-3 concrete output.
Delivery runs over the consumer MCP endpoint using the documented
A2UI-over-MCP pattern.

## Protocol snapshot (as of 2026-07-04)

- Production spec: **v0.9.1**; **v1.0 Release Candidate** published
  (adds client→server RPC via `actionResponse`, action IDs, server→client
  `callFunction`, renames `theme` → `surfaceProperties`). v1.0 stable
  targeted Q4 2026 with a renderer certification program.
- Wire format: flat component array with ID adjacency, exactly one component
  with `id: "root"`, JSON Pointer data binding, template child binding
  (`children: { path, componentId }`), streamed messages with one top-level
  key each. MIME type `application/a2ui+json`.
- Catalogs are first-class: a catalog is a `catalogId` URI + Markdown
  `instructions` + component JSON Schemas + declared functions. Clients
  negotiate via `supportedCatalogIds` / `acceptsInlineCatalogs`. The "Basic"
  catalog (~17 primitives) is the one with open-source renderers.
- Transports: formal A2A extension; A2UI-over-MCP is a documented pattern
  (embedded resources in tool results); a formal MCP extension is under
  consideration upstream.
- Client surfaces: Gemini Enterprise (pinned to spec v0.8), Google Opal,
  Flutter GenUI SDK, CopilotKit React, stable Lit/Angular/React web renderers.
- No official Rust SDK exists; community crates are early-stage.

This design targets the **v1.0 RC wire format**. Implementation proceeds
against the RC; the Wire-Delta Watchlist below must be re-verified against
the stable spec before the crate is published or wired into the consumer
MCP endpoint.

## Goals

1. Render any MCP-exposed projection as an A2UI surface derived from
   `ServiceDef` + `derive_intents()` — same source of truth as every other
   renderer.
2. Deliver surfaces through the consumer MCP endpoint alongside the existing
   CRUD tool results, per the A2UI-over-MCP pattern.
3. Route A2UI action events into the existing write kernel
   (WriteDispatcher, Gate abilities, guard evaluation) — no parallel write
   path.
4. Degrade gracefully to the Basic catalog on any conformant client, and
   negotiate up to a richer ferro catalog where supported.

## Non-goals

- Replacing the json-ui wire format or server-side HTML rendering.
- Shipping a client-side renderer for the ferro catalog (server emission
  only; the catalog definition enables future client work).
- A2A transport support in v1 (message types stay transport-agnostic; the
  extension slot is preserved).
- Publishing to crates.io or wiring into the consumer MCP endpoint before
  v1.0 stable. The crate carries `publish = false` and stays out of
  `publish.yml` until the gate flips (the CI publish token cannot publish
  new crates; first publish is a local bootstrap).

## Architecture

### Position in the render stack

```
ServiceDef ──derive_intents()──► [IntentScore]
                                      │
        ┌──────────────┬──────────────┼──────────────┐
        ▼              ▼              ▼              ▼
  JsonUiRenderer   McpRenderer   TextRenderer   A2uiRenderer
  (ferro-json-ui)  (ferro-mcp-   (ferro-text)   (ferro-a2ui)
                    server)
        │              │              │              │
     Spec (HTML)    Tool[]         String     SurfaceRendering
```

JSON-UI and A2UI are sibling concrete outputs; neither derives from the
other. Shared derivation layers are reused, not duplicated:
`ferro-theme::IntentSlotTemplate` drives slot ordering,
`ferro-projections::FieldMeaning` drives input dispatch. Only component
emission is A2UI-specific.

### Crate layout

`ferro-a2ui` — publish Wave 1b. Dependencies: `ferro-projections`,
`ferro-theme`, `serde`, `serde_json`, `thiserror`. No dependency on
`ferro-json-ui`.

```
ferro-a2ui/src/
  lib.rs        A2uiRenderer + re-exports
  message.rs    A2uiMessage: CreateSurface | UpdateComponents |
                UpdateDataModel | DeleteSurface | ActionResponse | CallFunction
  component.rs  Component { id, component, props }, Dynamic{String,Number,Bool},
                template child binding
  catalog.rs    Basic-catalog constants + ferro catalog definition
  context.rs    A2uiContext { intent_index, tier, emission_mode, surface_id,
                templates: Option<ThemeTemplates>, send_data_model }
  builder/      one module per intent archetype (browse, focus, collect,
                process, summarize, analyze, track)
  actions.rs    ActionDef → action-event mapping
  error.rs      Error enum (thiserror)
```

### Renderer implementation

```rust
impl Renderer for A2uiRenderer {
    type Output = SurfaceRendering;
    type Context = A2uiContext;
}

pub struct SurfaceRendering {
    pub messages: Vec<A2uiMessage>,   // createSurface + component skeleton
    pub catalog_id: String,           // tier actually emitted
    pub data_contract: DataContract,  // bound JSON Pointer paths + shapes
}
```

`DataContract` is the seam between schema-level rendering and live data. The
renderer emits a static skeleton with JSON Pointer bindings; the host
application fills the `dataModel` from the same query path the MCP CRUD read
tools use, and pushes subsequent `updateDataModel` messages without
re-sending components. The output is a message list (not a document) so the
reactive subset of the protocol stays reachable (see Streaming).

## Component mapping

Root convention: exactly one component `id: "root"` per surface. Collections
use A2UI template binding — the direct analog of json-ui's `$each`.

### Emission modes

A2UI's mechanism for conditional UI is that the server owns the component
stream: components that do not apply are not sent, and state changes flow as
`updateComponents`. Two modes per archetype:

- **Template mode** (default for Browse, Track): one shared row template via
  template binding. Only guard-invariant actions may appear inside a
  template, since all rows render identically.
- **Materialized mode** (default for Process, available everywhere): the
  host renders per-record components with live data. Each record's component
  carries only its currently-valid actions (guards evaluated against the
  record), and state changes are streamed as `updateComponents`. No dead
  controls are ever emitted.

`A2uiContext::emission_mode` selects the mode; guard-dependent actions are
omitted (not disabled) in template mode.

### Archetype table

| Intent | Basic tier | ferro catalog tier |
|---|---|---|
| Browse | `List` + row-template `Card(Row(Text…))`; "load more" `Button` event (no pagination primitive) | `ferro:DataTable` |
| Focus | `Card(Column(` label/value `Row` pairs `))` + action `Button`s | — Basic suffices |
| Collect | `Column` of inputs by `FieldMeaning`: FreeText → `TextField`, Bool → `CheckBox`, Enum/Status → `ChoicePicker`, DateTime → `DateTimeInput`, bounded numeric → `Slider`; submit `Button` | — Basic suffices |
| Process | `Row` of per-state `Column`s; host pre-groups dataModel into `/lanes/N/items` (no client-side filtering in A2UI) | `ferro:KanbanBoard` |
| Summarize | Grid of `Card(Column(Text` value / label `))` | `ferro:StatCard` |
| Analyze | Degrades to stats + tabular list (charts unavailable in Basic) | `ferro:LineChart`, `ferro:BarChart` |
| Track | `List` of `Row(Text` timestamp, `Icon`, `Text)` | `ferro:Timeline` |

Tier selection is driven by negotiated `supportedCatalogIds`; the builder is
archetype-first so tier is a strategy parameter, not a code fork.

## Value formatting

Deliberate deviation from client-native formatting: the host pre-formats
display values (Money, Percentage, DateTime) into the dataModel as display
strings, alongside raw values for inputs and two-way binding. Rationale:
deterministic output across clients, server-authoritative locale handling,
and the Basic catalog's thin client-function registry. Where a negotiated
catalog declares formatting functions, the builder may bind those instead.
Trade-off acknowledged: clients cannot reformat pre-formatted strings using
platform conventions.

## Actions

### Event mapping

`ActionDef` and CRUD verbs map to A2UI action events named identically to
the MCP tools (`create_order`, `update_order`, `mark_paid_order`, …) — one
action vocabulary across modalities, one dispatch layer.

```json
{ "action": { "event": {
    "name": "mark_paid_order",
    "context": { "id": { "path": "id" } },
    "wantResponse": true } } }
```

Row-scope actions resolve `id` via relative binding inside template scope;
form submission carries the `/form` scope as context.

### Dispatch and responses

Incoming action events route into the existing write kernel: WriteDispatcher
executor (find-then-mutate, tenant-scoped), Gate `mcp_write_ability`
pre-authorization, live guard evaluation, override hooks. On success the
server replies `actionResponse { value }` and refreshes affected data via
`updateDataModel` (materialized mode may also send `updateComponents`).

### Validation errors

On validation failure: `actionResponse { error }` plus `updateDataModel`
writing per-field messages into `/form/errors/<field>` paths. The skeleton
includes `Text` components bound to those paths (empty string renders
nothing). User input survives by construction — A2UI two-way binding keeps
entered values in the client-side dataModel, giving `with_old_input()`
semantics without server round-tripping of field values.

### Confirmation

Destructive actions wrap in `Modal` (`entryPointChild` = trigger button,
`contentChild` = confirmation column whose confirm button fires the event
with `confirmed: true` context). The Modal covers human-in-the-loop
confirmation on A2UI surfaces; the two-step `request_confirm_* / confirm_*`
MCP tool flow remains for agent-initiated (non-surface) calls. Server-side
enforcement is identical in both paths.

## ferro catalog definition

- Versioned `catalogId` URI, e.g. `https://ferro-rs.dev/a2ui/catalog/v1`
  (final URI decided at implementation time; must be stable and versioned).
- Components: `DataTable`, `KanbanBoard`, `StatCard`, `LineChart`,
  `BarChart`, `Timeline` — JSON Schemas derived from the corresponding
  json-ui component prop shapes.
- Drift guard: a cross-crate test (hosted in the unpublished `app` crate to
  avoid dev-dependency cycles in publish waves) asserts the ferro catalog
  schemas stay consistent with the json-ui catalog.
- The catalog's `instructions` field (Markdown consumed by LLM clients) is
  authored to the same quality bar as MCP tool descriptions — it is part of
  the framework's agent-facing surface.

## MCP integration

Follows the documented A2UI-over-MCP pattern:

- **Delivery:** CRUD read tool results (`list_<service>`, …) are enriched
  with an `EmbeddedResource` (`application/a2ui+json`,
  `annotations.audience: ["user"]`) carrying the surface, alongside the
  `structuredContent` JSON the model reads. The agent sees data; the human
  sees an interactive surface. No new per-service tools.
- **Negotiation:** A2UI client capabilities read from the MCP `initialize`
  handshake (`capabilities.a2ui.clientCapabilities`), with per-call `_meta`
  fallback for stateless clients. `supportedCatalogIds` drives tier
  selection; no declared A2UI support → no embedded resource (zero overhead
  for non-A2UI clients).
- **Return path:** the guide's generic `action` tool (name, context,
  surfaceId) routed into WriteDispatcher; the `error` tool records
  client-side rendering errors. Tool names follow the upstream guide for
  client interoperability.
- **Changes:** `McpContext` gains negotiated A2UI capabilities;
  tool-schema rendering (`render_exposed_tools`) is unchanged; result
  assembly in the endpoint handler gains the surface-enrichment step; the
  host app supplies `dataModel` from the same query path as the list tools.

## Streaming and live surfaces (future work)

- v1 ships the static subset: one `createSurface` with the full skeleton and
  initial dataModel per tool result. Spec-conformant; no live channel over
  plain MCP request/response.
- Materialized-mode reactivity (host re-renders on state change, diffs, and
  emits `updateComponents` / `updateDataModel`) becomes deliverable when a
  streaming channel exists: MCP resource subscriptions, the formal MCP
  extension under consideration upstream, or the A2A transport extension.
- The live read-model runtime (`ferro-projection`) already produces per-key
  snapshots and broadcast deltas; mapping those deltas onto
  `updateDataModel` for subscribed surfaces is the natural follow-on and
  requires no changes to this design's message model.

## Testing

1. Serde round-trip tests against canonical JSON examples vendored from the
   pinned spec version.
2. Snapshot tests per intent × tier × emission mode.
3. DataContract completeness: a test walks emitted components collecting
   every `{ "path": … }` reference and asserts each appears in the
   DataContract (and vice versa for required bindings).
4. Catalog drift guard (see ferro catalog definition).
5. Upstream conformance suite: evaluate running emitted surfaces through the
   a2ui repo's conformance checks in CI once implementation starts.
6. End-to-end: the existing `:8090` MCP harness exercises
   negotiation → enriched tool result → `action` tool → `actionResponse`,
   including the validation-error flow.

## Fidelity edges (accepted)

- Pagination is an action ("load more"), not a component, in the Basic tier.
- Charts do not exist in the Basic tier; Analyze degrades to tabular stats.
- Guard-dependent actions are omitted in template mode (full guard fidelity
  requires materialized mode).
- Layouts, design meta, lint results, and theme tokens do not cross the
  wire — A2UI clients own styling; ferro's design governance constrains
  structure at emission time. `surfaceProperties` carries only
  `agentDisplayName` / `iconUrl`, populated from `APP_NAME` / `APP_URL` per
  the project-agnostic crate convention.

## Wire-delta watchlist (re-verify against v1.0 stable)

- MIME type `application/a2ui+json` (changed once already, in v0.9.1).
- `surfaceProperties` shape (renamed from `theme` in the RC).
- `actionResponse` / `actionId` semantics and required-ness.
- `callFunction` / `functionResponse` contract and catalog function
  declarations (`callableFrom`).
- Component prop renames (`variant` family, `TextField` props,
  `ChoicePicker` shape).
- Capability negotiation payloads (`server_capabilities` /
  `client_capabilities`, `acceptsInlineCatalogs`).
- Any formal MCP extension superseding the documented embedded-resource
  pattern.

## Implementation trigger and phasing

Phases 1–3 proceed now against the v1.0 RC (experimental, unpublished).
**Hard gate for phases 4–5:** A2UI v1.0 stable published (upstream target:
Q4 2026), preceded by a Wire-Delta Watchlist pass. Signals that raise
priority: a formal A2UI MCP extension lands; a mainstream consumer client
surface becomes broadly available; a ferro consumer application requests
A2UI output.

1. Wire types + serde round-trip against the RC spec.
2. Builder: archetypes × tiers × emission modes; DataContract.
3. Actions + write-kernel dispatch; validation-error flow.
4. *(gated)* MCP enrichment + negotiation; `:8090` e2e.
5. *(gated)* Catalog publication + drift guard + docs (`docs/src/`), MCP
   tool description updates, `publish = false` removal + `publish.yml`
   Wave 1b entry + local publish bootstrap.
