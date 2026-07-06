---
phase: 200-per-tenant-scoping-policy-authorization-dogfood-acceptance
verified: 2026-06-11T00:00:00Z
status: passed
score: 4/4
overrides_applied: 0
re_verification: null
---

# Phase 200: Per-Tenant Scoping, Policy Authorization, Dogfood Acceptance — Verification Report

**Phase Goal:** A tool call executes inside the token's tenant context via the existing multi-tenant middleware and is gated by the same policy layer as the web surface (no parallel permission system). The phase closes with a dogfood GO/NO-GO: a real MCP client completes browser login against a live consumer app and lists one projection's tenant-scoped data; a NO-GO triggers a design revision before completion.
**Verified:** 2026-06-11T00:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC-1: a token scoped to tenant A returns only tenant A's rows; tenant B only B's | VERIFIED | `dispatch.rs:153-163` injects bound `sea_orm::Value::BigInt(Some(tid))` predicate after the user-filter loop, covering both COUNT and SELECT. Tests `tenant_scoping`, `tenant_isolation` (dispatch unit); `tenant_a_isolation`, `tenant_b_isolation` (app integration) confirm bidirectional isolation. `tenant_fail_closed` confirms no rows leak when tenant_column=Some but tenant_id=None. |
| 2 | SC-2: a policy-denied call returns an MCP tool error with a clear message and no data disclosure | VERIFIED | `mcp.rs:143` calls `Gate::authorize_for`; `mcp_ability=None` fail-closed at line 131. `make_tool_deny_response` returns `{"result":{"content":[{"type":"text","text":"..."}],"isError":true}}`. `policy_deny_tool_error_shape` test asserts body excludes `orders`, `customer_name`, `total`, `status`, `tenant_id`, and any digit-only token. `deny_response_is_jsonrpc_success_not_transport_error` confirms no top-level `error` key. |
| 3 | SC-3: the /mcp tenant context is structurally identical to the web-surface multi-tenant middleware — no second permission system | VERIFIED | `routes.rs:57-64`: `/mcp` stack is `BearerAuthMiddleware → TenantMiddleware::new().resolver(JwtClaimResolver::new("tenant_id", ...))`. `/authorize` stack is `TenantMiddleware::new().resolver(SessionUserTenantResolver::new())`. `ferro-projections/Cargo.toml` has no `framework`/`ferro` dependency (only `serde`, `schemars`, `thiserror`). `tenant_context_parity` test drives `TenantMiddleware::handle` with a `JwtClaimResolver` and asserts `current_tenant()` is set exclusively by the middleware resolver path — comment explicitly states "no hand-set task-local". |
| 4 | SC-4: dogfood GO/NO-GO — a real MCP client completes browser login against a live app and calls tools/list then tools/call for one projection, receiving that tenant's rows | VERIFIED | `200-ACCEPTANCE.md` frontmatter: `verdict: GO`. First run was NO-GO (missing `SessionMiddleware`; design revision in commit `ee8aed92` — sessions table migration + global `SessionMiddleware::new(SessionConfig::from_env())`). Re-run GO in both tenant directions: Run A (`alice@acme.test`, `tenant_id=1`) → 2 rows all `tenant_id=1`; Run B (`bob@globex.test`, `tenant_id=2`) → 2 rows all `tenant_id=2`; `list_order` present in `tools/list`. `dogfood/run_dogfood.mjs` passes `node --check`. |

**Score:** 4/4 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-projections/src/service.rs` | `ServiceDef.tenant_column` + `mcp_ability` Option<String> fields + builder methods | VERIFIED | Lines 88/92: `pub tenant_column: Option<String>`, `pub mcp_ability: Option<String>` with `skip_serializing_if`. Builder methods at lines 134/141. Three serde tests: `tenant_and_ability_default_none_when_absent`, `tenant_column_and_mcp_ability_builder_sets_values`, `tenant_column_and_mcp_ability_skip_serializing_when_none`. |
| `ferro-mcp-server/src/dispatch.rs` | Tenant predicate injection + tenant_id parameter + fail-closed | VERIFIED | `tenant_id: Option<i64>` at line 114. Predicate injection at lines 153-163 using bound value `sea_orm::Value::BigInt(Some(tid))`. Fail-closed branch returns `Err(InvalidFilter("tenant context required but not present"))`. Tests: `tenant_scoping`, `tenant_isolation`, `tenant_fail_closed`, `non_tenant_unscoped`. |
| `ferro-mcp-server/src/jsonrpc.rs` | `handle_tools_call` forwards tenant_id to dispatch | VERIFIED | `tenant_id: Option<i64>` as last parameter at line 52. Forwarded to `dispatch(service, filters, limit, offset, db, tenant_id)` at line 83. |
| `app/src/controllers/mcp.rs` | Gate check + fail-closed + D-09 tool error + tenant_id forwarding | VERIFIED | `Gate::authorize_for` at line 143. `mcp_ability.as_deref()` fail-closed at line 131. `make_tool_deny_response` helper with `isError:true` at lines 29-43. `current_tenant().map(|t| t.id)` at line 155. `validate_bearer` is `#[cfg(test)]`-only (lines 179, 221) — not in production path. |
| `app/src/routes.rs` | `/mcp` stack: `BearerAuthMiddleware → TenantMiddleware(JwtClaimResolver("tenant_id"))`; `/authorize`: `TenantMiddleware(SessionUserTenantResolver)` | VERIFIED | Lines 57-74 confirm the exact middleware ordering. Comment at lines 50-52 documents the contract. |
| `app/src/bootstrap.rs` | `Gate::define("view-orders")` + two-tenant seed + global `SessionMiddleware` | VERIFIED | Line 64: `global_middleware!(SessionMiddleware::new(SessionConfig::from_env()))`. Line 85: `Gate::define("view-orders", ...)`. Lines 139-157: `acme` and `globex` tenant seed. Lines 169/179: `alice@acme.test` and `bob@globex.test` user seed. |
| `app/src/migrations/m20260611_create_orders_table.rs` | `orders` table with `id`, `customer_name`, `total`, `status`, `created_at`, `tenant_id` columns | VERIFIED | All six columns present including FK on `tenant_id → Tenants.Id`. |
| `app/src/migrations/m20260611_create_sessions_table.rs` | `sessions` table (dogfood fix) | VERIFIED | File exists; comment confirms it mirrors `framework::session::driver::database::sessions::Model`. Created in commit `ee8aed92`. |
| `app/src/migrations/m20260611_add_tenant_id_to_users.rs` | `users.tenant_id` | VERIFIED | Migration file exists. |
| `app/src/tests/mcp_tenant_isolation.rs` | Bidirectional isolation + tenant_context_parity (SC-3) | VERIFIED | `tenant_a_isolation` (line 239), `tenant_b_isolation` (line 297), `tenant_context_parity` (line 358). SC-3 parity confirmed: test doc comment at lines 4-5 states the single-mechanism invariant. |
| `dogfood/run_dogfood.mjs` | Scripted MCP client for acceptance run | VERIFIED | File exists. Passes `node --check`. Contains `tools/list`, `tools/call`, `/register`, `/token` invocations. Tenant-isolation check at lines 314-332 asserts every row's `tenant_id` matches authenticated tenant; exits non-zero on mismatch. |
| `200-ACCEPTANCE.md` | GO/NO-GO record | VERIFIED | Frontmatter `verdict: GO`. Contains both-direction evidence (Run A, Run B). NO-GO → design revision → GO sequence documented. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ServiceDef.tenant_column` | `dispatch.rs` predicate injection | plain metadata field read by dispatch | WIRED | `dispatch.rs:153` reads `service.tenant_column` |
| `ServiceDef.mcp_ability` | `mcp.rs` Gate check | plain metadata field read by handler | WIRED | `mcp.rs:131` reads `service.mcp_ability.as_deref()` |
| `dispatch.rs` tenant predicate | WHERE clause of both count_sql and data_sql | single `where_clauses.push` site before `where_str` build | WIRED | Summary confirms single injection site; `where_str` used identically for COUNT and SELECT |
| `handle_tools_call(tenant_id)` | `dispatch(... tenant_id)` | parameter forwarding | WIRED | `jsonrpc.rs:83` passes `tenant_id` as last argument to `dispatch` |
| `req.get::<serde_json::Value>()` principal sub | `User::find_by_id` | concrete user load for Gate | WIRED | `mcp.rs:67` reads principal; user loaded before Gate check |
| `current_tenant().map(|t| t.id)` | `handle_tools_call(... tenant_id)` | tenant forwarding to dispatch | WIRED | `mcp.rs:155` |
| Gate deny | `isError` tool-error envelope | `make_tool_deny_response` | WIRED | `mcp.rs:143-150`: `Err(_)` branch calls `make_tool_deny_response` |
| `BearerAuthMiddleware` | `TenantMiddleware(JwtClaimResolver)` | middleware ordering on `/mcp` | WIRED | `routes.rs:57-64` |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `dispatch.rs` | SQL result rows | SeaORM raw query against DB using bound params | Yes — parameterized COUNT + SELECT against real DB table; tenant predicate is bound value | FLOWING |
| `mcp.rs` tools/call | `tenant_id` | `ferro::current_tenant().map(|t| t.id)` (set by `TenantMiddleware` from JWT claim) | Yes — reads from resolved tenant context, not a stub | FLOWING |
| `mcp.rs` policy gate | `user` | `crate::models::users::User::find_by_id(user_id)` DB lookup | Yes — actual DB query | FLOWING |

---

### Behavioral Spot-Checks

Step 7b: SKIPPED (requires a running server; the dogfood acceptance run in `200-ACCEPTANCE.md` is the authoritative behavioral verification for SC-4; dispatch and controller unit tests are the authoritative behavioral verification for SC-1 and SC-2).

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| AMCP-10 | 200-01, 200-02, 200-07 | Tool call executes within token's tenant context via existing multi-tenant middleware; scoped to one tenant's rows | SATISFIED | Bound-parameter tenant predicate in `dispatch.rs`; `TenantMiddleware(JwtClaimResolver)` on `/mcp`; isolation tests pass; dogfood GO. |
| AMCP-11 | 200-05, 200-07 | Tool call gated by same policy layer as web surface; policy-denied call returns MCP tool error with no data disclosure | SATISFIED | `Gate::authorize_for` in `mcp.rs`; `mcp_ability=None` fail-closed; `isError:true` envelope; `policy_deny_tool_error_shape` no-disclosure assertions; same Gate registry as web surface. |

Both AMCP-10 and AMCP-11 appear in `REQUIREMENTS.md` traceability table as `Phase 200 | Complete`.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `app/src/controllers/mcp.rs:179` | `use ferro_mcp_oauth::{validate_bearer, ...}` in test block | Info | Inside `#[cfg(test)]` — not compiled in production. Retained for existing `invalid_token_returns_401` test. Not a production stub. |

No blockers or warnings found. The one Info item is intentional test scaffolding.

---

### Human Verification Required

None. All success criteria are verifiable programmatically:
- SC-1 and SC-3: covered by automated unit and integration tests with direct assertions.
- SC-2: covered by shape-assertion tests (`policy_deny_tool_error_shape`, `deny_response_is_jsonrpc_success_not_transport_error`).
- SC-4: `200-ACCEPTANCE.md` records an explicit GO verdict with observed row-level evidence from a live server run in both tenant directions; this satisfies the phase's own acceptance definition.

The optional "Claude Desktop GUI confirmation" noted in `200-ACCEPTANCE.md` is marked as not yet performed but is explicitly classified as optional in that document. The scripted run exercised the identical HTTP OAuth + MCP contract a GUI client uses.

---

### Gaps Summary

No gaps. All four success criteria are verified by code artifacts, tests, and the dogfood acceptance record.

The one design-revision loop (NO-GO → `ee8aed92` fix → GO) is correctly documented in `200-ACCEPTANCE.md` and commit history. The loop was resolved before phase completion, exactly as the phase goal requires.

---

_Verified: 2026-06-11T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
