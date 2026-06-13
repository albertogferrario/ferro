---
phase: 218-write-tool-rendering-from-actiondef
reviewed: 2026-06-13T00:00:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - ferro-mcp-server/src/schema.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - app/src/tests/mcp_tenant_isolation.rs
findings:
  critical: 0
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 218: Code Review Report

**Reviewed:** 2026-06-13
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Phase 218 adds `build_action_input_schema`, the write-tool emission loop in `render_exposed_tools`, the `render_action_tool` function, and the `disambiguate_write_tool_collisions` pass. The overall architecture is correct: `data_type_to_json_schema` is promoted and shared (single source of truth confirmed), `FieldMeaning::Sensitive` is excluded from both `properties` and `required[]` (T-218-01 confirmed), the guard filter is explicitly documented as a visibility-only pass (T-218-02 confirmed), and `handle_tools_call` correctly returns `-32601` for write-tool calls in Phase 218 with no executor. The `idempotentHint` annotation is correctly absent.

One warning: the collision-counter logic counts total tool occurrences rather than distinct services, which diverges from what the doc comment states and would misfire on an intra-service duplicate action name.

## Warnings

### WR-01: Collision counter counts total occurrences, not distinct services

**File:** `ferro-mcp-server/src/renderer.rs:103-108`

**Issue:** The function doc comment says "Count how many distinct services each write tool name appears in," but the implementation increments a single counter for every tool with that name regardless of which service emitted it. If one service had two `ActionDef`s with the same `name` (a data-model error, but not prevented by `ActionDef`'s API), the counter would reach 2 and both would be renamed to `<name>_on_<service>` as if they were a cross-service collision. The rename would be correct by coincidence, but the intent does not match the code. More importantly, a reader auditing the rename logic against the comment would conclude the logic is correct without noticing the gap.

**Fix:** Either align the code to the comment (count distinct service names per action name) or align the comment to the code (remove "distinct services" language):

Option A — align code to comment (strict correctness):
```rust
// Count how many distinct services each write tool name appears in.
let mut name_to_services: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
for (service_name, tool) in tagged.iter() {
    if !tool.name.starts_with("list_") {
        name_to_services
            .entry(tool.name.to_string())
            .or_default()
            .insert(service_name.clone());
    }
}

for (service_name, tool) in tagged.iter_mut() {
    if !tool.name.starts_with("list_")
        && name_to_services
            .get(tool.name.as_ref())
            .map_or(0, |s| s.len())
            > 1
    {
        let new_name = format!("{}_on_{}", tool.name, service_name);
        tool.name = new_name.into();
    }
}
```

Option B — align comment to code (acceptable if duplicate intra-service names are treated as equivalent to cross-service collisions):
```rust
// Count how many times each write tool name appears across all services.
// A count > 1 means the name would be ambiguous in tools/list, regardless
// of whether the duplicates come from one service or multiple.
```

## Info

### IN-01: Scope-rejection uses -32603 (internal error) for an auth/permission failure

**File:** `ferro-mcp-server/src/jsonrpc.rs:74-82`

**Issue:** When a read-scoped key calls a write tool, the function returns error code `-32603` (JSON-RPC Internal Error). `-32603` conventionally signals a server-side crash or unexpected state. A scope-gate failure is a predictable auth condition; returning `-32603` conflates it with server errors and makes client-side error handling harder.

**Fix:** The MCP spec does not define an `Unauthorized` code, but `-32600` (Invalid Request) or `-32601` (Method not found / not permitted) are both closer to the semantics than `-32603`. Alternatively, define a crate-level constant and document the choice:
```rust
// MCP has no dedicated auth code; use -32600 (Invalid Request) for scope
// failures so clients can distinguish auth errors from server crashes (-32603).
const ERR_SCOPE: i64 = -32600;

return json!({
    "error": {
        "code": ERR_SCOPE,
        "message": "scope insufficient: read key cannot call write tools"
    }
});
```

### IN-02: `handle_tools_call` top-level doc comment omits write-tool behavior

**File:** `ferro-mcp-server/src/jsonrpc.rs:47-53`

**Issue:** The doc comment describes only the read-tool path ("Strips the `list_` prefix from `name` to find the `ServiceDef`"). It does not mention that write-tool names (no `list_` prefix) fall through to the service lookup as the action name, fail to find a service match, and return `-32601`. The Phase 218 intent is explained by the in-line comment at line 62-65, but the top-level doc is misleading to a reader who starts there.

**Fix:**
```rust
/// Handle an MCP `tools/call` request.
///
/// For read tools (`list_<svc>`): strips the `"list_"` prefix to find the `ServiceDef`,
/// then delegates to `dispatch`.
///
/// For write tools (Phase 218): no executor exists yet. The service lookup finds no service
/// whose `.name` equals the action name, so this returns `-32601 Method not found`.
/// Write-tool dispatch is implemented in Phase 219.
///
/// Pagination keys are removed from `arguments` before passing the remainder as filters.
/// The filter-key allowlist and limit clamp live in `dispatch` (Phase 197 WR-01/WR-02).
```

### IN-03: No direct unit test for cross-service collision rename

**File:** `ferro-mcp-server/src/renderer.rs`

**Issue:** `disambiguate_write_tool_collisions` is tested indirectly through the single-service fixture tests (which never trigger a rename). There is no test verifying that two services each declaring an action with the same name produce correctly renamed tools (`<name>_on_<service_a>`, `<name>_on_<service_b>`), and that non-colliding tools from those same services are left untouched.

**Fix:** Add a unit test:
```rust
#[test]
fn test_collision_rename_across_services() {
    let svc_a = ServiceDef::new("invoice")
        .mcp_exposed(true)
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .action(ActionDef::new("approve"));

    let svc_b = ServiceDef::new("refund")
        .mcp_exposed(true)
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .action(ActionDef::new("approve"))
        .action(ActionDef::new("cancel")); // non-colliding, must not be renamed

    let tools = render_exposed_tools(&[svc_a, svc_b], &McpContext::default())
        .expect("render ok");

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(names.contains(&"approve_on_invoice"), "invoice approve must be renamed");
    assert!(names.contains(&"approve_on_refund"), "refund approve must be renamed");
    assert!(!names.contains(&"approve"), "bare 'approve' must not appear");
    assert!(names.contains(&"cancel"), "non-colliding 'cancel' must not be renamed");
}
```

---

_Reviewed: 2026-06-13_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
