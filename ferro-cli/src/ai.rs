//! AI-powered view generation via the Anthropic API.
//!
//! Provides two main functions:
//! - `call_anthropic`: Makes a blocking request to the Anthropic Messages API.
//! - `build_view_context`: Assembles a prompt with component catalog, project models, and routes.

use regex::Regex;
use std::fs;
use std::path::Path;

use crate::commands::generate_routes;

/// Concise reference of all 20 JSON-UI components with their props, types, and variants.
const COMPONENT_CATALOG: &str = r#"## Component Catalog

### Text
Props: content (String), element (h1|h2|h3|span|p)

### Button
Props: label (String), variant (default|secondary|destructive|outline|ghost|link), size (xs|sm|default|lg), disabled (Option<bool>), icon (Option<String>), icon_position (Option<left|right>)

### Card
Props: title (String), description (Option<String>), children (Vec<ComponentNode>), footer (Vec<ComponentNode>)

### Table
Props: columns (Vec<Column {key, label, format?}>), data_path (String), row_actions (Option<Vec<Action>>), empty_message (Option<String>), sortable (Option<bool>), sort_column (Option<String>), sort_direction (Option<asc|desc>)

### Form
Props: action (Action), fields (Vec<ComponentNode>), method (Option<GET|POST|PUT|PATCH|DELETE>)

### Input
Props: field (String), label (String), input_type (text|email|password|number|textarea|hidden|date|time|url|tel|search), placeholder (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>), description (Option<String>), default_value (Option<String>), data_path (Option<String>)

### Select
Props: field (String), label (String), options (Vec<SelectOption {value, label}>), placeholder (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>), description (Option<String>), default_value (Option<String>), data_path (Option<String>)

### Alert
Props: message (String), variant (info|success|warning|error), title (Option<String>)

### Badge
Props: label (String), variant (default|secondary|destructive|outline)

### Modal
Props: title (String), description (Option<String>), children (Vec<ComponentNode>), footer (Vec<ComponentNode>), trigger_label (Option<String>)

### Checkbox
Props: field (String), label (String), description (Option<String>), checked (Option<bool>), data_path (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>)

### Switch
Props: field (String), label (String), description (Option<String>), checked (Option<bool>), data_path (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>)

### Separator
Props: orientation (Option<horizontal|vertical>)

### DescriptionList
Props: items (Vec<DescriptionItem {label, value, format?}>), columns (Option<u8>)

### Tabs
Props: default_tab (String), tabs (Vec<Tab {value, label, children}>)

### Breadcrumb
Props: items (Vec<BreadcrumbItem {label, url?}>)

### Pagination
Props: current_page (u32), per_page (u32), total (u32), base_url (Option<String>)

### Progress
Props: value (u8 0-100), max (Option<u8>), label (Option<String>)

### Avatar
Props: src (Option<String>), alt (String), fallback (Option<String>), size (Option<xs|sm|default|lg>)

### Skeleton
Props: width (Option<String>), height (Option<String>), rounded (Option<bool>)

## Action
Props: handler (String "controller.method" format), method (GET|POST|PUT|PATCH|DELETE), confirm (Option<ConfirmDialog {title, message?, variant: default|danger}>), on_success (Option<ActionOutcome>), on_error (Option<ActionOutcome>)
Builders: Action::new("handler") (POST), Action::get("handler"), Action::delete("handler"), .confirm("title"), .confirm_danger("title")

## ComponentNode
Wraps every component: key (String), component (Component variant), action (Option<Action>), visibility (Option<Visibility>)

## JsonUiView Builder
JsonUiView::new().title("Title").layout("app").data(json).component(node).components(vec_of_nodes)
"#;

/// Call the Anthropic Messages API with the given prompt.
///
/// Reads `ANTHROPIC_API_KEY` from environment. Model defaults to `claude-opus-4-6`
/// but can be overridden via `FERRO_AI_MODEL`.
pub fn call_anthropic(prompt: &str) -> Result<String, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
        "ANTHROPIC_API_KEY not set. Export it with:\n  \
         export ANTHROPIC_API_KEY=sk-ant-...\n\
         Or use --no-ai for a static template."
            .to_string()
    })?;

    let model =
        std::env::var("FERRO_AI_MODEL").unwrap_or_else(|_| "claude-opus-4-6".to_string());

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 8192,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let client = reqwest::blocking::Client::new();
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

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response JSON: {e}"))?;

    json["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|item| item["text"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected response structure: {text}"))
}

/// Assemble a prompt with project context for AI view generation.
///
/// Includes:
/// 1. Component catalog (hardcoded const)
/// 2. Project models (scanned from `src/models/*.rs`)
/// 3. Project routes (parsed from `src/routes.rs`)
pub fn build_view_context(name: &str, description: &str) -> String {
    let mut prompt = String::new();

    prompt.push_str(
        "You are a Ferro framework JSON-UI view code generator. \
         Generate a Rust file that builds a JsonUiView.\n\n",
    );

    // Section 1: Component catalog
    prompt.push_str(COMPONENT_CATALOG);
    prompt.push('\n');

    // Section 2: Project models
    let models = scan_models();
    if !models.is_empty() {
        prompt.push_str("## Project Models\n");
        prompt.push_str(&models);
        prompt.push('\n');
    }

    // Section 3: Project routes
    let routes = scan_routes();
    if !routes.is_empty() {
        prompt.push_str("## Project Routes\n");
        prompt.push_str(&routes);
        prompt.push('\n');
    }

    prompt.push_str("## Instructions\n\n");
    prompt.push_str(&format!(
        "Generate a Rust file for `src/views/{name}.rs` with:\n\
         - A `//! {title} JSON-UI view` module doc comment\n\
         - `use ferro::{{...}};` imports (only import types actually used)\n\
         - A `pub fn view() -> JsonUiView` function using the builder pattern\n\
         - Use real route names for action handlers when matching routes exist\n\
         - Use data_path bindings for form fields when matching model fields exist\n\
         - Choose appropriate components for the described UI\n\
         - Use .layout(\"app\") unless the description suggests auth/login (use \"auth\")\n\n\
         View name: {name}\n\
         Description: {description}\n\n\
         Return ONLY the Rust source code. No markdown fences, no explanation.\n",
        name = name,
        title = to_title_case(name),
        description = description,
    ));

    prompt
}

/// Scan `src/models/*.rs` and extract struct fields using regex.
fn scan_models() -> String {
    let models_dir = Path::new("src/models");
    if !models_dir.exists() {
        return String::new();
    }

    let struct_re = Regex::new(r"pub\s+struct\s+(\w+)\s*\{").unwrap();
    let field_re = Regex::new(r"pub\s+(\w+)\s*:\s*([^,\n]+)").unwrap();

    let mut output = String::new();

    let entries: Vec<_> = match fs::read_dir(models_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return String::new(),
    };

    for entry in entries {
        let path = entry.path();
        if path.extension().map_or(true, |ext| ext != "rs") {
            continue;
        }
        if path.file_name().map_or(false, |n| n == "mod.rs") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Find struct definitions
        for struct_cap in struct_re.captures_iter(&content) {
            let struct_name = &struct_cap[1];
            let struct_start = struct_cap.get(0).unwrap().end();

            // Find the closing brace for this struct
            let rest = &content[struct_start..];
            let mut depth = 1;
            let mut struct_end = rest.len();
            for (i, ch) in rest.chars().enumerate() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            struct_end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let struct_body = &rest[..struct_end];
            let fields: Vec<String> = field_re
                .captures_iter(struct_body)
                .map(|cap| {
                    let field_name = cap[1].trim();
                    let field_type = cap[2].trim().trim_end_matches(',');
                    format!("{field_name} ({field_type})")
                })
                .collect();

            if !fields.is_empty() {
                output.push_str(&format!("### {}: {}\n", struct_name, fields.join(", ")));
            }
        }
    }

    output
}

/// Scan `src/routes.rs` and format route definitions.
fn scan_routes() -> String {
    let routes_file = Path::new("src/routes.rs");
    if !routes_file.exists() {
        return String::new();
    }

    let content = match fs::read_to_string(routes_file) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let routes = generate_routes::parse_routes_file(&content);
    let mut output = String::new();

    for route in &routes {
        let method = match route.method {
            generate_routes::HttpMethod::Get => "GET",
            generate_routes::HttpMethod::Post => "POST",
            generate_routes::HttpMethod::Put => "PUT",
            generate_routes::HttpMethod::Patch => "PATCH",
            generate_routes::HttpMethod::Delete => "DELETE",
        };

        let name_suffix = route
            .name
            .as_ref()
            .map(|n| format!(" (name: \"{}\")", n))
            .unwrap_or_default();

        output.push_str(&format!(
            "{} {} -> {}::{}{}\n",
            method, route.path, route.handler_module, route.handler_fn, name_suffix
        ));
    }

    output
}

/// Convert snake_case to Title Case.
fn to_title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut result = first.to_uppercase().to_string();
                    result.extend(chars);
                    result
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
