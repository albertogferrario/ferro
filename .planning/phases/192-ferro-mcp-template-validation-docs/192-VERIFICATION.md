---
phase: 192-ferro-mcp-template-validation-docs
verified: 2026-06-09
status: passed
score: 3/3
overrides_applied: 0
---

# Phase 192: ferro-mcp Template + Validation Docs — Verification Report

**Phase Goal:** Make the two-layer uniqueness pattern (proactive async `unique` rule + defensive `ConstraintMap`) discoverable to an agent and a human, so neither layer is used in isolation — via the ferro-mcp `action_handler` template and the validation docs page.
**Verified:** 2026-06-09 (grep + build; docs/template phase)
**Status:** passed
**Requirement:** VALID-06

---

## Success Criteria

| SC | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| SC1 | ferro-mcp `action_handler` template shows BOTH layers; no template shows `unique` without a downstream `ConstraintMap` | PASS | `code_templates.rs` `action_handler`: `AsyncValidator`+`unique(...).ignore(id)` (Layer 1) then `ConstraintMap::new().on(...).sqlite(...)` + `active.update(db.inner()).await.map_constraint(...)` (Layer 2). Catalog audit: only one `unique(` site, co-located with `ConstraintMap`. New placeholders `{{field}}`/`{{table}}`/`{{constraint_name}}` added. `cargo test --all-features` green (incl. `action_handler_template_registered`). |
| SC2 | `validation.md` has a dedicated async-rules section AND a dedicated constraint-mapping section | PASS | `## Async Rules (DB-backed)` (create form + edit form with `.ignore(record_id)`, `AsyncValidationError` arms) and `## Constraint Mapping` (`ConstraintMap` builder + `map_constraint` + two-layer rationale + PG/SQLite identity note). |
| SC3 | The two sections cross-reference each other | PASS | In-page links both ways: `](#constraint-mapping)` from the async section and `](#async-rules-db-backed)` from the mapping section. `## MCP Tools` updated to note the handler template demonstrates the two-layer pattern. |

**Score:** 3/3.

---

## API Accuracy (the main risk — verified)

Both the template and the docs use the real, current public API (confirmed against `framework/src/lib.rs` re-exports and the 190/191 source):
- Proactive: `AsyncValidator::new(&data).rules(field, rules![...]).async_rule(field, unique(table, col).ignore(id)).validate_async().await` → `AsyncValidationError::{Validation, Infra}`.
- Defensive: `ConstraintMap::new().on(constraint, field, message).sqlite("table.col")` then a **SeaORM-native** write `active.update(db.inner()).await.map_constraint(&map, &data, url)?` (`db = DB::connection()?`).

A checker BLOCKER was caught and fixed pre-execution: the original template used `record.save().await.map_constraint(...)`, which does not type-check because `MapConstraintExt` is impl'd only for `Result<T, sea_orm::DbErr>` (and `.save()` returns `Result<_, FrameworkError>`). Corrected to the SeaORM-native `.update/.insert(db.inner())` pattern that returns `Result<T, DbErr>`.

---

## Known Limitation (recorded, non-blocking)

The SC1 catalog audit is **file-level** (`unique(` and `ConstraintMap` co-occur in `code_templates.rs`), not per-template-block. It passes correctly now (one enriched template). If a future template introduces `unique(` in a separate block without its own `ConstraintMap`, the file-level grep would still pass — tighten to per-block scope before adding any such template.

---

## Quality Gate

`cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, `cargo test --all-features` — all green. mdbook builds clean.

---

_Verified: 2026-06-09 (grep + build, docs/template phase)_
