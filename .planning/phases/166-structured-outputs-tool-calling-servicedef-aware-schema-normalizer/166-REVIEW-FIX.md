---
phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
fixed_at: 2026-06-08T05:00:00Z
review_path: .planning/phases/166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer/166-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 166: Code Review Fix Report

**Fixed at:** 2026-06-08T05:00:00Z
**Source review:** `.planning/phases/166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer/166-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 8
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: Anthropic tool_result messages use wrong wire format

**Files modified:** `ferro-ai/src/client/mod.rs`, `ferro-ai/src/client/anthropic.rs`, `ferro-ai/src/tools/mod.rs`, `ferro-ai/src/complete.rs`, `ferro-ai/src/classifier/anthropic.rs`, `ferro-ai/src/client/openai.rs`
**Commit:** `cdd02259`
**Applied fix:** Added `tool_call_id: Option<String>` to `Message` so the provider call id is carried as structured data. In `anthropic.rs` `build_body`, `Role::Tool` messages now emit `{"role":"user","content":[{"type":"tool_result","tool_use_id":"<id>","content":"<text>"}]}` using `m.tool_call_id` as the real `tool_use_id` field. Updated `result_to_message` to store `block_id` in `tool_call_id` instead of encoding it into the content string. Updated all `Message` construction sites across the codebase (`complete.rs`, `classifier/anthropic.rs`, both client test modules) to include `tool_call_id: None`. Added regression test `test_build_body_tool_result_wire_format` in `client/anthropic.rs`.

### CR-02: Assistant tool_use message missing from conversation history

**Files modified:** `ferro-ai/src/client/mod.rs`, `ferro-ai/src/client/anthropic.rs`, `ferro-ai/src/client/openai.rs`, `ferro-ai/src/tools/mod.rs`
**Commit:** `cdd02259`
**Applied fix:** Extended `CompletionResponse::ToolUse` from a tuple variant to a struct variant with `blocks: Vec<ToolUseBlock>` and `assistant_content: String`. Both `anthropic.rs` and `openai.rs` `complete_with_tools` now populate `assistant_content` from the raw response (`json["content"].to_string()` for Anthropic, `json["choices"][0]["message"]["tool_calls"].to_string()` for OpenAI). The dispatch loop in `tools/mod.rs` now pushes `Message { role: Role::Assistant, content: assistant_content, .. }` into history BEFORE appending tool result messages, satisfying the alternating-role requirement. Updated the `LoopingClient` mock in tests to construct the new struct variant. Added regression test `dispatch_includes_assistant_turn_before_tool_results` verifying ordering and that `tool_call_id` is a structured field.

### CR-03: OpenAI tool result messages missing `tool_call_id` field

**Files modified:** `ferro-ai/src/client/openai.rs`, `ferro-ai/src/tools/mod.rs`, `ferro-ai/src/client/mod.rs`
**Commit:** `cdd02259`
**Applied fix:** In `openai.rs` `build_body`, `Role::Tool` messages now emit `{"role":"tool","tool_call_id":"<id>","content":"<text>"}` using `m.tool_call_id.as_deref()` as the real `tool_call_id` field. The id is no longer embedded in the content string. Added regression tests `test_build_body_tool_result_wire_format` (verifies `tool_call_id` is a top-level field and not in content) in `client/openai.rs`.

### WR-01: OpenAI `tool_choice` field ignores `ToolChoice::None`

**Files modified:** `ferro-ai/src/client/openai.rs`
**Commit:** `cdd02259`
**Applied fix:** Replaced the hardcoded `body["tool_choice"] = json!("auto")` with a `match` on `request.tool_choice`: `ToolChoice::None` emits `"none"`, `ToolChoice::Auto` and `None` both emit `"auto"`. Added import for `ToolChoice` in `openai.rs`. Added regression tests `test_build_body_tool_choice_none` and `test_build_body_tool_choice_auto` covering all three paths.

### WR-02: `warn@5` fires even when `max_iterations` equals 5 — dead branch

**Files modified:** `ferro-ai/src/tools/mod.rs`
**Commit:** `cdd02259`
**Applied fix:** Swapped the order of the two guards in the dispatch loop: the `iteration == 5 && self.max_iterations > 5` warn check now fires before the `iteration == self.max_iterations` cap check. When `max_iterations == 5`, iteration 5 hits the cap (correct — no infinite loop), but the warn guard's `self.max_iterations > 5` condition correctly prevents a false warning. The required cap is preserved with no bypass path (SC#5 intact).

### WR-03: `Error::ToolNotFound` is defined but never constructed

**Files modified:** `ferro-ai/src/error.rs`
**Commit:** `81864ff9`
**Applied fix:** Added a detailed doc comment on `Error::ToolNotFound` explaining that it is not currently constructed by `ToolRegistry::dispatch` (which surfaces unknown tool names to the LLM as `ToolError` messages per D-13/SC#6), and documenting it as reserved for future direct-dispatch helpers. Also added a regression test `dispatch_surfaces_unknown_tool_as_tool_error` confirming dispatch completes (not aborts) for unregistered tools and surfaces the error as a model-legible message.

### IN-01: `make_handler` not re-exported from crate root

**Files modified:** `ferro-ai/src/lib.rs`
**Commit:** `64fccabf`
**Applied fix:** Added `make_handler` to the `pub use tools::{...}` re-export line, making it available at `ferro_ai::make_handler` alongside `ToolDef`, `ToolError`, and `ToolRegistry`.

### IN-02: `PROJECTION_DEF_NAMES` activates the closing path for types that are not closed

**Files modified:** `ferro-ai/src/schema/mod.rs`
**Commit:** `9bdfce22`
**Applied fix:** Replaced the single-line doc comment on `PROJECTION_DEF_NAMES` with a detailed multi-line comment that explicitly distinguishes the trigger list (all 7 names) from the closed list (`FieldMeaning` and `Intent` only). The comment explains why the other names are trigger-only (already closed by schemars, or a struct), and provides a maintenance guide for adding new projection enum types with `Custom` escape hatches.

---

_Fixed: 2026-06-08T05:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
