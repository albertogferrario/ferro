---
phase: 192-ferro-mcp-template-validation-docs
plan: "01"
subsystem: ferro-mcp
tags: [mcp, templates, validation, async-validator, constraint-map, uniqueness]
dependency_graph:
  requires: [190-async-rule-infrastructure-unique-rule, 191-constraintmap-portable-unique-violation-detection]
  provides: [action_handler-two-layer-template]
  affects: [ferro-mcp/src/tools/code_templates.rs]
tech_stack:
  added: []
  patterns: [two-layer-uniqueness, AsyncValidator+unique, ConstraintMap+map_constraint]
key_files:
  modified:
    - ferro-mcp/src/tools/code_templates.rs
decisions:
  - "D-02 applied: enrich existing action_handler template in-place (not a new variant) — matches ROADMAP SC1 wording literally"
  - "write site uses SeaORM-native .update(db.inner()) not framework .save() — required for Result<T, DbErr> which MapConstraintExt targets"
  - "SC1 audit is file-level: grep for unique( co-occurring with ConstraintMap in the same .rs file"
metrics:
  duration: "430s"
  completed: "2026-06-09"
  tasks_completed: 2
  files_modified: 1
---

# Phase 192 Plan 01: Enrich action_handler Template with Two-Layer Uniqueness Pattern Summary

**One-liner:** action_handler code template now shows AsyncValidator+unique (proactive UX check) before the write and ConstraintMap+map_constraint at the SeaORM-native write site (TOCTOU defensive net) in one realistic update handler.

## What Was Built

The ferro-mcp `action_handler` CodeTemplate in `ferro-mcp/src/tools/code_templates.rs` was enriched in-place to demonstrate the complete two-layer uniqueness pattern an agent must use whenever a unique field is involved:

**Layer 1 — proactive (UX):** `AsyncValidator::new(&data).async_rule(field, unique(table, col).ignore(id)).validate_async().await` runs before the write. The `.ignore(id)` call excludes the current record so an update that keeps its own value does not falsely fail. On `Err(AsyncValidationError::Validation(e))`, `e.with_old_input(&data).into_action_error(back_url)` is returned. On `Err(AsyncValidationError::Infra(fe))`, `fe.into()` propagates the infrastructure failure.

**Layer 2 — defensive (concurrency net):** A `ConstraintMap` is built with `.on(constraint_name, field, message).sqlite(table.field)`, then the write is `active.update(db.inner()).await.map_constraint(&map, &data, back_url)?`. This closes the TOCTOU race: a concurrent insert that wins between the proactive check and the write produces the same field-level validation error instead of leaking a raw SQL error.

**Write site clarification:** `map_constraint` is implemented for `Result<T, sea_orm::DbErr>`. The write must be a SeaORM-native call (`.update(db.inner())` / `.insert(db.inner())`), not the framework-wrapped `.save()` which returns `FrameworkError` and does not type-check with `MapConstraintExt`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Enrich action_handler template with both validation layers | 6027bb47 | ferro-mcp/src/tools/code_templates.rs |
| 2 | SC1 catalog audit + ferro-mcp build + workspace quality gate | 5137f59b | (audit only, no code changes) |

## Decisions Made

- **D-02 applied:** Enriched the existing `action_handler` template in-place rather than adding a new `unique_form_handler` variant. This matches the ROADMAP SC1 wording "the `action_handler` code template includes both".
- **SeaORM-native write:** The template uses `.update(db.inner()).await.map_constraint(...)` not `.save().await?`, because `MapConstraintExt` is implemented for `Result<T, sea_orm::DbErr>` and `.save()` returns `FrameworkError`.
- **Three new placeholders added:** `{{field}}` (snake_case column name), `{{table}}` (snake_case table name), `{{constraint_name}}` (Postgres UNIQUE constraint identifier).
- **Imports updated** to six lines covering `ferro::{action, ActionError, ActionResult, Request, DB}`, `ferro::{AsyncValidator, AsyncValidationError, unique, rules, required, string}`, `ferro::{ConstraintMap, MapConstraintExt}`, `sea_orm::{ActiveModelTrait, EntityTrait}`, and both entity module paths.

## SC1 Invariant Audit

SC1 passed: `unique(` appears in the file only where `ConstraintMap` also appears (same file). The file-level audit command:

```bash
if grep -q 'unique(' ferro-mcp/src/tools/code_templates.rs && ! grep -q 'ConstraintMap' ferro-mcp/src/tools/code_templates.rs; then
  echo "SC1 FAIL"; exit 1
else
  echo "SC1 audit pass"
fi
```

**Known limitation of this audit:** The check is file-level, not per-template-block. If a future contributor adds a second template that contains `unique(` without a `ConstraintMap` block in its own `code` string, the audit would still pass as long as the first enriched template also exists in the same file. A per-block audit (parsing each `code: r#"..."#` string independently) would catch that case. This limitation is recorded here for future tightening if a second template introduces `unique(`.

## Quality Gate

- `cargo fmt --all -- --check`: clean
- `cargo clippy --all --all-targets -- -D warnings`: clean
- `cargo test --all-features`: all suites pass, zero failures
- `action_handler_template_registered` test: green (asserts `#[action(redirect_to`, `ActionResult`, and imports containing `action`/`ActionError`/`ActionResult`/`Request` — all still present in the enriched template)

## Deviations from Plan

None — plan executed exactly as written. The checker-fixed plan specified the SeaORM-native `.update(db.inner())` pattern and that was honored directly.

## Known Stubs

None. The template is illustrative source code (a string literal read by agents), not runtime logic with data wiring. The sample identifiers `pages`/`slug`/`pages_slug_unique` are the sanctioned illustrative exception per D-07.

## Threat Flags

None. This plan edits a Rust string literal (a code template). No new runtime, no untrusted-input path, no new attack surface introduced.

## Self-Check: PASSED

- `ferro-mcp/src/tools/code_templates.rs` exists and contains `AsyncValidator`, `ConstraintMap`, `map_constraint`, `{{constraint_name}}`
- Commit `6027bb47` exists (Task 1)
- Commit `5137f59b` exists (Task 2)
