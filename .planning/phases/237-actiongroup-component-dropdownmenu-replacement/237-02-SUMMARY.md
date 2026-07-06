# Plan 237-02 Summary — Atomic ActionGroup↔DropdownMenu registration swap

**Status:** Complete
**Requirements:** SC-4
**Completed:** 2026-06-22

## What was built

The atomic lockstep registration swap that makes `ActionGroup` a live public component and
removes `DropdownMenu` from the public surface — applied across all registration sites in one
commit so the runtime drift guard never observes a divergent `BUILTIN_TYPES`/`BUILTIN_SPECS` state.

- `ferro-json-ui/src/render/mod.rs` — `BUILTIN_TYPES` swaps `"DropdownMenu"` → `"ActionGroup"`; the
  dispatch `match` arm now routes `"ActionGroup" => containers::render_action_group(...)` and the
  DropdownMenu arm is removed.
- `ferro-json-ui/src/catalog.rs` — `BUILTIN_SPECS` swaps the DropdownMenu tuple for an `ActionGroup`
  tuple (slot_fields `&[]`); `builtin_types_count_drift_guard` history comment updated, count stays **47**.
- `ferro-json-ui/src/lib.rs` — public export swaps `DropdownMenuProps` out, `ActionGroupProps`/`ActionItem` in.
- `ferro-mcp/src/tools/json_ui_catalog.rs` — `test_all_components_present` `expected[]` fixed to the
  full **47-name** list: added the pre-existing-missing `"SegmentedControl"` + `"SidebarLayout"` (0.2.69
  regression) AND swapped `"DropdownMenu"` → `"ActionGroup"`. Count mirror stays 47.

`DropdownMenuAction` (DataTable/Kanban row-action carrier) and `render_menu_item` were retained.

## Deviation note

The interrupted Wave 2 executor folded part of Plan 237-03's code into this plan's commits
(`projection/builder.rs` migration to ActionGroup + deletion of the now-dead `render_dropdown_menu`).
This is consistent with the dependency order (the dead-code removal can only follow this plan's
dispatch-arm removal) and was verified correct. The corresponding 237-03 code acceptance criteria
are satisfied; 237-03's remaining work was the docs migration.

## Commits

- `b37b7b0e` feat(237-02): atomic ActionGroup↔DropdownMenu registration swap
- `5807f370` feat(237-02): fix ferro-mcp expected[] mirror + migrate projection builder
- `715bf225` style(237-02): apply rustfmt to catalog.rs and atoms.rs import blocks

## Verification

- `cargo test -p ferro-json-ui` — **620 passed**, incl. `builtin_types_count_drift_guard` (count 47).
- `cargo test -p ferro-mcp` — **303 passed**, incl. `test_all_components_present` (47 names).
- `cargo clippy -p ferro-json-ui -p ferro-mcp --all-targets -- -D warnings` — clean.
- `cargo check --workspace` — entire workspace type-checks (no external `DropdownMenu` references; blast radius contained to ferro-json-ui + ferro-mcp).
- `cargo fmt --all -- --check` — clean.

## Self-Check: PASSED

Disk note: this plan's verification was run scoped to the affected crates + a workspace `cargo check`
(rather than full `cargo test --all-features`) due to a disk-space constraint during execution; the
full `--all-features` gate is run in Plan 237-04. Coverage is complete because grep confirmed zero
`DropdownMenu` references anywhere outside ferro-json-ui.
