---
phase: 121
plan: "05"
subsystem: docs
tags: [json-ui, expressions, json-schema, documentation]
dependency_graph:
  requires: []
  provides: [expressions-doc, json-schema-doc]
  affects: [docs/src/SUMMARY.md]
tech_stack:
  added: []
  patterns: [mdBook nav, JSON Schema draft-07]
key_files:
  created:
    - docs/src/json-ui/expressions.md
    - docs/src/json-ui/json-schema.md
  modified:
    - docs/src/SUMMARY.md
decisions:
  - "expressions.md is a dedicated reference page separate from data-binding.md to allow direct linking and avoid overloading the existing page"
  - "json-schema.md documents --component flag alongside --output/--pretty for completeness"
metrics:
  duration: "~5 minutes"
  completed: "2026-05-15T16:42:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 1
---

# Phase 121 Plan 05: Expression System and JSON Schema Documentation Summary

Two new JSON-UI reference pages covering the expression system (`$data`/`$template`) and the `ferro json-ui:schema` CLI, plus SUMMARY.md nav update.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create expressions.md | 8403f0b1 | docs/src/json-ui/expressions.md |
| 2 | Create json-schema.md and update SUMMARY.md | 307afa19 | docs/src/json-ui/json-schema.md, docs/src/SUMMARY.md |

## What Was Built

**expressions.md** — Dedicated reference for the two-expression-type system:
- `$data` with RFC 6901 JSON Pointer syntax, type preservation, and missing-path behaviour
- `$template` with `{/path}` placeholder substitution, escape sequences, and always-string output
- Scope table: resolved in `element.props` only, not in title/action/visible/children
- Single-pass guarantee and injection-prevention rationale
- Hard cap section explicitly listing `$if`, `$for`, `$state`, `$bind`, `$map`, `$reduce` as non-existent
- Infallible semantics (malformed expressions degrade to literal JSON, no panic)

**json-schema.md** — Reference for the JSON Schema export CLI:
- CLI flags: `--output`, `--pretty`, `--component`
- VS Code `.vscode/settings.json` integration pattern
- Other-editor note (JSON Schema draft-07 compatible)
- Coverage table (Spec shape, element fields, per-component props, expression objects, custom plugins)
- Partial schema output snippet illustrating the structure
- Keep-up-to-date guidance (re-run after adding plugins or upgrading Ferro)

**docs/src/SUMMARY.md** — Two new entries added after Plugins in the JSON-UI section, in the target order (Expressions, JSON Schema).

## Decisions Made

- expressions.md is a dedicated reference page (not merged into data-binding.md) to allow direct linking from ferrotype errors and MCP catalog
- The `--component` flag is documented alongside the standard flags even though the plan examples only showed `--output`/`--pretty` — it is verified present in json_ui_schema.rs

## Deviations from Plan

None — plan executed exactly as written. The `--component` flag inclusion is supported by the verified source (json_ui_schema.rs line 10 signature) and improves completeness without scope change.

## Known Stubs

None.

## Threat Flags

None. No new network endpoints or auth paths introduced — documentation files only.

## Self-Check: PASSED

- docs/src/json-ui/expressions.md: FOUND
- docs/src/json-ui/json-schema.md: FOUND
- docs/src/SUMMARY.md contains expressions.md: FOUND (line 55)
- docs/src/SUMMARY.md contains json-schema.md: FOUND (line 56)
- Commit 8403f0b1: FOUND
- Commit 307afa19: FOUND
