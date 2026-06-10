# Security Audit — Phase 198: Streamable HTTP Endpoint / Unauthenticated Challenge

**Audit date:** 2026-06-10
**Phase:** 198 — streamable-http-endpoint-unauthenticated-challenge (Plans 01 + 02 + CR fixes)
**ASVS Level:** L1
**Auditor:** gsd-security-auditor (claude-sonnet-4-6)
**Result:** SECURED — 11/11 threats closed

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-198-01 | Spoofing (V2 Authn) | mitigate | CLOSED | `ferro-mcp-server/src/auth.rs:20-23` — `extract_bearer` body discards `authorization_header` and unconditionally returns `BearerOutcome::Unauthenticated`. `Authenticated` variant appears only at line 13 (enum definition). Two unit tests at lines 30-43 assert this for both `None` and `Some("Bearer …")` inputs. |
| T-198-02 | Tampering / Info Disclosure (V5) | mitigate | CLOSED | `ferro-mcp-server/src/jsonrpc.rs` contains zero occurrences of `is_filter_field` or `MAX_LIMIT` — allowlist enforcement stays exclusively in `ferro-mcp-server/src/dispatch.rs:130-136` (field iterate + `is_filter_field` check) and `dispatch.rs:113` (`limit.min(MAX_LIMIT)`). No re-implementation or weakening in jsonrpc.rs. |
| T-198-03 | DoS (unbounded result set) | mitigate | CLOSED | `ferro-mcp-server/src/dispatch.rs:10` — `MAX_LIMIT: u64 = 100`; applied at line 113 (`let limit = limit.min(MAX_LIMIT)`). `MAX_OFFSET: u64 = i64::MAX as u64` at line 16, applied at line 114 (`let offset = offset.min(MAX_OFFSET)`). Pagination keys stripped from filters in `jsonrpc.rs:77-80` before handoff to `dispatch`. Both clamps were added per CR fixes (WR-01/WR-02). |
| T-198-04 | Elevation / method probing | mitigate | CLOSED | `ferro-mcp-server/src/jsonrpc.rs:56-63` — service resolution requires `s.name == service_name && s.mcp_exposed`; any other name returns `json!({ "error": { "code": -32601, "message": "Method not found" } })`. Non-exposed services are unreachable by construction. |
| T-198-05 | DNS rebinding (Origin) | accept (deferred) | CLOSED-accepted | `app/src/controllers/mcp.rs:3` — `//! TODO(phase-199): validate Origin header (DNS-rebinding prevention per MCP spec).` marker present. Phase 198 ships no production traffic path beyond the 401 challenge; deferred to Phase 199 per approved planning decision. |
| T-198-06 | Spoofing (V2 Authn) | mitigate | CLOSED | `app/src/controllers/mcp.rs:44` — `req.header("Authorization")` read before body. Line 47-49 — `extract_bearer` called; `BearerOutcome::Unauthenticated` arm returns `Err(challenge_response(&config))` (HTTP 401 + `WWW-Authenticate`). Unit test `bearer_seam_always_challenges` at line 116-126 asserts both `None` and bearer-token inputs produce `Unauthenticated`. |
| T-198-07 | Info Disclosure (401 body / header) | mitigate | CLOSED | `app/src/controllers/mcp.rs:25-33` — `challenge_response` returns `HttpResponse::new().status(401).header("WWW-Authenticate", challenge)` with no body. Metadata URL is the public RFC 9728 path (`/.well-known/oauth-protected-resource`). Unit test `challenge_response_has_correct_header` at line 97-113 asserts exact header value. |
| T-198-08 | Tampering (header/body read order) | mitigate | CLOSED | `app/src/controllers/mcp.rs:44` — `req.header("Authorization")` (borrows `req`) precedes line 53 `req.json().await` (consumes `req`). Comment at line 43 documents the ordering constraint. The `Unauthenticated` return at line 48 exits before `req.json()` is ever reached in Phase 198. |
| T-198-09 | DoS / protocol confusion (GET /mcp) | mitigate | CLOSED | `app/src/routes.rs:44` — explicit `get!("/mcp", controllers::mcp::method_not_allowed)` registered. `app/src/controllers/mcp.rs:88-90` — `method_not_allowed` returns `Err(HttpResponse::new().status(405).header("Allow", "POST"))`. Router verification confirmed in 198-02-SUMMARY.md: `match_route` returns `None` (→ 404) on method mismatch; explicit GET handler required and present. |
| T-198-10 | DNS rebinding (Origin) | accept (deferred) | CLOSED-accepted | `app/src/controllers/mcp.rs:3` — same `TODO(phase-199)` marker as T-198-05. Deferred to Phase 199 per approved planning decision; Phase 198 live path terminates at the 401 challenge. |
| CR-01 | Info Disclosure / Header injection (V5/V14) | mitigate | CLOSED | `ferro-mcp-server/src/config.rs:25-27` — `fn sanitize_identity(raw: String) -> String` filters all ASCII control characters (`.is_ascii_control()` covers CR `\r` and LF `\n`) applied to both `APP_NAME` and `APP_URL` at the `Default::default()` trust boundary (lines 32-37). Tests at lines 56-71 assert CRLF stripped and normal URL preserved. The sanitized `config.app_url` flows into the `WWW-Authenticate` header in `mcp.rs:27`. |

---

## Accepted Risks Log

| Threat ID | Accepted Risk | Condition for Re-evaluation |
|-----------|---------------|----------------------------|
| T-198-05 | DNS rebinding via Origin header on the pure dispatch path (no live traffic in Phase 198 — all requests terminate at the 401 challenge) | Phase 199: add Origin validation before any authenticated traffic flows through `POST /mcp` |
| T-198-10 | DNS rebinding via Origin header on the live `POST /mcp` surface (same condition — Phase 198 always returns 401) | Phase 199: same as T-198-05; both threats are resolved by the same Origin-validation implementation |

---

## Unregistered Threat Flags

None. The 198-02-SUMMARY.md `## Threat Surface Scan` section states: "No new network endpoints, auth paths, file access patterns, or schema changes beyond those in the plan's threat model (T-198-06 through T-198-10)." No unregistered flags were surfaced in either summary.

---

## Scope Notes

- ASVS L1 target. The two deferred DNS-rebinding threats (T-198-05, T-198-10) are acceptable at L1 given the Phase 198 design constraint that all requests terminate at the 401 challenge with no authenticated traffic.
- CR-01 (header injection via `APP_URL`) was identified and fixed during code review before this audit. The fix is in `ferro-mcp-server/src/config.rs:25-27` and is verified closed above.
- Implementation files are read-only from this audit's perspective. No gaps requiring implementation changes were found.
