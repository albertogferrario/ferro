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
residual_risks: 0
---

# Phase 217 Security Audit

**ASVS Level:** 1
**Block on:** high
**Result:** All five declared threats CLOSED. The two residual risks (RR-01/RR-02) from the initial audit were resolved by the `/gsd-code-review-fix` pass (commits `5495e812`, `f99c2b9e`) — the sample-app live HTTP path now wires `resolve_tenant` and populates `McpContext.scope` from the authenticated principal, so the API-key auth and scope gate are enforced end-to-end.

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-217-01 | Information Disclosure (cross-tenant) | mitigate | CLOSED | `dispatch.rs:153-166` — fail-closed tenant predicate injection; `tenant_id` NEVER read from call payload, always from `handle_tools_call`'s `tenant_id` parameter (which callers derive from the auth principal, not from `call_params`). `jsonrpc.rs:61` reads only `call_params["name"]`; `jsonrpc.rs:106` passes `tenant_id` (function parameter) not any `call_params` field to `dispatch`. `mcp_tenant_isolation.rs:239-285` — `api_key_cross_tenant_isolation` test asserts `rows.len()==2 AND per-row tenant_id==Some(1) AND assert_ne!(row_tid, Some(2))`. All 4 integration tests pass (217-02-SUMMARY.md). |
| T-217-02 | Information Disclosure (at rest) | mitigate | CLOSED | `migration.rs:112` — `McpApiKeys::KeyHash` column defined as `string().not_null()` with no plaintext column in the schema. `validate.rs:157-161` — `validate_api_key` calls `hash_mcp_api_key(token)` before the SQL lookup; the raw token is never persisted. `validate.rs:115-126` — `generate_mcp_api_key()` returns `(raw_key, key_hash)` and does not persist either value (pure Rust function). Plaintext grep of `migration.rs` returns no plaintext/raw_key column. |
| T-217-03 | Spoofing | mitigate | CLOSED | `validate.rs:163-167` — `Ok(None) => return BearerCheck::Invalid` (unknown key); `Err(_) => return BearerCheck::Invalid` (DB error). `validate.rs:170-173` — revoked key (non-null `revoked_at`) returns `BearerCheck::Invalid`. `validate.rs:152-154` — non-`ferro_`-prefixed token returns `Unauthenticated`. `auth.rs:21-29` — `resolve_tenant` branches on `ferro_` prefix; non-matching tokens go to `validate_bearer`; empty header returns `Unauthenticated`. `jsonrpc.rs:58-59` — `tenant_id: Option<i64>` is a function parameter populated by the caller from the resolved principal, never from the tool-call payload (grep of `call_params` in `jsonrpc.rs` shows only `call_params["name"]` and `call_params.get("arguments")` — no tenant extraction). |
| T-217-04 | Elevation (scope creep) | mitigate | CLOSED (framework + live path) | `jsonrpc.rs:64-78` — scope gate fires BEFORE service lookup: `is_write_tool = !tool_name.starts_with("list_")`, `key_scope = ctx.scope.as_deref().unwrap_or("read_write")`, returns `-32603` with "scope insufficient" message for `is_write_tool && key_scope == "read"`. `migration.rs:113-117` — `scope` column on `mcp_api_keys` from the first migration. `validate.rs:197-201` — `scope` included in `BearerCheck::Authenticated` principal. `mcp_tenant_isolation.rs:167-194` — `read_scope_key_rejected_on_write_tool_name` test passes. **End-to-end (resolved post-audit, commit `f99c2b9e`):** `app/src/controllers/mcp.rs` now extracts `scope` from the authenticated principal and builds a real `McpContext { tenant_id, scope, .. }` for both `tools/list` and `tools/call`, so the gate fires through the sample-app live path. |
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

## Residual Risks (RESOLVED post-audit by `/gsd-code-review-fix 217`)

> Both residual risks below were closed after the audit by the code-review-fix pass (commits `5495e812` CR-01, `f99c2b9e` CR-02). Verification: build clean, 127 tests green, clippy `-D warnings` clean on the touched crates. They are retained here for the audit trail.

### RR-01 (RESOLVED): API key auth path now wired into the sample-app live HTTP path (CR-01)

**Finding (from 217-REVIEW.md CR-01):** `app/src/middleware/bearer_auth.rs:37` calls `validate_bearer` (JWT-only). A `ferro_`-prefixed API key hits `decode_token`, fails JWT parse, and returns `BearerCheck::Invalid` → 401. The `resolve_tenant` unifier in `ferro-mcp-server/src/auth.rs` is exported but never invoked in the live HTTP path.

**Framework layer:** The mitigation functions exist and are unit/integration tested (`resolve_tenant`, `validate_api_key`, all `mcp_tenant_isolation` tests pass).

**ASVS L1 assessment:** Under this phase's declared scope ("framework capability + synthetic validation only"), the framework layer mitigations are complete and tested. The consumer-app wiring gap is a follow-up item for the consumer-adoption pass. This gap does NOT rise to a blocking open threat for ASVS L1 at the framework layer.

**Resolution (commit `5495e812`):** `BearerAuthMiddleware` now calls `ferro_mcp_server::resolve_tenant(auth_header, &db, &oauth_config)` (DB connection obtained via `ferro::DB::connection()`, fail-closed), so `ferro_`-prefixed API keys reach `validate_api_key` and resolve a tenant in the live HTTP path.

---

### RR-02 (RESOLVED): McpContext::scope now populated from principal in sample-app controller (CR-02)

**Finding (from 217-REVIEW.md CR-02):** `app/src/controllers/mcp.rs:158` calls `handle_tools_call(..., &McpContext::default())`. `McpContext::default()` sets `scope: None`, which maps to `"read_write"` in `jsonrpc.rs:68`. A `read`-scoped API key can call any write tool through the sample app.

**Framework layer:** The scope gate logic in `jsonrpc.rs:64-78` is correct and fires when `ctx.scope == Some("read")`. The gate is exercised by `read_scope_key_rejected_on_write_tool_name` (passes). The gap is that the sample-app controller never passes a non-default `McpContext`.

**ASVS L1 assessment:** The framework-layer mitigation exists and is tested. The end-to-end bypass exists only in the sample app (which is also blocked by RR-01 before reaching this code path). Not a blocking gap for the framework layer; is a real functional bypass in the sample-app consumer path.

**Resolution (commit `f99c2b9e`):** The controller now extracts `scope` from the authenticated principal and builds a real `McpContext { tenant_id, scope, .. }` for both `tools/list` and `tools/call`; a JWT principal (no `scope` field) leaves `scope: None` → `"read_write"`, preserving the OAuth path. The scope gate now fires end-to-end through the sample-app live path.

---

## Assessment

All five declared threats have their mitigations present in the implementation files and verified by passing tests. The framework security surface is complete for ASVS L1.

The two residual risks (RR-01, RR-02) from the initial audit — the sample-app consumer wiring not connecting the framework-layer mitigations to the live HTTP request path — were resolved post-audit by the `/gsd-code-review-fix 217` pass (commits `5495e812`, `f99c2b9e`). The live HTTP path now resolves `ferro_` API keys and enforces the scope gate end-to-end; build clean, 127 tests green, clippy `-D warnings` clean. No open residual risks remain for this phase.
