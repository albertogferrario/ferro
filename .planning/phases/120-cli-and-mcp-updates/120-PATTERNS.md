# Phase 120: CLI & MCP Updates - Pattern Map

**Mapped:** 2026-04-21
**Files analyzed:** 6
**Analogs found:** 6 / 6

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-cli/src/commands/make_json_view.rs` | command | request-response | `ferro-cli/src/commands/make_json_view.rs` (v12.0) | self — substitution |
| `ferro-cli/src/ai.rs` | utility | request-response | `ferro-cli/src/ai.rs` (v12.0, current) | self — additive |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | tool | CRUD | `ferro-mcp/src/tools/json_ui_catalog.rs` (v12.0) | self — additive |
| `ferro-mcp/src/tools/json_ui_inspect.rs` | tool | file-I/O | `ferro-mcp/src/tools/json_ui_inspect.rs` | self — rewrite |
| `ferro-mcp/src/tools/json_ui_generate.rs` | tool | CRUD | `ferro-mcp/src/tools/json_ui_generate.rs` (v12.0) | self — string substitution |
| `ferro-mcp/src/tools/code_templates.rs` | tool | CRUD | `ferro-mcp/src/tools/code_templates.rs` | self — template replacement |

All six files are modifications of existing files — no net-new files. Analogs are the files themselves (current state on v12.0 branch vs. target state). Where behavior differs between master and v12.0, the v12.0 version is the starting point for the edit.

---

## Pattern Assignments

### `ferro-cli/src/commands/make_json_view.rs` (command, request-response)

**Analog:** `ferro-cli/src/commands/make_json_view.rs` — current v12.0 state (which already removed the v1 catalog import and `build_view_context`).

**Imports pattern** (lines 1-11, same on both branches):
```rust
use console::style;
use std::fs;
use std::path::Path;

use crate::ai;
use crate::templates;
```

**D-01: File path change** — replace `.rs` extension with `.json` throughout:
```rust
// BEFORE (current v12.0)
let view_file = views_dir.join(format!("{file_name}.rs"));
let mod_file = views_dir.join("mod.rs");

// AFTER (Phase 120 target)
let view_file = views_dir.join(format!("{file_name}.json"));
// mod_file variable removed entirely — JSON files are not Rust modules
```

**D-01: Remove mod.rs logic** — delete the following blocks entirely:
- Lines 54-66: mod.rs exists-and-contains check
- Lines 120-141: mod.rs update/create after writing view file
- `update_mod_file` function (lines 206-242)

**D-02: Two-pass AI orchestration** (new logic in the `Ok(_)` arm of `env::var("ANTHROPIC_API_KEY")`):
```rust
Ok(_) => {
    let desc = description.as_deref().unwrap_or(&title);
    println!("{} Generating view with AI...", style("⏳").cyan());

    let (system_p1, user_p1) = ai::build_json_view_pass1(&file_name, desc);

    let pass1_result = match ai::call_anthropic_plain(&system_p1, &user_p1) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{} AI Pass 1 failed: {}", style("Warning:").yellow().bold(), e);
            eprintln!("{}", style("Falling back to static template.").dim());
            return templates::json_view_template(&file_name, &title, layout_name);
        }
    };

    let schema = ferro_json_ui::global_catalog().json_schema().clone();
    let (system_p2, user_p2) = ai::build_json_view_pass2(&pass1_result, &schema);

    match ai::call_anthropic_structured(&system_p2, &user_p2, schema) {
        Ok(json_str) => {
            // D-03: validate the structured output
            match ferro_json_ui::spec::Spec::from_json(&json_str)
                .and_then(|spec| ferro_json_ui::global_catalog().validate(&spec).map(|_| json_str.clone()))
            {
                Ok(valid_json) => valid_json,
                Err(e) => {
                    eprintln!(
                        "{} Spec validation failed: {:?}",
                        style("Warning:").yellow().bold(),
                        e
                    );
                    eprintln!("{}", style("Falling back to static template.").dim());
                    templates::json_view_template(&file_name, &title, layout_name)
                }
            }
        }
        Err(e) => {
            eprintln!("{} AI Pass 2 failed: {}", style("Warning:").yellow().bold(), e);
            eprintln!("{}", style("Falling back to static template.").dim());
            templates::json_view_template(&file_name, &title, layout_name)
        }
    }
}
```

**D-01 + D-06: Usage message update** — replace the `println!` block at lines 144-156 with v2 guidance:
```rust
println!("Usage:");
println!("  {} Use the view in a handler:", style("1.").dim());
println!();
println!("     #[handler]");
println!("     pub async fn {file_name}(req: Request) -> Response {{");
println!("         let data = serde_json::json!({{}});");
println!("         JsonUi::render_file(\"views/{file_name}.json\", data)");
println!("     }}");
println!();
```

**Fallback pattern** — warning + dim text before returning static template. Used consistently throughout this file. Copy from existing lines 84-91:
```rust
eprintln!(
    "{} AI generation failed: {}",
    style("Warning:").yellow().bold(),
    e
);
eprintln!("{}", style("Falling back to static template.").dim());
templates::json_view_template(&file_name, &title, layout_name)
```

---

### `ferro-cli/src/ai.rs` (utility, request-response)

**Analog:** `ferro-cli/src/ai.rs` — v12.0 version (lines 1-82 of the file on this branch are the `call_anthropic` function with `//!` prefill).

**Imports pattern** — add `ferro_json_ui::global_catalog` to existing imports (already present on v12.0):
```rust
use ferro_json_ui::global_catalog;
use regex::Regex;
use std::fs;
use std::path::Path;
use crate::commands::generate_routes;
```

**Existing `call_anthropic` shape** (lines 21-82 current) — do NOT modify:
- Has `"//!"` assistant prefill in the messages array
- Returns `Ok(format!("//!{response_text}"))` — prepends the prefill
- Used by other code that depends on this behavior

**New function 1: `call_anthropic_plain`** — copy `call_anthropic` and remove both prefill nodes:
```rust
/// Call the Anthropic API expecting plain-text response (no assistant prefill).
///
/// Used for Pass 1 of two-pass JSON-UI generation where the model returns
/// a plain-text component plan. The `//!` prefill from `call_anthropic`
/// is intentionally absent here.
pub fn call_anthropic_plain(system: &str, user_prompt: &str) -> Result<String, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| { /* same error text */ })?;
    let model = std::env::var("FERRO_AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".to_string());

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "temperature": 0.2,
        "system": [{ "type": "text", "text": system, "cache_control": {"type": "ephemeral"} }],
        "messages": [{ "role": "user", "content": user_prompt }]
        // NO assistant prefill block
    });

    // ... same HTTP client, POST, error handling as call_anthropic ...

    let response_text = json["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|item| item["text"].as_str())
        .ok_or_else(|| format!("Unexpected response structure: {text}"))?;

    Ok(response_text.to_string())   // NO "//!" prepend
}
```

**New function 2: `call_anthropic_structured`** — uses `tool_use` mechanism (see RESEARCH.md Pattern 1):
```rust
/// Call the Anthropic API using tool_use for structured JSON output (Pass 2).
///
/// Forces model output to conform to the JSON-UI Spec schema via tool_choice.
/// Returns the raw JSON string from the tool_use input block.
pub fn call_anthropic_structured(
    system: &str,
    user_prompt: &str,
    input_schema: serde_json::Value,
) -> Result<String, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| { /* same error text */ })?;
    let model = std::env::var("FERRO_AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".to_string());

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "temperature": 0.2,
        "system": [{ "type": "text", "text": system, "cache_control": {"type": "ephemeral"} }],
        "tools": [{
            "name": "emit_spec",
            "description": "Emit the complete v2 JSON-UI spec for the requested view.",
            "input_schema": input_schema
        }],
        "tool_choice": { "type": "tool", "name": "emit_spec" },
        "messages": [{ "role": "user", "content": user_prompt }]
    });

    // ... same HTTP client and POST/error as call_anthropic ...

    // Different response extraction: tool_use block, not text block
    let spec_value = json["content"]
        .as_array()
        .and_then(|arr| arr.iter().find(|item| item["type"] == "tool_use"))
        .and_then(|item| item.get("input"))
        .cloned()
        .ok_or_else(|| "no tool_use block in response".to_string())?;

    serde_json::to_string(&spec_value)
        .map_err(|e| format!("serializing spec: {e}"))
}
```

**New function 3: `build_json_view_pass1`** — replaces `build_view_context` for Pass 1:
```rust
/// Assemble system and user prompts for JSON-UI view generation Pass 1 (plain text plan).
///
/// Returns `(system_prompt, user_prompt)`.
/// System: role + catalog prompt (cacheable). User: name + description (dynamic).
pub fn build_json_view_pass1(name: &str, description: &str) -> (String, String) {
    let catalog_prompt = global_catalog().prompt();
    let system = format!(
        "You are a Ferro JSON-UI view planner. Given a view name and description, \
         produce a plain-text component plan: which components to use, what data \
         each displays, what actions are present. Be concise.\n\n\
         {catalog_prompt}"
    );
    let user = format!(
        "View name: {name}\nDescription: {description}"
    );
    (system, user)
}
```

**New function 4: `build_json_view_pass2`** — assembles Pass 2 prompts from Pass 1 output:
```rust
/// Assemble system and user prompts for JSON-UI view generation Pass 2 (structured spec).
///
/// `pass1_result` is the plain-text component plan from Pass 1.
/// Returns `(system_prompt, user_prompt)`.
pub fn build_json_view_pass2(pass1_result: &str, _schema: &serde_json::Value) -> (String, String) {
    let system = "You are a Ferro JSON-UI view generator. Use the tool to emit the complete \
                  v2 JSON spec for the view described in the component plan below."
        .to_string();
    let user = pass1_result.to_string();
    (system, user)
}
```

**Deletion:** `build_view_context` is replaced by `build_json_view_pass1` + `build_json_view_pass2`. Delete the entire `build_view_context` function. On master this is lines 84-174; on v12.0 it has a slightly different system prompt but same shape.

---

### `ferro-mcp/src/tools/json_ui_catalog.rs` (tool, CRUD)

**Analog:** `ferro-mcp/src/tools/json_ui_catalog.rs` — v12.0 state, which already sources from `global_catalog()` (Phase 117 migration). The struct itself has `components`, `plugin_components`, `builder_api`, `action_api` — the same fields as today.

**D-04: Add two fields to `JsonUiCatalog`** (additive to line 17):
```rust
#[derive(Debug, Serialize)]
pub struct JsonUiCatalog {
    pub components: Vec<CatalogComponent>,
    pub plugin_components: Vec<CatalogComponent>,
    pub builder_api: String,
    pub action_api: String,
    // NEW:
    pub json_schema: serde_json::Value,
    pub component_schemas: std::collections::HashMap<String, serde_json::Value>,
}
```

**D-04: Populate new fields in `execute()`** — add after the existing `components` and `plugin_components` are built (see RESEARCH.md Pattern 3, lines 248-259):
```rust
// After building components and plugin_components:
let json_schema = cat.json_schema().clone();
let component_schemas: std::collections::HashMap<String, serde_json::Value> = cat
    .components_sorted()
    .chain(cat.plugin_components_sorted())
    .filter_map(|spec| {
        cat.component_schema(&spec.name)
            .map(|s| (spec.name.clone(), s.clone()))
    })
    .collect();

JsonUiCatalog {
    components,
    plugin_components,
    builder_api: BUILDER_API.to_string(),
    action_api: ACTION_API.to_string(),
    json_schema,
    component_schemas,
}
```

**Note on BUILDER_API and ACTION_API:** These string constants remain as-is. They describe the v1 builder pattern on master; on v12.0 they will reference the flat-spec API. The Phase 120 scope only adds new fields — it does not touch these strings.

**Test pattern** — new tests should follow the existing `test_serialization` pattern (lines 1209-1222). Assert the new keys appear in the serialized JSON:
```rust
assert!(json_str.contains("json_schema"));
assert!(json_str.contains("component_schemas"));
```

---

### `ferro-mcp/src/tools/json_ui_inspect.rs` (tool, file-I/O)

**Analog:** `ferro-mcp/src/tools/json_ui_inspect.rs` (current file on v12.0) — the `execute()` function and `ViewInfo` struct are the templates; the regex scanning body is replaced.

**Keep unchanged:**
- `JsonUiViewList` struct (lines 11-14)
- `ViewInfo` struct (lines 17-31) — field names stay the same, semantics change
- `ComponentSchemaInfo` struct (lines 34-46)
- `inspect_component` function (lines 77-105) — with Pitfall 5 fix below

**D-05: Remove `BUILTIN_TYPES` const** (lines 49-70) — deleted entirely.

**D-05: Fix `inspect_component` to use `global_catalog()`** — replace the `is_builtin` check:
```rust
pub fn inspect_component(component_type: &str) -> ComponentSchemaInfo {
    use ferro_json_ui::global_catalog;
    let cat = global_catalog();

    // Replace BUILTIN_TYPES lookup with catalog query
    if cat.component_schema(component_type).is_some() {
        // Built-in: it has a schema in the catalog's built-in map
        let catalog_result = super::json_ui_catalog::execute(Some(component_type));
        let entry = catalog_result.components.into_iter().next();
        ComponentSchemaInfo {
            name: component_type.to_string(),
            is_plugin: false,
            props_schema: None,
            catalog_entry: entry,
        }
    } else {
        // Plugin or unknown
        let catalog_result = super::json_ui_catalog::execute(Some(component_type));
        let plugin_entry = catalog_result.plugin_components.into_iter().next();
        let schema = ferro_json_ui::with_plugin(component_type, |plugin| plugin.props_schema());
        ComponentSchemaInfo {
            name: component_type.to_string(),
            is_plugin: true,
            props_schema: schema,
            catalog_entry: plugin_entry,
        }
    }
}
```

**D-05: Rewrite `execute()` to scan `src/views/*.json`** — copy the file-walk skeleton from RESEARCH.md Pattern 2. Key differences from current:
- Extension filter: `.json` instead of `.rs`; no `mod.rs` skip needed
- No regex compilation — parse raw `serde_json::Value` instead
- Extract `title`, `layout`, `components_used`, `actions` from JSON structure, not regex

The existing `execute()` signature remains the same: `pub fn execute(project_root: &Path, filter: Option<&str>) -> JsonUiViewList`.

**Copy pattern for fs::read_dir** from current `execute()` lines 127-135:
```rust
let entries: Vec<_> = match fs::read_dir(&views_dir) {
    Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
    Err(_) => {
        return JsonUiViewList { views: Vec::new(), total: 0 }
    }
};
```

**JSON field extraction pattern** (from RESEARCH.md Pattern 2, lines 212-235):
```rust
let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
let title = json["title"].as_str().map(String::from);
let layout = json["layout"].as_str().map(String::from);
let components_used: Vec<String> = json["elements"]
    .as_object()
    .map(|m| {
        let mut types: Vec<String> = m.values()
            .filter_map(|el| el["type"].as_str().map(String::from))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        types.sort();
        types
    })
    .unwrap_or_default();
let actions: Vec<String> = json["elements"]
    .as_object()
    .map(|m| m.values()
        .filter_map(|el| el["action"].as_object()
            .and_then(|a| a.get("handler"))
            .and_then(|h| h.as_str())
            .map(String::from))
        .collect())
    .unwrap_or_default();
let relative = path.strip_prefix(project_root).unwrap_or(&path)
    .to_string_lossy().to_string();
```

**Test update** — `test_serialization` test currently hardcodes `.rs` file path and `"Table"` in `components_used`. Update to use `.json` path and v2 field structure.

---

### `ferro-mcp/src/tools/json_ui_generate.rs` (tool, CRUD)

**Analog:** `ferro-mcp/src/tools/json_ui_generate.rs` — v12.0 state (which already uses `global_catalog().prompt()` in `execute()`).

**D-07: Replace `VIEW_EXAMPLE` const** — the const currently contains v1 builder-pattern Rust code. Replace entire string value with a v2 JSON spec example (from RESEARCH.md Code Examples, lines 349-368):
```rust
const VIEW_EXAMPLE: &str = r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "User List",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "User List" },
      "children": ["heading", "users_table"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "User List", "element": "h1" }
    },
    "users_table": {
      "type": "DataTable",
      "props": {
        "columns": [
          {"key": "name", "label": "Name"},
          {"key": "email", "label": "Email"}
        ],
        "data_path": "/data/users",
        "empty_message": "No users found."
      }
    }
  }
}"#;
```

**D-07: Update `ViewConventions` in `execute()`** — the three string fields that reference v1 patterns:
```rust
conventions: ViewConventions {
    file_location: "src/views/{name}.json".to_string(),        // was "{name}.rs"
    function_signature: String::new(),                           // removed — JSON files have no function signature
    import_pattern: String::new(),                               // removed — no imports in JSON
    layout_default: "dashboard".to_string(),
},
```

**Alternative:** if removing fields breaks consumers, replace with v2 equivalents:
```rust
file_location: "src/views/{name}.json".to_string(),
function_signature: "JsonUi::render_file(\"views/{name}.json\", data)".to_string(),
import_pattern: "use ferro::JsonUi;".to_string(),
layout_default: "dashboard".to_string(),
```

**D-07: Update `list_existing_views()`** — change extension filter from `.rs` to `.json` and remove `mod.rs` skip (JSON files have no equivalent):
```rust
fn list_existing_views(project_root: &Path) -> Vec<String> {
    // ...
    for entry in entries {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {  // was "rs"
            continue;
        }
        // NO mod.rs skip — JSON files don't have a module index
        if let Some(name) = path.file_name() {
            views.push(name.to_string_lossy().to_string());
        }
    }
    // ...
}
```

**Test update** — `test_example_not_empty` asserts `result.example.contains("JsonUiView")` and `result.example.contains("pub fn view()")`. After D-07, update assertions to:
```rust
assert!(result.example.contains("$schema"));
assert!(result.example.contains("ferro-json-ui/v2"));
assert!(result.example.contains("elements"));
```

**Test update** — `test_conventions_populated` asserts `file_location == "src/views/{name}.rs"`. Update to `"src/views/{name}.json"`.

---

### `ferro-mcp/src/tools/code_templates.rs` (tool, CRUD)

**Analog:** `ferro-mcp/src/tools/code_templates.rs` (current file) — the `json_view_templates()` function (lines 902-1141) and the `CodeTemplate` struct shape (lines 14-28) are the exact templates to copy and modify.

**D-06: Replace all 3 `json_view_templates()` templates with v2 JSON spec strings:**

Template 1 — `basic_view` (replace the `code` field only; keep `name`, `category`, `description`, `imports`, `placeholders` shape):
```rust
CodeTemplate {
    name: "basic_view".to_string(),
    category: "json_view".to_string(),
    description: "Minimal JSON-UI v2 spec with title, heading, and card".to_string(),
    code: r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "{{title}}",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "{{title}}" },
      "children": ["heading"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "{{title}}", "element": "h1" }
    }
  }
}"#.to_string(),
    imports: vec![],
    placeholders: vec![
        Placeholder { name: "{{view_name}}".to_string(), description: "View file name (snake_case)".to_string(), example: "dashboard".to_string() },
        Placeholder { name: "{{title}}".to_string(), description: "Page title".to_string(), example: "Dashboard".to_string() },
    ],
},
```

Template 2 — `list_view`:
```rust
CodeTemplate {
    name: "list_view".to_string(),
    category: "json_view".to_string(),
    description: "JSON-UI v2 spec with DataTable for listing resources".to_string(),
    code: r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "{{title}}",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "{{title}}" },
      "children": ["header", "table"]
    },
    "header": {
      "type": "PageHeader",
      "props": {
        "title": "{{title}}",
        "actions": []
      }
    },
    "table": {
      "type": "DataTable",
      "props": {
        "columns": [
          {"key": "id", "label": "ID"},
          {"key": "name", "label": "Name"}
        ],
        "data_path": "/data/{{entity}}s",
        "empty_message": "No {{entity}}s found."
      }
    }
  }
}"#.to_string(),
    imports: vec![],
    placeholders: vec![
        Placeholder { name: "{{view_name}}".to_string(), description: "View file name (snake_case)".to_string(), example: "users_index".to_string() },
        Placeholder { name: "{{title}}".to_string(), description: "Page title".to_string(), example: "Users".to_string() },
        Placeholder { name: "{{Entity}}".to_string(), description: "Entity name PascalCase".to_string(), example: "User".to_string() },
        Placeholder { name: "{{entity}}".to_string(), description: "Entity name snake_case".to_string(), example: "user".to_string() },
    ],
},
```

Template 3 — `form_view`:
```rust
CodeTemplate {
    name: "form_view".to_string(),
    category: "json_view".to_string(),
    description: "JSON-UI v2 spec with Form, Input fields, and submit button".to_string(),
    code: r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "{{title}}",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "{{title}}" },
      "children": ["form"]
    },
    "form": {
      "type": "Form",
      "props": {
        "action": { "handler": "{{action_handler}}", "method": "POST" }
      },
      "children": ["name_field", "submit"]
    },
    "name_field": {
      "type": "Input",
      "props": {
        "field": "name",
        "label": "Name",
        "input_type": "text",
        "required": true
      }
    },
    "submit": {
      "type": "Button",
      "props": { "label": "Save {{Entity}}", "variant": "default" }
    }
  }
}"#.to_string(),
    imports: vec![],
    placeholders: vec![
        Placeholder { name: "{{view_name}}".to_string(), description: "View file name (snake_case)".to_string(), example: "users_create".to_string() },
        Placeholder { name: "{{title}}".to_string(), description: "Page title".to_string(), example: "Create User".to_string() },
        Placeholder { name: "{{Entity}}".to_string(), description: "Entity name PascalCase".to_string(), example: "User".to_string() },
        Placeholder { name: "{{action_handler}}".to_string(), description: "Route handler name".to_string(), example: "users.store".to_string() },
    ],
},
```

**D-06: New fourth template `json_view_handler`** — Rust handler using `JsonUi::render_file` (from RESEARCH.md Code Examples, lines 371-378):
```rust
CodeTemplate {
    name: "json_view_handler".to_string(),
    category: "json_view".to_string(),
    description: "Handler that serves a JSON-UI v2 view file using JsonUi::render_file".to_string(),
    code: r#"#[handler]
pub async fn {{view_name}}(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/{{view_name}}.json", data)
}"#.to_string(),
    imports: vec![
        "use ferro::{handler, Request, Response, JsonUi};".to_string(),
        "use serde_json::json;".to_string(),
    ],
    placeholders: vec![
        Placeholder {
            name: "{{view_name}}".to_string(),
            description: "Handler function name and JSON file stem (snake_case)".to_string(),
            example: "dashboard".to_string(),
        },
    ],
},
```

**Test update** — extend `test_all_categories_present` (already checks `json_view` category exists — no change needed). If a count-based test exists: update from 3 to 4 templates for `json_view` category (no such test exists currently per RESEARCH.md Pitfall 6).

---

## Shared Patterns

### HTTP client pattern (ferro-cli/src/ai.rs)
**Source:** `ferro-cli/src/ai.rs` lines 48-60
**Apply to:** All new `call_anthropic_*` functions
```rust
let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .build()
    .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

let response = client
    .post("https://api.anthropic.com/v1/messages")
    .header("x-api-key", &api_key)
    .header("anthropic-version", "2023-06-01")
    .header("content-type", "application/json")
    .json(&body)
    .send()
    .map_err(|e| format!("API request failed: {e}"))?;

let status = response.status();
let text = response
    .text()
    .map_err(|e| format!("Failed to read response body: {e}"))?;

if !status.is_success() {
    return Err(format!("Anthropic API error ({status}): {text}"));
}
```

### API key read pattern (ferro-cli/src/ai.rs)
**Source:** `ferro-cli/src/ai.rs` lines 22-27
**Apply to:** `call_anthropic_plain`, `call_anthropic_structured`
```rust
let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
    "ANTHROPIC_API_KEY not set. Export it with:\n  \
     export ANTHROPIC_API_KEY=sk-ant-...\n\
     Or use --no-ai for a static template."
        .to_string()
})?;
```

### fs::read_dir walk pattern (ferro-mcp/src/tools/json_ui_inspect.rs)
**Source:** `ferro-mcp/src/tools/json_ui_inspect.rs` lines 127-135
**Apply to:** `json_ui_inspect.rs` rewritten `execute()`, `json_ui_generate.rs` `list_existing_views()`
```rust
let entries: Vec<_> = match fs::read_dir(&views_dir) {
    Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
    Err(_) => {
        return JsonUiViewList { views: Vec::new(), total: 0 }
    }
};
```

### Warning + fallback pattern (ferro-cli/src/commands/make_json_view.rs)
**Source:** `ferro-cli/src/commands/make_json_view.rs` lines 84-92
**Apply to:** All fallback paths in `make_json_view.rs::run()`
```rust
eprintln!(
    "{} <context>: {}",
    style("Warning:").yellow().bold(),
    e
);
eprintln!("{}", style("Falling back to static template.").dim());
templates::json_view_template(&file_name, &title, layout_name)
```

### Catalog access pattern (ferro-mcp/src/tools/json_ui_catalog.rs v12.0)
**Source:** `ferro-mcp/src/tools/json_ui_catalog.rs` lines 45-50 on v12.0 branch
**Apply to:** `json_ui_inspect.rs` `inspect_component()`, `json_ui_catalog.rs` `execute()`, `ferro-cli/src/ai.rs` prompt builders
```rust
use ferro_json_ui::global_catalog;
let cat = global_catalog();
// then call: cat.json_schema(), cat.validate(&spec), cat.prompt(),
//            cat.component_schema(name), cat.components_sorted(), cat.plugin_components_sorted()
```

---

## No Analog Found

All six files have direct analogs (they are modifications of existing files). No net-new files require external pattern reference.

| File | Resolution |
|------|-----------|
| `ferro-cli/src/ai.rs` (new functions) | Copy `call_anthropic` structure; apply RESEARCH.md Pattern 1 for `call_anthropic_structured` |
| `ferro-cli/src/templates/make.rs` `json_view_template()` | Return v2 JSON string (RESEARCH.md Code Examples) instead of Rust source string |

---

## Key Observations for Planner

1. **`ferro-cli/src/templates/make.rs` is also a target.** `json_view_template()` (lines 103-138) currently returns a Rust source string. After D-01, it must return a v2 JSON string. This file was mentioned in RESEARCH.md but not in the scope list — planner should include it.

2. **All catalog API calls require the v12.0 branch.** `global_catalog()`, `json_schema()`, `validate()`, `prompt()`, `components_sorted()`, `plugin_components_sorted()`, `component_schema()` are implemented in `ferro-json-ui/src/catalog.rs` which exists only on the `v12.0/json-ui-v2` branch (Phase 117). These calls will compile only on that branch.

3. **`Spec::from_json` is used in D-03 validation.** The `spec` module is from Phase 115/119 (`ferro_json_ui::spec::Spec`). Confirm the import path before using.

4. **Dependency order from RESEARCH.md:** D-04 (catalog fields) → D-05 (inspect rewrite) → D-06 (code_templates) → D-07 (json_ui_generate) → D-02 (two-pass AI) → D-01 (make_json_view output + templates).

5. **`BUILDER_API` and `ACTION_API` string constants in `json_ui_catalog.rs`** describe the v1 builder API on master. On v12.0 these will need updating to describe the flat-spec API — but RESEARCH.md confirms this is NOT in Phase 120 scope (D-24 preserves the struct shape).

## Metadata

**Analog search scope:** `ferro-cli/src/`, `ferro-mcp/src/tools/`, `ferro-json-ui/src/` (v12.0 branch for catalog API)
**Files read:** 8 source files (6 target files + catalog.rs on v12.0 + json_ui_catalog.rs on v12.0)
**Pattern extraction date:** 2026-04-21
