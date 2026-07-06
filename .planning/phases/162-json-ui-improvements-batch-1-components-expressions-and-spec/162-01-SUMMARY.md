---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
plan: 01
subsystem: ui
tags: [ferro-json-ui, checkbox, multi-select, component, catalog, render]

# Dependency graph
requires: []
provides:
  - CheckboxListProps struct in component.rs (field, options, options_path, selected_path, label, description, disabled, error)
  - render_checkbox_list function in render/form.rs with XSS-safe HTML emission
  - CheckboxList entry in BUILTIN_TYPES, dispatch arm, BUILTIN_SPECS catalog, lib.rs re-export
affects:
  - 162-03 (Wave 1: also modifies component.rs, render/form.rs, catalog.rs)
  - 162-04 (Wave 1: RichTextEditor plugin registration)
  - 162-05 (Wave 2: ferro-mcp catalog count bump to 40 + RichTextEditor)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "CheckboxList uses options_path fallback: static options.is_empty() -> resolve options_path -> unwrap_or_default"
    - "Selected options resolved via selected_path as Vec<String> against data"
    - "All string interpolations in render_checkbox_list pass through html_escape (XSS defense-in-depth)"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/form.rs
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/catalog.rs
    - ferro-json-ui/src/lib.rs

key-decisions:
  - "BUILTIN_TYPES count 39->40; catalog.rs count assertions bumped in same commit as dispatch arm"
  - "prompt() budget bumped 8KB->9KB to accommodate CheckboxList entry (actual: 8348 bytes)"
  - "ferro-mcp test_all_components_present left at 39 per scope constraint — Plan 162-05 owns that bump"
  - "TDD pattern: RED (compile-fail tests) before struct/function implementation, GREEN in same task"

patterns-established:
  - "CheckboxList follows SelectProps naming (options not items, selected_path not default_value_path)"
  - "Multi-select group wraps in <fieldset>/<legend> for accessibility, matching a11y pattern from Checkbox"

requirements-completed: []

# Metrics
duration: 10min
completed: 2026-05-16
---

# Phase 162 Plan 01: CheckboxList Component Summary

**CheckboxListProps struct + render_checkbox_list with data-driven options and XSS-safe HTML, registered in BUILTIN_TYPES=40 and catalog**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-16T14:20:36Z
- **Completed:** 2026-05-16T14:30:31Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added `CheckboxListProps` (8 fields: field, options, options_path, selected_path, label, description, disabled, error) to component.rs with schema smoke test and serde roundtrip test
- Implemented `render_checkbox_list` in render/form.rs: resolves options from `options_path` when static list is empty, resolves pre-selected values from `selected_path`, emits one `<input type="checkbox">` per option with `checked`/`disabled` attributes, all strings HTML-escaped
- Wired CheckboxList into BUILTIN_TYPES (count 40), dispatch arm, BUILTIN_SPECS catalog, and lib.rs re-export; bumped all three count assertions atomically

## Task Commits

1. **Task 1: CheckboxListProps struct + schema/serde tests** - `3edd61f5` (feat)
2. **Task 2: render_checkbox_list + 5 behavior tests** - `29eb752a` (feat)
3. **Task 3: BUILTIN_TYPES + dispatch + catalog + lib re-export** - `d5b008c9` (feat)

## Files Created/Modified

- `ferro-json-ui/src/component.rs` - Added CheckboxListProps struct after CheckboxProps; added 2 tests to schema_smoke_tests module
- `ferro-json-ui/src/render/form.rs` - Added render_checkbox_list function + 5 unit tests; added CheckboxListProps/SelectOption imports
- `ferro-json-ui/src/render/mod.rs` - Added "CheckboxList" to BUILTIN_TYPES, dispatch arm, bumped count assertion 39->40
- `ferro-json-ui/src/catalog.rs` - Added CheckboxListProps to import, CheckboxList to BUILTIN_SPECS, bumped count assertions 39->40, bumped prompt budget 8KB->9KB
- `ferro-json-ui/src/lib.rs` - Re-exported CheckboxListProps in component re-export block

## Decisions Made

- Prompt budget bumped from 8 KB to 9 KB — the new component added ~156 bytes to the catalog prompt, pushing it to 8348 bytes. The original 8 KB was calibrated at 39 components; 9 KB gives headroom for the remaining Phase 162 additions.
- `ferro-mcp/src/tools/json_ui_catalog.rs` `test_all_components_present` was intentionally left at count 39 per plan scope constraint. Plan 162-05 will bump it to 40 atomically with RichTextEditor registration.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Three catalog count assertions in catalog.rs not flagged in plan**
- **Found during:** Task 3 (register in catalog)
- **Issue:** catalog.rs has its own `builtin_types_count_is_39` and `builtin_specs_len_matches_dispatch` tests asserting 39, plus `prompt_under_size_budget` at 8 KB — none of these were mentioned in the plan's action for Task 3
- **Fix:** Bumped both count assertions to 40; bumped prompt budget to 9 KB (actual size: 8348 bytes)
- **Files modified:** ferro-json-ui/src/catalog.rs
- **Verification:** `cargo test -p ferro-json-ui --all-features` exits 0
- **Committed in:** d5b008c9 (Task 3 commit)

**2. [Rule 1 - Bug] Temporary #[allow(dead_code)] on render_checkbox_list between Task 2 and Task 3**
- **Found during:** Task 2 post-commit lint check
- **Issue:** `render_checkbox_list` is `pub(crate)` but has no callers until Task 3 wires the dispatch arm; clippy -D warnings fires dead_code
- **Fix:** Added `#[allow(dead_code)]` in Task 2 commit; removed in Task 3 commit when dispatch arm was wired
- **Files modified:** ferro-json-ui/src/render/form.rs
- **Verification:** clippy passes after each task commit
- **Committed in:** 29eb752a (Task 2), removed in d5b008c9 (Task 3)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - Bug)
**Impact on plan:** Both fixes necessary for test correctness and clippy compliance. No scope creep.

## Issues Encountered

- `ferro-mcp::test_all_components_present` fails on the full workspace `cargo test --all-features` run (count 39 vs actual 40). This is expected — plan scope constraint explicitly prohibits touching `json_ui_catalog.rs`. Plan 162-05 owns this fix. The `cargo test -p ferro-json-ui --all-features` suite (the per-plan target) passes cleanly.

## Known Stubs

None — CheckboxList is fully wired: props struct, renderer, BUILTIN_TYPES, dispatch arm, catalog entry, lib re-export.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The XSS surface (T-162-01-01) was mitigated by `html_escape` on all string interpolations; the injection surface (T-162-01-02) was mitigated by `serde_json::from_value::<SelectOption>` deserialization. Both verified by unit tests (`checkbox_list_escapes_html_in_option_label`).

## Next Phase Readiness

- CheckboxList is fully usable by gestiscilo onboarding step 2 (services list) via `ferro = { path = "../ferro" }` local patch
- Wave 1 continues with Plans 162-03 (SwitchProps.compact + ImageProps.inline_svg) and 162-04 (RichTextEditorProps + plugin), both extending the same files — additions are cleanly scoped to CheckboxList only in this plan
- Plan 162-05 (Wave 2) must bump `ferro-mcp/src/tools/json_ui_catalog.rs` count to 40 (CheckboxList) + 1 (RichTextEditor plugin) = catalog reports 40 built-ins + 2 plugin components

---
*Phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec*
*Completed: 2026-05-16*
