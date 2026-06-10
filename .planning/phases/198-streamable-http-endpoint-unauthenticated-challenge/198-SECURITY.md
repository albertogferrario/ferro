---
phase: 198
slug: streamable-http-endpoint-unauthenticated-challenge
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-10
---

# Phase 198 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| HTTP client → `POST /mcp` | Untrusted request body + `Authorization` header cross the framework middleware chain into the handler | JSON-RPC envelope, bearer credential |
| HTTP client → `GET /mcp` | Untrusted GET probe — must not leak an SSE surface or `200` | none |
| MCP client → `tools/call` arguments | Untrusted JSON `arguments` (filter keys, `limit`/`offset`) cross into the read path | filter values, pagination ints |
| Environment → header values | `APP_URL`/`APP_NAME` env values flow into the `WWW-Authenticate` header | operator-sourced strings |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-198-01 | Spoofing (V2 Authn) | `ferro-mcp-server/src/auth.rs::extract_bearer` | mitigate | Seam unconditionally returns `Unauthenticated` (auth.rs:20-23); `BearerOutcome::Authenticated` is never constructed (enum def only, line 13). Two unit tests assert `None` and bearer-token inputs both yield `Unauthenticated`. | closed |
| T-198-02 | Tampering / Info Disclosure (V5) | `jsonrpc::handle_tools_call` → `dispatch` | mitigate | Filter keys validated by `is_filter_field` allowlist inside `dispatch.rs`; `jsonrpc.rs` contains zero `is_filter_field`/`MAX_LIMIT` occurrences — guards are not re-implemented or weakened. | closed |
| T-198-03 | DoS (unbounded result set) | `tools/call` read path | mitigate | `dispatch.rs` clamps `limit` to `MAX_LIMIT=100` (line 113) and `offset` to `MAX_OFFSET=i64::MAX as u64` (line 114); pagination keys stripped from filters (jsonrpc.rs:77-80). | closed |
| T-198-04 | Elevation / method probing | `jsonrpc::handle_tools_call` resolution | mitigate | Service resolution requires `s.mcp_exposed`; unknown/non-exposed tool name → JSON-RPC `-32601` (jsonrpc.rs:56-63). | closed |
| T-198-05 | DNS rebinding (Origin header) | live HTTP surface | accept (deferred) | Phase 198 terminates every request at the `401` challenge — no live traffic path. Origin validation deferred to Phase 199; `// TODO(phase-199): validate Origin header` present (mcp.rs:3). | closed (accepted) |
| T-198-06 | Spoofing (V2 Authn) | `app/src/controllers/mcp.rs::handle` | mitigate | Handler reads `Authorization` header (line 44) then calls `extract_bearer` (always `Unauthenticated`) → returns `401 + WWW-Authenticate` (line 48) before body parse. Unit test `bearer_seam_always_challenges`. | closed |
| T-198-07 | Info Disclosure (401 body) | `mcp::challenge_response` | mitigate | `401` response carries no body, only the `WWW-Authenticate` challenge with the public RFC 9728 metadata path (no secret). Unit test asserts exact header value. | closed |
| T-198-08 | Tampering (partial read / smuggling) | header vs body read order | mitigate | `req.header("Authorization")` (mcp.rs:44) precedes `req.json()` (mcp.rs:53); the `Unauthenticated` return at line 48 exits before the body is read in Phase 198 (Ferro single-read guarantee). | closed |
| T-198-09 | DoS / protocol confusion (GET) | `GET /mcp` | mitigate | Explicit `get!("/mcp", method_not_allowed)` registered (routes.rs:44) returning `405 + Allow: POST` (mcp.rs:88-90); no SSE offered. Ferro router's 404-on-method-mismatch verified, not assumed. | closed |
| T-198-10 | DNS rebinding (Origin header) | live `POST /mcp` surface | accept (deferred) | Same deferral as T-198-05; `// TODO(phase-199)` marker present. | closed (accepted) |
| CR-01 | Info Disclosure / Header injection (V5/V14) | `mcp::challenge_response` ← `config.app_url` | mitigate | `sanitize_identity` (config.rs:25-27) strips all ASCII control chars (incl. CR/LF) from `APP_NAME`/`APP_URL` at the `from_env`/`Default` trust boundary (lines 32-37), before the value flows into the `WWW-Authenticate` header. Two tests assert CRLF stripped and a clean URL preserved. Found in code review, fixed in commit `4e08948`. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-198-01 | T-198-05 / T-198-10 | Origin-header (DNS-rebinding) validation is deferred to Phase 199. Phase 198 ships no live production traffic path — every request terminates at the `401` challenge before reaching any read or stream. The deferral was an explicit, approved planning decision in both 198-01 and 198-02 PLAN threat models, with `// TODO(phase-199)` markers in the handler. | Planning (PLAN threat models, approved) | 2026-06-10 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-10 | 11 | 11 | 0 | gsd-security-auditor (ASVS L1) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-10
