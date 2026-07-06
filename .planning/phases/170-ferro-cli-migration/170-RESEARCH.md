# Phase 170: ferro-cli Migration - Research

**Researched:** 2026-06-08
**Domain:** Rust CLI async→sync bridge, ferro-ai SDK integration
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Bridge async `LlmClient::complete` with `tokio::runtime::Runtime::new().block_on(...)` inside `generate_with_ai`. Do not make `run()` or `main()` async.
- **D-02:** Route both passes through `AiConfig::from_env()` → `Box<dyn LlmClient>` → `client.complete(CompletionRequest { .. })`. Do NOT use `ferro_ai::complete::<T>()`. ROADMAP SC#2 literal wording is wrong — planner must annotate it as "through the ferro-ai SDK / `LlmClient`".
- **D-03:** Preserve two-pass flow exactly (Pass 1 plain text, Pass 2 structured + catalog schema). Migration only — no generation redesign.
- **D-03b:** Prompt builders (`build_json_view_pass1/2`, `scan_models`, `scan_routes`) move out of `ai.rs` into `make_json_view.rs` or a sibling `make_json_view_prompts.rs`. They have no Anthropic coupling.
- **D-04:** Gate on `AiConfig::from_env()` success/failure, not `ANTHROPIC_API_KEY` presence. `--no-ai` flag short-circuits before any client construction.
- **D-04b:** Provider/model/key controlled by `FERRO_AI_PROVIDER` / `FERRO_AI_MODEL` / `FERRO_AI_API_KEY`. Old `ANTHROPIC_API_KEY` + `FERRO_AI_MODEL` reads inside `ai.rs` are removed.
- **D-05:** No `temperature`, no `cache_control`. Map `max_tokens` per pass (Pass 1 ~1024, Pass 2 ~4096). Accept the loss for this phase.
- **D-06:** Keep `reqwest` `blocking` feature — `api_check.rs` still uses `reqwest::blocking::Client`. SC#1 is scoped to the deleted AI client only.

### Claude's Discretion

- Where relocated prompt-builder/scan helpers live (inline in `make_json_view.rs` vs `commands/make_json_view_prompts.rs`).
- One `Runtime` for the whole command vs per-call — prefer one runtime built once in `generate_with_ai`, reused across both passes.
- Whether to add a regression test asserting static-template fallback produces a catalog-valid spec when `AiConfig::from_env()` errors.

### Deferred Ideas (OUT OF SCOPE)

- `temperature: Option<f32>` on `CompletionRequest` — future ferro-ai SDK enhancement.
- Prompt `cache_control` (ephemeral) on system prompts — future SDK enhancement.
- `make:json-view` v2 redesign — Phase 173.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AISDK-06 | `ferro-cli/src/ai.rs` blocking client deleted; ferro-cli depends on ferro-ai and routes all LLM calls through it | All findings below directly enable this: SDK entry point confirmed, async bridge pattern confirmed, Cargo.toml changes identified, publish.yml wave ordering verified |
</phase_requirements>

---

## Summary

Phase 170 is a transport-swap migration. The blocking Anthropic-only `ferro-cli/src/ai.rs` (411 lines) is deleted and replaced with calls through the `ferro-ai` SDK's `AiConfig::from_env()` → `Box<dyn LlmClient>` → `client.complete()` path. The blast radius is tightly contained: one command module (`make_json_view.rs`), one module declaration (`lib.rs`), and `Cargo.toml`.

The central technical challenge is the async→sync bridge. `ferro-cli/src/main.rs:507` is a plain `fn main()` — no `#[tokio::main]`, no runtime. The `LlmClient::complete` method is `async fn`. The bridge is `tokio::runtime::Runtime::new()?.block_on(...)` constructed once inside `generate_with_ai`, reused across both passes. tokio "full" is already a ferro-cli dependency so no new dep is needed.

The provider gating changes from `std::env::var("ANTHROPIC_API_KEY")` to `AiConfig::from_env()` return value. `from_env()` reads `FERRO_AI_PROVIDER` (default "anthropic"), `FERRO_AI_MODEL`, `FERRO_AI_API_KEY`, and `FERRO_AI_BASE_URL`. It returns `Err(Error::Config)` when the provider is unknown or a required key is absent. Backward compat: for provider="anthropic", `ANTHROPIC_API_KEY` is accepted as a fallback when `FERRO_AI_API_KEY` is not set.

**Primary recommendation:** Implement in two focused tasks — (1) wire `ferro-ai` dep + async bridge + call sites + prompt relocation, (2) update tests and fix any lint.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| LLM provider selection / auth | ferro-ai SDK | — | `AiConfig::from_env()` owns all env reads and client construction |
| Async→sync bridge | ferro-cli (call site) | — | Bridge lives at the boundary, not in the SDK |
| Two-pass prompt building | ferro-cli (`make_json_view`) | — | Prompt builders are transport-agnostic, stay in the CLI |
| Catalog schema for Pass 2 | ferro-json-ui | ferro-cli (calls it) | `global_catalog().json_schema()` is the source of truth |
| Static-template fallback | ferro-cli (`templates`) | — | Unchanged from current implementation |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-ai` | workspace (0.2.x) | `AiConfig`, `LlmClient`, `CompletionRequest` | The SDK being migrated to; already in workspace |
| `tokio` | 1, features = ["full"] | `Runtime::new().block_on()` for async→sync bridge | Already a ferro-cli dependency [VERIFIED: ferro-cli/Cargo.toml] |
| `reqwest` | 0.12, features = ["blocking", "json"] | Keep as-is; `blocking` used by `api_check.rs` | Must NOT drop `blocking` feature [VERIFIED: ferro-cli/Cargo.toml + api_check.rs] |

### No New Dependencies Required

tokio "full" is already present. The only Cargo.toml change is adding `ferro-ai = { path = "../ferro-ai", version = "0.2" }` to `[dependencies]`.

**Installation (Cargo.toml addition):**
```toml
ferro-ai = { path = "../ferro-ai", version = "0.2" }
```

---

## Architecture Patterns

### System Architecture Diagram

```
ferro make:json-view <name> [--description] [--layout] [--no-ai]
          │
          ▼
  make_json_view::run()   [sync]
          │
          ├─── --no-ai? ──────────────────────────────────────────► templates::json_view_template()
          │                                                                      │
          │                                                                      ▼
          │                                                             write src/views/{name}.json
          │
          ▼
  AiConfig::from_env()   [sync]
          │
          ├─── Err(Config) ───────────────────────────────────────► eprintln! yellow warning
          │                                                        ► templates::json_view_template()
          │
          ▼
  generate_with_ai()   [sync wrapper]
     tokio::runtime::Runtime::new()   [constructed ONCE here]
          │
          ├── Pass 1: runtime.block_on(client.complete(req_pass1))
          │         req: system=pass1_system, messages=[user_msg], max_tokens=1024, schema=None
          │         ── Err ──► eprintln! yellow + fallback to static template
          │
          ▼
     Pass 1 plain-text plan
          │
          ├── Pass 2: runtime.block_on(client.complete(req_pass2))
          │         req: system=pass2_system, messages=[user_msg], max_tokens=4096,
          │              schema=Some(global_catalog().json_schema().clone())
          │         ── Err ──► eprintln! yellow + fallback to static template
          │
          ▼
     JSON string from provider
          │
          ├── Spec::from_json(&json_str)  ── Err ──► eprintln! yellow + fallback
          │
          ▼
     catalog.validate(&spec)  ── Err ──► eprintln! yellow + fallback
          │
          ▼
     write src/views/{name}.json
```

### Recommended Project Structure Change

```
ferro-cli/src/
├── ai.rs                        # DELETE — replaced entirely
├── lib.rs                       # Remove `pub mod ai;`
├── commands/
│   ├── make_json_view.rs        # Rewrite generate_with_ai(); receive prompt helpers
│   └── make_json_view_prompts.rs  (optional split — Claude's discretion)
└── Cargo.toml                   # Add ferro-ai dep
```

### Pattern 1: Async→Sync Bridge at the Call Boundary

**What:** Construct a `tokio::runtime::Runtime` once in the sync function that needs to call async code. Reuse it across multiple `block_on` calls.

**When to use:** Any sync function in a non-async binary needing to call `async fn`.

**Pitfall avoided:** `tokio::runtime::Handle::current()` + `block_on` panics with "Cannot start a runtime from within a runtime" if called inside an existing tokio context. `fn main()` in ferro-cli is NOT inside a runtime (verified: no `#[tokio::main]`), so `Runtime::new()` is safe.

**Example:**
```rust
// Source: verified against tokio docs + codebase analysis
fn generate_with_ai(file_name: &str, title: &str, layout_name: &str, description: &str) -> String {
    let client = match ferro_ai::AiConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} No AI provider configured: {}", style("Info:").yellow().bold(), e);
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} Failed to create tokio runtime: {}", style("Warning:").yellow().bold(), e);
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    // Pass 1
    let (sys1, usr1) = build_json_view_pass1(file_name, description);
    let req1 = ferro_ai::CompletionRequest {
        system: Some(sys1),
        messages: vec![ferro_ai::Message { role: ferro_ai::Role::User, content: usr1, tool_call_id: None }],
        max_tokens: 1024,
        model_override: None,
        schema: None,
        tools: None,
        tool_choice: None,
    };
    let pass1_result = match rt.block_on(client.complete(req1)) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{} AI Pass 1 failed: {}", style("Warning:").yellow().bold(), e);
            eprintln!("{}", style("Falling back to static template.").dim());
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    // Pass 2
    let schema = ferro_json_ui::global_catalog().json_schema().clone();
    let (sys2, usr2) = build_json_view_pass2(&pass1_result);
    let req2 = ferro_ai::CompletionRequest {
        system: Some(sys2),
        messages: vec![ferro_ai::Message { role: ferro_ai::Role::User, content: usr2, tool_call_id: None }],
        max_tokens: 4096,
        model_override: None,
        schema: Some(schema),
        tools: None,
        tool_choice: None,
    };
    let json_str = match rt.block_on(client.complete(req2)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} AI Pass 2 failed: {}", style("Warning:").yellow().bold(), e);
            eprintln!("{}", style("Falling back to static template.").dim());
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    // Validation unchanged
    // ...
}
```

### Pattern 2: Provider Gating via AiConfig

**Old gate (to delete):**
```rust
match std::env::var("ANTHROPIC_API_KEY") {
    Ok(_) => { /* AI path */ }
    Err(_) => { /* static template */ }
}
```

**New gate (D-04):**
```rust
match ferro_ai::AiConfig::from_env() {
    Ok(client) => generate_with_ai(file_name, title, layout_name, desc),
    Err(_) => {
        if description.is_some() {
            eprintln!("{} No AI provider configured ...", style("Info:").yellow().bold());
        }
        templates::json_view_template(file_name, title, layout_name)
    }
}
```

### Anti-Patterns to Avoid

- **Using `tokio::runtime::Handle::current().block_on()`:** Panics at runtime if called from within an existing tokio context. Use `Runtime::new()` instead.
- **Constructing one Runtime per pass:** Wasteful — construct once in `generate_with_ai`, call `block_on` twice on the same instance.
- **Using `complete::<T>()`:** Derives schema from `schemars::schema_for!(T)`. `ferro_json_ui::Spec` does not implement `JsonSchema`. Pass 1 needs no schema at all. Pass 2 needs the runtime-built catalog schema, not a schemars-derived schema. Wrong tool for both passes.
- **Force-deriving `JsonSchema` on `Spec`:** Would diverge the generated schema from the catalog validator (`catalog.validate(&spec)` is the truth). Correctness hazard, explicitly rejected in D-02.
- **Removing `reqwest` `blocking` feature:** `api_check.rs` line 1: `use reqwest::blocking::Client;`. Removing it breaks api_check compilation.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Provider selection / auth | Custom env-var parsing per provider | `AiConfig::from_env()` | Already handles 4 providers + fallback compat for `ANTHROPIC_API_KEY` |
| Async HTTP to LLM | `reqwest::blocking::Client` calling Anthropic API | `client.complete(CompletionRequest)` | Provider-agnostic; handles Anthropic tool_use extraction, OpenAI response parsing, error mapping |
| Schema for structured output | Hand-building JSON schema | `global_catalog().json_schema()` | Catalog schema is the validation source of truth; hand-building diverges from it |

---

## Specific Questions — Answered

### Q1: Async→Sync Bridge

`fn main()` is confirmed plain `fn main()` at line 507 of `ferro-cli/src/main.rs` [VERIFIED: source read]. No `#[tokio::main]`, no existing runtime. `tokio::runtime::Runtime::new()?.block_on(...)` is correct and safe here.

Construct one `Runtime` at the top of `generate_with_ai`, pass both `block_on` calls through it. There is no nested-runtime risk because the calling stack is fully synchronous through to `main()`.

### Q2: Exact Call Shape and Return Type

`LlmClient::complete` signature [VERIFIED: ferro-ai/src/client/mod.rs]:
```rust
async fn complete(&self, request: CompletionRequest) -> Result<String, Error>;
```

For Pass 1 (`schema: None`): the provider returns the assistant's text content as a plain `String`. For Pass 2 (`schema: Some(catalog_schema)`): the Anthropic provider uses tool_use under the hood (wraps the schema as `emit_spec` tool input — same mechanism as the old `call_anthropic_structured`), extracts the tool input JSON, serializes it, and returns it as a JSON `String`. The caller receives the JSON string and calls `Spec::from_json(&json_str)` then `catalog.validate(&spec)` — unchanged from today.

`CompletionRequest.schema` field docs [VERIFIED: ferro-ai/src/client/mod.rs]: "Passed through to the provider as-is." The catalog schema (`serde_json::Value`) passes through unchanged.

### Q3: Error Mapping

`ferro_ai::Error` variants relevant to `generate_with_ai` [VERIFIED: ferro-ai/src/error.rs]:

| Variant | When it occurs | User-facing message strategy |
|---------|---------------|------------------------------|
| `Error::Config(msg)` | Missing/unknown provider — returned by `from_env()` before any HTTP call | Gate check: treat as "no AI configured", fall back to static template with info message |
| `Error::Provider { status, message }` | HTTP error from LLM API | `format!("AI error ({status:?}): {message}")` → yellow warning → fallback |
| `Error::Unsupported` | Provider doesn't support `complete()` (not currently an issue) | Yellow warning → fallback |
| Other variants | Should not occur in the two-pass flow | Yellow warning with `e.to_string()` → fallback |

The `Error` type implements `Display` via `thiserror`, so `e.to_string()` produces a human-readable message for any variant. The catch-all error path in `generate_with_ai` can use `e` directly.

### Q4: Provider Gating

[VERIFIED: ferro-ai/src/config.rs]

`AiConfig::from_env()` reads:
- `FERRO_AI_PROVIDER` (default "anthropic")
- `FERRO_AI_MODEL` (optional — overrides provider default)
- `FERRO_AI_API_KEY` (required for anthropic/openai/groq; not required for ollama)
- `FERRO_AI_BASE_URL` (optional override)

Returns `Err(Error::Config("FERRO_AI_API_KEY not set".into()))` when provider requires a key and none is set. For anthropic, `ANTHROPIC_API_KEY` is accepted as a fallback (backward compat confirmed in source).

Gate pattern: call `AiConfig::from_env()` at the point `make_json_view::run()` decides whether to use AI. On `Ok(client)`, pass the client into `generate_with_ai`. On `Err`, emit info message if `--description` was provided, then use static template.

### Q5: max_tokens Parity

[VERIFIED: ferro-cli/src/ai.rs]

Old values: Pass 1 `"max_tokens": 1024`, Pass 2 `"max_tokens": 4096`. These map directly to `CompletionRequest.max_tokens: u32`. No conversion needed.

### Q6: Test Impact

[VERIFIED: ferro-cli/src/commands/make_json_view.rs tests + ferro-cli/src/templates/mod.rs tests]

Tests in `make_json_view.rs` that survive unchanged:
- `to_snake_case_basic` — pure function, no AI dependency
- `to_title_case_basic` — pure function, no AI dependency
- `is_valid_identifier_accepts_snake_case` — pure function, no AI dependency
- `is_valid_identifier_rejects_invalid` — pure function, no AI dependency
- `static_fallback_produces_valid_spec` — calls `crate::templates::json_view_template` directly, no AI dependency

These tests have zero dependency on `ai::`. They survive the deletion of `ai.rs` without any changes.

Tests in `templates/mod.rs` — none reference `ai::`. All survive.

The only test breakage is the `pub mod ai;` declaration in `lib.rs` — deleting `ai.rs` without removing the declaration causes a compilation error. That mod line must be removed from `lib.rs` as part of the migration.

No existing test exercises the AI path through `generate_with_ai` (correct — the AI path requires a live provider and is not unit-tested). The optional regression test (Claude's discretion) would cover: `AiConfig::from_env()` returns `Err` → `generate_with_ai` produces a catalog-valid static template. This can be a synchronous test since it only exercises the fallback, not the async path.

### Q7: publish.yml Wave Ordering

[VERIFIED: .github/workflows/publish.yml]

- `ferro-ai` is in **Wave 1b**: `WAVE1B_CRATES="ferro-projections ferro-ai ferro-stripe ..."`
- `ferro-cli` is in **Wave 3**: `WAVE3_CRATES="ferro-cli ferro-bundle"`

Wave 3 runs after Wave 1b (confirmed by step ordering in the workflow). Adding `ferro-cli → ferro-ai` dependency requires **no change** to `publish.yml` — ferro-ai is already published before ferro-cli in the current wave structure.

---

## Common Pitfalls

### Pitfall 1: Nested Runtime Panic

**What goes wrong:** Calling `tokio::runtime::Runtime::new()` inside a function that is already executing inside a tokio runtime (e.g., a function called from `#[tokio::main]`) panics with "Cannot start a runtime from within a runtime".

**Why it happens:** tokio prevents nested runtimes to avoid deadlocks.

**How to avoid:** Verified that `ferro-cli/src/main.rs` uses `fn main()` — no existing runtime. Safe to construct `Runtime::new()` in `generate_with_ai`. As long as the CLI stays synchronous (D-01), this is not a risk.

**Warning signs:** If a future phase adds `#[tokio::main]` to ferro-cli main(), the `Runtime::new()` pattern would break. At that point, switch to `tokio::task::spawn_blocking` or just make the call sites `async`.

### Pitfall 2: Dropping the reqwest blocking Feature

**What goes wrong:** Removing `blocking` from `reqwest` features in `ferro-cli/Cargo.toml` causes `api_check.rs` line 1 (`use reqwest::blocking::Client;`) to fail to compile.

**Why it happens:** SC#1 says "no `reqwest::blocking::Client` in ferro-cli" but this must be read as scoped to the deleted AI client. `api_check.rs` legitimately uses the blocking client.

**How to avoid:** Keep `reqwest = { version = "0.12", features = ["blocking", "json"] }` exactly as-is.

### Pitfall 3: Over-deleting the scan Helpers

**What goes wrong:** Deleting `ai.rs` entirely including `scan_models`, `scan_routes`, `build_json_view_pass1`, `build_json_view_pass2` — these are used in `generate_with_ai` and are transport-agnostic.

**Why it happens:** "Delete `ai.rs`" can be read as "delete the whole file".

**How to avoid:** Relocate the four helpers (`build_json_view_pass1`, `build_json_view_pass2`, `scan_models`, `scan_routes`) to `make_json_view.rs` (or `make_json_view_prompts.rs`) BEFORE deleting `ai.rs`. The transport functions (`call_anthropic_plain`, `call_anthropic_structured`, `call_anthropic`, `generate_json_view`) are the only things being replaced — not the prompt/scan logic.

### Pitfall 4: Using generate_json_view from ai.rs Instead of generate_with_ai

**What goes wrong:** The old `ai.rs` contains `generate_json_view()` which is a higher-level orchestrator that internally calls `call_anthropic_plain` and `call_anthropic_structured`. `make_json_view.rs` does NOT call this function — it has its own `generate_with_ai()` that calls the lower-level functions directly. Confusing the two could lead to partially migrated state.

**Why it happens:** Both do the two-pass flow. The difference: `generate_with_ai` in the command module has the validation + fallback logic; `generate_json_view` in `ai.rs` has its own (slightly different) validation path.

**How to avoid:** Only `generate_with_ai()` in `make_json_view.rs` needs to be rewired. `ai::generate_json_view` in `ai.rs` is deleted along with the file — its call sites do not exist in the command module.

### Pitfall 5: Forgetting to Remove `pub mod ai;` from lib.rs

**What goes wrong:** `ferro-cli/src/lib.rs` line 7: `pub mod ai;`. If `ai.rs` is deleted without removing this line, the crate fails to compile with "file not found for module `ai`".

**How to avoid:** The lib.rs edit is a one-line deletion — include it explicitly in the plan task that deletes `ai.rs`.

---

## Code Examples

### AiConfig::from_env() confirmed signature and behavior

```rust
// Source: ferro-ai/src/config.rs (verified)
impl AiConfig {
    pub fn from_env() -> Result<Box<dyn LlmClient>, Error>
}
// Returns Err(Error::Config(...)) when:
// - FERRO_AI_PROVIDER is unknown
// - Required key missing (anthropic/openai/groq with no FERRO_AI_API_KEY or ANTHROPIC_API_KEY)
// Returns Ok(Box<dyn LlmClient>) when configured
// Note: for anthropic, ANTHROPIC_API_KEY is accepted as fallback for backward compat
```

### CompletionRequest struct (all fields, Pass 1 and Pass 2)

```rust
// Source: ferro-ai/src/client/mod.rs (verified)
// Pass 1 — plain text, no schema
CompletionRequest {
    system: Some(pass1_system_prompt),
    messages: vec![Message { role: Role::User, content: pass1_user_prompt, tool_call_id: None }],
    max_tokens: 1024,
    model_override: None,
    schema: None,          // No schema for plain-text pass
    tools: None,
    tool_choice: None,
}

// Pass 2 — structured output using catalog schema
CompletionRequest {
    system: Some(pass2_system_prompt),
    messages: vec![Message { role: Role::User, content: pass2_user_prompt, tool_call_id: None }],
    max_tokens: 4096,
    model_override: None,
    schema: Some(ferro_json_ui::global_catalog().json_schema().clone()),  // catalog schema pass-through
    tools: None,
    tool_choice: None,
}
```

### ferro_ai public re-exports to confirm import paths

The research confirms `AiConfig`, `CompletionRequest`, `Message`, `Role`, `LlmClient` are all accessible. Need to verify exact re-export paths from `ferro-ai`'s `lib.rs` to determine import syntax. [ASSUMED — check `ferro-ai/src/lib.rs` during implementation; use `ferro_ai::AiConfig`, `ferro_ai::client::CompletionRequest`, etc.]

---

## File Change Summary

| File | Change | Notes |
|------|--------|-------|
| `ferro-cli/src/ai.rs` | DELETE | Transport functions removed; helpers relocated |
| `ferro-cli/src/lib.rs` | Remove `pub mod ai;` (line 7) | One-line deletion |
| `ferro-cli/src/commands/make_json_view.rs` | Major rewrite of `generate_with_ai()` + receive relocated helpers | Remove `use crate::ai;`; add `use ferro_ai::{...}` |
| `ferro-cli/Cargo.toml` | Add `ferro-ai = { path = "../ferro-ai", version = "0.2" }` | No other dep changes |
| `.github/workflows/publish.yml` | NO CHANGE | ferro-ai already in Wave 1b; ferro-cli in Wave 3; ordering correct |

Optionally (Claude's discretion):
| `ferro-cli/src/commands/make_json_view_prompts.rs` | NEW — relocated helpers | If planner prefers split module |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — workspace test runner |
| Quick run command | `cargo test -p ferro-cli --lib` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AISDK-06 | `ai.rs` deleted, no `reqwest::blocking::Client` in AI path | compilation check | `cargo test -p ferro-cli --lib` | N/A — verified at compile time |
| AISDK-06 | `make_json_view` pure-function tests still pass | unit | `cargo test -p ferro-cli --lib -- make_json_view` | ✅ exists |
| AISDK-06 | static fallback produces catalog-valid spec | unit | `cargo test -p ferro-cli --lib -- static_fallback_produces_valid_spec` | ✅ exists |
| AISDK-06 | clippy clean | lint | `cargo clippy -p ferro-cli --all-targets -- -D warnings` | N/A — lint run |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-cli --lib`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

None — existing test infrastructure covers all phase requirements. No new test files needed (the optional regression test is discretionary, not required for AISDK-06).

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | n/a (CLI tool, no user auth) |
| V3 Session Management | no | n/a |
| V4 Access Control | no | n/a |
| V5 Input Validation | no | view name validated by existing `is_valid_identifier` check |
| V6 Cryptography | no | API key read from env, not generated or stored |

No new security surface introduced. The migration removes a direct HTTP client and delegates to the SDK — which handles API key hygiene (the `Error::Provider` docs explicitly state the message "MUST NOT contain the API key or auth header").

---

## Open Questions

1. **ferro-ai lib.rs re-exports**
   - What we know: `AiConfig`, `LlmClient`, `CompletionRequest`, `Message`, `Role` are all defined in ferro-ai.
   - What's unclear: Exact public re-export paths from `ferro-ai/src/lib.rs` — whether to use `ferro_ai::AiConfig` vs `ferro_ai::config::AiConfig`, etc.
   - Recommendation: Implementer reads `ferro-ai/src/lib.rs` first to confirm import paths. Likely `ferro_ai::AiConfig` and `ferro_ai::client::{CompletionRequest, Message, Role}` based on standard module structure — but verify.

2. **Pass 2 JSON response format**
   - What we know: `client.complete()` returns `Result<String, Error>`. The Anthropic provider uses tool_use internally to enforce structured output.
   - What's unclear: Whether the returned `String` is already pretty-printed JSON or compact JSON. The old `call_anthropic_structured` used `serde_json::to_string_pretty`. The new SDK may return compact JSON.
   - Recommendation: `Spec::from_json` calls `serde_json::from_str` internally, so compact vs pretty doesn't affect correctness. The file written to disk was pretty-printed by `serde_json::to_string_pretty` in the old code — if the new code writes compact JSON, the file format changes slightly but functionality is identical. Accept this minor formatting difference or add a `serde_json::to_string_pretty` pass on the returned string before writing.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | ferro-ai re-exports `AiConfig`, `LlmClient`, `CompletionRequest`, `Message`, `Role` at crate root or accessible module paths | Code Examples | Import paths would need adjustment — low risk, one-line fix |
| A2 | The returned `String` from `client.complete()` with a schema is valid JSON that `Spec::from_json` can parse directly | Q2 answer | If SDK wraps the string in extra structure, Pass 2 parsing would fail — medium risk, verify with a smoke test |

---

## Sources

### Primary (HIGH confidence)

- `ferro-cli/src/ai.rs` — full source read; transport functions, prompt builders, scan helpers all confirmed
- `ferro-cli/src/commands/make_json_view.rs` — full source read; `generate_with_ai` flow, test inventory confirmed
- `ferro-ai/src/config.rs` — full source read; `AiConfig::from_env()` behavior, env vars, error variants confirmed
- `ferro-ai/src/client/mod.rs` — full source read; `LlmClient` trait, `CompletionRequest` fields confirmed
- `ferro-ai/src/complete.rs` — full source read; why `complete::<T>()` cannot be used (schemars vs catalog schema) confirmed
- `ferro-ai/src/error.rs` — full source read; all `Error` variants confirmed
- `ferro-cli/src/lib.rs` — confirmed `pub mod ai;` at line 7
- `ferro-cli/Cargo.toml` — confirmed tokio "full", reqwest "blocking"+"json", ferro-ai absent
- `ferro-cli/src/commands/api_check.rs` — confirmed `use reqwest::blocking::Client;`
- `ferro-cli/src/main.rs` — confirmed plain `fn main()` at line 507; no `#[tokio::main]`
- `.github/workflows/publish.yml` — confirmed ferro-ai in Wave 1b, ferro-cli in Wave 3

### Secondary (MEDIUM confidence)

- tokio documentation: `Runtime::new().block_on()` is the correct sync→async bridge pattern for non-async binaries [ASSUMED — standard tokio usage, well-established pattern]

---

## Metadata

**Confidence breakdown:**
- Code inventory (what to delete, what to relocate): HIGH — direct source reads
- SDK call shape (CompletionRequest fields, return type): HIGH — direct source reads
- Async→sync bridge correctness: HIGH — `fn main()` confirmed sync, tokio pattern is standard
- publish.yml wave ordering: HIGH — direct workflow read
- Test impact: HIGH — all test files read

**Research date:** 2026-06-08
**Valid until:** 2026-07-08 (stable internal APIs — ferro workspace)
