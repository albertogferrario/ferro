---
phase: 198
slug: streamable-http-endpoint-unauthenticated-challenge
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-10
validated: 2026-06-10
---

# Phase 198 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` + `tokio` (async) |
| **Config file** | `ferro-mcp-server/Cargo.toml` (`[dev-dependencies] tokio = { version = "1", features = ["full", "macros"] }`) |
| **Quick run command** | `cargo test -p ferro-mcp-server` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (full clippy+test on this workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-server`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 198-pure-dispatch | jsonrpc | 1 | AMCP-05 (SC-1a) | — | `initialize` returns `protocolVersion: "2025-03-26"` + `capabilities.tools` | unit | `cargo test -p ferro-mcp-server jsonrpc` | ✅ `tests/jsonrpc_integration.rs::initialize_returns_correct_protocol_version` | ✅ green |
| 198-tools-list | jsonrpc | 1 | AMCP-05 (SC-1b) | — | `tools/list` returns exactly the `mcp_exposed` projections | unit | `cargo test -p ferro-mcp-server jsonrpc` | ✅ `tests/jsonrpc_integration.rs::tools_list_returns_only_exposed` | ✅ green |
| 198-tools-call | jsonrpc | 1 | AMCP-05 (SC-1c) | T-198 input | `tools/call` returns dispatch rows; strips `list_` prefix; unknown tool → `-32601`; unknown filter → `-32602` (WR-02); allowlist + limit clamp retained (197 WR-01/WR-02) | integration (in-memory DB) | `cargo test -p ferro-mcp-server` | ✅ `jsonrpc_integration.rs::{tools_call_returns_rows, tools_call_unknown_tool_is_method_not_found, tools_call_unknown_filter_is_invalid_params}` + `dispatch_integration.rs::{dispatch_non_filterable_field_rejected, dispatch_limit_pagination_returns_subset_with_full_total, dispatch_unknown_filter_key_returns_err}` | ✅ green |
| 198-bearer-seam | auth | 1 | AMCP-06 (SC-2) | T-198 authn | `extract_bearer` has NO path returning `Authenticated` in Phase 198 | unit | `cargo test -p ferro-mcp-server auth` | ✅ `src/auth.rs::tests` (None + bearer-token, both `Unauthenticated`) | ✅ green |
| 198-401-challenge | handler | 2 | AMCP-06 (SC-2) | T-198 authn | unauthenticated → `401` + `WWW-Authenticate: Bearer resource_metadata="{APP_URL}/.well-known/oauth-protected-resource"` | unit | `cargo test -p app mcp` | ✅ `app/src/controllers/mcp.rs::tests::challenge_response_has_correct_header` (+ `bearer_seam_always_challenges`) | ✅ green |
| 198-route-mount | handler | 2 | AMCP-05 (SC-3) | — | `post!("/mcp")` registered in app router via same middleware stack | compile-time | `cargo build -p app` | ✅ `app/src/routes.rs` (`post!("/mcp")` + `get!("/mcp")`) compiles | ✅ green |
| 198-no-oauth | jsonrpc | 1 | AMCP-06 (SC-4) | — | all four paths exercised without a live server/OAuth | integration (no server) | `cargo test -p ferro-mcp-server` | ✅ `tests/jsonrpc_integration.rs` (5 tests, no HTTP server, no OAuth) | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `ferro-mcp-server/tests/common/mod.rs` — `setup_db()` in-memory SQLite fixture + `item_service()` for cross-file reuse (Open Question 2)
- [x] `ferro-mcp-server/tests/jsonrpc_integration.rs` — covers AMCP-05 SC-1a/b/c + SC-4 (5 tests)
- [x] `ferro-mcp-server/src/auth.rs` — `extract_bearer` unit tests inline (SC-2 seam-always-unauthenticated)
- [x] `tokio` dev-dependency present in `ferro-mcp-server/Cargo.toml` with `full` + `macros` features

*Existing dispatch_integration.rs covers the read path (197); new files cover the JSON-RPC + auth surface.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real MCP client follows the `WWW-Authenticate` challenge | AMCP-06 | Requires a live server + an MCP client; out of Phase 198 scope (deferred to Phase 199/200 dogfood) | Deferred — Phase 200 dogfood gate exercises this end-to-end |

*Phase 198's automated tests fully cover SC-1..SC-4 without a live server.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (`tests/common/mod.rs`, `jsonrpc_integration.rs`)
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated 2026-06-10

---

## Validation Audit 2026-06-10

| Metric | Count |
|--------|-------|
| Requirements audited | 7 |
| COVERED | 7 |
| PARTIAL | 0 |
| MISSING | 0 |
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Manual-only (deferred to Phase 200 dogfood) | 1 |

**Result:** Nyquist-compliant. All SC-1..SC-4 acceptance criteria have green automated tests; no auditor spawn required.

**Evidence (re-run 2026-06-10):**
- `cargo test -p ferro-mcp-server` → 24 tests pass (14 unit incl. auth seam, 5 dispatch, 5 jsonrpc)
- `cargo test -p app mcp` → 2 tests pass

**Note:** Implemented coverage exceeds the draft plan — the code-review fix added `tools_call_unknown_filter_is_invalid_params` (WR-02, `InvalidFilter → -32602`), bringing `jsonrpc_integration.rs` to 5 tests. SC-3 (route mount) is verified at compile time only; no runtime route-registration assertion exists, which is appropriate for a static `routes!` registration.
