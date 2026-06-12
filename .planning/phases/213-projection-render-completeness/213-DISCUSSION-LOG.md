# Phase 213: Projection Render Completeness - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-12
**Phase:** 213-projection-render-completeness
**Mode:** `--auto` (recommended defaults auto-selected; CONTEXT.md pre-existed from the Phase 209 close-out and was kept)
**Areas discussed:** Gap B (actions), Gap A (kanban), Gap C (statcard), Gap D (ImageUrl), Gap E (app-shell), Sequencing, Verification

---

## Gap B — actions slot wiring

| Option | Description | Selected |
|--------|-------------|----------|
| Wire from `ServiceDef.actions` | emit Button/DropdownMenu elements from the existing `actions: Vec<ActionDef>` | ✓ |
| Leave deferred | keep `emit_actions_placeholder` empty | |

**Choice:** wire actions (D-03). Highest leverage — every migrated view is read-only without it; it is why Browse (Staff) fell short of parity despite rendering data. Prioritized first.

## Gap A — Process kanban columns + card binding

| Option | Description | Selected |
|--------|-------------|----------|
| Derive columns from state machine | one `KanbanColumnProps` per `state_machine` state; set `data_path` to bind cards | ✓ |
| Keep single placeholder column | current `emit_kanban_root` behavior | |

**Choice:** derive from state machine, bind cards (D-01). Fallback to single column when `state_machine` is None.

## Gap C — Summarize StatCard value binding

| Option | Description | Selected |
|--------|-------------|----------|
| Bind values via `$data` per stat field | one data-bound StatCard per Money/Quantity read-only field | ✓ |
| Keep empty value | current `value: String::new()` | |

**Choice:** bind values (D-02). Confirm `StatCardProps.value` accepts a data-bound expression; smallest extension if not.

## Gap D — ImageUrl columns

| Option | Description | Selected |
|--------|-------------|----------|
| Render ImageUrl as image column | DataTable column emit includes `FieldMeaning::ImageUrl` | ✓ |
| Keep excluded | current behavior | |

**Choice:** render image column (D-04).

## Gap E — app-shell / layout context

| Option | Description | Selected |
|--------|-------------|----------|
| Document composition pattern now, defer first-class layout context | consumer embeds the projection spec into their layout via merge | ✓ |
| Add layout context to VisualContext now | first-class app-shell slot | |

**Choice:** document composition + defer first-class layout (D-05) unless a gap forces it. Lowest priority.

## Sequencing

**Choice:** B (actions) → A (kanban) → C (statcard) → D (ImageUrl) → E (layout) (D-06). May split into per-gap sub-phases at plan time.

## Verification

**Choice:** per-gap render test (Spec contains the bound component) + re-verification against the gestiscilo probe branches feat/207/feat/208 via the Phase 209 dev-server + Chrome MCP harness (D-07). Phase 207 catalog `derive_intents` invariants must stay green — rendering changes only, never classification (D-08).

## Claude's Discretion

Exact component-prop shapes, the `$data` binding syntax specifics, and whether Gap E ships or defers are Claude's to resolve at plan/execute time, consistent with D-01–D-08.

## Deferred Ideas

- First-class app-shell/layout context (Gap E) if composition docs suffice.
- A chart/visualization `FieldMeaning` (gestiscilo Statistics SVG-chart) — out of scope.
- Resuming/merging the gestiscilo Slice A migrations — happens in the gestiscilo repo after 213 ships and re-verification passes.
