---
phase: 198-streamable-http-endpoint-unauthenticated-challenge
verified: 2026-06-10T00:00:00Z
status: passed
score: 4/4
overrides_applied: 0
---

# Phase 198: Streamable HTTP Endpoint (Unauthenticated Challenge) Verification Report

**Phase Goal:** The application server mounts a Streamable HTTP MCP endpoint. An unauthenticated request to it returns 401 with a WWW-Authenticate header that a standard MCP client can follow to discover the protected-resource metadata. Authenticated calls are not yet wired (Phase 199).
**Verified:** 2026-06-10
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth | Status | Evidence |
| --- | ----- | ------ | -------- |
| 1   | POST /mcp handles initialize, tools/list, and tools/call JSON-RPC methods over Streamable HTTP | VERIFIED | `jsonrpc.rs` implements all three as pure functions; handler in `mcp.rs` dispatches to them; 4 integration tests pass |
| 2   | An unauthenticated POST /mcp returns HTTP 401 with a WWW-Authenticate header referencing the protected-resource metadata URL | VERIFIED | `challenge_response()` formats `Bearer resource_metadata="{app_url}/.well-known/oauth-protected-resource"`; `challenge_response_has_correct_header` test asserts status 401 and exact header value |
| 3   | The endpoint integrates into the application server via the same middleware stack as other framework routes | VERIFIED | `post!("/mcp", ...)` and `get!("/mcp", ...)` registered in `app/src/routes.rs` inside the top-level `routes!` block — same macro stack as all other routes |
| 4   | Integration tests exercise the three JSON-RPC methods and the 401 path without requiring a live OAuth server | VERIFIED | `cargo test -p ferro-mcp-server`: 21 tests pass (4 jsonrpc, 5 dispatch, 12 unit); `cargo test -p app mcp`: 2 tests pass — no HTTP server, no OAuth |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `ferro-mcp-server/src/config.rs` | McpServerConfig with app_name/app_url/version from env | VERIFIED | Reads `APP_NAME`/`APP_URL` from env with `"Ferro"`/`"http://localhost"` fallbacks; `from_env()` alias present; no forbidden literals |
| `ferro-mcp-server/src/auth.rs` | BearerOutcome seam; extract_bearer always Unauthenticated | VERIFIED | `BearerOutcome::Authenticated` present only as `#[allow(dead_code)]` enum variant; `extract_bearer` body discards its argument and always returns `Unauthenticated`; two unit tests assert the invariant |
| `ferro-mcp-server/src/jsonrpc.rs` | handle_initialize / handle_tools_list / handle_tools_call | VERIFIED | All three pure async functions present and exported; `"2025-03-26"` literal; `config.app_name` in serverInfo; `strip_prefix("list_")` in tools/call; `is_filter_field`/`MAX_LIMIT` absent (guards stay in dispatch.rs) |
| `ferro-mcp-server/tests/common/mod.rs` | Shared setup_db() + item_service() fixture | VERIFIED | `pub async fn setup_db` creates in-memory SQLite with 3 rows; `pub fn item_service()` with `.mcp_exposed(true)` |
| `ferro-mcp-server/tests/jsonrpc_integration.rs` | Integration coverage for three JSON-RPC methods | VERIFIED | 4 tests: protocol version, tools/list filter, tools/call rows, unknown-tool -32601 error |
| `app/src/controllers/mcp.rs` | Thin ferro adapter: bearer seam → 401 challenge | VERIFIED | Header read at line 44 (before `req.json()` at line 53); `extract_bearer` call; `challenge_response` formats RFC 9728 header; `method_not_allowed` returns 405 + `Allow: POST` |
| `app/src/routes.rs` | post!("/mcp") + get!("/mcp") registration | VERIFIED | Both routes present in top-level `routes!` block |
| `app/src/projections/order.rs` | order projection marked mcp_exposed(true) | VERIFIED | `.mcp_exposed(true)` at line 12 of `service_def()` builder chain |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `ferro-mcp-server/src/jsonrpc.rs` | `render_exposed_tools` | `handle_tools_list` call | WIRED | `render_exposed_tools(services, &McpContext)` in `handle_tools_list` body |
| `ferro-mcp-server/src/jsonrpc.rs` | `dispatch` | `handle_tools_call` after stripping `list_` prefix | WIRED | `dispatch(service, filters, limit, offset, db).await` at line 82 |
| `ferro-mcp-server/src/jsonrpc.rs` | `config.app_name` | serverInfo construction in `handle_initialize` | WIRED | `"name": config.app_name` in `json!` block |
| `app/src/controllers/mcp.rs` | `ferro_mcp_server::extract_bearer` | bearer seam call before dispatch | WIRED | `extract_bearer(authorization.as_deref())` at line 47, Authorization header pre-read at line 44 |
| `app/src/controllers/mcp.rs` | WWW-Authenticate header | 401 challenge response | WIRED | `challenge_response()` formats and sets `WWW-Authenticate` header; called at line 48 on `Unauthenticated` match arm |
| `app/src/routes.rs` | `controllers::mcp::handle` | `post!("/mcp", ...)` | WIRED | Line 43: `post!("/mcp", controllers::mcp::handle).name("mcp.endpoint")` |

### Data-Flow Trace (Level 4)

Not applicable for Phase 198. The `POST /mcp` handler is always-challenging in this phase (every live request returns 401 before any data is read from the database). The authenticated dispatch path and `exposed_services()` are wired but deliberately unreachable until Phase 199. The dispatch data path is verified at the unit level by `jsonrpc_integration.rs` tests against an in-memory SQLite fixture.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| ferro-mcp-server: all 21 tests pass | `cargo test -p ferro-mcp-server` | 12 unit + 5 dispatch + 4 jsonrpc, 0 failures | PASS |
| app mcp unit tests pass | `cargo test -p app mcp` | 2 tests: `challenge_response_has_correct_header`, `bearer_seam_always_challenges` — ok | PASS |
| `initialize` returns `"2025-03-26"` | `initialize_returns_correct_protocol_version` integration test | `resp["result"]["protocolVersion"] == "2025-03-26"`, `capabilities.tools` is object, `serverInfo.name == "TestApp"` | PASS |
| `tools/call` unknown tool returns -32601 | `tools_call_unknown_tool_is_method_not_found` | `resp["error"]["code"] == -32601` | PASS |
| `extract_bearer` never returns Authenticated | `any_bearer_is_still_unauthenticated_in_phase_198` unit test | matches `Unauthenticated` for any bearer token value | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| AMCP-05 | 198-01, 198-02 | Application serves MCP endpoint over Streamable HTTP supporting initialize, tools/list, tools/call | SATISFIED | `jsonrpc.rs` pure dispatch functions; `mcp.rs` adapter; routes registered; integration tests green |
| AMCP-06 | 198-01, 198-02 | Unauthenticated request returns 401 with WWW-Authenticate referencing protected-resource metadata | SATISFIED | `challenge_response()` formats exact RFC 9728 header; unit test asserts status 401 + exact header value |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `app/src/controllers/mcp.rs` | 16, 24 | `#[allow(dead_code)]` on `exposed_services` and `challenge_response` | Info | Intentional: `#[handler]` macro expansion prevents the dead-code lint from tracing calls through the macro; documented in SUMMARY. These functions ARE called from `handle` body. Not a stub. |
| `app/src/controllers/mcp.rs` | 3 | `// TODO(phase-199): validate Origin header` | Info | Intentional deferred work per plan threat model T-198-05 and T-198-10. Phase 198 ships no production-authenticated traffic — 401 on every request. Tracked. |

No blockers. No stubs affecting goal delivery.

### Human Verification Required

None. All critical invariants are machine-verifiable:

- Security invariant (extract_bearer never returns Authenticated) is verified by two unit tests.
- 401 challenge header format is verified by `challenge_response_has_correct_header` unit test.
- JSON-RPC dispatch correctness is verified by four integration tests with in-memory SQLite.
- Route registration is verified by static source inspection.

Live end-to-end HTTP testing (actual POST to a running server) is explicitly out of scope for Phase 198 — the plan states this phase delivers no production-authenticated traffic path. Phase 199 will exercise the full HTTP surface.

### Gaps Summary

No gaps. All four success criteria are met by the codebase.

---

_Verified: 2026-06-10_
_Verifier: Claude (gsd-verifier)_
