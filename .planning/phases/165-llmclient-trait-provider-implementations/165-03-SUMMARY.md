---
phase: 165-llmclient-trait-provider-implementations
plan: "03"
subsystem: ferro-ai
tags: [llm, ollama, ndjson, streaming, embeddings, tdd]
dependency_graph:
  requires: [165-01, 165-02]
  provides: [OllamaClient, parse_ollama_line, parse_ollama_embedding]
  affects: [ferro-ai/src/client/ollama.rs]
tech_stack:
  added: [async-stream 0.3 try_stream!]
  patterns: [NDJSON streaming via bytes_stream, pub(crate) parse helpers for unit testing]
key_files:
  modified: [ferro-ai/src/client/ollama.rs]
decisions:
  - NDJSON streaming via bytes_stream + async_stream::try_stream! (not reqwest-eventsource)
  - base_url defaults to http://localhost:11434 with no auth header
  - system prompt injected as role:system message before conversation messages
  - pub(crate) parse helpers enable offline unit testing without a live server
metrics:
  duration: ~10 minutes
  completed: 2026-06-08
  tasks_completed: 1
  files_modified: 1
---

# Phase 165 Plan 03: OllamaClient Implementation Summary

OllamaClient implementing LlmClient with NDJSON streaming (not SSE), no-auth local default, and embeddings via /api/embed.

## What Was Built

`ferro-ai/src/client/ollama.rs` — full `OllamaClient` implementing `LlmClient`:

- **Struct:** `{ client: reqwest::Client, model: Option<String>, base_url: String }` — no api_key field; 60s timeout (T-165-04).
- **`default_model()`** returns `"llama3.1"` or model override.
- **`complete()`** POSTs to `{base_url}/api/chat` with `stream:false`, extracts `message.content`.
- **`complete_stream()`** POSTs with `stream:true`, takes `response.bytes_stream()`, wraps in `Box::pin(try_stream! { ... })` — line-delimited NDJSON parse (NOT SSE; never `.eventsource()`).
- **`embed()`** POSTs to `{base_url}/api/embed` with `{model, input}`, extracts `embeddings[0]` as `Vec<f32>`.
- **`parse_ollama_line()`** — `pub(crate)` helper: parses one NDJSON line → `(Option<String>, bool)` (token, done).
- **`parse_ollama_embedding()`** — `pub(crate)` helper: extracts `embeddings[0]` from `/api/embed` response.

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED (test) | `3d0ec886` | Failing tests committed before any implementation |
| GREEN (feat) | `fa4a5cce` | All 8 tests passing after implementation |

## Tests

8 unit tests, all passing:
- `test_ollama_default_model` — default resolves to `"llama3.1"`
- `test_ollama_model_override` — override `"mistral"` is respected
- `test_ollama_default_base_url` — base_url is `"http://localhost:11434"` when None
- `test_parse_ollama_line_token` — non-empty content line → `(Some("The"), false)`
- `test_parse_ollama_line_done` — empty content + `done:true` → `(None, true)`
- `test_parse_ollama_embedding` — extracts `[0.1, -0.2]` with float precision
- `test_parse_ollama_embedding_missing` — empty array → `Err(Deserialization(...))`
- `test_ollama_is_object_safe` — `Box<dyn LlmClient>` instantiation compiles

## Acceptance Criteria

- `impl LlmClient for OllamaClient` — present
- `"llama3.1"` default model — present
- `/api/chat` and `/api/embed` endpoints — present
- `try_stream!` and `bytes_stream()` — present (NDJSON streaming)
- No `eventsource` call — the word appears only in warning comments, never as a method call
- `cargo test -p ferro-ai client::ollama` — 8/8 green
- `cargo clippy -p ferro-ai --all-targets -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- `cargo test --all-features -p ferro-ai` — 52/52 green

## Commits

| Hash | Type | Description |
|------|------|-------------|
| `3d0ec886` | test | add failing tests for OllamaClient (RED) |
| `fa4a5cce` | feat | implement OllamaClient with NDJSON streaming and embeddings |

## Deviations from Plan

None — plan executed exactly as written.

`build_body` signature was reformatted to satisfy `cargo fmt` (line length); no logic change.

## Threat Surface Scan

No new trust boundaries introduced beyond what the plan's threat model covers:
- `T-165-03` (SSRF via base_url): mitigated — base_url is operator-configured, default is loopback.
- `T-165-04` (DoS/stream timeout): mitigated — 60s reqwest client timeout applied.
- `T-165-06` (info disclosure): mitigated — no API key exists; errors carry only `e.to_string()` and provider response text.

## Self-Check: PASSED

- `ferro-ai/src/client/ollama.rs` — FOUND
- commit `3d0ec886` (RED) — FOUND
- commit `fa4a5cce` (GREEN) — FOUND
