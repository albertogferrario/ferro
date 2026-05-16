# Phase 146: Add KeyValueEditor component — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-22
**Phase:** 146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va
**Mode:** `--auto` (all gray areas auto-resolved with recommended defaults)
**Areas discussed:** Props structure, Suggested keys UX, Row add/remove UI, JSON serialization format, Runtime architecture, Data binding

---

## Props structure

| Option | Description | Selected |
|--------|-------------|----------|
| Mirror existing form component shape | field, label, data_path, error + suggested_keys, allow_custom_keys | ✓ |

**Auto-selected:** Mirror existing form component shape
**Notes:** `KeyValueEditorProps` has `field`, `label`, `suggested_keys`, `allow_custom_keys`, `data_path`, `error` — consistent with Input/Select/Checkbox/Switch.

---

## Suggested keys UX

| Option | Description | Selected |
|--------|-------------|----------|
| `<datalist>` on key input | Native browser autocomplete, no extra JS, matches InputProps::list pattern | ✓ |
| Custom dropdown | More control, requires JS | |
| Select element only | Restricts to predefined keys, triggered by allow_custom_keys=false | |

**Auto-selected:** `<datalist>` for free-text mode; `<select>` when `allow_custom_keys=false`
**Notes:** Matches existing `InputProps::list` datalist pattern in render.rs.

---

## Row add/remove UI

| Option | Description | Selected |
|--------|-------------|----------|
| "+ Add row" button below rows + "×" per row | Standard compact table layout | ✓ |
| Inline "+" on last row | Less discoverable | |

**Auto-selected:** "+ Add row" button below, "×" delete per row
**Notes:** Three-column layout (key | value | ×). Empty rows excluded from serialization.

---

## JSON serialization format

| Option | Description | Selected |
|--------|-------------|----------|
| Object `{"k":"v"}` | Simple, maps to HashMap<String,String> server-side | ✓ |
| Array `[{"key":"k","value":"v"}]` | Preserves order and allows duplicates | |

**Auto-selected:** Object format
**Notes:** Duplicate keys last-write wins; array format deferred to a future need.

---

## Runtime architecture

| Option | Description | Selected |
|--------|-------------|----------|
| New `runtime/key_value_editor.rs` module | Single-concern, follows existing pattern | ✓ |
| Embed in form_guards.rs | Avoids new file but violates single-concern principle | |

**Auto-selected:** New `runtime/key_value_editor.rs` with `setupKeyValueEditor()`
**Notes:** `<template>` element for row cloning. `data-kv-editor` wrapper attribute selector.

---

## Data binding

| Option | Description | Selected |
|--------|-------------|----------|
| `data_path` resolves JSON object → seed rows | Consistent with all other form components | ✓ |
| Static default_value string | Less ergonomic for server-rendered data | |

**Auto-selected:** `data_path` resolves to JSON object, seeds initial rows
**Notes:** Hidden field initialized at render time; JS syncs only on user interaction.

---

## Claude's Discretion

- Tailwind class selection for row layout and button styling
- HTML `<template>` element rendering position within the component wrapper
- Exact ARIA attributes on key/value inputs
- Test coverage structure in render.rs

## Deferred Ideas

- Array-format serialization for ordered/duplicate key pairs
- Drag-to-reorder rows
- Client-side duplicate key highlighting
- `min_rows` / `max_rows` constraints
