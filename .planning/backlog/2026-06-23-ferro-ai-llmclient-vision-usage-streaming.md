# Feedback: `ferro-ai` `LlmClient` lacks vision, token-usage surfacing, streaming-with-finalization, and auth/rate-limit error classification

**Source:** Downstream AI-native consumer app `u` (private; Share→AI-synthesis→markdown-vault chat product), field assessment 2026-06-23. Same consumer that filed the `ferro-inertia` first-load-shell, `ferro-mcp-transport`, and broadcast backlog items.
**Severity:** Capability gap — blocks the consumer from routing its **capture (image+text synthesis)** and **chat (streaming + tool-calling)** hot paths through `ferro-ai`'s provider-abstracted `LlmClient`. Today `u` calls `adk-anthropic` and `async-openai` *directly* in those paths and uses `ferro-ai` only for text classification (`Classifier`). The direct-SDK coupling is exactly the duplication Ferro exists to remove, and it blocks free local-model (Ollama) dev/testing.
**Ferro version inspected:** local path-dep `ferro-ai` in the `u` tree as of 2026-06-23 (`adk-anthropic 0.9`, `async-openai 0.41` on the consumer side).

## Planning Note

This is a downstream-perspective sketch, not an inside-Ferro design. When promoting from backlog to phase(s), the Ferro planning agent should reconcile against `.planning/VISION.md`, `.planning/ROADMAP.md`, and the existing `ferro-ai/src/client/` surface before drafting `PLAN.md`. The four gaps below are separable and differ greatly in difficulty — **they likely warrant separate phases** (gaps 1/2/4 are small and independent; gap 3 is the substantial one). Suggested phase split is given per gap.

**You may (and should) read the consumer code directly.** The `u` repo is a sibling working tree at `/Users/alberto/repositories/webfucktory/u`. The file:line references below are the *consumer contracts these capabilities must satisfy* — treat them as acceptance fixtures. If a contract is ambiguous, read the cited file. (The consumer will integrate on its side once these land; it is not blocked from reading, only from a clean swap.)

---

## Problem statement

`ferro-ai`'s `LlmClient` (`ferro-ai/src/client/mod.rs:153-191`) already abstracts Anthropic / OpenAI / Ollama with `complete`, `complete_stream`, `embed`, and `complete_with_tools`, plus structured output (`schema`) and base-URL injection (`AnthropicClient::new_with_base_url`, etc.). That abstraction is the right home for a downstream multimodal, metered, streaming-with-tools consumer. But four capabilities are missing, and each blocks a specific consumer path:

1. **No vision / image input.** `Message.content` is a bare `String` (`mod.rs:46-59`); there is no image content anywhere in the crate. The consumer's capture **killer feature** is a single zero-tool vision call that OCRs + classifies + synthesizes a shared image in one request — unrepresentable today.
2. **No token-usage surfacing.** `complete` / `complete_stream` / `complete_with_tools` return `String` / `TokenStream` / `CompletionResponse` and **discard the provider response `usage{input,output}` block.** Every cost-metering and the platform spend circuit-breaker on the consumer depend on real token counts; without this they silently collapse to zero.
3. **No streaming-with-finalized-message.** `complete_stream` yields only text token `String`s; `complete_with_tools` is non-streaming. The consumer's chat loop needs to stream tokens **and** receive the finalized turn (stop_reason + `tool_use` blocks + usage) so it can run a tool-dispatch loop while streaming. This single missing primitive is *the documented reason* the consumer's chat renderer was built directly on `adk-anthropic` rather than `ferro-ai`.
4. **No typed auth/rate-limit error classification.** The consumer's BYO-key fallthrough needs to distinguish 401 (bad key → fall through to platform) from 429 (rate limit) from other errors. `ferro-ai::Error::Provider{status: Option<u16>}` exposes the raw status (`ferro-ai/src/error.rs`), but there are no `is_authentication()` / `is_rate_limit()` helpers for parity with `adk_anthropic::Error`.

---

## Gap 1 — Vision / image input  *(suggested: one small phase)*

**Proposed shape (backward-compatible, additive — keeps every text caller compiling):**
Add an optional `images` field to `Message` rather than changing `content: String` to a content-parts enum (the enum is a hard break across ~15 in-crate literals + the consumer). Derive `Default` on `Message` (add `#[default] User` to `Role`) so the field add is mechanical.

```rust
#[derive(Debug, Clone, Default)]
pub struct ImageContent { pub media_type: String, pub base64_data: String } // e.g. "image/png", base64 (no data: prefix)

pub struct Message {
    pub role: Role,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub images: Option<Vec<ImageContent>>, // NEW — only meaningful on Role::User
}
```

Serialize in each provider's `build_body` `Role::User` arm — **if `images` empty/None, emit today's bare-string form byte-identically; else emit a content-block array:**
- **Anthropic** (`client/anthropic.rs:114`): `{type:"image", source:{type:"base64", media_type, data}}` blocks, then a `{type:"text", text}` block (matches the consumer's `[Image, Text]` ordering, `u/src/capture/synthesize.rs:288-292`).
- **OpenAI** (`client/openai.rs:90`): `{type:"image_url", image_url:{url:"data:<media_type>;base64,<data>"}}`.
- **Ollama** (`client/ollama.rs:79`): top-level `images:[<base64>]` array on the message (base64 only, no media_type). Vision needs a vision-capable model via `model_override` / `FERRO_AI_MODEL` (`llava`, `llama3.2-vision`, `qwen2.5-vl`); a text-only model silently ignores the field — model selection is the caller's responsibility, do **not** hard-reject.

**Acceptance:** `build_body` unit tests per provider (mirroring existing `anthropic.rs:400-465` schema/stream tests) asserting the image content-block shape; text callers unchanged; `complete`/`complete_stream`/`complete_with_tools` all carry images for free (request-side only — no change to SSE/NDJSON parsing).

---

## Gap 2 — Token-usage surfacing  *(suggested: one small phase, ideally before/with Gap 1)*

The provider responses already contain `usage` (Anthropic `usage{input_tokens,output_tokens}`; OpenAI `usage{prompt_tokens,completion_tokens}`; Ollama `prompt_eval_count`/`eval_count`) — `ferro-ai` parses the body but drops it.

**Proposed shape:** surface usage from every completion entry point. Either a richer return type (e.g. `CompletionResponse` gains a `usage: Option<Usage>` / a `CompleteOutcome { text, usage }`) or a parallel `*_with_usage` method. The streaming path must surface usage on stream finalization (see Gap 3). Define a provider-agnostic `Usage { input_tokens: u32, output_tokens: u32 }`.

**Acceptance:** `complete` returns non-zero input/output token counts for a stubbed response carrying a `usage` block; the consumer's `pricing_map::cost_micros` (`u/src/llm/pricing_map.rs:35-40`) and platform breaker (`u/src/chat/quota.rs:148-168`) can be fed from it. **Without this, consumer metering and the spend cap die** — it is not optional for a production swap.

---

## Gap 3 — Streaming-with-finalized-message + tool loop  *(suggested: its own phase — this is the substantial one)*

This is the load-bearing capability and the reason the consumer's chat stayed on `adk-anthropic`. Today `ferro-ai` forces a choice: stream text (no tool blocks, no stop_reason, no usage) OR get tool blocks non-streaming. The consumer needs both at once.

**Reference implementation to study:** `adk-anthropic`'s accumulating-stream + final-message oneshot pattern, as consumed at `u/src/chat/renderer.rs:276-433` (the `client.stream` + `AccumulatingStream` + `final_rx` loop). The required primitive: a streaming call that yields text token deltas **and** resolves a finalized turn carrying `stop_reason`, the `tool_use` content blocks, and `usage` — so the caller can run: stream deltas → on `tool_use` finalize, dispatch tools, append assistant-then-tool_result messages (ordering contract already documented at `mod.rs:106-114`), loop.

**Consumer contracts this must preserve** (see `u/src/chat/renderer.rs`): per-delta token emission with a sentinel-hold prefix (`safe_visible_prefix`, renderer.rs:604-618); assistant-turn-precedes-tool_result ordering (renderer.rs:401); aggregated usage across tool rounds (renderer.rs:396-397); a final `Done{source, model}` frame (`u/src/chat/frame.rs:55-58`). The consumer's replay/persistence (`chat_frames` log, `chat_ws.rs`) is LLM-client-independent and needs nothing here.

**Acceptance:** a single streaming entry point that a tool-dispatch loop can drive end-to-end (text streams while tools are still callable), exposing finalized stop_reason + tool blocks + usage. Provider-abstracted across Anthropic + OpenAI (Ollama may return `Unsupported` for the tool path — acceptable; it serves the text/non-tool case for free local testing).

---

## Gap 4 — Auth / rate-limit error helpers  *(suggested: fold into Gap 2's phase — trivial)*

Add `Error::is_authentication(&self) -> bool` (status 401/403) and `Error::is_rate_limit(&self) -> bool` (status 429) over the existing `Provider{status}` variant, for parity with `adk_anthropic::Error::is_authentication()`.

**Acceptance:** the consumer's BYO-fatal classification (`u/src/llm/resolver.rs:189`, `u/src/chat/renderer.rs:570`) can be expressed as `err.is_authentication() || err.is_rate_limit()` on `ferro_ai::Error`, preserving the "first-send only" fall-through semantics.

---

## Why this is a framework concern, not a downstream concern

Vision content blocks, usage accounting, and streaming-with-tool-finalization are **modality-agnostic LLM transport plumbing** — the same shape every multimodal, metered, streaming Ferro+AI app needs, identical to how `ferro-ai` already owns provider abstraction, structured output, and the tool-call wire mapping. A downstream app hand-rolling image-block serialization, usage parsing, and a streaming accumulator per provider is precisely the per-app duplication Ferro removes. The per-app part is only the prompts, the tool set, and the pricing table; the transport is framework infrastructure. The proof: `ferro-ai` already abstracts the *hard* parts (tool-call wire format across Anthropic/OpenAI, schema normalization) — these four gaps are the remaining surface for a production consumer.

## Consumer reference / fixtures (watch this code in `/Users/alberto/repositories/webfucktory/u`)

- `src/capture/synthesize.rs` — the Anthropic vision call (`synthesize_image`, image-block shape, `:288-292`), the text call (`synthesize`), structured-output (system-prompt-embedded today) + `parse_and_validate` post-deserialize security gate, and `usage` read for cost (`:246-247`). The **only** `src/capture/` file importing `adk-anthropic` (a CI grep invariant pins this). D-17: capture is an **Anthropic-only** pipeline (OpenAI-BYO tenants synthesize on the platform Anthropic key) — Ollama/OpenAI never serve capture in production; Ollama is a dev/test lever only.
- `src/chat/renderer.rs` — the streaming + tool loop (`:276-433`), `Done{source,model}` (`:737-744`), `meter_turn` cost metering (`:776-828`), BYO fallthrough + breaker (`:149-197`), `openai_completion` non-streaming path (`:439-532`).
- `src/llm/resolver.rs` — `resolve_llm` precedence (Anthropic BYO Sonnet > OpenAI BYO > platform Haiku) and `is_byo_fatal`; how `(provider, model, key)` maps to client construction (the consumer will add a `build_llm_client(resolved, cfg) -> Box<dyn LlmClient>` factory + a config-gated `Ollama` branch for free local testing — additive, precedence untouched when unset).
- `src/llm/pricing_map.rs` — `usage → cost_micros` (consumes the tokens Gap 2 must surface).
- `src/capture/classify.rs` — the **existing, green** `ferro-ai` consumer (`AnthropicProvider` + `Classifier` + `ClassifierConfig`): the proven integration template to mirror, and proof the base-URL/test seam works.
- Regression surface the consumer must keep green after integrating (≈68 test binaries; the load-bearing ones): `capture_image_capture`, `capture_image_classification_eval`, `capture_url_happy_path`, `capture_schema_invalid_rejects`, `capture_adversarial_injection_zero_writes`, `chat_tool_loop`, `chat_turn_streams`, `chat_sentinel_split`, `chat_meters_usage`, `chat_replay_from_cursor`, `byo_fallthrough_to_platform`, `platform_circuit_breaker`. `chat_symbol_pin.rs` is a compile-time pin on `adk-anthropic` symbols and will be retired when chat moves off adk.

## Downstream value unlocked

Once these land, the consumer can: (a) run capture + chat through one `ferro-ai` seam instead of two direct SDKs; (b) use **free local Ollama models** for dev/UAT (currently blocked — the consumer has no Anthropic key for testing, which is what surfaced this); (c) test its image-synthesis (Phase 07) and chat (Phase 05/08) UAT without paid API calls. Suggested consumer-side integration order once ferro-ai ships: capture-text → capture-image → resolver/Ollama → chat (the riskiest, last).
