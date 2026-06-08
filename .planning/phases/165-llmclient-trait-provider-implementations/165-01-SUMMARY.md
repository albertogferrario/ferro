---
phase: 165-llmclient-trait-provider-implementations
plan: "01"
subsystem: ferro-ai
tags: [llm, trait, error, streaming, dependencies]
dependency_graph:
  requires: []
  provides:
    - ferro-ai/src/client/mod.rs (LlmClient trait, CompletionRequest, Message, Role, TokenStream)
    - ferro-ai/src/error.rs (Error::Provider struct variant, Error::Unsupported, Error::is_retryable)
    - ferro-ai/Cargo.toml (reqwest stream, reqwest-eventsource, futures, async-stream)
  affects:
    - ferro-ai/src/classifier/mod.rs (retry arm updated to is_retryable, ClassifierConfig::default empty model)
    - ferro-ai/src/classifier/anthropic.rs (Provider error construction updated, dead helpers removed)
    - ferro-ai/src/lib.rs (pub mod client added)
tech_stack:
  added:
    - reqwest-eventsource 0.6 (SSE parsing for Anthropic + OpenAI streaming)
    - futures 0.3 (BoxStream type alias, StreamExt)
    - async-stream 0.3 (try_stream! macro for Ollama NDJSON)
  patterns:
    - async_trait object-safe trait (mirrors ClassificationProvider pattern)
    - BoxStream<'static, Result<String, Error>> as TokenStream type alias
    - Error struct variant with Option<u16> status for retry logic
key_files:
  created:
    - ferro-ai/src/client/mod.rs
    - ferro-ai/src/client/anthropic.rs (empty stub for Plan 02)
    - ferro-ai/src/client/openai.rs (empty stub for Plan 02)
    - ferro-ai/src/client/ollama.rs (empty stub for Plan 03)
  modified:
    - ferro-ai/Cargo.toml
    - ferro-ai/src/error.rs
    - ferro-ai/src/lib.rs
    - ferro-ai/src/classifier/mod.rs
    - ferro-ai/src/classifier/anthropic.rs
decisions:
  - "TokenStream defined as BoxStream<'static, Result<String, Error>> — hides reqwest-eventsource from public API (D-09)"
  - "ClassifierConfig::default().model is now String::new() — resolved from client.default_model() at call time (D-03)"
  - "Error::Provider restructured to struct variant with Option<u16> status — enables status-based is_retryable() (D-14)"
  - "is_permanent_provider_error string-sniff deleted — replaced by is_retryable() on Error enum"
  - "Empty provider stubs compile as valid Rust modules — Wave 2 plans edit only their own files"
metrics:
  duration: 286s
  completed: "2026-06-08T02:01:30Z"
  tasks_completed: 3
  files_modified: 9
---

# Phase 165 Plan 01: LlmClient Trait Foundation Summary

LlmClient trait with CompletionRequest/Message/Role/TokenStream types, restructured Error enum with status-based retry logic, and empty provider module skeleton wired into ferro-ai.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add streaming dependencies | 05eb1ce0 | ferro-ai/Cargo.toml |
| 2 (RED) | Failing test for Error restructuring | fc59bb98 | ferro-ai/src/error.rs |
| 2 (GREEN) | Error enum restructured + classifier updated | b9a54cab | error.rs, classifier/mod.rs, classifier/anthropic.rs |
| 3 | LlmClient trait + client module skeleton | b2158bc7 | client/mod.rs, client/anthropic.rs, client/openai.rs, client/ollama.rs, lib.rs, classifier/anthropic.rs |

## Verification Results

- `cargo build -p ferro-ai` — green, no warnings
- `cargo test -p ferro-ai test_error_is_retryable` — 1 passed
- `cargo test -p ferro-ai` — 22 passed, 0 failed
- `cargo metadata --format-version 1 -q` — deps resolve

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Dead Code] Removed is_permanent_error / is_transient_error from classifier/anthropic.rs**
- **Found during:** Task 3 build (2 dead_code warnings)
- **Issue:** After D-14 restructured `Error::Provider` and replaced `is_permanent_provider_error` string-sniff with `is_retryable()`, the `is_permanent_error(status: u16)` and `is_transient_error(status: u16)` helper functions in `classifier/anthropic.rs` became unreachable dead code. Their associated tests also referenced deleted symbols.
- **Fix:** Deleted both functions and their two test cases from `classifier/anthropic.rs`. The equivalent behavior now lives in `Error::is_retryable()` in `error.rs`.
- **Files modified:** `ferro-ai/src/classifier/anthropic.rs`
- **Commit:** b9a54cab

**2. [Rule 1 - Compile Error] Updated classifier/mod.rs retry arm and test helpers for Provider struct variant**
- **Found during:** Task 2 GREEN phase (8 compile errors)
- **Issue:** After restructuring `Error::Provider(String)` → `Error::Provider { status, message }`, all construction sites in `classifier/mod.rs` (retry arm pattern match + two test helpers using tuple syntax) and `classifier/anthropic.rs` (HTTP error mapping) produced compile errors.
- **Fix:** Updated retry arm to `Err(e) if !e.is_retryable()` per PATTERNS.md; updated test helpers to `Error::Provider { status: Some(500), message: "..." }` / `{ status: Some(401), ... }`; updated anthropic.rs HTTP error mapping to struct variant.
- **Files modified:** `ferro-ai/src/classifier/mod.rs`, `ferro-ai/src/classifier/anthropic.rs`
- **Commit:** b9a54cab

**3. [Rule 1 - Test Fix] Updated test_build_request_body_contains_output_config for D-03**
- **Found during:** Task 2 GREEN phase
- **Issue:** `test_build_request_body_contains_output_config` used `ClassifierConfig::default()` and asserted `body["model"] == "claude-sonnet-4-6"`. After D-03 changed `ClassifierConfig::default().model` to `String::new()`, the assertion would fail.
- **Fix:** Changed the test config to `ClassifierConfig { model: "claude-sonnet-4-6".to_string(), ..Default::default() }` — explicit model, not relying on the default.
- **Files modified:** `ferro-ai/src/classifier/anthropic.rs`
- **Commit:** b9a54cab

## Known Stubs

- `ferro-ai/src/client/anthropic.rs` — intentional stub: `// AnthropicClient implemented in Plan 02`
- `ferro-ai/src/client/openai.rs` — intentional stub: `// OpenAiClient implemented in Plan 02`
- `ferro-ai/src/client/ollama.rs` — intentional stub: `// OllamaClient implemented in Plan 03`

These are load-bearing empty modules required for `pub mod` declarations to resolve. Plan 02 and Plan 03 fill them in.

## Threat Flags

No new security-relevant surface introduced. `Error::Provider.message` doc-comment explicitly states it must not contain the API key or auth header (T-165-01 mitigation in place).

## Self-Check: PASSED
