# Pitfalls Research — v12.1 AI Milestone

**Domain:** Multi-provider AI SDK in Rust + AI-assisted scaffolding CLI for an Axum-based framework
**Researched:** 2026-05-15
**Confidence:** HIGH (codebase inspection of ferro-ai, verified against current provider docs, Axum internals, and production reports)

---

## 1. Multi-Provider Abstraction Traps

**Pitfall: Lowest-Common-Denominator Trait Surface**

The `ClassificationProvider` trait already exists and is intentionally minimal: `classify_raw(system, user, schema, config) -> Value`. When adding streaming, tool calling, and embeddings, there is pressure to fold all capabilities into one trait. This is wrong. Providers diverge on every capability axis: Anthropic uses `output_config.format.json_schema`, OpenAI uses `response_format.json_schema.schema`, Groq uses OpenAI-compatible format with its own streaming gaps, Ollama's `/api/chat` streaming differs from its `/v1/chat/completions` compatibility layer. A single trait that tries to express all of these collapses to the lowest common denominator and forces callers to work around it.

**Prevention:** Model the SDK as capability traits, not a monolith. Define separate traits: `CompletionProvider`, `StreamingProvider`, `EmbeddingProvider`, `ToolProvider`. Let each provider implement only what it supports. The `AnthropicProvider` implements all four; `Ollama` implements `CompletionProvider` and `EmbeddingProvider` but `StreamingProvider` only when tools are absent. Callers bind to the specific trait they need.

**Phase:** SDK foundation (first ferro-ai expansion phase). Trait surface is load-bearing for every downstream phase. If it is wrong, everything built on it must be rebuilt.

---

**Pitfall: Assuming `tool_choice: "any"` Works Everywhere**

Anthropic returns a 400 error when `tool_choice: {"type": "any"}` is combined with extended thinking. OpenAI and Groq have no such restriction. If the `ToolProvider` trait passes `tool_choice` as a parameter without provider-aware validation, callers will get opaque 400 errors from Anthropic but succeed on OpenAI with the same configuration.

**Prevention:** The `ToolProvider` implementation for Anthropic must validate that extended thinking and forced tool choice are not combined before sending the request. Document this constraint explicitly in the provider struct doc comment. Do not push this validation into the trait — it is provider-specific.

**Phase:** Tool calling phase. Must be addressed before any integration test that spans providers.

---

**Pitfall: Assuming OpenAI's Compatibility Layer is Faithful**

Ollama's `/v1/chat/completions` (OpenAI-compatible endpoint) silently drops tool calls when streaming is enabled as of mid-2025. Groq's SSE implementation is missing `finish_reason` in some streaming chunks, which breaks parsers that rely on it for stream termination. The compatibility layer is not the same as the native API.

**Prevention:** For Ollama, disable streaming when tools are present (`stream: false`). Document this in the `OllamaProvider` struct. Test tool calling specifically against the native `/api/chat` endpoint, not the compatibility layer. For Groq, handle missing `finish_reason` in the stream parser by also checking for the `[DONE]` SSE sentinel.

**Phase:** Each new provider implementation phase. Each provider needs a compatibility test that covers streaming + tool calling together, not in isolation.

---

**Pitfall: JSON Schema Portability Across Providers**

No provider accepts top-level `$ref` in structured output schemas. Anthropic additionally requires `additionalProperties: false` on every object in the schema, prohibits recursive schemas, and does not support numerical constraints (`minimum`, `maximum`) or string constraints (`minLength`, `maxLength`). OpenAI's `chat/completions` and `responses` API endpoints use different field structures for the schema. Gemini requires explicit `type` on array `items`.

The `schemars` crate generates schemas from Rust structs via `schema_for!(T)`. By default, `schemars` emits `$defs` with `$ref` references for complex types. This output will be rejected by Anthropic's structured output endpoint.

**Prevention:** Implement a schema normalizer that runs `schemars` output through a post-processing step before sending to each provider: resolve `$ref` references inline, add `additionalProperties: false` to all objects for Anthropic, strip unsupported constraints per provider. Place this in a `schema` module inside `ferro-ai`. The normalizer runs once per call and is not on the hot path.

**Phase:** Structured output phase. Must exist before any use of `schemars` with a live provider.

---

## 2. Async / Streaming in Axum Handlers

**Pitfall: Using `reqwest::blocking` Inside the Server**

The existing `ferro-cli/src/ai.rs` uses `reqwest::blocking::Client`. This is correct for CLI commands (no Tokio runtime in scope). If any of this code is copy-pasted into an Axum handler, it will panic at runtime: `reqwest::blocking` cannot run inside a Tokio runtime. The panic message is `Cannot start a runtime from within a runtime.`

**Prevention:** Handlers that proxy LLM responses must use `reqwest::Client` (async) throughout. Add a compile-time marker or doc comment on the CLI-only `call_anthropic` function making the restriction explicit. Do not share the `ai.rs` module between CLI and handler code — they have different runtime contexts.

**Phase:** Any phase that adds SSE streaming to Axum handlers. The divergence between CLI and handler usage must be resolved before mixing them.

---

**Pitfall: Tower-http `CompressionLayer` Breaks SSE**

`CompressionLayer` buffers the response body before compressing it. SSE responses must not be buffered — the client receives no tokens until the buffer fills. The symptoms are: SSE handler appears to work in tests, but the browser receives all tokens at once after a long pause. The middleware does not warn; it silently buffers.

**Prevention:** SSE routes must be excluded from `CompressionLayer` either by not applying it at the router level for streaming routes, or by using Axum's per-route layer composition to skip compression. Do not apply `CompressionLayer` globally and assume SSE routes are exempt.

**Phase:** SSE streaming phase. Add an integration test that verifies token-by-token delivery, not just final output correctness.

---

**Pitfall: Slow Client Suspends the Provider Task**

Axum SSE uses a `Stream`-based response. When the client's TCP receive buffer fills (slow browser, mobile, or dropped connection), backpressure propagates up through the network stack into the task writing to the stream. The server task that is forwarding LLM tokens suspends. The LLM connection stays open and accumulates tokens that cannot be sent. On Anthropic's API, this means the connection timeout clock runs while nothing is being consumed.

**Prevention:** Wrap the SSE write side in a timeout: if a single `yield` takes longer than a threshold (500ms is reasonable), close the SSE connection and drop the LLM response stream. Use `tokio::time::timeout` around each `yield`. The LLM client should independently have a per-token deadline. This prevents runaway server tasks from open connections to slow clients.

**Phase:** SSE streaming phase. Must be in the initial implementation, not added later when problems appear in production.

---

**Pitfall: Missing `keep-alive` on Long-Running Streams**

Long LLM responses can take 30–90 seconds. Load balancers and reverse proxies (nginx, Caddy, AWS ALB) have idle connection timeouts, typically 60 seconds. If no bytes are sent during a long reasoning step, the proxy closes the connection mid-stream. The client sees a truncated response with no error.

**Prevention:** Configure SSE keep-alive pings. Axum's `Sse::keep_alive()` method sends a `:ping\n\n` comment periodically. Set the interval to 15 seconds. This does not affect the client's event parsing. Document the keep-alive interval in the handler so it is not removed without understanding the consequence.

**Phase:** SSE streaming phase. Default keep-alive should be part of any SSE handler scaffold.

---

## 3. Structured Output Reliability

**Pitfall: Schema Compliance Does Not Guarantee Semantic Correctness**

Provider structured output features (Anthropic's `output_config.json_schema`, OpenAI's `response_format.json_schema`) guarantee syntactic conformance. They do not guarantee correct answers. A classifier that always returns `"confidence": 0.99` is schema-compliant but operationally wrong. Required fields force the model to hallucinate when no good answer exists. The failure mode is silent: valid JSON, wrong data, crashes no parser.

**Prevention:** Schema compliance and business logic validation are separate steps. After deserialization, validate fields that have business constraints (confidence in range, non-empty strings where required, enum values that correspond to real entities). The existing `ClassifierConfig.confidence_threshold` is correct but insufficient alone. Add post-deserialization validators as part of `ClassificationResult`.

**Phase:** Every phase that uses structured outputs. The validator pattern should be established in the SDK foundation phase, not retrofitted.

---

**Pitfall: Model Returns a Refusal Instead of JSON**

When using Anthropic structured outputs, the model can return a `stop_reason: "end_turn"` with the content being a refusal text block instead of a JSON block. The existing `ferro-ai` parser looks for `content[0].text` and calls `serde_json::from_str`. If the model refused, this parses to an `Error::Deserialization` with an opaque message like "expected value at line 1 column 1", hiding the fact that a refusal occurred.

**Prevention:** Before calling `serde_json::from_str`, check whether the text content starts with `{` or `[`. If it does not, check for common refusal patterns and return a dedicated `Error::Refusal(String)` variant. The caller can then decide whether to retry with a relaxed prompt or surface the refusal to the user.

**Phase:** SDK foundation phase. The `Error` enum and the response parser are already in `ferro-ai`; the refusal check should be added there, not deferred.

---

**Pitfall: `schemars` Default Output Not Accepted by Anthropic**

`schema_for!(T)` from `schemars` produces schemas with `$defs` and `$ref` references for any type that appears more than once, and does not add `additionalProperties: false`. Both patterns are rejected by Anthropic's structured output endpoint. The error from Anthropic is a 400 with a message about unsupported schema features; the schema that caused the rejection is not echoed back, making it hard to debug.

**Prevention:** Write a `ferro_ai::schema::for_structured_output` function that takes the `schemars`-generated `RootSchema`, resolves all `$ref` references inline, adds `additionalProperties: false` to every object, and strips unsupported keywords before returning the cleaned `serde_json::Value`. This function is the only path for generating schemas for structured output calls. Include a test that round-trips a complex struct through this function and verifies the output against Anthropic's documented constraints.

**Phase:** Structured output phase. Block provider integration tests on this being present.

---

## 4. Tool Calling Complexity

**Pitfall: Unbounded Tool Call Loops**

An LLM agent that can call tools will sometimes enter a loop: tool result → model reasons → tool call → tool result → ... A Claude Code instance in July 2025 consumed 1.67 billion tokens in 5 hours in a loop before the user noticed, generating an estimated $16,000–$50,000 in charges. The loop did not crash or error; it just continued.

**Prevention:** The `ToolProvider` orchestration layer must enforce a `max_iterations: u32` parameter. After `max_iterations` tool call + response pairs, force a final extraction step rather than allowing another tool call. Default to 10 iterations. Log a warning at 5 and an error at 10. Make the limit configurable but never unbounded. This is not optional — treat it as a hard invariant in the loop.

**Phase:** Tool calling phase. The limit must be present in the initial implementation, not added after observing runaway behavior.

---

**Pitfall: Tool Error → Model Cannot Interpret → Loop**

When a tool returns an error, the model receives the error message as a tool result. If the error message is not actionable (stack trace, internal ID, database constraint error), the model will often retry the same tool call with the same arguments, producing the same error, looping. The loop runs until `max_iterations` fires or until the user kills the process.

**Prevention:** Tool implementations must return user-legible error messages, not raw error chains. Define a `ToolError` type with a `message: String` field intended for the model to read. Map internal errors to actionable descriptions before returning them to the orchestrator. The orchestrator should not pass raw `Display` output from `anyhow::Error` or `Box<dyn Error>` back to the model.

**Phase:** Tool calling phase. Establish the `ToolError` type before registering any tools.

---

**Pitfall: Parallel Tool Calls Return Out-of-Order**

Both OpenAI and Anthropic can request multiple tool calls in a single response. If the orchestrator executes them with `tokio::spawn` for parallelism, results arrive out of order. Providers require tool results to be submitted in the same order as the tool call IDs in the request. Submitting results out of order produces a 400 error with a message about mismatched tool use IDs.

**Prevention:** Collect all tool calls from the model's response, execute them in parallel with `tokio::join_all`, and then reassemble results in the original order by matching on the `tool_use_id` field before submitting. The join is on the IDs, not on execution order.

**Phase:** Tool calling phase. Test with a multi-tool request to confirm ordering is preserved.

---

## 5. Rate Limiting and Cost Control

**Pitfall: CLI Commands With No Rate Limiting or Cost Cap**

`ferro ai:make` and `ferro ai:explain` will make LLM API calls on behalf of the developer. Without limits, a developer writing a shell script around `ferro ai:make` in a loop (scaffolding 50 models from a CSV) will hit Anthropic's rate limits and generate unexpected costs. Rate limits (429) surface as transient errors that the existing retry logic will retry, amplifying the problem.

**Prevention:** Add three controls to every AI CLI command: (1) a `--dry-run` flag that prints the prompt and estimated token count without calling the API; (2) a configurable `FERRO_AI_MAX_TOKENS_PER_COMMAND` env var that aborts before sending if the prompt exceeds the limit (default 100K tokens); (3) exponential backoff for 429 responses with a maximum wait of 30 seconds and a maximum of 3 retries before surfacing the error. The dry-run flag is the most user-visible safety net.

**Phase:** AI CLI commands phase. All three controls must be present in the first CLI command shipped.

---

**Pitfall: Retry Logic Amplifies Rate-Limit Cost**

The existing `Classifier` retry logic retries `max_retries` times on transient errors, including 429. A 429 retry with a 1-second delay (current default) can still exceed rate limits because the retry itself is within the rate limit window. Retrying a 429 immediately is the same as sending the original request again too fast.

**Prevention:** When the error is a 429, extract the `retry-after` header from Anthropic's response and wait that duration before retrying. If no header is present, use exponential backoff with jitter starting at 5 seconds. The current `retry_delay` default of 1 second is too short for 429 handling. The `is_transient_error` classification for 429 is correct, but the delay logic must be rate-limit-aware.

**Phase:** SDK foundation phase. Fix before any production use; the current implementation will amplify rate limit pressure.

---

## 6. Context Window Management for AI Scaffolding

**Pitfall: ferro-mcp Introspection Output Is Too Large for a Single Prompt**

`ferro ai:make <description>` will use `ferro-mcp` introspection as context: routes, models, schema, events, and generation hints. For a large application (the sample `app/` already has dozens of routes and models), the introspection output easily exceeds 50K tokens. Sending the full introspection as a system prompt has two problems: cost (system prompts are billed at input token rates) and quality degradation (models perform worse on tasks when the context is dominated by irrelevant information).

Research shows performance degrades reliably once effective context exceeds ~128K tokens for most models, and the degradation begins well below the advertised context limit.

**Prevention:** Apply selective context loading: include only the models and routes that are semantically relevant to the user's description. For `ferro ai:make "order checkout form"`, include only models with fields related to "order", "checkout", or "payment", and only the routes that match those models. Use string matching against the description as a first-pass filter. This is not semantic search — string matching against route names and model field names is sufficient for CLI scaffolding and avoids introducing an embeddings dependency.

**Phase:** AI CLI commands phase. The context selection logic should be built before the prompt construction, not after observing cost or quality problems.

---

**Pitfall: Component Catalog in Prompt Is Unbounded**

The existing `build_view_context` in `ferro-cli/src/ai.rs` includes the full `COMPONENT_CATALOG` in the system prompt. For v12.0 JSON-UI v2, the catalog will grow. Embedding the full catalog schema in every AI generation request is expensive and, per the v12.0 decision record, explicitly rejected: "Full catalog schema in AI prompts — 36-component oneOf produces 40-80 KB schema, too large for system prompts."

**Prevention:** Use per-component schemas for generation (as decided in v12.0), not the full catalog. The `ferro ai:make` command selects which components are likely needed based on the description, injects only those component schemas, and provides a component list summary for the others. This is already the documented direction for v12.0 MCP integration; carry it through to the CLI commands.

**Phase:** AI CLI commands phase. The prompt construction strategy from v12.0 must be applied here; do not regress to full-catalog prompts.

---

## 7. Integration with Existing ferro-ai Crate

**Pitfall: `reqwest::blocking::Client` in ferro-cli vs Async in ferry-ai**

`ferro-cli/src/ai.rs` uses `reqwest::blocking::Client` with `reqwest::blocking::ClientBuilder`. `ferro-ai/src/classifier/anthropic.rs` uses `reqwest::Client` (async). These cannot share provider implementations. If the CLI commands are refactored to use `ferro-ai` directly, the blocking client must be removed and the CLI binary must run inside a Tokio runtime.

**Prevention:** The AI CLI commands expansion should route through `ferro-ai`'s async providers using `tokio::runtime::Runtime::block_on` at the CLI entry point, or by adding `#[tokio::main]` to the CLI binary. Do not maintain two parallel HTTP client paths for AI calls. The blocking client in `ferro-cli/src/ai.rs` should be deleted when the CLI commands migrate to `ferro-ai`.

**Phase:** AI CLI commands phase. The migration must happen in full; a hybrid state where some CLI commands use blocking and others use async via `ferro-ai` is a maintenance trap.

---

**Pitfall: `async_trait` and `dyn ClassificationProvider` Are Coupled**

The existing `ClassificationProvider` trait uses `#[async_trait]` to achieve object safety (`Arc<dyn ClassificationProvider>`). Rust 1.75 stabilized native `async fn` in traits, but native async traits are not dyn-compatible without the `trait_variant` crate or manual boxing. Removing `async_trait` while keeping `dyn ClassificationProvider` will break compilation.

**Prevention:** Keep `async_trait` on the provider traits. It is not deprecated; it has reduced applicability but is still required for dyn-compatible async traits. If native async trait support improves to include dyn compatibility before this milestone ships, reassess. Do not perform the migration speculatively.

**Phase:** No migration needed. This is a false target; keep `async_trait` and note it in the `ClassificationProvider` doc comment.

---

**Pitfall: Adding OpenAI/Groq Providers Breaks the Current `ClassifierConfig.model` Default**

`ClassifierConfig::default()` hardcodes `model: "claude-sonnet-4-6"`. When a user configures an OpenAI or Groq provider and uses `ClassifierConfig::default()`, the default model is wrong for the provider. There is no type-level enforcement — the mismatch surfaces only at runtime as a 400 error from the provider.

**Prevention:** The SDK expansion must change `ClassifierConfig` to either: (a) have no default model (force the caller to supply one), or (b) have the provider supply its own default model via a method on the provider trait. Option (b) is cleaner because the provider knows its own supported models. Add `fn default_model(&self) -> &str` to `ClassificationProvider`. Remove the hardcoded default from `ClassifierConfig` or make it `Option<String>` and resolve it through the provider at call time.

**Phase:** SDK foundation phase. Fix before any provider other than Anthropic is added.

---

## 8. Axum SSE Response and Streaming Architecture

**Pitfall: Axum `Sse<S>` Requires the Stream to Be `Unpin`**

Axum's `Sse<S>` response wrapper requires `S: Stream + Unpin`. When constructing a stream from an `async fn` or from an LLM response body using `futures::stream::unfold`, the result is not `Unpin` by default. The compiler error ("the trait bound `impl Stream<...>: Unpin` is not satisfied") is correct but the fix is non-obvious: `Box::pin` or `tokio_stream::wrappers::ReceiverStream` from an `mpsc::channel`.

**Prevention:** For handler-to-LLM token streams, use the channel pattern: spawn a background task that reads from the LLM response stream and sends tokens through a `tokio::sync::mpsc::channel`, then wrap the receiver in `ReceiverStream` which is `Unpin`. This also naturally handles the backpressure and timeout patterns described in Section 2. Document this as the canonical SSE handler pattern in `ferro-ai`.

**Phase:** SSE streaming phase. Include a working example handler in the crate documentation; do not leave this as an exercise for the caller.

---

**Pitfall: Anthropic Response Parsing Assumes `content[0].text` Is JSON**

The current `AnthropicProvider::classify_raw` extracts `content[0].text` and calls `serde_json::from_str`. With the GA structured output API (`output_config.format.json_schema`), Anthropic returns the structured JSON in `content[0].text` as a JSON string. This is correct for the current implementation.

However, for streaming responses, Anthropic sends content as a sequence of delta events (`content_block_delta` with `delta.type = "text_delta"` and `delta.text = "<chunk>"`). The streaming parser must accumulate these deltas and parse the assembled string at the end, not parse each chunk as JSON. Parsing chunks individually will fail because each chunk is a partial JSON string.

**Prevention:** The streaming provider implementation must buffer `text_delta` chunks and parse the complete assembled string after receiving `message_stop`. Do not attempt streaming + structured output parsing simultaneously (parse per-chunk). For non-structured streaming, yield each delta immediately to the Axum SSE stream.

**Phase:** SSE streaming phase. Keep structured output parsing (blocking, via `classify_raw`) separate from streaming token delivery. Do not combine them in the same code path.

---

## Phase-Specific Warning Matrix

| Phase Topic | Pitfall | Mitigation |
|---|---|---|
| SDK provider trait | Monolithic trait forcing LCD surface | Capability traits: `CompletionProvider`, `StreamingProvider`, `EmbeddingProvider`, `ToolProvider` |
| Anthropic structured output | `schemars` emits `$ref`; rejected as 400 | `schema::for_structured_output()` normalizer before any provider call |
| Structured output | Model returns refusal, parsed as deserialization error | `Error::Refusal` variant; check `text` before JSON parse |
| Multi-provider `ClassifierConfig` | Default model `"claude-sonnet-4-6"` wrong for OpenAI/Groq | Provider-supplied `default_model()` or `Option<String>` in config |
| SSE in handlers | `reqwest::blocking` panics in Tokio context | Delete blocking path; CLI entry point uses `block_on` |
| SSE in handlers | `CompressionLayer` buffers SSE | Exclude SSE routes from compression middleware |
| SSE in handlers | Slow client suspends provider task | `tokio::time::timeout` on each SSE yield; close on exceeded threshold |
| SSE in handlers | Reverse proxy closes idle long connections | `Sse::keep_alive()` at 15-second interval |
| Tool calling | Unbounded iteration loops, $50K incidents documented | `max_iterations` hard limit, default 10 |
| Tool calling | Tool errors cause retry loops | `ToolError` with model-legible `message` field |
| Tool calling | Parallel tool results sent out of order | Join on `tool_use_id`; submit results in original request order |
| Rate limiting | 429 retried too fast, amplifies rate pressure | Extract `retry-after` header; exponential backoff for 429 |
| CLI cost | No cost visibility before invoking model | `--dry-run` flag + token count estimate |
| Context size | Full `ferro-mcp` output exceeds 50K tokens | Selective context: filter to description-relevant models and routes |
| Ollama provider | OpenAI compat layer drops tool calls when streaming | `stream: false` when tools present on Ollama |
| Provider config | `tool_choice: "any"` + extended thinking → 400 on Anthropic | Validate in `AnthropicToolProvider` before sending |

---

## Sources

- [Axum SSE backpressure thread, Rust Users Forum](https://users.rust-lang.org/t/axum-sse-and-backpressure/133061)
- [LLM API differences that break your code (FutureSearch)](https://futuresearch.ai/blog/llm-provider-quirks/)
- [Structured output reliability in production (TianPan.co, 2026-04)](https://tianpan.co/blog/2026-04-20-structured-output-reliability-production)
- [Anthropic structured outputs docs](https://platform.claude.com/docs/en/build-with-claude/structured-outputs)
- [Ollama streaming + tool calling bug report](https://github.com/ollama/ollama/issues/12557)
- [LLM tool calling in production — infinite loop failure mode (Medium)](https://medium.com/@komalbaparmar007/llm-tool-calling-in-production-rate-limits-retries-and-the-infinite-loop-failure-mode-you-must-2a1e2a1e84c8)
- [Tool-use API design: 5 patterns that prevent agent loops](https://dev.to/adamo_software/tool-use-api-design-for-llms-5-patterns-that-prevent-agent-loops-and-silent-failures-f29)
- [Axum SSE docs](https://docs.rs/axum/latest/axum/response/sse/)
- [async fn in traits stabilization (Rust blog)](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html)
- [Context window management strategies (GetMaxim)](https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots/)
- [Groq streaming finish_reason bug report](https://community.groq.com/t/groq-api-bug-report-missing-finish-reason-in-streaming-responses/775)
- [reqwest-eventsource crate](https://docs.rs/reqwest-eventsource/)
- [Structured output comparison across providers (Medium)](https://medium.com/@rosgluk/structured-output-comparison-across-popular-llm-providers-openai-gemini-anthropic-mistral-and-1a5d42fa612a)
