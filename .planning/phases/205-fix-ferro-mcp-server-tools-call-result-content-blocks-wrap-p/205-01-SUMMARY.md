---
phase: 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p
plan: 01
subsystem: api
tags: [mcp, rmcp, ferro-mcp-server, content-blocks, jsonrpc, serde]

requires:
  - phase: 200-per-tenant-scoping-policy-authorization-dogfood-acceptance
    provides: dispatch() function + DispatchResult struct with tenant-scoped rows

provides:
  - "handle_tools_call Ok arm emits a valid CallToolResult via structured() (D-01/D-02/D-03)"
  - "D-04 interop regression test that parses the emitted result with the MCP client's own type"
  - "tools_call_returns_rows integration test updated to assert the correct MCP envelope shape"

affects: [205-02, 205-03, ferro-mcp-server, mcp-clients]

tech-stack:
  added: []
  patterns:
    - "CallToolResult::structured(payload) — wraps a serde_json::Value into a valid MCP content envelope with one text block + structuredContent + isError:false"
    - "D-04 interop pattern: serialize server output then deserialize with client's own type to detect schema mismatches before field tests"

key-files:
  created: []
  modified:
    - ferro-mcp-server/src/jsonrpc.rs
    - ferro-mcp-server/tests/jsonrpc_integration.rs

key-decisions:
  - "D-01/D-02/D-03: use CallToolResult::structured(payload) with payload={rows,total,limit,offset}; total/limit/offset nested inside structuredContent, not as top-level result siblings"
  - "D-04: regression test deserializes emitted result via serde_json::from_value::<CallToolResult> — exercises rmcp custom Deserialize (model.rs:1646), the assertion that would have caught the original bug"
  - "D-05: error arms (-32601/-32602/-32603) left byte-for-byte unchanged"
  - "Rule 1 fix: jsonrpc_integration.rs tools_call_returns_rows was asserting the old broken shape (content.len()==3); updated to assert content.len()==1 + structuredContent.rows.len()==3"

patterns-established:
  - "interop test pattern: always parse server-emitted output with the CLIENT's own type, not just with the server's own assertions — prevents shape mismatches that unit tests silently miss"

requirements-completed: [AMCP-03]

duration: ~3min
completed: 2026-06-12
---

# Phase 205 Plan 01: Fix ferro-mcp-server tools/call Result Content Blocks Summary

**Replaced bare-row `content` array in `handle_tools_call` with `CallToolResult::structured(payload)`, yielding a valid MCP envelope (one `type:text` content block + `structuredContent` + `isError:false`) that strict MCP clients parse without Zod errors.**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-06-12T00:26:56Z
- **Completed:** 2026-06-12T00:30:16Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Fixed `handle_tools_call` Ok arm: now calls `CallToolResult::structured(payload)` where `payload = {rows, total, limit, offset}`, producing a well-formed MCP result that a strict client (Claude Code's SDK Zod schema) can parse
- Added D-04 inline interop regression test `tools_call_result_parses_as_valid_mcp_content` in `jsonrpc.rs` — deserializes the emitted result via `serde_json::from_value::<CallToolResult>`, exercises the custom Deserialize at rmcp model.rs:1646, and asserts `content[0].type==text`, `structuredContent` fields present, tenant-1 row count==2
- Error arms (-32601/-32602/-32603) preserved byte-for-byte

## Task Commits

1. **Task 1: Replace Ok arm with CallToolResult::structured (D-01/D-02/D-03)** - `4ea9caac` (fix)
2. **Task 2: Add D-04 interop regression test + Rule 1 fix to integration test** - `425b75a4` (test)

## Files Created/Modified

- `ferro-mcp-server/src/jsonrpc.rs` — fixed Ok arm + new `#[cfg(test)] mod tests` with D-04 interop test
- `ferro-mcp-server/tests/jsonrpc_integration.rs` — updated `tools_call_returns_rows` to assert new correct MCP envelope shape (Rule 1 fix)

## Decisions Made

- `CallToolResult::structured(payload)` is the correct constructor: it sets `content: vec![Content::text(value.to_string())]`, `structured_content: Some(value)`, `is_error: Some(false)` — exactly the D-01/D-02/D-03 requirements
- `json!({ "result": tool_result })` serializes `CallToolResult` inline via its `Serialize` derive; no `serde_json::to_value()` intermediate needed
- `total`/`limit`/`offset` nested inside `structuredContent` payload (D-02), not as top-level siblings of `"result"`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated `tools_call_returns_rows` integration test to assert new correct shape**
- **Found during:** Task 2 (running `cargo test -p ferro-mcp-server`)
- **Issue:** The existing `tools_call_returns_rows` test in `tests/jsonrpc_integration.rs` asserted `content.len() == 3` (old broken shape: one content item per row). With the fix, `content` has exactly 1 text block; rows live under `structuredContent.rows`.
- **Fix:** Updated test to assert `content.len() == 1`, `content[0]["type"] == "text"`, and `structuredContent.rows.len() == 3`
- **Files modified:** `ferro-mcp-server/tests/jsonrpc_integration.rs`
- **Verification:** `cargo test -p ferro-mcp-server` — 17 unit tests + 10 integration tests all pass
- **Committed in:** `425b75a4` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug: test asserting broken shape)
**Impact on plan:** Required for test suite to be green; no scope creep. The integration test was encoding the bug, not protecting against it.

## Docs Grep Verification

A repo-wide grep confirmed no documentation source file (`docs/src/`) describes the old `content: rows` / top-level `total/limit/offset` shape. The only occurrence was `jsonrpc.rs:86` (the code line fixed in Task 1). No docs or MCP-catalog edit is required.

```
grep -r '"content": result\.rows' docs/src/  → exit 1 (no matches)
grep -r '"content": result\.rows' ferro-mcp-server/src/  → exit 1 (no matches)
```

## Issues Encountered

None beyond the Rule 1 integration test fix above.

## Next Phase Readiness

- `handle_tools_call` now emits a valid MCP envelope; a strict client no longer Zod-rejects it
- Plan 02 (`mcp_tenant_isolation.rs` integration test navigation path update) is unblocked — the `result["result"]["content"]` → `result["result"]["structuredContent"]["rows"]` path change documented in 205-PATTERNS.md is ready to apply
- Plan 03 (dogfood re-verification with live MCP client) is unblocked

---
*Phase: 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p*
*Completed: 2026-06-12*
