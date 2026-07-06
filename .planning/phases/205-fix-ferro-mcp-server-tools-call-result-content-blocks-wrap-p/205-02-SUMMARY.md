---
phase: 205-fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p
plan: "02"
subsystem: ferro-mcp-server / app tests
tags: [mcp, tenant-isolation, regression, test-update]
dependency_graph:
  requires: ["205-01"]
  provides: ["tenant-isolation regression proof under post-fix envelope"]
  affects: ["app/src/tests/mcp_tenant_isolation.rs"]
tech_stack:
  added: []
  patterns: ["structuredContent.rows navigation", "content[0].type==text assertion"]
key_files:
  modified:
    - app/src/tests/mcp_tenant_isolation.rs
decisions:
  - "Navigate structuredContent.rows (not result.content) for row assertions — matches CallToolResult::structured() wire shape"
  - "Add content[0][type]=text assertion in both tests to lock the post-fix envelope shape and catch future regressions"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-12"
  tasks_completed: 2
  files_modified: 1
---

# Phase 205 Plan 02: Tenant Isolation Test Re-pointing Summary

Re-point both tenant-isolation integration tests from the old bare-rows navigation path (`result["result"]["content"]` as row array) to the new `CallToolResult::structured()` envelope produced by Plan 01 (`result["result"]["structuredContent"]["rows"]`), and add a `content[0]["type"]=="text"` assertion to lock the valid-content-block shape.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Re-point tenant_a_isolation to structuredContent.rows + assert content-block shape | 1f58c411 | app/src/tests/mcp_tenant_isolation.rs |
| 2 | Re-point tenant_b_isolation to structuredContent.rows + assert content-block shape | 9a61155d | app/src/tests/mcp_tenant_isolation.rs |

## What Changed

Both `tenant_a_isolation` and `tenant_b_isolation` in `app/src/tests/mcp_tenant_isolation.rs` had their row-navigation block replaced:

**Before (old, broken shape):**
```rust
let rows = result["result"]["content"]
    .as_array()
    .expect("result.content must be an array");
```

**After (new, post-fix shape):**
```rust
let content = result["result"]["content"]
    .as_array()
    .expect("result.content must be an array");
assert_eq!(
    content[0]["type"].as_str(),
    Some("text"),
    "content[0] must be a text block (type=text) — locks the post-fix shape"
);

let rows = result["result"]["structuredContent"]["rows"]
    .as_array()
    .expect("structuredContent.rows must be an array");
```

The per-row `tenant_id` assertions and the cross-tenant leak checks are unchanged — only the navigation path to `rows` moved.

## Verification Results

### Scoped tests
- `cargo test -p app -- tenant_isolation`: 3 passed (tenant_a_isolation, tenant_b_isolation, tenant_context_parity)
- `cargo test -p ferro-mcp-server`: 27 passed (17 unit + 5 dispatch integration + 5 jsonrpc integration), including `jsonrpc::tests::tools_call_result_parses_as_valid_mcp_content` (D-04 interop from Plan 01)

### Quality gate
- `cargo fmt --all -- --check`: clean (no output)
- `cargo clippy -p app --all-targets -- -D warnings`: clean (no warnings)
- Full `--all-features` gate: deferred to phase verifier (Plan 03 manual dogfood gate); disk at 33Gi/33% free, adequate but tight per project notes

### Schema churn
None — `docs/protocol/schemas/*.json` not regenerated (no schema-touching tests ran).

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced. This plan only modifies test navigation paths.

## Threat Model Coverage

| Threat | Status |
|--------|--------|
| T-205-04: Cross-tenant leak under new envelope | Mitigated — both tests assert every row has the expected tenant_id and no row has the other tenant_id, navigating `structuredContent.rows` |
| T-205-05: Test navigation path drift / silent regression | Mitigated — `content[0]["type"]=="text"` assertion added to both tests; a future regression to bare-rows would fail this assertion loudly |

## Self-Check: PASSED

- `app/src/tests/mcp_tenant_isolation.rs`: modified and committed
- Commits 1f58c411 and 9a61155d: both present in git log
- Both tenant isolation tests: green
- `structuredContent` appears 6 times in the test file (≥2 required)
- `content[0]["type"]` assertions present at lines 261 and 322
