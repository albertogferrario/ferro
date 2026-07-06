# Phase 100: AI Structured Classification & Confirmation Primitives - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Add two generic framework primitives: (1) structured AI classification — a provider-abstracted wrapper for LLM structured JSON output with configurable schema, model selection, confidence threshold, and retry behavior; (2) confirmation state machine — a store-backed pending action system with composite keys, typed payloads, configurable TTL expiry, and confirm/reject/timeout lifecycle with ferro-events integration. Both are reusable framework primitives, not gestiscilo-specific.

</domain>

<decisions>
## Implementation Decisions

### AI Classification API Shape
- Provider trait abstraction: `ClassificationProvider` trait with `classify` method. Ship with `AnthropicProvider` implementation. Future providers (OpenAI, local models) can be added without changing caller code
- Async-only: all classification calls are async (reqwest async client). No blocking mode — framework handlers are already async
- Low-confidence handling: return error variant with the best guess + score, not silent failure. Callers decide how to handle (ask user to clarify, fallback to default, log for prompt improvement). System must be designed so callers can feed corrections back for future improvement
- Configurable retry: `ClassifierConfig` has `max_retries` (default 1) and `retry_delay` for transient API errors (timeout, network failure). On permanent errors (auth, bad request), fail immediately
- No fallback chain: classifier tries the configured provider. On failure, returns error. Callers handle fallback logic in their own handlers

### Confirmation Lifecycle
- Pluggable store backend: `ConfirmationStore` trait with DashMap in-memory default + optional Redis implementation for persistence across restarts
- Typed payload: `PendingAction<T: Serialize + DeserializeOwned>` stores the action data alongside the key. On confirm, caller gets back the stored T — avoids re-fetching context
- Per-action TTL with global default: global default TTL (e.g., 60s) overridable per pending action. "Confirm delete within 30s" vs "Confirm transfer within 120s"
- TTL expiry via tokio::spawn for in-memory store. On expiry, fire `ConfirmationExpired` ferro-events event for observability (logging, analytics, user notification)
- Lifecycle: request_confirmation() → pending → confirm(key) | reject(key) | timeout → expired event

### Crate Architecture
- Single `"ai"` feature flag in framework Cargo.toml covering both primitives
- MCP introspection tools: `list_pending_confirmations` + `test_classifier` for debugging during development
- Documentation in `docs/src/features/`

### Claude's Discretion
- Crate split (one crate vs two) — decide based on coupling analysis between classification and confirmation
- API shape: generic struct `Classifier<T>` vs trait-based per domain — pick what fits Ferro patterns
- Classification result shape: whether to include confidence/reasoning in result type or keep it minimal
- Key design for confirmations: string composite vs typed struct
- CLI scaffolding: whether to add `ferro make:*` commands — evaluate if there's enough boilerplate to justify
- Logging strategy: tracing spans + optional on_result callback for prompt improvement tracking
- Cost guard: whether to build rate limiting into the classifier or rely on existing Ferro rate limiter middleware
- Listing by scope: whether ConfirmationStore supports `list_pending(scope)` or point lookups only — decide based on MCP tool needs

</decisions>

<specifics>
## Specific Ideas

- "System must be able to improve" — low-confidence results should be observable and feedable back into prompt tuning, not silently discarded
- Driven by gestiscilo.it v2.4 (owner WhatsApp commands) but extracted as generic framework primitives
- ConfirmationExpired event enables notifying users that their pending action timed out (e.g., WhatsApp message: "Your delete request expired, please confirm again")

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-cli/src/ai.rs` — Existing blocking Anthropic API client with `call_anthropic()`. Reference for API interaction patterns, but new crate will use async reqwest
- `ferro-events` — Event dispatcher for `ConfirmationExpired` event integration
- `ferro-queue` — `TenantScopeProvider` trait pattern is a reference for the `ConfirmationStore` trait design
- `DashMap` already used conceptually in codebase (CONVENTIONS.md recommends it over Arc<Mutex> for shared state)
- `moka` cache used in TenantLookup and ThemeMiddleware — pattern for TTL-based storage if applicable

### Established Patterns
- Feature-gated crates: `#[cfg(feature = "...")]` with re-exports in `framework/src/lib.rs` (projections, stripe, theme, json-ui, inertia)
- New crate conventions: thiserror Error enum, builder APIs, workspace member in Cargo.toml
- Provider/trait abstraction: `ClassificationProvider` mirrors `TenantResolver`, `ThemeResolver`, `StorageDriver` patterns
- Async trait methods: `#[async_trait]` for trait-based async APIs
- MCP tools: `ferro-mcp/src/tools/` with tool registration in `ferro-mcp/src/service.rs`

### Integration Points
- `framework/src/lib.rs` — Feature-gated re-exports behind `#[cfg(feature = "ai")]`
- `framework/Cargo.toml` — Optional dependency on new crate(s) with `"ai"` feature flag
- `ferro-mcp/src/tools/` — New MCP introspection tools
- `ferro-events` — ConfirmationExpired event type
- `.github/workflows/publish.yml` — Add new crate(s) to appropriate publish wave
- `docs/src/features/` — Documentation for both primitives

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 100-ai-structured-classification-confirmation-primitives*
*Context gathered: 2026-03-22*
