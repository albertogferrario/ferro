# Phase 243: App integration, e2e, envelope guard & catalog/docs - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-24
**Phase:** 243-app-integration-e2e-envelope-guard-catalog-docs
**Mode:** `--auto` (recommended defaults auto-selected)
**Areas discussed:** E2E drive surface, order projection flip, envelope-guard extension, confirmation-flow e2e, catalog/docs scope

---

## E2E drive surface (CI gate vs live :8090)

| Option | Description | Selected |
|--------|-------------|----------|
| In-process handle_tools_call harness as the CI gate; live :8090 bearer = manual UAT | Mirrors mcp_write_dispatch.rs/single_source.rs; same kernel + scope path; no live server in CI | ✓ |
| Live :8090 HTTP + bearer as the CI gate | Heavier (HTTP server + bearer auth in CI); duplicates the kernel path the in-process harness already covers | |

**User's choice:** In-process CI gate + live drive as manual UAT (auto, recommended).
**Notes:** Consistent with the "CI-testable without live spend" principle; the live :8090 +
chrome-mcp harness is recorded as a HUMAN-UAT smoke.

---

## App order projection flip

| Option | Description | Selected |
|--------|-------------|----------|
| .creatable/.updatable/.deletable + .mcp_write_ability("manage-orders") | Additive to the existing order.rs; ability matches the ferro-mcp-server test fixtures | ✓ |
| Introduce a new minimal projection for the demo | Rejected — the order projection is the canonical Process/StateMachine example; flipping it proves the SM interaction (status server-set on create, never an update input) | |

**User's choice:** Flip order.rs (auto, recommended).
**Notes:** SM present → create sets status=draft server-side; soft_delete_column=deleted_at
(Phase 239 substrate). validate() passes (CRUD-07).

---

## Structured-envelope regression guard extension

| Option | Description | Selected |
|--------|-------------|----------|
| Extend the existing content[0].type==text + structuredContent assertions per CRUD verb | Reuses mcp_tenant_isolation.rs pattern | ✓ |
| New bespoke envelope assertion helper | Rejected — the existing pattern is the established lock | |

**User's choice:** Extend existing pattern (auto, recommended).

---

## Confirmation-flow e2e

| Option | Description | Selected |
|--------|-------------|----------|
| Drive delete without token → confirmation_required(+request_tool); then request_confirm→confirm→soft-delete; feature-gated | Mirrors single_source.rs destructive-path gating | ✓ |
| Skip confirmation in e2e | Rejected — SC#3 explicitly requires it | |

**User's choice:** Full confirm-flow e2e (auto, recommended).

---

## Catalog/docs scope

| Option | Description | Selected |
|--------|-------------|----------|
| Update ferro-mcp code_templates + generation_context + docs/src for the CRUD opt-in; keep separate from crud_operations.rs; verify component drift-guard not tripped | Teaches the authoring agent the opt-in; avoids surface conflation | ✓ |
| Also rewrite crud_operations.rs | Rejected — that is a separate developer-MCP introspection tool (D-09) | |

**User's choice:** Authoring surface + docs, scoped (auto, recommended).
**Notes:** CRUD tools are not json-ui components → component-count drift guards must stay
unchanged (a trip here is a false positive).

---

## Claude's Discretion
- Test-module layout/fixture naming; confirmation feature-gating per single_source.rs precedent;
  exact docs/code_templates wording.

## Deferred Ideas
None — integration/closeout phase; Tracks B–D are a future milestone.
