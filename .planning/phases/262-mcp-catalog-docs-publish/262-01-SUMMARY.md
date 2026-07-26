---
phase: 262-mcp-catalog-docs-publish
plan: "01"
subsystem: ferro-mcp
tags: [generation-context, live-projection, memoize, asset-macro, drift-guard, mcp, v17.0]
dependency_graph:
  requires: [ferro-json-ui/src/catalog.rs, ferro-json-ui/src/component.rs, ferro-json-ui/src/runtime/live_fragment.rs]
  provides: [LiveProjectionGuidance, live_projection field on GenerationContext, live_projection_drift_guard test]
  affects: [ferro-mcp generation_context tool output]
tech_stack:
  added: []
  patterns: [drift-guard test mirroring register_composition_drift_guard, compact D-04 guidance style, registry-derive for builtin name + runtime attribute verification]
key_files:
  created: []
  modified:
    - ferro-mcp/src/tools/generation_context.rs
decisions:
  - "D-01: SC-1 pre-satisfied — verified both drift guards green at 53, no re-implementation"
  - "D-02: No additive gap found in LiveFragment catalog output — LiveFragmentProps schema derives projection/key/template automatically via schemars; template as opaque Value slot is accepted pattern"
  - "D-03: LiveProjectionGuidance added with live_fragment, container_contract, memoize, asset_macro, docs fields covering all three v17.0 capabilities"
  - "D-04: Compact one-to-two-sentence fields with docs/src pointer, backslash continuation style"
  - "D-05: live_projection_drift_guard asserts LiveFragment in global_catalog(), data-live-fragment/data-channel in FERRO_RUNTIME_JS and prose, memoize/asset! names in prose"
metrics:
  duration: "187 seconds (~3 min)"
  completed_date: "2026-07-26"
  tasks_completed: 2
  files_modified: 1
---

# Phase 262 Plan 01: MCP Generation Context — Live Projection Guidance Summary

**One-liner:** Added `LiveProjectionGuidance` struct to `generation_context.rs` covering LiveFragment binding contract, `#[memoize]` request-scoped dedup, and `asset!()` content-hashed embed, drift-guarded against `global_catalog()` and `FERRO_RUNTIME_JS`.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Add LiveProjectionGuidance struct + live_projection field + execute() assembly + drift-guard test | 9c6cd8b9 | ferro-mcp/src/tools/generation_context.rs |
| 2 | Record SC-1 catalog evidence (verification-only) and audit LiveFragment props output | — (verification-only, no code change) | — |

## SC-1 Catalog Evidence (Pre-Existing, Verification-Only)

Both drift guards were already green before this plan (Phase 260 Plan 04 owns the count bump). This phase re-ran them and recorded the results:

- `cargo test -p ferro-json-ui -- tests::builtin_types_count_drift_guard` — **test result: ok. 1 passed; 0 failed** (asserts BUILTIN_TYPES.len() == 53)
- `cargo test -p ferro-mcp -- tools::json_ui_catalog::tests::test_all_components_present` — **test result: ok. 1 passed; 0 failed** (asserts catalog.components.len() == 53 incl. LiveFragment by name)

Phase 262 Plan 01's SC-1 contribution: ran both tests, recorded green result. The count/mirror bump is owned by Phase 260 Plan 04.

## D-02 Audit: LiveFragment Props Output

`LiveFragmentProps` struct at `ferro-json-ui/src/component.rs:753-763` has exactly three fields: `projection: String`, `key: String`, `template: serde_json::Value`. The `BUILTIN_SPECS` entry at `catalog.rs:382` carries an adequate description: "Binds a child template to a ferro-projection per-key snapshot; re-renders in place on each delta via server-push HTML over the ferro-broadcast WebSocket." The schemars-derived schema exposes all three field names automatically. The `template` field renders as an opaque `{}` Value slot — same shape as other container components with child/template slots (accepted, known pattern).

**Outcome: no additive gap found. No `catalog.rs` modification made or needed.**

## SC-2: Generation Context Guidance (The Killer-Feature Deliverable)

`LiveProjectionGuidance` struct added with five fields:

- `live_fragment`: When to use LiveFragment, the `projection`/`key`/`template` prop contract, first-paint behavior with absent snapshot (empty child {}), one-binding-pattern limitation (non-goal).
- `container_contract`: Server wraps rendered child in `<div data-live-fragment data-channel="projection.{name}.{key}">`, HTML-escaped server-side (server-controlled, not user-injectable), no-WASM client runtime subscribes over `/_ferro/ws` and swaps innerHTML on each `fragment` event.
- `memoize`: `#[memoize]` annotation with `use ferro::memoize`, request-scoped dedup per (callsite, args), concurrent caller coalescing, error caching, graceful no-op outside request scope, complements eager_loading/BatchLoad (not cross-request caching).
- `asset_macro`: `asset!("path")` embed via include_bytes!, OnceLock lazy registration, content-hashed `&'static str` URL, `ferro::bundle` serving required, `ferro assets fetch` CLI for author-time downloads.
- `docs`: Pointer to docs/src/json-ui/components.md, runtime-primitives.md, ferro-assets.md, projections.md.

### Drift Guard: `live_projection_drift_guard`

Mirrors the `register_composition_drift_guard` pattern with three assertions:

1. `"LiveFragment"` is in `ferro_json_ui::global_catalog().components_sorted()` AND appears in the prose.
2. `"data-live-fragment"` and `"data-channel"` both appear in `ferro_json_ui::FERRO_RUNTIME_JS` AND in the prose.
3. `"memoize"` and `"asset!"` both appear in the combined memoize/asset_macro/docs prose.

Test result: **ok. 2 passed; 0 failed** (drift guard + updated has_all_sections test).

## Verification Results

```
cargo fmt --all -- --check                                              EXIT 0
cargo test -p ferro-mcp -- live_projection_drift_guard                 ok. 1 passed
cargo test -p ferro-mcp -- test_generation_context_has_all_sections    ok. 1 passed
cargo test -p ferro-json-ui -- builtin_types_count_drift_guard (SC-1)  ok. 1 passed
cargo test -p ferro-mcp -- test_all_components_present (SC-1 mirror)   ok. 1 passed
```

Acceptance criteria:

- [x] `grep -c 'struct LiveProjectionGuidance'` → 1
- [x] `grep -c 'pub live_projection: LiveProjectionGuidance'` → 1
- [x] `grep -c 'fn live_projection_drift_guard'` → 1
- [x] `grep -c 'data-live-fragment'` → 3 (prose ×2 + test assertion ×1; ≥ 2 required)
- [x] `cargo test -p ferro-mcp -- live_projection_drift_guard` exits 0
- [x] `cargo test -p ferro-mcp -- test_generation_context_has_all_sections` exits 0
- [x] `cargo fmt --all -- --check` exits 0
- [x] `catalog.rs:1303` NOT modified (SC-1 verification-only per D-01)

## Deviations from Plan

None — plan executed exactly as written. SC-1 was verification-only as specified; D-02 audit found no gap as the RESEARCH had predicted (resolved Open Question 1).

## Known Stubs

None. All five `LiveProjectionGuidance` fields are populated with substantive prose in `execute()`.

## Threat Flags

None. This plan adds documentation text and tests only — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The `container_contract` prose correctly describes the server-authoritative, HTML-escaped channel security posture (T-262-01 mitigated by prose content; T-262-02 mitigated by drift guard test).

## Self-Check: PASSED

- FOUND: ferro-mcp/src/tools/generation_context.rs
- FOUND: .planning/phases/262-mcp-catalog-docs-publish/262-01-SUMMARY.md
- FOUND: commit 9c6cd8b9
