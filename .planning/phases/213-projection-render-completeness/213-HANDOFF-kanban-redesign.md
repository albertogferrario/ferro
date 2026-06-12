# Phase 213 — Handoff: KanbanBoard structure/content redesign (Gap A root fix)

**Status as of 2026-06-12:** Gaps B, A, C, D, E are implemented and committed; all ferro unit tests green. The gestiscilo **integration re-verify of Gap A (Orders kanban) found a real bug** the unit tests missed. The fix is a small renderer redesign, scoped below. Gap B/D re-verify on the Staff branch (feat/208) is **not yet done**.

## What works (committed, do not redo)
- 213-01..213-05 committed on ferro `master`. Gap B (actions DropdownMenu + row_actions), Gap C (StatCard `value_path`), Gap D (`ColumnFormat::Image`) all unit-tested green; docs (Gap E composition pattern) written. `cargo test -p ferro-json-ui` (605) + `cargo test -p ferro-projections --test catalog` (22 frozen) pass.
- Gap A builder change is committed: `emit_kanban_root` (ferro-json-ui/src/projection/builder.rs ~L395) DOES derive one `KanbanColumnProps` per `state_machine` state. Verified live: the gestiscilo Orders spec now carries 5 columns (confermato/in_corso/rientrato/chiuso/annullato). **The builder column derivation is correct — keep it.**

## The bug (why Gap A renders blank end-to-end)
Live test: gestiscilo `feat/207` Orders page `/dashboard/cassa/ordini` rendered **blank** despite the spec having 5 columns. Root cause in **`ferro-json-ui/src/render/containers.rs::render_kanban_board` (~L334-348)**:
- `emit_kanban_root` sets BOTH `columns` (5 static, from state machine) AND `data_path: Some("/data/{name}/columns")`.
- The renderer treats `data_path` as a **replacement** for `columns`: it resolves the path, gets an empty array (the gestiscilo handler provides `orders` + `kanban_columns` at top level, NOT `/data/order/columns`), `unwrap_or_default()` → empty → `if columns.is_empty() { return String::new() }` → **blank board.** The 5 static columns are discarded.

Confirmed via browser: `columnCount:5, dataPath:"/data/order/columns", hasDataAtPath:false, renderedColumns:0`.

## The design flaw (the real issue — user-directed: "no workarounds, root fixes")
`KanbanBoardProps` conflates two different concerns into the `columns`/`data_path` pair:
- **Column STRUCTURE** (which lanes, order, labels) = schema → from the state machine. Fixed.
- **Column CONTENT** (cards + counts) = runtime data → from handler entities.

`data_path` wholesale-replaces the column list, so structure and content fight over one input. A band-aid (fall back to static columns when `data_path` is empty — was tried and **reverted** per user direction) only makes empty lanes show; cards still can't populate. **Do it properly.**

## The proper design (implement this)
A kanban is **fixed lanes + items sorted into them by a status field.** Refactor `KanbanBoardProps`:
- `columns: Vec<KanbanColumnProps>` — **structure only** (id + title), from the state machine. Always rendered.
- `items_path: Option<String>` — JSON-pointer to a **flat array** of entities (gestiscilo handler already merges `orders`).
- `group_by: Option<String>` — the field on each item that selects its column (the `Status` field, e.g. `status`; for orders the column id = the status value).

Then:
1. **component.rs** — update `KanbanBoardProps`: keep `columns`; add `items_path` + `group_by`; remove the column-replacing semantics of `data_path` (repurpose/rename `data_path` → `items_path`, or drop `data_path` and add the two new fields). Keep it additive/serde-skip-none. Keep `Catalog::validate` + catalog tests green.
2. **containers.rs `render_kanban_board`** (~L334) — render the static `columns` ALWAYS; resolve `items_path` → flat array; bucket each item into the column whose id == `item[group_by]`; compute per-column count; render each item as a card (reuse existing column-children/card rendering). No more "data_path replaces columns / blank when empty."
3. **builder.rs `emit_kanban_root`** (~L395) — keep the state-machine column derivation; set `items_path` to the entity list path (e.g. `/data/{name}/items` or `/{name}` — must match where the handler merges the flat list; see gestiscilo seam below) and `group_by` to the name of the field with `FieldMeaning::Status` that drives the state machine.
4. **Tests** — the current Gap A unit test `kanban_root_derives_columns_from_state_machine` asserts `data_path == Some("/data/{name}/columns")` — **that encoded the wrong contract; update it** to assert `items_path` + `group_by`. Add a **renderer** test (containers.rs) that feeds static columns + an items array and asserts items land in the correct lanes with correct counts (this is the test the unit suite was missing — it's why the bug shipped).

## Gestiscilo integration seam
- `feat/207` Orders handler (src/controllers/cassa/orders.rs) merges `orders` (flat array, each with a `status` field) + `kanban_columns` (old controller-built, now unused by the projection). Decide the `items_path`/`group_by` so the projection consumes the existing `orders` array + `status` field. Minor handler tweak may be needed so the flat orders land at the `items_path` the builder emits (or set the builder's `items_path` to match the handler's `orders` key). Document and keep feat/207 edits minimal — it stays a probe branch (unmerged).
- Order status values in the data must match column ids (state names): confermato/in_corso/rientrato/chiuso/annullato. Verify the handler's `status` values match the state machine state names (they should — same domain).

## Re-verify harness (and the cargo gotcha that cost a build)
To test local ferro against gestiscilo (probe branches pin ferro 0.2.54 from crates.io; local workspace is **0.2.55**):
1. `git checkout feat/207-orders-projection-migration` in `/Users/alberto/repositories/gestiscilo-it/app` (ensure clean tree first — its Cargo.lock can block checkout; `git checkout -- Cargo.toml Cargo.lock`).
2. Append `[patch.crates-io]` to gestiscilo Cargo.toml pointing the **15 direct ferro deps** (ferro-rs→framework, ferro-json-ui, ferro-whatsapp, ferro-ai, ferro-storage, ferro-notifications, ferro-events, ferro-wallet, ferro-reservation, ferro-audit, ferro-orm, ferro-projection, ferro-broadcast, ferro-deployments, ferro-assets) to their local workspace paths under `/Users/alberto/repositories/albertogferrario/ferro/`. (NOT ferro-stripe — it's independently 0.9.0.)
3. **CRITICAL GOTCHA:** `ferro serve` alone will NOT pick up the patch — the existing Cargo.lock already satisfies `^0.2.54` with registry 0.2.54, so cargo never re-resolves and the patches report "**not used in the crate graph**" (you run the old 0.2.54 binary and waste a full build). You MUST first force re-resolution: `cargo update ferro-rs ferro-json-ui ferro-whatsapp ferro-ai ferro-storage ferro-notifications ferro-events ferro-wallet ferro-reservation ferro-audit ferro-orm ferro-projection ferro-broadcast ferro-deployments ferro-assets` (transitive ferro-projections/ferro-theme/etc. follow via the workspace). Confirm `grep -A3 'name = "ferro-json-ui"' Cargo.lock` shows `version = "0.2.55"` before building.
4. `ferro serve --backend-only` (port 8080, from gestiscilo dir, `PATH` includes `~/.cargo/bin`). Big build (~30 local ferro crates).
5. Login: navigate `http://localhost:8080/dashboard/cassa/ordini`; if redirected to `/accedi`, submit email `jetskiadriatic@gestiscilo.it` (magic-link dev auto-login → lands on /dashboard). Chrome MCP profile `chrome-devtools-3` (others' profiles are locked).
6. **Cleanup after:** `git checkout -- Cargo.toml Cargo.lock` on the branch, `git checkout master` — keep feat/207 + feat/208 pristine and unmerged.
7. To see cards (not just empty lanes), insert a couple of orders for tenant `jetskiadriatic` (business id 3) at different statuses via `sqlite3 database.db` (delete them after — leave the DB clean; staff probe rows were already cleaned).

## Remaining after the Gap A fix
- Re-verify **feat/208 Staff** (Gap B + D): `/dashboard/staff` should show a row-actions dropdown (View/Edit/Toggle/Delete) + page "Nuovo" CTA + `avatar_url` rendered as an image column. Insert a couple staff rows (tenant 3) to see them (Browse data binding was already confirmed working in Phase 209). Same harness.
- Then ROADMAP SC#6 (both probe branches reach functional parity) is met → finish Phase 213 (`/gsd-verify-work 213` or mark complete).

## State
- ferro `master`: 213-01..05 committed; band-aid reverted; only this handoff + the blank-render screenshot (`screenshots/after213-orders-kanban.png`) are new/uncommitted (committing with this doc).
- gestiscilo: on master, clean; feat/207 + feat/208 preserved unmerged.
- `213-CONTEXT.md` D-01 originally said "set data_path /data/{name}/columns" — that decision is **superseded** by this handoff (structure/content split). Update D-01 when implementing.
