# A2UI renderer assessment (ferro-a2ui)

**Captured:** 2026-07-04
**Design:** complete — see `docs/superpowers/specs/2026-07-04-ferro-a2ui-design.md`
**Implementation:** in progress against the v1.0 RC as an experimental,
unpublished workspace crate (`publish = false`, not in `publish.yml`).
**Gate for MCP wiring + publication:** A2UI v1.0 **stable** published
(upstream target Q4 2026; v1.0 is a Release Candidate today, v0.9.1 is the
production spec), preceded by a Wire-Delta Watchlist pass. Priority-raising
signals: a formal A2UI MCP extension lands upstream; a mainstream consumer
client surface becomes broadly available; a ferro consumer application
requests A2UI output.
**Next re-check:** 2026-10-01 (start of the upstream Q4 stable window) —
verify v1.0 stable status, the MCP extension (upstream issue #648), and
consumer client surfaces; if the gate fired, run the design's Wire-Delta
Watchlist, then wire + publish.

## Status check (2026-07-04)

- v1.0 RC published (client→server RPC `actionResponse`, action IDs,
  `callFunction`, `theme` → `surfaceProperties`); stable targeted Q4 2026
  with a renderer certification program.
- Governance moved to a neutral `a2ui-project` GitHub org; active weekly
  commit cadence; CopilotKit remains the named non-Google partner.
- A2UI-over-MCP is now a documented upstream pattern (embedded resources in
  tool results, `application/a2ui+json`, generic `action`/`error` tools,
  capability negotiation via `initialize` or `_meta`); a formal MCP
  extension is under consideration.
- No mainstream consumer client surface yet: Gemini Enterprise is pinned to
  spec v0.8; Opal, Flutter GenUI, CopilotKit, and the web renderers are the
  broadest surfaces.
- No official Rust SDK; community crates are early-stage.

## What

A2UI (a2ui.org, Apache 2.0) is an open protocol for agent-generated
declarative UI: flat streaming JSON component model with ID adjacency,
client-declared component catalogs, JSON Pointer data binding, and actions
streamed back to the agent. It runs over A2A and MCP.

## Why it maps onto ferro cleanly

- Its component model (flat map + ID adjacency + catalog + declarative-only
  security) is structurally equivalent to ferro-json-ui's spec shape —
  independent convergence on the same design. Template child binding is the
  direct analog of json-ui's `$each`; JSON Pointer binding of `$data`.
- The `Renderer` trait is modality-agnostic; A2UI is a natural additional
  output crate (`ferro-a2ui`), alongside ferro-json-ui (HTML), ferro-text,
  and the MCP renderer, projecting the same `ServiceDef`/specs.
- Ferro apps already expose a per-tenant consumer MCP endpoint; the
  documented A2UI-over-MCP pattern lets those endpoints return interactive
  surfaces to A2UI clients alongside the JSON the model reads.

## Decided design (summary — full spec in docs/superpowers/specs/)

- Direct renderer: `A2uiRenderer` walks `ServiceDef` + intents, reusing
  `ferro-theme::IntentSlotTemplate` and `FieldMeaning` dispatch; no
  ferro-json-ui dependency. Output = messages + catalog id + DataContract.
- Tiered catalogs: Basic-catalog composition by default; negotiated
  versioned ferro catalog (DataTable, KanbanBoard, StatCard, charts,
  Timeline) via `supportedCatalogIds`.
- Two emission modes: template (shared row template, guard-invariant actions
  only) and materialized (per-record components, guard-accurate actions,
  `updateComponents` on state change).
- Actions reuse MCP tool names and dispatch through the existing write
  kernel (WriteDispatcher, Gate abilities, guards); validation errors via
  `actionResponse { error }` + error paths in the dataModel.
- MCP delivery: enrich CRUD read tool results with an `EmbeddedResource`
  surface when the client negotiates A2UI support; generic `action`/`error`
  tools for the return path.

## Near-term inspiration (independent of the trigger)

- Progressive/streaming spec rendering — the flat element map already
  permits incremental emit.
- Catalog capability negotiation — client declares supported components;
  relevant to embedded rendering contexts.

## Explicit non-goals

- Replacing the json-ui wire format or server-side HTML rendering.
- Publishing or wiring into the consumer MCP endpoint before v1.0 stable.
- Shipping a client-side renderer for the ferro catalog.
