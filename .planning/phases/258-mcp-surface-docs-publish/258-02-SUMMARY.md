---
phase: 258-mcp-surface-docs-publish
plan: "02"
subsystem: docs
tags: [docs, json-ui, components, layouts, spec-construction, pos, register]
dependency_graph:
  requires: [258-01]
  provides: [POS-12-SC3]
  affects: [docs/src/json-ui/components.md, docs/src/json-ui/layouts.md, docs/src/json-ui/spec-construction.md]
tech_stack:
  added: []
  patterns: [mdBook, neutral-docs-voice, format-anchor-replication]
key_files:
  created: []
  modified:
    - docs/src/json-ui/components.md
    - docs/src/json-ui/layouts.md
    - docs/src/json-ui/spec-construction.md
decisions:
  - "Extended existing pages only per D-08; no new SUMMARY.md pages added"
  - "Five new Commerce Component sections follow exact ### Tile format anchor verbatim"
  - "layouts.md fill_viewport before Register Layout Template (dependency order)"
  - "cassa.rs Rust snippet lifted directly as the register_template() docs example"
  - "mdBook build exits 0; SUMMARY.md unchanged"
metrics:
  duration_seconds: ~900
  completed_date: "2026-07-06"
  tasks_completed: 2
  files_modified: 2
---

# Phase 258 Plan 02: Human-Facing Docs — Five New Components + Register Projection Surface

## One-liner

Five new Commerce Component sections with verified props tables and the register projection surface (fill_viewport, Register Layout Template, builder API additions) documented in existing docs pages, with a green mdBook build.

## What Was Built

### Task 1 — Five new component sections in `docs/src/json-ui/components.md`

Five `###` sections inserted under `## Commerce Components`, after `### Tile` and before `## Kanban Components`, following the `### Tile` format anchor verbatim:

| Section | Props | Key coverage |
|---------|-------|--------------|
| `### TileGrid` | 7 props | data_path/$each loop, form_id pairing, fill_viewport requirement, tap-to-add model |
| `### SelectionPanel` | 4 props | form_id pairing, disable_on_submit/data-disable-on-submit, idempotency pointer, single-source-of-truth statement |
| `### FilterTabs` | 2 props | standalone vs integrated strip (TileGrid categories_path alternative) |
| `### QuantityStepper` | 4 props | data-qty-inc/data-qty-dec contract, Form placement |
| `### Numpad` | 2 props | target_field/data-numpad-target, NOT in v1 register template (author-composable) |

All props copied from verified ground truth (`component.rs:1412–1529`). Type-column conventions followed exactly (`type \| null`, `string[]`, `"quantity" \| "price"`). Interaction-model and double-submit coverage per D-09.

**Commit:** `f6df9c21`

### Task 2 — Register projection surface docs + mdBook build gate

**`docs/src/json-ui/layouts.md`** — two new `##` sections appended:

- `## fill_viewport`: spec-level flag, ferro-fill CSS chain, supported layouts (app/dashboard only), lint-rules table (`register-fill-viewport`, `register-grid-fill`, `fill-viewport-layout-unknown`), JSON snippet.
- `## Register Layout Template`: `register_template()` helper, VisualContext pattern (Rust snippet lifted from `app/src/controllers/cassa.rs`), emitted composition (fill_viewport Grid + Form + TileGrid + SelectionPanel), seven-intent-unchanged statement, cross-links to TileGrid/SelectionPanel sections.

**`docs/src/json-ui/spec-construction.md`** — `### Builder API additions` subsection inserted after the SpecBuilder section:

- `SpecBuilder::fill_viewport(bool) -> Self` with lint-rule pointer and layouts.md cross-link
- `ElementBuilder::each(path, as_) -> Self` with expressions.md `$each` cross-link
- Rust code block showing both in context

**mdBook build:** exits 0. SUMMARY.md unchanged (no new pages per D-08, `create-missing = false` constraint observed).

**Commit:** `06b15a77`

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All sections contain complete, verified content derived from authoritative sources (component.rs props, cassa.rs controller, rules.rs lint rule IDs).

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced — documentation only.

## Self-Check: PASSED

- `docs/src/json-ui/components.md` modified: FOUND (f6df9c21)
- `docs/src/json-ui/layouts.md` modified: FOUND (06b15a77)
- `docs/src/json-ui/spec-construction.md` modified: FOUND (06b15a77)
- Five component sections present: verified (`grep -c` = 5)
- `## fill_viewport` in layouts.md: verified (count = 1)
- `## Register Layout Template` in layouts.md: verified (count = 1)
- `fill_viewport` / `.each(` in spec-construction.md: verified (count = 4)
- SUMMARY.md unchanged: verified (git diff empty)
- mdBook build exits 0: verified
