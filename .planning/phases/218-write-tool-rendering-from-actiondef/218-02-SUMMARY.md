---
phase: 218-write-tool-rendering-from-actiondef
plan: "02"
subsystem: ferro-mcp-server
tags: [tdd, green-tests, mcp, write-tools, actiondef, guard-filter, collision-pass, ci-gate]
dependency_graph:
  requires: [218-01-schema-green, build_action_input_schema]
  provides: [218-02-renderer-green, render_action_tool, disambiguate_write_tool_collisions]
  affects:
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/jsonrpc.rs
    - app/src/tests/mcp_tenant_isolation.rs
tech_stack:
  added: []
  patterns: [tdd-green-wave, guard-filtered-visibility, write-tool-annotation, collision-disambiguation]
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/jsonrpc.rs
    - app/src/tests/mcp_tenant_isolation.rs
decisions:
  - "render_action_tool documented VISIBILITY-only (not auth gate) — Phase 219 enforces server-side; 217 scope gate is read/write boundary"
  - "Collision pass uses tagged Vec<(String, Tool)> to preserve service association through collection; slice parameter (&mut [(String, Tool)]) per clippy::ptr_arg"
  - "mcp_tenant_isolation.rs long-line fmt drift committed here to keep cargo fmt --all -- --check clean (pre-existing, unrelated to phase logic)"
metrics:
  duration_seconds: 1187
  completed_date: "2026-06-13T20:51:54Z"
  tasks_completed: 3
  files_modified: 3
---

# Phase 218 Plan 02: Write-Tool Rendering from ActionDef (GREEN) — Summary

One-liner: Extend `render_exposed_tools` with `render_action_tool` (guard-filtered, annotated, description-fallback) and a cross-service collision disambiguation pass — turning all 6 Plan 00 renderer RED tests and the SC#5 jsonrpc strict-deser test GREEN, then passing the full fmt+clippy+test CI gate.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | render_action_tool + extended render_exposed_tools + collision pass | 382ae2f6 | ferro-mcp-server/src/renderer.rs |
| 2 | SC#5 GREEN + Phase 219 routing comment in handle_tools_call | e1d3ce81 | ferro-mcp-server/src/jsonrpc.rs |
| 3 | Full CI gate (fmt + clippy + test) + fmt/clippy fixes | 7bfc5e5e | ferro-mcp-server/src/renderer.rs, app/src/tests/mcp_tenant_isolation.rs |

## What Changed

### `ferro-mcp-server/src/renderer.rs`

Three additions in one file:

1. **Import extended:** `ActionDef` added to top-level `use ferro_projections::{...}` — used by `render_action_tool`.

2. **`render_exposed_tools` replaced:** Iterator `.map().collect()` changed to explicit `for` loop with a tagged `Vec<(String, Tool)>` internal accumulator. For each `mcp_exposed` service: pushes the read tool first, then iterates `service.actions` and pushes one write tool per `ActionDef` (via `render_action_tool`). Calls `disambiguate_write_tool_collisions` before stripping tags.

3. **`render_action_tool` added** (private helper): Guard-checks all `action.preconditions` against `ctx.evaluated_guards` — any explicit `Some(&false)` returns `Ok(None)`. Description fallback: `action.description` → `action.display_name` → `"{action.name} {service.name}"`. Schema via `crate::schema::build_action_input_schema`. Annotations: `.read_only(false).destructive(action.transition_trigger.is_some())`. Includes the VISIBILITY-not-auth doc comment (T-218-02 mitigation).

4. **`disambiguate_write_tool_collisions` added** (private): Counts write tool name occurrences across services; renames any with count > 1 to `<name>_on_<service_name>`. Read tools (`list_*`) excluded. Parameter is `&mut [(String, Tool)]` (clippy::ptr_arg).

### `ferro-mcp-server/src/jsonrpc.rs`

One change: 4-line comment added above `let service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name)` in `handle_tools_call` — explains that write-tool names result in a service lookup miss → `-32601`, which is correct Phase 218 behavior (no executor); write-tool dispatch is Phase 219.

### `app/src/tests/mcp_tenant_isolation.rs`

Formatting-only: rustfmt reformatted two long `handle_tools_call(...)` call sites (pre-existing drift unrelated to Phase 218 logic). Committed here to keep `cargo fmt --all -- --check` clean.

## Test Results

### renderer.rs (11 tests, all GREEN)

```
test renderer::tests::adding_field_changes_schema ... ok
test renderer::tests::test_render_read_only ... ok
test renderer::tests::test_render_schema_embedded ... ok
test renderer::tests::test_guard_false_omits_tool ... ok          ← Plan 00 RED → GREEN
test renderer::tests::test_one_write_tool_per_action ... ok        ← Plan 00 RED → GREEN
test renderer::tests::test_mcp_exposed_filter ... ok
test renderer::tests::test_guard_true_includes_tool ... ok         ← Plan 00 RED → GREEN
test renderer::tests::test_guard_absent_includes_tool ... ok       ← Plan 00 RED → GREEN
test renderer::tests::test_render_tool_name ... ok
test renderer::tests::test_write_tool_annotations_non_transition ... ok  ← Plan 00 RED → GREEN
test renderer::tests::test_write_tool_annotations_transition ... ok       ← Plan 00 RED → GREEN

test result: ok. 11 passed; 0 failed
```

### jsonrpc.rs (2 tests, all GREEN)

```
test jsonrpc::tests::write_tools_definitions_parse_as_valid_mcp_tool ... ok  ← Plan 00 RED → GREEN
test jsonrpc::tests::tools_call_result_parses_as_valid_mcp_content ... ok

test result: ok. 2 passed; 0 failed
```

### Full CI Gate

```
cargo fmt --all -- --check        → CLEAN
cargo clippy --all --all-targets -- -D warnings  → 0 warnings, 0 errors
cargo test --all-features         → all test result lines: ok, 0 failed
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Two clippy warnings in `disambiguate_write_tool_collisions`**
- **Found during:** Task 3 (`cargo clippy --all --all-targets -- -D warnings`)
- **Issue 1:** `&mut Vec<(String, Tool)>` parameter — clippy::ptr_arg recommends `&mut [(String, Tool)]`
- **Issue 2:** Nested `if !starts_with("list_") { if count > 1 { ... } }` — clippy::collapsible_if
- **Fix:** Changed parameter type to `&mut [(String, Tool)]`; collapsed the two `if` guards with `&&`
- **Files modified:** `ferro-mcp-server/src/renderer.rs`
- **Commit:** 7bfc5e5e

**2. [Rule 3 - Blocking] Pre-existing fmt drift in `app/src/tests/mcp_tenant_isolation.rs`**
- **Found during:** Task 3 (`cargo fmt --all -- --check`)
- **Issue:** Two long `handle_tools_call(...)` call sites exceeded rustfmt line width — pre-existing, not introduced by this phase
- **Fix:** Applied `cargo fmt --all` to reformat; committed alongside the clippy fix
- **Files modified:** `app/src/tests/mcp_tenant_isolation.rs`
- **Commit:** 7bfc5e5e

## Security Coverage

| Threat | Mitigation | Status |
|--------|-----------|--------|
| T-218-02: Guard filter misread as auth gate | `render_action_tool` doc comment: "VISIBILITY filter, NOT an authorization gate"; Phase 219 comment in handle_tools_call | GREEN — visibility semantics pinned by 6 renderer tests |
| T-218-03: Malformed tool definition breaks strict clients | SC#5 `write_tools_definitions_parse_as_valid_mcp_tool` desers every tool via `rmcp::model::Tool` | GREEN |
| T-218-01: Sensitive input disclosure (inherited) | `build_action_input_schema` excludes `FieldMeaning::Sensitive` (Plan 01); exercised by SC#5 fixture | GREEN (verified by Plan 01 tests) |

## Known Stubs

None. All write-tool definitions are fully rendered from `ActionDef`. The write-tool `tools/call` path intentionally returns `-32601` in Phase 218 (no executor) — this is documented behavior, not a stub, and is the correct 218 state. Phase 219 wires dispatch.

## Threat Flags

None. No new network endpoints, auth paths, or schema changes at trust boundaries introduced by this plan. The `render_action_tool` visibility filter is documented as non-authoritative.

## Self-Check: PASSED

- FOUND: ferro-mcp-server/src/renderer.rs
- FOUND: ferro-mcp-server/src/jsonrpc.rs
- FOUND: .planning/phases/218-write-tool-rendering-from-actiondef/218-02-SUMMARY.md
- FOUND commit 382ae2f6 (Task 1)
- FOUND commit e1d3ce81 (Task 2)
- FOUND commit 7bfc5e5e (Task 3)
- `grep -q ".read_only(false)" ferro-mcp-server/src/renderer.rs` → FOUND
- `grep -q "render_action_tool" ferro-mcp-server/src/renderer.rs` → FOUND
- `grep -q "VISIBILITY" ferro-mcp-server/src/renderer.rs` → FOUND
- Renderer tests: 11/11 GREEN
- SC#5 test: GREEN
- Full CI gate: fmt CLEAN, clippy 0 warnings, test 0 failed
