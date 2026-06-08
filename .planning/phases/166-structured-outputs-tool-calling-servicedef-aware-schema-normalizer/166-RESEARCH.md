# Phase 166: Structured Outputs, Tool Calling & ServiceDef-aware Schema Normalizer — Research

**Researched:** 2026-06-08
**Domain:** Rust crate extension — JSON Schema normalization, provider structured-output APIs, tool-calling loop, async handler ergonomics
**Confidence:** HIGH (all critical facts verified against source files and official docs)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Public free function `ferro_ai::complete::<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>` where `T: schemars::JsonSchema + serde::DeserializeOwned`. Internal flow: `schema_for::<T>()` → `schema::for_structured_output(...)` → set `CompletionRequest.schema` → `client.complete(...)` → parse JSON into T.
- **D-02:** A request-taking variant (`complete_into::<T>`) is Claude's discretion.
- **D-03:** `ferro_ai::schema::for_structured_output(...) -> serde_json::Value`. Targets schemars 1.x (Draft 2020-12: `$defs` + `#/$defs/...` refs; also handles legacy `#/definitions/`). Resolves `$ref`/`$defs` inline recursively with cycle guard, adds `additionalProperties: false` to every object schema, strips Anthropic-rejected constraints.
- **D-04:** PRESERVE: `type`, `properties`, `items`, `required`, `enum`, `additionalProperties`, `oneOf`/`anyOf` for tagged variants. STRIP: rejected Anthropic keywords. `enum` preservation is non-negotiable.
- **D-05:** Normalizer input parameter type is Claude's discretion.
- **D-06 (central):** The ServiceDef-aware path closes `FieldMeaning` and `Intent` enums — drops the `Custom(String)` untagged branch from the LLM-facing schema and emits a closed `{"type":"string","enum":[...known snake_case variants...]}`. Rust types keep `Custom` for Rust deserialization; LLM cannot produce a non-known value.
- **D-07:** Detection at runtime by inspecting generated schema's `$defs` for ferro-projections type names (`ServiceDef`, `FieldMeaning`, `Intent`, `Cardinality`, `ActionDef`, `GuardDef`, `StateDef`). No second public entry point, no stable-Rust specialization.
- **D-08:** Valid-value source is `ferro-projections` own schema output, not a hardcoded list in ferro-ai.
- **D-09:** SC#3 structural-guarantee test: construct ServiceDef JSON with invalid `FieldMeaning` and invalid `Intent`, validate against ServiceDef-aware normalized schema, assert FAILS; valid ServiceDef passes. Uses `jsonschema` crate (dev-dep).
- **D-10:** Trade-off acknowledged: closing enums removes Custom values from LLM path. Intended for v12.1.
- **D-11:** `ToolDef { name, description, parameters_schema, handler }` where handler is `Box<dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ToolError>> + Send + Sync>`.
- **D-12:** `max_iterations: u32` required at construction (`ToolRegistry::new(max_iterations)`). No Default, no zero-arg constructor, no unbounded override. Warning at 5, error at hard cap.
- **D-13:** `ToolError { message: String }` — model-legible. Never raw panics or DB strings.
- **D-14:** Tool-use goes through `LlmClient` layer, not parallel HTTP. `CompletionRequest` extended with tools support. Planner decides exact shape.

### Claude's Discretion
- `complete_into::<T>` escape hatch signature (D-02)
- Normalizer input parameter type — schemars `Schema` vs `serde_json::Value` (D-05)
- Internal module layout (`schema.rs`, `tools/` submodule, etc.)
- JSON-Schema validator crate for SC#3 test (recommended: `jsonschema` at 0.46.x)
- Exact client-layer tool-extension shape (D-14)
- Extraction technique for projection valid-value set (D-08)

### Deferred Ideas (OUT OF SCOPE)
- Renderer-as-tool adapter (Phase 171+)
- Tool calling in streaming context (v12.1 future)
- Conversation memory / multi-session history
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AISDK-02 | `ferro_ai::complete::<T>()` backed by a JSON Schema normalizer that resolves schemars `$ref`/`$defs` incompatibility with provider structured-output APIs. ServiceDef-aware: when T is ServiceDef (or contains one), normalizer emits constrained schema that locks LLM to valid projection shapes. | Anthropic schema constraints documented (§Schema Normalizer). schemars 1.x output shape verified. ServiceDef-aware path: enum closing algorithm designed (§ServiceDef-aware Path). All confirmed against actual source files. |
| AISDK-03 | Developer can register Rust functions as AI tools; SDK dispatches tool-use calls automatically with hard `max_iterations` guard. | Anthropic tool-use request/response format documented (§Tool Calling). OpenAI function-calling format confirmed. Async handler pattern documented. Loop termination via `stop_reason` confirmed. |
</phase_requirements>

---

## Summary

Phase 166 builds three distinct but interdependent capabilities on top of the Phase 165 `LlmClient` foundation: (1) a typed ergonomic `complete::<T>()` wrapper, (2) a JSON Schema normalizer that makes schemars 1.x output compatible with Anthropic and OpenAI structured-output APIs, and (3) a `ToolRegistry` with an enforced `max_iterations` guard. The hardest part is the ServiceDef-aware path: `FieldMeaning` and `Intent` each carry a `#[serde(untagged)] Custom(String)` variant that causes schemars to emit an open `anyOf` schema — the normalizer must detect this and replace it with a closed `enum` constraint to fulfill SC#3.

The good news: almost all infrastructure is already in place. `CompletionRequest.schema` exists (Phase 165 D-11), `ferro-projections` types all derive `JsonSchema`, `serde_json` is already a dependency of `ferro-ai`, and `jsonschema` 0.46.5 is already in the workspace (used by `ferro-json-ui`). No new external HTTP clients needed — `ToolRegistry::dispatch` reuses the existing provider clients via `CompletionRequest` extension.

The Anthropic structured-output reject-list is now authoritatively documented: `$schema`, `$id`, `title`, `examples`, numeric bounds (`minimum`, `maximum`, `multipleOf`), string bounds (`minLength`, `maxLength`), regex `pattern` (with caveats), recursive schemas, and complex types within `enum`. The `enum` keyword itself is PRESERVED — it is the locking mechanism for the ServiceDef-aware path.

**Primary recommendation:** Implement in four sequential waves: (W0) schema module + normalizer + enum-closing algorithm with tests, (W1) `complete::<T>()` + ServiceDef detection + SC#3 test, (W2) `ToolDef`/`ToolError`/`ToolRegistry` struct definitions + `CompletionRequest` tool extension, (W3) `ToolRegistry::dispatch` loop implementation + iteration tests.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `complete::<T>()` typed wrapper | ferro-ai (library) | — | Thin ergonomic layer over existing `LlmClient::complete`; no framework wiring |
| JSON Schema normalization | ferro-ai::schema (module) | — | Pure data transformation, no IO; owned by ferro-ai as the schema producer |
| ServiceDef-aware enum closing | ferro-ai::schema (module) | ferro-projections (read-only) | Normalizer reads ferro-projections types to derive valid variants; does not call any projection runtime |
| Tool handler execution | Application code | ferro-ai (dispatch loop) | ferro-ai owns the dispatch loop with `max_iterations`; actual handler closures live in application code |
| Provider HTTP for tool calls | ferro-ai::client (existing) | — | D-14: tool-use goes through existing Anthropic/OpenAI clients via `CompletionRequest` extension |
| Tool error surfacing | ferro-ai::tools (module) | — | `ToolError { message }` wraps handler results before sending back to LLM |

---

## Standard Stack

### Core (all already in workspace or direct add)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `schemars` | 1.2.0 [VERIFIED: Cargo.lock] | Generate Draft 2020-12 schemas from Rust types | Already used by ferro-projections; all projection types derive `JsonSchema` |
| `serde_json` | 1.x [VERIFIED: ferro-ai/Cargo.toml] | Schema manipulation as `serde_json::Value` | Already a dependency of ferro-ai |
| `ferro-projections` | workspace [VERIFIED: source] | Projection type definitions to close enums against | Locked by D-08; no cycle (ferro-projections has no ferro-ai dep) |
| `futures` | 0.3 [VERIFIED: ferro-ai/Cargo.toml] | `BoxFuture` for async tool handlers | Already a dependency of ferro-ai |

### Supporting (new additions)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `jsonschema` (dev-dep) | 0.46.5 [VERIFIED: Cargo.lock] | Validate JSON instance against schema in SC#3 test | Already in workspace (ferro-json-ui uses it); add as dev-dep to ferro-ai |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `jsonschema` 0.46 | `jsonschema` 0.40+ | Same workspace version avoids double-compilation; 0.46.5 already locked |
| Runtime `$defs` inspection for detection | Trait-based specialization | Trait approach has impl conflict on stable Rust (D-07); runtime inspection is simpler |
| Hand-rolling `$ref` resolution | A ref-resolution crate | No widely-used Rust crate for this; algorithm is ~50 lines; avoids new dependency |

**Installation (new deps to add to ferro-ai/Cargo.toml):**
```toml
[dependencies]
schemars = { version = "1", features = ["derive"] }
ferro-projections = { path = "../ferro-projections", version = "0.2" }

[dev-dependencies]
jsonschema = { version = "0.46", default-features = false }
```

---

## Architecture Patterns

### System Architecture Diagram

```
complete::<T>(client, prompt)
         │
         ├─ schema_for::<T>()                    [schemars 1.x → Draft 2020-12 RootSchema]
         │         └─ RootSchema.to_value()      [→ serde_json::Value with $defs + $refs]
         │
         ├─ schema::for_structured_output(value)
         │         ├─ resolve_refs()             [inline all $ref/$defs recursively, cycle guard]
         │         ├─ add_additional_properties_false()  [every object node]
         │         ├─ strip_rejected_keywords()  [title, $schema, $id, examples, min/maxLength, etc.]
         │         └─ if projection_type_detected($defs)
         │                 └─ close_projection_enums()   [FieldMeaning, Intent → closed enum]
         │
         ├─ CompletionRequest { schema: Some(normalized), messages: [user: prompt], max_tokens }
         │
         ├─ client.complete(request)             [existing AnthropicClient / OpenAiClient]
         │         └─ response text (JSON string)
         │
         └─ serde_json::from_str::<T>(text)      [→ Result<T, Error::Deserialization>]


ToolRegistry::dispatch(messages, client)
         │
         loop (iteration < max_iterations):
         ├─ extend CompletionRequest with tools array
         ├─ client.complete_with_tools(request)
         │         └─ response: text OR tool_use blocks
         │
         ├─ if stop_reason == "end_turn"  → return accumulated messages
         ├─ if stop_reason == "tool_use"  → dispatch each tool_use block
         │         ├─ look up handler in registry by name
         │         ├─ call handler(input) → Result<Value, ToolError>
         │         │         ├─ Ok(value)  → tool_result content
         │         │         └─ Err(e)     → tool_result with ToolError.message
         │         └─ append tool_result messages for next iteration
         ├─ iteration == 5   → warn!(...)
         └─ iteration == max → error!(...); return Error
```

### Recommended Module Layout

```
ferro-ai/src/
├── client/          # Phase 165 — LlmClient trait + provider impls (extended for tools)
│   ├── mod.rs       # CompletionRequest extended: tools: Option<Vec<ToolRequest>>, tool_choice
│   ├── anthropic.rs # build_body extended for tools array + tool_result
│   ├── openai.rs    # build_body extended for function-calling format
│   └── ollama.rs    # tool calls unsupported → Error::Unsupported
├── schema/          # NEW: JSON Schema normalizer
│   └── mod.rs       # for_structured_output(), resolve_refs(), close_projection_enums()
├── tools/           # NEW: ToolDef, ToolRegistry, ToolError
│   └── mod.rs       # ToolDef, ToolRegistry, ToolError, dispatch loop
├── complete.rs      # NEW: complete::<T>(), complete_into::<T>()
├── classifier/      # unchanged
├── confirmation/    # unchanged
├── config.rs        # unchanged
├── error.rs         # extended: SchemaError, ToolError variants
└── lib.rs           # re-export complete, schema::for_structured_output, ToolDef, ToolRegistry, ToolError
```

---

## Research Priority Findings

### 1. Anthropic Structured-Output JSON Schema Constraints [VERIFIED: official docs]

**Supported keywords (PRESERVE in normalizer):**
- `type`, `properties`, `required`, `additionalProperties` (must be `false` for objects)
- `enum` (strings, numbers, bools, or nulls only — no complex objects; this is the locking mechanism)
- `const`, `anyOf`, `allOf` (with `$ref` limitations), `$ref`, `$defs`/`definitions`
- `items` (for typed arrays), `minItems` (values 0 and 1 only)
- String formats: `date-time`, `time`, `date`, `duration`, `email`, `hostname`, `uri`, `ipv4`, `ipv6`, `uuid`
- `default` (for all supported types)

**Rejected keywords (STRIP in normalizer):**
- `$schema`, `$id`, `title`, `examples` — schema metadata, always rejected
- `minimum`, `maximum`, `multipleOf` — numeric bounds
- `minLength`, `maxLength` — string bounds
- `pattern` (backreferences and lookahead/lookbehind) — complex regex rejected; simple patterns may work
- Recursive schemas — not supported (but ferro-projections types are non-recursive after `$ref` inlining)
- External `$ref` (`http://...`) — only internal `#/$defs/` refs supported
- `additionalProperties` set to anything other than `false` (the normalizer always sets it to `false`)
- Complex types within `enum` values

**Important:** Using unsupported keywords results in a **400 error** with details. The SDKs (Python, TypeScript, etc.) perform automatic stripping — this is exactly what the Rust normalizer must implement manually.

**Note on `title`:** Schemars 1.x currently emits `title` fields. The normalizer must strip them.

**Note on `format`:** The supported string formats (`date-time`, `email`, `uri`, etc.) are PRESERVED. Only `format: "int32"`, `format: "float"`, and other non-string formats should be stripped. Research finding: schemars 1.x emits `"format": "int32"` on integer fields and `"format": "float"` on float fields. These must be stripped. [VERIFIED: schemars docs show `"format": "int32"` in example output]

### 2. schemars 1.x Output Shape [VERIFIED: Context7 + ferro-projections tests]

**schemars version in use:** 1.2.0 [VERIFIED: Cargo.lock]

**Top-level output from `schema_for!`:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "TypeName",
  "type": "object",
  "properties": { ... },
  "required": [...],
  "$defs": { "SubType": { ... } }
}
```

**Key fields the normalizer must strip at top level:** `$schema`, `title`
**Key fields to handle:** `$defs` (resolve and remove), refs in the form `#/$defs/TypeName`

**Untagged enum output shape (FieldMeaning, Intent):**

`FieldMeaning` has 18 known unit variants (all serialized as snake_case strings) + `#[serde(untagged)] Custom(String)`. Schemars emits the unit variants as a `type: "string", enum: [...]` inside the first branch of an `anyOf`, and the `Custom(String)` as a second `type: "string"` branch without constraints:

```json
{
  "description": "Semantic field meaning. Known variants: identifier, ...",
  "anyOf": [
    {
      "type": "string",
      "enum": ["identifier", "foreign_key", "entity_name", "email", "phone",
               "url", "image_url", "money", "percentage", "quantity", "status",
               "category", "boolean", "free_text", "created_at", "updated_at",
               "date_time", "sensitive"]
    },
    {
      "type": "string"
    }
  ]
}
```

`Intent` follows the same pattern with 7 known variants: `["browse", "focus", "collect", "process", "summarize", "analyze", "track"]`.

**The closing algorithm:** Replace the `anyOf` with the first branch only (the closed `enum`). The `description` is preserved. This is a pure JSON transformation on the `$defs` entry for `FieldMeaning` / `Intent`. [ASSUMED for exact shape — inferred from schemars docs + existing test confirming `description` at top level; run `schema_for!(FieldMeaning).to_value()` in Wave 0 test to confirm exact shape]

**How to extract known variant values (D-08):** Extract from the first `anyOf` branch's `enum` array in the schemars output itself — no hardcoded list in ferro-ai. This satisfies D-08: the valid values come from ferro-projections' own schema, not a copy.

### 3. $ref/$defs Inline Resolution Algorithm [ASSUMED based on JSON Schema spec + schemars output]

The normalizer must resolve all `#/$defs/TypeName` references inline before removing the `$defs` object. Algorithm:

```
fn resolve_refs(schema: Value, defs: &Map) -> Value:
    if schema is {"$ref": "#/$defs/Name"}:
        return resolve_refs(defs[Name].clone(), defs)  // with visited: HashSet for cycle guard
    else if schema is object:
        return schema with each value replaced by resolve_refs(value, defs)
    else if schema is array:
        return schema with each element replaced by resolve_refs(elem, defs)
    else:
        return schema unchanged
```

**Cycle guard:** Track visited `$def` names in a `HashSet<String>`. If a name is encountered again during its own resolution, emit a `{"type": "object"}` placeholder (or the partially resolved form). The ferro-projections types are well-behaved (no genuine recursion — `StateMachine.states` is `Vec<StateDef>` resolved as `items` + the `$ref` to `StateDef`; after resolution each `StateDef` is a self-contained object).

**After resolution:** Remove the `$defs` key from the root schema. The result is a flat schema with no `$ref` anywhere. [VERIFIED: Anthropic docs say external `$ref` not supported; internal $ref via `$defs` is supported, but resolving inline is safer and avoids any provider-side ref resolution bugs]

### 4. ServiceDef-aware Enum Closing [VERIFIED: source files + schemars docs]

**Detection (D-07):** After calling `schema_for::<T>()`, inspect the generated schema's `$defs` keys. If any of `["FieldMeaning", "Intent", "ServiceDef", "Cardinality", "ActionDef", "GuardDef", "StateDef"]` appears in `$defs`, activate the ServiceDef-aware path.

**Closing algorithm for FieldMeaning:**
1. Locate `$defs["FieldMeaning"]` in the schema JSON.
2. Confirm it has `anyOf` with (at least) two branches.
3. Extract the first branch (must have `"type": "string", "enum": [...]`).
4. Replace the entire `FieldMeaning` `$defs` entry with just that branch (keeping `description` if present at the outer level).
5. Repeat for `Intent` → `["browse", "focus", "collect", "process", "summarize", "analyze", "track"]`.

**Known variant lists (from source files) [VERIFIED]:**
- `FieldMeaning`: `identifier`, `foreign_key`, `entity_name`, `email`, `phone`, `url`, `image_url`, `money`, `percentage`, `quantity`, `status`, `category`, `boolean`, `free_text`, `created_at`, `updated_at`, `date_time`, `sensitive` (18 variants)
- `Intent`: `browse`, `focus`, `collect`, `process`, `summarize`, `analyze`, `track` (7 variants)
- `Cardinality`: already a closed enum (`one_to_one`, `one_to_many`, `many_to_one`, `many_to_many`) — no Custom variant; schemars already emits a closed `enum`; no closing needed

**Important:** The closing happens BEFORE `$ref` inlining. The normalizer closes the `$defs` entries first, then inlines refs. This way, every usage of `#/$defs/FieldMeaning` throughout the schema resolves to the closed form.

### 5. Anthropic Tool-Use Request/Response Format [VERIFIED: official docs]

**Request — tools array:**
```json
{
  "model": "claude-sonnet-4-6",
  "max_tokens": 4096,
  "tools": [
    {
      "name": "get_weather",
      "description": "Get current weather for a location",
      "input_schema": {
        "type": "object",
        "properties": {
          "location": {"type": "string", "description": "City and state"}
        },
        "required": ["location"],
        "additionalProperties": false
      }
    }
  ],
  "messages": [{"role": "user", "content": "What's the weather in SF?"}]
}
```

**Response — tool_use block:**
```json
{
  "stop_reason": "tool_use",
  "content": [
    {"type": "text", "text": "I'll check that for you."},
    {
      "type": "tool_use",
      "id": "toolu_01A09q90qw90lq917835lq9",
      "name": "get_weather",
      "input": {"location": "San Francisco, CA"}
    }
  ]
}
```

**Tool result — sent back as user message:**
```json
{
  "role": "user",
  "content": [
    {
      "type": "tool_result",
      "tool_use_id": "toolu_01A09q90qw90lq917835lq9",
      "content": "Currently 72°F and sunny in San Francisco."
    }
  ]
}
```

**Loop termination:** `stop_reason == "end_turn"` means no more tool calls; `stop_reason == "tool_use"` means dispatch and loop. The `max_iterations` guard in `ToolRegistry::dispatch` catches infinite loops.

**Schema constraints for tools:** Same as structured-output JSON Schema constraints (same reject-list). The `input_schema` field IS the normalized tool schema.

### 6. OpenAI Function-Calling Format [VERIFIED: existing openai.rs + OpenAI docs]

OpenAI uses `response_format.type = "json_schema"` for structured output (already implemented in Phase 165). For tool calling:

**Request:**
```json
{
  "model": "gpt-4o",
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get weather",
        "parameters": {
          "type": "object",
          "properties": {"location": {"type": "string"}},
          "required": ["location"],
          "additionalProperties": false
        },
        "strict": true
      }
    }
  ],
  "messages": [...],
  "tool_choice": "auto"
}
```

**Response tool call:**
```json
{
  "choices": [{
    "finish_reason": "tool_calls",
    "message": {
      "role": "assistant",
      "tool_calls": [
        {
          "id": "call_abc123",
          "type": "function",
          "function": {"name": "get_weather", "arguments": "{\"location\":\"SF\"}"}
        }
      ]
    }
  }]
}
```

**Tool result:**
```json
{
  "role": "tool",
  "content": "72°F and sunny",
  "tool_call_id": "call_abc123"
}
```

**Key difference from Anthropic:** OpenAI uses `"role": "tool"` with `tool_call_id` (not `tool_use_id`); Anthropic uses `"role": "user"` with `type: "tool_result"`. The `ToolRegistry::dispatch` loop must handle both formats via provider-specific message construction.

**Ollama tool support:** Ollama supports tool calling when `stream: false` (streaming drops tool calls — confirmed deferred). Returns in OpenAI-compatible format. [ASSUMED based on Ollama API docs pattern — verify at implementation time]

### 7. CompletionRequest Extension for Tool Support (D-14) [ASSUMED design]

The existing `CompletionRequest` must carry tool definitions. Recommended extension:

```rust
pub struct CompletionRequest {
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub model_override: Option<String>,
    pub schema: Option<serde_json::Value>,  // existing — structured output schema
    // NEW:
    pub tools: Option<Vec<ToolRequest>>,    // tool definitions for tool-calling loop
    pub tool_choice: Option<ToolChoice>,    // auto / none / specific tool
}

pub struct ToolRequest {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,  // already normalized
}

pub enum ToolChoice {
    Auto,
    None,
}
```

The completion response must also be extended to carry either text OR tool-use blocks. A `CompletionResponse` enum is cleaner than returning a raw `String` from `complete()` when tools are involved:

```rust
pub enum CompletionResponse {
    Text(String),
    ToolUse(Vec<ToolUseBlock>),
}

pub struct ToolUseBlock {
    pub id: String,      // for tool_result reference
    pub name: String,
    pub input: serde_json::Value,
}
```

**Planner decides:** Whether to add `complete_with_tools` as a new `LlmClient` method or extend `complete` to return `CompletionResponse`. The key constraint (D-14) is single source of HTTP — no parallel implementation.

### 8. Async Tool Handler Ergonomics in Rust [VERIFIED: Context7 / Rust pattern]

The handler type from D-11:
```rust
Box<dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ToolError>> + Send + Sync>
```

`BoxFuture<'static, T>` is `Pin<Box<dyn Future<Output = T> + Send + 'static>>` from `futures::future`. Usage:

```rust
use futures::future::BoxFuture;

// Registration:
registry.register("my_tool", |input: serde_json::Value| -> BoxFuture<'static, Result<serde_json::Value, ToolError>> {
    Box::pin(async move {
        // handler body
        Ok(serde_json::json!({"result": "done"}))
    })
});
```

**Pitfall:** Handlers that capture `&references` won't satisfy `'static`. Handlers must own all their data (or use `Arc<T>` for shared state). Document this in the public API.

**Common helper pattern:**
```rust
pub fn make_handler<F, Fut>(f: F) -> Box<dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ToolError>> + Send + Sync>
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<serde_json::Value, ToolError>> + Send + 'static,
{
    Box::new(move |input| Box::pin(f(input)))
}
```

### 9. Error Variants to Add [VERIFIED: existing error.rs]

The existing `Error` enum must gain:

```rust
/// Schema normalization failed (e.g., invalid schemars output structure).
#[error("schema normalization error: {0}")]
SchemaError(String),

/// Tool dispatch exceeded max_iterations.
#[error("tool dispatch exceeded max_iterations ({0})")]
ToolIterationLimit(u32),

/// Tool not found in registry.
#[error("tool not found: {0}")]
ToolNotFound(String),
```

`ToolError { message: String }` is a SEPARATE type (not an `Error` variant) because it carries model-legible errors — it is returned to the LLM, not to the caller. The `dispatch` loop returns `Error::ToolIterationLimit` to the Rust caller; individual tool errors surface as `ToolError` JSON to the LLM.

### 10. jsonschema 0.46 API for SC#3 Test [VERIFIED: Context7 + Cargo.lock]

Version 0.46.5 is already in Cargo.lock. API:

```rust
use serde_json::json;

// Compile schema once:
let compiled = jsonschema::draft202012::new(&normalized_schema)?;

// Validate instance:
let instance = json!({"name": "order", "fields": [{"name": "id", "data_type": "integer", "meaning": "totally_bogus"}]});
let result = compiled.validate(&instance);
assert!(result.is_err(), "invalid FieldMeaning must fail validation");

// Valid instance passes:
let valid_instance = json!({"name": "order", "fields": [{"name": "id", "data_type": "integer", "meaning": "identifier"}]});
assert!(compiled.validate(&valid_instance).is_ok());
```

The `enum` keyword validation is confirmed to work in jsonschema 0.46 [VERIFIED: Context7 shows `EnumValidator` checks instance type then compares against allowed values].

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Schema validation for test | Custom validator | `jsonschema` 0.46 | Already in workspace; correct `enum` enforcement confirmed |
| Async future boxing | Manual `Pin<Box<dyn Future>>` | `futures::future::BoxFuture` | Already a dep; standard pattern |
| Schema generation | Custom schema builder | `schemars::schema_for!` + `JsonSchema` derive | All projection types already derive it; tested |
| Provider HTTP | New reqwest client | Existing `AnthropicClient`/`OpenAiClient` | D-14: reuse existing clients |

**Key insight:** The complex part of this phase (the Schema normalizer) is hand-rolled by design because there is no widely-used Rust crate for "normalize schemars output to Anthropic's requirements." The algorithm is well-understood (~100 lines) and must be custom to accommodate the ServiceDef-aware closing pass.

---

## Common Pitfalls

### Pitfall 1: Stripping `enum` when stripping `format`
**What goes wrong:** The normalizer's strip pass accidentally removes `enum` constraints when iterating keyword lists to strip.
**Why it happens:** `enum` is on the preserve list, but an overly broad strip implementation might iterate all non-`type`/`properties` keys.
**How to avoid:** The strip function must use an explicit allowlist (not a denylist). Only strip keys NOT in the preserve set: `[$schema, $id, title, examples, minimum, maximum, multipleOf, minLength, maxLength, pattern, format (numeric only)]`.
**Warning signs:** SC#3 test passes but SC#2 unit test fails; or `Cardinality` stops being an enum in the normalized schema.

### Pitfall 2: Closing `FieldMeaning` after inlining `$ref`
**What goes wrong:** The normalizer resolves `$ref` → inline BEFORE closing the projection enums, so the open `anyOf` gets inlined throughout the schema before being closed. The closing pass then has to walk the entire inlined schema to find all occurrences.
**Why it happens:** Natural code order puts ref-inlining first.
**How to avoid:** Always close projection enums in `$defs` FIRST, THEN inline refs. This way every inlined occurrence is already closed.
**Warning signs:** SC#3 test fails because FieldMeaning inline occurrences still accept any string.

### Pitfall 3: `serde_json::Value` mutation while iterating
**What goes wrong:** Attempting to modify schema object fields while iterating map keys causes borrow errors or incorrect behavior.
**How to avoid:** Use immutable recursive descent + rebuild pattern (construct new `Value` rather than mutating in-place). Alternatively, collect keys to modify first, then apply modifications.

### Pitfall 4: Tool handler `'static` lifetime
**What goes wrong:** Handler closure captures a reference (e.g., `&db_pool`) — doesn't satisfy `'static`. Compilation error.
**How to avoid:** Wrap shared state in `Arc<T>`. Document in API that all captures must be owned or `Arc`-wrapped.

### Pitfall 5: Anthropic `stop_reason` vs OpenAI `finish_reason`
**What goes wrong:** The dispatch loop checks `stop_reason` but OpenAI returns `finish_reason: "tool_calls"`, not `stop_reason: "tool_use"`.
**How to avoid:** The provider-specific `complete_with_tools` implementation in each client translates the provider's stop indicator to a common `CompletionResponse` variant before returning to the dispatch loop.

### Pitfall 6: `additional_properties: false` on `anyOf`/`oneOf` parent objects
**What goes wrong:** Adding `additionalProperties: false` to EVERY object including intermediate composition nodes breaks the schema (the composition object has no `properties` key, so all properties are rejected).
**How to avoid:** Only add `additionalProperties: false` to schemas that have `type: "object"` AND a `properties` key. Skip composition schemas (`anyOf`, `oneOf`, `allOf`) that don't define properties directly.

### Pitfall 7: `$defs` detection key case sensitivity
**What goes wrong:** schemars 1.x uses `$defs` (Draft 2020-12). Detection code looking for `definitions` (Draft 7) will miss ferro-projections types.
**How to avoid:** Normalizer handles both `$defs` and `definitions` (D-03 says "defensively handle legacy `#/definitions/` too") but detection for the ServiceDef-aware path only looks in `$defs`.

---

## Code Examples

### Schema Normalization Entry Point
```rust
// Source: designed for this phase; no existing example
pub fn for_structured_output(schema: serde_json::Value) -> serde_json::Value {
    let defs = extract_defs(&schema);  // pull $defs map
    let mut root = schema;

    // Step 1: Close projection enums in $defs FIRST
    if let Some(defs_mut) = root.get_mut("$defs").and_then(|d| d.as_object_mut()) {
        close_projection_enum(defs_mut, "FieldMeaning");
        close_projection_enum(defs_mut, "Intent");
    }

    // Step 2: Inline all $refs (with cycle guard)
    let mut visited = std::collections::HashSet::new();
    root = resolve_refs(root, &defs, &mut visited);

    // Step 3: Remove $defs (all refs now inlined)
    if let Some(obj) = root.as_object_mut() {
        obj.remove("$defs");
        obj.remove("definitions");
    }

    // Step 4: Strip rejected keywords + add additionalProperties: false
    normalize_object(&mut root);
    root
}
```

### Closing a Projection Enum
```rust
// Source: designed for this phase
fn close_projection_enum(defs: &mut serde_json::Map<String, serde_json::Value>, name: &str) {
    if let Some(entry) = defs.get_mut(name) {
        if let Some(any_of) = entry.get("anyOf") {
            // Extract first branch which has {"type": "string", "enum": [...known...]}
            if let Some(closed_branch) = any_of.as_array().and_then(|arr| arr.first()) {
                let desc = entry.get("description").cloned();
                let mut closed = closed_branch.clone();
                if let (Some(desc), Some(obj)) = (desc, closed.as_object_mut()) {
                    obj.insert("description".into(), desc);
                }
                *entry = closed;
            }
        }
    }
}
```

### SC#3 Structural Guarantee Test
```rust
// Source: designed for this phase (dev-test in ferro-ai/tests/ or #[cfg(test)])
#[test]
fn servicedef_aware_schema_rejects_invalid_field_meaning() {
    use ferro_projections::ServiceDef;
    let raw = schemars::schema_for!(ServiceDef).to_value();
    let normalized = ferro_ai::schema::for_structured_output(raw);

    let invalid = serde_json::json!({
        "name": "order",
        "fields": [{
            "name": "total",
            "data_type": "float",
            "meaning": "totally_bogus"  // invalid FieldMeaning
        }]
    });
    let validator = jsonschema::draft202012::new(&normalized).unwrap();
    assert!(validator.validate(&invalid).is_err(), "invalid FieldMeaning must fail");

    let valid = serde_json::json!({
        "name": "order",
        "fields": [{
            "name": "total",
            "data_type": "float",
            "meaning": "money"  // valid FieldMeaning
        }]
    });
    assert!(validator.validate(&valid).is_ok(), "valid FieldMeaning must pass");
}
```

### complete::<T>() Implementation
```rust
// Source: designed for this phase
pub async fn complete<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let raw_schema = schemars::schema_for!(T).to_value();
    let normalized = schema::for_structured_output(raw_schema);

    let request = CompletionRequest {
        system: None,
        messages: vec![Message { role: Role::User, content: prompt.to_string() }],
        max_tokens: 4096,
        model_override: None,
        schema: Some(normalized),
        tools: None,
        tool_choice: None,
    };

    let text = client.complete(request).await?;
    serde_json::from_str::<T>(&text).map_err(|e| Error::Deserialization(e.to_string()))
}
```

### ToolRegistry Dispatch Loop (sketch)
```rust
// Source: designed for this phase
pub async fn dispatch(
    &self,
    mut messages: Vec<Message>,
    client: &dyn LlmClient,
) -> Result<Vec<Message>, Error> {
    for iteration in 0..=self.max_iterations {
        if iteration == self.max_iterations {
            tracing::error!("tool dispatch hit max_iterations={}", self.max_iterations);
            return Err(Error::ToolIterationLimit(self.max_iterations));
        }
        if iteration == 5 {
            tracing::warn!("tool dispatch at iteration 5 of {}", self.max_iterations);
        }

        let request = self.build_request(messages.clone());
        let response = client.complete_with_tools(request).await?;

        match response {
            CompletionResponse::Text(text) => {
                messages.push(Message { role: Role::Assistant, content: text });
                return Ok(messages);
            }
            CompletionResponse::ToolUse(blocks) => {
                messages.push(Message { /* assistant with tool_use blocks */ });
                for block in blocks {
                    let result = self.dispatch_one(&block).await;
                    messages.push(result_to_message(block.id, result));
                }
            }
        }
    }
    unreachable!()
}
```

---

## Runtime State Inventory

Step 2.5: SKIPPED — this is a greenfield feature addition, not a rename/refactor/migration phase.

---

## Environment Availability

Step 2.6: SKIPPED — this phase adds no external service dependencies. `jsonschema` is already in Cargo.lock. `schemars` is already used by `ferro-projections`. No new CLI tools, databases, or external services needed.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | schemars 1.x emits `anyOf` with first branch = closed `enum` and second = open `type: "string"` for untagged enum variants | Research Priority 2 | The closing algorithm targets the wrong schema shape; must run `schema_for!(FieldMeaning).to_value()` in Wave 0 test to confirm |
| A2 | The closing algorithm can use the first `anyOf` branch directly since schemars always puts known variants first | ServiceDef-aware Path | If schemars puts the open string branch first, the extraction logic must be updated |
| A3 | Ollama supports tool calling in non-streaming mode with OpenAI-compatible format | Tool Calling research | OllamaClient tool support may need to return `Error::Unsupported` if not confirmed |
| A4 | `jsonschema::draft202012::new(&schema)` correctly enforces the `enum` keyword for string values | SC#3 test design | If the validator doesn't enforce `enum` on strings, SC#3 test passes vacuously; confirmed from Context7 `EnumValidator` source |

**Note:** A1 and A2 are the highest-risk assumptions. Both are resolved by a Wave 0 unit test that prints `schema_for!(FieldMeaning).to_value()` and asserts its structure.

---

## Open Questions

1. **`complete_with_tools` vs extended `complete` signature (D-14)**
   - What we know: `LlmClient::complete` returns `String`; tools need structured `tool_use` response
   - What's unclear: Add a new `async fn complete_with_tools(...) -> Result<CompletionResponse, Error>` method to `LlmClient`, or extend `complete` to return an enum
   - Recommendation: Add `complete_with_tools` — cleaner separation; missing-capability providers return `Error::Unsupported`; existing `complete` callers unaffected

2. **`pattern` keyword handling**
   - What we know: Anthropic says simple `pattern` works; backreferences/lookahead don't; schemars may emit patterns on some types
   - What's unclear: Does schemars 1.2.0 ever emit `pattern` for ferro-projections types?
   - Recommendation: Strip `pattern` unconditionally in the normalizer; add it back to description; document as known limitation

3. **Ollama tool support in Phase 166**
   - What we know: Ollama supports tool calling with `stream: false`; streaming tool calls are deferred
   - What's unclear: Exact Ollama tool request format and whether it's OpenAI-compatible
   - Recommendation: Implement `OllamaClient::complete_with_tools` returning `Error::Unsupported` initially; upgrade in a later phase when needed (not a blocker for SC#4-6)

---

## Validation Architecture

`workflow.nyquist_validation` is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) + tokio::test for async |
| Config file | none — standard Rust test runner |
| Quick run command | `cargo test -p ferro-ai` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Location |
|--------|----------|-----------|-------------------|----------|
| AISDK-02 SC#1 | `complete::<T>()` returns `Result<T, Error>`; never exposes schemars/serde_json to caller | unit | `cargo test -p ferro-ai complete_returns_typed_result` | ferro-ai/src/complete.rs #[cfg(test)] |
| AISDK-02 SC#2 | `for_structured_output` resolves `$ref`/`$defs`, adds `additionalProperties:false`, strips rejected Anthropic keywords | unit | `cargo test -p ferro-ai schema_normalizer_strips_rejected_keywords` | ferro-ai/src/schema/mod.rs #[cfg(test)] |
| AISDK-02 SC#3 | ServiceDef-aware path: invalid `FieldMeaning`/`Intent` fails validation against normalized schema | unit | `cargo test -p ferro-ai servicedef_schema_rejects_invalid_field_meaning` | ferro-ai/tests/projection_schema.rs or #[cfg(test)] |
| AISDK-03 SC#4 | `ToolDef` carries `name`, `description`, `parameters_schema`, async handler closure | unit | `cargo test -p ferro-ai tool_def_construction` | ferro-ai/src/tools/mod.rs #[cfg(test)] |
| AISDK-03 SC#5 | `ToolRegistry::dispatch` enforces `max_iterations`; warns at 5; errors at cap | unit (mock client) | `cargo test -p ferro-ai tool_registry_enforces_max_iterations` | ferro-ai/src/tools/mod.rs #[cfg(test)] |
| AISDK-03 SC#6 | Tool errors carry `ToolError { message }` not raw panics | unit | `cargo test -p ferro-ai tool_error_is_model_legible` | ferro-ai/src/tools/mod.rs #[cfg(test)] |
| Regression SC#7 | Existing `Classifier<T>` tests remain green | unit | `cargo test -p ferro-ai classifier` | existing tests |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-ai`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-ai/src/schema/mod.rs` — schema normalizer module (new file)
- [ ] `ferro-ai/src/complete.rs` — `complete::<T>()` + `complete_into::<T>()` (new file)
- [ ] `ferro-ai/src/tools/mod.rs` — `ToolDef`, `ToolRegistry`, `ToolError` (new file)
- [ ] `ferro-ai/tests/projection_schema.rs` — SC#3 structural guarantee test (or inline `#[cfg(test)]`)
- [ ] Structural probe test: verify `schema_for!(FieldMeaning).to_value()` has `anyOf` with expected shape (resolves A1/A2)

---

## Security Domain

Phase 166 is an internal Rust library (no HTTP server surface). ASVS categories do not apply to this phase. Security considerations:

- **Tool handler inputs** are `serde_json::Value` from LLM output. Handlers that perform DB/filesystem operations must validate inputs independently — `ToolRegistry` passes raw LLM JSON, it does not validate semantics.
- **API key exposure**: `ToolError { message }` must NEVER include the FERRO_AI_API_KEY. Existing `Error::Provider` already has this constraint documented; tool errors derive from handler results, not from provider responses directly.
- **No new network endpoints** introduced by this phase.

---

## Sources

### Primary (HIGH confidence)

- `ferro-ai/src/client/mod.rs` — `CompletionRequest`, `LlmClient`, `Message`/`Role`/`TokenStream` [VERIFIED]
- `ferro-ai/src/client/anthropic.rs` — `build_body` with `output_config.format.json_schema` passthrough [VERIFIED]
- `ferro-ai/src/client/openai.rs` — `build_body` with `response_format.json_schema` + `strict: true` [VERIFIED]
- `ferro-ai/src/error.rs` — existing `Error` enum variants [VERIFIED]
- `ferro-projections/src/field.rs` — `FieldMeaning` with 18 known variants + `#[serde(untagged)] Custom(String)` [VERIFIED]
- `ferro-projections/src/intent.rs` — `Intent` with 7 known variants + `#[serde(untagged)] Custom(String)` [VERIFIED]
- `ferro-projections/src/relationship.rs` — `Cardinality` is already a closed enum [VERIFIED]
- `ferro-projections/Cargo.toml` — no ferro-ai dep → no cycle [VERIFIED]
- `ferro-ai/Cargo.toml` — current deps: reqwest, serde_json, futures, async-trait, tracing [VERIFIED]
- `Cargo.lock` — schemars 1.2.0, jsonschema 0.46.5 [VERIFIED]
- `.github/workflows/publish.yml` — ferro-ai and ferro-projections both in WAVE1B [VERIFIED]
- Anthropic structured-output docs — reject-list [VERIFIED: platform.claude.com/docs/en/docs/build-with-claude/structured-outputs]
- Anthropic tool-use docs — request/response format [VERIFIED: platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools]
- Context7 `/gresau/schemars` — `schema_for!`, Draft 2020-12 output, untagged enum anyOf shape [VERIFIED]
- Context7 `/websites/rs_jsonschema_0_40_0` — `draft202012::new`, `EnumValidator` [VERIFIED]

### Secondary (MEDIUM confidence)

- schemars `to_value()` output for FieldMeaning untagged enum — inferred from docs + existing ferro-projections test structure; needs Wave 0 confirmation

### Tertiary (LOW confidence)

- Ollama tool-calling format (OpenAI-compatible) — assumed from documentation patterns; verify at OllamaClient implementation time

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps verified from Cargo.lock/Cargo.toml
- Schema normalizer algorithm: HIGH — Anthropic constraints documented; schemars output confirmed in docs
- ServiceDef-aware closing: HIGH for design; MEDIUM for exact anyOf shape (Wave 0 probe resolves)
- Tool calling: HIGH for Anthropic format (official docs); MEDIUM for OpenAI (inferred from existing code); LOW for Ollama (assumed)
- Error handling: HIGH — existing error.rs provides exact extension points
- Test architecture: HIGH — jsonschema enum validation confirmed

**Research date:** 2026-06-08
**Valid until:** 2026-07-08 (Anthropic API formats can change; check release notes if >30 days)
