---
phase: 256
plan: "03"
subsystem: ferro-json-ui / ferro-mcp
tags: [json-ui, pos, builtins, lockstep, catalog]
dependency_graph:
  requires: [256-02-SUMMARY.md]
  provides: [QuantityStepper count-50, Numpad count-51, SelectionPanel count-52]
  affects: [ferro-json-ui/src/render/atoms.rs, ferro-json-ui/src/render/containers.rs, ferro-json-ui/src/catalog.rs, ferro-mcp/src/tools/json_ui_catalog.rs]
tech_stack:
  added: []
  patterns: [BUILTIN_TYPES+BUILTIN_SPECS lockstep, HIT_TARGET_{MIN,NUMPAD} constants, data-* attribute contracts]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/catalog.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs
decisions:
  - "SelectionPanel data-selection-total is display-only; server re-validates total from hidden inputs on POST (T-256-10)"
  - "Prompt size budget bumped 12 KB → 13 KB after 3 new POS specs pushed catalog over cap"
  - "Test assertions use HIT_TARGET_{MIN,NUMPAD} constants rather than raw literals to satisfy render_functions_use_constants_not_literals drift-guard"
metrics:
  duration: "~30 min (resumed from previous context)"
  completed: "2026-07-06T01:10:49Z"
  tasks_completed: 3
  files_modified: 5
---

# Phase 256 Plan 03: POS Builtins — QuantityStepper, Numpad, SelectionPanel Summary

Three final POS builtins registered atomically (one commit per component), bringing
the builtin count from 49 to 52 (final). Both catalog count guards (ferro-json-ui and
ferro-mcp mirror) bumped lockstep with each commit.

## Tasks Completed

| # | Task | Commit | Count |
|---|------|--------|-------|
| 1 | Register QuantityStepper | fc802fc6 | 50 |
| 2 | Register Numpad | 3616b2e3 | 51 |
| 3 | Register SelectionPanel | e4914e01 | 52 |

## SelectionPanel Attribute Contract (D-06..D-15)

Full `data-*` surface emitted by `render_selection_panel`:

**Root element**
```
data-selection-panel
data-selection-form="{form_id}"
class="flex flex-col h-full min-h-0 overscroll-contain"
```

**Line template (inside `<template data-selection-line-template>`)**
```
data-selection-line                   — line wrapper
data-selection-line-name              — item name text node
data-selection-dec                    — decrement button
data-selection-line-qty               — quantity display span
data-selection-inc                    — increment button
data-selection-remove                 — remove button (×)
data-selection-line-total             — per-line subtotal text node
```

**Lines container**
```
data-selection-lines
class="flex-1 overflow-y-auto min-h-0"
```

**Empty state**
```
data-selection-empty
```
Shown when no items are selected. Message from `empty_message` prop (default: "No items
selected"). Hidden when at least one line is present (JS runtime responsibility).

**Total row**
```
data-selection-total                  — always present
data-selection-currency="{currency}"  — present only when currency prop is set
```
Initial rendered value: `0.00`. This is a **display-only** field — the server must
re-validate the total from the hidden qty inputs on POST (T-256-10).

**Confirm slot**
Children from `el.children` are rendered after the total row via `render_element`.
Intended for a submit/confirm button wired to the `data-selection-form` form ID.

**Props struct**
```rust
pub struct SelectionPanelProps {
    pub form_id: String,
    pub empty_message: Option<String>,  // default: "No items selected"
    pub currency: Option<String>,       // e.g. "EUR" — drives data-selection-currency
}
```

## QuantityStepper Attribute Contract

```
data-qty-dec="{field}"      — decrement button
data-qty-display="{field}"  — current quantity display span
data-qty-inc="{field}"      — increment button
data-qty-input="{field}"    — hidden input (name="{field}", value="0")
data-qty-min="{n}"          — optional lower bound
data-qty-max="{n}"          — optional upper bound
data-qty-step="{n}"         — optional step (default 1)
```

Hit targets: dec and inc buttons both carry `HIT_TARGET_MIN` (≥44px, WCAG 2.5.5).

## Numpad Attribute Contract

```
data-numpad                           — root container
data-numpad-target="{target_field}"   — field name this numpad drives
data-numpad-mode="price"              — present only when mode=Price
data-numpad-display                   — display span (shows current value)
data-numpad-key="{digit|backspace|clear|00}"  — 12 key buttons
data-numpad-input                     — hidden input carrying the committed value
```

Hit targets: all 12 keys carry `HIT_TARGET_NUMPAD` (≥56px).
Glyph mapping: "backspace" → U+232B (⌫), "clear" → "C".

## RULE_COMPONENTS Final State

`register-selection-present` in ferro-mcp:
```rust
&["Grid", "Numpad", "SelectionPanel"]
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `render_functions_use_constants_not_literals` drift-guard failure**
- **Found during:** Task 3 CI gate
- **Issue:** Test assertions in `quantity_stepper_targets_44px` and `numpad_keys_56px`
  (atoms.rs) used the raw guarded literals `"min-h-[44px] min-w-[44px]"` and
  `"min-h-[56px] min-w-[56px]"` as pattern strings in `.matches(...)`. The
  `render_functions_use_constants_not_literals` guard (added this phase) scans all
  `.rs` files in `src/render/` — including test code — so these triggered the assertion.
- **Fix:** Replaced both with `crate::render::classes::HIT_TARGET_MIN` and
  `crate::render::classes::HIT_TARGET_NUMPAD` respectively. rustfmt reformatted the
  chain on the `NUMPAD` call to multi-line.
- **Files modified:** `ferro-json-ui/src/render/atoms.rs`
- **Commit:** e4914e01 (bundled into Task 3 commit)

**2. [Rule 2 - Missing] `prompt_under_size_budget` cap exceeded**
- **Found during:** Task 3 CI gate
- **Issue:** Adding three POS component specs to `BUILTIN_SPECS` pushed the catalog
  prompt over the 12 KB cap. Plan explicitly instructs to bump the cap with a Phase-256
  history comment.
- **Fix:** Bumped cap from `12 * 1024` to `13 * 1024` with history comment in catalog.rs.
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** e4914e01 (bundled into Task 3 commit)

## Known Stubs

None — all three components are fully wired with real render implementations and
BUILTIN_SPECS catalog entries. No placeholders.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: display-only-total | ferro-json-ui/src/render/containers.rs | `data-selection-total` renders a client-side display value; POST handler must re-sum hidden qty inputs — documented in T-256-10 rustdoc on `render_selection_panel` |

## Self-Check: PASSED

All key files found on disk. All three task commits (fc802fc6, 3616b2e3, e4914e01) present in git log.
