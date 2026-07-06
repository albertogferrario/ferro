# Phase 172: MCP Tool Wrappers - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 7 (3 new, 4 modified)
**Analogs found:** 7 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-mcp/src/tools/ai_scaffold.rs` | service (async core) | request-response + LLM streaming | `ferro-mcp/src/tools/ai.rs` (`test_classifier`) + `ferro-cli/src/commands/ai_make.rs` | role-match |
| `ferro-mcp/src/tools/ai_explain_core.rs` | service (async core) | request-response + LLM streaming | `ferro-cli/src/commands/ai_explain.rs` full body + `ferro-mcp/src/tools/ai.rs` | role-match |
| `ferro-mcp/src/tools/relevance.rs` | utility | transform | `ferro-cli/src/relevance.rs` (verbatim relocation) | exact |
| `ferro-mcp/src/tools/mod.rs` | config (module registry) | — | current `ferro-mcp/src/tools/mod.rs` | exact |
| `ferro-mcp/src/service.rs` | service (tool registration) | request-response | existing `#[tool]` methods at lines 1670–1705 | exact |
| `ferro-cli/src/commands/ai_make.rs` | utility (thin wrapper) | request-response | current `run()` body at lines 490–715 | exact |
| `ferro-cli/src/commands/ai_explain.rs` | utility (thin wrapper) | request-response | current `run()` body at lines 284–376 | exact |

---

## Pattern Assignments

### `ferro-mcp/src/tools/ai_scaffold.rs` (new async core, request-response + LLM)

**Primary analog:** `ferro-cli/src/commands/ai_make.rs` (body of `run()`)
**Shape analog:** `ferro-mcp/src/tools/ai.rs` (`test_classifier` function)

**Module-level doc and imports** — follow `ai.rs` lines 1–9:
```rust
//! Core logic for the `ai_scaffold` MCP tool and the `ferro ai:make` CLI wrapper.
//!
//! `scaffold_core` is the single definition site for ServiceDef generation.

use ferro_ai::{AiConfig, CompleteOptions};
use ferro_projections::ServiceDef;
use std::path::Path;
use crate::tools::{database_schema, generation_context, list_models, list_projections, list_routes, relevance};
```

Note: no `#[cfg(feature = "projections")]` — `ferro-mcp` depends on `ferro-projections` unconditionally (`ferro-mcp/Cargo.toml:25`).

**Core function signature** — return `Result<ServiceDef, String>`, no `process::exit`, no coloring:
```rust
/// Generate a `ServiceDef` from a natural-language description using live introspection.
///
/// Returns `Ok(ServiceDef)` on success. Errors are model-legible strings — no
/// `process::exit`, no `eprintln!`, no `console::style`. Those stay in the CLI wrapper.
pub async fn scaffold_core(
    description: &str,
    project_root: &Path,
) -> Result<ServiceDef, String> {
    // 1. AiConfig::from_env() → map Err to String (not process::exit)
    // 2. Sync introspection: call directly (no block_on — already async)
    // 3. Async introspection: .await (not rt.block_on)
    // 4. Build candidates → relevance::select_relevant()
    // 5. sanitize_description() + assemble prompt
    // 6. complete_with::<ServiceDef>(...).await → map Err to String
    // 7. service.validate() → map Err to String
    // 8. Ok(service)
}
```

**Introspection call pattern** (moved verbatim from `ai_make.rs:524–615`):
```rust
// Sync — call directly in async context, no .await:
let models = list_models::execute(root).unwrap_or_default();         // ai_make.rs:525
let gen_ctx = generation_context::execute();                          // ai_make.rs:526
let projections = list_projections::execute(root, None);              // ai_make.rs:527

// Async — .await directly (no rt.block_on; that bridge stays CLI-only):
let routes = list_routes::execute(root).await.unwrap_or_else(|_| {  // ai_make.rs:531
    ferro_mcp::tools::list_routes::RoutesInfo {
        routes: vec![],
        source: ferro_mcp::tools::list_routes::RouteSource::StaticAnalysis,
    }
});
let schema = database_schema::execute(root, None).await               // ai_make.rs:539
    .unwrap_or_else(|_| ferro_mcp::tools::database_schema::SchemaInfo { tables: vec![] });
```

**Candidate assembly pattern** (verbatim from `ai_make.rs:544–617`):
```rust
let mut candidates: Vec<relevance::Candidate> = Vec::new();

// Tier 3: projections (highest)
for p in &projections.projections {
    let mut tokens: std::collections::HashSet<String> =
        relevance::tokenize(&p.name).into_iter().collect();
    // ... extend with service_name, display_name tokens
    candidates.push(relevance::Candidate { label, tokens, serialized, tier: 3 });
}
// Tier 2: models (tier: 2), Tier 1: routes (tier: 1), Tier 0: schema tables (tier: 0)
let selected = relevance::select_relevant(&description, candidates);
```

**Prompt assembly and LLM call** (verbatim from `ai_make.rs:619–672`):
```rust
let system_prompt =
    "You are a Ferro framework expert. Generate a valid ferro_projections::ServiceDef \
     for the described domain service. Use ONLY the introspection context provided. \
     Reference actual model names, field names, and route patterns from the context. \
     Do NOT use generic placeholders — every field should reflect the real project."
        .to_string();

let safe_description = sanitize_description(description);
let user_prompt = format!(
    "Project introspection:\n{context_block}\n\n\
     <description>\n{safe_description}\n</description>"
);
let max_tokens = resolve_max_tokens();   // reads FERRO_AI_MAX_TOKENS_PER_COMMAND

let service: ServiceDef = ferro_ai::complete_with::<ServiceDef>(
    client.as_ref(),
    &user_prompt,
    CompleteOptions { max_tokens, system: Some(system_prompt), model_override: None },
).await.map_err(|e| e.to_string())?;

service.validate().map_err(|e| format!("ServiceDef validation failed: {e}"))?;
Ok(service)
```

**Helper functions to relocate from `ai_make.rs`** — keep same bodies:
- `sanitize_description(description: &str) -> String` (lines 473–477)
- `resolve_max_tokens() -> u32` (lines 373–378)
- `ai_config_error_message(e: &ferro_ai::Error) -> String` (lines 388–392)

**Error handling pattern** — never panic, never exit. Map all errors to `String`:
```rust
// Source: ai.rs lines 64–86 (test_classifier error handling template)
let client = AiConfig::from_env().map_err(|e| ai_config_error_message(&e))?;
let service = ferro_ai::complete_with::<ServiceDef>(...).await.map_err(|e| e.to_string())?;
```

**Test pattern** — mirror `ai.rs` tests + `ai_make.rs` tests; use `ENV_LOCK` for env-touching tests:
```rust
// Source: ferro-cli/src/commands/mod.rs:78 — the process-wide env mutex
// Relocate to ferro-mcp (e.g. in lib.rs or a test_support module):
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// Source: ferro-mcp/src/tools/ai.rs:229–338 — test structure template
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;  // already in ferro-mcp dev-dependencies (Cargo.toml:40)

    #[test]
    fn sanitize_description_strips_xml_delimiters() {
        // mirrors ferro-cli test — relocate here
    }
}
```

---

### `ferro-mcp/src/tools/ai_explain_core.rs` (new async core, request-response)

**Primary analog:** `ferro-cli/src/commands/ai_explain.rs` (full body)
**Shape analog:** `ferro-mcp/src/tools/ai.rs` for the result struct pattern

**Imports** — no `rt` parameter, no `cfg(feature)` guard:
```rust
// Source: ferro-cli/src/commands/ai_explain.rs:10–16
use crate::tools::{
    explain_model::ModelExplanation,
    explain_route::RouteExplanation,
    inspect_projection::{InspectResult, ProjectionDetail},
};
use ferro_ai::{AiConfig, CompletionRequest};
use ferro_ai::client::{Message, Role};
use std::path::Path;
```

**`ResolvedTarget` enum** — copy verbatim from `ai_explain.rs:24–33`, change visibility to `pub`:
```rust
// Source: ferro-cli/src/commands/ai_explain.rs:24-33
pub enum ResolvedTarget {
    Service(ProjectionDetail),
    Route(RouteExplanation),
    Model(ModelExplanation),
    NotFound(String),
}
```

**`resolve_kind_priority`** — copy verbatim from `ai_explain.rs:48–67`:
```rust
// Source: ferro-cli/src/commands/ai_explain.rs:48-67
pub fn resolve_kind_priority(
    found_service: bool,
    found_route: bool,
    found_model: bool,
    type_override: Option<&str>,
) -> &str { ... }
```

**`resolve_target`** — key mutation: remove `rt: &tokio::runtime::Runtime` parameter, replace `rt.block_on(explain_route::execute(...))` with `explain_route::execute(...).await`. Source at `ai_explain.rs:78–125`:
```rust
// BEFORE (CLI, sync): rt.block_on(explain_route::execute(root, target))
// AFTER (MCP, async): explain_route::execute(root, target).await

pub async fn resolve_target(
    root: &Path,
    target: &str,
    type_override: Option<&str>,
) -> ResolvedTarget {
    use crate::tools::{explain_model, explain_route, inspect_projection};

    match type_override {
        Some("service") => match inspect_projection::execute(root, target) {  // still sync
            InspectResult::Found(d) => return ResolvedTarget::Service(d),
            InspectResult::NotFound(_) => return ResolvedTarget::NotFound(...),
        },
        Some("route") => match explain_route::execute(root, target).await {   // .await not rt.block_on
            Ok(r) => return ResolvedTarget::Route(r),
            Err(_) => return ResolvedTarget::NotFound(...),
        },
        // ... model branch same pattern
    }
}
```

**`explain_core` — two-branch result** (D-03 from CONTEXT.md):
```rust
pub async fn explain_core(
    target: &str,
    type_override: Option<&str>,
    project_root: &Path,
) -> Result<serde_json::Value, String> {
    let resolved = resolve_target(project_root, target, type_override).await;
    match resolved {
        // Zero-token branch: ProjectionDetail is already Serialize — no LLM call
        // Source: ferro-mcp/src/tools/inspect_projection.rs:9 (#[derive(Serialize)] on ProjectionDetail)
        ResolvedTarget::Service(detail) => {
            serde_json::to_value(&detail).map_err(|e| e.to_string())
        }
        // Prose branches: share the exact prompt builders + CompletionRequest pattern
        // from ai_explain.rs:328-374
        ResolvedTarget::Route(r) => {
            let (sys, user) = build_route_prompt(&r);
            let prose = call_llm_prose(sys, user).await?;
            Ok(serde_json::json!({ "prose": prose }))
        }
        ResolvedTarget::Model(m) => {
            let (sys, user) = build_model_prompt(&m);
            let prose = call_llm_prose(sys, user).await?;
            Ok(serde_json::json!({ "prose": prose }))
        }
        ResolvedTarget::NotFound(msg) => Err(msg),
    }
}
```

**Prose completion helper** — derived from `ai_explain.rs:348–374`:
```rust
// Source: ferro-cli/src/commands/ai_explain.rs:348-374
async fn call_llm_prose(system: String, user: String) -> Result<String, String> {
    let client = AiConfig::from_env().map_err(|e| e.to_string())?;
    let max_tokens = resolve_max_tokens_with_default(2048);
    let req = CompletionRequest {
        system: Some(system),
        messages: vec![Message { role: Role::User, content: user, tool_call_id: None }],
        max_tokens,
        model_override: None,
        schema: None,     // <-- prose, not structured JSON
        tools: None,
        tool_choice: None,
    };
    client.complete(req).await.map_err(|e| e.to_string())
}
```

**Prompt builders to relocate** — copy verbatim from `ai_explain.rs`, make `pub`:
- `build_service_prompt(detail: &ProjectionDetail) -> (String, String)` (lines 141–208)
- `build_route_prompt(r: &RouteExplanation) -> (String, String)` (lines 212–231)
- `build_model_prompt(m: &ModelExplanation) -> (String, String)` (lines 235–258)
- `resolve_max_tokens_with_default(default: u32) -> u32` (lines 267–272)

**`ProjectionDetail` serialization shape** — the structured branch returns this directly via `serde_json::to_value`. Exact fields (from `inspect_projection.rs:10–30`):
```
ProjectionDetail {
  name: String,
  file: String,
  service_name: String,
  display_name: Option<String>,
  fields: Vec<FieldInfo { name, data_type, meaning, readable, writable }>,
  relationships: Vec<String>,
  actions: Vec<String>,
  has_state_machine: bool,
  intent_hints: Vec<String>,   // e.g. "Primary(Browse)"
}
```
All fields are strings/bools — no typed `Intent` or `ActionDef` enums in the output. SC#2's "Intent, FieldMeaning, ActionDef, StateMachine" vocabulary is present as the string values in `intent_hints`, `fields[].meaning`, `actions`, and `has_state_machine`.

---

### `ferro-mcp/src/tools/relevance.rs` (new utility — verbatim relocation)

**Source:** `ferro-cli/src/relevance.rs` (entire file, lines 1–169)

**Only change required:** `pub(crate)` → `pub` on all exported items so the CLI thin wrapper can import them via `ferro_mcp::tools::relevance::*`:
```rust
// BEFORE (ferro-cli/src/relevance.rs:5)
pub(crate) const INPUT_BUDGET_CHARS: usize = 8000;
pub(crate) fn tokenize(s: &str) -> Vec<String> { ... }
pub(crate) struct Candidate { ... }
pub(crate) fn select_relevant(description: &str, ...) -> Vec<String> { ... }

// AFTER (ferro-mcp/src/tools/relevance.rs)
pub const INPUT_BUDGET_CHARS: usize = 8000;
pub fn tokenize(s: &str) -> Vec<String> { ... }
pub struct Candidate { ... }
pub fn select_relevant(description: &str, ...) -> Vec<String> { ... }
```

All tests from `ferro-cli/src/relevance.rs:97–169` move here unchanged (`use super::*` still works).

---

### `ferro-mcp/src/tools/mod.rs` (modified — add 3 module declarations)

**Analog:** existing `ferro-mcp/src/tools/mod.rs` (alphabetical `pub mod` list, lines 1–63)

**Pattern** (copy from any existing entry, e.g. line 3):
```rust
// Add in alphabetical order alongside existing entries:
pub mod ai_explain_core;   // after pub mod ai;
pub mod ai_scaffold;        // after pub mod ai_explain_core;
pub mod relevance;          // alphabetically near 'r' entries
```

---

### `ferro-mcp/src/service.rs` (modified — add 2 params structs + 2 tool methods)

**Params structs — analog:** `TestClassifierParams` at lines 333–343. Copy derives exactly:
```rust
// Source: ferro-mcp/src/service.rs:333-343
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TestClassifierParams {
    pub system_prompt: String,
    pub user_prompt: String,
    pub schema_json: String,
    pub model: Option<String>,
}

// New structs follow identical derive + field pattern:
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiScaffoldParams {
    /// Natural-language description of the service to scaffold.
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiExplainParams {
    /// Target: route path (e.g. "/orders/{id}"), model name, or service name.
    pub target: String,
    /// Optional type hint: "service", "route", or "model". Auto-detect if omitted.
    pub type_override: Option<String>,
}
```

**Tool method registration — analog:** `test_classifier` at lines 1670–1688. Copy structure exactly:
```rust
// Source: ferro-mcp/src/service.rs:1670-1688

#[tool(
    name = "test_classifier",
    description = "Test an AI classification by sending a prompt...\n\n\
        **When to use:** ...\n\n**Note:** ...\n\n**Returns:** ...\n\n**Combine with:** ..."
)]
pub async fn test_classifier(&self, params: Parameters<TestClassifierParams>) -> String {
    let ai_params = tools::ai::TestClassifierParams { ... };
    let result = tools::ai::test_classifier(&self.project_root, ai_params).await;
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
}
```

New tool methods follow the identical structure:
```rust
#[tool(
    name = "ai_scaffold",
    description = "Generate a ferro_projections::ServiceDef from a natural-language \
        description using the project's live introspection as context.\n\n\
        **When to use:** Starting a new service; getting a typed ServiceDef to pass \
        to renderers or inspect_projection.\n\n\
        **Returns:** A ServiceDef JSON object (same shape as `ferro ai:make` output). \
        Does NOT write files — use `ferro ai:make` when you want the .rs file written \
        to src/projections/.\n\n\
        **Note:** Makes a real LLM API call. Costs tokens. Requires FERRO_AI_PROVIDER, \
        FERRO_AI_API_KEY, FERRO_AI_MODEL.\n\n\
        **Combine with:** inspect_projection to see existing projections, \
        list_projections to avoid naming collisions, ai_explain to understand a \
        generated ServiceDef."
)]
pub async fn ai_scaffold(&self, params: Parameters<AiScaffoldParams>) -> String {
    let result = tools::ai_scaffold::scaffold_core(
        &params.0.description,
        &self.project_root,
    ).await;
    match result {
        Ok(service_def) => serde_json::to_string_pretty(&service_def)
            .unwrap_or_else(|_| "{}".to_string()),
        Err(e) => serde_json::to_string_pretty(&serde_json::json!({
            "success": false,
            "error": e,
        }))
        .unwrap_or_else(|_| r#"{"success":false}"#.to_string()),
    }
}
```

Note: the error JSON uses `{ "success": false, "error": "..." }` (from D-04). The success path returns the `ServiceDef` directly (not wrapped in `{ "success": true, ... }`), matching D-02 "return the ServiceDef as a pretty JSON object."

**`list_pending_confirmations` analog** (lines 1699–1705) — for tools that take params but don't use them:
```rust
// Source: ferro-mcp/src/service.rs:1699-1705
pub async fn list_pending_confirmations(
    &self,
    #[allow(unused_variables)] _params: Parameters<ListPendingConfirmationsParams>,
) -> String { ... }
```
`ai_explain` takes `params.0.target` and `params.0.type_override` — both are used, so no `#[allow(unused_variables)]`.

---

### `ferro-cli/src/commands/ai_make.rs` (modified — thin wrapper)

**What stays:** `emit_service_def_source`, `render_output`, `resolve_projection_path`, `OutputResult`, all `emit_*` helpers, `#[cfg(feature = "projections")]` guards everywhere, `console::style` usage, `process::exit`.

**What is deleted from `run()`:** everything from step 1 (`AiConfig::from_env`) through step 8 (`service.validate()`) inclusive. Replaced by a single call to the relocated core:

```rust
// New run() body (thin wrapper pattern):
#[cfg(feature = "projections")]
pub fn run(description: String, dry_run: bool) {
    use console::style;
    use ferro_mcp::tools::relevance;  // relevance is now re-exported from ferro-mcp

    // Tokio bridge stays CLI-side (D-04)
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} Failed to create tokio runtime: {e}", style("Error:").red().bold());
            std::process::exit(1);
        }
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let service = match rt.block_on(ferro_mcp::tools::ai_scaffold::scaffold_core(
        &description,
        &cwd,
    )) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} {e}", style("Error:").red().bold());
            std::process::exit(1);
        }
    };

    // render_output, console output — stays CLI-only
    match render_output(&service, dry_run, &cwd) { ... }
}
```

**Import update:** `crate::relevance::*` → `ferro_mcp::tools::relevance::*` (the module is deleted from ferro-cli). The `#[cfg(feature = "projections")]` guard stays on `run()` — the relocated core in `ferro-mcp` is unconditional but the CLI wrapper is still feature-gated.

---

### `ferro-cli/src/commands/ai_explain.rs` (modified — thin wrapper)

**What stays:** CLI output (`println!`, `eprintln!`), `process::exit`, dry-run prompt printing.

**What is deleted:** the *local* copies of `ResolvedTarget`, `resolve_kind_priority`, `resolve_target`, `build_service_prompt`, `build_route_prompt`, `build_model_prompt`. These move to `ferro-mcp` (now `pub`) and the CLI imports them.

**⚠ CLI behavior MUST stay prose-only (CONTEXT.md Deferred Ideas).** The CLI thin wrapper does **NOT** call `explain_core` — `explain_core` returns the structured projection JSON for a `ServiceDef`, and switching the CLI to print that is an explicitly deferred change. Instead the CLI wrapper calls the relocated **public** `resolve_target` + `build_*_prompt` + prose path directly, preserving the current prose-only + `--dry-run`-prints-prompt contract. This is governed by Plan 04 Task 1's `<action>`.

**New `run()` body pattern (prose-preserving):**
```rust
#[cfg(feature = "projections")]
pub fn run(target: String, type_override: Option<String>, dry_run: bool) {
    use console::style;
    use ferro_mcp::tools::ai_explain_core::{
        resolve_target, build_service_prompt, build_route_prompt, build_model_prompt, ResolvedTarget,
    };

    let rt = match tokio::runtime::Runtime::new() { ... };
    let cwd = std::path::Path::new(".");

    let resolved = rt.block_on(resolve_target(cwd, &target, type_override.as_deref()));
    let (system, user) = match &resolved {
        ResolvedTarget::Service(d) => build_service_prompt(d),
        ResolvedTarget::Route(r)   => build_route_prompt(r),
        ResolvedTarget::Model(m)   => build_model_prompt(m),
        ResolvedTarget::NotFound(msg) => { eprintln!("{} {msg}", style("Error:").red().bold()); std::process::exit(1); }
    };

    if dry_run {                       // unchanged contract: print the assembled prompt, no LLM
        println!("{system}\n---\n{user}");
        return;
    }
    // ... existing CLI prose completion (AiConfig::from_env + CompletionRequest, schema: None) prints prose ...
}
```
*Note: the structured-JSON output path (`explain_core`) is reserved for the MCP tool only; the CLI adopting it is a Deferred Idea per CONTEXT.md.*

**Test file update:** tests that currently call `resolve_kind_priority`, `build_service_prompt`, etc. directly must be moved to `ferro-mcp/src/tools/ai_explain_core.rs` alongside the relocated implementation. Tests that only validate thin-wrapper CLI behavior (exit codes, output formatting) can stay. The `use crate::commands::ENV_LOCK` reference in tests at `ai_explain.rs:386` changes to a module-level `static` in the new ferro-mcp test module.

---

### `Cargo.toml` (modified — workspace version bump)

**Analog:** current line 36:
```toml
# Source: Cargo.toml:36
version = "0.2.46"

# Change to:
version = "0.2.47"
```

No new crate entries needed anywhere: `ferro-mcp/Cargo.toml` already has `ferro-ai` (line 23) and `ferro-projections` (line 25) unconditionally. `ferro-cli/Cargo.toml` already depends on `ferro-mcp` (line 45).

---

## Shared Patterns

### Tool Return Type: Always `String` (pretty JSON)
**Source:** `ferro-mcp/src/service.rs` — every `#[tool]` method, e.g. lines 388–393
**Apply to:** `ai_scaffold` and `ai_explain` tool methods in `service.rs`
```rust
// Success path — return the payload directly as pretty JSON
serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())

// Error path — encode error in JSON, never panic, never exit
serde_json::to_string_pretty(&serde_json::json!({ "success": false, "error": e }))
    .unwrap_or_else(|_| r#"{"success":false}"#.to_string())
```

### Params Struct Derives
**Source:** `ferro-mcp/src/service.rs:333`
**Apply to:** `AiScaffoldParams`, `AiExplainParams`
```rust
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
```

### `.env` Loading in Tests
**Source:** `ferro-mcp/src/tools/ai.rs:52–55`
**Apply to:** Any test in `ferro-mcp` that calls `AiConfig::from_env()`
```rust
let env_path = project_root.join(".env");
if env_path.exists() {
    let _ = dotenvy::from_path(&env_path);
}
```

### ENV_LOCK for Parallel Test Safety
**Source:** `ferro-cli/src/commands/mod.rs:78`
**Apply to:** All tests in `ferro-mcp` that set `FERRO_AI_MAX_TOKENS_PER_COMMAND` or other env vars
```rust
// Add to ferro-mcp (e.g. in src/lib.rs or a dedicated test_support module):
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// Usage in tests:
let _guard = ENV_LOCK.lock().unwrap();
std::env::set_var("FERRO_AI_MAX_TOKENS_PER_COMMAND", "1234");
```

### AiConfig Error Mapping (no process::exit in core)
**Source:** `ferro-mcp/src/tools/ai.rs:75–87` (test_classifier error handling)
**Apply to:** `scaffold_core`, `call_llm_prose` in the relocated cores
```rust
// Return Err(String), never eprintln! or process::exit:
let client = AiConfig::from_env().map_err(|e| {
    format!("AI provider not configured: {e}. Set FERRO_AI_PROVIDER, FERRO_AI_API_KEY, FERRO_AI_MODEL.")
})?;
```

### `#[cfg(feature = "projections")]` — only in ferro-cli, never in ferro-mcp
**Source:** `ferro-mcp/Cargo.toml:25` (unconditional dep) vs `ferro-cli/Cargo.toml` (optional behind feature)
**Apply to:** All new code in `ferro-mcp` — no feature guards. Guards stay on the CLI thin wrappers.

---

## No Analog Found

All files have analogs. No entries needed here.

---

## Metadata

**Analog search scope:** `ferro-mcp/src/`, `ferro-cli/src/commands/`, `ferro-cli/src/`
**Files read:** 12 source files
**Pattern extraction date:** 2026-06-08
