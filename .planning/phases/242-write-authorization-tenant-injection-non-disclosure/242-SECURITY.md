---
phase: 242
slug: write-authorization-tenant-injection-non-disclosure
audited: 2026-06-24T00:00:00Z
asvs_level: 1
block_on: high
threats_total: 7
threats_closed: 7
threats_open: 0
status: SECURED
---

# Phase 242 Security Audit

**Phase:** 242 — Write authorization, tenant injection & non-disclosure
**ASVS Level:** 1
**Threats Closed:** 7/7
**Auditor:** Claude (gsd-security-auditor)
**Post-fix:** Verified against code state after CR-01 and CR-02 fixes (commits db230d41, 63e5a4aa)

---

## Threat Verification

| Threat ID | Sub-check | Category | Disposition | Status | Evidence |
|-----------|-----------|----------|-------------|--------|----------|
| T-242-01a | Fail-closed write-ability gate scoped to CRUD tools (CR-01 fix) | Elevation / Authorization bypass | mitigate | CLOSED | `ferro-mcp-server/src/write_dispatch.rs:131-157` — `is_crud_write_tool` detector strips confirm prefixes, checks exactly one CRUD verb prefix against service CRUD flags; gate fires only when `is_crud_write_tool && ctx.write_authorized != Some(true)`. Transition-action tools bypass (`is_crud_write_tool = false`). |
| T-242-01b | Host Gate evaluation for write tools; transition actions get `None` | Elevation / Authorization bypass | mitigate | CLOSED | `app/src/controllers/mcp.rs:328-376` — `find_map` over `["create_", "update_", "delete_"]` with single `strip_prefix` per verb. Returns `Some(Gate::authorize_for(...).is_ok())` for CRUD tools, `None` for read/transition-action tools. `McpContext { write_authorized }` populated at line 385. |
| T-242-01c | Scope gate: `read`-scope key rejects write tools | Elevation / Authorization bypass | mitigate | CLOSED | `ferro-mcp-server/src/jsonrpc.rs:75-86` — `if is_write_tool && key_scope == "read"` returns `-32603` before any dispatch. |
| T-242-01d | Deny envelope is generic; no row/column/service/filter leakage | Information Disclosure | mitigate | CLOSED | `write_dispatch.rs:158-163` — envelope is `{"error": {"code": -32603, "message": "authorization: write ability denied"}}` with no operational detail. |
| T-242-02a | `derive_crud_plan` reads `tenant_column` from `svc.tenant_column` only — never agent input | Elevation of Privilege | mitigate | CLOSED | `ferro-projections/src/executor.rs:247, 274, 293` — all three arms use `svc.tenant_column.as_ref().map(|col| TenantColumn { column: col.clone() })` (grep count = 3). |
| T-242-02b | `tenant_id` bound as `sea_orm::Value::BigInt(Some(tenant_id))` via `placeholder()` helper — never string-interpolated | Tampering / SQL Injection | mitigate | CLOSED | `framework/src/write/mod.rs:323, 379, 443, 471, 521, 575, 583, 626, 636, 1965` — 10 bind sites, all via `placeholder(backend, idx)`. No `_tenant_id` remains (grep = 0). No `tenant_column: _` remains (grep = 0). Create tenant placeholder index = `columns.len() + 1` (not +2) at line 313. |
| T-242-02c | Tenant column excluded from write input schemas via `is_server_injected_field` / `is_write_excluded_field` | Elevation of Privilege | mitigate | CLOSED | `ferro-projections/src/service.rs:236-265` — `is_server_injected_field` and `is_write_excluded_field` implemented and tested (lines 2127-2305). |
| T-242-03a | Update/Delete carry `AND <tenant_column> = ?` predicate; 0 rows → existing `WriteError::RecordNotFound`, no new variant | Information Disclosure | mitigate | CLOSED | `framework/src/write/mod.rs:424-430` (Update), `503-510` (Delete). `RecordNotFound` at line 62 is the only not-found variant; `enum WriteError` has no new cross-tenant variant. Tests `crud_cross_tenant_update_not_found` (line 2063) and `crud_cross_tenant_delete_not_found` (line 2099) assert `Err(RecordNotFound)` AND row is physically unchanged. |
| T-242-03b | Post-INSERT SELECT carries tenant predicate (CR-02 fix); post-UPDATE SELECT also tenant-scoped | Information Disclosure | mitigate | CLOSED | `framework/src/write/mod.rs:370-387` — post-INSERT SELECT conditional: `SELECT * FROM {table} WHERE id = ? AND {tc_col} = {t_ph}` with `placeholder(backend, 2)` and two bound values when `tenant_column` is `Some`. Post-UPDATE SELECT at lines 462-471 mirrors same pattern. |
| T-242-03c | `write_authorized` is a DEDICATED `McpContext` field, NOT `evaluated_guards` | Information Disclosure | avoid | CLOSED | `ferro-mcp-server/src/renderer.rs:32` — `pub write_authorized: Option<bool>` is a separate field. `grep "evaluated_guards" write_dispatch.rs` returns only a clarifying comment at line 122, no live read. |
| T-242-04 | `ServiceDef::validate()` errors on CRUD verb without `mcp_write_ability` (boot-time fail-fast) | Elevation / Repudiation | mitigate | CLOSED | `ferro-projections/src/service.rs:504-510` — rule `(self.creatable || self.updatable || self.deletable) && self.mcp_write_ability.is_none()` returns `Err(Error::Validation(...))`. Test `validate_rejects_crud_verb_without_write_ability` at line 2314 covers all three verbs + passing case. `validate()` body unchanged from shipped `5cb17d60`. |

*(Counts: 11 sub-checks across 7 logical threat items — all CLOSED.)*

---

## Regression Tests Verified Present

| Test Name | File | Covers |
|-----------|------|--------|
| `transition_action_not_denied_by_write_ability_gate` | `ferro-mcp-server/src/write_dispatch.rs:1944` | CR-01: transition-action tools not blocked by write-ability gate |
| `write_authorized_none_denies` | `ferro-mcp-server/src/write_dispatch.rs` | T-242-01: `None` → deny |
| `write_authorized_false_denies` | `ferro-mcp-server/src/write_dispatch.rs` | T-242-01: `Some(false)` → deny |
| `write_authorized_true_proceeds` | `ferro-mcp-server/src/write_dispatch.rs` | T-242-01: `Some(true)` → dispatch |
| `crud_create_injects_tenant` | `framework/src/write/mod.rs` | T-242-02: tenant bound on INSERT |
| `crud_update_tenant_predicate` | `framework/src/write/mod.rs` | T-242-02/03: same-tenant update succeeds |
| `crud_delete_tenant_predicate` | `framework/src/write/mod.rs` | T-242-02/03: same-tenant delete succeeds |
| `crud_cross_tenant_update_not_found` | `framework/src/write/mod.rs:2063` | T-242-03: cross-tenant → RecordNotFound + row unchanged |
| `crud_cross_tenant_delete_not_found` | `framework/src/write/mod.rs:2099` | T-242-03: cross-tenant → RecordNotFound + deleted_at stays NULL |
| `validate_rejects_crud_verb_without_write_ability` | `ferro-projections/src/service.rs:2314` | T-242-04: boot-time fail-fast |

---

## Accepted Risks Log

| ID | Finding | Rationale |
|----|---------|-----------|
| WR-01 | No denial audit written for CRUD-verb guard failures | CRUD verbs have no action-level guards today. The comment at `write_dispatch.rs:436` marks this as a future extension point. Deferred until CRUD guard support lands; no live gap. |
| IN-01 | Write-ability denial uses `-32603` transport-level envelope vs. `write_tool_error_result` shape | By design: authorization denials (scope gate, tenant fail-closed, write-ability gate) are uniformly transport-level. Execution-level outcomes use the tool-error shape. Documented in comment at `write_dispatch.rs:151-156`. |

---

## Unregistered Threat Flags

None. SUMMARY.md `## Threat Flags` section carries no flags that fall outside the registered threat IDs.

---

_Audited: 2026-06-24T00:00:00Z_
_Auditor: Claude (gsd-security-auditor)_
_Scope: Phase 242 post-fix code (commits db230d41, 63e5a4aa, 4a57149a)_
