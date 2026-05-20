---
phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
plan: "03"
subsystem: ui
tags: [json-ui, data-table, interpolation, row-actions, template]

requires:
  - phase: 175-01
    provides: MAX_NESTING_DEPTH raised to 16 (unblocks deep specs)
  - phase: 175-02
    provides: tab rendering fix (context for the same patch batch)

provides:
  - "{row.X} placeholder resolution in DataTable template_url and template_actions"
  - "Three regression-pinning tests for the alias, back-compat, and missing-key invariant"

affects: [175-04, 175-05, 175-06, json-ui, data-table consumers]

tech-stack:
  added: []
  patterns:
    - "Dual-form interpolation: substitute {col_key} AND {row.col_key} in a single loop pass — additive alias, not a rewrite"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/render/data.rs

key-decisions:
  - "D-F6 honored: additive alias only — {X} unchanged, {row.X} added as a second replace() call in the same loop body"
  - "Missing keys leave both forms literal — the loop only fires when the key exists in the row object"
  - "HTML escaping not duplicated — substituted values flow to html_escape downstream in render_inline_dropdown and row-action emission, same as before"

patterns-established:
  - "Dual-form URL placeholder pattern: when adding a new placeholder alias, add a second url.replace() call in the same loop rather than a separate pass"

requirements-completed: []

duration: 7min
completed: 2026-05-20
---

# Phase 175 Plan 03: DataTable {row.X} Placeholder Interpolation (F6) Summary

**Additive {row.X} alias in DataTable URL templating — per-row delete form actions now resolve from row data instead of rendering as literal curly braces**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-05-20T18:39:29Z
- **Completed:** 2026-05-20T18:46:35Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- `template_url` and `template_actions` both substitute `{row.X}` as an alias for `{X}` in a single loop pass
- Bare `{X}` form continues to resolve identically — no behavior change for existing consumers
- Missing keys leave both `{X}` and `{row.X}` literal (existing invariant preserved)
- Three new tests pin: resolved alias, back-compat, and missing-key invariant

## Task Commits

1. **Task 1: Add failing tests for {row.X} interpolation + back-compat assertion** - `858d4032` (test)
2. **Task 2: Extend template_actions and template_url to substitute {row.X} as an alias** - `b3037190` (feat)

## Files Created/Modified

- `ferro-json-ui/src/render/data.rs` — two identical `url.replace(&format!("{{row.{col_key}}}"), &val_str)` lines added (one in `template_url`, one in `template_actions`); three new tests appended to the tests module

## Decisions Made

- D-F6 applied as specified: additive alias, single-pass extension of the existing loop body. No second loop, no architectural change.
- HTML escaping not re-applied inside the substitution — values flow to `html_escape` at the point of emission in `render_inline_dropdown` and the `<a href>` / `<form action>` emitters, consistent with the existing column-key substitution path.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None. The RED state for Task 1 was confirmed: test 1 (`data_table_row_prefix_placeholder_resolved`) failed because the form action contained the literal string `{row.delete_url}`. After Task 2, all three new tests and all 18 existing data_table tests passed. The missing-key test (`data_table_row_prefix_missing_key_leaves_placeholder`) happened to pass in the RED phase as well — this is correct because without a substitution path, `{row.nonexistent}` is left literal by default, which satisfies the invariant even before the fix lands.

## Threat Model Verification

- **T-175-03-01 (I/S):** Verified — the substituted value in `template_actions` flows to `render_inline_dropdown`, which passes the final URL through `html_escape` before writing it into `<form action="...">` or `<a href="...">`. The `template_url` result is also passed through `html_escape` at the call sites in `render_data_table`. The F6 change does not bypass any escape. (Confirmed by reading `render_inline_dropdown` and its callers in `render_data_table`.)
- **T-175-03-02 (T):** Accepted — path-traversal interpretation is the routing layer's responsibility; DataTable is a faithful presentation layer.
- **T-175-03-03 (T):** Accepted — documented in RESEARCH.md A2; no conflict.

## Known Stubs

None — the fix is complete. The consumer's Assenze tab per-row delete form action will now resolve to the row's `delete_url` field value.

## Next Phase Readiness

- F6 is closed. The consumer's DataTable Approach A row-action pattern (`action: "{row.delete_url}"`) works across all DataTable consumers.
- Ready to continue with 175-04 (F2 — CheckboxGroup), 175-05 (F4 — Switch), 175-06 (F5 — file input + enctype).

---
*Phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test*
*Completed: 2026-05-20*
