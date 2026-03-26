---
phase: 109-cli-reference-completeness
plan: 01
subsystem: documentation
tags: [cli, reference, docs]
dependency_graph:
  requires: []
  provides: [complete-cli-reference]
  affects: [docs/src/reference/cli.md]
tech_stack:
  added: []
  patterns: [four-part-template, command-summary-table]
key_files:
  created: []
  modified:
    - docs/src/reference/cli.md
decisions:
  - generate-routes documented as internal note under generate-types, not as standalone command (per research — no Commands enum variant in main.rs)
  - projection:check entry includes prominent projections feature gate callout
  - make:policy body section added without adding duplicate table row (row already existed)
  - Command Summary table rows re-sorted alphabetically within functional groups
metrics:
  duration: 148s
  completed: 2026-03-26
  tasks_completed: 2
  files_modified: 1
---

# Phase 109 Plan 01: CLI Reference Completeness Summary

Added documentation for all 13 previously undocumented CLI commands to `docs/src/reference/cli.md`, including 12 new standalone sections and a route-generation note under `generate-types`, plus 11 new Command Summary table rows.

## What Was Built

- **12 new `### \`ferro\`` command sections** added to `docs/src/reference/cli.md`
- **1 internal note** about route generation added under the existing `generate-types` section
- **11 new rows** added to the Command Summary table (making `make:policy` complete — its full body section was the 12th addition)
- **New section:** `## Validation & Diagnostics` containing `api:check`, `projection:check`, and `validate:contracts`

### New Command Sections Added

| Command | Section | Format |
|---------|---------|--------|
| `ferro clean` | Development Commands | Synopsis + Options table + numbered steps |
| `ferro make:api` | Code Generators | Synopsis + Options table + numbered steps |
| `ferro make:api-key` | Code Generators | Synopsis + Options table + numbered steps |
| `ferro make:lang` | Code Generators | Synopsis + Options table + generated files |
| `ferro make:policy` | Code Generators | Synopsis + Options table + generated file + code example |
| `ferro make:projection` | Code Generators | Synopsis + Options table + generated file |
| `ferro make:stripe` | Code Generators | Synopsis + Options table + generated files |
| `ferro make:theme` | Code Generators | Synopsis + Options table + generated files |
| `ferro make:whatsapp` | Code Generators | Synopsis + generated files |
| `ferro api:check` | Validation & Diagnostics | Synopsis + Options table + numbered steps |
| `ferro projection:check` | Validation & Diagnostics | Feature gate callout + Synopsis + Options table + numbered steps |
| `ferro validate:contracts` | Validation & Diagnostics | Synopsis + Options table + numbered steps |

### Internal Note (not a standalone command)
- `generate-routes` — noted under `generate-types` as "also runs route generation internally"

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 + Task 2 | fb74d34d | docs(109-01): add 12 missing CLI command body sections to cli.md |

## Verification Results

| Check | Result |
|-------|--------|
| `### \`ferro` heading count | 49 (37 existing + 12 new) |
| Command Summary table rows | 49 (38 existing + 11 new) |
| All 13 command names present | Yes |
| No duplicate `make:policy` row | Yes (1 row only) |
| `projection:check` feature gate notice | Yes |
| `generate-routes` is internal note only | Yes (no standalone heading) |

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- docs/src/reference/cli.md — FOUND
- 109-01-SUMMARY.md — FOUND
- Commit fb74d34d — FOUND
