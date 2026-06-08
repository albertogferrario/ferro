---
phase: 165-llmclient-trait-provider-implementations
plan: "02"
subsystem: ferro-ai
tags: [llm, anthropic, openai, sse, streaming, embeddings, reqwest-eventsource]
dependency_graph:
  requires: [165-01]
  provides: [AnthropicClient, OpenAiClient, LlmClient-impls]
  affects: [ferro-ai/src/client/]
tech_stack:
  added: [reqwest-eventsource (stream::unfold SSE), parse_anthropic_delta, parse_openai_delta, parse_embedding, OpenAiDelta enum]
  patterns: [async_trait impl on concrete struct, stream::unfold for SSE, bearer_auth for OpenAI, x-api-key for Anthropic]
key_files:
  created: []
  modified:
    - ferro-ai/src/client/anthropic.rs
    - ferro-ai/src/client/openai.rs
decisions:
  - "Message import placed in #[cfg(test)] module only — clippy -D warnings rejects unused top-level imports even when used only in test code"
  - "parse_anthropic_delta and parse_openai_delta extracted as pub(crate) helpers for unit-testability without live server"
  - "OpenAiDelta enum (Done/Token/Skip) makes the [DONE] sentinel handling explicit and testable"
  - "reqwest-eventsource stays module-internal per D-09 — not pub-used anywhere"
metrics:
  duration: "450s"
  completed: "2026-06-08"
  tasks_completed: 2
  files_modified: 2
---

# Phase 165 Plan 02: Provider Implementations (Anthropic + OpenAI) Summary

AnthropicClient and OpenAiClient implementing LlmClient with real SSE streaming via reqwest-eventsource, schema-gated structured output, and full unit test coverage.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | AnthropicClient — struct, build_body, complete, complete_stream (SSE), embed→Unsupported | 6eace988 | ferro-ai/src/client/anthropic.rs |
| 2 | OpenAiClient — struct (base_url for Groq), complete, complete_stream (SSE + [DONE]), embed | 30bce94c | ferro-ai/src/client/openai.rs |

## What Was Built

### AnthropicClient (`ferro-ai/src/client/anthropic.rs`)

- `AnthropicClient { client, api_key, model }` with 60s timeout (T-165-04)
- `default_model()` returns `"claude-sonnet-4-6"` or explicit override
- `build_body()` — gates `output_config.format.type = "json_schema"` on `request.schema` being `Some` (D-11); system prompt uses ephemeral prompt caching; sets `"stream": bool`
- `complete()` — POST to `api.anthropic.com/v1/messages` with `x-api-key` + `anthropic-version: 2023-06-01` headers; extracts `content[0].text`
- `complete_stream()` — SSE via `reqwest_eventsource::RequestBuilderExt::eventsource()`; `stream::unfold` loop filtering `content_block_delta` events; terminates on `message_stop`
- `embed()` — returns `Err(Error::Unsupported)` (D-13; Anthropic has no embeddings endpoint)
- `parse_anthropic_delta(data: &str) -> Option<String>` — unit-testable delta extractor
- 10 unit tests: default model, model override, body with/without schema, stream flag, embed unsupported, delta parsing, object safety

### OpenAiClient (`ferro-ai/src/client/openai.rs`)

- `OpenAiClient { client, api_key, model, base_url }` with 60s timeout
- `default_model()` returns `"gpt-4o"` or explicit override
- `base_url` defaults to `"https://api.openai.com"`; Groq uses `"https://api.groq.com/openai"`
- `build_body()` — gates `response_format.type = "json_schema"` on schema presence; `json_schema.strict = true`
- `complete()` — POST to `{base_url}/v1/chat/completions` with `bearer_auth`; extracts `choices[0].message.content`
- `complete_stream()` — SSE via eventsource; `parse_openai_delta()` handles `[DONE]` sentinel before JSON-parsing (Pitfall 4); also terminates on non-null `finish_reason`
- `embed()` — POST to `{base_url}/v1/embeddings`, model `text-embedding-3-small`; extracts `data[0].embedding` as `Vec<f32>`
- `OpenAiDelta` enum (Done/Token/Skip) for explicit delta state
- `parse_openai_delta(data: &str) -> OpenAiDelta` and `parse_embedding(json) -> Result<Vec<f32>, Error>` — unit-testable helpers
- 12 unit tests: default model, base URLs, Groq base_url, response_format with/without schema, delta parsing (Done/Token/Skip/finish_reason), embedding parse, object safety

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] `Message` import placement**
- **Found during:** Task 1+2 clippy pass
- **Issue:** `cargo clippy -- -D warnings` flags `Message` as unused at module level because it is only used inside `#[cfg(test)]` blocks — `use super::*` in the test module does not cause the parent-level import to be counted as used
- **Fix:** Removed `Message` from top-level `use crate::client::{...}` in both files; added `use crate::client::Message;` inside each `#[cfg(test)] mod tests` block
- **Files modified:** `ferro-ai/src/client/anthropic.rs`, `ferro-ai/src/client/openai.rs`
- **Commit:** 30bce94c (included in Task 2 commit after fmt pass)

**2. [Rule 1 - Bug] `cargo fmt` reformatting**
- **Found during:** Wave verification
- **Issue:** Initial write used compact struct initialization style; `cargo fmt` reformatted struct literal fields to multi-line and function parameter lists
- **Fix:** Applied `cargo fmt --all`; no logic changes
- **Files modified:** `ferro-ai/src/client/anthropic.rs`, `ferro-ai/src/client/openai.rs`, `ferro-ai/src/classifier/anthropic.rs` (pre-existing)

## Security Verification

- T-165-02 (API key disclosure): `api_key` in `x-api-key` / `bearer_auth` headers only; never appears in error messages (error mapping uses `e.to_string()` on reqwest error and `resp.text()` on response body — neither echoes request headers)
- T-165-04 (DoS): 60s timeout on both `reqwest::Client` constructors
- T-165-05 (eventsource as public surface): `reqwest_eventsource` imported only inside provider modules; no `pub use` at any level

## Known Stubs

None. Both clients are fully implemented with real SSE streaming, request body construction, and response parsing. No placeholder values or TODO markers.

## Self-Check

### Files Exist
- `ferro-ai/src/client/anthropic.rs` — FOUND
- `ferro-ai/src/client/openai.rs` — FOUND

### Commits Exist
- `6eace988` (Task 1) — FOUND
- `30bce94c` (Task 2) — FOUND

### Tests Green
- `cargo test --all-features -p ferro-ai` — 44 passed, 0 failed

### Clippy Clean
- `cargo clippy --all --all-targets -- -D warnings` — 0 errors

## Self-Check: PASSED
