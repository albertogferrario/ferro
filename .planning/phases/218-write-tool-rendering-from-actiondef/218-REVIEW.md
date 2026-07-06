---
phase: 218-write-tool-rendering-from-actiondef
reviewed: 2026-06-13T23:30:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - ferro-mcp-server/src/schema.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - app/src/tests/mcp_tenant_isolation.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: issues_found
---

# Phase 218: Code Review Report (Re-Review)

**Reviewed:** 2026-06-13T23:30:00Z
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Re-review following commit 52232d5d (WR-01 fix). All four files were read in full.

**WR-01 confirmed fixed.** `disambiguate_write_tool_collisions` now uses `HashMap<String, HashSet<String>>` to accumulate distinct service names per action name. The rename fires only when `s.len() > 1`, i.e., when the name spans more than one distinct service. Intra-service duplicates (one distinct service) are left untouched. `list_*` tools are excluded from both the counting pass and the rename pass. The implementation matches the doc comment. No new clippy footguns: `or_default()` is idiomatic, `map_or(0, |s| s.len())` is correct.

**IN-03 confirmed fixed.** Two regression tests were added in the same commit: `test_collision_rename_across_services` verifies cross-service rename and non-rename of non-colliding tools; `test_intra_service_duplicate_not_renamed` verifies intra-service duplicates are untouched. Both tests exercise the paths described by the fix and are structurally sound.

**Remaining findings from the prior review:**

- IN-01 (scope rejection uses -32603) is unchanged. The code path is present and the issue is real but minor.
- IN-02 (doc comment on `handle_tools_call` omits write-tool behavior) is unchanged. The inline comment at line 62–65 still explains the behavior, but the top-level doc does not.

All critical and security focus areas are clean:

- `build_action_input_schema` reuses `pub(crate) data_type_to_json_schema` — single source of truth confirmed.
- `FieldMeaning::Identifier` injection is present and correct; no-Identifier case silently skips (documented, tested at `test_action_schema_no_identifier_field_is_silent_noop`).
- `FieldMeaning::Sensitive` excluded from both `properties` and `required[]` (T-218-01, tested at `test_action_schema_excludes_sensitive_input`).
- Guard filter is VISIBILITY-only, explicitly documented in `render_action_tool` doc comment (T-218-02).
- `read_only(false)` set on all write tools; `destructive` tied to `action.transition_trigger.is_some()` (D-04).
- `idempotent_hint` is correctly absent.
- `handle_tools_call` returns -32601 for write-tool calls in Phase 218 — correct for no-executor state, documented inline.
- No bare `.unwrap()` in production code paths (only in test helpers, which is acceptable).

## Info

### IN-01: Scope-rejection returns -32603 (internal error) for an auth-scope failure

**File:** `ferro-mcp-server/src/jsonrpc.rs:74`
**Issue:** When a read-scoped key calls a write tool, the error code is `-32603` (Internal Error). That code conventionally signals a server-side crash or unexpected state. A predictable scope-gate failure returning `-32603` conflates auth errors with server errors, making client-side error handling harder.
**Fix:** Use `-32600` (Invalid Request) or a documented crate-level constant instead:
```rust
// MCP has no dedicated auth code; -32600 (Invalid Request) is closer than
// -32603 (Internal Error) for a predictable scope failure.
return json!({
    "error": {
        "code": -32600,
        "message": "scope insufficient: read key cannot call write tools"
    }
});
```

### IN-02: Top-level doc comment on `handle_tools_call` omits write-tool behavior

**File:** `ferro-mcp-server/src/jsonrpc.rs:47-53`
**Issue:** The doc comment describes only the read-tool path ("Strips the `list_` prefix from `name` to find the `ServiceDef`"). It does not mention that write-tool calls fall through the service lookup to -32601. A reader starting from the doc comment gets an incomplete picture; the behavior is only explained by the inline comment at line 62-65.
**Fix:** Extend the doc comment:
```rust
/// Handle an MCP `tools/call` request.
///
/// For read tools (`list_<svc>`): strips the `"list_"` prefix to find the `ServiceDef`,
/// then delegates to `dispatch`. Pagination keys are removed from `arguments` before
/// passing the remainder as filters.
///
/// For write tools (Phase 218): no executor exists yet. The service lookup finds no service
/// whose `.name` equals the action name, so this returns `-32601 Method not found`.
/// Write-tool dispatch is implemented in Phase 219.
///
/// The filter-key allowlist and limit clamp live in `dispatch` (Phase 197 WR-01/WR-02).
```

---

_Reviewed: 2026-06-13T23:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
