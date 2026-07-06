---
phase: 165
slug: llmclient-trait-provider-implementations
status: verified
threats_open: 0
asvs_level: 2
created: 2026-06-08
---

# Phase 165 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| ferro-ai → provider HTTPS (Anthropic / OpenAI / Groq) | Outbound HTTPS to provider API. | API key in request headers (`x-api-key` / `Authorization: Bearer`) only. |
| ferro-ai → Ollama HTTP | Local-default `http://localhost:11434`; no auth. | Prompt / completion payloads. No secret. |
| operator env → AiConfig | `FERRO_AI_PROVIDER/MODEL/API_KEY/BASE_URL` — operator-configured, not end-user-supplied. | Provider selection, model, API key, base URL. |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-165-01 | Information Disclosure | `Error::Provider.message` + classifier retry path | mitigate | `message` carries only provider response text; doc-comment forbids key/auth header (`error.rs:11-17`). Retry guard `!e.is_retryable()` never reconstructs or logs the key (`classifier/mod.rs:149`). | closed |
| T-165-02 | Information Disclosure | API key in request headers + `AiConfig` env read | mitigate | Key only in `x-api-key` (`client/anthropic.rs:126,172`) / `bearer_auth` (`client/openai.rs:158,199,246`); no tracing in client/config modules. `Error::Config` names the missing var, not its value (`config.rs:54,59,64`). | closed |
| T-165-03 | Tampering / SSRF | `base_url`-derived URLs (OpenAiClient / OllamaClient) | accept | `base_url` read from `FERRO_AI_BASE_URL` server-side env only (`config.rs:48`); no request-parameter path. Ollama defaults to loopback (`client/ollama.rs:38`). Accepted-risk premise verified: no user-controlled input reaches `base_url`. | closed |
| T-165-04 | Denial of Service | Streaming + non-streaming HTTP reads | mitigate | All three constructors apply `.timeout(Duration::from_secs(60))` (`client/anthropic.rs:35`, `client/openai.rs:39`, `client/ollama.rs:35`), bounding streaming and non-streaming paths. | closed |
| T-165-05 | Information Disclosure | `reqwest-eventsource` as public surface | mitigate | Imported module-internally only (`client/anthropic.rs:5`, `client/openai.rs:5`); zero eventsource symbols in any `pub use` in `lib.rs` / `client/mod.rs`. | closed |
| T-165-06 | Information Disclosure | Ollama error paths | mitigate | Ollama client has no `api_key` field; error mapping uses `e.to_string()` + `resp.text()` only — no secret to leak. | closed |
| T-165-07 | Spoofing / Config error | unknown `FERRO_AI_PROVIDER` | mitigate | Fail-fast at construction: `unknown => Err(Error::Config(...))` (`config.rs:69-71`); test `from_env_fails_on_unknown_provider` confirms. No silent fallthrough. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| R-165-01 | T-165-03 | `base_url` is operator-configured via `FERRO_AI_BASE_URL` (server-side env), never end-user-supplied at request time. Ollama default is loopback. No SSRF surface from user input; premise verified in audit. | Alberto Giancarlo Ferrario | 2026-06-08 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-08 | 7 | 7 | 0 | gsd-security-auditor (sonnet) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-08
