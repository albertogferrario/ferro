---
phase: 200
slug: per-tenant-scoping-policy-authorization-dogfood-acceptance
asvs_level: 1
audited_at: 2026-06-11
verdict: SECURED
threats_total: 16
threats_closed: 16
threats_open: 0
---

# Phase 200 — Security Audit

**Auditor:** gsd-security-auditor  
**ASVS Level:** 1  
**block_on:** high  
**Verdict:** SECURED — 16/16 threats closed, 0 open.

---

## Threat Verification

| Threat ID | Severity | Category | Disposition | Status | Evidence |
|-----------|----------|----------|-------------|--------|----------|
| T-200-01 | HIGH | Info Disclosure (cross-tenant read) | mitigate | CLOSED | `ferro-mcp-server/src/dispatch.rs:153-163` — tenant predicate injected as bound `sea_orm::Value::BigInt(Some(tid))` AFTER the user-filter loop, using only the `tenant_id` fn arg. Tests: `tenant_scoping` (line 297), `tenant_isolation` (line 312) in dispatch.rs; `tenant_a_isolation` (line 239), `tenant_b_isolation` (line 297) in `app/src/tests/mcp_tenant_isolation.rs`. |
| T-200-02 | HIGH | Tampering/EoP (filter-payload tenant override) | mitigate | CLOSED | `dispatch.rs:129-148` — filter loop iterates only `filters.as_object()` keys; each key is checked against the field allowlist via `is_filter_field`; only then is the clause pushed. Tenant predicate push at line 153 is outside this loop entirely. Unknown/non-filterable keys → `Err(InvalidFilter)`, never interpolated. |
| T-200-FAILOPEN | HIGH | Info Disclosure (fail-closed) | mitigate | CLOSED | `dispatch.rs:160-165` — `tenant_column=Some` + `tenant_id=None` branch returns `Err(InvalidFilter("tenant context required but not present"))` before any SQL is built. Test: `tenant_fail_closed` (line 328). |
| T-200-03 | HIGH | EoP (authz bypass) | mitigate | CLOSED | `app/src/controllers/mcp.rs:143` — `ferro::authorization::Gate::authorize_for(&user, ability, None)` called with the concrete loaded User before dispatch; uses the same registry as the web surface (defined once in `bootstrap.rs:85`). |
| T-200-03b | HIGH | EoP (no ability) | mitigate | CLOSED | `mcp.rs:131-138` — `service.mcp_ability.as_deref()` is `None` → immediate return of `make_tool_deny_response(...)` before reaching Gate or dispatch. Test: `policy_deny_no_ability` (line 255). |
| T-200-04 | MED/HIGH | Info Disclosure (deny body) | mitigate | CLOSED | `mcp.rs:33-44` — `make_tool_deny_response` produces `{"result":{"content":[...],"isError":true}}` with no `error` key at top level. Tests: `policy_deny_tool_error_shape` (line 322) asserts absence of `orders`, `customer_name`, `total`, `status`, `tenant_id`, digit-only tokens, `rows`, `total` keys; `deny_response_is_jsonrpc_success_not_transport_error` (line 395) confirms no top-level `error` key. |
| T-200-05 | HIGH | Spoofing/context confusion | mitigate | CLOSED | `app/src/routes.rs:57-63` — middleware stack is `BearerAuthMiddleware → TenantMiddleware::new().resolver(JwtClaimResolver::new("tenant_id", ...)).on_failure(Forbidden)`. Bearer runs first (line 57), inserts `serde_json::Value` principal; Tenant runs next (line 59). `bearer_auth.rs:35` explicitly sets `expected_tenant: None`. Test: `tenant_context_parity` (line 358) drives `TenantMiddleware::handle` with `JwtClaimResolver`, captures `current_tenant()` from inside `Next`, and asserts it equals the JWT claim — no hand-set task-local. |
| T-200-EOP-TENANT | HIGH | EoP (unknown tenant claim) | mitigate | CLOSED | `routes.rs:59-62` — `JwtClaimResolver::new("tenant_id", crate::tenant_lookup::get())` backed by `DbTenantLookup` with a real DB lookup (id closure in `tenant_lookup.rs:49-62`). Unknown `tenant_id` → `DbTenantLookup` miss → `on_failure(TenantFailureMode::Forbidden)` → 403. |
| T-200-NEUTRALIZED | MED | Info Disclosure (/authorize tenant binding) | mitigate | CLOSED | `routes.rs:69-76` — `/authorize` group has `TenantMiddleware::new().resolver(SessionUserTenantResolver::new()).on_failure(Allow)`. `SessionUserTenantResolver` (tenant_resolver.rs:38-61) reads `Auth::id()` → `User::find_by_id` → `user.tenant_id` → `Tenant::find_by_id`. `bootstrap.rs:64` mounts `SessionMiddleware::new(SessionConfig::from_env())` as first global middleware so session cookie is issued, enabling login to persist and `/authorize` to resolve the real tenant_id into the minted token. |
| T-200-03a | MED | EoP (Gate registry) | mitigate | CLOSED | `bootstrap.rs:85-90` — `Gate::define("view-orders", ...)` registered once in `bootstrap::register()`; same Gate singleton used by both web and MCP surfaces. |
| T-200-SUB | MED | Spoofing (sub parse) | mitigate | CLOSED | `mcp.rs:73-76` — `principal["sub"].as_str().and_then(|s| s.parse().ok()).ok_or_else(|| HttpResponse::new().status(400))`; non-numeric or absent sub → 400, never a default user. |
| T-200-SCHEMA | MED | Tampering (orders FK) | mitigate | CLOSED | `app/src/migrations/m20260611_create_orders_table.rs:30-35` — `Orders::TenantId` is `.big_integer().not_null()` with `ForeignKey::create().from(Orders::Table, Orders::TenantId).to(Tenants::Table, Tenants::Id)`. |
| T-200-COLMATCH | MED | Correctness→disclosure (orders columns) | mitigate | CLOSED | Migration (lines 21-30) defines columns `CustomerName`, `Total`, `Status`, `CreatedAt`, `TenantId` (+ `Id`) mapping to snake-case `customer_name`, `total`, `status`, `created_at`, `tenant_id`, `id`. `app/src/projections/order.rs:16-20` declares identical field names. |
| T-200-COUPLE | LOW | Architecture (ferro-projections dep isolation) | mitigate | CLOSED | `ferro-projections/Cargo.toml` dependencies: `schemars`, `serde`, `serde_json`, `thiserror` only. No `framework`, `ferro`, `sea-orm`, or auth crate present. `tenant_column` and `mcp_ability` are `Option<String>` in `service.rs:88,92`. |
| T-200-INFO | LOW | Info Disclosure (ServiceDef serialize) | accept | CLOSED | Accepted risk documented: `tenant_column` and `mcp_ability` serialize only developer-authored declarations (not runtime data). Both fields carry `#[serde(skip_serializing_if = "Option::is_none")]` (`service.rs:87,91`) so they are absent from serialized output when not set. Test: `tenant_column_and_mcp_ability_skip_serializing_when_none` (service.rs line 1326). |
| T-200-DOGFOOD | n/a | Acceptance integrity | mitigate | CLOSED | `200-ACCEPTANCE.md` frontmatter: `verdict: GO`, `status: COMPLETE`. First run was NO-GO (missing `SessionMiddleware`); design revision in commit `ee8aed92` (sessions table migration + global `SessionMiddleware` in `bootstrap.rs:64` + `SESSION_SECURE=false`). Re-run GO in both tenant directions: Run A (`alice@acme.test`, `tenant_id=1`) → 2 rows all `tenant_id=1`; Run B (`bob@globex.test`, `tenant_id=2`) → 2 rows all `tenant_id=2`. |

---

## Threat Flags from SUMMARY.md

The only `## Threat Flags` section appears in `200-04-SUMMARY.md` and reads:

> None introduced beyond those in the plan's threat model (T-200-05, T-200-EOP-TENANT, T-200-NEUTRALIZED, T-200-03a).

All four flags mentioned map directly to registered threat IDs in the threat register above. No unregistered flags.

---

## Accepted Risks Log

| Threat ID | Category | Rationale |
|-----------|----------|-----------|
| T-200-INFO | Info Disclosure (ServiceDef serialize) | `tenant_column` and `mcp_ability` are developer-authored schema declarations, not runtime data. Exposure in serialized output (e.g., `tools/list` schema) discloses only the column name and Gate ability name — both controlled by the developer and already visible in source code. `skip_serializing_if = "Option::is_none"` limits exposure to projections that explicitly set these fields. |

---

## Notes

- `validate_bearer` and `challenge_response` in `mcp.rs` are inside `#[cfg(test)]` only (lines 18-27, 178). Not compiled in production. Intentional test scaffolding, no production impact.
- The NO-GO → GO loop in dogfood acceptance is correctly documented as a design revision, not a phase bypass. The session middleware gap was architectural, caught by the acceptance gate as designed.
