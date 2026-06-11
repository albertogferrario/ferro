---
phase: 203-oauth-device-authorization-grant-rfc-8628
asvs_level: 1
audited_at: 2026-06-11
threats_total: 19
threats_closed: 19
threats_open: 0
status: SECURED
---

# Phase 203: Security Audit Report

**ASVS Level:** 1
**Threats Closed:** 19/19
**Threats Open:** 0/19
**Result:** SECURED

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-203-USERCODE-BRUTE | Spoofing | mitigate | CLOSED | device.rs:50 — `USER_CODE_CHARSET` 20-char const; device.rs:156-159 — 8-char sample, `format!("{}-{}")` → 20^8 keyspace; single-use via Cache::forget (token.rs:252-253); DEVICE_CODE_TTL 600s (device.rs:43) |
| T-203-DEVICECODE-ENTROPY | Info Disclosure | mitigate | CLOSED | device.rs:140-141 — `generate_device_code()` delegates to `pkce::generate_auth_code()`; pkce.rs:17 — `rand::thread_rng().gen::<[u8;32]>()` → 256 bits; URL_SAFE_NO_PAD encode |
| T-203-USERCODE-NORMALIZE | Tampering | mitigate | CLOSED | device.rs:167-173 — `normalize_user_code` (uppercase + filter `-`/` `); store uses normalized at device.rs:267+294; GET handler normalizes at device.rs:506 before cache lookup |
| T-203-DISCOVERY-HOST | Tampering | mitigate | CLOSED | discovery.rs:27-38 — `authorization_server_metadata(app_url: &str)`; `device_authorization_endpoint` = `format!("{}/device_authorization", app_url)` (line 33); no hardcoded host; handler reads `sanitized_app_url()` (line 55) |
| T-203-INVALID-CLIENT | Spoofing | mitigate | CLOSED | device.rs:251-262 — `find_by_client_id` called; `client.is_none()` → 400 `invalid_client` before codes are generated |
| T-203-CSRF | Tampering | mitigate | CLOSED | device.rs:622-632 — `get_csrf_token()` fetched from session; `form.token.as_bytes().ct_eq(session_csrf.as_bytes())` via `subtle::ConstantTimeEq`; mismatch → 400 |
| T-203-TENANT-BYPASS | EoP | mitigate | CLOSED | device.rs:670 — `tenant_id = current_tenant().map(\|t\| t.id)` (session, not form); `approve_device_grant` called with session-sourced tenant_id (device.rs:672); `current_tenant()` set by TenantMiddleware with SessionUserTenantResolver (routes.rs:101-103) |
| T-203-OPEN-REDIRECT | Spoofing | mitigate | CLOSED | device.rs:479-496 — `user_code` filtered: length==9, hyphen at byte 4, all other bytes in `USER_CODE_CHARSET`; malformed → `"/device"` with no query param; `url_encode` applied to valid codes before use in stored URL |
| T-203-XSS | Tampering | mitigate | CLOSED | device.rs:402-404 — `render_confirm_html` calls `html_escape(client_name)`, `html_escape(device_code)`, `html_escape(csrf_token)`; device.rs:428-429 — `render_terminal_page` escapes both title and message; `html_escape` defined at authorize.rs:196 |
| T-203-DEVICECODE-REPLAY | Tampering | mitigate | CLOSED | token.rs:252-253 — `Cache::forget(&device_cache_key(device_code))` AND `Cache::forget(&usercode_cache_key(&grant.normalized_user_code))` both called BEFORE token mint or any further validation |
| T-203-DEVICECODE-EXPIRY | Info Disclosure | mitigate | CLOSED | token.rs:202 — `if now_unix - grant.created_at > DEVICE_CODE_TTL.as_secs() as i64` → 400 `expired_token`; uses the constant (IN-01 fixed) |
| T-203-CLAIMS-DIVERGE | Spoofing/Confusion | mitigate | CLOSED | token.rs:267-274 — device arm calls `build_claims(grant.user_id.expect(...), grant.tenant_id, &config.app_url, 3600)` + `mint_token(&claims, &config.token_secret)` — identical call site shape as auth-code arm at token.rs:149-151 |
| T-203-MOUNT-TENANT | EoP | mitigate | CLOSED | routes.rs:97-104 — `/device` GET+POST group wrapped in `TenantMiddleware::new().resolver(SessionUserTenantResolver::new()).on_failure(TenantFailureMode::Allow)` |
| T-203-REGRESSION | Tampering | mitigate | CLOSED | 203-REVIEW-FIX.md gate result: `cargo fmt` ✓, `cargo clippy -D warnings` ✓, `cargo test -p ferro-mcp-oauth` ✓ 79 tests passed |
| T-203-CLIENT-BINDING | Spoofing/Token-confusion | mitigate | CLOSED | token.rs:258-259 — `if grant.client_id != form.client_id { return Err(json_error(400, "invalid_grant", "client_id mismatch")) }` in Approved arm, after both Cache::forget calls (forget-first discipline); regression test `device_grant_wrong_client_id_returns_invalid_grant` at token.rs:843 verifies 400 invalid_grant + no access_token + grant consumed |
| T-203-DISCOVERY-DISCLOSURE | Info Disclosure | accept | CLOSED-accepted | RFC 8414 requires public discovery metadata; no secrets in the response body (issuer, endpoints, supported methods only) |
| T-203-DEVICECODE-FORM-LEAK | Info Disclosure | accept | CLOSED-accepted | device_code is already delivered to the polling client over TLS in the POST /device_authorization response; embedding it as a same-origin hidden form field is not a new disclosure channel |
| T-203-DEVICECODE-BRUTE | Info Disclosure | accept | CLOSED-accepted | device_code is 256-bit random (pkce::generate_auth_code); offline brute-force is computationally infeasible; rate-limiting beyond RFC slow_down is documented as a hardening deferral |
| T-203-POLL-RACE | Tampering | accept | CLOSED-accepted | Non-atomic cache get+put in the Pending polling path is benign: a racing poll reads authorization_pending and retries; no security state is corrupted |
| T-203-MOUNT-PUBLIC | Spoofing | accept | CLOSED-accepted | POST /device_authorization is public by RFC 8628 §3.1 design (client devices have no session); client registration abuse mitigation is a documented deferral |

---

## Unregistered Flags

None. All threat flags from the SUMMARY.md files map to registered threat IDs above.

---

## Accepted Risks Log

| Threat ID | Rationale |
|-----------|-----------|
| T-203-DISCOVERY-DISCLOSURE | RFC 8414 public-by-design; metadata contains no secrets |
| T-203-DEVICECODE-FORM-LEAK | device_code already transmitted to polling client over TLS; no new disclosure surface |
| T-203-DEVICECODE-BRUTE | 256-bit entropy; rate-limiting is a hardening deferral, not a gap |
| T-203-POLL-RACE | Racing polls get authorization_pending; no security state corrupted |
| T-203-MOUNT-PUBLIC | RFC §3.1 public endpoint; registration abuse mitigation deferred |

---

_Audited: 2026-06-11_
_Auditor: Claude (gsd-security-auditor)_
_ASVS Level: 1_
