---
phase: 256-component-renderers-builtin-lockstep
plan: "04"
subsystem: ferro-json-ui / runtime
tags: [json-ui, pos, runtime, selection-panel, qty-stepper, es5, lockstep]
dependency_graph:
  requires: [256-03-SUMMARY.md]
  provides: [setupSelection reconciler, initQtyButton bounds, FERRO_RUNTIME_JS with 16 setup functions]
  affects: [ferro-json-ui/src/runtime/selection.rs, ferro-json-ui/src/runtime/tiles.rs, ferro-json-ui/src/runtime/mod.rs]
tech_stack:
  added: []
  patterns:
    - "Input-event-driven reconciler: one delegated form.addEventListener('input') code path drives the entire panel view"
    - "Template clone: tmpl.content.cloneNode(true) fills name/qty/line-total; delegated-click attrs wired onto clone before appendChild"
    - "Tile-DOM metadata read: input.closest('[data-filter-text]') → getAttribute('data-filter-text') + getAttribute('data-unit-price')"
    - "Line lookup key: data-selection-line-field={field} set by runtime on each cloned line"
    - "Integer-cents display: (n/100).toFixed(2) is presentational only; hidden inputs carry raw integer values"
    - "Bounded qty clamp: Math.min(Math.max(current + delta*step, min), max) with Infinity as no-max sentinel"
key_files:
  created:
    - ferro-json-ui/src/runtime/selection.rs
  modified:
    - ferro-json-ui/src/runtime/mod.rs
    - ferro-json-ui/src/runtime/tiles.rs
decisions:
  - "Line lookup uses data-selection-line-field attribute set by the runtime on each cloned line (not data-qty-input value)"
  - "Re-query linesEl after appendChild(clone) because the DocumentFragment is emptied when moved into the DOM"
  - "Delegated click attr wired onto cloned buttons before append so the panel click handler can read them without per-element binding"
  - "formatMoney is a top-level function in the IIFE scope (hoisted); reconcile is a named inner function (closure over panel state)"
  - "initQtyButton uses Infinity as the no-max sentinel (null rawMax → Infinity); parseInt(null) is NaN which falls through to Infinity"
metrics:
  duration: "~17 min"
  completed: "2026-07-06T02:06:00Z"
  tasks_completed: 3
  files_modified: 3
---

# Phase 256 Plan 04: SelectionPanel Reconciler + initQtyButton Bounds Summary

**ES5 input-event-driven SelectionPanel reconciler with template-clone line management, tile-DOM metadata reads, integer-cents running total, and initQtyButton QuantityStepper bounds (min/max/step)**

## Performance

- **Duration:** ~17 min
- **Completed:** 2026-07-06T02:06:00Z
- **Tasks:** 3
- **Files modified:** 3 (1 created, 2 modified)

## Accomplishments

- `runtime/selection.rs` created: `setupSelection()` — no-op when no `[data-selection-panel]` exists; `initSelectionPanel()` wires form-scoped delegated input listener + panel-scoped delegated click handler for inc/dec/remove; `reconcile()` inner function iterates all `[data-qty-input]` in form scope, clones template lines for qty > 0, removes lines for qty = 0, updates qty/line-total display, recomputes integer-cents running total, toggles EmptyState via `style.display`; `formatMoney()` presentational cents formatter
- `runtime/mod.rs` wired in one D-06 commit: `mod selection;` + `selection::SOURCE` push (placed after `filters::SOURCE`, before `kanban::SOURCE`) + `setupSelection` in dispatcher `setups` array + `"setupSelection"` added to both `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` drift-list tests
- `runtime/tiles.rs` extended: `initQtyButton` now reads `data-qty-step` (default 1), `data-qty-min` (default 0), `data-qty-max` (Infinity when absent) from the button element and applies `Math.min(Math.max(current + delta * step, min), max)` — tap-to-add tiles with no bounds attrs are unaffected

## Reconcile Algorithm

`reconcile()` is a named function declaration inside `initSelectionPanel` (closure over `form`, `tmpl`, `linesEl`, `emptyEl`, `totalEl`, `currency`):

```
for each input = form.querySelectorAll('[data-qty-input]'):
  field = sanitized getAttribute('data-qty-input')
  qty   = parseInt(input.value)
  if qty > 0:
    tile  = input.closest('[data-filter-text]')
    name  = tile.getAttribute('data-filter-text') || field
    unit  = parseInt(tile.getAttribute('data-unit-price')) || 0
    lineCents = unit * qty
    totalCents += lineCents
    lineEl = linesEl.querySelector('[data-selection-line-field="' + field + '"]')
    if !lineEl:
      clone = tmpl.content.cloneNode(true)
      newLine = clone.querySelector('[data-selection-line]')
      newLine.setAttribute('data-selection-line-field', field)
      wire inc/dec/remove attrs onto cloned buttons
      set nameEl.textContent = name  ← textContent only (T-256-13)
      linesEl.appendChild(clone)
      lineEl = linesEl.querySelector('[data-selection-line-field="' + field + '"]')
    lineEl.querySelector('[data-selection-line-qty]').textContent = qty
    lineEl.querySelector('[data-selection-line-total]').textContent = formatMoney(lineCents, currency)
  else:
    lineEl = linesEl.querySelector('[data-selection-line-field="' + field + '"]')
    if lineEl: lineEl.parentNode.removeChild(lineEl)
totalEl.textContent = formatMoney(totalCents, currency)
emptyEl.style.display = hasLines ? 'none' : ''
linesEl.style.display = hasLines ? '' : 'none'
```

**Line lookup key:** `data-selection-line-field="{field}"` — set by the runtime on each cloned `[data-selection-line]` wrapper before appending. The cloned line is re-queried after `linesEl.appendChild(clone)` because the DocumentFragment is emptied when moved into the DOM.

## Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2 | setupSelection + mod.rs wiring (D-06 commit) | 77c8e50c | ferro-json-ui/src/runtime/selection.rs, ferro-json-ui/src/runtime/mod.rs |
| 3 | initQtyButton min/max/step bounds | e05fbfcf | ferro-json-ui/src/runtime/tiles.rs |

## Semantic-Token Class Literals in selection.rs

No raw CSS class strings are introduced in `selection.rs`. The reconciler writes only `textContent` and toggles `style.display` — it does not construct or inject CSS classes. The `variant_classes_use_semantic_tokens` test passes unchanged (still satisfied by existing modules).

## Runtime Bundle State After Plan 04

16 setup functions in FERRO_RUNTIME_JS:
`setupScrollPreserve`, `setupSSE`, `setupTabs`, `setupDismissibles`, `setupNotifications`, `setupDropdowns`, `setupKanban`, `setupSidebar`, `setupFormGuards`, `setupTiles`, `setupNumpad`, `setupFilters`, **`setupSelection`** (new), `setupModals`, `setupToasts`, `setupLazyHeroes`

Both drift-list tests confirm this. `bundle_is_single_iife`, `variant_classes_use_semantic_tokens`, and all 17 runtime tests pass.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All three tasks are fully implemented. `setupSelection` is a complete, no-stub reconciler. `initQtyButton` bounds are read from real attributes with correct defaults.

## Threat Flags

No new threat surface beyond what the plan's threat model covers. All four T-256-12..T-256-15 mitigations applied:
- T-256-12: field sanitized with `field.replace(/["\\\]]/g, '')` in every querySelector construction (4 sites)
- T-256-13: all DOM writes use `textContent` (name, qty, total, running total)
- T-256-14: running total is display-only; confirmed in the selection.rs header comment
- T-256-15: integer-cents arithmetic throughout; the single float is `(n/100).toFixed(2)` in `formatMoney` (presentational only)

## Self-Check: PASSED

- `ferro-json-ui/src/runtime/selection.rs` — FOUND
- `ferro-json-ui/src/runtime/mod.rs` — FOUND (contains `mod selection;`, `selection::SOURCE`, `setupSelection` ×3)
- `ferro-json-ui/src/runtime/tiles.rs` — FOUND (contains `Math.min(Math.max`, `data-qty-step`, `data-qty-min`, `data-qty-max`)
- Commit 77c8e50c — FOUND in git log
- Commit e05fbfcf — FOUND in git log
- All 17 runtime tests pass (verified by `cargo test -p ferro-json-ui --all-features runtime`)
