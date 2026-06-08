# Security Audit — Phase 165: LlmClient Trait & Provider Implementations

**Phase:** 165 — llmclient-trait-provider-implementations
**ASVS Level:** 2
**Audited:** 2026-06-08
**Result:** SECURED — 7/7 threats closed

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-165-01 | Information Disclosure | mitigate | CLOSED | `error.rs:11-17` — `message` field doc-comment explicitly states it must not contain the API key or auth header. `#[error("ai provider error ({status:?}): {message}")]` interpolates only `status` and `message`; no secret source. Retry guard at `classifier/mod.rs:149` matches `!e.is_retryable()` — never reconstructs or logs the key. |
| T-165-02 | Information Disclosure | mitigate | CLOSED | `client/anthropic.rs:126,172` — key placed only in `x-api-key` header via `.header()`. `client/openai.rs:158,199,246` — key placed only via `.bearer_auth()`. No tracing calls in any client or config module. Error mapping uses `e.to_string()` (reqwest send error) and `resp.text()` (provider body); neither echoes request headers. `config.rs:54,59,64,70` — `Error::Config` messages name the missing variable (`"FERRO_AI_API_KEY not set"`), not its value. |
| T-165-03 | Tampering / SSRF | accept | CLOSED | `config.rs:48` — `base_url` is read only from `FERRO_AI_BASE_URL` (server-side env var), not from any request parameter. `client/openai.rs:42` — defaults to `"https://api.openai.com"`. `client/ollama.rs:38` — defaults to `"http://localhost:11434"` (loopback). No user-controlled input flows into `base_url` construction. Accepted risk documented: operator-configured server-side env only. |
| T-165-04 | Denial of Service | mitigate | CLOSED | `client/anthropic.rs:35` — `.timeout(Duration::from_secs(60))`. `client/openai.rs:39` — `.timeout(Duration::from_secs(60))`. `client/ollama.rs:35` — `.timeout(Duration::from_secs(60))`. All three constructors apply the 60-second timeout bounding both streaming and non-streaming paths. |
| T-165-05 | Information Disclosure | mitigate | CLOSED | `client/anthropic.rs:5` and `client/openai.rs:5` — `use reqwest_eventsource::{Event, RequestBuilderExt}` scoped to module, never `pub use`. `client/mod.rs` — no eventsource import or re-export. `lib.rs` — no eventsource symbol in any `pub use` line. `client/ollama.rs` — no eventsource reference (NDJSON path). Zero public eventsource surface confirmed by grep. |
| T-165-06 | Information Disclosure | mitigate | CLOSED | `client/ollama.rs` — no `api_key` field; `OllamaClient` struct carries only `client`, `model`, `base_url`. Error mapping at lines 133,172,196 uses `e.to_string()` and `resp.text()` — provider response text only. No secret to leak. |
| T-165-07 | Spoofing / Config error | mitigate | CLOSED | `config.rs:69-71` — `unknown => Err(Error::Config(format!("unknown FERRO_AI_PROVIDER: '{unknown}'")))` — fail-fast at `AiConfig::from_env()` call time, not at first LLM call. Test `from_env_fails_on_unknown_provider` verifies `Err(Error::Config(_))` for `FERRO_AI_PROVIDER="bogus"`. |

---

## Accepted Risk Log

| Threat ID | Rationale |
|-----------|-----------|
| T-165-03 | SSRF via `base_url` is not actionable at this layer. `FERRO_AI_BASE_URL` is a server-side operator-configured environment variable, not end-user input. The Ollama default is loopback. No code path allows a request parameter to influence `base_url`. If the deployment model changes to allow user-controlled base URLs, this threat must be re-evaluated and escalated. |

---

## Unregistered Flags

None. No SUMMARY.md `## Threat Flags` section introduced surface beyond the registered threat register.

---

## Scope

Files audited:
- `ferro-ai/src/error.rs`
- `ferro-ai/src/config.rs`
- `ferro-ai/src/lib.rs`
- `ferro-ai/src/client/mod.rs`
- `ferro-ai/src/client/anthropic.rs`
- `ferro-ai/src/client/openai.rs`
- `ferro-ai/src/client/ollama.rs`
- `ferro-ai/src/classifier/anthropic.rs`
- `ferro-ai/src/classifier/provider.rs`
- `ferro-ai/Cargo.toml`
