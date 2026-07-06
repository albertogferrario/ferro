---
phase: 135-servicedef-derivation-bridge
plan: "02"
subsystem: ferro-mcp
tags: [mcp, projections, servicedef, intent-derivation]
dependency_graph:
  requires: ["135-01"]
  provides: ["generate_projection MCP tool"]
  affects: ["ferro-mcp"]
tech_stack:
  added: []
  patterns: ["MCP tool registration via #[tool] macro", "ModelDetails -> ModelMetadata conversion bridge"]
key_files:
  created:
    - ferro-mcp/src/tools/generate_projection.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs
decisions:
  - "Used matching_signals field name from IntentScore (not signals) — matched actual ferro-projections struct"
  - "Placed generate_projection in alphabetical position in mod.rs (between explain_route and generate_types)"
metrics:
  duration: "~6min"
  completed: "2026-04-17T17:59:55Z"
  tasks_completed: 2
  files_modified: 3
---

# Phase 135 Plan 02: Generate Projection MCP Tool Summary

## One-liner

`generate_projection` MCP tool bridging list_models -> ModelMetadata -> ServiceDef::from_model() -> derive_intents() in a single agent call.

## What Was Built

Added the `generate_projection` MCP tool to ferro-mcp, completing the ServiceDef derivation bridge. Agents can now call a single tool with a model name and receive a fully derived ServiceDef JSON, ranked intent scores, and a list of fields requiring manual enrichment.

### Pipeline implemented

```
list_models::execute() -> ModelDetails
  -> FieldInfo -> FieldMetadata conversion
  -> ModelMetadata
  -> ServiceDef::from_model()
  -> derive_intents()
  -> GenerateProjectionResult { service_def, intents, inferred_field_count, manual_enrichment_needed }
```

### Files created/modified

- `ferro-mcp/src/tools/generate_projection.rs` — execute() function with full bridge pipeline
- `ferro-mcp/src/tools/mod.rs` — module registration in alphabetical order
- `ferro-mcp/src/service.rs` — GenerateProjectionParams struct + #[tool] handler

## Decisions Made

- Used `matching_signals` (not `signals`) — matched the actual `IntentScore` struct field name in ferro-projections.
- Placed `generate_projection` alphabetically in `mod.rs` between `explain_route` and `generate_types`.
- `manual_enrichment_needed` hardcoded to `["actions", "state_machine", "relationships"]` — these are the three ServiceDef fields that cannot be inferred from field types alone and require hand-authoring.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy uninlined format args**
- **Found during:** Task 2 verification
- **Issue:** `format!("Model '{}' not found. Available: {:?}", model_name, available)` violated clippy::uninlined_format_args
- **Fix:** Rewrote as `format!("Model '{model_name}' not found. Available: {available:?}")`
- **Files modified:** ferro-mcp/src/tools/generate_projection.rs
- **Commit:** included in task 1 commit (1a092b41) after fmt fix

**2. [Rule 1 - Bug] rustfmt reordering in mod.rs**
- **Found during:** Task 2 verification (cargo fmt --check)
- **Issue:** `generate_projection` was inserted after `generation_context` instead of alphabetically before `generate_types`
- **Fix:** Reordered to `generate_projection -> generate_types -> generation_context`

## Known Stubs

None — all fields in GenerateProjectionResult are wired to real data sources.

## Verification

- `cargo fmt --all -- --check`: pass
- `cargo clippy --all --all-targets -- -D warnings`: pass
- `cargo test --all-features`: pass
