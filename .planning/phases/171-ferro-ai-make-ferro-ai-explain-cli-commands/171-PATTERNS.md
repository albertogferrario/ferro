# Phase 171: ferro ai:make & ferro ai:explain CLI Commands - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 6 (2 new commands, 1 new emitter module, 1 SDK modification, 1 main.rs registration, 1 lexical filter)
**Analogs found:** 5 / 6 (lexical relevance filter has no direct analog)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-cli/src/commands/ai_make.rs` | command (controller) | request-response + file-I/O | `ferro-cli/src/commands/make_json_view.rs` | role-match (same async→sync bridge, same AiConfig gating, same file-write pattern) |
| `ferro-cli/src/commands/ai_explain.rs` | command (controller) | request-response | `ferro-cli/src/commands/make_json_view.rs` pass-1 plain-text path | role-match |
| ServiceDef → Rust-builder source emitter (fn in `ai_make.rs`) | utility (code emitter) | transform | `ferro-cli/src/commands/make_projection.rs::projection_template` (lines 529–551) | partial-match (static template; emitter is the dynamic equivalent) |
| `ferro-ai/src/complete.rs` (modify) | SDK entry point | request-response | existing `complete::<T>()` in same file (lines 57–81) | exact (refactor-in-place: extract options, delegate) |
| `ferro-cli/src/main.rs` (modify) | CLI registration | — | `#[cfg(feature = "projections")] ProjectionCheck` variant (line 259–265); dispatch arm (lines 632–638) | exact |
| Lexical relevance filter (inline module or `src/relevance.rs`) | utility | transform | `to_snake_case` / `is_valid_identifier` helpers in `make_projection.rs` (lines 568–566) | partial-match (tokenization helpers only; scoring is novel) |

---

## Pattern Assignments

### `ferro-cli/src/commands/ai_make.rs` (command, request-response + file-I/O)

**Primary analog:** `ferro-cli/src/commands/make_json_view.rs`

**Imports pattern** (`make_json_view.rs` lines 7–14):
```rust
use console::style;
use ferro_ai::client::{Message, Role};
use ferro_ai::{AiConfig, CompletionRequest};
use std::fs;
use std::path::Path;
```
For `ai_make.rs`, replace the json-ui imports with:
```rust
use console::style;
use ferro_ai::{AiConfig, CompleteOptions, complete_with};
use ferro_mcp::tools::{
    database_schema, generation_context, list_models, list_projections, list_routes,
};
use ferro_projections::ServiceDef;
use std::fs;
use std::path::Path;
```

**AiConfig fail-fast pattern** (`make_json_view.rs` lines 61–80):
```rust
match AiConfig::from_env() {
    Ok(client) => {
        // proceed
    }
    Err(_) => {
        if description.is_some() {
            eprintln!(
                "{} No AI provider configured. Set FERRO_AI_API_KEY ...",
                style("Info:").yellow().bold(),
            );
        }
        // fallback
    }
}
```
For `ai_make.rs` the adaptation is: no fallback path exists. On `Err(e)` from `AiConfig::from_env()`, exit immediately naming all three env vars (`FERRO_AI_PROVIDER`, `FERRO_AI_API_KEY`, `FERRO_AI_MODEL`):
```rust
let client = match ferro_ai::AiConfig::from_env() {
    Ok(c) => c,
    Err(e) => {
        eprintln!(
            "{} AI provider not configured: {}\n  Set FERRO_AI_PROVIDER, FERRO_AI_API_KEY, and FERRO_AI_MODEL.",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
};
```

**Async→sync bridge pattern** (`make_json_view.rs` lines 126–137):
```rust
let rt = match tokio::runtime::Runtime::new() {
    Ok(r) => r,
    Err(e) => {
        eprintln!(
            "{} Failed to create tokio runtime: {}",
            style("Warning:").yellow().bold(),
            e
        );
        // in ai_make.rs: exit(1), no fallback template
        std::process::exit(1);
    }
};

// Reuse rt for all async calls:
let routes = rt.block_on(list_routes::execute(&project_root));
let schema = rt.block_on(database_schema::execute(&project_root, None));
let service_def = rt.block_on(complete_with::<ServiceDef>(client.as_ref(), &prompt, opts))?;
```
Sync tools (no `rt.block_on` needed):
```rust
let models = list_models::execute(&project_root);
let gen_ctx = generation_context::execute();       // not Result — always succeeds
let projections = list_projections::execute(&project_root, None);
```

**File-duplicate guard + file-write pattern** (`make_projection.rs` lines 408–416, 472–484):
```rust
if projection_file.exists() {
    eprintln!(
        "{} Projection '{}' already exists at {}",
        style("Info:").yellow().bold(),
        file_name,
        projection_file.display()
    );
    std::process::exit(0);
}
// ...
if let Err(e) = fs::write(&projection_file, &content) {
    eprintln!(
        "{} Failed to write projection file: {}",
        style("Error:").red().bold(),
        e
    );
    std::process::exit(1);
}
println!("{} Created {}", style("✓").green(), projection_file.display());
```

**mod.rs registration** (`make_projection.rs` lines 486–498, `update_mod_file` lines 634–669):
```rust
// Call directly — function signature:
fn update_mod_file(mod_file: &Path, file_name: &str) -> Result<(), String>
// Inserts `pub mod {file_name};` after the last existing `pub mod` line,
// or at top (after doc comments) if none exist.
```
`ai_make.rs` should call an internal copy or re-export of this function rather than importing it from `make_projection.rs` (it is currently `fn`, not `pub fn`). Options: (a) promote `update_mod_file` to `pub(crate)` in `make_projection.rs`, or (b) copy the ~35-line function into `ai_make.rs`.

**--dry-run branch** (pattern from `docker_init.rs` / `do_init.rs` — flag already precedented):
```rust
if dry_run {
    let json = serde_json::to_string_pretty(&service_def)
        .expect("ServiceDef is always serializable");
    println!("{json}");
    return;
}
```

---

### `ferro-cli/src/commands/ai_explain.rs` (command, request-response)

**Primary analog:** `ferro-cli/src/commands/make_json_view.rs` (pass-1 plain-text `client.complete` path, lines 139–165)

**Plain-text LLM call pattern** (no schema, `schema: None`) — `make_json_view.rs` lines 141–165:
```rust
let req = CompletionRequest {
    system: Some(system_prompt),
    messages: vec![Message {
        role: Role::User,
        content: user_prompt,
        tool_call_id: None,
    }],
    max_tokens: 2048,  // for ai:explain
    model_override: None,
    schema: None,       // prose output — no structured schema
    tools: None,
    tool_choice: None,
};
let explanation = match rt.block_on(client.complete(req)) {
    Ok(text) => text,
    Err(e) => {
        eprintln!("{} LLM call failed: {}", style("Error:").red().bold(), e);
        std::process::exit(1);
    }
};
println!("{explanation}");
```

**Target resolution order** (novel logic — D-05, corrected per RESEARCH pitfall 7): attempt `inspect_projection` first (service), then `explain_route`, then `explain_model`. Use `--type` override to skip auto-detect:
```rust
// Pseudo-pattern: service → route → model auto-detect
let target_context = if let Some(ref t) = target_type {
    resolve_by_type(rt, &project_root, target, t)
} else {
    resolve_auto(rt, &project_root, target)
};
```

**--dry-run for ai:explain** (prints the assembled prompt, no LLM call):
```rust
if dry_run {
    println!("=== Assembled prompt ===");
    println!("{system_prompt}");
    println!("---");
    println!("{user_prompt}");
    return;
}
```

---

### ServiceDef → Rust-builder source emitter (function(s) in `ai_make.rs`)

**Closest analog:** `ferro-cli/src/commands/make_projection.rs::projection_template` (lines 529–551) — the static empty template this emitter replaces with a fully-populated builder chain.

**Static template to extend** (`make_projection.rs` lines 529–551):
```rust
format!(
    r#"use ferro::{{
    DataType, FieldMeaning, ServiceDef,
}};

/// Build the {display_name} service projection.
pub fn {name}_service() -> ServiceDef {{
    ServiceDef::new("{name}")
        .display_name("{display_name}")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
}}
"#
)
```

**Builder API target** (from `ferro-projections/src/service.rs` lines 82–100 and `action.rs` lines 41–80):
- `ServiceDef::new(name)` — required
- `.display_name(s)` / `.description(s)` — optional
- `.field(name, DataType::X, FieldMeaning::Y)` — required=true, readable+writable
- `.optional_field(name, DataType::X, FieldMeaning::Y)` — required=false
- `.guard(GuardDef::new(name))` — security/precondition guard
- `.action(ActionDef::new(name).display_name(...).precondition(...).effect(...))` — verb
- `.relationship(RelationshipDef::new(name, Cardinality::X).target(...))` — relations
- `.intent_hint(IntentHint { intent: Intent::Browse, weight: 1.0 })` — intent weighting
- `.state_machine(StateMachine { ... })` — optional FSM

**DataType emit map** (from `field.rs` lines 8–21 — serde renames to snake_case so emitter cannot use serde):
```rust
fn emit_data_type(dt: &DataType) -> &'static str {
    match dt {
        DataType::String   => "DataType::String",
        DataType::Integer  => "DataType::Integer",
        DataType::Float    => "DataType::Float",
        DataType::Boolean  => "DataType::Boolean",
        DataType::DateTime => "DataType::DateTime",
        DataType::Date     => "DataType::Date",
        DataType::Json     => "DataType::Json",
        DataType::Binary   => "DataType::Binary",
        DataType::Uuid     => "DataType::Uuid",
        DataType::Enum     => "DataType::Enum",
    }
}
```

**FieldMeaning emit map** (from `field.rs` lines 35–56 — `Custom` variant uses `#[serde(untagged)]`):
```rust
fn emit_field_meaning(m: &FieldMeaning) -> String {
    match m {
        FieldMeaning::Identifier => "FieldMeaning::Identifier".into(),
        FieldMeaning::ForeignKey => "FieldMeaning::ForeignKey".into(),
        FieldMeaning::EntityName => "FieldMeaning::EntityName".into(),
        FieldMeaning::Email      => "FieldMeaning::Email".into(),
        // ... all 18 known variants ...
        FieldMeaning::Custom(s)  => format!(r#"FieldMeaning::Custom("{s}".into())"#),
    }
}
```

**FieldDef builder selection** (from `field.rs` lines 59–72):
```rust
// required=true, is_list=false → .field(...)
// required=false, is_list=false → .optional_field(...)
// is_list=true → .list_field(...) or hand-build a FieldDef
// readable=false → .write_only_field(...)
// writable=false → .read_only_field(...)
```

**Emitter output template shape** (file `src/projections/<name>.rs`):
```rust
use ferro::{
    ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef,
    // add RelationshipDef, StateMachine, Intent, IntentHint as needed
};

/// Build the {DisplayName} service projection.
pub fn {name}_service() -> ServiceDef {
    ServiceDef::new("{name}")
        .display_name("{DisplayName}")
        .description("{description}")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        // ... remaining fields ...
        .guard(GuardDef::new("authenticated"))
        .action(ActionDef::new("create") ...)
}
```

---

### `ferro-ai/src/complete.rs` (modify — add `CompleteOptions` + `complete_with`)

**Analog:** existing `complete::<T>()` function in the same file (lines 57–81).

**Existing function to refactor** (`complete.rs` lines 57–81):
```rust
pub async fn complete<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let raw_schema = serde_json::to_value(schemars::schema_for!(T))
        .map_err(|e| Error::SchemaError(format!("schema_for serialization: {e}")))?;
    let normalized = schema::for_structured_output(raw_schema);

    let request = CompletionRequest {
        system: None,
        messages: vec![Message {
            role: Role::User,
            content: prompt.to_string(),
            tool_call_id: None,
        }],
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

**Adaptation:** extract body into `complete_with`, keep `complete` as delegate:
```rust
pub struct CompleteOptions {
    pub max_tokens: u32,
    pub system: Option<String>,
    pub model_override: Option<String>,
}

impl Default for CompleteOptions {
    fn default() -> Self {
        Self { max_tokens: 4096, system: None, model_override: None }
    }
}

pub async fn complete_with<T>(
    client: &dyn LlmClient,
    prompt: &str,
    opts: CompleteOptions,
) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let raw_schema = serde_json::to_value(schemars::schema_for!(T))
        .map_err(|e| Error::SchemaError(format!("schema_for serialization: {e}")))?;
    let normalized = schema::for_structured_output(raw_schema);

    let request = CompletionRequest {
        system: opts.system,
        messages: vec![Message { role: Role::User, content: prompt.to_string(), tool_call_id: None }],
        max_tokens: opts.max_tokens,
        model_override: opts.model_override,
        schema: Some(normalized),
        tools: None,
        tool_choice: None,
    };

    let text = client.complete(request).await?;
    serde_json::from_str::<T>(&text).map_err(|e| Error::Deserialization(e.to_string()))
}

pub async fn complete<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    complete_with(client, prompt, CompleteOptions::default()).await
}
```

**Re-export in `ferro-ai/src/lib.rs`** — add alongside line 65 (`pub use complete::complete;`):
```rust
pub use complete::{complete, complete_with, CompleteOptions};
```

---

### `ferro-cli/src/main.rs` (modify — clap enum variants + dispatch arms)

**Analog:** `ProjectionCheck` variant (lines 259–265) and its dispatch (lines 635–638).

**Variant registration pattern** (`main.rs` lines 259–265):
```rust
/// Validate service projections for structural issues
#[cfg(feature = "projections")]
#[command(name = "projection:check")]
ProjectionCheck {
    #[arg(long)]
    name: Option<String>,
},
```

**New variants to add** (after `ProjectionCheck` or alongside the `make:*` family):
```rust
/// Generate a service projection from a natural-language description
#[cfg(feature = "projections")]
#[command(name = "ai:make")]
AiMake {
    /// Natural-language description of the service to generate
    description: String,
    /// Print the produced ServiceDef as JSON without writing files
    #[arg(long)]
    dry_run: bool,
},

/// Explain a route, model, or service in projection terms
#[cfg(feature = "projections")]
#[command(name = "ai:explain")]
AiExplain {
    /// Route path, model name, or projection name to explain
    target: String,
    /// Force resolution kind (route | model | service)
    #[arg(long)]
    r#type: Option<String>,
    /// Print the assembled prompt without making the LLM call
    #[arg(long)]
    dry_run: bool,
},
```

**Dispatch arms pattern** (`main.rs` lines 635–638):
```rust
#[cfg(feature = "projections")]
Commands::ProjectionCheck { name } => {
    commands::projection_check::execute(name.as_deref());
}
```

**New dispatch arms** (add at end of `match cli.command`, before closing brace at line 764):
```rust
#[cfg(feature = "projections")]
Commands::AiMake { description, dry_run } => {
    commands::ai_make::run(description, dry_run);
}
#[cfg(feature = "projections")]
Commands::AiExplain { target, r#type, dry_run } => {
    commands::ai_explain::run(target, r#type, dry_run);
}
```

**`mod` declarations in `commands/mod.rs` or inline** — add:
```rust
pub mod ai_make;
pub mod ai_explain;
```

---

### Lexical relevance filter (inline in `ai_make.rs` or `ferro-cli/src/relevance.rs`)

**No direct analog.** Closest helpers: `to_snake_case` / `is_valid_identifier` in `make_projection.rs` lines 568–566 and `make_json_view.rs` lines 286–283 (identifier tokenization building blocks only).

**Identifier tokenizer** (adapt from `to_snake_case` in `make_json_view.rs` lines 286–299):
```rust
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 { result.push('_'); }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
```
The relevance filter reuses this CamelCase-splitting logic for tokenizing item names. The scoring (set intersection count) and budget-gating are novel — no analog in the codebase.

**Scoring algorithm** (novel, from RESEARCH.md Pattern 5):
1. Tokenize description: split on whitespace + CamelCase + `_`, lowercase.
2. For each candidate item: collect tokens from `name + description + field_names`.
3. Score = `|description_tokens ∩ item_tokens|` (set cardinality).
4. Sort descending; keep top-N while cumulative serialized size ≤ `INPUT_BUDGET_CHARS` (8000).
5. `generation_context` always prepended unconditionally (it is sync, always succeeds, small).

---

## Shared Patterns

### AiConfig fail-fast (AI-required commands)
**Source:** `ferro-cli/src/commands/make_json_view.rs` lines 61–80 (but adapted — no fallback)
**Apply to:** `ai_make.rs`, `ai_explain.rs`
```rust
let client = match ferro_ai::AiConfig::from_env() {
    Ok(c) => c,
    Err(e) => {
        eprintln!(
            "{} AI provider required. Configure FERRO_AI_PROVIDER, FERRO_AI_API_KEY, FERRO_AI_MODEL.\n  Error: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
};
```

### Async→sync tokio bridge
**Source:** `ferro-cli/src/commands/make_json_view.rs` lines 126–137
**Apply to:** `ai_make.rs`, `ai_explain.rs`
One `Runtime::new()` per command function, reused for all async calls (`list_routes::execute`, `database_schema::execute`, LLM call). Sync tool fns (`list_models`, `generation_context`, `list_projections`, `inspect_projection`) called directly without `rt.block_on`.

### DB-unavailable graceful degradation
**Source:** RESEARCH.md Pitfall 1 — no existing analog for this pattern in the codebase.
**Apply to:** `ai_make.rs`
```rust
let schema_info = rt.block_on(database_schema::execute(&project_root, None))
    .unwrap_or_default();  // empty SchemaInfo{tables:[]} is valid sparse context
```

### console::style output formatting
**Source:** `ferro-cli/src/commands/make_json_view.rs` lines 41, 67, 88–91; `make_projection.rs` lines 405, 480–481
**Apply to:** All new command files
```rust
println!("{} Created {}", style("✓").green(), file_path.display());
eprintln!("{} {}", style("Error:").red().bold(), message);
eprintln!("{} {}", style("Info:").yellow().bold(), message);
```

### `#[cfg(feature = "projections")]` gating
**Source:** `ferro-cli/src/main.rs` line 259
**Apply to:** Both `AiMake` and `AiExplain` clap variants and their dispatch arms.
The `projections` feature is already in `default` (`Cargo.toml` line 54 confirmed by RESEARCH): no Cargo.toml changes required. The cfg gate ensures clean non-default builds.

---

## No Analog Found

| File / Component | Role | Data Flow | Reason |
|---|---|---|---|
| Lexical relevance scorer | utility | transform | No scoring/ranking logic exists in the repo; tokenization helpers exist but set-intersection scoring is novel |
| DB-unavailable graceful fallback | error handling | — | `database_schema::execute` is always called in a running-app context in existing code; CLI graceful fallback is new |

---

## Metadata

**Analog search scope:** `ferro-cli/src/commands/`, `ferro-ai/src/`, `ferro-projections/src/`, `ferro-cli/src/main.rs`
**Files scanned (read):** 9 source files + 2 planning docs
**Pattern extraction date:** 2026-06-08
