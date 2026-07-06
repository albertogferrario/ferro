# Issue: ferro-ai cannot stream AND call tools in one agent loop

## Summary

`ferro-ai`'s Anthropic client exposes two mutually-exclusive completion modes:

- `complete_stream` — streams text token-by-token, but **ignores `tool_use` content
  blocks entirely**. It only surfaces `content_block_delta` text deltas and closes the
  stream on `message_stop`. `StopReason::ToolUse` and `tool_use` blocks are never emitted.
- `complete_with_tools` — parses `tool_use` blocks and returns
  `CompletionResponse::ToolUse { blocks, assistant_content }`, but is **non-streaming**
  (a single `.send()` + `.json()` round-trip). The user sees nothing until the full
  response arrives.

There is no `complete_stream_with_tools`. A conversational agent that must BOTH stream
tokens to the user AND dispatch tools mid-conversation cannot be built on the `LlmClient`
trait as it stands.

## Location

`ferro-ai/src/client/anthropic.rs`

- `complete_stream` — `content_block_delta` text-only `stream::unfold`; closes on
  `message_stop`. No handling of `content_block_start` with a `tool_use` block, no
  `input_json_delta` accumulation, no `StopReason::ToolUse` surfacing.
- `complete_with_tools` — `self.client.post(...).send().await` then `resp.json()`; checks
  `json["stop_reason"] == "tool_use"` and parses blocks via
  `parse_anthropic_tool_use_blocks`. Non-streaming by construction.

## Reproduction

1. Build an agent loop that needs to stream assistant text to a WebSocket while also
   invoking tools (e.g. a chat agent over a vault tool surface).
2. Try `complete_stream`: tool calls are silently dropped — the model's `tool_use` turn
   never reaches the caller, so the agent can never dispatch a tool.
3. Fall back to `complete_with_tools`: tool dispatch works, but the user waits for the
   entire turn (including every tool round) with no streamed output.

## Expected Behavior

A streaming completion that surfaces both text deltas and tool-use intent in one stream,
so the caller can render tokens live AND react to `tool_use`:

- emit text deltas as they arrive (as `complete_stream` does today);
- accumulate `input_json_delta` fragments into the tool input;
- on `content_block_start` for a `tool_use` block + `message_delta` carrying
  `stop_reason: "tool_use"`, surface the assembled tool-use block(s) and a terminal
  `StopReason::ToolUse` to the caller so it can run the tools and continue the loop.

Proposed surface: a `complete_stream_with_tools` on `LlmClient` returning a stream of a
sum type along the lines of `StreamItem::{ Text(String), ToolUse(Vec<ToolUseBlock>),
Done(StopReason) }`. This mirrors the Anthropic SSE event shape (`content_block_start` /
`content_block_delta` with `text_delta` | `input_json_delta` / `message_delta` with
`stop_reason`).

## Actual Behavior

The two modes are disjoint: streaming OR tools, never both in one loop.

## Impact / downstream

`u` (the U knowledge-base SaaS, Phase 4 chat backend) hit this directly. The chat agent
must stream Haiku tokens to a WebSocket while dispatching a read-only vault tool surface
(search/read/backlinks/resolve) mid-turn. Because `ferro-ai` cannot do streaming-with-
tools, **u's `ConversationalRenderer` was built directly on `adk-anthropic 0.9.1`** rather
than `ferro-ai` — using `adk-anthropic`'s `AccumulatingStream` plus its `InputJsonDelta` /
`StopReason::ToolUse` content-block shape, which already supports the combined loop.

This is the adk-anthropic shape a `complete_stream_with_tools` would need to expose. u's
build does **not** depend on a ferro change for Phase 4 — it ships on adk-anthropic — but
the divergence means the agent loop lives in u instead of being reusable framework surface.
Closing this gap would let `ConversationalRenderer`-style loops migrate back onto
`ferro-ai`.

## Suggested fix

Add `complete_stream_with_tools` to the Anthropic client (and the `LlmClient` trait) that:

1. reuses the existing SSE plumbing in `complete_stream`;
2. additionally handles `content_block_start`/`content_block_delta` for `tool_use` blocks,
   accumulating `input_json_delta` fragments;
3. reads `message_delta.stop_reason` and yields a terminal item carrying any assembled
   `ToolUseBlock`s + the stop reason.

Reference implementation to mirror: `adk-anthropic 0.9.1` `AccumulatingStream` +
`ContentBlockDelta::{TextDelta, InputJsonDelta}` + `StopReason::ToolUse`.

## References

- u Phase 4 — `.planning/phases/04-chat-backend/04-RESEARCH.md` OQ-2 (recommendation:
  ship on adk-anthropic in u, file the ferro-ai gap).
- u `src/chat/renderer.rs` — the `ConversationalRenderer` stream→tool→stream loop built on
  adk-anthropic that this issue would let move onto ferro-ai.
