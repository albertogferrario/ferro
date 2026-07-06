---
phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
reviewed: 2026-06-08T00:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - ferro-ai/src/schema/mod.rs
  - ferro-ai/src/complete.rs
  - ferro-ai/src/tools/mod.rs
  - ferro-ai/src/client/mod.rs
  - ferro-ai/src/client/anthropic.rs
  - ferro-ai/src/client/openai.rs
  - ferro-ai/src/client/ollama.rs
  - ferro-ai/src/classifier/anthropic.rs
  - ferro-ai/src/error.rs
  - ferro-ai/src/lib.rs
  - ferro-ai/tests/projection_schema.rs
  - ferro-ai/Cargo.toml
findings:
  critical: 3
  warning: 3
  info: 2
  total: 8
status: issues_found
---

# Phase 166: Code Review Report

**Reviewed:** 2026-06-08T00:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

The schema normalizer (`for_structured_output`), the `complete::<T>()` entry point, and the
`ToolRegistry` dispatch loop structure are all correct in isolation. The projection-enum closing
pass handles both schemars shapes (FieldMeaning enum-array vs Intent const-per-variant), the
cycle guard is sound, and the iteration cap has no bypass path.

Three critical bugs exist entirely in the tool-calling wire format layer — they affect real
provider API calls and will produce 4xx errors at runtime. They do not compromise the schema
normalizer or the iteration-safety guarantee, but they make `dispatch` non-functional against
live Anthropic and OpenAI endpoints.

## Critical Issues

### CR-01: Anthropic tool_result messages use wrong wire format

**File:** `ferro-ai/src/client/anthropic.rs:66-73` and `ferro-ai/src/tools/mod.rs:185-197`

**Issue:** `build_body` maps `Role::Tool` to `{"role":"user","content":"[tool_use_id:id] text"}`.
The Anthropic Messages API rejects this. Tool results must be structured content blocks:
`{"role":"user","content":[{"type":"tool_result","tool_use_id":"<id>","content":"<text>"}]}`.
The `result_to_message` helper encodes the id into the content string as `[tool_use_id:id] text`,
but `build_body` never parses this encoding — it passes the raw string as the `content` field.

**Fix:**

Option A — handle `Role::Tool` specially in `build_body` by parsing the encoded id prefix:
```rust
// In build_body, replace the uniform message mapping:
Role::Tool => {
    // Content is encoded as "[tool_use_id:ID] BODY" by result_to_message.
    let (tool_use_id, body) = parse_tool_result_content(&m.content);
    return serde_json::json!({
        "role": "user",
        "content": [{"type": "tool_result", "tool_use_id": tool_use_id, "content": body}]
    });
}
```

Option B (cleaner) — give `Message` a structured content variant instead of encoding into
a string, so providers can serialize correctly without string parsing.

### CR-02: Assistant tool_use message missing from conversation history

**File:** `ferro-ai/src/tools/mod.rs:249-254`

**Issue:** When `complete_with_tools` returns `CompletionResponse::ToolUse(blocks)`, the dispatch
loop pushes only tool result messages — it never records the assistant's tool_use response in the
message history. Both Anthropic and OpenAI require the assistant's tool_use/tool_calls block to
appear in the conversation before the corresponding tool_result messages. Sending tool results
without the preceding assistant turn violates alternating-role requirements and causes API errors
on the next iteration.

The `CompletionResponse::ToolUse(blocks)` variant carries only the parsed `ToolUseBlock` list,
discarding the raw assistant content. The raw response is needed to reconstruct the assistant
message.

```rust
// Current (broken):
CompletionResponse::ToolUse(blocks) => {
    for block in &blocks {
        let result = self.call_tool(block).await;
        messages.push(Self::result_to_message(&block.id, result));
    }
}

// Required shape (Anthropic):
// 1. Push assistant message with tool_use content blocks
// 2. Then push tool_result user message
messages.push(Message {
    role: Role::Assistant,
    content: /* raw assistant content array as string, or restructure CompletionResponse */,
});
for block in &blocks {
    let result = self.call_tool(block).await;
    messages.push(Self::result_to_message(&block.id, result));
}
```

**Fix:** Extend `CompletionResponse::ToolUse` to carry the raw assistant content alongside the
parsed blocks, or add a new `assistant_raw: String` field, so the dispatch loop can reconstruct
the assistant message before appending tool results.

### CR-03: OpenAI tool result messages missing `tool_call_id` field

**File:** `ferro-ai/src/client/openai.rs:68-80` and `ferro-ai/src/tools/mod.rs:185-197`

**Issue:** OpenAI's Chat Completions API requires tool result messages to include a `tool_call_id`
field: `{"role":"tool","tool_call_id":"call_xxx","content":"result text"}`. The current code
maps `Role::Tool` to `{"role":"tool","content":"[tool_use_id:id] text"}` — the id is encoded
inside the content string but never extracted as a separate `tool_call_id` field. The OpenAI API
will return a 400 error for any request that includes a `role: "tool"` message without
`tool_call_id`.

**Fix:** Same structural fix as CR-01 — either parse the id prefix in `build_body` for the
OpenAI case:
```rust
Role::Tool => {
    let (call_id, body) = parse_tool_result_content(&m.content);
    serde_json::json!({
        "role": "tool",
        "tool_call_id": call_id,
        "content": body,
    })
}
```
or switch to a structured `Message` content type to avoid encoding/decoding round-trips.

## Warnings

### WR-01: OpenAI `tool_choice` field ignores `ToolChoice::None`

**File:** `ferro-ai/src/client/openai.rs:116`

**Issue:** `build_body` unconditionally sets `tool_choice = "auto"` whenever `tools` is present,
regardless of `request.tool_choice`. A caller setting `tool_choice: Some(ToolChoice::None)` to
suppress tool invocation is silently ignored. Anthropic correctly implements both variants
(`auto` / `none`); OpenAI does not.

```rust
// Current: ignores request.tool_choice for OpenAI
body["tool_choice"] = serde_json::json!("auto");

// Fix: respect the field
if let Some(choice) = &request.tool_choice {
    body["tool_choice"] = match choice {
        ToolChoice::Auto => serde_json::json!("auto"),
        ToolChoice::None => serde_json::json!("none"),
    };
}
```

### WR-02: `warn@5` fires even when `max_iterations` equals 5 — dead branch

**File:** `ferro-ai/src/tools/mod.rs:222-235`

**Issue:** The dispatch loop is `for iteration in 0..=self.max_iterations`. When
`max_iterations == 5`, iteration 5 hits the cap check first (`iteration == self.max_iterations`)
and returns `Err` before reaching `if iteration == 5`. The advisory warning is never emitted
for any `max_iterations <= 5`. This is a logic ordering issue: the two guards should be checked
in documentation order (warn first, cap second), or the warn threshold should be documented as
"only fires when `max_iterations > 5`".

```rust
// Fix: swap order so warn fires before cap check on the same iteration
for iteration in 0..=self.max_iterations {
    if iteration == 5 && self.max_iterations > 5 {
        warn!(iteration, max = self.max_iterations, "tool dispatch at iteration 5");
    }
    if iteration == self.max_iterations {
        error!(...);
        return Err(Error::ToolIterationLimit(self.max_iterations));
    }
    // ... rest of loop
}
```

### WR-03: `Error::ToolNotFound` is defined but never constructed

**File:** `ferro-ai/src/error.rs:50`

**Issue:** `Error::ToolNotFound(String)` is a public API variant. The dispatch loop deliberately
does not use it — unknown tool names are surfaced to the LLM as a `ToolError` message (correct
per D-13/SC#6). But the dead variant remains in the public error type, misleading callers who
might pattern-match on it expecting it to be reachable. Either remove it or document precisely
when it would be returned (e.g., a future `dispatch_single` helper).

## Info

### IN-01: `make_handler` not re-exported from crate root

**File:** `ferro-ai/src/lib.rs:67`

**Issue:** `make_handler` is the documented ergonomic entry point for building `ToolDef` handlers
(the doc example in `tools/mod.rs` shows `use ferro_ai::tools::{make_handler, ToolDef, ...}`),
but it is not included in the top-level `pub use tools::{ToolDef, ToolError, ToolRegistry}`.
Callers must import it as `ferro_ai::tools::make_handler`. The three sibling items (`ToolDef`,
`ToolError`, `ToolRegistry`) are all re-exported at the crate root; consistency suggests
`make_handler` should be too.

**Fix:** Add `make_handler` to the re-export in `lib.rs`:
```rust
pub use tools::{make_handler, ToolDef, ToolError, ToolRegistry};
```

### IN-02: `PROJECTION_DEF_NAMES` activates the closing path for types that are not closed

**File:** `ferro-ai/src/schema/mod.rs:52-60`

**Issue:** `PROJECTION_DEF_NAMES` lists `"Cardinality"`, `"ActionDef"`, `"GuardDef"`, and
`"StateDef"` as trigger names for `has_projection_defs`. If any of these appear in `$defs`,
the closing pass activates — but `close_projection_enum` is only called for `"FieldMeaning"`
and `"Intent"`. The other names are trigger-only: they cause the pass to run, but nothing
happens to their definitions. This is currently harmless (closing `FieldMeaning`/`Intent` is
idempotent whether or not the other types are present), but the mismatch between the trigger
list and the actual closed types is a documentation gap that could mislead future maintainers
adding new projection types.

**Fix:** Document the distinction explicitly in `PROJECTION_DEF_NAMES` or add a comment in
`for_structured_output` explaining that the trigger list is a superset of the closed list by
design (to activate the pass for any schema containing projection types, even if only
FieldMeaning/Intent need closing in that specific schema).

---

_Reviewed: 2026-06-08T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
