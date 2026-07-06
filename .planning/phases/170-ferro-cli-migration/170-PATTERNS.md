# Phase 170: ferro-cli Migration - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 4 (3 modified, 1 deleted)
**Analogs found:** 4 / 4

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-cli/src/commands/make_json_view.rs` | command (transport swap) | request-response | `ferro-cli/src/commands/mcp.rs` (runtime pattern) + self (control flow unchanged) | exact for bridge; self for logic |
| `ferro-cli/src/ai.rs` | DELETE — blocking HTTP client | — | n/a | — |
| `ferro-cli/src/lib.rs` | config | — | self (one-line deletion) | exact |
| `ferro-cli/Cargo.toml` | config | — | self (add one dep) | exact |

---

## Pattern Assignments

### `ferro-cli/src/commands/make_json_view.rs` (transport swap)

This is the load-bearing change. Three sub-patterns apply.

---

#### Sub-pattern A: Async→Sync Bridge

**Best analog in ferro-cli:** `ferro-cli/src/commands/mcp.rs` lines 31–46

`mcp.rs` is the closest match: it constructs a `Runtime` with error handling (not `.unwrap()`) and calls `runtime.block_on(...)` on a single async fn. `make_json_view.rs` needs the same shape but calls `block_on` twice (Pass 1, Pass 2) on the same runtime instance.

```rust
// ferro-cli/src/commands/mcp.rs lines 31-46
// COPY THIS RUNTIME CONSTRUCTION PATTERN — error-handled, not .unwrap()
let runtime = match tokio::runtime::Runtime::new() {
    Ok(rt) => rt,
    Err(e) => {
        eprintln!(
            "{} Failed to create tokio runtime: {}",
            style("[ERROR]").red().bold(),
            e
        );
        std::process::exit(1);
    }
};

if let Err(e) = runtime.block_on(ferro_mcp::run()) { ... }
```

For `generate_with_ai`, adapt the fallback-return variant (not `process::exit`) and reuse the runtime across two `block_on` calls:

```rust
// Adaptation for generate_with_ai (construct once, call block_on twice):
let rt = match tokio::runtime::Runtime::new() {
    Ok(r) => r,
    Err(e) => {
        eprintln!("{} Failed to create tokio runtime: {}", style("Warning:").yellow().bold(), e);
        eprintln!("{}", style("Falling back to static template.").dim());
        return templates::json_view_template(file_name, title, layout_name);
    }
};

// Pass 1
let pass1_result = match rt.block_on(client.complete(req1)) { ... };
// Pass 2 — same rt
let json_str = match rt.block_on(client.complete(req2)) { ... };
```

Secondary analog: `ferro-cli/src/commands/auth_link.rs` lines 32–34 shows the simpler `.unwrap()` form; `mcp.rs` is preferred because it matches the error-handling style used elsewhere in the same binary.

---

#### Sub-pattern B: AiConfig::from_env() Provider Gating

**Analog:** `ferro-ai/src/config.rs` lines 77–137 (test call sites show usage shape)

`AiConfig::from_env()` returns `Result<Box<dyn LlmClient>, Error>`. Match on it directly — `Ok(client)` enters the AI path, `Err(_)` falls back to the static template. The old gate on `std::env::var("ANTHROPIC_API_KEY")` (currently at `make_json_view.rs` lines 59–78) is replaced wholesale.

```rust
// ferro-ai/src/config.rs lines 44-74 — AiConfig::from_env() signature
pub fn from_env() -> Result<Box<dyn LlmClient>, Error>
// Returns Err(Error::Config("FERRO_AI_API_KEY not set")) when key absent
// Returns Err(Error::Config("unknown FERRO_AI_PROVIDER: '...'")) for unknown provider
// For "anthropic", ANTHROPIC_API_KEY is accepted as fallback (backward compat)
```

Replacement gate in `make_json_view::run()` (replaces lines 59–78 of current `make_json_view.rs`):

```rust
// New gate — replaces the std::env::var("ANTHROPIC_API_KEY") block
let client = match ferro_ai::AiConfig::from_env() {
    Ok(c) => c,
    Err(_) => {
        if description.is_some() {
            eprintln!(
                "{} No AI provider configured. Set FERRO_AI_API_KEY (and optionally \
                 FERRO_AI_PROVIDER/FERRO_AI_MODEL), or use --no-ai to suppress this message.",
                style("Info:").yellow().bold(),
            );
        }
        return templates::json_view_template(&file_name, &title, layout_name);
        // NOTE: caller writes the file; the gate returns early before generate_with_ai
    }
};
```

---

#### Sub-pattern C: CompletionRequest struct literal + client.complete()

**Analog:** `ferro-ai/src/complete.rs` lines 65–79 — shows the canonical `CompletionRequest` literal with all 7 fields.

```rust
// ferro-ai/src/complete.rs lines 65-79 — canonical CompletionRequest literal
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
```

Pass 1 variant (plain text, no schema, max_tokens=1024):

```rust
// ferro-ai/src/client/mod.rs lines 119-144 — CompletionRequest field definitions
let req1 = ferro_ai::CompletionRequest {
    system: Some(sys1),
    messages: vec![ferro_ai::Message {
        role: ferro_ai::Role::User,
        content: usr1,
        tool_call_id: None,
    }],
    max_tokens: 1024,
    model_override: None,
    schema: None,          // No schema — Pass 1 is plain text
    tools: None,
    tool_choice: None,
};
```

Pass 2 variant (catalog schema, max_tokens=4096):

```rust
let req2 = ferro_ai::CompletionRequest {
    system: Some(sys2),
    messages: vec![ferro_ai::Message {
        role: ferro_ai::Role::User,
        content: usr2,
        tool_call_id: None,
    }],
    max_tokens: 4096,
    model_override: None,
    // schema field docs (client/mod.rs line 134): "Passed through to the provider as-is"
    // The catalog schema is the validation source of truth — do NOT use schemars here
    schema: Some(ferro_json_ui::global_catalog().json_schema().clone()),
    tools: None,
    tool_choice: None,
};
```

**Import paths confirmed from `ferro-ai/src/lib.rs` lines 61-74:**

```rust
use ferro_ai::{AiConfig, CompletionRequest, LlmClient};
// ferro_ai::client re-exports: Message, Role are NOT re-exported at crate root
// They live at ferro_ai::client::{Message, Role} — verify with:
// ferro-ai/src/lib.rs line 61: pub use client::{AnthropicClient, CompletionRequest, ...}
// Message and Role are NOT in that pub use list — access as ferro_ai::client::Message, ferro_ai::client::Role
// OR add them to the import: check lib.rs line 61-64 carefully
```

Note: `Message` and `Role` are NOT re-exported at the `ferro_ai` crate root (they are absent from `ferro-ai/src/lib.rs` lines 61–64's `pub use client::{...}` list). Use `ferro_ai::client::{Message, Role}` or add them to the existing re-export when implementing. The struct literal in `complete.rs` line 66 uses `use crate::client::{CompletionRequest, LlmClient, Message, Role}` (internal path) — the external consumer must use `ferro_ai::client::Message` and `ferro_ai::client::Role`.

---

#### Sub-pattern D: Error handling — fallback-return shape

**Analog:** `ferro-cli/src/commands/make_json_view.rs` lines 119–174 (current file — the existing fallback pattern is unchanged)

The existing `generate_with_ai` error handling shape (yellow warning + "Falling back to static template." dim + return static template) is the correct pattern. Copy it verbatim for the two new `block_on` call sites:

```rust
// Current make_json_view.rs lines 120-128 — this shape is preserved for both passes
Err(e) => {
    eprintln!(
        "{} AI Pass 1 failed: {}",
        style("Warning:").yellow().bold(),
        e
    );
    eprintln!("{}", style("Falling back to static template.").dim());
    return templates::json_view_template(file_name, title, layout_name);
}
```

The validation block (lines 148–174) is unchanged — `Spec::from_json` + `global_catalog().validate` stays identical.

---

### `ferro-cli/src/ai.rs` — DELETE

No pattern needed. The transport functions (`call_anthropic`, `call_anthropic_plain`, `call_anthropic_structured`, `generate_json_view`) are deleted entirely. The four prompt/scan helpers are relocated:

| Helper | Current location | Move to |
|--------|-----------------|---------|
| `build_json_view_pass1` | `ai.rs` lines 174–193 | `make_json_view.rs` (or `make_json_view_prompts.rs`) |
| `build_json_view_pass2` | `ai.rs` lines 199–213 | same |
| `scan_models` | `ai.rs` lines 302–371 | same |
| `scan_routes` | `ai.rs` lines 374–411 | same |

These helpers have zero transport coupling — copy them verbatim. `scan_models` uses `regex::Regex` and `std::fs`/`std::path::Path` (already in scope). `scan_routes` calls `crate::commands::generate_routes::parse_routes_file` — the import path is unchanged after relocation.

---

### `ferro-cli/src/lib.rs` — Remove `pub mod ai;`

**Analog:** self (current `ferro-cli/src/lib.rs` line 7)

One-line deletion:

```rust
// ferro-cli/src/lib.rs line 7 — DELETE this line
pub mod ai;
```

Result: the 6-line file loses one declaration. No other changes needed.

---

### `ferro-cli/Cargo.toml` — Add ferro-ai dependency

**Analog:** self (`ferro-cli/Cargo.toml` lines 44–46 show how workspace-local path deps are declared)

```toml
# ferro-cli/Cargo.toml lines 44-46 — pattern for workspace-local dep with path+version
ferro-json-ui = { path = "../ferro-json-ui", version = "0.2" }
ferro-mcp = { path = "../ferro-mcp", version = "0.2" }
```

Add after line 46:

```toml
ferro-ai = { path = "../ferro-ai", version = "0.2" }
```

Keep `reqwest` exactly as-is (line 47 — `blocking` feature must NOT be removed, per D-06):

```toml
# ferro-cli/Cargo.toml line 47 — do NOT touch this line
reqwest = { version = "0.12", features = ["blocking", "json"] }
```

No publish.yml changes needed — ferro-ai is already in Wave 1b, ferro-cli in Wave 3.

---

## Shared Patterns

### Tokio Runtime Construction in ferro-cli sync commands

**Source:** `ferro-cli/src/commands/mcp.rs` lines 31–41
**Apply to:** `generate_with_ai()` in `make_json_view.rs`

The `mcp.rs` pattern is the preferred one because it uses `match ... { Ok(rt) => rt, Err(e) => { eprintln!(...); process::exit(1); } }`. In `generate_with_ai`, substitute `process::exit(1)` with `return templates::json_view_template(...)` (command functions use early return, not exit, for non-fatal errors).

### Yellow warning + static fallback stderr style

**Source:** `ferro-cli/src/commands/make_json_view.rs` lines 120–128 (existing code)
**Apply to:** All new error paths in `generate_with_ai`

Pattern: `eprintln!("{} ...", style("Warning:").yellow().bold(), e)` followed by `eprintln!("{}", style("Falling back to static template.").dim())` followed by `return templates::json_view_template(...)`. This pattern is already in the file — new call sites copy the same structure.

### console::style imports

**Source:** `ferro-cli/src/commands/make_json_view.rs` line 7
**Apply to:** `make_json_view.rs` (already imported — no change needed)

```rust
use console::style;
```

---

## No Analog Found

None. Every pattern has a direct analog in the codebase.

---

## Key Observations for Planner

1. **`Message` and `Role` are not at the crate root.** `ferro-ai/src/lib.rs` lines 61–64 re-export `CompletionRequest` but NOT `Message` or `Role`. The implementer must use `ferro_ai::client::{Message, Role}` as the import path, or the planner can include a note to add them to the crate-root re-export if that is cleaner.

2. **The best async→sync bridge analog is in ferro-cli itself** — `mcp.rs` lines 31–46 and `auth_link.rs` lines 32–34, both already present in the same crate. No need to look outside the workspace.

3. **`AiConfig::from_env()` has no existing call site outside ferro-ai's own tests.** This phase is the first consumer. The call shape is confirmed from `ferro-ai/src/config.rs` lines 44–74 and the test call sites at lines 82–151.

4. **`generate_with_ai` signature change:** Currently takes `file_name`, `title`, `layout_name`, `description` and calls `ai::*` internally. After migration it must receive `client: Box<dyn LlmClient>` as an additional parameter (or construct it internally — planner's call). Receiving the client as a parameter is cleaner because it separates the gating decision (in `run()`) from the two-pass execution (in `generate_with_ai()`).

5. **SC#2 wording correction required.** ROADMAP Phase 170 SC#2 says "all LLM calls go through `ferro_ai::complete::<T>()`". This is incorrect for this phase — see CONTEXT.md D-02. The plan must annotate or correct SC#2 to read "through the ferro-ai SDK / `LlmClient::complete()`".

---

## Metadata

**Analog search scope:** `ferro-cli/src/`, `ferro-ai/src/`, `framework/src/`
**Files scanned:** 12
**Pattern extraction date:** 2026-06-08
