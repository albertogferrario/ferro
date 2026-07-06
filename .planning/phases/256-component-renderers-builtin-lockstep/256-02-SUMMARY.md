---
phase: 256-component-renderers-builtin-lockstep
plan: "02"
subsystem: ui
tags: [ferro-json-ui, ferro-mcp, render, builtin, tilegrid, filtertabs, pos, touch, filter, lockstep]

requires:
  - phase: 256-01
    provides: "TileProps.price_cents/color, render_tile tap-to-add, Grid row_weights, touch foundation constants"
  - phase: 255-pos-runtime-modules-double-submit-protection
    provides: "setupFilters, updateFilterTabClasses, data-filter-scope/tab/search/tokens/text attribute contract"

provides:
  - "render_tile_grid in containers.rs: data-filter-scope root, 16px search input, categories_path→strip via shared helper, full-literal col-class ladder, children via render_element"
  - "render_filter_tab_strip pub(crate) shared helper in atoms.rs: single markup source for TileGrid integrated strip AND FilterTabs standalone"
  - "render_filter_tabs in atoms.rs: data-filter-scope wrapper, all_label neutral English 'All' default, nearest-ancestor scope semantics"
  - "TileGrid registered as builtin count 48 (BUILTIN_TYPES, dispatch, BUILTIN_SPECS, catalog count guard, ferro-mcp mirror)"
  - "FilterTabs registered as builtin count 49 (same lockstep)"
  - "RULE_COMPONENTS extended: register-fill-viewport + register-grid-fill now map ['Grid','TileGrid']"

affects:
  - "256-03 through 256-05 (downstream renderers targeting same crates)"
  - "Phase 257 projection-builder: TileGrid + FilterTabs now pass catalog_validate"
  - "Phase 258 MCP catalog docs: count 49 is the lockstep baseline"

tech-stack:
  added: []
  patterns:
    - "Shared render_filter_tab_strip helper: single HTML source for both TileGrid integrated strip and standalone FilterTabs"
    - "categories_path resolve_path idiom: props.categories_path.as_deref().and_then(|p| resolve_path(data, p)).and_then(|v| v.as_array()).map(...).unwrap_or_default()"
    - "Full-literal column class ladder: exhaustive match on Option<u8> -> 'grid-cols-1/2/3/4'; default None = 'grid-cols-2'"
    - "Tab strip inactive classes: 'border-transparent text-text-muted hover:text-text' — D-12 lockstep with updateFilterTabClasses"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/catalog.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs

key-decisions:
  - "render_filter_tab_strip placed in atoms.rs (D-17): containers.rs calls it via super::atoms::render_filter_tab_strip — same cross-module call pattern already used by render_menu_item"
  - "FilterTabs added to ferro-mcp no_required list: all props optional (items defaults to empty = All-only strip; all_label defaults to 'All'); valid zero-prop usage"
  - "categories_path None = no strip element; Some + unresolved/empty = All tab only (graceful SC-5 path)"

metrics:
  duration: ~25min
  completed: "2026-07-06T00:42:40Z"
  tasks: 2
  files: 5
---

# Phase 256 Plan 02: TileGrid + FilterTabs Builtin Registration Summary

**TileGrid (count 48) and FilterTabs (count 49) registered as first-class catalog members; shared render_filter_tab_strip helper is the single markup source for both; SC-5 categories_path resolution tested end-to-end; both catalog and MCP mirror drift guards updated in lockstep.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-07-06T00:20:00Z
- **Completed:** 2026-07-06T00:42:40Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `render_filter_tab_strip` shared helper landed in `atoms.rs`: emits a `<div role="tablist">` with an All tab (active-initial: `border-primary text-primary font-semibold`) and one button per item (inactive-initial: `border-transparent text-text-muted hover:text-text`). This is the D-12 lockstep with `updateFilterTabClasses` in `runtime/filters.rs`. Space→hyphen token normalization mirrors `render_tile`.

- `render_tile_grid` in `containers.rs`:
  - Root div carries `data-filter-scope` always
  - Search input (`data-filter-search`, `text-base` 16px, iOS zoom-safe) when `search: true`
  - Integrated strip: `categories_path` resolved via `resolve_path(data, p)` → array → `render_filter_tab_strip`; None → no strip; unresolved/empty → All tab only (graceful)
  - Column class: exhaustive full-literal match (`grid-cols-1/2/3/4`, default `grid-cols-2`)
  - Children iterated via `render_element` (standard pipeline)

- `render_filter_tabs` in `atoms.rs`: wraps the shared strip in a `data-filter-scope` div; `all_label` defaults to `"All"` (neutral English, D-28)

- Lockstep registration (both components, one bump per component, two commits):
  - Task 1: `BUILTIN_TYPES` + dispatch + `BUILTIN_SPECS` + count guard `47→48` with History comment + ferro-mcp mirror `47→48` + `"TileGrid"` in expected + `register-fill-viewport`/`register-grid-fill` extended to `["Grid", "TileGrid"]`
  - Task 2: same lockstep for FilterTabs, count `48→49`

- 9 new tests total (4 for TileGrid in containers.rs, 5 for FilterTabs in atoms.rs); all passing alongside the 770+311 existing tests

## Render_filter_tab_strip Signature

```rust
// ferro-json-ui/src/render/atoms.rs:1471
pub(crate) fn render_filter_tab_strip(items: &[String], all_label: &str) -> String
```

Called from:
- `render_tile_grid` (containers.rs): `super::atoms::render_filter_tab_strip(&categories, "All")`
- `render_filter_tabs` (atoms.rs): `render_filter_tab_strip(&props.items, all_label)`

## categories_path resolve_path Idiom

```rust
let categories: Vec<String> = props
    .categories_path
    .as_deref()
    .and_then(|p| resolve_path(data, p))
    .and_then(|v| v.as_array())
    .map(|arr| arr.iter().filter_map(|x| x.as_str().map(String::from)).collect())
    .unwrap_or_default();
```

Strip is only emitted when `categories_path.is_some()`. Empty resolved array → All tab only (graceful).

## Column Class Ladder (render_tile_grid)

```rust
let col_class = match props.columns {
    Some(1) => "grid-cols-1",
    Some(2) | None => "grid-cols-2",
    Some(3) => "grid-cols-3",
    Some(4) => "grid-cols-4",
    _ => "grid-cols-2",
};
```

Full-literal only — `format!("grid-cols-{n}")` never used (SC-3, T-256-06).

## Task Commits

1. **Task 1 — TileGrid (count 48):** `912eacf0`
2. **Task 2 — FilterTabs (count 49):** `3af7dfee`

## Current Canonical Count

**49** — Plan 03 should start the lockstep from 49 when registering the next component.

## Prompt Budget

`prompt_under_size_budget` (catalog.rs:2144) passed at the existing 12 KB cap — no bump needed. Both components' props schemas are compact.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing `item_id` in SC-5 test Tile construction**
- **Found during:** Task 2 test run (`filter_tabs_tokens_match_tile_tokens`)
- **Issue:** `TileProps` requires `item_id` as a mandatory field; the test constructed `Element::new("Tile")` without it → serde decode failure
- **Fix:** Added `.prop("item_id", "p1")` to the Tile element builder in the test
- **Files modified:** `ferro-json-ui/src/render/atoms.rs`
- **Commit:** `3af7dfee`

**2. [Rule 2 - Missing critical functionality] FilterTabs needs no_required exemption in ferro-mcp**
- **Found during:** Task 2 CI gate (`test_components_have_props` failure)
- **Issue:** `FilterTabsProps` has no required fields (all optional); the ferro-mcp test asserts every component has ≥1 required prop unless explicitly exempted
- **Fix:** Added `"FilterTabs"` to the `no_required` exclusion list with a comment (items defaults to empty = All-only strip; all_label defaults to "All")
- **Files modified:** `ferro-mcp/src/tools/json_ui_catalog.rs`
- **Commit:** `3af7dfee`

**3. [Rule 1 - Bug] FilterTabs code placed in Task 1 prematurely causing unused-code clippy errors**
- **Found during:** Task 1 CI gate
- **Issue:** `render_filter_tabs` + `FilterTabsProps` import added to atoms.rs/catalog.rs before the dispatch arm existed → `dead_code` + `unused_imports` warnings under `-D warnings`
- **Fix:** Moved `render_filter_tabs` entirely to Task 2 (correct placement per plan)
- **Files modified:** `ferro-json-ui/src/render/atoms.rs`, `ferro-json-ui/src/catalog.rs`

---

**Total deviations:** 3 auto-fixed (all Rule 1/2 — caught at CI gate before commit)
**Impact on plan:** No scope change.

## Known Stubs

None — both components are fully wired: `render_tile_grid` iterates real children via `render_element`, resolves `categories_path` against bound data for the integrated strip; `render_filter_tabs` renders the shared strip with the correct attribute contract. The register screen composition (Phase 257 projector output) targets these registered builtins.

## Threat Flags

All T-256-05 mitigations applied:
- `html_escape()` on `all_label`, item labels, tokens, and resolved category strings in `render_filter_tab_strip`
- T-256-06 (column class injection) closed: exhaustive full-literal match, no `format!("grid-cols-{n}")`
- T-256-07 (catalog drift): both count guards bumped in the same commit per component with History comment audit trail; `component_rule_mapping_is_exhaustive` keeps RULE_COMPONENTS names valid

No new threat surface beyond what the plan's threat model covers.

## Self-Check

Files exist:
- `ferro-json-ui/src/render/atoms.rs` — render_filter_tab_strip + render_filter_tabs present
- `ferro-json-ui/src/render/containers.rs` — render_tile_grid present
- `ferro-json-ui/src/render/mod.rs` — TileGrid + FilterTabs in BUILTIN_TYPES and dispatch
- `ferro-json-ui/src/catalog.rs` — count guard at 49
- `ferro-mcp/src/tools/json_ui_catalog.rs` — mirror count 49, both names in expected

Commits verified: `912eacf0` (TileGrid), `3af7dfee` (FilterTabs) — both in `git log --oneline -4`.

## Self-Check: PASSED
