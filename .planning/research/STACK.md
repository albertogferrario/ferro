# Stack Research: v15.0 Agent-Operable App (Consumer MCP)

**Domain:** Projection-derived consumer MCP endpoint with write/act capabilities and inbound NL intent loop
**Researched:** 2026-06-13
**Confidence:** HIGH (all findings sourced from in-codebase verification + Context7 rmcp docs)

---

## What Already Exists — Do Not Re-Build

Everything below is already shipping in the ferro workspace. v15.0 extends these; it does not replace them.

| Component | What It Provides | Crate |
|-----------|-----------------|-------|
| `ferro-mcp-server` (0.2.x) | `McpRenderer` (Renderer impl), `handle_initialize/tools_list/tools_call`, `dispatch` (read-only, tenant-scoped, SQL), JSON-RPC layer, `BearerOutcome` stub | `ferro-mcp-server` |
| `ferro-mcp-oauth` | OAuth 2.1 authorization server (DCR, PKCE, authorize, consent, token, device), `validate_bearer` → `BearerCheck` (JWT sig + exp + audience + tenant), `McpTokenClaims` | `ferro-mcp-oauth` |
| `rmcp` 0.12 | `Tool`, `ToolAnnotations`, `CallToolResult` (with `.structured()` constructor), JSON Schema `Arc<JsonObject>` input; `ToolAnnotations::{read_only, destructive, idempotent}` hints | `rmcp = "0.12"` |
| `ferro-projections` | `ServiceDef` (fields, actions, guards, state_machine, mcp_exposed, tenant_column, mcp_ability), `ActionDef`/`InputDef`/`GuardDef`, `BaseContext` (evaluated_guards, verbosity), `Renderer` trait | `ferro-projections` |
| `ferro-ai` | `Classifier<T>` (provider-agnostic structured JSON output), `AiConfig::from_env()`, `AnthropicProvider`/`OpenAiClient`/`OllamaClient`, `ToolRegistry`/`ToolDef` | `ferro-ai` |
| `TenantScoped` trait | Typed tenant-scoped lookup, cross-tenant reads structurally impossible | `framework` |

The v12.6 consumer-MCP OAuth endpoint provides the transport and auth shell. The v14.0 projection surface (`evaluated_guards`, `BaseContext`, `FieldDef.render_hint`) provides the guard-filtering substrate. v15.0 adds write and intent-loop capabilities on top.

---

## Capability Analysis: What v15.0 Must Add

### (a) rmcp 0.12 API Surface for Dynamic Tool Registration

**Confidence: HIGH** — verified from Context7 docs + direct codebase reads of `ferro-mcp-server/src/renderer.rs` and `jsonrpc.rs`.

rmcp 0.12 is already used correctly in `ferro-mcp-server`. The projection→tools renderer (`McpRenderer`) already builds `Tool` objects at runtime from `ServiceDef` — this is dynamic by construction. The API surface required for v15.0's action tools is the same surface already in use.

**Tool construction (verified API):**
```rust
// Already in use (renderer.rs) — read tools
Tool::new(name, description, Arc::new(schema_map))
    .annotate(ToolAnnotations::new().read_only(true))

// For write/action tools: drop read_only, set destructive/idempotent hints
Tool::new(name, description, Arc::new(schema_map))
    .annotate(ToolAnnotations::new()
        .destructive(true)   // state-transition actions
        .read_only(false))   // or idempotent(true) for update-in-place
```

`Tool::new()` signature (verified via Context7):
```
Tool::new<N, D, S>(name: N, description: D, input_schema: S) -> Self
where
    N: Into<Cow<'static, str>>,
    D: Into<Cow<'static, str>>,
    S: Into<Arc<JsonObject>>,
```

No compile-time macro. Tools are registered at runtime by returning them from `handle_tools_list`. Dynamic registration is the current model; rmcp's `ToolRoute::new_dyn()` exists but is unused because `ferro-mcp-server` bypasses rmcp's built-in router entirely with a custom JSON-RPC dispatch.

**`tools/list` for action tools:**
`render_exposed_tools(services, &McpContext)` already returns `Vec<Tool>` (one per exposed `ServiceDef`, read tool). For v15.0, this function must be extended (or a sibling `render_action_tools()` added) to emit one additional `Tool` per guard-passing `ActionDef` on each exposed `ServiceDef`. Both lists are concatenated into the `tools` array in the `tools/list` response.

**`tools/call` result shape — spec-compliant typed content:**
The original bug (`content[]` without `type` field, which MCP clients Zod-reject) was fixed by Phase 197 by switching to `CallToolResult::structured(payload)`. This is verified by the interop test `tools_call_result_parses_as_valid_mcp_content` in `jsonrpc.rs`, which deserializes the emitted response through `CallToolResult`'s own custom `Deserialize`. Action tools must use the same pattern.

`CallToolResult` structure (verified via Context7):
```rust
pub struct CallToolResult {
    pub content: Vec<Content>,           // must contain typed blocks
    pub structured_content: Option<Value>,
    pub is_error: Option<bool>,
    pub meta: Option<Meta>,
}
```

`CallToolResult::structured(payload)` produces: `is_error: Some(false)`, one `content` block with `type: "text"`, `structured_content` = the payload JSON. This is the shape MCP clients expect.

For action tool results: use `CallToolResult::structured(json!({"success": true, "message": "..."}))` on success, or `json!({"result": ...})` with entity data. On business-logic rejection (guard failed at execution time, wrong state, validation error): use `CallToolResult { content: vec![text_block], structured_content: Some(json!({"error": "..."})), is_error: Some(true), meta: None }` rather than a JSON-RPC `-32602` error, because the call succeeded but the action was refused — the distinction matters for agent error handling.

**Do NOT upgrade rmcp:** No feature in v15.0 requires rmcp ≥1.5. The existing `Tool`, `ToolAnnotations`, `CallToolResult`, and `Arc<JsonObject>` input schema API is sufficient for both read and write tools. An upgrade would be a breaking change across three crates (`ferro-mcp`, `ferro-mcp-server`, `ferro-api-mcp`) and is not justified.

---

### (b) API-Key Auth for the MCP Endpoint

**Confidence: HIGH** — verified from `ferro-mcp-oauth/src/validate.rs` and `ferro-mcp-server/src/auth.rs`.

`ferro-mcp-oauth` ships a complete bearer-token validation stack: `validate_bearer(header, config, expected_tenant)` → `BearerCheck` with five cases (Unauthenticated, Invalid, Forbidden, Authenticated, Invalid). The current v12.6 flow issues OAuth JWTs with `tenant_id` claims. `BearerOutcome` in `ferro-mcp-server` is the application-seam stub.

**What v15.0 adds:** API-key bearer tokens are structurally identical to OAuth JWTs from the MCP endpoint's perspective — both arrive as `Authorization: Bearer <token>`. The difference is validation:

- **OAuth path (existing):** JWT decoded, `exp` + signature + audience + tenant checked via `validate_bearer`.
- **API-key path (new):** SHA-256 hash of the raw key looked up in a `api_keys` DB table that stores `{hashed_key, tenant_id, abilities[], active}`. No JWT. The DB record is the source of truth for expiry (soft: `active = false`) and scope.

The right addition is a second validation branch in `ferro-mcp-oauth/src/validate.rs`, not a second endpoint. The MCP HTTP handler reads the `Authorization: Bearer <token>` header and tries:
1. If the token is a JWT (starts with `eyJ`): `validate_bearer` via the existing OAuth path.
2. Otherwise: hash the token (SHA-256, then hex), lookup in `api_keys`, check `active = true`, extract `tenant_id` + ability scope.

Both paths produce the same `BearerCheck::Authenticated(principal)` outcome. The application handler is unchanged.

**What NOT to add:** A separate MCP endpoint for API-key auth. One endpoint, two validation branches, same downstream behavior.

**Crate placement:** The API-key validator belongs in `ferro-mcp-oauth` (the auth crate for the consumer MCP), not in `ferro-mcp-server`. However, the `api_keys` schema should reuse whatever the v8.1 `ferro make:api-key` command generates (verified: v8.1 shipped `ferro make:api-key` CLI). Confirm whether `framework/src/` already has an `api_keys` model before designing a new schema.

**New dependencies:** None. All required primitives are already in `ferro-mcp-oauth`:
- `sha2 = "0.10"` — already present
- `subtle = "2.5"` — already present (constant-time compare)
- `sea-orm` — already present for the DB lookup

---

### (c) ferro-ai for the NL→Intent Classification Loop

**Confidence: HIGH** — verified from `ferro-ai/src/classifier/mod.rs`, `ferro-ai/src/config.rs`, `ferro-ai/src/lib.rs`.

**What exists:** `ferro-ai` provides a provider-agnostic `Classifier<T>` that calls an LLM with a system prompt + user message + JSON schema and returns a typed Rust struct. The default provider is Anthropic. Default model (Anthropic): `claude-sonnet-4-6` (verified in `config.rs` test line `assert_eq!(client.default_model(), "claude-sonnet-4-6")`). The classifier is provider-agnostic: `FERRO_AI_PROVIDER` env var selects Anthropic/OpenAI/Groq/Ollama.

**What the inbound intent loop needs:**

The NL→intent classification maps a tenant's natural-language message to one of:
1. A named `ActionDef` on an exposed `ServiceDef` (message implies mutation / state transition).
2. A read query on an exposed `ServiceDef` (message implies browsing/looking up data).
3. Ambiguous / unrecognized (needs clarification).

The classification output type is defined in `ferro-mcp-server` (not in `ferro-ai`, which stays generic):

```rust
#[derive(serde::Deserialize)]
struct IntentClassification {
    service: String,         // name of the exposed ServiceDef
    action: Option<String>,  // None = read query; Some(name) = ActionDef name
    confidence: f64,         // gates LowConfidence rejection in Classifier
    arguments: serde_json::Value, // filter params (read) or action inputs (write)
}
```

The system prompt embeds the exposed services + their actions + their descriptions. Keep prompts compact: list service names, display names, descriptions, and action names only — not full `ServiceDef` JSON — to stay within token budgets. The schema passed to `Classifier<IntentClassification>::classify()` is derived from the struct above.

**No changes to ferro-ai itself** are needed for v15.0. The existing `Classifier<T>` API is sufficient. The loop logic (message → classify → dispatch → result) lives in `ferro-mcp-server/src/intent.rs` (new file).

**New dependency in ferro-mcp-server:**
```toml
ferro-ai = { path = "../ferro-ai", version = "0.2" }
```
Currently `ferro-mcp-server` has no `ferro-ai` dependency. This is the only new Cargo.toml change required.

**No circular dependency:** `ferro-ai` depends on `ferro-projections` (verified in `ferro-ai/Cargo.toml`). `ferro-mcp-server` depends on `ferro-projections`. Adding `ferro-ai` to `ferro-mcp-server` creates `ferro-mcp-server → ferro-ai → ferro-projections`. No cycle.

**Classification latency:** Each NL→intent call makes an LLM API round trip (Anthropic: ~500ms–2s). The MCP transport handles this fine (it is async). For low-latency deployments, `FERRO_AI_PROVIDER=ollama` with a local model is available with zero code changes.

---

### (d) Crate Placement: Projection→MCP-Tools Renderer

**Confidence: HIGH** — verified by CLAUDE.md v11.5 renderer-location rule + `ferro-mcp-server/src/renderer.rs` which already implements this pattern.

The rule from v11.5: `ferro-projections` owns the `Renderer` trait only. Concrete renderers for specific output formats live in their output crate. `McpRenderer` is already in `ferro-mcp-server`, which is the MCP output crate. This is correct and must not change.

**v15.0 does not need a new crate.** `ferro-mcp-server` is extended, not replaced:

- The existing `McpRenderer` renders `ServiceDef → Tool` as a read (`list_<name>`) tool with `readOnlyHint = true`.
- Write/action tools are rendered as additional `Tool` objects: one per guard-passing `ActionDef` on an exposed `ServiceDef`. These are produced in the same `render_exposed_tools` call (or a new `render_action_tools` sibling).
- `schema.rs` gains `build_action_input_schema(action: &ActionDef) → Value` which maps `ActionDef.inputs: Vec<InputDef>` to JSON Schema properties, mirroring `build_input_schema` for reads.
- `dispatch.rs` gains `execute_action(service, action_name, inputs, db, tenant_id)` for write dispatch, mirroring the existing read `dispatch()`.

**Why NOT a new crate:** Creating a second output crate for write tools (e.g., `ferro-mcp-actions`) would mean two crates rendering to the same output format (MCP `Tool` / `CallToolResult`). This is the "duplicate control surface" anti-pattern from `feedback_no_duplicate_control_surface.md`. All projection→MCP-tool rendering lives in `ferro-mcp-server`.

---

## Stack Summary: Additions for v15.0

| Addition | Type | Lives In | Why |
|----------|------|----------|-----|
| `build_action_input_schema(action)` | New function | `ferro-mcp-server/src/schema.rs` | Maps `ActionDef.inputs` to JSON Schema for tool input |
| Extended `render_exposed_tools` (or sibling) | New/extended function | `ferro-mcp-server/src/renderer.rs` | Emits both read tools and guard-filtered action tools |
| `execute_action(...)` | New function | `ferro-mcp-server/src/dispatch.rs` | Write dispatch: INSERT/UPDATE/state-transition via raw SQL or SeaORM |
| `McpContext` extended with tenant + guards | Struct change | `ferro-mcp-server/src/renderer.rs` | Phase 197 left `McpContext` empty; v15.0 embeds `BaseContext` (evaluated_guards) |
| Guard-filter in action tool listing | Logic | `ferro-mcp-server/src/renderer.rs` | `evaluated_guards.get(g).copied().unwrap_or(true)` — same rule as `TextRenderer` |
| API-key validation branch | New function | `ferro-mcp-oauth/src/validate.rs` | Parallel to `validate_bearer` for non-JWT tokens |
| `IntentClassification` struct + NL loop | New file | `ferro-mcp-server/src/intent.rs` | Coordinator: message → Classifier → dispatch |
| `ferro-ai` dependency | Cargo dep | `ferro-mcp-server/Cargo.toml` | Intent loop needs the classifier |

---

## Recommended Stack

### Core Technologies (unchanged)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `rmcp` | 0.12 (pin; do NOT upgrade) | MCP protocol types (`Tool`, `CallToolResult`, `ToolAnnotations`) | Already integrated; upgrade = breaking change across 3 crates with no required feature |
| `ferro-projections` | workspace | `ServiceDef`, `ActionDef`, `GuardDef`, `BaseContext`, `Renderer` trait | The source of truth for all tool definitions; `evaluated_guards` is the guard-filter substrate |
| `ferro-ai` | workspace (new dep in `ferro-mcp-server`) | `Classifier<T>` for NL→intent classification | Provider-agnostic; Anthropic `claude-sonnet-4-6` default; already in the workspace |
| `ferro-mcp-oauth` | workspace | Bearer token validation for both OAuth JWT and API-key paths | Auth shell from v12.6; API-key branch is an additive extension |
| `sea-orm` | 1.0 | Write dispatch (INSERT, UPDATE, state transition) for action tools | Already in `ferro-mcp-server`; action execution needs DB writes |
| `schemars` | 1 | JSON Schema generation for action tool `inputSchema` | Already in `ferro-mcp-server`; action schema mirrors read schema pattern |
| `serde_json` | 1.0 | JSON payloads for tool inputs/outputs | Already everywhere |

### Supporting Libraries (no version changes)

| Library | Current location | Notes for v15.0 |
|---------|-----------------|----------------|
| `sha2 = "0.10"` | `ferro-mcp-oauth` | API-key hashing; already present |
| `subtle = "2.5"` | `ferro-mcp-oauth` | Constant-time key comparison; already present |
| `jsonwebtoken = "9"` | `ferro-mcp-oauth` | JWT decode for OAuth token path; already present |
| `tokio = "1"` | everywhere | Async dispatch, LLM calls; already everywhere |
| `thiserror = "1"` | `ferro-mcp-server` | Error types for new dispatch paths; already present |
| `tracing = "0.1"` | `ferro-mcp-server` | Structured logging for dispatch/auth events; already present |

### What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| rmcp upgrade to ≥1.5 | Breaking change across `ferro-mcp`, `ferro-mcp-server`, `ferro-api-mcp`; no feature in v15.0 requires it | Stay on 0.12 |
| New output crate for write tools | "Duplicate control surface" — one output format, one output crate | Extend `ferro-mcp-server` |
| Separate MCP endpoint for API keys | Two endpoints for one protocol surface increases client complexity | One endpoint, two validation branches in `ferro-mcp-oauth` |
| Full OAuth scope redesign | v12.6 already ships OAuth scopes via `McpTokenClaims`; API keys reuse the same `mcp_ability` model from `ServiceDef` | Extend `mcp_ability` semantics to cover action scopes |
| Hard-coding Anthropic in the intent loop | Breaks the provider-agnostic contract of `ferro-ai` | Use `AiConfig::from_env()` → `Classifier<IntentClassification>` |

---

## Integration Points with Existing Surfaces

### ferro-projections → ferro-mcp-server (read tools, already working)

`ServiceDef.mcp_exposed = true` → `render_exposed_tools()` → one `Tool` per service named `list_<name>`, `readOnlyHint = true`, input schema from `is_filter_field` allowlist.

### ferro-projections → ferro-mcp-server (action tools, new for v15.0)

`ServiceDef.actions: Vec<ActionDef>`, filtered by `McpContext.base.evaluated_guards` → one `Tool` per guard-passing `ActionDef`. Input schema from `ActionDef.inputs: Vec<InputDef>` via `build_action_input_schema`. Tool name: `<service_name>_<action_name>` (snake_case, avoids collisions with read tools). `ToolAnnotations` set to `destructive(true)` for state-transition actions, `idempotent(true)` for idempotent updates.

Guard-filter logic (consistent with `TextRenderer`):
```rust
let guard_passes = |g: &str| {
    ctx.base.evaluated_guards
        .get(g)
        .copied()
        .unwrap_or(true) // absent = render (same rule as TextRenderer)
};
let visible_actions: Vec<&ActionDef> = service.actions.iter()
    .filter(|a| a.preconditions.iter().all(|g| guard_passes(g)))
    .collect();
```

### ferro-mcp-oauth → application handler seam

OAuth path: `validate_bearer(header, &oauth_config, Some(tenant_id))` → `BearerCheck::Authenticated(principal)`.
API-key path (new): `validate_api_key(header, &db, Some(tenant_id))` → same `BearerCheck::Authenticated(principal)`.

Both paths check `ServiceDef.mcp_ability` against the principal's ability scope (from `McpTokenClaims.abilities` or `api_keys.abilities`).

### ferro-ai → ferro-mcp-server (intent loop, new for v15.0)

```rust
// In ferro-mcp-server/src/intent.rs (new file)
use ferro_ai::{AiConfig, Classifier, ClassifierConfig};

pub async fn classify_message(
    message: &str,
    services: &[ServiceDef],   // exposed + guard-filtered per tenant
) -> Result<IntentClassification, ferro_ai::Error> {
    let client = Arc::new(AiConfig::from_env()?);
    let classifier = Classifier::<IntentClassification>::new(
        client,
        ClassifierConfig::default(), // model from FERRO_AI_MODEL, provider from FERRO_AI_PROVIDER
    );
    let schema = build_intent_schema(services); // names + descriptions only, not full ServiceDef JSON
    classifier.classify(INTENT_SYSTEM_PROMPT, message, &schema).await
        .map(|r| r.value)
}
```

---

## Cargo.toml Changes Required

**`ferro-mcp-server/Cargo.toml`** — add one dependency:
```toml
ferro-ai = { path = "../ferro-ai", version = "0.2" }
```

No other crate requires new external dependencies. All required libraries (`sha2`, `subtle`, `sea-orm`, `schemars`, `serde_json`, `rmcp`, `jsonwebtoken`) are already present in the relevant crates.

---

## Version Compatibility

| Package | Constraint | Note |
|---------|-----------|------|
| `rmcp` | pin at `"0.12"`, do NOT upgrade | ≥1.5 is a breaking change across 3 crates; no v15.0 feature requires it |
| `ferro-ai` | workspace version | Provider-agnostic; `FERRO_AI_PROVIDER` selects Anthropic (default `claude-sonnet-4-6`) / OpenAI / Groq / Ollama |
| `sea-orm` | 1.0 | Write dispatch needs `insert` + `update` operations; 1.0 API stable |
| `schemars` | 1 | `build_action_input_schema` follows same pattern as `build_input_schema`; already in `ferro-mcp-server` |

---

## Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| rmcp 0.12 API surface | HIGH | Verified from direct codebase read (renderer.rs, jsonrpc.rs) + Context7 docs confirming `Tool::new`, `ToolAnnotations`, `CallToolResult::structured` |
| API-key auth pattern | HIGH | `ferro-mcp-oauth/src/validate.rs` read directly; extension point is clear; all required deps already present |
| ferro-ai classifier integration | HIGH | `ferro-ai/src/classifier/mod.rs` and `config.rs` read directly; `Classifier<T>` API and provider model confirmed |
| Crate placement decision | HIGH | v11.5 rule + `ferro-mcp-server/src/renderer.rs` is already the MCP output crate with `McpRenderer` |
| Guard-filter in action tools | HIGH | v14.0 `BaseContext.evaluated_guards` + `TextRenderer` rule documented in ferro-text; `unwrap_or(true)` pattern confirmed |
| Write dispatch SQL | MEDIUM | Read dispatch in `dispatch.rs` is a verified pattern; write (INSERT/UPDATE) follows the same `Statement::from_sql_and_values` approach but is not yet implemented |

---

## Sources

- `ferro-mcp-server/src/renderer.rs` — existing `McpRenderer`, `Tool::new`, `ToolAnnotations` usage
- `ferro-mcp-server/src/jsonrpc.rs` — `CallToolResult::structured()` usage, D-04 interop test verifying `type: "text"` content block
- `ferro-mcp-server/src/dispatch.rs` — read dispatch pattern, tenant-scoping, filter allowlist
- `ferro-mcp-server/src/schema.rs` — `build_input_schema`, `is_filter_field`
- `ferro-mcp-server/Cargo.toml` — confirms `rmcp = "0.12"`, `ferro-ai` absence, existing deps
- `ferro-mcp-oauth/src/validate.rs` — `validate_bearer`, `BearerCheck` enum, JWT + tenant validation logic
- `ferro-mcp-oauth/Cargo.toml` — confirms `sha2`, `subtle`, `jsonwebtoken` already present
- `ferro-projections/src/service.rs` — `ServiceDef` (mcp_exposed, tenant_column, mcp_ability, actions, guards)
- `ferro-projections/src/action.rs` — `ActionDef`, `InputDef`, `GuardDef` structures
- `ferro-ai/src/classifier/mod.rs` — `Classifier<T>`, `ClassifierConfig`, retry/confidence behavior
- `ferro-ai/src/config.rs` — `AiConfig::from_env()`, default model `claude-sonnet-4-6`, provider selection
- `ferro-ai/Cargo.toml` — confirms `ferro-projections` dependency (no circular dep when added to `ferro-mcp-server`)
- Context7 `/websites/rs_rmcp` — `Tool::new` signature, `ToolAnnotations` fields, `CallToolResult` struct, `ToolRoute::new_dyn` for dynamic registration — HIGH confidence

---
*Stack research for: ferro v15.0 Agent-Operable App (Consumer MCP)*
*Researched: 2026-06-13*
