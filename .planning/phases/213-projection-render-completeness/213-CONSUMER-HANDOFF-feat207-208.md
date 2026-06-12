# Consumer handoff — land COMP-01 migration on ferro 0.2.55 (gestiscilo feat/207 + feat/208)

**Audience:** a gestiscilo session (the consumer repo owns this code; ferro side is
validation-only). **Prereq:** bump gestiscilo to ferro 0.2.55 (the Gap A kanban
structure/content split shipped there). **Source evidence:** ferro Phase 213
integration re-verify (2026-06-13) — both shapes below were proven live against a
local 0.2.55 build.

---

## What ferro 0.2.55 now expects (the contract)

`KanbanBoard` (Process intent) is now **fixed lanes + a flat item array bucketed by
a status field** — not pre-shaped column/children data. `emit_kanban_root` emits:

- `columns` — one lane per state-machine state (id = state name, always rendered)
- `items_path = /data/{service.name}` — a **flat** entity array (same path
  `DataTable`/Browse reads)
- `group_by = <Status field>` — `lane.id == item[group_by]`
- `card_title_key = <EntityName field>` (else identifier), `card_description_key =
  <Money field>`
- `row_actions` / `row_key` — from `ServiceDef.actions` (URL pattern
  `/{service.name}/{row_key}/{action.name}`)

`DataTable` (Browse) unchanged: `data_path = /data/{service.name}`; `ImageUrl`
fields now render as an `<img>` column (Gap D).

---

## feat/207 — Orders kanban (`src/controllers/cassa/orders.rs::index`)

**Problem:** the handler merges `orders` (top-level, pre-shaped
`{id,title,description,actions}`) + `kanban_columns` — the *old* data_path/children
model. Under 0.2.55 the kanban renders empty lanes (no cards) because nothing
resolves at `/data/order`.

**Fix:** feed the flat raw array at `/data/order`. Proven-minimal shape:

```rust
let orders_raw: Vec<serde_json::Value> = flat_orders.iter().map(|o| json!({
    "id":            o.id,
    "status":        o.status.as_str(),     // == lane id: confermato/in_corso/rientrato/chiuso/annullato
    "customer_name": o.customer_name,        // card title  (EntityName)
    "total_cents":   o.total_cents,          // card desc    (Money)
})).collect();

let merged = spec.merge_data(json!({
    "_sidebar": ..., "_header": ..., "_sse_url": ...,
    "data": { "order": orders_raw },          // <-- /data/order
}));
```

Then delete the now-dead `orders` / `kanban_columns` merge keys and the
`build_order_kanban_actions` / `build_order_kanban_description` /
`build_status_kanban_columns` helpers.

**Two open design items (decide consumer-side):**

1. **Card actions are guard-conditional per order** (advance/revert/pay-cash/cancel
   depend on `can_pay`/`can_cancel`/`is_editable`). `KanbanBoardProps.row_actions`
   is *static per spec* with an optional per-item `visible_if` (a boolean field on
   the item). To map the guards: declare the transitions as `ServiceDef.actions`
   and have the handler emit per-order boolean flags (`can_advance`, `can_cancel`,
   …) that the actions reference via `visible_if`. **Or** ship the kanban without
   card actions for v1 (transitions stay on the order detail page) — the board is
   fully usable without them.
2. **`total_cents` binds raw** (renders `1600`, not `€16,00`). For formatting,
   provide a pre-formatted display field and point the card description at it, or
   file a ferro follow-up to format `Money` in card rendering. Minor polish.

**Action-URL prefix gap:** projection-emitted action URLs are `/order/{row_key}/…`,
but gestiscilo routes live under `/dashboard/cassa/ordini/…`. If you wire
`ServiceDef.actions`, the action handler/name must carry the real route (or ferro
needs a base-path mechanism — flag back to ferro if so).

---

## feat/208 — Staff list (`src/controllers/staff/list.rs`)

**Working live on 0.2.55:** Browse data binding (`/data/staff`) and Gap D — the
`avatar_url` (`FieldMeaning::ImageUrl`) field renders as an `<img>` column. (In the
re-verify an *external* test URL got `/storage/`-prefixed and broke; real
storage-key avatar_urls — which the controller already signs at render time — work
correctly. No action.)

**Missing — the projection has nothing to render:** the staff `ServiceDef` declares
**no actions** and no page CTA, so there is no row-actions dropdown and no "Nuovo"
button. ferro's Gap B (DataTable `row_actions` from `ServiceDef.actions`) is
unit-proven; it just isn't exercised here.

**Fix:** declare staff actions on the `ServiceDef`, e.g.:

```rust
let service = ServiceDef::new("staff")
    .display_name("Staff")
    /* …existing fields… */
    .action(ActionDef::new("view"))       // Vedi dettagli
    .action(ActionDef::new("edit"))       // Modifica
    .action(ActionDef::new("toggle"))     // Attiva/Disattiva
    .action(ActionDef::new("delete"));    // destructive
```

→ renders a per-row kebab dropdown with URLs `/staff/{id}/{action}` (apply the
`/dashboard/staff` prefix note above). The page **"Nuovo" CTA** is a page-header
action: confirm whether the projection emits a PageHeader create-action from the
ServiceDef; if not, that is a ferro follow-up (page-level create action) — flag it
back rather than hand-wiring around the projection.

---

## Definition of done (consumer side)

- feat/207 Orders: kanban renders lanes **with cards** bucketed by status against
  real data; decide card-actions item (1) above.
- feat/208 Staff: row-actions dropdown renders from declared actions; "Nuovo" CTA
  resolved (consumer or ferro follow-up).
- Both branches reach merge-worthy parity → COMP-01 (Phase 209) unblocks; merge
  feat/207 + feat/208, retire the probe status.

## Flag back to ferro if you hit
- Action-URL base-path prefixing (projection emits `/{service}/…`, no `/dashboard`).
- Page-level create ("Nuovo") action not emitted by the projection.
- `Money` card-description formatting (raw cents vs localized currency).
