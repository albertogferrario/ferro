---
phase: 195-close-the-loop-by-default
plan: "03"
subsystem: ferro-mcp
tags: [mcp, projection, checkpoint, generators, inline-hook, async]
dependency_graph:
  requires: [195-01, 195-02]
  provides: [195-04]
  affects: [ferro-mcp/src/tools/generate_projection.rs, ferro-mcp/src/tools/json_ui_generate.rs, ferro-mcp/src/service.rs]
tech_stack:
  added: []
  patterns:
    - Option<VerdictSummary> embedded in generator result structs with skip_serializing_if
    - speculative checkpoint anchor derived as {model_lowercase}_service
    - safe degradation via .ok().map(|v| v.summary()) — Err maps to None, not failure
    - None-guard on json_ui_generate: model=None skips run_for entirely (SC-1)
key_files:
  created: []
  modified:
    - ferro-mcp/src/tools/generate_projection.rs
    - ferro-mcp/src/tools/json_ui_generate.rs
    - ferro-mcp/src/service.rs
decisions:
  - generate_projection embeds checkpoint only when projection resolves; omits field (skip_serializing_if) on first run
  - json_ui_generate skips run_for entirely when model=None to avoid vacuous all-not_checked summary (SC-1 / Pitfall 3)
  - One-way dependency preserved: generators import checkpoint_projection; checkpoint_projection has no reverse import
metrics:
  duration: ~20min
  completed: "2026-06-10"
  tasks: 3
  files_modified: 3
requirements: [CHK-07]
---

# Phase 195 Plan 03: Close the Loop by Default Summary

Generators embed a compact `VerdictSummary` immediately after generating, so an agent receives field→column seam verification without a separate checkpoint call. Two generators wired, service handlers awaited, one-way dependency preserved.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | generate_projection async execute + VerdictSummary field | ac2d09e9 | generate_projection.rs, service.rs |
| 2 | json_ui_generate async execute + speculative model checkpoint | 581ae931 | json_ui_generate.rs, service.rs |
| 3 | Await generator handlers in service.rs + update tool descriptions | 871bb683 | service.rs, generate_projection.rs, json_ui_generate.rs |

## What Was Built

### generate_projection

- `GenerateProjectionResult` gains `checkpoint: Option<crate::tools::checkpoint_projection::VerdictSummary>` with `#[serde(skip_serializing_if = "Option::is_none")]`.
- `execute` made async; after the existing generation logic, derives anchor `{model_name.to_lowercase()}_service`, calls `checkpoint_projection::run_for(...).await.ok().map(|v| v.summary())`.
- First-run degradation: when the projection does not yet exist `run_for` returns `Err`, `.ok()` maps to `None`, field is omitted from serialized output.
- Three tests added: `generate_projection_no_projection_omits_checkpoint`, `generate_projection_with_projection_embeds_checkpoint`, `generate_projection_result_checkpoint_skip_when_none`.

### json_ui_generate

- `JsonUiGenerationContext` gains `checkpoint: Option<crate::tools::checkpoint_projection::VerdictSummary>` after `description`, using identical `#[serde(skip_serializing_if = "Option::is_none")]` pattern.
- `execute` made async; speculative anchor logic: `match model { Some(m) => run_for(...).await.ok().map(...), None => None }`.
- `model=None` path skips `run_for` entirely — no anchor to derive, no vacuous all-`not_checked` summary embedded (SC-1 / Pitfall 3).
- Existing tests updated to `#[tokio::test]` + `.await`; struct literals in existing tests gain `checkpoint: None`.
- Three new tests: `json_ui_generate_no_model_omits_checkpoint`, `json_ui_generate_with_model_no_projection_omits_checkpoint`, `json_ui_generate_with_resolving_model_embeds_checkpoint`.

### service.rs handler updates

- `generate_projection` handler: `.await` added to `tools::generate_projection::execute(...)` call.
- `json_ui_generate` handler: `.await` added to `tools::json_ui_generate::execute(...)` call.
- `generate_projection` tool description updated: documents `checkpoint` field (status + next_steps; null/omitted on first run).
- `json_ui_generate` tool description updated: documents optional `checkpoint` when model anchor resolves.

## Acceptance Criteria

All green:

- `grep checkpoint: Option<crate::tools::checkpoint_projection::VerdictSummary> generate_projection.rs` — line 29
- `grep pub async fn execute generate_projection.rs` — line 42
- `grep checkpoint_projection::run_for generate_projection.rs` — line 99
- `grep checkpoint: Option<crate::tools::checkpoint_projection::VerdictSummary> json_ui_generate.rs` — line 35
- `grep pub async fn execute json_ui_generate.rs` — line 115
- `grep "None => None" json_ui_generate.rs` — line 134
- `grep generate_projection::execute.*\.await service.rs` — line 1654
- `grep json_ui_generate::execute service.rs` shows `.await` on next lines — lines 1359-1364
- `cargo test -p ferro-mcp`: 294 passed
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings`: clean

## Deviations from Plan

None — plan executed exactly as written.

The `test_conventions_populated`, `test_example_not_empty`, and `test_component_catalog_not_empty` tests in json_ui_generate.rs were converted from `#[test]` to `#[tokio::test]` as a necessary consequence of `execute` becoming async (Rule 3 — fix blocking issue). This was anticipated by the plan's note about updating existing tests.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes beyond what is documented in the plan's threat model. The speculative anchor `{model.to_lowercase()}_service` passes through `validate_name` inside `run_for` — invalid chars produce `Err` → `.ok()` → `None`, no path traversal reaches cache write (T-195-07 mitigated as planned).

## Self-Check

**Files exist:**
- `ferro-mcp/src/tools/generate_projection.rs` — FOUND
- `ferro-mcp/src/tools/json_ui_generate.rs` — FOUND
- `ferro-mcp/src/service.rs` — FOUND

**Commits exist:**
- ac2d09e9 — Task 1: generate_projection async execute with inline checkpoint VerdictSummary
- 581ae931 — Task 2: json_ui_generate async execute with speculative model checkpoint
- 871bb683 — Task 3: await generator handlers in service.rs + update tool descriptions

## Self-Check: PASSED
