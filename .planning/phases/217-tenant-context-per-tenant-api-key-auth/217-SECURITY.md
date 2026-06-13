---
phase: 217-tenant-context-per-tenant-api-key-auth
audited: 2026-06-13
asvs_level: 1
block_on: high
auditor: gsd-security-auditor
status: secured
threats_total: 5
threats_closed: 5
threats_open: 0
residual_risks: 2
---

# Phase 217 Security Audit

**ASVS Level:** 1
**Block on:** high
**Result:** All five declared threats CLOSED at the framework layer. Two residual risks documented (not blockers under this phase's scope).

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-217-01 | Information Disclosure (cross-tenant) | mitigate | CLOSED | `dispatch.rs:153-166` — fail-closed tenant predicate injection; `tenant_id` NEVER read from call payload, always from `handle_tools_call`'s `tenant_id` parameter (which callers derive from the auth principal, not from `call_params`). `jsonrpc.rs:61` reads only `call_params["name"]`; `jsonrpc.rs:106` passes `tenant_id` (function parameter) not any `call_params` field to `dispatch`. `mcp_tenant_isolation.rs:239-285` — `api_key_cross_tenant_isolation` test asserts `rows.len()==2 AND per-row tenant_id==Some(1) AND assert_ne!(row_tid, Some(2))`. All 4 integration tests pass (217-02-SUMMARY.md). |
| T-217-02 | Information Disclosure (at rest) | mitigate | CLOSED | `migration.rs:112` — `McpApiKeys::KeyHash` column defined as `string().not_null()` with no plaintext column in the schema. `validate.rs:157-161` — `validate_api_key` calls `hash_mcp_api_key(token)` before the SQL lookup; the raw token is never persisted. `validate.rs:115-126` — `generate_mcp_api_key()` returns `(raw_key, key_hash)` and does not persist either value (pure Rust function). Plaintext grep of `migration.rs` returns no plaintext/raw_key column. |
| T-217-03 | Spoofing | mitigate | CLOSED | `validate.rs:163-167` — `Ok(None) => return BearerCheck::Invalid` (unknown key); `Err(_) => return BearerCheck::Invalid` (DB error). `validate.rs:170-173` — revoked key (non-null `revoked_at`) returns `BearerCheck::Invalid`. `validate.rs:152-154` — non-`ferro_`-prefixed token returns `Unauthenticated`. `auth.rs:21-29` — `resolve_tenant` branches on `ferro_` prefix; non-matching tokens go to `validate_bearer`; empty header returns `Unauthenticated`. `jsonrpc.rs:58-59` — `tenant_id: Option<i64>` is a function parameter populated by the caller from the resolved principal, never from the tool-call payload (grep of `call_params` in `jsonrpc.rs` shows only `call_params["name"]` and `call_params.get("arguments")` — no tenant extraction). |
| T-217-04 | Elevation (scope creep) | mitigate | CLOSED (framework layer); RESIDUAL RISK (consumer wiring) | `jsonrpc.rs:64-78` — scope gate fires BEFORE service lookup: `is_write_tool = !tool_name.starts_with("list_")`, `key_scope = ctx.scope.as_deref().unwrap_or("read_write")`, returns `-32603` with "scope insufficient" message for `is_write_tool && key_scope == "read"`. `migration.rs:113-117` — `scope` column on `mcp_api_keys` from the first migration. `validate.rs:197-201` — `scope` included in `BearerCheck::Authenticated` principal. `mcp_tenant_isolation.rs:167-194` — `read_scope_key_rejected_on_write_tool_name` test passes. **RESIDUAL RISK (see below):** `app/src/controllers/mcp.rs:158` passes `&McpContext::default()` (scope: None → maps to "read_write"), so the scope gate cannot fire end-to-end through the sample app live path. Classified as consumer-wiring residual, not a phase framework gap. |
| T-217-05 | Denial of Service (release) | mitigate | CLOSED | `.github/workflows/publish.yml:275` — `WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-oauth ferro-mcp-server"` — `ferro-mcp-oauth` precedes `ferro-mcp-server` in the wave loop. `ferro-mcp-oauth/Cargo.toml` does not reference `ferro-mcp-server` (no cycle, confirmed by 217-03-SUMMARY.md self-check). |

---

## Unregistered Threat Flags

The following threat flags appeared in SUMMARY.md files across Plans 00-03 and are assessed here:

- **Plan 00 SUMMARY:** No new threat surface declared — skeleton fails closed, no real auth path active. Maps to T-217-03 (disposition: accept in Plan 00 wave, mitigate in Plans 01/02). Informational only.
- **Plan 01 SUMMARY:** No new threat surface. Changes confined to `ferro-mcp-oauth` library crate with no HTTP surface. Informational only.
- **Plan 02 SUMMARY:** No new threat surface. Test-only file changes. Informational only.
- **Plan 03 SUMMARY:** No new threat surface. Publish-wave ordering and docs only. Informational only.

No unregistered flags requiring escalation.

---

## Residual Risks (Non-blocking, ASVS L1, this phase scope)

### RR-01: API key auth path dead in sample-app live HTTP path (CR-01)

**Finding (from 217-REVIEW.md CR-01):** `app/src/middleware/bearer_auth.rs:37` calls `validate_bearer` (JWT-only). A `ferro_`-prefixed API key hits `decode_token`, fails JWT parse, and returns `BearerCheck::Invalid` → 401. The `resolve_tenant` unifier in `ferro-mcp-server/src/auth.rs` is exported but never invoked in the live HTTP path.

**Framework layer:** The mitigation functions exist and are unit/integration tested (`resolve_tenant`, `validate_api_key`, all `mcp_tenant_isolation` tests pass).

**ASVS L1 assessment:** Under this phase's declared scope ("framework capability + synthetic validation only"), the framework layer mitigations are complete and tested. The consumer-app wiring gap is a follow-up item for the consumer-adoption pass. This gap does NOT rise to a blocking open threat for ASVS L1 at the framework layer.

**Action required before live API-key auth is usable:** `BearerAuthMiddleware` must be updated to call `resolve_tenant` instead of `validate_bearer` directly (fix described in 217-REVIEW.md CR-01).

---

### RR-02: McpContext::scope never populated from principal in sample-app controller (CR-02)

**Finding (from 217-REVIEW.md CR-02):** `app/src/controllers/mcp.rs:158` calls `handle_tools_call(..., &McpContext::default())`. `McpContext::default()` sets `scope: None`, which maps to `"read_write"` in `jsonrpc.rs:68`. A `read`-scoped API key can call any write tool through the sample app.

**Framework layer:** The scope gate logic in `jsonrpc.rs:64-78` is correct and fires when `ctx.scope == Some("read")`. The gate is exercised by `read_scope_key_rejected_on_write_tool_name` (passes). The gap is that the sample-app controller never passes a non-default `McpContext`.

**ASVS L1 assessment:** The framework-layer mitigation exists and is tested. The end-to-end bypass exists only in the sample app (which is also blocked by RR-01 before reaching this code path). Not a blocking gap for the framework layer; is a real functional bypass in the sample-app consumer path.

**Action required before scope enforcement is live:** After fixing RR-01, the controller must populate `McpContext::scope` from `principal["scope"]` (fix described in 217-REVIEW.md CR-02).

---

## Assessment

All five declared threats have their mitigations present in the framework implementation files and verified by passing tests. The framework security surface is complete for ASVS L1.

Two residual risks (RR-01, RR-02) document that the sample-app consumer wiring does not yet connect the framework-layer mitigations to the live HTTP request path. These are consumer-adoption follow-ups, not framework gaps. They are high severity if the sample app is deployed as-is with real API keys — but under the declared v15.0 phase scope (framework capability + synthetic validation), they do not block phase completion.

Recommended follow-up phase: implement RR-01 + RR-02 fixes in the sample-app consumer before any deployment that issues real `ferro_`-prefixed API keys.
