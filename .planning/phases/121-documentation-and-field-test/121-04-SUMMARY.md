---
phase: 121
plan: "04"
subsystem: docs
tags: [json-ui, documentation, v2-spec, layouts, plugins]
dependency_graph:
  requires: []
  provides: [layouts-v2-docs, plugins-v2-docs]
  affects: [docs/src/json-ui/layouts.md, docs/src/json-ui/plugins.md]
tech_stack:
  added: []
  patterns: [v2-json-spec, spec-file-layout-field, register_layout, register_plugin]
key_files:
  modified:
    - docs/src/json-ui/layouts.md
    - docs/src/json-ui/plugins.md
decisions:
  - "layouts.md: layout selected via spec file field, not builder method"
  - "plugins.md: all usage examples in JSON spec format, no Rust component construction"
metrics:
  duration: "81s"
  completed: "2026-05-15"
  tasks_completed: 1
  files_modified: 2
---

# Phase 121 Plan 04: layouts.md and plugins.md v2 Rewrite Summary

**One-liner:** Rewrote layouts.md and plugins.md to use v2 JSON spec format — `"layout"` field in spec, element-level `"type"` for plugins, zero v1 builder symbols.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Rewrite layouts.md and plugins.md for v2 | 82053a9a | docs/src/json-ui/layouts.md, docs/src/json-ui/plugins.md |

## What Was Built

**layouts.md:** Replaced the v1 `JsonUiView::new().layout("dashboard")` builder pattern with the v2 `"layout"` field in spec files. Documents all four built-in layouts (`"dashboard"`, `"app"`, `"auth"`, and the no-field default) with a complete spec file example for each. Custom layout registration via `register_layout` shown with `LayoutContext` fields documented.

**plugins.md:** Replaced `ComponentNode` / `Component::Plugin(PluginProps {...})` Rust construction with v2 element JSON format — plugin usage is now a `"type": "PluginName"` element in a spec file, identical in shape to built-in components. Built-in Map plugin documented with full props table and multi-marker spec example. Custom plugin authoring documented via `JsonUiPlugin` trait implementation and `register_plugin` registration.

## Verification Results

```
V1 symbols (JsonUiView|ComponentNode|Component::|.layout()):  0 matches
layouts.md "layout": occurrences:                             5 (>= 3 required)
plugins.md '"type": "Map"' occurrences:                       2 (>= 1 required)
register_layout occurrences (layouts.md):                     2
register_plugin occurrences (plugins.md):                     2
```

All acceptance criteria passed.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — documentation only, no new attack surface.

## Self-Check: PASSED

- `docs/src/json-ui/layouts.md` exists: FOUND
- `docs/src/json-ui/plugins.md` exists: FOUND
- Commit `82053a9a` exists: FOUND
