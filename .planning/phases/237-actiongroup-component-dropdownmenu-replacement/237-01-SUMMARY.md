---
phase: 237-actiongroup-component-dropdownmenu-replacement
plan: "01"
subsystem: ferro-json-ui
tags: [json-ui, component, action-group, render, tdd]
dependency_graph:
  requires: []
  provides: [ActionItem, ActionGroupProps, render_action_group]
  affects: [ferro-json-ui/src/component.rs, ferro-json-ui/src/render/containers.rs]
tech_stack:
  added: []
  patterns:
    - ActionItem/ActionGroupProps prop structs with full serde+JsonSchema derives
    - partition/overflow/visible_if render pattern in render_action_group
    - ActionItem→DropdownMenuAction conversion at render_menu_item call boundary
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/containers.rs
decisions:
  - ActionItem→DropdownMenuAction conversion at render_menu_item call boundary (avoids duplicating form-wrap logic)
  - "#[allow(dead_code)] on render_action_group and private helpers until plan 02 wires dispatch"
  - Private helper button_variant_classes derives ButtonVariant CSS at default size matching atoms.rs render_button_inner
metrics:
  duration: "~25 min"
  completed: "2026-06-22"
  tasks_completed: 2
  files_modified: 2
---

# Phase 237 Plan 01: ActionItem + ActionGroupProps + render_action_group Summary

`ActionItem` + `ActionGroupProps` structs and `render_action_group` renderer with full partition/overflow/destructive-last/visible_if/form-wrap behavior, 9 tests green, clippy clean.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add ActionItem + ActionGroupProps structs with schema tests | 38c8e177 | ferro-json-ui/src/component.rs |
| 2 | Implement render_action_group with partition + overflow + form-wrap tests | 8582b372 | ferro-json-ui/src/render/containers.rs |
| fmt | Apply rustfmt | 6a633eb2 | ferro-json-ui/src/render/containers.rs |

## What Was Built

### Task 1: ActionItem + ActionGroupProps (component.rs)

Both structs inserted after `ButtonGroupProps` (~line 936):

- `ActionItem`: `label`, `action: Action`, `destructive: bool` (default false), `variant: Option<ButtonVariant>`, `icon: Option<String>`, `visible_if: Option<String>`. Fail-closed row gate semantics documented in doc comments.
- `ActionGroupProps`: `items: Vec<ActionItem>`, `menu_id: String` (required), `max_inline: Option<u8>`, `overflow_label: Option<String>`, `row_key: Option<String>`. All optionals skip-if-none.
- Both derive `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema` (D-07).
- `DropdownMenuAction` kept untouched (D-11).
- Tests: `schema_for_action_item_generates` + `schema_for_action_group_props_generates` pass.

### Task 2: render_action_group (containers.rs)

Implemented `render_action_group` after `render_button_group` with:

**Partition logic (D-01..D-04):**
- Non-destructive items ≤ `max_inline` (default 2) → inline buttons.
- Non-destructive items beyond cap → overflow kebab in input order.
- Destructive items → overflow kebab, appended last (D-01, never count toward max_inline).
- Overflow kebab hidden when `overflow` vec is empty (D-03).

**Inline rendering:**
- GET items: `<a href="...">` link styled with `ButtonVariant` CSS (via `button_variant_classes` private helper).
- Non-GET items: `<form action="..." method="post">` + optional `_method` spoof + `<button type="submit">` (D-15). Mirrors `render_menu_item`'s non-GET branch.

**Overflow kebab:**
- Trigger button with `popovertarget="{menu_id}"`, `aria-label="{overflow_label}"`, three-dot SVG — identical to `render_dropdown_menu` pattern.
- Popover panel `popover id="{menu_id}" data-popover-menu`.
- Each overflow item: `ActionItem` converted to `DropdownMenuAction` and rendered via `atoms::render_menu_item` (single source of truth for menu-item HTML).

**visible_if gate (T-237-01):**
- `action_item_visible` private fn mirrors `action_visible_for_row` in data.rs:445.
- Fail-closed: absent/falsy field hides the item. Applied before partitioning.

**Security:**
- T-237-01 (visible_if fail-closed): pinned by `action_group_visible_if` test.
- T-237-02 (injection): all dynamic text through `html_escape`.
- T-237-03 (method spoof): non-GET uses `<form method="post">` + `_method` spoof, not bare POST button.

**7 behavior tests in containers.rs:**
1. `render_action_group_inline_and_overflow` — 5 items, 2 inline + 3 overflow
2. `action_group_no_overflow_hides_kebab` — 2 items, no kebab emitted
3. `action_group_destructive_ordering` — A, DEL(destructive), B → A+B inline, DEL in kebab last
4. `action_group_non_get_wraps_form` — POST item → `<form method="post">`
5. `action_group_get_renders_link` — GET item → `<a href>`
6. `action_group_data_binding_parity` — deterministic rendering, same input = same output
7. `action_group_visible_if` — present/truthy shows, absent hides (fail-closed)

## Verification

```
cargo fmt --all -- --check          ✅ clean
cargo clippy --all --all-targets -- -D warnings  ✅ clean (full workspace)
cargo test -p ferro-json-ui         ✅ 593 passed, 0 failed
```

## Deviations from Plan

### Auto-applied: #[allow(dead_code)] until dispatch wiring

**Found during:** Task 2  
**Issue:** `render_action_group`, `button_variant_classes`, and `action_item_visible` are not yet dispatched (plan 02's responsibility). Under `-D warnings`, unused `pub(crate)` functions are errors even when called only from `#[cfg(test)]`.  
**Fix:** Added `#[allow(dead_code)]` to all three new functions with a comment noting they become call sites in plan 02. This matches the environment note in the plan: "ActionGroup isn't dispatched yet... a commit boundary would break compilation."  
**No behavioral impact.**

### Style: fmt applied as a separate commit

`cargo fmt` reformatted test assertions and inline expressions into multi-line form (long assert! calls). Applied as a follow-up commit after tests passed.

## SC Coverage

- SC-1: partition/overflow/destructive-last/kebab-hidden — 3 tests green.
- SC-2: $data-binding-parity + visible_if fail-closed — 2 tests green.
- SC-3: non-GET form-wrap + GET link — 2 tests green.
- Component NOT yet registered in dispatch (plan 02).

## Known Stubs

None — this plan adds props + renderer only, not dispatch wiring. Plan 02 wires `ActionGroup` into `BUILTIN_TYPES`, dispatch, and catalog.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced. T-237-01/02/03 mitigations implemented and pinned by tests.

## Self-Check: PASSED

- `ferro-json-ui/src/component.rs` — ActionItem and ActionGroupProps present: ✅
- `ferro-json-ui/src/render/containers.rs` — render_action_group present: ✅
- Commits 38c8e177, 8582b372, 6a633eb2 exist: ✅
- 593 tests pass: ✅
- clippy clean: ✅
- fmt clean: ✅
