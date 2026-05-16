# Feature Landscape: v12.1 AI SDK + AI-Assisted Scaffolding

**Domain:** AI SDK for a Rust web framework (ferro v12.1)
**Researched:** 2026-05-15
**Confidence:** HIGH — verified against Laravel AI SDK (laravel.com/docs/13.x/ai-sdk, live), Vercel AI SDK (ai-sdk.dev, live), Spring AI (docs.spring.io, live), and Rig (github.com/0xPlaygrounds/rig, v0.37.0).

---

## Existing ferro-ai Capabilities (Do Not Re-implement)

Before classifying what to build, what already exists:

| Capability | Location | State |
|---|---|---|
| Anthropic structured JSON output (classification) | `ferro-ai/src/classifier/` | Shipped, production-quality |
| `ClassificationProvider` trait (single-provider abstraction) | `ferro-ai/src/classifier/provider.rs` | Shipped, Anthropic-only |
| Confidence threshold + retry logic | `ferro-ai/src/classifier/mod.rs` | Shipped |
| Confirmation state machine (TTL, events) | `ferro-ai/src/confirmation/` | Shipped |
| `call_anthropic()` blocking HTTP helper (CLI use) | `ferro-cli/src/ai.rs` | Shipped, Anthropic-only |
| `ferro make:json-view` (AI-powered view generation) | `ferro-cli/src/commands/make_json_view.rs` | Shipped |
| Model + route scanning for prompt context | `ferro-cli/src/ai.rs` `scan_models()` / `scan_routes()` | Shipped, regex-based |
| `ferro-mcp` `explain_route` tool (MCP, rule-based) | `ferro-mcp/src/tools/explain_route.rs` | Shipped, no LLM |

The gap: `ferro-ai` is Anthropic-only, sync-only in the CLI path, has no provider-agnostic async LLM client, no streaming, no embeddings, no OpenAI/Groq/Ollama support, and no `ai:make` / `ai:explain` CLI commands.

---

## Table Stakes

Features users expect in an AI SDK attached to a web framework. Absence makes the SDK feel incomplete or worse than just using `async-openai` directly.

| Feature | Why Expected | Complexity | Ferro-Specific Dependencies |
|---|---|---|---|
| **Multi-provider async LLM client** | Laravel AI (10 providers), Vercel AI SDK (30+), Spring AI (12+), Rig (20+) all multi-provider. Single-provider SDKs are considered deficient. | Medium — trait object dispatch + per-provider HTTP clients; reqwest already in workspace | `ClassificationProvider` trait exists; needs async chat completion, not just classification |
| **Structured output via JSON Schema** | Every comparable SDK does this. `ferro-ai` already does it via `Classifier<T>`. Needs to be exposed as a general `complete::<T>()` not just classification. | Low — already built; API surface refactor + schemars derive | `schemars` needed; `ClassifierConfig` can generalize |
| **SSE streaming (tokens to HTTP)** | Vercel AI SDK and Laravel AI SDK both list streaming as primary feature. Without it, chat-style features require polling. Axum has native `Sse<impl Stream>` support. | Medium — async stream wrapping provider SSE; Axum `response::sse` is stable API | `ferro-broadcast` for WS already exists; SSE is the HTTP equivalent for token streams |
| **`ferro ai:make <description>`** | The stated v12.1 goal. Laravel has `make:agent`, Vercel has code generation patterns. Without it, v12.1 ships no user-visible improvement over v3.0's `make:json-view`. | High — MCP introspection context assembly, multi-file scaffolding, write to disk | Depends on `ferro-mcp` introspection data (routes, models, schema); `make:json-view` is the precedent |
| **`ferro ai:explain <target>`** | GitHub Copilot `explain`, JetBrains AI Assistant `explain code`, Laravel AI dev tools all surface this. Developers orient to unfamiliar code via explanation, not just route metadata. | Medium — assemble context from MCP tools, call LLM, stream/print result | `explain_route` MCP tool exists but is rule-based, not LLM-based. `generation_context` MCP tool feeds it. |
| **API key detection + actionable error** | All SDKs fail gracefully when keys are absent. `make:json-view` already does this with `--no-ai` fallback. Pattern must apply to new commands. | Low — already implemented in `call_anthropic()`; copy pattern | None beyond existing pattern |
| **`FERRO_AI_MODEL` env override** | Already in `call_anthropic()`. Must apply uniformly to all AI commands, not just view generation. | Low | None; already done in one place, needs centralizing |

---

## Differentiators

Features that distinguish ferro's AI SDK from a thin wrapper. These are what v12.1 is worth shipping for.

| Feature | Value Proposition | Complexity | Ferro-Specific Dependencies |
|---|---|---|---|
| **MCP-context-aware `ai:make`** | Laravel's `make:agent` generates an agent class with no project knowledge. Ferro's `ai:make` can assemble live context from `ferro-mcp` (routes, models, DB schema, existing handlers, `generation_context`) before calling the LLM. The scaffold matches the actual project, not a generic template. This is the killer feature — it's what makes `ai:make` a real tool vs a template expander. | High — MCP tool invocation from CLI context, context assembly, multi-file output, structured output for generated code | `ferro-mcp` tools: `list_routes`, `list_models`, `db_schema`, `generation_context`, `code_templates`. These exist and are production-quality. |
| **LLM-backed `ai:explain` (not rule-based)** | Existing `explain_route` MCP tool is rule-based heuristics. LLM-backed explanation reads the actual handler code + model + middleware chain, then produces plain-English explanation of business logic, not just metadata. Developer onboarding time drops significantly. | Medium — read handler source + MCP context, LLM prompt, stream output | `get_handler` MCP tool reads handler source. `explain_route` provides structure. Chain them. |
| **Improved `make:json-view` via ServiceDef** | Current `make:json-view` uses regex to scan models. v12.0 ships `ServiceDef::from_model()` and structured JSON-UI specs. v12.1 can use `ServiceDef` introspection + JSON Schema component constraints for far more accurate view generation with validation guarantees. | Medium — replace regex scanning with `ServiceDef` derivation + catalog.prompt() from v12.0 | `ServiceDef::from_model()` (v11.5), JSON-UI v2 catalog with `prompt()` (v12.0 target). Depends on v12.0 landing first. |
| **Structured output derive macro** | Vercel AI SDK uses Zod schema inference. Laravel AI uses `JsonSchema $schema` builder. Ferro can use `schemars::JsonSchema` derive to auto-generate schema from a typed struct — the Rust type system becomes the schema. Zero boilerplate, compile-time safety. | Low-Medium — `schemars` integration on top of existing `Classifier<T>`; `complete::<T>()` API wrapping schema extraction | `ferro-ai` already calls JSON Schema endpoint; `schemars` needs to be added as a dep |
| **Tool calling with Rust function registration** | Rig, Spring AI, Laravel AI all support this. Register a Rust function as an AI tool; the SDK dispatches tool-use calls. In ferro's context, this enables runtime handlers that call LLM tools to fetch data, query the DB, or trigger side effects — a foundation for in-app agents. | High — tool registry, dispatch loop, serialization of inputs/outputs, async execution | No current dependency; new primitive. Requires provider support for tool_use/function_calling. |
| **Embeddings + cosine similarity** | Laravel AI has `SimilaritySearch` tool and pgvector integration. Vercel AI SDK has embeddings API. Needed for semantic search in ferro apps. Even without pgvector, cosine similarity on in-memory vectors is useful. | Medium — new provider method + optional pgvector feature flag | SeaORM already in workspace for pgvector integration path |

---

## Anti-Features

Features to explicitly not build in v12.1. These are patterns from comparable ecosystems that would be wrong for ferro.

| Anti-Feature | Why Avoid | What to Do Instead |
|---|---|---|
| **Bundled agent runtime / agentic loop inside ferro-ai** | Laravel AI has an `Agent` class that is a full conversational agent with memory, middleware, and queueing. This makes sense in PHP/Laravel where agents are application-level objects. In ferro, the agent IS the external coding agent (Claude Code, Cursor) talking to `ferro-mcp`. Building a separate in-framework agent runtime creates two competing agent models. The v12.1 AI SDK provides primitives (LLM client, structured output, tool calling) that an application can compose — it does not become an opinionated agent framework. | Expose `complete::<T>()`, streaming, and tool calling as primitives. Let applications compose agents. ferro-mcp is the surface for external agents. |
| **Conversation memory / session management in SDK** | Laravel AI's `RemembersConversations` trait persists conversation history to DB. This is application concern, not framework SDK concern. Every application has different memory requirements. | Provide message history type in the API (`Vec<Message>`) so callers supply history. Storage is caller responsibility. |
| **Multi-modal generation (image, audio, TTS, STT)** | Laravel AI supports image generation, TTS, STT, transcription. These are high-complexity, low-relevance for a Rust web framework SDK at v12.1. No ferro application has requested them. They add surface without compressing anything. | Defer to post-v1.0. The `LlmProvider` trait can extend to multi-modal later. |
| **Vector store integration (Qdrant, LanceDB)** | Rig has 10+ vector store integrations. These are per-application infrastructure decisions. Embedding a specific vector store in ferro-ai couples the SDK to infra choices. | Provide embeddings API (generate vectors). pgvector via SeaORM as an opt-in feature flag is the right boundary. External stores are application responsibility. |
| **Provider failover / automatic fallback** | Laravel AI has failover syntax: `provider: [Lab::OpenAI, Lab::Anthropic]`. This is a comfort feature that masks misconfiguration and adds implicit behavior. Ferro's correctness-first stance prefers explicit error handling. | Return `Result<_, AiError>` with provider-specific error types. Applications implement their own fallover logic if needed. |
| **LLM-powered CLI for non-ferro questions** | Generic AI assistants in CLI (explain any error, help with Docker, general coding questions) are out of scope. The CLI AI features are ferro-specific: scaffolding ferro code, explaining ferro routes/models. | Scope AI CLI commands to ferro artifacts only. |
| **UI hooks / client-side streaming components in ferro-ai** | Vercel AI SDK's `useChat`, `useObject`, `useCompletion` are React hooks. Ferro has two frontend paths: Inertia (React) and JSON-UI (server-rendered). React hooks belong in the Inertia/frontend layer. JSON-UI streaming text is a JSON-UI component concern. | SSE endpoint support in `ferro-ai` streams tokens from the server. Front-end consumption is a separate concern handled in each rendering path independently. |
| **`make:agent` command** | Laravel generates agent class stubs. In ferro, applications build handlers + tools, not named agent classes. The agent is external. A `make:agent` command generates the wrong mental model for ferro's architecture. | `ferro ai:make` generates handler + model + routes + view — real application code, not an agent wrapper. |

---

## What Makes Scaffolding Real vs Gimmicky

This section addresses the core question for `ferro ai:make` and `ferro ai:explain`.

### The Gimmick Failure Mode

A gimmicky scaffolding command does the following:
1. Takes a string description
2. Sends it to an LLM with a generic system prompt
3. Prints generated code to stdout

Result: Code that uses the wrong model names, invents route names that don't exist, uses framework APIs incorrectly, and cannot be dropped into the project without significant editing.

Evidence from PostHog's retrospective on their "Wizard" tool: "It was a single-shot edit driven by an LLM, but scaffolded by conventional code that hoped to find the right files — if your project was just a little weird, even just part of a monorepo, you were out of luck." ([PostHog LLM code generation at scale](https://posthog.com/blog/correct-llm-code-generation))

The differentiating factor in successful agentic tools is: "the orchestration, memory structures, and tool abstractions surrounding the model" — not the model itself. ([arxiv 2603.05344](https://arxiv.org/html/2603.05344v3))

### What Real Scaffolding Requires

**1. Live project context, not static templates.**

The prompt sent to the LLM must include current project state:
- Actual models with their fields (from `list_models` + `db_schema` MCP tools)
- Actual routes with their handlers (from `list_routes` MCP tool)
- Actual handler patterns (from `code_templates` MCP tool)
- `generation_context` MCP tool output (app domain, naming conventions)

`ferro-mcp` already provides all of this at production quality with 35+ tools. The `scan_models()` and `scan_routes()` regex approach in `ferro-cli/src/ai.rs` is a weak approximation — the MCP context is the correct source.

**2. Structured output, not freeform code generation.**

Freeform Rust code generation is unreliable. The correct approach for `ai:make`:
- Step 1: Generate a structured scaffold plan (`ScaffoldPlan` struct) via `complete::<ScaffoldPlan>()` — what files to create, model fields, handler signatures, route paths.
- Step 2: Expand each file from the plan using targeted per-file prompts with the plan as constraints.

This matches the pattern described in the DEV Community scaffolding article: "uses structured output for code generation ensuring variables are properly typed and validated — this generated code serves as guided generation for the LLM."

**3. Template-guided, not freeform generation.**

The LLM should fill in a known template structure, not invent the file layout. `ferro-cli/src/templates/` has templates for controllers, models, migrations, and views. `code_templates` MCP tool exposes these to agents. `ai:make` should use the same template machinery as `make:scaffold`, with AI determining what fills each template slot (field names, handler logic, route paths) rather than inventing the file structure from scratch.

**4. Multi-file output with `mod.rs` wiring.**

`make:scaffold` already writes 5+ files (model, entity, migration, controller, routes entry) and wires them together. `ai:make` must produce the same complete set. Single-file AI generation is incomplete scaffolding — it moves the wiring burden onto the developer.

**5. `ai:explain` must read actual source code.**

The `explain_route` MCP tool produces rule-based metadata (guards, related routes, usage examples) but no understanding of business logic. An LLM-backed `ai:explain` must feed actual handler source (via `get_handler` MCP tool) into the prompt. Without the source, the explanation is shallow metadata reformatting. With the source, the LLM can identify validation rules, explain side effects (events fired, jobs queued), and describe the intent of the logic — what is invisible from route metadata alone.

**6. Graceful degradation when context is unavailable.**

If `ferro-mcp` cannot gather context (no database connection, project not compiled), `ai:make` falls back to template-only scaffolding with a clear warning. This mirrors the `ANTHROPIC_API_KEY` fallback in `make:json-view`. The fallback must still produce compilable code.

---

## Feature Dependencies

```
ferro-ai multi-provider async LlmProvider trait
    → ferro ai:make (requires async LLM call from CLI process)
    → ferro ai:explain (requires streaming output to terminal)
    → SSE handler streaming (requires async stream wrapping in axum handlers)

ferro-mcp context tools (already exist, already production-quality)
    → ferro ai:make killer feature (live project context in prompt)
    → improved make:json-view via ServiceDef

v12.0 JSON-UI v2 catalog.prompt() (v12.0 must ship first)
    → improved make:json-view structured output

schemars::JsonSchema derive
    → complete::<T>() structured output API
    → ai:make ScaffoldPlan structured generation

tool calling primitive (v12.1+ direction, not MVP)
    → in-handler agent patterns
```

---

## MVP Recommendation for v12.1

Build in this priority order. Each item is independently usable. Together they form a coherent milestone with one clear killer feature.

**Phase 1 — SDK foundation:** Expand `ferro-ai` to a multi-provider async LLM client. Generalize `ClassificationProvider` to `LlmProvider` with chat completion + streaming. Add OpenAI + Groq providers. Add `complete::<T>()` with `schemars` schema extraction. Centralize `FERRO_AI_MODEL` and `FERRO_AI_PROVIDER` env vars. This is the foundation; nothing else works without it.

**Phase 2 — `ferro ai:make`:** The killer feature. Assemble context from `ferro-mcp` tools (direct library call, not subprocess), generate `ScaffoldPlan` via `complete::<ScaffoldPlan>()`, write files using existing template machinery from `make:scaffold`. This is the item that makes v12.1 worth shipping.

**Phase 3 — SSE streaming:** Add async streaming to `LlmProvider` trait + Axum `Sse<>` response helper. Required for `ai:explain` interactive output and for applications building chat interfaces.

**Phase 4 — `ferro ai:explain`:** Build on streaming. Read handler source via `get_handler` MCP tool + `explain_route` context, stream LLM explanation to terminal. Lower complexity than `ai:make`; high daily-use value for developer onboarding.

**Phase 5 — Improved `make:json-view`:** Depends on v12.0 `catalog.prompt()` and `ServiceDef`. Replace regex scanning with structured introspection. Mechanical once v12.0 lands.

**Defer to v13.0+:** Embeddings + cosine similarity, tool calling dispatch loop, pgvector integration. Real features; not v12.1 scope.

---

## Sources

- [Laravel AI SDK docs (12.x)](https://laravel.com/docs/12.x/ai-sdk) — fetched live 2026-05-15, HIGH confidence
- [Laravel AI-Assisted Development (Boost)](https://laravel.com/docs/13.x/ai) — fetched live 2026-05-15, HIGH confidence
- [Vercel AI SDK 6 announcement](https://vercel.com/blog/ai-sdk-6) — HIGH confidence
- [Spring AI tool calling reference](https://docs.spring.io/spring-ai/reference/api/tools.html) — HIGH confidence
- [Rig v0.37.0 GitHub](https://github.com/0xPlaygrounds/rig) — fetched live 2026-05-15, HIGH confidence
- [PostHog LLM code generation retrospective](https://posthog.com/blog/correct-llm-code-generation) — MEDIUM confidence, practitioner account
- [arxiv 2603.05344 — AI coding agent scaffolding](https://arxiv.org/html/2603.05344v3) — MEDIUM confidence
- [DEV Community — scaffolding with MCP + structured output](https://dev.to/vuong_ngo/scaling-ai-assisted-development-how-scaffolding-solved-my-monorepo-chaos-1g1k) — MEDIUM confidence
- ferro codebase: `ferro-ai/src/`, `ferro-cli/src/ai.rs`, `ferro-cli/src/commands/make_json_view.rs`, `ferro-mcp/src/tools/` — HIGH confidence (primary source)
