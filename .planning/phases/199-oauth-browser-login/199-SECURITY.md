---
phase: 199
slug: oauth-browser-login
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-10
---

# Phase 199 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.
> OAuth 2.0 authorization server for the MCP endpoint — the threat surface is the deliverable.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| env → process | `MCP_TOKEN_SECRET` crosses from operator environment into the HMAC signing key | secret (256-bit HMAC key) |
| MCP client → `/register` | untrusted DCR JSON (redirect_uris, client_name) crosses into DB persistence | untrusted client metadata |
| MCP client → `/.well-known/*` | public unauthenticated reads | public endpoint URLs (no secret) |
| browser → `GET /authorize` | untrusted query params (client_id, redirect_uri, code_challenge) | untrusted authorization request |
| browser → `POST /authorize` | consent form submission | CSRF-protected approve/deny |
| `/authorize` → `ferro::Cache` | server-minted single-use code crosses into the TTL store | short-lived auth code + PKCE challenge |
| MCP client → `/token` | code + code_verifier exchange | replay + PKCE surface |
| MCP client → `POST /mcp` | bearer JWT + Origin header cross into validation before dispatch | audience/tenant-bound access token |
| browser → `POST /auth/login` | login whose post-auth redirect resumes the OAuth flow | session-bound return-to path |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-199-13 | Spoofing | config.rs `from_env` | mitigate (HIGH) | `Err(MissingSecret)` when `MCP_TOKEN_SECRET` unset; no silent fallback (config.rs:64) | closed |
| T-199-14 | Spoofing | config.rs `from_env` | mitigate (HIGH) | reject secret < 32 bytes → `Err(SecretTooShort)` (config.rs:65-67) | closed |
| T-199-IDENT | Information disclosure | config.rs identity | mitigate (LOW) | `sanitize_identity` strips ASCII control/CRLF from APP_NAME/APP_URL (config.rs:12-14, 22, 58-61) | closed |
| T-199-CYCLE | Tampering | crate dependency graph | mitigate (correctness) | `ferro-mcp-oauth → framework` only; no `ferro-mcp-server` dep; publish.yml Wave 2 (Cargo.toml, publish.yml:274) | closed |
| T-199-05 | Tampering | register.rs redirect_uri scheme | mitigate (HIGH) | scheme allowlist: only `https://` or `http://localhost`; else 400 (register.rs:39-42, 80-86) | closed |
| T-199-DCR | Spoofing | register.rs client_id | mitigate (LOW) | 16-byte CSPRNG URL-safe-base64 client_id, non-sequential (register.rs:66-70) | closed |
| T-199-04a | Tampering | register.rs / store.rs | mitigate (HIGH) | redirect_uris stored verbatim for later exact-match (store.rs:45, register.rs:88-89) | closed |
| T-199-DISC | Information disclosure | discovery.rs | accept (LOW) | discovery is public by RFC; exposes only APP_URL-derived endpoints, never reads the secret (discovery.rs:8) | closed |
| T-199-06 | Spoofing | jwt.rs `decode_token` | mitigate (HIGH) | `validation.algorithms = vec![Algorithm::HS256]` — rejects `alg=none`/client alg (jwt.rs:89) | closed |
| T-199-07 | Spoofing | jwt.rs `decode_token` | mitigate (HIGH) | same HS256 pin blocks RS256→HS256 key confusion (jwt.rs:89) | closed |
| T-199-08 | Elevation | jwt.rs + validate.rs + mcp.rs | mitigate (HIGH) | `set_audience` exact aud; `InvalidAudience → Forbidden(403)` (jwt.rs:91, validate.rs:75, mcp.rs:62) | closed |
| T-199-09 | Elevation | validate.rs + mcp.rs tenant | mitigate (HIGH) | `claims.tenant_id` vs `expected_tenant`; mismatch/missing-when-expected → Forbidden(403); claim key exactly `tenant_id` matching JwtClaimResolver (validate.rs:83-91, jwt.rs:31, resolver.rs:211, mcp.rs:52-53) | closed |
| T-199-11 | Information disclosure | pkce.rs `verify_s256` | mitigate (MEDIUM) | `subtle::ConstantTimeEq` for S256 compare — no timing oracle (pkce.rs:10, 33) | closed |
| T-199-17 | Spoofing | jwt.rs `build_claims` | mitigate (HIGH) | `iss` and `aud` both from one config; no third-party AS delegation (jwt.rs:56-57) | closed |
| T-199-01 | Spoofing | authorize.rs + consent.rs | mitigate (HIGH) | reject absent `code_challenge` / `method != "S256"` at GET and POST /authorize → 400 error page (authorize.rs:65-85, consent.rs:158-165) | closed |
| T-199-02 | Repudiation | token.rs | mitigate (HIGH) | `Cache::forget` immediately after `Cache::get`, BEFORE any validation; single-use even on failure (token.rs:59-64) | closed |
| T-199-03 | Elevation | consent.rs `Cache::put` | mitigate (MEDIUM) | 60s TTL on auth code (`Some(Duration::from_secs(60))`) (consent.rs:219-224) | closed |
| T-199-04 | Tampering | authorize.rs + token.rs redirect_uri | mitigate (HIGH) | exact-string redirect_uri match at /authorize (→ error page, never redirect) AND /token (authorize.rs:126-134, token.rs:78-84) | closed |
| T-199-10 | Spoofing | consent.rs `authorize_post` | mitigate (MEDIUM) | CSRF `_token` vs session `get_csrf_token()` via constant-time `ct_eq` before approve/deny (consent.rs:125-141) | closed |
| T-199-12 | Information disclosure | consent.rs CSRF compare | mitigate (MEDIUM) | `subtle::ConstantTimeEq` for CSRF compare (consent.rs:134) | closed |
| T-199-16 | Tampering | token.rs | mitigate (HIGH) | code record carries client_id + redirect_uri, both re-validated at exchange (anti code-substitution) (token.rs:75-84, store.rs:17-26) | closed |
| T-199-XSS | Tampering | consent.rs render | mitigate (MEDIUM) | HTML-escape attacker-controlled `client_name` before embedding; test confirms `<script>` neutralized (consent.rs:70, authorize.rs:198-209, consent.rs:301-319) | closed |
| T-199-401 | Spoofing | mcp.rs bearer mapping | mitigate (HIGH) | invalid/expired/bad-sig → `Invalid` → 401 `WWW-Authenticate: Bearer error="invalid_token"` (RFC 6750) (mcp.rs:58-61) | closed |
| T-199-15 | Spoofing | mcp.rs Origin check | mitigate (MEDIUM) | present Origin not matching APP_URL → 403 (DNS-rebinding); absent Origin allowed for SDK clients (mcp.rs:39-43) | closed |
| T-199-13b | Spoofing | mcp.rs `from_env` at seam | mitigate (HIGH) | `MCP_TOKEN_SECRET` unset → 401 challenge (fail-closed; never accept unvalidated) (mcp.rs:49) | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-199-01 | T-199-DISC | OAuth discovery documents (`.well-known/*`) are public by RFC 8414/9728 design. They expose only endpoint URLs derived from `APP_URL`; the handlers never read `MCP_TOKEN_SECRET`, so no secret-presence is leaked. | Alberto | 2026-06-10 |
| AR-199-02 | T-199-RETURNTO | `oauth_return_to` post-login redirect (auth_controller.rs:139) replays a value the `/authorize` handler stored. Today that value is always a server-constructed **relative** `/authorize?{query}` path (same-origin), so it is not an open-redirect vector. Hardening note carried forward: add an explicit relative-path / same-origin guard before redirecting if any future code path can store an absolute URL into `oauth_return_to`. Not a blocker at ASVS L1. | Alberto | 2026-06-10 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-10 | 25 | 25 | 0 | gsd-security-auditor (ASVS L1) |

*25 unique threat IDs across 5 plans (T-199-08 and T-199-09 appear in both the crypto-core and seam-wiring plans; counted once). All verified against implementation source with file:line evidence.*

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-10
