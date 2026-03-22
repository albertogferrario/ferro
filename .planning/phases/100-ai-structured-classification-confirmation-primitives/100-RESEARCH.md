# Phase 100: AI Structured Classification & Confirmation Primitives - Research

**Researched:** 2026-03-22
**Domain:** Anthropic structured outputs API + DashMap TTL state machine
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**AI Classification API Shape:**
- Provider trait abstraction: `ClassificationProvider` trait with `classify` method. Ship with `AnthropicProvider` implementation. Future providers (OpenAI, local models) can be added without changing caller code.
- Async-only: all classification calls are async (reqwest async client). No blocking mode.
- Low-confidence handling: return error variant with the best guess + score, not silent failure. Callers decide how to handle. System must be designed so callers can feed corrections back for future improvement.
- Configurable retry: `ClassifierConfig` has `max_retries` (default 1) and `retry_delay` for transient API errors (timeout, network failure). On permanent errors (auth, bad request), fail immediately.
- No fallback chain: classifier tries the configured provider. On failure, returns error. Callers handle fallback logic.

**Confirmation Lifecycle:**
- Pluggable store backend: `ConfirmationStore` trait with DashMap in-memory default + optional Redis implementation for persistence across restarts.
- Typed payload: `PendingAction<T: Serialize + DeserializeOwned>` stores the action data alongside the key. On confirm, caller gets back the stored T.
- Per-action TTL with global default: global default TTL (e.g., 60s) overridable per pending action.
- TTL expiry via tokio::spawn for in-memory store. On expiry, fire `ConfirmationExpired` ferro-events event for observability.
- Lifecycle: request_confirmation() → pending → confirm(key) | reject(key) | timeout → expired event.

**Crate Architecture:**
- Single `"ai"` feature flag in framework Cargo.toml covering both primitives.
- MCP introspection tools: `list_pending_confirmations` + `test_classifier` for debugging.
- Documentation in `docs/src/features/`.

### Claude's Discretion

- Crate split (one crate vs two) — decide based on coupling analysis between classification and confirmation.
- API shape: generic struct `Classifier<T>` vs trait-based per domain — pick what fits Ferro patterns.
- Classification result shape: whether to include confidence/reasoning in result type or keep it minimal.
- Key design for confirmations: string composite vs typed struct.
- CLI scaffolding: whether to add `ferro make:*` commands — evaluate if there's enough boilerplate to justify.
- Logging strategy: tracing spans + optional on_result callback for prompt improvement tracking.
- Cost guard: whether to build rate limiting into the classifier or rely on existing Ferro rate limiter middleware.
- Listing by scope: whether ConfirmationStore supports `list_pending(scope)` or point lookups only — decide based on MCP tool needs.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AI-01 | `ClassificationProvider` trait with `classify` async method | Verified: async_trait pattern used in TenantResolver, ThemeResolver, StorageDriver |
| AI-02 | `AnthropicProvider` implementation using structured JSON output (`output_config.format`) | Verified: Anthropic structured outputs API documented; existing `ferro-cli/src/ai.rs` is the reference implementation (blocking, to be made async) |
| AI-03 | `ClassifierConfig` with model, max_tokens, max_retries, retry_delay, confidence_threshold fields | Verified: mirrors StripeConfig + TenantLookup config patterns |
| CONF-01 | `ConfirmationStore` trait with request_confirmation, confirm, reject, list_pending methods | Verified: CacheStore trait is the structural template |
| CONF-02 | `InMemoryConfirmationStore` with DashMap + tokio::spawn TTL expiry | Verified: DashMap v6 already in workspace; broadcast uses the same pattern |
| CONF-03 | `ConfirmationExpired` event integrated with ferro-events dispatch | Verified: Event trait + dispatch pattern from ferro-events |
</phase_requirements>

---

## Summary

Phase 100 adds two independent but co-deployed framework primitives: a provider-abstracted AI classification wrapper and a DashMap-backed confirmation state machine.

The AI classification system wraps the Anthropic structured outputs API (`output_config.format.type = "json_schema"`), available as of early 2026 on Opus 4.6, Sonnet 4.6, Sonnet 4.5, Opus 4.5, and Haiku 4.5. The existing `ferro-cli/src/ai.rs` is the reference implementation in this codebase but uses blocking reqwest and raw text output. Phase 100 moves to async reqwest and the new `output_config` field for guaranteed schema compliance.

The confirmation store is a straightforward DashMap-based state machine. DashMap v6 is already in the workspace (framework, ferro-broadcast, ferro-cache, ferro-storage). The critical pitfall is holding DashMap guards across `.await` points — the ferro-broadcast codebase explicitly documents this with `drop(channel); // Release DashMap guard before await`. TTL expiry uses `tokio::spawn` + `tokio::time::sleep`, then fires a ferro-events `ConfirmationExpired` event.

**Primary recommendation:** Single crate `ferro-ai` containing both primitives — they share no code but share a feature flag and publish wave. One new Wave 1 crate, one `"ai"` feature flag in `framework/Cargo.toml`, four MCP tools.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `reqwest` | `0.12` | Async HTTP client for Anthropic API calls | Tokio-native, already used throughout workspace |
| `serde_json` | `1` | JSON schema serialization/deserialization | Workspace standard, `preserve_order` feature |
| `dashmap` | `6` | Lock-free concurrent HashMap for pending confirmations | Already in workspace (framework, ferro-broadcast, ferro-cache) |
| `tokio` | `1` | Async runtime + time::sleep for TTL expiry | Workspace standard |
| `async-trait` | `0.1` | `#[async_trait]` on ClassificationProvider + ConfirmationStore | Workspace standard for trait-based async |
| `thiserror` | `2` | Error enum derivation | New leaf crate convention (ferro-lang, ferro-stripe, ferro-theme all use v2) |
| `serde` | `1` | Serialize/DeserializeOwned bounds on PendingAction<T> | Workspace standard |
| `tracing` | `0.1` | Structured logging for classifier calls + TTL events | Workspace standard |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `ferro-events` | workspace | ConfirmationExpired event dispatch | In-memory store TTL expiry callback |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `dashmap` (in-memory store) | `moka` | moka has native TTL eviction, but per-expiry event callbacks require custom logic anyway; DashMap + tokio::spawn is simpler for this use case |
| `tokio::spawn` TTL | `tokio_util::time::DelayQueue` | DelayQueue is more accurate but requires tokio-util dep; spawn + sleep is the established Ferro pattern |
| `output_config.format` | tool use with `strict: true` | Tool use structured output also valid; `output_config` is simpler for classification (no tool invocation round-trip) |

**Installation:**
```bash
# New crate — add to workspace Cargo.toml members
# framework/Cargo.toml: ferro-ai = { path = "../ferro-ai", optional = true }
# ferro-ai/Cargo.toml: reqwest, serde_json, dashmap, tokio, async-trait, thiserror, serde, tracing, ferro-events
```

---

## Architecture Patterns

### Recommended Crate Structure

One crate `ferro-ai` containing both primitives (they ship together, share a feature flag, share an Error enum):

```
ferro-ai/
├── Cargo.toml
├── src/
│   ├── lib.rs              # pub use exports
│   ├── error.rs            # Error enum (thiserror)
│   ├── classifier/
│   │   ├── mod.rs          # Classifier<T> struct, ClassifierConfig
│   │   ├── provider.rs     # ClassificationProvider trait
│   │   └── anthropic.rs    # AnthropicProvider
│   └── confirmation/
│       ├── mod.rs          # ConfirmationStore trait, PendingAction<T>
│       ├── store.rs        # InMemoryConfirmationStore (DashMap)
│       └── events.rs       # ConfirmationExpired event type
```

### Reasoning: one crate vs two

Coupling analysis: classification and confirmation share zero code. However:
- They are co-deployed under a single feature flag per CONTEXT.md
- They share an Error enum (cleaner than two small crates with two Error enums)
- Publish workflow complexity does not justify splitting
- Precedent: `ferro-theme` ships tokens + templates together despite being conceptually separate

**Use one crate.**

### Pattern 1: ClassificationProvider trait

**What:** Async trait that any AI backend implements.
**When to use:** All AI classification calls go through this; allows swapping AnthropicProvider for a test double.

```rust
// Source: mirrors TenantResolver pattern in framework/src/tenant/
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait ClassificationProvider: Send + Sync {
    async fn classify_raw(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
        config: &ClassifierConfig,
    ) -> Result<serde_json::Value, Error>;
}
```

### Pattern 2: Classifier<T> generic facade

**What:** Generic wrapper taking `T: DeserializeOwned` — the return type is the schema.
**When to use:** Callers call `Classifier::new(provider, config).classify(system, user).await` and get back `ClassificationResult<T>`.

```rust
// Source: mirrors Stripe facade pattern (OnceLock not used here — Classifier is instantiated by caller)
pub struct Classifier<T: DeserializeOwned + Serialize> {
    provider: Arc<dyn ClassificationProvider>,
    config: ClassifierConfig,
    _phantom: std::marker::PhantomData<T>,
}

pub struct ClassifierConfig {
    pub model: String,              // default: "claude-sonnet-4-6"
    pub max_tokens: u32,            // default: 1024
    pub max_retries: u32,           // default: 1
    pub retry_delay: Duration,      // default: 1s
    pub confidence_threshold: f64,  // default: 0.7 (callers inspect result.confidence)
}

pub struct ClassificationResult<T> {
    pub value: T,
    pub confidence: Option<f64>,   // provider may return confidence metadata
    pub raw_json: serde_json::Value, // for prompt improvement feedback
}

pub enum ClassificationError {
    LowConfidence { best_guess: serde_json::Value, confidence: f64 },
    ProviderError(Error),
    Deserialization(serde_json::Error),
}
```

**Note on confidence:** The Anthropic `output_config.format` API does NOT return a confidence score in the response — it returns structured JSON matching the schema. To get a confidence signal, the schema itself must include a `confidence: f64` field. The `ClassificationResult.confidence` field is populated from that schema field if present, otherwise `None`. This is the correct approach for the WhatsApp command classification use case: include `confidence` in the schema definition.

### Pattern 3: AnthropicProvider async implementation

**What:** Calls `https://api.anthropic.com/v1/messages` with `output_config.format` field.
**When to use:** Default provider.

```rust
// Source: ferro-cli/src/ai.rs (blocking reference) + Anthropic structured outputs docs
// Key changes from blocking reference:
// - reqwest::Client (async, not blocking::Client)
// - output_config.format.type = "json_schema" replaces assistant prefill
// - Error detection: 400/401/422 = permanent (no retry), 429/500/529 = transient (retry)

let body = serde_json::json!({
    "model": config.model,
    "max_tokens": config.max_tokens,
    "system": [{
        "type": "text",
        "text": system_prompt,
        "cache_control": {"type": "ephemeral"}
    }],
    "messages": [{"role": "user", "content": user_prompt}],
    "output_config": {
        "format": {
            "type": "json_schema",
            "schema": schema
        }
    }
});
```

### Pattern 4: ConfirmationStore trait + InMemoryConfirmationStore

**What:** Trait with DashMap implementation. PendingAction<T> stores serialized payload as `serde_json::Value` (type-erased in store, restored on confirm).
**When to use:** Any handler that needs "confirm destructive action within N seconds."

```rust
// Source: CacheStore trait pattern (ferro-cache/src/cache.rs)
#[async_trait]
pub trait ConfirmationStore: Send + Sync {
    async fn request_confirmation(
        &self,
        key: &str,
        payload: serde_json::Value,
        ttl: Duration,
    ) -> Result<(), Error>;

    async fn confirm(&self, key: &str) -> Result<Option<serde_json::Value>, Error>;
    async fn reject(&self, key: &str) -> Result<bool, Error>;
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, Error>;
    async fn list_pending(&self) -> Result<Vec<PendingActionInfo>, Error>;
}

// Key design: composite key as String
// "whatsapp:user123:delete_expense:abc" — callers join segments with ":"
// No typed struct needed — String composite is flexible and testable
```

**TTL expiry with DashMap — critical pattern:**

```rust
// Source: ferro-broadcast/src/broadcaster.rs "Release DashMap guard before await" comment
// PITFALL: Do NOT hold DashMap guard across .await — causes deadlock in tokio

impl InMemoryConfirmationStore {
    fn spawn_ttl_expiry(&self, key: String, ttl: Duration) {
        let store = self.inner.clone(); // Arc<DashMap<...>>
        tokio::spawn(async move {
            tokio::time::sleep(ttl).await;
            // Remove FIRST, then dispatch event — never hold guard across dispatch
            let removed = store.remove(&key);
            if removed.is_some() {
                // ConfirmationExpired is fire-and-forget
                dispatch_sync(ConfirmationExpired { key });
            }
        });
    }
}
```

### Pattern 5: ConfirmationExpired event

**What:** Ferro event type dispatched on TTL expiry.
**When to use:** Listeners (e.g., WhatsApp notifier) subscribe to this event.

```rust
// Source: ferro-events Event trait pattern
#[derive(Clone, Debug)]
pub struct ConfirmationExpired {
    pub key: String,
    pub expired_at: chrono::DateTime<chrono::Utc>,
}

impl Event for ConfirmationExpired {
    fn name(&self) -> &'static str { "ConfirmationExpired" }
}
```

### Pattern 6: Framework feature-gated re-exports

**What:** `framework/src/lib.rs` re-exports under `#[cfg(feature = "ai")]`.
**When to use:** User adds `ferro-rs = { features = ["ai"] }` to their Cargo.toml.

```rust
// Source: framework/src/lib.rs existing stripe/projections/theme pattern
#[cfg(feature = "ai")]
pub use ferro_ai::{
    AnthropicProvider, ClassificationResult, Classifier, ClassifierConfig,
    ClassificationProvider, ConfirmationExpired, ConfirmationStore,
    Error as AiError, InMemoryConfirmationStore, PendingActionInfo,
};
```

### Recommended Project Structure (ferro-ai crate)

```
ferro-ai/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs
    ├── classifier/
    │   ├── mod.rs          # Classifier<T>, ClassifierConfig, ClassificationResult
    │   ├── provider.rs     # ClassificationProvider trait
    │   └── anthropic.rs    # AnthropicProvider (reqwest async)
    └── confirmation/
        ├── mod.rs          # ConfirmationStore trait, PendingAction<T>, PendingActionInfo
        ├── store.rs        # InMemoryConfirmationStore
        └── events.rs       # ConfirmationExpired
```

### Anti-Patterns to Avoid

- **Holding DashMap guard across `.await`:** DashMap guards use `parking_lot::RwLock` internally. Tokio's task scheduler may not release the guard before preempting. Pattern: assign to local, `drop(guard)` explicitly, THEN `.await`. ferro-broadcast already documents this.
- **Returning confidence from output_config response:** Anthropic's structured output endpoint returns only the JSON matching the schema — no metadata about confidence. Confidence must be part of the schema itself (user's responsibility to include it).
- **Blocking reqwest in async context:** ferro-cli uses `reqwest::blocking::Client`. ferro-ai MUST use `reqwest::Client` (async). Mixing blocking and async panics inside tokio.
- **Storing typed `PendingAction<T>` in DashMap:** Type-erasing to `serde_json::Value` in the store allows the trait to be object-safe and the DashMap to hold heterogeneous payloads. Caller deserializes back to `T` after `confirm()`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP client for API | Custom TCP connection | `reqwest::Client` | TLS, connection pooling, timeout, redirect handling |
| TTL eviction with callbacks | Custom timer loop | `tokio::time::sleep` + `tokio::spawn` | Tokio scheduler is accurate; no added dependency |
| Schema validation of API response | Custom JSON validator | `output_config.format` from Anthropic API | API guarantees schema compliance; no need to re-validate |
| Event dispatch on TTL | Custom callback system | `ferro_events::dispatch_sync()` | Already in codebase, subscribers can react without coupling |
| Concurrent map for pending actions | `Arc<Mutex<HashMap>>` | `DashMap` | Sharded lock, faster under concurrent read/write |

**Key insight:** The Anthropic structured output guarantee means the response body is already valid JSON matching the schema. No post-processing validation is needed — just `serde_json::from_str::<T>()`.

---

## Common Pitfalls

### Pitfall 1: DashMap deadlock on await
**What goes wrong:** Code holds a `DashMap` ref/guard across an `.await` point. Tokio parks the task, but the `parking_lot` mutex inside DashMap remains locked. When another task tries to access the same shard, it deadlocks.
**Why it happens:** DashMap uses `parking_lot::RwLock`, which is not async-aware.
**How to avoid:** Always drop the guard before awaiting. In the TTL expiry path: call `store.remove(&key)` (which drops the guard immediately), capture the returned value, then dispatch the event.
**Warning signs:** Task hangs indefinitely under concurrent load; `tokio-console` shows tasks blocked on a mutex.

### Pitfall 2: Retrying on 400/422 (bad request) errors
**What goes wrong:** Classifier retries on schema validation failures or bad API keys, burning quota.
**Why it happens:** Generic retry logic treats all non-200 responses as retryable.
**How to avoid:** Partition error codes — permanent errors (400, 401, 403, 422) fail immediately. Transient errors (429, 500, 503, 529) trigger retry with delay.
**Warning signs:** Logs show multiple identical 400-series requests in sequence.

### Pitfall 3: Phantom TTL — spawn outlives store
**What goes wrong:** `InMemoryConfirmationStore` is dropped but spawned TTL tasks still hold an `Arc` clone of the inner DashMap. Tasks complete but fire `ConfirmationExpired` for a store that no longer logically exists.
**Why it happens:** `tokio::spawn` tasks live for the duration of the runtime, not the struct that spawned them.
**How to avoid:** Use `Arc::strong_count` awareness or `tokio::task::AbortHandle` stored per key. Simpler: accept the behavior — the event fires harmlessly, no listeners are attached if the store is gone.
**Warning signs:** `ConfirmationExpired` events firing after tests complete.

### Pitfall 4: Non-Send type in PendingAction<T>
**What goes wrong:** `Classifier<T>` or `ConfirmationStore` stored in a handler but `T` is not `Send`, causing compile errors.
**Why it happens:** `tokio::spawn` requires `Send`. DashMap is `Send` but the values stored must also be `Send`.
**How to avoid:** Store `serde_json::Value` (always `Send + Sync`) in DashMap. Type parameters are only at the public API boundary; the store is type-erased internally.

### Pitfall 5: Schema serde mismatches
**What goes wrong:** Rust struct derives `Serialize/Deserialize` but uses `#[serde(rename_all = "camelCase")]` while the JSON schema uses `snake_case`. API returns JSON Claude can't map to the schema.
**Why it happens:** `schemars` generates schema from the derive, but the `output_config.schema` submitted to Anthropic must match the actual serde output format.
**How to avoid:** Generate the JSON schema via `schemars::schema_for!(T)` and submit that schema to `output_config.format.schema`. This guarantees alignment.

---

## Code Examples

Verified patterns from official sources and codebase inspection:

### Anthropic Structured Output Request Body
```rust
// Source: https://platform.claude.com/docs/en/build-with-claude/structured-outputs
// Verified: available on Opus 4.6, Sonnet 4.6, Sonnet 4.5, Opus 4.5, Haiku 4.5

let schema = schemars::schema_for!(MyOutputType);
let schema_value = serde_json::to_value(&schema).unwrap();

let body = serde_json::json!({
    "model": "claude-sonnet-4-6",
    "max_tokens": 1024,
    "system": [{
        "type": "text",
        "text": system_prompt,
        "cache_control": {"type": "ephemeral"}
    }],
    "messages": [{"role": "user", "content": user_prompt}],
    "output_config": {
        "format": {
            "type": "json_schema",
            "schema": schema_value
        }
    }
});
```

### Error Classification for Retry Logic
```rust
// Source: Anthropic API error codes (official docs)
fn is_permanent_error(status: u16) -> bool {
    matches!(status, 400 | 401 | 403 | 404 | 422)
}

fn is_transient_error(status: u16) -> bool {
    matches!(status, 429 | 500 | 503 | 529)
}
```

### DashMap TTL Expiry (safe pattern)
```rust
// Source: ferro-broadcast/src/broadcaster.rs guard-drop pattern + ferro-events dispatch_sync
fn spawn_expiry(store: Arc<DashMap<String, StoredAction>>, key: String, ttl: Duration) {
    tokio::spawn(async move {
        tokio::time::sleep(ttl).await;
        let removed = store.remove(&key); // Guard is dropped immediately by remove()
        if removed.is_some() {
            dispatch_sync(ConfirmationExpired {
                key,
                expired_at: chrono::Utc::now(),
            });
        }
    });
}
```

### Feature-Gated Framework Re-export
```rust
// Source: framework/src/lib.rs — stripe/projections/theme pattern
// framework/Cargo.toml: ferro-ai = { path = "../ferro-ai", version = "0.1", optional = true }
// features: ai = ["dep:ferro-ai"]
#[cfg(feature = "ai")]
pub use ferro_ai::{
    AnthropicProvider, ClassificationResult, Classifier, ClassifierConfig,
    ClassificationProvider, ConfirmationExpired, ConfirmationStore, Error as AiError,
    InMemoryConfirmationStore, PendingActionInfo,
};
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Prompt engineering + regex parsing for structured output | `output_config.format.type = "json_schema"` | Early 2026 (available on Claude Opus 4.6+) | Schema-guaranteed JSON — no post-processing validation needed |
| Blocking reqwest in ferro-cli | Async reqwest in ferro-ai | This phase | Correct for Tokio; blocking would panic |
| Assistant prefill for format control | Native `output_config` | This phase | Cleaner, model-native, no fragile prefill tricks |

**Deprecated/outdated:**
- `ferro-cli/src/ai.rs` assistant prefill pattern: The `"role": "assistant", "content": "//!"` trick in the CLI is valid but unnecessary when using `output_config.format`. Do not replicate in ferro-ai.
- `reqwest::blocking::Client`: Used in ferro-cli because the CLI is synchronous. ferro-ai is async — must use `reqwest::Client`.

---

## Open Questions

1. **schemars version for schema generation**
   - What we know: `ferro-projections` uses `schemars = "1"`. The workspace does not yet have schemars in ferro-ai's dependency set.
   - What's unclear: Whether to add schemars as a dependency of ferro-ai or require callers to pass `serde_json::Value` schemas manually.
   - Recommendation: Do not depend on schemars in ferro-ai. The `classify` method takes `serde_json::Value` as the schema argument. Callers who want to generate the schema from a Rust type use `schemars::schema_for!()` themselves. Keeps ferro-ai lean.

2. **Redis ConfirmationStore scope**
   - What we know: CONTEXT.md specifies "optional Redis implementation" as a future path.
   - What's unclear: Whether it belongs in Phase 100 or is deferred.
   - Recommendation: Ship only `InMemoryConfirmationStore` in Phase 100. Redis implementation is a future plan. The trait design must be Redis-compatible (async, string keys, JSON values).

3. **MCP test_classifier security**
   - What we know: `test_classifier` MCP tool should allow debugging — but it would make real API calls.
   - What's unclear: Whether to require ANTHROPIC_API_KEY in MCP server context or mock.
   - Recommendation: `test_classifier` runs against the configured AnthropicProvider using the ambient `ANTHROPIC_API_KEY`. Document as a "costs tokens" tool. Same pattern as existing MCP tools that read from live DB.

---

## Validation Architecture

> `workflow.nyquist_validation` is not set to false in `.planning/config.json` — validation section included.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` |
| Config file | None (cargo test) |
| Quick run command | `cargo test -p ferro-ai` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AI-01 | ClassificationProvider trait is object-safe | unit | `cargo test -p ferro-ai -- classifier::provider` | Wave 0 |
| AI-02 | AnthropicProvider sends correct output_config body structure | unit (mock) | `cargo test -p ferro-ai -- classifier::anthropic` | Wave 0 |
| AI-03 | ClassifierConfig defaults, retry logic distinguishes permanent vs transient errors | unit | `cargo test -p ferro-ai -- classifier::config` | Wave 0 |
| CONF-01 | ConfirmationStore trait operations: request, confirm, reject, list_pending | unit | `cargo test -p ferro-ai -- confirmation::store` | Wave 0 |
| CONF-02 | InMemoryConfirmationStore TTL expiry removes entry and dispatches event | unit (tokio::test + sleep) | `cargo test -p ferro-ai -- confirmation::store::ttl` | Wave 0 |
| CONF-03 | ConfirmationExpired implements ferro_events::Event trait | unit | `cargo test -p ferro-ai -- confirmation::events` | Wave 0 |

**Note on AI-02 testing:** The `AnthropicProvider` test MUST mock the HTTP call (not make real API calls in CI). Pattern: inject a reqwest mock or use a trait for the HTTP client. Simplest: test the request body construction separately from the actual HTTP round-trip. The test verifies the `serde_json` body contains the correct `output_config.format` structure.

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-ai`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `ferro-ai/src/lib.rs` — new crate, does not exist
- [ ] `ferro-ai/Cargo.toml` — new crate, does not exist
- [ ] `ferro-ai/src/error.rs` — new
- [ ] `ferro-ai/src/classifier/mod.rs` — new
- [ ] `ferro-ai/src/classifier/provider.rs` — new
- [ ] `ferro-ai/src/classifier/anthropic.rs` — new
- [ ] `ferro-ai/src/confirmation/mod.rs` — new
- [ ] `ferro-ai/src/confirmation/store.rs` — new
- [ ] `ferro-ai/src/confirmation/events.rs` — new

All files are new — this is a new crate. No existing test infrastructure to gap-fill; the crate must be created from scratch in Wave 0 of the first plan.

---

## Sources

### Primary (HIGH confidence)
- Anthropic Structured Outputs official docs — https://platform.claude.com/docs/en/build-with-claude/structured-outputs — verified `output_config.format.type = "json_schema"` request format, model availability (Opus 4.6, Sonnet 4.6, Sonnet 4.5, Opus 4.5, Haiku 4.5), Zero Data Retention behavior
- `ferro-cli/src/ai.rs` — blocking reference implementation in this codebase; source of truth for headers, model env var, cache_control, temperature 0.2
- `ferro-broadcast/src/broadcaster.rs` — DashMap guard-drop pattern (`drop(channel); // Release DashMap guard before await`)
- `ferro-events/src/dispatcher.rs` — Event trait + dispatch_sync pattern
- `framework/src/lib.rs` — feature-gated re-export pattern for new crates
- `ferro-cache/src/cache.rs` — CacheStore trait as structural template for ConfirmationStore
- `framework/Cargo.toml` — existing feature flags (stripe, projections, theme), dashmap v6 already present

### Secondary (MEDIUM confidence)
- DashMap docs + deadlock issue tracker — https://github.com/xacrimon/dashmap — confirmed parking_lot-based locking, deadlock risk on await (multiple confirmed issues)
- Anthropic API HTTP status codes — permanent (400/401/403/422) vs transient (429/500/529) — verified via official Anthropic error handling docs referenced in structured outputs page

### Tertiary (LOW confidence)
- None — all critical findings verified against official sources or direct codebase inspection

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already in workspace or verified in official docs
- Architecture: HIGH — patterns directly mirror existing ferro-stripe, ferro-cache, ferro-events implementations
- Pitfalls: HIGH — DashMap deadlock is documented in ferro-broadcast itself; API retry rules from official Anthropic docs

**Research date:** 2026-03-22
**Valid until:** 2026-04-22 (Anthropic API is stable; DashMap patterns are stable)
