---
phase: 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p
verified: 2026-06-12T12:00:00Z
status: passed
score: 8/8
overrides_applied: 0
re_verification: null
gaps: []
deferred: []
human_verification: []
---

# Phase 205: Fix ferro-mcp-server tools/call result content blocks — Verification Report

**Phase Goal:** The ferro-mcp-server `tools/call` success result is a valid MCP `CallToolResult` (one `type:text` content block + `structuredContent` carrying `{rows,total,limit,offset}`), so a strict MCP client parses it without Zod errors; a client-schema interop regression test deserializes the emitted result with the client's own rmcp type; the live :8090 browser-OAuth dogfood (alice@acme.test → list_order) re-runs to GO with tenant scoping intact.
**Verified:** 2026-06-12T12:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | tools/call Ok arm emits valid `CallToolResult` via `CallToolResult::structured(payload)` | VERIFIED | `jsonrpc.rs:96` — `let tool_result = CallToolResult::structured(payload);` present; `json!({ "result": tool_result })` at line 97 |
| 2 | `content[0]` is a `type:text` content block (not a bare projection row) | VERIFIED | D-04 test at `jsonrpc.rs:188-193` asserts `content_json[0]["type"] == Some("text")`; live dogfood observed `content[0].type == "text"` |
| 3 | `rows/total/limit/offset` live under `structuredContent`, not as top-level result keys | VERIFIED | `jsonrpc.rs:90-95` builds payload with all four keys then passes to `structured()`; D-04 test asserts `sc.get("rows")`, `sc.get("total")`, `sc.get("limit")`, `sc.get("offset")` are all `Some` |
| 4 | `isError` is `false` on a successful dispatch | VERIFIED | `jsonrpc.rs:181` — `assert_eq!(parsed.is_error, Some(false))` in D-04 test; live dogfood observed `result.isError == false` |
| 5 | D-04 interop regression test parses the emitted result with `rmcp::model::CallToolResult` custom Deserialize | VERIFIED | `jsonrpc.rs:167-208` — `tools_call_result_parses_as_valid_mcp_content` calls `serde_json::from_value(response["result"].clone())` into `CallToolResult`; commit `425b75a4` |
| 6 | Tenant isolation tests navigate `structuredContent.rows` and assert `content[0].type==text` | VERIFIED | `mcp_tenant_isolation.rs:257-268` (tenant_a) and `317-328` (tenant_b) — both use `result["result"]["structuredContent"]["rows"]` and assert `content[0]["type"] == Some("text")` |
| 7 | Error arms (-32601/-32602/-32603) preserved byte-for-byte | VERIFIED | `jsonrpc.rs:64` (-32601), `jsonrpc.rs:103` (-32602), `jsonrpc.rs:105` (-32603) — all present and unchanged |
| 8 | Live :8090 browser-OAuth dogfood (alice@acme.test → list_order) returns GO with 2 Acme orders, no Zod errors | VERIFIED | `205-ACCEPTANCE.md` records GO verdict; observed `content[0].type=="text"`, `structuredContent.total:2`, `isError:false`, exactly 2 rows both `tenant_id:1`; Globex rows correctly excluded |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp-server/src/jsonrpc.rs` | Fixed Ok arm + inline D-04 interop regression test | VERIFIED | Contains `CallToolResult::structured(payload)` at line 96; `#[cfg(test)] mod tests` with `tools_call_result_parses_as_valid_mcp_content` at lines 109-209 |
| `app/src/tests/mcp_tenant_isolation.rs` | tenant_a/tenant_b re-pointed to `structuredContent.rows` + `content[0].type==text` assertion | VERIFIED | Both tests updated; `structuredContent` appears 6 times; `content[0]["type"]` assertions at lines 261 and 322 |
| `.planning/phases/205-.../205-ACCEPTANCE.md` | GO/NO-GO verdict with observed row count and parse result | VERIFIED | GO verdict recorded; envelope details tabulated; 2 Acme orders observed; method: live browser-OAuth on :8090 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `handle_tools_call` Ok arm | `rmcp::model::CallToolResult::structured` | `payload = json!({rows,total,limit,offset})` | WIRED | `jsonrpc.rs:90-97` — payload built and passed to `structured()` |
| `tools_call_result_parses_as_valid_mcp_content` | `serde_json::from_value::<CallToolResult>` | deserializes `response["result"]` with client's own type | WIRED | `jsonrpc.rs:178` — `serde_json::from_value(response["result"].clone())` |
| `tenant_a_isolation` / `tenant_b_isolation` | `result["result"]["structuredContent"]["rows"]` | navigation path change from bare `content` array | WIRED | `mcp_tenant_isolation.rs:266-268` and `327-329` |
| tenant isolation tests | `content[0]["type"] == text` | content-block shape lock assertion | WIRED | `mcp_tenant_isolation.rs:260-264` and `321-325` |
| `ferro-mcp-server/tests/jsonrpc_integration.rs` `tools_call_returns_rows` | new envelope shape (content.len()==1 + structuredContent.rows) | Rule 1 fix applied in commit `425b75a4` | WIRED | `jsonrpc_integration.rs:56-61` — `content.len()==1`, `content[0]["type"]=="text"`, `structuredContent.rows` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `handle_tools_call` Ok arm | `result.rows` | `dispatch(service, filters, limit, offset, db, tenant_id)` — SQL query with tenant predicate | Yes — SeaORM query against live DB; D-04 test seeds 4 rows, asserts tenant-1 filter returns 2 | FLOWING |

### Behavioral Spot-Checks

Disk constraint applies (95% used). Scoped builds were run by executors and are documented in SUMMARY files. Static verification substituted for compile-step checks.

| Behavior | Evidence | Status |
|----------|----------|--------|
| `CallToolResult::structured` import present and used | `jsonrpc.rs:10` import; `jsonrpc.rs:96` usage | PASS |
| Old `"content": result.rows` shape removed | grep on `jsonrpc.rs` finds no match | PASS |
| D-04 test exists and uses `from_value::<CallToolResult>` | `jsonrpc.rs:178` confirmed | PASS |
| Integration test `tools_call_returns_rows` asserts new shape | `jsonrpc_integration.rs:56-61` confirmed | PASS |
| Tenant isolation tests pass (reported by Plan 02 executor) | `cargo test -p app -- tenant_isolation` 3 passed (per 205-02 SUMMARY) | PASS |
| All 4 phase commits present in git history | `4ea9caac`, `425b75a4`, `1f58c411`, `9a61155d` verified via `git log` | PASS |
| Live dogfood GO verdict recorded | `205-ACCEPTANCE.md` confirmed; GO with 2 Acme orders | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AMCP-03 | 205-01, 205-03 | Tool call returns rows as MCP structured content with output shape derived from projection | SATISFIED | `CallToolResult::structured(payload)` in `handle_tools_call`; live dogfood confirmed valid `CallToolResult` envelope |
| AMCP-10 | 205-02, 205-03 | Tool call executes within token's tenant context; tenant-scoped token returns only that tenant's rows | SATISFIED | Both tenant isolation tests navigate `structuredContent.rows` and assert per-row `tenant_id`; live dogfood confirmed alice@acme.test sees exactly 2 Acme orders (Globex excluded) |

Note: the REQUIREMENTS.md traceability table maps AMCP-03 to Phase 197 and AMCP-10 to Phase 200. Phase 205 is a defect-fix phase that re-exercises and confirms these requirements after fixing the result-formatting bug introduced in Phase 197. The ROADMAP.md Phase 205 entry explicitly names both requirements.

### Anti-Patterns Found

None. No TODO/FIXME/PLACEHOLDER markers found in modified files. No stub returns (`return null`, `return []`, `return {}`) in production paths. The Ok arm is fully implemented with real data flow from `dispatch()` through `CallToolResult::structured()`.

### Human Verification Required

None. All truths are verifiable statically or via the committed live-dogfood acceptance record (`205-ACCEPTANCE.md`). The human checkpoint (Plan 03 Task 2) was completed and recorded.

### Gaps Summary

No gaps. All 8 observable truths verified. All artifacts exist, are substantive, and are wired. The live dogfood GO verdict (recorded in `205-ACCEPTANCE.md`) confirms end-to-end correctness against a real MCP client with the full browser-OAuth chain and tenant scoping intact.

---

_Verified: 2026-06-12T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
