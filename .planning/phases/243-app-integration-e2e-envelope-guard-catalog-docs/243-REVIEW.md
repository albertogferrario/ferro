---
phase: 243-app-integration-e2e-envelope-guard-catalog-docs
reviewed: 2026-06-24T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - app/Cargo.toml
  - app/src/projections/order.rs
  - app/src/tests/crud_e2e.rs
  - app/src/tests/mod.rs
  - docs/src/features/projections.md
  - ferro-mcp/src/tools/code_templates.rs
  - ferro-mcp/src/tools/generation_context.rs
findings:
  critical: 0
  warning: 1
  info: 4
  total: 5
status: issues_found
---

# Phase 243: Code Review Report

**Reviewed:** 2026-06-24
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Phase 243 flips the sample app's `order` projection to the CRUD data surface,
adds an in-process MCP CRUD e2e harness (`app/src/tests/crud_e2e.rs`), and
documents the projection-CRUD opt-in across `ferro-mcp` (`code_templates.rs`,
`generation_context.rs`) and `docs/src/features/projections.md`.

The work is high quality. The e2e test assertions were cross-referenced against
the shipped kernel (`framework/src/write/mod.rs::execute_crud_plan` /
`dispatch_write`) and the MCP write router
(`ferro-mcp-server/src/write_dispatch.rs`); every load-bearing assertion is
accurate:

- **Tenant isolation** is real and correct. `execute_crud_plan` appends
  `AND tenant_column = ?` to UPDATE/DELETE (and to the post-INSERT SELECT), so a
  cross-tenant write returns `rows_affected()==0 → WriteError::RecordNotFound`,
  which the MCP layer renders as `isError:true` with `error_kind:"not_found"`
  and **no** `structuredContent.result` object. The test's non-disclosure
  assertions (`crud_cross_tenant_non_disclosure`) hold exactly.
- **Write-auth gate** (`crud_write_requires_write_authorization`) matches
  `write_dispatch.rs:157` (`is_crud_write_tool && write_authorized != Some(true)
  → -32603 "authorization: write ability denied"`).
- **Server-side status injection** (D-05): a StateMachine on `order` causes
  `derive_crud_plan` to set `status = initial_state ("draft")` on create and to
  exclude `status` from the update patch — the test pins both.
- **Audit prefix divergence** (`mcp.crud.create_order` vs `web.crud.create_order`)
  matches `dispatch_write` line 798–802; the `record_id == ""` assumption on
  create is correct (`inputs.get("id").unwrap_or_default()` with no `id` at
  create time).
- **Confirmation flow** envelope (`action == "delete_order"` from the
  `confirm_delete_order` strip-prefix; `cfm_` token prefix) matches `handle_confirm`.

The only correctness issue found is a doc/code drift in the existing "Write
Contracts" `dispatch_write` example, which now omits the required trailing
`crud_plan` argument and would not compile as written. The remaining items are
minor documentation/comment precision notes.

## Warnings

### WR-01: `dispatch_write` doc example omits the required `crud_plan` argument

**File:** `docs/src/features/projections.md:596-607`
**Issue:** The "Write Contracts" example calls `dispatch_write(...)` with eight
arguments ending in `is_confirmed`, but the shipped signature
(`framework/src/write/mod.rs:681-691`) requires a trailing
`crud_plan: Option<&CrudPlan>` parameter after `is_confirmed`. The example as
written does not compile against the current API — a reader copying it for a
transition write would get an arity error. The same page's new MCP CRUD section
is otherwise accurate; this is a pre-existing example that the phase touched the
surrounding section of but did not update.
**Fix:** Add the trailing `crud_plan` argument (None for the transition path):
```rust
let outcome = dispatch_write(
    action,
    &inputs,
    tenant_id,         // from auth, never the body
    db,
    &dispatcher,
    transition_guard,  // derived from the StateMachine
    "web",             // audit channel
    #[cfg(feature = "confirmation")]
    false,
    None,              // crud_plan: None on the transition path
)
.await;
```

## Info

### IN-01: `order.rs` comment states delete is "confirmation-gated" unconditionally

**File:** `app/src/projections/order.rs:18`
**Issue:** The inline comment `.deletable(true) // derives delete_order tool,
confirmation-gated (CRUD-03)` reads as if delete is always confirmation-gated.
Per `dispatch_write` (mod.rs:749-756), the confirmation seam only fires under
`#[cfg(feature = "confirmation")]`; with the feature off, `delete_order`
soft-deletes directly (exercised by `crud_cycle_create_list_update_delete`).
The docs (`projections.md:678`) state this correctly ("confirmation-gated when
the `confirmation` feature is enabled").
**Fix:** Tighten the comment to match: `// derives delete_order tool;
confirmation-gated only when the `confirmation` feature is on (CRUD-03)`.

### IN-02: `total` seed values collide with auto-increment id expectations only by convention

**File:** `app/src/tests/crud_e2e.rs:107-125, 440-443`
**Issue:** `seed_two_tenants` inserts orders with explicit ids 1–4, and
`crud_cycle_create_list_update_delete` asserts the first created row gets
`id >= 5`. This relies on SQLite continuing the auto-increment sequence past the
highest explicitly-inserted id. The behavior is correct for SQLite, but the
coupling between the seed's explicit ids and the `>= 5` assertion is implicit;
a future change to the seed count (e.g. adding a fifth seed row) would silently
require updating the magic `5`.
**Fix:** Optional — derive the threshold from the seed set
(`const SEEDED_ROWS: i64 = 4;` then assert `new_id > SEEDED_ROWS`) so the seed
size and the assertion stay in sync. Not a defect; a maintainability note.

### IN-03: `is_manager` guard evaluator is dead on the CRUD path but retained

**File:** `app/src/tests/crud_e2e.rs:201-234`
**Issue:** `make_test_write_dispatcher` installs a full `is_manager` guard
evaluator, but every test in this file drives CRUD verbs only, where
`dispatch_write` is invoked with `crud_plan=Some(..)` and `transition_guard=None`
— the guard evaluator is never called (the file's own header comment notes
this). The closure is correct but unreachable within this test module.
**Fix:** None required. The comment at line 151-153 already documents the intent
("included for completeness ... never invoked on CRUD calls"). Retaining it
keeps the dispatcher a faithful mirror of `mcp_write_dispatch.rs`. Noting only
so a future reader does not mistake the unused branch for a coverage gap.

### IN-04: generation_context CRUD comment lists `mcp_ability` prerequisite but not `mcp_write_ability`-validate coupling

**File:** `ferro-mcp/src/tools/generation_context.rs:91-100`
**Issue:** The "Option B" CRUD comment lists the write gate
(`.mcp_write_ability(...)`) and the `deleted_at` / `tenant_column` / read-gate
prerequisites, but does not mention that `ServiceDef::validate()` fails at boot
(CRUD-07) when a write flag is set without `mcp_write_ability`. The richer
`code_templates.rs` projection_crud template and `docs/src/features/projections.md`
both call this out; the generation_context snippet is terser and omits it.
**Fix:** Optional — add one line:
`// mcp_write_ability is REQUIRED when any write flag is true (validate() fails at boot otherwise).`
Improves parity across the three introspection surfaces (held to the same
quality bar per CLAUDE.md).

---

_Reviewed: 2026-06-24_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
