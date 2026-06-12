---
phase: 213-projection-render-completeness
plan: 06-gap-a-root-fix
subsystem: ferro-json-ui
tags: [json-ui, projection, kanban, structure-content-split, integration-verified]
dependency_graph:
  requires: [213-01, 213-02, 213-HANDOFF-kanban-redesign]
  provides: [kanban-structure-content-split, orders-kanban-content-bound]
  affects:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/render/data.rs
    - ferro-json-ui/src/projection/builder.rs
tech_stack:
  added: []
  patterns: [prescribed-card-field-key-binding, renderer-side-bucketing]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/render/data.rs
    - ferro-json-ui/src/projection/builder.rs
    - docs/src/json-ui/components.md
    - docs/src/json-ui/data-binding.md
    - docs/src/json-ui/expressions.md
decisions:
  - "KanbanBoardProps mirrors MediaCardGridProps (prescribed card + field-key bindings), NOT a new control surface — resolves the no-duplicate-control-surface concern; $each stays the escape hatch for custom card structure"
  - "Renderer buckets a flat items array by group_by; handler stays flat (items_path=/data/{name}, same path DataTable reads) — bucketing lives in the projection/renderer layer, not the handler (projection thesis)"
  - "data_path (column-replacing semantics) removed — it was the blank-board root cause; columns are now always-rendered structure"
metrics:
  duration: ~90m (incl. integration re-verify)
  completed_date: "2026-06-13"
  tasks_completed: 4
  tasks_pending: 0
---

# Phase 213-06 — Gap A root fix: KanbanBoard structure/content split

## Problem

The 213-01..05 work was unit-green but the gestiscilo integration re-verify
(handoff `213-HANDOFF-kanban-redesign.md`) found the Orders kanban rendered
**blank**. Root cause: `KanbanBoardProps` conflated lane *structure* and card
*content* in one `columns`/`data_path` pair. `emit_kanban_root` set both static
`columns` (from the state machine) and `data_path`; the renderer treated
`data_path` as a wholesale replacement, resolved it empty (the handler provided
a flat `orders` array, not `/data/order/columns`), and `if columns.is_empty()`
returned an empty string — discarding the 5 valid columns. Unit tests missed it
because no renderer test fed static-columns + a data path together.

## Fix (structure/content split)

`KanbanBoardProps` now models a kanban as **fixed lanes + items sorted into them
by a status field**, mirroring `MediaCardGridProps`:

- `columns` — lane structure (id + title), **always rendered**.
- `items_path` — JSON-pointer to a flat entity array.
- `group_by` — field selecting the lane (`column.id == item[group_by]`).
- `card_title_key` / `card_description_key` / `row_actions` / `row_key` — the
  prescribed-card field bindings (reuse `data.rs` dropdown/templating helpers,
  promoted to `pub(crate)`).
- `data_path` **removed**.

`render_kanban_board` renders the static columns always, buckets the
`items_path` array by `group_by`, computes per-lane counts, and renders each
item as a card. A new renderer test (`render_kanban_board_buckets_items_into_columns`)
covers the path the unit suite was missing. `emit_kanban_root` emits
`items_path=/data/{name}` (same flat path `DataTable` reads — handler stays
flat), `group_by`=the `Status` field, card keys from `EntityName`/`Money`, and
`row_actions` from `service.actions`.

## Why this design (no duplicate control surface)

`$each` already templates custom cards into a fixed `KanbanColumn`. The
field-key card binding here is **not** a second copy of that — it is the same
convention `DataTable` and `MediaCardGrid` already use for prescribed cards.
`KanbanBoard` now joins them under one convention; `$each` remains the escape
hatch for fully-custom card structure.

## Validation

- **Unit:** `ferro-json-ui` 608 passed; `ferro-projections --test catalog`
  22 frozen passed; `cargo fmt` clean; `cargo clippy -p ferro-json-ui
  --all-targets` clean. Full ferro workspace compiled cleanly via the gestiscilo
  integration builds (feat/207 + feat/208).
- **Integration (feat/207 Orders, live):** the Orders kanban now renders all 5
  lanes (confermato/in_corso/rientrato/chiuso/annullato) with correct per-lane
  counts; 5 seeded orders bucketed into the right lanes by `status`; card title
  bound `customer_name`, description bound `total_cents`. Blank-board pathology
  gone. Screenshot: `screenshots/after213fix-orders-kanban-cards.png`. A minimal
  throwaway handler tweak (feed `/data/order` flat) drove the check; reverted —
  feat/207 stays pristine/unmerged.
- **Integration (feat/208 Staff, live):** Browse data binding + Gap D
  (`avatar_url` ImageUrl → `<img>` column) confirmed rendering. Gap B
  (DataTable row-actions) accepted on unit coverage — the staff probe declares
  no service actions, so nothing renders; that is a **feat/208 consumer-wiring
  gap** (logged below), not a ferro defect.

## Follow-ups (consumer-side, gestiscilo)

- feat/207 Orders: handler should merge raw orders at `/data/order` with
  `status` + bindable fields (the temp probe edit shows the shape); per-order
  guard-conditional actions (advance/revert/pay/cancel) need a row-level
  `visible_if` story to map onto `KanbanBoardProps.row_actions`.
- feat/208 Staff: declare `.action()`s on the staff `ServiceDef` (+ a page
  "Nuovo" CTA) to surface the row-actions dropdown the projection supports.
