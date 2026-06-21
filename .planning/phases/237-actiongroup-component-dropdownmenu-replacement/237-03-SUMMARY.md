# Plan 237-03 Summary — Projection codegen + docs migration + dead-code removal

**Status:** Complete
**Requirements:** SC-5
**Completed:** 2026-06-22

## What was built

The last in-tree DropdownMenu consumers migrated to ActionGroup, and the now-dead
`render_dropdown_menu` helper removed.

**Code (landed in the interrupted Wave 2 run — commits `5807f370` etc.):**
- `ferro-json-ui/src/projection/builder.rs` — `emit_actions_placeholder` now builds
  `Vec<ActionItem>` + `ActionGroupProps` and emits `element_with_props("ActionGroup", props)`.
  Import swapped to `ActionGroupProps, ActionItem` (kept `DropdownMenuAction` for DataTable/Kanban
  emitters). Its test renamed to `actions_slot_emits_action_group_from_service_actions`, decoding
  `ActionGroupProps`.
- `ferro-json-ui/src/render/atoms.rs` — `render_dropdown_menu` and its two tests
  (`dropdown_menu_emits_actions`, `dropdown_menu_get_action_renders_anchor`) **deleted** (dead after
  Plan 02 removed the dispatch arm; DataTable/Kanban use `render_inline_dropdown`). `render_menu_item`
  retained; `DropdownMenuProps` removed from the atoms import. Clean under `-D warnings`.

**Docs (this session — commit `9f6948bf`):**
- `docs/src/json-ui/components.md` — Forms-category row `DropdownMenu` → `ActionGroup`; the
  `### DropdownMenu` section replaced with a neutral `### ActionGroup` section documenting D-05/D-06
  props (`items`/`$data` binding, required `menu_id`, `max_inline` default 2, `overflow_label` default
  "Azioni", `row_key`; per-item `label`/`action`/`destructive`/`variant`/`icon`/`visible_if`) plus a
  JSON example showing inline buttons + a kebab + a destructive item rendered last.
- `docs/src/features/projections.md` — page-level action note `DropdownMenu` → `ActionGroup`.
- `docs/src/json-ui/expressions.md` — correlated-children example `DropdownMenu` → `ActionGroup`.
- `docs/book/*` left untouched (generated mdbook output, regenerated from `docs/src/`).

## Refinement to CONTEXT D-10

D-10 originally said keep `render_dropdown_menu`. Verified-current-code analysis (RESEARCH Open
Q1, resolved) showed it is dead after the dispatch-arm removal — the reused building blocks are
`render_menu_item` + the kebab trigger/panel HTML (embedded in `render_action_group`), not
`render_dropdown_menu`. CONTEXT D-10 was updated accordingly; this plan's deletion is consistent.

## Verification

- `grep -rn "DropdownMenu" docs/src/` — **empty**.
- `grep -rn "DropdownMenuProps" ferro-json-ui/src/` — **empty** (public props fully gone).
- `grep -n 'element_with_props("ActionGroup"' ferro-json-ui/src/projection/builder.rs` — matches.
- `grep -n "fn render_dropdown_menu" ferro-json-ui/src/render/atoms.rs` — **none**; `render_menu_item` retained.
- `cargo test -p ferro-json-ui` — 620 passed (incl. `actions_slot_emits_action_group_from_service_actions`).
- `cargo clippy -p ferro-json-ui -p ferro-mcp --all-targets -- -D warnings` — clean (no dead-code/unused-import).

## Self-Check: PASSED
