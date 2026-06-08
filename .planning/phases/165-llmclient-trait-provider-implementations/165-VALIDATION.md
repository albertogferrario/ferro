---
phase: 165
slug: llmclient-trait-provider-implementations
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 165 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in; existing `#[tokio::test]` async tests) |
| **Config file** | none — workspace `Cargo.toml` + `ferro-ai/Cargo.toml` (`[dev-dependencies] tokio` already present) |
| **Quick run command** | `cargo test -p ferro-ai` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30–120 s (clean build longer; incremental ~30 s) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-ai`
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green (fmt + clippy `-D warnings` + test `--all-features`)
- **Max feedback latency:** ~120 s

---

## Per-Task Verification Map

> Skeleton mapped to the 6 Success Criteria; planner refines per actual task IDs.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 165-xx | error | 1 | AISDK-01 (SC1/SC6) | T-165-01 | `Error::Unsupported` returned, never panic; provider errors carry status not raw secrets | unit | `cargo test -p ferro-ai error::` | ❌ W0 | ⬜ pending |
| 165-xx | client-trait | 1 | AISDK-01 (SC1) | — | `LlmClient` object-safe; `Box<dyn LlmClient>` instantiable | unit (compile) | `cargo test -p ferro-ai client::` | ❌ W0 | ⬜ pending |
| 165-xx | default-model | 1 | AISDK-01 (SC4) | — | `default_model()` per provider; no hardcoded string in `ClassifierConfig::default()` | unit | `cargo test -p ferro-ai` + `! grep -q 'claude-sonnet-4-6' ferro-ai/src/classifier/mod.rs` | ❌ W0 | ⬜ pending |
| 165-xx | providers-http | 2 | AISDK-01 (SC2) | T-165-02 | request body shape per provider; API key in header only, never logged | unit (mock HTTP) | `cargo test -p ferro-ai providers::` | ❌ W0 | ⬜ pending |
| 165-xx | streaming | 2 | AISDK-01 (SC1/SC6) | — | `complete_stream` yields tokens; `reqwest-eventsource` not re-exported | unit/integration | `cargo test -p ferro-ai stream::` | ❌ W0 | ⬜ pending |
| 165-xx | aiconfig | 2 | AISDK-01 (SC3) | T-165-01 | unknown `FERRO_AI_PROVIDER` → `Error::Config` at construction, not first call | unit | `cargo test -p ferro-ai config::` | ❌ W0 | ⬜ pending |
| 165-xx | classifier-bridge | 3 | AISDK-01 (SC5) | — | existing `Classifier<T>` tests green; public API preserved | unit | `cargo test -p ferro-ai classifier::` | ✅ (exists, updated) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Add `[dev-dependencies]` needed for mocked-HTTP tests if the planner chooses `wiremock` (else use `#[ignore]`-gated live tests + pure-unit request-builder tests)
- [ ] No new test framework install — `cargo test` + existing `tokio` dev-dep cover all phase requirements

*Existing infrastructure covers all phase requirements; only optional mock-HTTP dev-dep may be added.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live provider round-trip (Anthropic/OpenAI/Ollama actually return tokens) | AISDK-01 | No API keys in CI; live network | Set `FERRO_AI_PROVIDER`/`FERRO_AI_API_KEY`, run `#[ignore]`-gated tests with `cargo test -p ferro-ai -- --ignored` locally |

*All non-network behaviors (trait shape, error classification, default_model, request-body construction, stream parsing of fixed fixtures, config dispatch) have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have automated verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120 s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
