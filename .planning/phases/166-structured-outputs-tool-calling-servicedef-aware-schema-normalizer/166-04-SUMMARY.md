---
phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
plan: "04"
subsystem: ferro-ai
tags: [tool-calling, tool-registry, tooldef, toolerror, dispatch-loop, max-iterations, sc4, sc5, sc6, d-14, wave-4]
dependency_graph:
  requires: [166-01, 166-02, 166-03]
  provides: [tool-calling-layer, tooldef, toolerror, toolregistry, dispatch-loop, complete_with_tools, sc4, sc5, sc6, aisdk-03]
  affects: [ferro-ai]
tech_stack:
  added: []
  patterns: [boxfuture-async-handler, bounded-dispatch-loop, model-legible-error-boundary, tool-use-wire-format]
key_files:
  created: [ferro-ai/src/tools/mod.rs]
  modified:
    - ferro-ai/src/client/mod.rs
    - ferro-ai/src/client/anthropic.rs
    - ferro-ai/src/client/openai.rs
    - ferro-ai/src/client/ollama.rs
    - ferro-ai/src/complete.rs
    - ferro-ai/src/classifier/anthropic.rs
    - ferro-ai/src/lib.rs
decisions:
  - "complete_with_tools added as a separate LlmClient trait method with default Err(Unsupported) — existing complete() callers unaffected, Ollama inherits the default (D-14, A3)"
  - "Role::Tool maps to 'user' in Anthropic wire format and 'tool' in OpenAI/Ollama — provider-specific translation in build_body"
  - "Unknown tool names surfaced to LLM as model-recoverable ToolError strings, not Error::ToolNotFound — lets model adapt its tool selection without aborting the loop"
  - "ToolRegistry::dispatch loop range is 0..=max_iterations; the hard cap check is at iteration==max_iterations before calling complete_with_tools — guarantees no more than max_iterations provider calls"
metrics:
  duration: "~453 seconds (~8 minutes)"
  completed: "2026-06-08T06:25:00Z"
  tasks_completed: 3
  tasks_total: 3
  files_created: 1
  files_modified: 7
---

# Phase 166 Plan 04: Tool Calling Layer — ToolDef/ToolError/ToolRegistry + complete_with_tools Summary

The tool-calling plan: `LlmClient` extended with `complete_with_tools` (D-14, single HTTP source), `ToolDef`/`ToolError`/`ToolRegistry` with hard `max_iterations` guard, and the bounded dispatch loop proving SC#4 (ToolDef shape), SC#5 (max_iterations required + enforced, warn@5/error@cap), and SC#6 (ToolError model-legible boundary). No live network in CI — all tests use mock clients.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend client layer for tool support (D-14) | e02f96f4 | ferro-ai/src/client/mod.rs, anthropic.rs, openai.rs, ollama.rs, complete.rs, classifier/anthropic.rs |
| 2 | ToolError + ToolDef + ToolRegistry construction (SC#4, SC#6) | d31c5170 | ferro-ai/src/tools/mod.rs, ferro-ai/src/lib.rs |
| 3 | ToolRegistry::dispatch loop + max_iterations enforcement (SC#5) + rustfmt | 0cb33137 | ferro-ai/src/tools/mod.rs, client files, lib.rs |

## Decisions Made

**complete_with_tools as a separate trait method (D-14):** Added as a default `Err(Error::Unsupported)` method on `LlmClient`. Existing `complete()` callers (classifier, `complete::<T>()`) are completely unaffected. Ollama inherits the Unsupported default (A3 — tool calling in non-streaming Ollama deferred). This is the cleanest separation per RESEARCH Open Question 1 recommendation.

**Role::Tool wire format:** The `Role::Tool` variant in `build_body` maps to `"user"` for Anthropic (consistent with Anthropic's wire format where tool results are sent as `role: "user"` with `type: "tool_result"` content) and `"tool"` for OpenAI/Ollama. The dispatch loop's `result_to_message` includes the `tool_use_id` reference in the content string so downstream provider adapters can extract it.

**Unknown tool handling:** When the LLM requests an unknown tool name, the dispatch loop surfaces it to the LLM as a `ToolError { message: "tool '...' is not registered" }` — a model-recoverable error string. `Error::ToolNotFound` is reserved for genuinely unrecoverable internal mismatches. This keeps the dispatch loop running so the model can select a registered tool.

**Dispatch loop cap boundary:** The loop runs `for iteration in 0..=max_iterations`. At `iteration == max_iterations` (before any provider call), the hard cap fires. This guarantees the provider is called at most `max_iterations` times.

**CompletionRequest literals (cross-wave obligation):** All `CompletionRequest { ... }` struct literals across the crate updated with `tools: None, tool_choice: None` per the Plan 03 documented obligation. Affected: `complete.rs`, `classifier/anthropic.rs` (production + 2 tests), `client/anthropic.rs` (3 tests), `client/openai.rs` (2 tests).

## Verification Results

- `cargo test -p ferro-ai tool` — 6/6 green:
  - `tool_def_construction` (SC#4)
  - `tool_error_is_model_legible` (SC#6)
  - `tool_registry_requires_max_iterations`
  - `tool_registry_enforces_max_iterations` (SC#5, T-166-01)
  - `dispatch_returns_on_text`
  - `dispatch_surfaces_tool_error` (SC#6 end-to-end, T-166-02)
- `cargo test -p ferro-ai` — 74 unit tests + 6 integration tests green (no regression)
- `cargo test -p ferro-ai classifier` — 8/8 green (no regression)
- `cargo clippy --all --all-targets -- -D warnings` — clean (full workspace)
- `cargo fmt --all -- --check` — clean
- `cargo build -p ferro-ai` — exits 0 (all CompletionRequest literals compile)

## Deviations from Plan

### Auto-fixed Issues

**[Rule 1 - Bug] Role enum non-exhaustive match in build_body**
- **Found during:** Task 1 — after adding `Role::Tool` variant, the existing `match m.role { Role::User => "user", Role::Assistant => "assistant" }` match arms in all three `build_body` implementations became non-exhaustive.
- **Issue:** Clippy would have flagged this as a non-exhaustive pattern match warning (-D warnings = build failure).
- **Fix:** Extended all three `build_body` role match arms: Anthropic maps `Role::Tool` to `"user"` (wire format); OpenAI and Ollama map `Role::Tool` to `"tool"`.
- **Files modified:** ferro-ai/src/client/anthropic.rs, openai.rs, ollama.rs
- **Commit:** e02f96f4

**[Rule 2 - Missing] `parse_anthropic_tool_use_blocks` and `parse_openai_tool_calls` helpers**
- **Found during:** Task 1 — implementing `complete_with_tools` required parsing provider-specific tool-use response shapes into the common `ToolUseBlock` type.
- **Issue:** Plan mentioned the parsing but didn't specify it as a separate named function; adding it as `pub(crate)` named helpers follows the existing `parse_anthropic_delta` / `parse_openai_delta` / `parse_embedding` pattern and makes the code unit-testable.
- **Fix:** Added `parse_anthropic_tool_use_blocks(content: &Value) -> Vec<ToolUseBlock>` and `parse_openai_tool_calls(json: &Value) -> Vec<ToolUseBlock>` as `pub(crate)` functions.
- **Files modified:** ferro-ai/src/client/anthropic.rs, openai.rs
- **Commit:** e02f96f4

## Threat Mitigations Verified

| Threat | Mitigation | Test |
|--------|------------|------|
| T-166-01 (runaway loop / cost) | `max_iterations` REQUIRED at construction; no Default, no zero-arg ctor; `Error::ToolIterationLimit` at cap | `tool_registry_enforces_max_iterations` |
| T-166-02 (error info-leak to model) | `ToolError { message }` only surfaced; never raw panics, stack traces, or DB strings | `dispatch_surfaces_tool_error`, `tool_error_is_model_legible` |
| T-166-03 (untrusted LLM input) | Documented in `ToolDef.parameters_schema` rustdoc — handler implementations validate their own inputs; SDK passes raw `serde_json::Value` | rustdoc (architectural, not a test gate) |
| T-166-04 (API key in error) | Reuses existing `Error::Provider { status, message }` pattern — message carries provider response text only | existing error.rs constraint |

## Known Stubs

None — all three success criteria (SC#4, SC#5, SC#6) are fully implemented production code, not stubs. The dispatch loop is real; the ToolError boundary is enforced; the iteration cap has no override path.

## Threat Surface Scan

No new network endpoints, auth paths, or file access introduced. The `complete_with_tools` implementations reuse the existing Anthropic and OpenAI HTTP clients (same base URL, same auth header, same timeout). No new trust boundary surfaces beyond those already documented in the plan's `<threat_model>`.

## Self-Check: PASSED

- `ferro-ai/src/tools/mod.rs` contains `pub struct ToolRegistry`: confirmed
- `ferro-ai/src/tools/mod.rs` contains `pub struct ToolError`: confirmed
- `ferro-ai/src/tools/mod.rs` contains `pub struct ToolDef`: confirmed
- `ferro-ai/src/tools/mod.rs` contains `pub async fn dispatch`: confirmed
- `ferro-ai/src/tools/mod.rs` has NO `impl Default for ToolRegistry`: confirmed
- `ferro-ai/src/client/mod.rs` contains `complete_with_tools`: confirmed
- `ferro-ai/src/client/mod.rs` contains `enum CompletionResponse`: confirmed
- Commit e02f96f4 exists: confirmed
- Commit d31c5170 exists: confirmed
- Commit 0cb33137 exists: confirmed
- `cargo test -p ferro-ai` — 74+6 green: confirmed
- `cargo clippy --all --all-targets -- -D warnings` — clean: confirmed
