# ferro-a2ui

A2UI renderer for Ferro service projections. **Experimental** — targets the
A2UI v1.0 release candidate and is not published to crates.io; publication
and consumer MCP endpoint wiring are gated on A2UI v1.0 stable.

## What it renders

`A2uiRenderer` implements the `ferro_projections::Renderer` trait: it walks a
`ServiceDef` and its derived intents and emits an A2UI `createSurface` message
— a flat component list with JSON Pointer data bindings. Each of the seven
structural intents (Browse, Focus, Collect, Process, Summarize, Analyze,
Track) maps to an archetype builder; slot ordering follows
`ferro_theme::IntentSlotTemplate`, with theme overrides honored.

## Catalog tiers

- **Basic** (default): every archetype is composed from the A2UI v1.0 Basic
  catalog's 18 primitives, so any spec-compliant client can render it.
- **Ferro** (negotiated): clients that declare the ferro catalog receive rich
  components instead — `DataTable`, `KanbanBoard`, `StatCard`, `LineChart`,
  `BarChart`, `Timeline`. `catalog::ferro_catalog()` returns the catalog
  definition with per-component JSON Schemas.

## Emission modes

Collection archetypes emit in one of two modes:

- **Template**: one shared row template bound via `{path, componentId}`;
  compact, but only guard-invariant actions can be shown.
- **Materialized**: one component subtree per record from
  `A2uiContext::records`, with guard-accurate action buttons per record
  (records may carry `_allowed_actions`). Process defaults to materialized
  when records are supplied.

## Data contract

Rendering returns `SurfaceRendering { messages, catalog_id, data_contract }`.
The `DataContract` lists every JSON Pointer path the skeleton binds (with `*`
wildcards for template scopes); the host fills the surface data model from
the contract alone. An integration test walks every emitted binding across
all archetypes and tiers to guarantee the contract stays complete.

Design: `docs/superpowers/specs/2026-07-04-ferro-a2ui-design.md`.
