---
phase: 165
slug: llmclient-trait-provider-implementations
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-08
validated: 2026-06-08
---

# Phase 165 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in; `#[tokio::test]` async tests for network-shaped behavior) |
| **Config file** | none — workspace `Cargo.toml` + `ferro-ai/Cargo.toml` (`[dev-dependencies] tokio` already present) |
| **Quick run command** | `cargo test -p ferro-ai --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~0.5 s incremental (`-p ferro-ai --lib`); clean workspace build longer |

*No mock-HTTP framework was added. All provider behavior is verified through `pub(crate)` pure request-builder / delta-parser helpers exercised against fixtures, so no live server is required.*

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-ai --lib`
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green (fmt + clippy `-D warnings` + test `--all-features`)
- **Max feedback latency:** ~120 s

---

## Per-Task Verification Map

> Mapped to the 6 ROADMAP Success Criteria (SC1–SC6) and the actual implemented tasks across Plans 01–04.

| Task | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Verification | Status |
|------|------|------|-------------|------------|-----------------|-----------|------------------------|--------|
| Error restructure + `is_retryable` | 01 | 1 | AISDK-01 (SC1/SC5) | T-165-01 | status-based retry; `Provider.message` never carries API key | unit | `test_error_is_retryable` | ✅ green |
| `LlmClient` trait + skeleton | 01 | 1 | AISDK-01 (SC1) | — | object-safe; `Box<dyn LlmClient>` instantiable | unit (compile) | `test_anthropic_is_object_safe`, `test_openai_is_object_safe`, `test_ollama_is_object_safe`, `test_classification_provider_is_object_safe` | ✅ green |
| AnthropicClient | 02 | 2 | AISDK-01 (SC1/SC2) | T-165-02, T-165-04 | x-api-key header only; 60s timeout; embed → `Unsupported` | unit | `test_anthropic_default_model{,_override}`, `test_build_body_with_schema`, `test_build_body_without_schema_omits_output_config`, `test_build_body_stream_flag`, `test_anthropic_embed_unsupported`, `test_parse_anthropic_delta{,_empty_text,_non_delta_event}` | ✅ green |
| OpenAiClient (+ Groq base_url) | 02 | 2 | AISDK-01 (SC1/SC2) | T-165-02, T-165-04 | bearer_auth header only; 60s timeout; `[DONE]` sentinel handled | unit | `test_openai_default_model`, `test_openai_default_base_url`, `test_openai_groq_base_url`, `test_build_body_response_format_with_schema`, `test_build_body_no_response_format_without_schema`, `test_parse_openai_delta_{done,token,skip_empty_content,finish_reason}`, `test_parse_embedding{,_missing}` | ✅ green |
| OllamaClient | 03 | 2 | AISDK-01 (SC1/SC2) | T-165-03, T-165-04 | loopback default base_url; 60s timeout; NDJSON (not SSE) | unit | `test_ollama_default_model`, `test_ollama_model_override`, `test_ollama_default_base_url`, `test_parse_ollama_line_{token,done}`, `test_parse_ollama_embedding{,_missing}` | ✅ green |
| `AiConfig::from_env` factory | 04 | 3 | AISDK-01 (SC3) | T-165-01, T-165-07 | unknown `FERRO_AI_PROVIDER` → `Error::Config` at construction; `Error::Config` names the missing var, not its value | unit | `from_env_fails_on_unknown_provider`, `from_env_anthropic_missing_key_errors`, `from_env_anthropic_with_explicit_key`, `from_env_groq_base_url_default`, `from_env_ollama_default_model` | ✅ green |
| `ClassifierConfig::default()` model fix | 04 | 3 | AISDK-01 (SC4) | — | no hardcoded `"claude-sonnet-4-6"`; resolved via `default_model()` | unit + grep guard | `test_classifier_config_defaults`, `test_classify_request_empty_model_uses_client_default`, `test_classify_request_shape_with_explicit_model`; `! grep -q 'claude-sonnet-4-6' ferro-ai/src/classifier/mod.rs` | ✅ green |
| AnthropicProvider thin adapter | 04 | 3 | AISDK-01 (SC5) | — | classifier public API preserved; single HTTP source of truth (D-10) | unit | `test_classification_result_deserialization`, `test_classification_extracts_confidence`, `test_retry_on_transient_error`, `test_no_retry_on_permanent_error` | ✅ green |
| `reqwest-eventsource` not re-exported | 04 | 3 | AISDK-01 (SC6) | T-165-05 | SSE dependency stays `pub(crate)` | grep guard | `! grep -q 'eventsource' ferro-ai/src/lib.rs` | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Suite result:** `cargo test -p ferro-ai --lib` → 57 passed, 0 failed, 0 ignored (0.50s).

---

## Wave 0 Requirements

- [x] No new test framework installed — `cargo test` + existing `tokio` dev-dep cover all phase requirements.
- [x] No mock-HTTP dev-dep needed — provider request/response logic is exercised through `pub(crate)` pure helpers (`parse_anthropic_delta`, `parse_openai_delta`, `parse_ollama_line`, `parse_embedding`, `build_body`) against fixtures.

*Existing infrastructure covered all phase requirements; no Wave 0 additions were required.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live provider round-trip (Anthropic/OpenAI/Ollama actually return tokens over the network) | AISDK-01 | No API keys in CI; live network | Set `FERRO_AI_PROVIDER` / `FERRO_AI_API_KEY` (and `FERRO_AI_BASE_URL` for Ollama), then exercise `complete` / `complete_stream` against the live endpoint locally |

*All non-network behaviors — trait shape & object-safety, error classification & retry, `default_model`, request-body construction (schema-gated), SSE/NDJSON delta parsing on fixtures, embedding parsing, and `from_env` config dispatch — have automated verification. The single manual-only item is an irreducible live-network check that cannot be sampled in CI; it is supplementary to, not a gap in, the automated coverage of the six success criteria.*

---

## Validation Sign-Off

- [x] All tasks have automated verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (none required)
- [x] No watch-mode flags
- [x] Feedback latency < 120 s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated 2026-06-08 — 57/57 automated tests green; all 6 success criteria have automated verification.

---

## Validation Audit 2026-06-08

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Manual-only (irreducible live-network) | 1 |

Audited shipped artifacts (Plans 01–04) against the planning-time draft. All placeholder task IDs (`165-xx`) replaced with the implemented task map. Every success criterion (SC1–SC6) maps to green automated tests or a passing structural grep guard. No MISSING or PARTIAL classifications — no `gsd-nyquist-auditor` spawn required. Phase marked `nyquist_compliant: true`.
