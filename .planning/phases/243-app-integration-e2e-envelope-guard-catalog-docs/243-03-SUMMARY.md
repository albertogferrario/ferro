---
phase: 243-app-integration-e2e-envelope-guard-catalog-docs
plan: 03
subsystem: authoring, docs
tags: [ferro-mcp, code_templates, generation_context, projections, crud, documentation]

requires:
  - phase: 243-01
    provides: order projection CRUD flip (creatable/updatable/deletable + mcp_write_ability)
  - phase: 243-02
    provides: CRUD e2e harness (create→list→update→delete, envelope guard, auth gate, parity)

provides:
  - projection_crud code template category in ferro-mcp/src/tools/code_templates.rs
  - generation_context crud_handler extended with Option B: projection-CRUD opt-in
  - docs/src/features/projections.md MCP CRUD Opt-In section
  - full workspace gate green (fmt + clippy --all-targets + test --all-features)

affects: [ferro-mcp, docs, future phases consuming CRUD MCP tooling docs]

tech-stack:
  added: []
  patterns:
    - "code_templates: new category projection_crud guarded by test_all_categories_present assertion"
    - "generation_context: Option A (REST) + Option B (projection CRUD) pattern in crud_handler"
    - "docs: ## MCP CRUD Opt-In section as peer to ## MCP Tools — prerequisites table, derived tool set, auth, confirmation flow, D-09 separation"

key-files:
  created: []
  modified:
    - ferro-mcp/src/tools/code_templates.rs
    - ferro-mcp/src/tools/generation_context.rs
    - docs/src/features/projections.md

key-decisions:
  - "Template includes .soft_delete_column('deleted_at') to match the shipped order.rs exactly (line 19)"
  - "generation_context Option B comment covers the full prerequisite set: tenant_column, deleted_at, mcp_ability reads, mcp_write_ability writes, status excluded when StateMachine present"
  - "docs MCP CRUD Opt-In section uses builder chain matching order.rs (view-orders/manage-orders ability names)"
  - "json-ui drift guards stay at 47 (CRUD tools are MCP tools, not json-ui components — D-10)"
  - "crud_operations.rs not touched (D-09 — separate developer-MCP SQL surface)"
  - "docs/protocol/schemas/*.json test churn reverted (unrelated to phase)"

requirements-completed: [CRUD-01, CRUD-02, CRUD-03]

duration: 12min
completed: 2026-06-24
---

# Phase 243 Plan 03: Catalog/Docs — MCP CRUD Opt-In Summary

**`ferro-mcp` authoring surface and `docs/src/` brought to the same quality bar as the Rust CRUD API: `projection_crud` code template category, `generation_context` opt-in prose, and a `## MCP CRUD Opt-In` docs section matching the shipped order projection — workspace gate clean.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-06-24T10:37:44Z
- **Completed:** 2026-06-24T10:49:12Z
- **Tasks:** 3 / 3
- **Files modified:** 3 source files

## Accomplishments

### Task 1: code_templates projection_crud category + generation_context crud_handler extension

Added `projection_crud_templates()` to `ferro-mcp/src/tools/code_templates.rs`:
- New private fn returning one `CodeTemplate` with `category: "projection_crud"`
- Template shows all four builder calls matching the shipped `order.rs`: `.mcp_write_ability`, `.creatable(true)`, `.updatable(true)`, `.deletable(true)`, `.soft_delete_column("deleted_at")`
- Template documents derived tools (`create_{{service}}`, `update_{{service}}`, `delete_{{service}}`, `list_{{service}}`) and the status exclusion rule when a StateMachine exists
- `build_templates()` extended with `templates.extend(projection_crud_templates())`
- `test_all_categories_present` extended with `categories.contains("projection_crud")` guard assertion

Extended `crud_handler` in `ferro-mcp/src/tools/generation_context.rs`:
- Original handler content becomes "Option A: Traditional REST handler (web surface)"
- "Option B: Projection-derived MCP CRUD tools (agent surface)" appended as comment block
- Prerequisites documented: `tenant_column`, `deleted_at` column, `mcp_ability` for reads, `mcp_write_ability` for writes, StateMachine status exclusion
- No struct field added — only string content of existing `crud_handler` field changed; `is_empty()` test continues to pass

`crud_operations.rs` not touched (D-09 confirmed via `git diff --quiet`).

### Task 2: docs/src MCP CRUD Opt-In section + verify json-ui drift guards unchanged at 47

Added `## MCP CRUD Opt-In` section to `docs/src/features/projections.md` after `## MCP Tools`:
- Enabling CRUD tools: four builder calls matching `app/src/projections/order.rs` (view-orders / manage-orders ability names)
- Prerequisites table: `tenant_column`, `deleted_at` column, `mcp_write_ability` required (CRUD-07)
- Derived tool set table: create/update/delete/list with status/id/created_at/tenant_id exclusion rules
- Authorization note: `read_write` scope + `write_authorized: Some(true)` for write tools; list has no write-auth requirement
- Confirmation flow: `request_confirm_delete_<svc>` → `confirm_delete_<svc>` with single-use tokens
- D-09 separation note: `crud_operations.rs` is a separate developer-MCP SQL surface

Verified drift guards:
- `ferro-json-ui/src/catalog.rs` line 1101: `assert_eq!(crate::render::BUILTIN_TYPES.len(), 47)` — unchanged
- `ferro-mcp/src/tools/json_ui_catalog.rs` line 293-297: `catalog.components.len() == 47` — unchanged
- `cargo test -p ferro-json-ui catalog` passed with all 39 catalog tests green

### Task 3: Full BLOCKING workspace gate

Full gate run (per CLAUDE.md exact command):
- `cargo fmt --all -- --check` — exit 0 (no formatting drift)
- `cargo clippy --all --all-targets -- -D warnings` — exit 0 (no warnings; 2 crates checked, clean)
- `cargo test --all-features` — exit 0 (132 test-result groups, all "ok")

Confirmation feature exercised by `--all-features`:
- `tests::crud_e2e::tests::delete_order_confirmation_flow` — passed
- `tests::crud_e2e::tests::crud_write_requires_write_authorization` — passed
- `tests::crud_e2e::tests::crud_cross_tenant_non_disclosure` — passed
- `write_dispatch::confirmation_tests::delete_bare_call_returns_confirmation_required` — passed

Schema churn: `docs/protocol/schemas/protocol.json` and `service-def.json` regenerated by the test suite (Phase 94 export test). Reverted via `git checkout --` per project memory `project_schema_export_test_dirties_tree`. Not committed with the phase.

D-09 confirmed: `git diff --quiet ferro-mcp/src/tools/crud_operations.rs` exits 0.

## Deviations from Plan

### Auto-added: soft_delete_column in template

**Found during:** Task 1
**Issue:** PATTERNS.md code template did not include `.soft_delete_column("deleted_at")`, but the shipped `order.rs` (line 19) includes it: `.soft_delete_column("deleted_at") // CRUD-03/04: list_order excludes soft-deleted rows`. The existing uncommitted work in the tree already included this call.
**Fix:** Kept `.soft_delete_column("deleted_at")` in the template — it matches the shipped projection more accurately and the generation_context Option B comment also documents it. Template → real projection parity preserved.
**Files modified:** ferro-mcp/src/tools/code_templates.rs (existing uncommitted state)
**Rule:** Rule 2 (missing critical documentation detail) — the template would be incomplete without it.

## Known Stubs

None — this plan is documentation-only; no UI rendering, no data flow stubs.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced. Documentation only.

## Self-Check

- `ferro-mcp/src/tools/code_templates.rs` modified — verified `grep -c "projection_crud"` = 6 (fn + category + test + 3 others)
- `ferro-mcp/src/tools/generation_context.rs` modified — verified `grep -c "creatable"` >= 1
- `docs/src/features/projections.md` modified — verified `grep -c "MCP CRUD Opt-In"` = 1
- Commits verified: a7a82c61, 1ebc09d3
- `crud_operations.rs` unchanged: `git diff --quiet` exits 0
- Drift guards at 47: `cargo test -p ferro-json-ui catalog` passed
- Schema churn reverted: `git status docs/protocol/` shows clean

## Self-Check: PASSED
