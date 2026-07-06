# Phase 171: ferro ai:make & ferro ai:explain CLI Commands - Research

**Researched:** 2026-06-08
**Domain:** ferro-cli command authoring, ferro-ai structured output, ferro-mcp in-process introspection, ServiceDef source emitter
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Call `ferro_mcp::tools::*::execute` directly in-process. Async fns bridged via `tokio::runtime::Runtime::new().block_on(...)`. Sync fns called directly.
- **D-02:** Deterministic lexical relevance filter (token overlap). `generation_context` always included.
- **D-03:** `ai:make` emits a single Rust builder source file at `src/projections/<snake>.rs` + mod.rs registration. `--dry-run` prints ServiceDef as pretty JSON. ServiceDef → Rust-builder-source emitter must be built from scratch.
- **D-04:** Add `complete_with::<T>(client, prompt, CompleteOptions { max_tokens, system, model_override })` to ferro-ai. Keep `complete::<T>()` as zero-config delegate. `FERRO_AI_MAX_TOKENS_PER_COMMAND` maps to request `max_tokens`.
- **D-05:** `ai:explain <target>` auto-detects kind: route → model → service. Optional `--type route|model|service`. Projection-framed when ServiceDef found; prose fallback via explain_route/explain_model.
- **D-06:** AI-required fail-fast via `AiConfig::from_env()`. `projections` feature is default (already true — see `ferro-cli/Cargo.toml:54`). `--dry-run` on both commands.

### Claude's Discretion
- Exact lexical-relevance scoring formula and top-N cutoff.
- Default `max_tokens` per command (ai:make ~8192, ai:explain ~2048 as starting points).
- Prompt wording / system-prompt structure.
- Whether `complete_with` carries `model_override` in addition to `max_tokens` + `system`.
- CLI flag surface beyond `--dry-run` / `--type`.

### Deferred Ideas (OUT OF SCOPE)
- Embedding-based semantic relevance reranking.
- `ai_scaffold` / `ai_explain` MCP tool wrappers (Phase 172).
- `make:json-view` v2 (Phase 173).
- `temperature` on `CompletionRequest`.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AICLI-01 | `ferro ai:make <description>` → typed `ServiceDef` via live ferro-mcp introspection filtered to relevant items | D-01 (execute fns), D-02 (filter), D-03 (emitter), D-04 (complete_with) |
| AICLI-02 | Uses structured outputs (AISDK-02 ServiceDef-aware path), no ScaffoldPlan intermediary | D-04 (complete_with path), schema/mod.rs ServiceDef-aware normalizer |
| AICLI-03 | `ferro ai:explain <route\|model\|service>` → projection-framed when ServiceDef found; prose fallback otherwise | D-05 (target resolution), derive_intents, explain_route/explain_model |
</phase_requirements>

---

## Summary

Phase 171 ships the two killer CLI commands of the v12.1 milestone. Both commands are CLI-only Rust additions to `ferro-cli/src/commands/`; no new crates, no new framework types. The central new implementation unit is the **ServiceDef → Rust-builder source emitter** (D-03) — a function that walks a live `ServiceDef` value and writes back idiomatic `.field(...).action(...).guard(...)` builder code. Everything else is wiring: the ferro-mcp tool `execute` fns are already typed and public, the Phase 166 `complete::<T>()` path already handles ServiceDef-aware schema normalization automatically (by presence of projection type names in `$defs`), and `make:projection`'s file-writing logic is a direct reuse target.

The async→sync bridge precedent from Phase 170 (`make_json_view.rs:126`) is clear: one `tokio::runtime::Runtime::new()` instance, reused across all async calls in the command, via `rt.block_on(...)`. The two async tools (`list_routes::execute`, `database_schema::execute`) and the LLM call are all bridged this way. Four sync tools (`list_models::execute`, `generation_context::execute`, `list_projections::execute`, `inspect_projection::execute`) require no bridge.

**Key pre-implementation finding:** `ferro-cli/Cargo.toml:54` already lists `projections` in `default = ["projections"]`. D-06 says "make projections a default feature" — this work is already done. The planner does NOT need a Cargo.toml change for the feature gate; it only needs to confirm the `#[cfg(feature = "projections")]` guard on `MakeProjection` and `ProjectionCheck` variants is handled for the new `ai:make`/`ai:explain` enum variants.

`database_schema::execute` makes a live database connection. It will fail if no database is reachable. The command must gracefully handle this — fall back to an empty schema section in the context rather than aborting. The same applies to `list_routes::execute`, which tries the runtime HTTP endpoint first and falls back to static analysis of `src/routes.rs`.

**Primary recommendation:** Build in this order: (1) `complete_with::<T>()` in ferro-ai, (2) lexical relevance filter module, (3) ServiceDef → Rust-builder emitter, (4) `ai:make` command wiring, (5) `ai:explain` command wiring, (6) clap registration + dispatch.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ServiceDef → Rust-builder source emission | ferro-cli command | — | Pure code-generation concern; consumes a ferro-projections type but generates no framework runtime output |
| Lexical relevance filtering | ferro-cli command | — | CLI-specific concern; not reusable across crates yet |
| In-process introspection wiring | ferro-cli command | ferro-mcp (library) | CLI calls mcp tool fns directly |
| `complete_with` options variant | ferro-ai (SDK) | — | SDK surface change; must be re-exported via ferro-ai lib.rs |
| ServiceDef-aware schema normalization | ferro-ai schema module | ferro-projections (types) | Already implemented; activated automatically by `$defs` presence check |
| Projection-framed explain output | ferro-cli command | ferro-projections (derive_intents) | Command assembles the narrative; projection crate provides `derive_intents` |

---

## Standard Stack

### Core (all already in ferro-cli/Cargo.toml)
| Library | Version | Purpose | Note |
|---------|---------|---------|------|
| `ferro-mcp` | workspace 0.2 | In-process introspection source | `path = "../ferro-mcp"` at line 45 |
| `ferro-ai` | workspace 0.2 | `complete_with`, `AiConfig`, `CompletionRequest` | `path = "../ferro-ai"` at line 46 |
| `ferro-projections` | workspace 0.2 | `ServiceDef`, `derive_intents` | `path = "../ferro-projections"` at line 47; feature-gated but already default |
| `tokio` | 1 (full features) | Runtime bridge for async tool fns | Line 34 |
| `clap` | 4 (derive feature) | New subcommand variants | Line 22 |
| `serde_json` | 1 | `to_string_pretty` for --dry-run | Line 43 |
| `console` | 0.15 | Styled output (matches existing commands) | Line 24 |

**No new dependencies needed.** All required crates are already in ferro-cli/Cargo.toml.

### New Items to Add to ferro-ai
| Item | Location | Purpose |
|------|---------|---------|
| `CompleteOptions` struct | `ferro-ai/src/complete.rs` | `max_tokens: u32`, `system: Option<String>`, `model_override: Option<String>` |
| `complete_with::<T>()` | `ferro-ai/src/complete.rs` | Parameterized entry; `complete::<T>()` delegates here with defaults |
| Re-export of `complete_with` | `ferro-ai/src/lib.rs` | Public API surface |

---

## Architecture Patterns

### System Architecture Diagram

```
ferro ai:make <description>
        │
        ▼
  [D-06] AiConfig::from_env() ──Err──► fail-fast (named env vars)
        │Ok
        ▼
  [D-01] In-process MCP calls (sync bridge)
    ├── list_models::execute(&root) → Vec<ModelDetails>       [SYNC]
    ├── generation_context::execute() → GenerationContext     [SYNC]
    ├── list_projections::execute(&root, None) → ProjectionList [SYNC]
    ├── rt.block_on(list_routes::execute(&root)) → RoutesInfo  [ASYNC]
    └── rt.block_on(database_schema::execute(&root, None))      [ASYNC]
            └── Err → empty section (DB not available is non-fatal)
        │
        ▼
  [D-02] Lexical relevance filter
    ├── Tokenize description (split snake_case / CamelCase / words)
    ├── Score each item (name + field names vs description tokens)
    ├── Sort descending, take top-N under token budget
    └── Always include generation_context verbatim
        │
        ▼
  [D-04] Build prompt (system + user context + description)
        │
        ▼
  complete_with::<ServiceDef>(client, prompt, CompleteOptions{
      max_tokens: env("FERRO_AI_MAX_TOKENS_PER_COMMAND").unwrap_or(8192),
      system: Some(system_prompt),
      model_override: None,
  })
    └── internally: schema::for_structured_output(schemars::schema_for!(ServiceDef))
                    └── ServiceDef-aware path activated (FieldMeaning + Intent closed)
        │
        ▼
  --dry-run? → serde_json::to_string_pretty(&service_def) → stdout, exit 0
        │
        ▼
  [D-03] ServiceDef → Rust-builder source emitter
        │
        ▼
  File write: src/projections/<snake>.rs
  mod.rs registration: update_mod_file() (reused from make_projection.rs)
```

```
ferro ai:explain <target>
        │
        ▼
  [D-06] AiConfig::from_env() ──Err──► fail-fast
        │Ok
        ▼
  [D-05] Target resolution (auto-detect or --type override)
    ├── Try route: rt.block_on(explain_route::execute(&root, target))
    ├── Try model: rt.block_on(explain_model::execute(&root, target))
    └── Try service: inspect_projection::execute(&root, target) [SYNC]
        │
        ▼
  ServiceDef found?
    ├── YES → projection-framed path:
    │     ├── derive_intents(&service_def)
    │     ├── Collect FieldMeaning distribution
    │     ├── List ActionDefs + GuardDefs
    │     └── StateMachine transitions if present
    │     → Build LLM prompt with projection vocabulary
    └── NO → prose fallback:
          └── Use explain_route / explain_model result as context
        │
        ▼
  --dry-run? → print assembled prompt → stdout, exit 0
        │
        ▼
  complete_with::<String>(client, prompt, CompleteOptions{
      max_tokens: env("FERRO_AI_MAX_TOKENS_PER_COMMAND").unwrap_or(2048),
      system: Some(explain_system_prompt),
      model_override: None,
  })
  Note: ai:explain returns prose (String), not a typed struct.
  Use plain client.complete(request) or complete_with::<String>.
        │
        ▼
  println!("{}", explanation)
```

### Recommended Project Structure
```
ferro-cli/src/commands/
├── ai_make.rs           # new: ai:make command
├── ai_explain.rs        # new: ai:explain command
└── make_projection.rs   # existing: reuse update_mod_file, projection template helpers

ferro-ai/src/
└── complete.rs          # add complete_with::<T>() + CompleteOptions
```

### Pattern 1: Async→Sync Bridge (from Phase 170)
**Source:** `ferro-cli/src/commands/make_json_view.rs:126`

```rust
// Source: ferro-cli/src/commands/make_json_view.rs:126
let rt = match tokio::runtime::Runtime::new() {
    Ok(r) => r,
    Err(e) => {
        eprintln!("{} Failed to create tokio runtime: {}", style("Warning:").yellow().bold(), e);
        std::process::exit(1);
    }
};

// Reuse the same rt for all async calls:
let routes = rt.block_on(list_routes::execute(&project_root));
let schema = rt.block_on(database_schema::execute(&project_root, None));
let text = rt.block_on(client.complete(request))?;
```

**When to use:** Any ferro-cli command that calls async functions. `main()` is sync; `Runtime::new()` is safe (no existing tokio context).

### Pattern 2: complete_with design (D-04)

```rust
// Source: to be added to ferro-ai/src/complete.rs
pub struct CompleteOptions {
    pub max_tokens: u32,
    pub system: Option<String>,
    pub model_override: Option<String>,
}

impl Default for CompleteOptions {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            system: None,
            model_override: None,
        }
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

// complete<T> delegates to complete_with with defaults:
pub async fn complete<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    complete_with(client, prompt, CompleteOptions::default()).await
}
```

### Pattern 3: clap subcommand registration

**Source:** `ferro-cli/src/main.rs` lines 250–264 and ~507 (dispatch `match cli.command`)

```rust
// In the Commands enum (ferro-cli/src/main.rs):
/// Generate a service projection from a natural-language description (AI-powered)
#[command(name = "ai:make")]
AiMake {
    /// Natural-language description of the service to generate
    description: String,
    /// Print the produced ServiceDef as JSON without writing files
    #[arg(long)]
    dry_run: bool,
},

/// Explain a route, model, or service projection in projection terms (AI-powered)
#[command(name = "ai:explain")]
AiExplain {
    /// Route path, model name, or service/projection name to explain
    target: String,
    /// Force resolution as a specific kind (route, model, or service)
    #[arg(long)]
    r#type: Option<String>,
    /// Print the assembled context prompt without making the LLM call
    #[arg(long)]
    dry_run: bool,
},
```

```rust
// In match cli.command (ferro-cli/src/main.rs ~line 510):
Commands::AiMake { description, dry_run } => {
    commands::ai_make::run(description, dry_run);
}
Commands::AiExplain { target, r#type, dry_run } => {
    commands::ai_explain::run(target, r#type, dry_run);
}
```

Both commands must appear WITHOUT `#[cfg(feature = "projections")]` since the feature is already default and these commands require it unconditionally. If built without it, they should fail at `AiConfig::from_env()` or type-checking; the cleaner approach is to gate them with `#[cfg(feature = "projections")]` just like `ProjectionCheck` (main.rs:259) — they will be absent in non-default builds but present in all normal builds.

### Pattern 4: ServiceDef → Rust-builder source emitter (D-03 — new)

This is the central new component. It walks a `ServiceDef` value and emits idiomatic Rust source. The output must match the convention that `list_projections::execute` scans for: `pub fn <name>_service() -> ServiceDef { ... }`.

```rust
// To be built in ferro-cli/src/commands/ai_make.rs

fn emit_service_def_source(service: &ServiceDef) -> String {
    let name = &service.name;
    let fn_name = format!("{name}_service");
    // Build chain: ServiceDef::new → .display_name → .description → fields → ...
    let mut lines = vec![
        format!(r#"    ServiceDef::new("{name}")"#),
    ];
    if let Some(ref dn) = service.display_name {
        lines.push(format!(r#"        .display_name("{dn}")"#));
    }
    if let Some(ref desc) = service.description {
        let escaped = desc.replace('"', r#"\""#);
        lines.push(format!(r#"        .description("{escaped}")"#));
    }
    for field in &service.fields {
        let builder = match (field.readable, field.writable, field.required, field.is_list) {
            (true, false, _, false) => "read_only_field",
            (false, true, _, false) => "write_only_field",
            (_, _, false, false) => "optional_field",
            (_, _, true, true) => "list_field",
            _ => "field",
        };
        let dt = format!("DataType::{:?}", field.data_type); // needs proper mapping
        let meaning = emit_field_meaning(&field.meaning);
        lines.push(format!(r#"        .{builder}("{}", {dt}, {meaning})"#, field.name));
    }
    for guard in &service.guards {
        lines.push(format!(r#"        .guard(GuardDef::new("{}"))"#, guard.name));
    }
    for action in &service.actions {
        lines.push(emit_action_def(action));
    }
    for rel in &service.relationships {
        lines.push(emit_relationship_def(rel));
    }
    for hint in &service.intent_hints {
        lines.push(emit_intent_hint(hint));
    }
    if let Some(ref sm) = service.state_machine {
        lines.push(emit_state_machine(sm));
    }
    // ... format into source file
}
```

Key mapping needed:
- `DataType::String` → `"DataType::String"` (use `serde_json::to_string(&dt)` then capitalize)
- `FieldMeaning::Custom("x")` → `FieldMeaning::Custom("x".into())`
- `FieldMeaning::Money` → `"FieldMeaning::Money"` (use `serde_json::to_string` → strip quotes → to_pascal_case)

### Pattern 5: Lexical relevance filter (D-02)

**Algorithm (deterministic, unit-testable):**

1. Tokenize description: split on whitespace, then split each token on `_` and capital-letter transitions (CamelCase → tokens), lowercase all.
2. For each candidate item (route, model, existing projection): collect tokens from `name + description + field_names`.
3. Compute score = `|description_tokens ∩ item_tokens|` (set intersection count). Tie-break by item type (projections > models > routes > schema tables).
4. Sort descending by score, keep top-N items where cumulative serialized size stays under `INPUT_BUDGET_CHARS` (e.g. 8000 chars — leaves room for system prompt + response).
5. `generation_context` is serialized and prepended unconditionally (it is small and fixed).

**Tokenization of identifiers:**

```rust
fn tokenize_identifier(s: &str) -> Vec<String> {
    // Split on underscores, then split CamelCase transitions
    let snake_tokens: Vec<&str> = s.split('_').collect();
    let mut tokens = Vec::new();
    for part in snake_tokens {
        // Split "OrderItem" → ["order", "item"]
        let mut cur = String::new();
        for ch in part.chars() {
            if ch.is_uppercase() && !cur.is_empty() {
                tokens.push(cur.to_lowercase());
                cur = String::new();
            }
            cur.push(ch);
        }
        if !cur.is_empty() {
            tokens.push(cur.to_lowercase());
        }
    }
    tokens
}
```

### Anti-Patterns to Avoid

- **Calling `ferro mcp` as a subprocess.** Use `ferro_mcp::tools::*::execute` directly (D-01). A subprocess requires the app to be running, adds overhead, and is architecturally wrong.
- **Calling `complete::<ServiceDef>()` without system prompt or max_tokens override.** Use `complete_with` (D-04). The base `complete` hardcodes 4096 tokens which may be too small for a context-rich ServiceDef prompt.
- **Aborting on DB unavailable.** `database_schema::execute` will return `Err` if the database is unreachable. Handle gracefully — emit an empty schema section in the context rather than `std::process::exit(1)`.
- **Panicking on runtime failure.** Match `tokio::runtime::Runtime::new()` and exit with a clear error (see Phase 170 pattern).
- **Writing the emitter output without calling `service.validate()`.** The planner should include a validate step: if `service_def.validate()` returns `Err`, print the error and abort. `Ok(warnings)` should print warnings but proceed.
- **`ai:explain` defaulting to prose.** Per REQUIREMENTS.md: projection-framed is the default; prose is the fallback only when no ServiceDef is found.
- **Path traversal via `description`.** See Security Domain section.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ServiceDef JSON schema | Manual JSON | `schemars::schema_for!(ServiceDef)` | Already derives JsonSchema |
| Schema normalization for LLM | Manual $ref resolution | `ferro_ai::schema::for_structured_output()` | Already handles ServiceDef-aware closing |
| Intent derivation | Custom intent logic | `ferro_projections::derive_intents(&service)` | 5-analyzer system already built |
| File-safe snake_case names | Custom sanitizer | Reuse `to_snake_case` from `make_projection.rs` | Handles identifier validation |
| mod.rs registration | Custom append | Reuse `update_mod_file` from `make_projection.rs:634` | Handles existing entries, line ordering |
| Async runtime | `async fn main` | `tokio::runtime::Runtime::new().block_on(...)` | ferro-cli main() is sync by design |
| env-driven LLM client | Direct env reads | `ferro_ai::AiConfig::from_env()` | Unified provider abstraction |

---

## Common Pitfalls

### Pitfall 1: database_schema is a network call
**What goes wrong:** `database_schema::execute` opens a real database connection (`ferro-mcp/src/tools/database_schema.rs:28-44`). In a project without `.env` or with a stopped database, it returns `McpError`. If the command panics or exits on this error, `ai:make` becomes unusable outside a running project.
**Why it happens:** The mcp tool is designed for the iron path (running app); the CLI must be usable in planning contexts.
**How to avoid:** Wrap in `match` or `.unwrap_or_default()`. An empty `SchemaInfo { tables: vec![] }` is a valid (if sparse) context contribution.
**Warning signs:** `Failed to connect` in the error message from `database_schema::execute`.

### Pitfall 2: list_routes tries HTTP first
**What goes wrong:** `list_routes::execute` at line 97 tries `http://localhost:8080` first. In a non-running project this will time out (reqwest default ~30s). The command hangs.
**Why it happens:** The tool prefers runtime routes. For `ai:make` the app is likely not running.
**How to avoid:** The tool already has a static-analysis fallback (line 107) that reads `src/routes.rs`. The timeout comes from the HTTP client. Check whether ferro-mcp's `fetch_runtime_routes` uses a short timeout — if it does, this is acceptable. If not, consider calling the static-analysis path directly or accepting the short delay.
**Actual code:** `list_routes.rs:97` — uses `fetch_runtime_routes` which calls reqwest. Check if there is a timeout set.

### Pitfall 3: ServiceDef emitter — FieldMeaning::Custom serialization
**What goes wrong:** `FieldMeaning::Custom("sku")` must emit `FieldMeaning::Custom("sku".into())` in the generated Rust code. If the emitter uses `serde_json::to_string` naively, it gets `"sku"` (bare string) which is valid JSON but needs mapping to the `Custom("sku".into())` Rust syntax.
**Why it happens:** The `Custom` variant has `#[serde(untagged)]` so it serializes as a bare string, indistinguishable from known variants at the JSON level. The emitter must check for unknown variant strings explicitly.
**How to avoid:** In the emitter, check if the `FieldMeaning` matches any known variant first; if none match, emit `FieldMeaning::Custom("{value}".into())`.

### Pitfall 4: Duplicate mod.rs entry if projection already exists
**What goes wrong:** `update_mod_file` in `make_projection.rs:418-429` exits early if the file already exists or if the mod entry already exists. This behavior should be preserved in `ai:make` — if `src/projections/<name>.rs` already exists, ask the user to delete it or use a different name.
**Why it happens:** Re-running `ai:make user` would overwrite an existing hand-written projection.
**How to avoid:** Check `projection_file.exists()` before proceeding. Exit with a clear message (same pattern as `make_projection.rs:408-416`).

### Pitfall 5: `#[cfg(feature = "projections")]` guard on new variants
**What goes wrong:** `ProjectionCheck` in main.rs line 259 is guarded with `#[cfg(feature = "projections")]`. If `AiMake` and `AiExplain` are NOT guarded and the feature is disabled, they will fail to compile because they use `ServiceDef` types that are unavailable.
**Why it happens:** The `ferro-projections` crate is optional (even though it is in `default`).
**How to avoid:** Gate `AiMake` and `AiExplain` with `#[cfg(feature = "projections")]` — same as `ProjectionCheck`. Since `projections` is in `default`, they will be present in all normal builds.

### Pitfall 6: ServiceDef emitter — DataType serializes in snake_case
**What goes wrong:** `DataType` has `#[serde(rename_all = "snake_case")]`. `serde_json::to_string(&DataType::DateTime)` → `"date_time"`. But in Rust code we need `DataType::DateTime`. The emitter cannot use serde for variant names in code generation.
**How to avoid:** Maintain a direct match arm for DataType variants → their Rust identifier strings:
```rust
fn emit_data_type(dt: &DataType) -> &'static str {
    match dt {
        DataType::String => "DataType::String",
        DataType::Integer => "DataType::Integer",
        DataType::Float => "DataType::Float",
        DataType::Boolean => "DataType::Boolean",
        DataType::DateTime => "DataType::DateTime",
        DataType::Date => "DataType::Date",
        DataType::Json => "DataType::Json",
        DataType::Binary => "DataType::Binary",
        DataType::Uuid => "DataType::Uuid",
        DataType::Enum => "DataType::Enum",
    }
}
```

### Pitfall 7: `ai:explain` on a model that also has a projection
**What goes wrong:** The auto-detect order is route → model → service. If a user runs `ferro ai:explain user` and there is both a `User` model and a `user_service` projection, the model explanation fires first (not the projection-framed one).
**Why it happens:** The resolution order puts model before service.
**How to avoid:** Flip the order to: **service → route → model** for `ai:explain`. The projection-framed explanation is the primary goal (SC#4). Only fall back to model/route when no ServiceDef is found.

---

## Code Examples

### ferro-mcp execute signatures (verified from source)

```rust
// Source: ferro-mcp/src/tools/list_routes.rs:95
pub async fn execute(project_root: &Path) -> Result<RoutesInfo>
// Returns: RoutesInfo { routes: Vec<RouteInfo>, source: RouteSource }
// RouteInfo: { method: String, path: String, handler: String, name: Option<String>, middleware: Vec<String> }

// Source: ferro-mcp/src/tools/list_models.rs:165
pub fn execute(project_root: &Path) -> Result<Vec<ModelDetails>>
// ModelDetails: { name: String, table: Option<String>, path: String, fields: Vec<FieldInfo> }
// FieldInfo: { name: String, field_type: String, is_primary_key: bool, is_nullable: bool }

// Source: ferro-mcp/src/tools/database_schema.rs:28
pub async fn execute(project_root: &Path, table_filter: Option<&str>) -> Result<SchemaInfo>
// SchemaInfo: { tables: Vec<TableInfo> }
// TableInfo: { name: String, columns: Vec<ColumnInfo> }
// ColumnInfo: { name: String, data_type: String, nullable: bool, primary_key: bool, default_value: Option<String> }

// Source: ferro-mcp/src/tools/generation_context.rs:59
pub fn execute() -> GenerationContext
// (not Result — always succeeds, returns static content)

// Source: ferro-mcp/src/tools/list_projections.rs:31
pub fn execute(project_root: &Path, filter: Option<&str>) -> ProjectionList
// (not Result)
// ProjectionList: { projections: Vec<ProjectionInfo>, total: usize }

// Source: ferro-mcp/src/tools/inspect_projection.rs:48
pub fn execute(project_root: &Path, name: &str) -> InspectResult
// InspectResult: Found(ProjectionDetail) | NotFound(ProjectionNotFound)
// ProjectionDetail has parsed fields/actions/relationships from source text — not typed ServiceDef

// Source: ferro-mcp/src/tools/explain_route.rs:35
pub async fn execute(project_root: &Path, route_path: &str) -> Result<RouteExplanation>

// Source: ferro-mcp/src/tools/explain_model.rs:50
pub async fn execute(project_root: &Path, model_name: &str) -> Result<ModelExplanation>
```

**Critical note on `inspect_projection`:** This returns `ProjectionDetail` with string-parsed fields (names/types/meanings as strings), NOT a typed `ServiceDef`. For `ai:explain` projection framing, the command needs to actually **execute the Rust function** defined in `src/projections/<name>.rs` — but it cannot do so from another binary. The practical approach is to use the `ProjectionDetail` strings from `inspect_projection` to reconstruct the structural vocabulary for the explain prompt, not to call `derive_intents` at CLI time. Alternatively, if the project compiles, the projection function could be called. The simpler path: use `inspect_projection` output (field meanings, actions, relationships, intent hints, state machine presence) as the projection context for the LLM prompt rather than calling `derive_intents` live. The LLM receives the projection vocabulary and constructs the framed explanation.

### AiConfig::from_env signature

```rust
// Source: ferro-ai/src/config.rs:44
pub fn from_env() -> Result<Box<dyn LlmClient>, Error>
// Err(Error::Config(_)) on missing key or unknown provider
// Env vars: FERRO_AI_PROVIDER (default "anthropic"), FERRO_AI_MODEL, FERRO_AI_API_KEY, FERRO_AI_BASE_URL
```

### make_projection file-writing reuse

```rust
// Source: ferro-cli/src/commands/make_projection.rs:379–506

// Key logic to reuse:
// 1. to_snake_case(name) → file_name
// 2. is_valid_identifier(&file_name) → guard
// 3. let projections_dir = Path::new("src/projections");
// 4. fs::create_dir_all(projections_dir) if not exists
// 5. Check projection_file.exists() → exit if duplicate
// 6. fs::write(&projection_file, &content)
// 7. update_mod_file(&mod_file, &file_name) OR create new mod.rs

// update_mod_file (line 634): appends `pub mod {file_name};` after last pub mod declaration
// or at top after doc comments if none exist
```

### Cargo.toml feature gate (already done)

```toml
# Source: ferro-cli/Cargo.toml:53-55
[features]
default = ["projections"]
projections = ["dep:ferro-projections"]
```

`projections` is ALREADY in `default`. D-06 says to make it default — this is already the state. No Cargo.toml change needed.

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| `complete::<T>()` hardcodes max_tokens=4096, system=None | `complete_with::<T>(opts)` adds configurable max_tokens, system, model_override | Cost guard enabled; system prompt supported for large contexts |
| make_projection: empty ServiceDef::new() scaffold | ai:make: full populated ServiceDef from LLM | AI-native projection creation |
| explain_route/explain_model: static inference (infer_purpose, infer_business_context) | ai:explain: LLM-generated from actual introspection + projection vocabulary | Semantically richer explanations |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `fetch_runtime_routes` in list_routes.rs uses a short HTTP timeout (under 5s), making the runtime-first strategy acceptable in CLI context | Pitfall 2 | If timeout is 30s, `ai:make` hangs for 30s on startup in non-running projects |
| A2 | `ai:explain` with a plain `String` T for `complete_with` will route through generic normalization (no ServiceDef-aware path), which is correct since the output is prose | Architecture Patterns | If the schema normalizer mishandles `String` T, the LLM call may fail |

**If A1 is wrong:** Planner should add a task to call `parse_routes_from_files` directly instead of going through the runtime-try-first path, or to check for a configurable timeout in the mcp tool.

---

## Open Questions

1. **`fetch_runtime_routes` timeout**
   - What we know: `list_routes::execute` tries `http://localhost:8080` first.
   - What's unclear: The timeout duration on the reqwest call inside `fetch_runtime_routes` — not read in this research session.
   - Recommendation: Planner should add a task to verify the timeout in `ferro-mcp/src/tools/list_routes.rs` (the `fetch_runtime_routes` function not shown above) and consider bypassing the runtime-first path in CLI context if the timeout is long.

2. **`inspect_projection` returns parsed strings, not typed ServiceDef**
   - What we know: `InspectResult::Found(ProjectionDetail)` has string fields (`meaning: String`, `relationships: Vec<String>`, `actions: Vec<String>`, `intent_hints: Vec<String>`). There is no deserialized `ServiceDef` available from the mcp tool.
   - What's unclear: Whether the planner intended `derive_intents` to be called live from CLI (requires executing the Rust function — impossible) or whether the explain framing should be reconstructed from the string-parsed `ProjectionDetail`.
   - Recommendation: Use `ProjectionDetail` strings as the vocabulary input to the LLM prompt for `ai:explain`. The LLM constructs the projection-framed explanation from this structured vocabulary. This is consistent with SC#6 ("references only what introspection reports").

3. **`ai:explain` return type for `complete_with`**
   - What we know: `complete_with::<T>()` requires `T: JsonSchema + DeserializeOwned`. `String` implements `DeserializeOwned` but `schemars::schema_for!(String)` generates a schema that will constrain the output to a JSON string — the LLM response will need to be quoted JSON.
   - Recommendation: For `ai:explain`, build the `CompletionRequest` manually (as `make_json_view.rs` does for its pass-1 plain-text call) without a `schema` field, calling `client.complete(request)` directly. This avoids the type-mismatch. The output is free-form prose.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `FERRO_AI_API_KEY` env var | Both commands | User-configured | — | Fail-fast with named env var message |
| Database (`.env` DB_URL) | `database_schema::execute` | Optional | — | Empty schema section in context |
| `src/routes.rs` | `list_routes::execute` (static fallback) | Project-dependent | — | Empty routes section |
| `src/projections/` | `list_projections::execute` | Project-dependent | — | Returns empty list (valid) |

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` |
| Config file | None (workspace-level) |
| Quick run command | `cargo test -p ferro-cli -p ferro-ai --lib` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AICLI-01 | Lexical relevance filter scores and selects items | unit | `cargo test -p ferro-cli relevance` | ❌ Wave 0 |
| AICLI-01 | Context includes generation_context always | unit | `cargo test -p ferro-cli context_always_includes_generation` | ❌ Wave 0 |
| AICLI-02 | `complete_with::<ServiceDef>()` uses ServiceDef-aware normalizer | unit | `cargo test -p ferro-ai complete_with_servicedef_schema` | ❌ Wave 0 |
| AICLI-02 | `complete::<T>()` delegates to `complete_with` with defaults | unit | `cargo test -p ferro-ai complete_delegates_to_complete_with` | ❌ Wave 0 |
| AICLI-03 | `ai:explain` resolution order: service before route/model | unit | `cargo test -p ferro-cli explain_resolution_order` | ❌ Wave 0 |
| D-03 | ServiceDef emitter round-trip: `emit_service_def_source` output parses back to equivalent ServiceDef | unit | `cargo test -p ferro-cli emitter_round_trip` | ❌ Wave 0 |
| D-03 | `--dry-run` prints pretty JSON to stdout, writes nothing | integration | `cargo test -p ferro-cli dry_run_no_file_write` | ❌ Wave 0 |
| D-04 | `FERRO_AI_MAX_TOKENS_PER_COMMAND` env controls request max_tokens | unit | `cargo test -p ferro-cli max_tokens_env_applied` | ❌ Wave 0 |
| D-06 | Missing `FERRO_AI_API_KEY` exits with clear message naming the env var | unit | `cargo test -p ferro-cli ai_make_requires_ai_config` | ❌ Wave 0 |
| D-06 | `ai:explain --dry-run` prints assembled prompt without LLM call | unit | `cargo test -p ferro-cli explain_dry_run_no_llm_call` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-cli -p ferro-ai --lib`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `ferro-cli/src/commands/ai_make.rs` — new file, covers AICLI-01/02, D-03, D-04
- [ ] `ferro-cli/src/commands/ai_explain.rs` — new file, covers AICLI-03, D-05
- [ ] Tests in `ferro-cli/src/commands/ai_make.rs` — emitter round-trip, dry-run, max_tokens
- [ ] Tests in `ferro-ai/src/complete.rs` — complete_with delegate, CompleteOptions defaults
- [ ] `ferro-cli/src/relevance.rs` (or inline in ai_make.rs) — lexical filter unit tests

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Sanitize description → file/module name (reuse `to_snake_case` + `is_valid_identifier` from make_projection.rs) |
| V2 Authentication | no | No user auth in CLI commands |
| V3 Session Management | no | Stateless CLI |
| V6 Cryptography | no | No crypto operations |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal: `ai:make "../../etc/passwd"` → snake_case → file write outside `src/projections/` | Tampering | `to_snake_case` strips non-alphanumeric; `is_valid_identifier` validates; always join with a fixed `src/projections/` base path (never accept absolute paths from user input) |
| Prompt injection: description contains `\nIgnore previous instructions\n` | Elevation of privilege | Wrap description in a delimited block in the prompt (e.g., `<description>...</description>`); the structured output schema (ServiceDef) constrains the output format regardless |
| LLM-generated Rust written to source tree | Tampering | Source is written only to `src/projections/<name>.rs`; `--dry-run` preview before commit is the standard workflow; the file content is a ServiceDef builder, not arbitrary Rust |
| Token exhaustion: very long description triggers large LLM context | Denial of Service | `FERRO_AI_MAX_TOKENS_PER_COMMAND` caps response tokens; input context is bounded by the relevance filter budget (`INPUT_BUDGET_CHARS`) |
| Module name injection: name like `mod foo; use std::fs` | Tampering | `is_valid_identifier` validation already used in `make_projection.rs:382`; reuse exactly |

**Existing sanitization helpers (verified):**
- `to_snake_case(name)` in `make_projection.rs` — converts to lowercase snake_case
- `is_valid_identifier(name)` in `make_projection.rs` — rejects names containing path separators or non-identifier chars
- Both can be called from `ai_make.rs` as module-private re-implementations or by extracting them to a shared `ferro-cli/src/naming.rs` module

---

## Sources

### Primary (HIGH confidence)
- `ferro-mcp/src/tools/list_routes.rs` — execute signature, return types, async behavior
- `ferro-mcp/src/tools/list_models.rs` — execute signature, sync, return types
- `ferro-mcp/src/tools/database_schema.rs` — execute signature, async, live DB connection
- `ferro-mcp/src/tools/generation_context.rs` — execute signature, sync, no Result
- `ferro-mcp/src/tools/list_projections.rs` — execute signature, sync, no Result
- `ferro-mcp/src/tools/inspect_projection.rs` — InspectResult shape, string-parsed fields
- `ferro-mcp/src/tools/explain_route.rs` — async, Result<RouteExplanation>
- `ferro-mcp/src/tools/explain_model.rs` — async, Result<ModelExplanation>
- `ferro-projections/src/service.rs` — ServiceDef full shape, all builder methods
- `ferro-projections/src/field.rs` — FieldDef, DataType, FieldMeaning (18 known variants + Custom)
- `ferro-projections/src/action.rs` — ActionDef, GuardDef, InputDef
- `ferro-projections/src/relationship.rs` — RelationshipDef, Cardinality, NavigationHint
- `ferro-projections/src/state.rs` — StateMachine, StateDef, Transition, Warning
- `ferro-projections/src/intent.rs` — Intent (7 known + Custom), IntentHint, IntentScore
- `ferro-projections/src/derive.rs` — derive_intents signature
- `ferro-ai/src/complete.rs` — complete::<T>() implementation, hardcoded max_tokens=4096
- `ferro-ai/src/schema/mod.rs` — for_structured_output, ServiceDef-aware path trigger
- `ferro-ai/src/client/mod.rs` — CompletionRequest shape (system, messages, max_tokens, model_override, schema, tools, tool_choice)
- `ferro-ai/src/config.rs` — AiConfig::from_env() signature, env vars, Error::Config
- `ferro-cli/src/main.rs` — Commands enum structure, dispatch match, sync main()
- `ferro-cli/src/commands/make_projection.rs` — projection_template, update_mod_file, file-write flow
- `ferro-cli/src/commands/make_json_view.rs` — Runtime::new().block_on pattern (Phase 170 bridge)
- `ferro-cli/Cargo.toml` — existing deps, `default = ["projections"]` already set

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against actual Cargo.toml
- Architecture: HIGH — verified against source code; all function signatures cited with file:line
- Pitfalls: HIGH for code-derived pitfalls; MEDIUM for A1 (runtime timeout, not verified)

**Research date:** 2026-06-08
**Valid until:** 2026-07-08 (stable codebase; ferry-ai SDK changes would invalidate)
