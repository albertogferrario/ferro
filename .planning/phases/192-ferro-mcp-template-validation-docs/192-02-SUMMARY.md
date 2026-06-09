---
phase: 192-ferro-mcp-template-validation-docs
plan: "02"
subsystem: docs
tags: [validation, async-rules, constraint-mapping, documentation]
dependency_graph:
  requires: [192-01]
  provides: [VALID-06-docs]
  affects: [docs/src/features/validation.md]
tech_stack:
  added: []
  patterns: [two-layer-uniqueness-docs]
key_files:
  modified:
    - docs/src/features/validation.md
decisions:
  - "Placed both new sections after ## Rules Reference and before ## Best Practices — stable anchor placement, top-level ## so mdbook auto-kebab anchors are predictable"
  - "MCP Tools section extended with both validation and handler category notes — handler note added alongside existing validation prose, not replacing it"
  - "Used db.inner() in constraint-mapping example matching SeaORM-native write pattern from action_handler template"
metrics:
  duration: "426s"
  completed: "2026-06-09"
  tasks_completed: 2
  files_modified: 1
---

# Phase 192 Plan 02: Async Rules + Constraint Mapping Documentation Summary

**One-liner:** Async Rules (DB-backed) and Constraint Mapping sections added to validation.md with cross-links and MCP Tools handler-template note, closing VALID-06 documentation surface.

## What Was Built

Two new top-level sections in `docs/src/features/validation.md`:

**`## Async Rules (DB-backed)`** — covers `AsyncValidator` with the `unique` async rule for create forms (no exclude-self) and edit forms (`.ignore(id)` / `.ignore_on`). Shows the three-arm match on `AsyncValidationError::Validation` / `AsyncValidationError::Infra`. Explains fail-fast (no DB query for a field that failed sync rules). Closes with a cross-link to `## Constraint Mapping`.

**`## Constraint Mapping`** — covers the `ConstraintMap` builder (`.on(...)`, `.sqlite(...)`), `map_constraint` on `Result<T, DbErr>`, the two-layer rationale (proactive UX catch before write + defensive TOCTOU net at write), and the Postgres-vs-SQLite identity note (constraint NAME vs `table.column` from error message). Opens with a cross-link back to `## Async Rules (DB-backed)`.

**`## MCP Tools` update** — extended to mention the `handler` category `action_handler` template demonstrates the full two-layer pattern; both `category: "validation"` and `category: "handler"` uses documented.

## Verification Results

All Task 1 greps passed:
- `## Async Rules` heading present
- `## Constraint Mapping` heading present
- `ConstraintMap` and `AsyncValidator` symbols present
- `.ignore(` and `.sqlite(` / `map_constraint` present
- Cross-links `](#constraint-mapping)` and `](#async-rules-db-backed)` both present
- `action_handler` / handler template note present

Task 2 API audit:
- All five symbols (`AsyncValidator`, `AsyncValidationError`, `ConstraintMap`, `MapConstraintExt`, `unique`) confirmed in `framework/src/lib.rs` re-exports

Docs build: `mdbook build docs` — SUCCESS (HTML book written; no broken links, no malformed fences).

Gate: `cargo fmt --all -- --check` clean, `cargo clippy --all --all-targets -- -D warnings` clean, `cargo test --all-features` all pass.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 | 367bc1fa | docs(192-02): add async rules and constraint mapping sections to validation.md |
| Task 2 | (verification-only) | No file changes — mdbook build + symbol audit only |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None. Docs-only edit; no runtime, no new attack surface.

## Self-Check: PASSED

- `docs/src/features/validation.md` — modified and committed at 367bc1fa
- All verification greps passed
- mdbook build succeeded
- Rust gate (fmt + clippy + test) passed
