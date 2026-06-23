# Phase 242: Write authorization, tenant injection & non-disclosure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-24
**Phase:** 242-write-authorization-tenant-injection-non-disclosure
**Mode:** `--auto` (recommended defaults auto-selected)
**Areas discussed:** Write-ability Gate enforcement, Tenant injection wiring, Non-disclosure envelope, Authz-deny vs non-disclosure, CRUD-07 verification

---

## Write-ability Gate enforcement

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated fail-closed authorization signal in McpContext, enforced in ferro-mcp-server write path | Host runs the real Gate; ferro-mcp-server enforces the pre-evaluated result fail-closed, separate from visibility guards | ✓ |
| Reuse `evaluated_guards` map keyed by ability name | Fewer new fields, but conflates visibility filter with auth gate | |
| Live `Gate::authorize_for` call inside ferro-mcp-server | Introduces a policy/Gate dependency into the channel crate, diverges from read path | |

**User's choice:** Dedicated fail-closed signal (auto, recommended).
**Notes:** Grounded in `renderer.rs:210` which explicitly warns the guard map is a visibility
filter, NOT an authorization gate. Researcher to confirm the carrier shape (boolean vs
ability-keyed map) and how the host derives the Gate principal from tenant_id + scope (D-03).

---

## Tenant injection wiring

| Option | Description | Selected |
|--------|-------------|----------|
| derive fills tenant_column; execute binds runtime tenant_id | derive_crud_plan sets `Some(TenantColumn{column})` from svc.tenant_column; execute_crud_plan adds INSERT col (create) / `AND col=?` (update/delete) | ✓ |
| Store tenant_id in the plan | Rejected — plan is pure/serializable; runtime tenant must come from auth | |

**User's choice:** derive/execute split (auto — design locked by Phase 241 D-09).
**Notes:** `execute_crud_plan` already takes `_tenant_id`; the `TenantColumn` slot was built
for exactly this. Non-tenant projections (svc.tenant_column unset) stay unscoped.

---

## Non-disclosure envelope

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse existing WriteError::RecordNotFound | Tenant predicate → 0 rows → existing `error_kind:not_found` envelope; no new code | ✓ |
| Distinct cross-tenant error | Rejected — a distinct signal is itself a disclosure vector | |

**User's choice:** Reuse RecordNotFound (auto, recommended).
**Notes:** The SQL predicate IS the non-disclosure mechanism; a foreign-tenant row is
unaddressable and reads identically to a missing one.

---

## Authz-deny vs non-disclosure

| Option | Description | Selected |
|--------|-------------|----------|
| Distinct: explicit authz deny vs opaque not-found | Scope/Gate deny is an explicit permission error; target existence is non-disclosing | ✓ |
| Collapse all denials into not-found | Rejected — hides legitimate permission errors and harms agent UX | |

**User's choice:** Distinct response classes (auto, recommended).
**Notes:** "Can I write at all" is not secret; "does this specific foreign row exist" is.

---

## CRUD-07 verification

| Option | Description | Selected |
|--------|-------------|----------|
| Verify-only boot-time test | Assert validate() rejects CRUD-without-mcp_write_ability; rule shipped 5cb17d60 | ✓ |
| Re-implement validation | Rejected — already shipped | |

**User's choice:** Verify-only (auto — locked).

---

## Claude's Discretion

- Carrier field shape for write authorization (boolean vs ability-keyed map) — pending researcher confirmation (D-03)
- Test fixture layout, SQL placeholder/dialect details, exact McpContext field name

## Deferred Ideas

None — all out-of-scope items (app flip, e2e, regression guard, docs) are already Phase 243.
