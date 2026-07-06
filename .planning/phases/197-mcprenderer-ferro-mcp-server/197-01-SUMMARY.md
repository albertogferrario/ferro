---
phase: 197-mcprenderer-ferro-mcp-server
plan: "01"
subsystem: ferro-mcp-server
tags: [mcp, projections, crate-scaffold, tdd]
dependency_graph:
  requires: [ferro-projections]
  provides: [ferro-mcp-server crate skeleton, ServiceDef.mcp_exposed marker]
  affects: [ferro-projections/src/service.rs, workspace Cargo.toml]
tech_stack:
  added: [ferro-mcp-server crate, rmcp (server+macros+base64), sea-orm]
  patterns: [Renderer trait impl stub, consuming builder, serde default field, TDD RED/GREEN]
key_files:
  created:
    - ferro-mcp-server/Cargo.toml
    - ferro-mcp-server/README.md
    - ferro-mcp-server/src/lib.rs
    - ferro-mcp-server/src/error.rs
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/schema.rs
    - ferro-mcp-server/src/dispatch.rs
  modified:
    - ferro-projections/src/service.rs
    - Cargo.toml
decisions:
  - rmcp features set to server+macros+base64 (not schemars-only) because rmcp model module unconditionally imports pastey and base64 regardless of feature flags; transport-io (stdio) is still excluded
  - mcp_exposed uses #[serde(default)] without skip_serializing_if so explicit true values serialize
metrics:
  duration: "322s (~5m)"
  completed: "2026-06-10"
  tasks_completed: 3
  files_changed: 8
---

# Phase 197 Plan 01: ferro-mcp-server Scaffold Summary

New `ferro-mcp-server` output crate with `McpRenderer` stub implementing the `Renderer` trait, plus `mcp_exposed: bool` opt-in marker on `ServiceDef` in `ferro-projections`.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Crate manifest (Cargo.toml), README, error module | d448eaae |
| 2 | Stub modules: lib.rs, renderer.rs, schema.rs, dispatch.rs | c3a525e8 |
| 3 (TDD) | mcp_exposed field + builder on ServiceDef | d957a080 (RED), 91bf349a (GREEN) |

## Verification Results

- `cargo build -p ferro-mcp-server`: exit 0
- `cargo clippy -p ferro-mcp-server --all-targets -- -D warnings`: clean
- `cargo test -p ferro-projections mcp_exposed`: 2 passed
- `grep -q ferro-mcp-server ferro-projections/Cargo.toml`: returns nothing (SC-4 preserved)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rmcp feature set adjusted from `["schemars"]` to `["server", "macros", "base64"]`**
- **Found during:** Task 2 (first build attempt)
- **Issue:** The plan specified `rmcp = { version = "0.12", default-features = false, features = ["schemars"] }` but rmcp's model module unconditionally imports `pastey::paste` (requires `macros` feature) and `base64::engine` (requires `base64` feature), and `model/tool.rs` references `crate::handler::server` (requires `server` feature). The `schemars`-only feature set does not compile.
- **Fix:** Changed features to `["server", "macros", "base64"]`. The `server` feature enables `transport-async-rw` (tokio io-util + tokio-util/codec) but does NOT enable `transport-io` (tokio/io-std, the stdio transport). The constraint from the plan — no stdio transport — is preserved. The spirit of the constraint (no transport-io feature) is upheld.
- **Files modified:** ferro-mcp-server/Cargo.toml
- **Commit:** c3a525e8

## TDD Gate Compliance

- RED commit: d957a080 — `test(197-01): add failing tests for ServiceDef.mcp_exposed`
- GREEN commit: 91bf349a — `feat(197-01): add mcp_exposed field + builder to ServiceDef`
- Both gates present in sequence. REFACTOR not needed (minimal field + builder, no cleanup required).

## Known Stubs

| Stub | File | Line | Reason |
|------|------|------|--------|
| `McpRenderer::render` returns `Err(Render("not yet implemented"))` | ferro-mcp-server/src/renderer.rs | 24 | Intentional — plan 02 fills the real rendering logic |
| `build_input_schema` returns empty schema object | ferro-mcp-server/src/schema.rs | 6 | Intentional — plan 02 fills schema derivation |
| `dispatch` returns empty DispatchResult | ferro-mcp-server/src/dispatch.rs | 15 | Intentional — plan 03 fills SQL dispatch |

These stubs do not block this plan's goal (scaffold + mcp_exposed marker); they are the defined Wave 1 deliverable.

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundary crossings introduced in this plan. All new code is pure data types and non-executing stubs. The `mcp_exposed` field defaults to `false` (opt-in), satisfying T-197-02.

## Self-Check: PASSED
