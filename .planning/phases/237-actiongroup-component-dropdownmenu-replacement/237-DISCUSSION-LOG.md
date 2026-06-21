# Phase 237: ActionGroup Action Primitive + DropdownMenu Replacement - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-22
**Phase:** 237-actiongroup-component-dropdownmenu-replacement
**Mode:** `--auto` (all gray areas auto-selected, recommended defaults chosen)
**Areas discussed:** Partition rules · Prop shape · DropdownMenu retention boundary · Migration scope · Builtin-count handling · Form-wrapping

---

## Partition / layout rules

| Option | Description | Selected |
|--------|-------------|----------|
| Destructive excluded from cap; hide empty kebab; order=primary | Destructive always overflow+last, never counts toward `max_inline`; kebab hidden when nothing overflows; first item is primary | ✓ |
| Destructive counts toward cap | Simpler counting but defeats the "destructive always in kebab" guarantee | |
| Always render kebab glyph | Consistent affordance but shows an empty menu | |
| Explicit `primary: bool` flag | Second control surface duplicating input order | |

**Auto-selected:** recommended — destructive never counts toward `max_inline`; kebab hidden when overflow empty; primary = input order (D-01–D-04).
**Notes:** Order-as-primary avoids a duplicate control surface (CLAUDE.md no-duplicate-control-surface).

---

## Prop shape

| Option | Description | Selected |
|--------|-------------|----------|
| Seed sketch as-is, `menu_id` required, reuse visible_if semantics | `ActionItem` {label, action, destructive, variant, icon, visible_if}; `ActionGroupProps` {items, menu_id, max_inline, overflow_label, row_key} | ✓ |
| Auto-generate `menu_id` | Removes a required field but breaks popover pairing parity with DropdownMenu | |

**Auto-selected:** recommended — research-seed prop sketch finalized; `menu_id` required (D-05–D-08).
**Notes:** Mirrors `DropdownMenuProps.menu_id`; full `$data` binding + `{row_key}` + `visible_if` parity.

---

## DropdownMenu retention boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Keep render helper internal, remove public component | `render_dropdown_menu` stays `pub(crate)` for ActionGroup overflow + DataTable/Kanban rows; public `DropdownMenu` removed | ✓ |
| Delete kebab rendering entirely | Would force re-implementing the kebab inside ActionGroup and break DataTable rows | |

**Auto-selected:** recommended — internal helper retained, public surface removed (D-09–D-11).
**Notes:** `DropdownMenuAction` retained as internal row-action carrier (rename deferred).

---

## Migration scope

| Option | Description | Selected |
|--------|-------------|----------|
| Migrate codegen + all internal specs + docs | `emit_actions_placeholder` emits ActionGroup; all example/test specs + json-ui docs migrated | ✓ |
| Codegen only, defer docs | Leaves docs referencing a removed public component | |

**Auto-selected:** recommended — full internal migration incl. docs (D-12–D-13).

---

## Builtin-count handling

| Option | Description | Selected |
|--------|-------------|----------|
| One-for-one swap, count stays 47 | +ActionGroup −public DropdownMenu; drift guard stays 47, history comment records swap; mcp mirror stays 47 | ✓ |
| Net +1 (keep DropdownMenu public too) | Contradicts replace-not-wrap | |

**Auto-selected:** recommended — swap, count 47 (D-14).
**Notes:** Verify the `expected` name arrays in catalog.rs and json_ui_catalog.rs both swap the name.

---

## Form-wrapping non-GET inline actions

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-wrap non-GET inline in `<form>` reusing Button form path | POST inline buttons get a CSRF `<form>`; GET = plain link; reuses existing Button form emitter | ✓ |
| Leave bare buttons | Reintroduces the hand-built `form_toggle_active` workaround | |

**Auto-selected:** recommended — auto-wrap reusing existing CSRF form path (D-15).

---

## Claude's Discretion

- `render_action_group` file placement (likely `render/containers.rs`).
- Containers vs atoms section for `BUILTIN_TYPES` registration.
- Internal helper signatures for sharing the kebab renderer.
- Optional `DropdownMenuAction → ActionItem` type alias (cosmetic).

## Deferred Ideas

- Renaming `DropdownMenuAction` across DataTable/Kanban (cosmetic, wide cascade).
- New action semantics (async/optimistic, new confirm variants).
- gestiscilo consumer adoption — separate consumer-repo phase, blocked on published 0.2.72.
