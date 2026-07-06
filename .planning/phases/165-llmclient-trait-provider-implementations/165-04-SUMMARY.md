---
phase: 165-llmclient-trait-provider-implementations
plan: "04"
subsystem: ferro-ai
tags: [llm-client, config, classifier, provider-convergence]
requirements: [AISDK-01]

dependency_graph:
  requires: [165-01, 165-02, 165-03]
  provides: [AiConfig::from_env, AnthropicProvider-adapter, classifier-convergence]
  affects: [ferro-ai/src/config.rs, ferro-ai/src/classifier/anthropic.rs, ferro-ai/src/classifier/mod.rs, ferro-ai/src/lib.rs, ferro-ai/src/client/mod.rs]

tech_stack:
  added: []
  patterns:
    - AiConfig::from_env() startup-time dispatch pattern (D-06)
    - Arc<AnthropicClient> thin adapter over ClassificationProvider (D-10)
    - ENV_LOCK Mutex for serializing env-var tests

key_files:
  created:
    - ferro-ai/src/config.rs
  modified:
    - ferro-ai/src/classifier/anthropic.rs
    - ferro-ai/src/client/mod.rs
    - ferro-ai/src/lib.rs
    - ferro-ai/src/error.rs (rustfmt style only)
    - Cargo.lock

decisions:
  - Client type re-exports added to client/mod.rs (AnthropicClient, OpenAiClient, OllamaClient) to support flat lib.rs re-exports
  - ENV_LOCK static Mutex used in config tests to serialize process-wide env-var mutations
  - ANTHROPIC_API_KEY fallback kept in AiConfig::from_env for backward compatibility (secondary to FERRO_AI_API_KEY)

metrics:
  duration: "~24 minutes"
  completed: "2026-06-08T02:42:26Z"
  tasks_completed: 3
  files_modified: 6
---

# Phase 165 Plan 04: Classifier Convergence & Public Exports Summary

AiConfig::from_env() factory dispatching anthropic/openai/groq/ollama to Box<dyn LlmClient> at startup, AnthropicProvider rewired as thin adapter over AnthropicClient (duplicated HTTP deleted), and public re-exports added.

## What Was Built

**Task 1 — AiConfig::from_env() factory + lib.rs re-exports (SC#3, SC#6)**

Created `ferro-ai/src/config.rs` with `AiConfig::from_env()` reading `FERRO_AI_PROVIDER`, `FERRO_AI_MODEL`, `FERRO_AI_API_KEY`, `FERRO_AI_BASE_URL`. Dispatches:
- `"anthropic"` → `AnthropicClient` (key from `FERRO_AI_API_KEY`, fallback `ANTHROPIC_API_KEY`)
- `"openai"` → `OpenAiClient` (key required)
- `"groq"` → `OpenAiClient` with `https://api.groq.com/openai` base URL (key required)
- `"ollama"` → `OllamaClient` (no key)
- Unknown → `Err(Error::Config)` at construction time (D-06, SC#3)

Added `pub use client::{AnthropicClient, LlmClient, OllamaClient, OpenAiClient, TokenStream}` and `pub use config::AiConfig` to `lib.rs`. `reqwest-eventsource` is NOT re-exported (SC#6, D-09). Added re-exports of concrete client types to `client/mod.rs`.

**Task 2 — AnthropicProvider thin adapter + classifier cleanup (D-10, D-03, D-14)**

Rewrote `classifier/anthropic.rs` as a thin `Arc<AnthropicClient>` adapter. The previous inline reqwest HTTP (POST to `api.anthropic.com`, response parsing, error mapping) was deleted entirely. `classify_raw` now builds a `CompletionRequest` and delegates to `self.client.complete()`. `build_request_body`, inline reqwest call, and helpers (`is_permanent_error`, `is_transient_error`) are all deleted.

`classifier/mod.rs` had D-03 (`String::new()` default model) and D-14 (`is_retryable()` retry guard) already applied from Plans 01-03. Confirmed green.

**Task 3 — Phase gate**

- `cargo fmt --all -- --check`: clean
- `cargo clippy --all --all-targets -- -D warnings`: clean
- `cargo test -p ferro-ai --lib`: 57 tests pass
- `cargo doc -p ferro-ai --no-deps`: builds clean
- `cargo test --all-features`: ENOSPC (errno=28) on `ferro-json-ui` and `ferro-queue` test binaries during linking — pre-existing disk space constraint (7.3Gi free on 460Gi disk), unrelated to ferro-ai changes. ferro-ai, ferro-mcp pass cleanly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Client type re-exports in client/mod.rs**
- **Found during:** Task 1 compile
- **Issue:** `lib.rs` `pub use client::{AnthropicClient, ...}` failed — sub-crate types not re-exported from `client/mod.rs`
- **Fix:** Added `pub use anthropic::AnthropicClient; pub use ollama::OllamaClient; pub use openai::OpenAiClient;` to `client/mod.rs`
- **Files modified:** `ferro-ai/src/client/mod.rs`
- **Commit:** 9115ea08

**2. [Rule 1 - Bug] Missing LlmClient import in classifier/anthropic.rs**
- **Found during:** Task 2 compile
- **Issue:** `self.client.complete()` failed — `LlmClient` trait not in scope for `Arc<AnthropicClient>`
- **Fix:** Added `use crate::client::LlmClient;` to imports
- **Files modified:** `ferro-ai/src/classifier/anthropic.rs`
- **Commit:** 2d7e57d7

**3. [Rule 1 - Bug] Env-var test parallelism flakiness**
- **Found during:** Task 1 test run
- **Issue:** `from_env_fails_on_unknown_provider` and `from_env_anthropic_missing_key_errors` failed because ambient `ANTHROPIC_API_KEY` was visible during parallel test execution
- **Fix:** Added `static ENV_LOCK: Mutex<()>` to serialize all env-var tests; each test sets and removes its own vars inside the lock
- **Files modified:** `ferro-ai/src/config.rs`
- **Commit:** 9115ea08

## Known Stubs

None — all implementations are fully wired. `AiConfig::from_env()` dispatches to live client constructors; `AnthropicProvider.classify_raw` delegates to `AnthropicClient::complete`.

## Threat Flags

No new security surface introduced beyond what the plan's threat model covers.

T-165-01: Retry guard uses `is_retryable()` / status — API key never logged or reconstructed. Mitigated.
T-165-02: `AiConfig::from_env()` moves key directly into provider constructor; `Error::Config` messages name the missing var, not its value. Mitigated.
T-165-07: Unknown `FERRO_AI_PROVIDER` → `Error::Config` at construction. Mitigated.

## Self-Check

**Created files exist:**
- `ferro-ai/src/config.rs` — exists (created in this plan)

**Modified files confirmed changed:**
- `ferro-ai/src/classifier/anthropic.rs` — rewired; `api.anthropic.com` absent; `Arc<AnthropicClient>` present
- `ferro-ai/src/lib.rs` — `pub use config::AiConfig` and `pub use client::{...}` added

**Commits exist:**
- `9115ea08` — feat(165-04): AiConfig::from_env() factory + client re-exports
- `2d7e57d7` — feat(165-04): rewire AnthropicProvider onto AnthropicClient
- `fdc4248b` — chore(165-04): rustfmt formatting pass on config.rs

## Self-Check: PASSED
