---
phase: 167-embeddings-pgvector
plan: "01"
subsystem: ferro-ai
tags: [embeddings, cosine-similarity, llm-client, env-config, d13-fix]
dependency_graph:
  requires: [165-llmclient-trait-provider-implementations]
  provides: [ferro_ai::embed, ferro_ai::cosine_similarity, Error::Sqlx]
  affects: [ferro-ai/src/embed.rs, ferro-ai/src/similarity.rs, ferro-ai/src/error.rs, ferro-ai/src/client/ollama.rs, ferro-ai/src/client/openai.rs, ferro-ai/src/lib.rs]
tech_stack:
  added: []
  patterns: [thin-delegate-free-function, panic-contract-pure-helper, env-var-FERRO_AI_EMBED_MODEL, crate-level-test-env-lock]
key_files:
  created:
    - ferro-ai/src/embed.rs
    - ferro-ai/src/similarity.rs
  modified:
    - ferro-ai/src/error.rs
    - ferro-ai/src/client/ollama.rs
    - ferro-ai/src/client/openai.rs
    - ferro-ai/src/lib.rs
    - ferro-ai/src/config.rs
decisions:
  - "Error::Sqlx added unconditionally (no #[cfg]) per D-12; pgvector reachability documented in rustdoc"
  - "crate::ENV_LOCK added to lib.rs to serialize env-var tests across modules; per-module locks insufficient for cross-module FERRO_AI_EMBED_MODEL access"
metrics:
  duration_minutes: 30
  completed_date: "2026-06-08"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 7
---

# Phase 167 Plan 01: Embeddings SDK surface (AISDK-04 + D-13 fix) Summary

Pure-Rust `cosine_similarity`, thin `embed()` free function, `Error::Sqlx` variant, and `FERRO_AI_EMBED_MODEL` env-knob fix so Ollama and OpenAI resolve a real embedding model.

## Tasks Completed

| Task | Name | Commit | Key files |
|------|------|--------|-----------|
| 1 | cosine_similarity pure-Rust helper + panic-contract tests | 243f44c1 | ferro-ai/src/similarity.rs |
| 2 | embed() free function + Error::Sqlx variant + lib.rs re-exports | 896b1560 | ferro-ai/src/embed.rs, error.rs, lib.rs |
| 3 | D-13 embed_model() fix in Ollama + OpenAI providers | 26a8e619 | client/ollama.rs, client/openai.rs, config.rs, lib.rs |

## What Was Built

**`ferro_ai::cosine_similarity(a, b)`** — pure arithmetic dot-product formula with panic contract on empty slices and dimension mismatch. Five inline tests: identical → 1.0, orthogonal → 0.0, opposite → -1.0, two `#[should_panic]` cases. Zero new dependencies (AISDK-04 promise).

**`ferro_ai::embed(client, text)`** — one-line delegate to `client.embed(text).await`, symmetric with `ferro_ai::complete`. Two `#[tokio::test]` cases covering the delegate path and `Error::Unsupported` propagation.

**`Error::Sqlx(String)`** — appended unconditionally after `StoreError`; pgvector-only reachability documented in rustdoc. `is_retryable()` wildcard arm covers it with no change (false).

**D-13 fix** — `OllamaClient::embed_model()` and `OpenAiClient::embed_model()` each read `FERRO_AI_EMBED_MODEL` with provider-specific defaults (`nomic-embed-text` / `text-embedding-3-small`). The Ollama `embed()` bug (sending chat model `llama3.1` to `/api/embed`) is fixed. Four env-var tests covering default and override cases.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Cross-module env-var test interference**
- **Found during:** Task 3
- **Issue:** Plan prescribed per-module `static ENV_LOCK` instances for `ollama.rs` and `openai.rs`. Since both modules modify `FERRO_AI_EMBED_MODEL`, and each has its own `Mutex` instance, tests from the two modules can run concurrently — the per-module locks do not serialize across module boundaries. This caused `embed_model_from_env` (ollama) to fail intermittently when `openai::embed_model_default` cleared the var mid-assertion.
- **Fix:** Added `pub static ENV_LOCK: std::sync::Mutex<()>` to `lib.rs` under `#[cfg(test)]`. Updated `ollama.rs`, `openai.rs`, and `config.rs` tests to all acquire `crate::ENV_LOCK`, serializing all env-var tests to a single process-level mutex.
- **Files modified:** ferro-ai/src/lib.rs, ferro-ai/src/client/ollama.rs, ferro-ai/src/client/openai.rs, ferro-ai/src/config.rs
- **Commit:** 26a8e619

## Verification

- `cargo test -p ferro-ai` — 91 unit tests passed, 0 failed
- `cargo clippy -p ferro-ai --all-targets -- -D warnings` — clean
- `cargo fmt -p ferro-ai -- --check` — clean
- `cargo tree -p ferro-ai | grep pgvector` — no output (zero new deps in non-feature build)

## Known Stubs

None. All symbols are fully implemented and re-exported.

## Threat Flags

No new network endpoints, auth paths, or trust boundaries introduced. `embed()` delegates over the existing `LlmClient::embed()` path (already TLS + auth-scrubbed). T-167-01 (D-13 correctness) mitigated as planned.

## Self-Check: PASSED

- ferro-ai/src/similarity.rs: FOUND
- ferro-ai/src/embed.rs: FOUND
- ferro-ai/src/error.rs contains Sqlx(String): FOUND
- ferro-ai/src/lib.rs contains pub use embed::embed: FOUND
- ferro-ai/src/lib.rs contains pub use similarity::cosine_similarity: FOUND
- ferro-ai/src/client/ollama.rs contains FERRO_AI_EMBED_MODEL: FOUND
- ferro-ai/src/client/openai.rs contains Self::embed_model(): FOUND
- Commits 243f44c1, 896b1560, 26a8e619: FOUND in git log
