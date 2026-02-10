//! AI-powered view generation via the Anthropic API.
//!
//! Provides two main functions:
//! - `call_anthropic`: Makes a blocking request to the Anthropic Messages API.
//! - `build_view_context`: Assembles a prompt with component catalog, project models, and routes.

use regex::Regex;
use std::fs;
use std::path::Path;

use crate::commands::generate_routes;

/// Concise reference of all JSON-UI components (20 built-in + plugin components).
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

## Plugin Components

Plugin components use the same JSON syntax as built-in components. Their JS/CSS assets are loaded automatically.

### Map
Props: center ([f64; 2] required), zoom (u8 0-18, default 13), height (String, default "400px"), markers (Vec<{lat, lng, popup?}>), tile_url (Option<String>), attribution (Option<String>), max_zoom (Option<u8>)
Example JSON: {"type": "Map", "center": [51.505, -0.09], "zoom": 13, "markers": [{"lat": 51.5, "lng": -0.09, "popup": "Hello"}]}
Note: Leaflet CSS/JS loaded via CDN automatically. Works inside Tabs/Modals (IntersectionObserver handles resize).

## Action
Props: handler (String "controller.method" format), method (GET|POST|PUT|PATCH|DELETE), confirm (Option<ConfirmDialog {title, message?, variant: default|danger}>), on_success (Option<ActionOutcome>), on_error (Option<ActionOutcome>)
Builders: Action::new("handler") (POST), Action::get("handler"), Action::delete("handler"), .confirm("title"), .confirm_danger("title")

## ComponentNode
Wraps every component: key (String), component (Component variant), action (Option<Action>), visibility (Option<Visibility>)

## JsonUiView Builder
JsonUiView::new().title("Title").layout("app").data(json).component(node).components(vec_of_nodes)
"#;

/// Call the Anthropic Messages API with separate system and user prompts.
///
/// Reads `ANTHROPIC_API_KEY` from environment. Model defaults to `claude-sonnet-4-5`
/// but can be overridden via `FERRO_AI_MODEL`.
///
/// Uses Anthropic best practices: system prompt with cache_control, assistant prefill,
/// temperature 0.2 for deterministic output, and 60-second HTTP timeout.
pub fn call_anthropic(system: &str, user_prompt: &str) -> Result<String, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
        "ANTHROPIC_API_KEY not set. Export it with:\n  \
         export ANTHROPIC_API_KEY=sk-ant-...\n\
         Or use --no-ai for a static template."
            .to_string()
    })?;

    let model = std::env::var("FERRO_AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".to_string());

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 8192,
        "temperature": 0.2,
        "system": [
            {
                "type": "text",
                "text": system,
                "cache_control": {"type": "ephemeral"}
            }
        ],
        "messages": [
            {"role": "user", "content": user_prompt},
            {"role": "assistant", "content": "//!"}
        ]
    });

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

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response JSON: {e}"))?;

    let response_text = json["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|item| item["text"].as_str())
        .ok_or_else(|| format!("Unexpected response structure: {text}"))?;

    // Prepend the assistant prefill that is not included in the response
    Ok(format!("//!{response_text}"))
}

/// Assemble system and user prompts for AI view generation.
///
/// Returns `(system_prompt, user_prompt)` where:
/// - System prompt: role, rules, component catalog, few-shot example (static, cacheable)
/// - User prompt: project models, routes, view name and description (dynamic, per-request)
pub fn build_view_context(name: &str, description: &str) -> (String, String) {
    // System prompt: static content that benefits from prompt caching
    let system = format!(
        "You are a Ferro framework JSON-UI view code generator. Generate only valid Rust source \
         code for src/views/ files.\n\n\
         Rules:\n\
         - Import only types actually used from `use ferro::{{...}};`\n\
         - Use the builder pattern: `JsonUiView::new().title().layout().component()`\n\
         - Use .layout(\"app\") unless the view is for auth (use \"auth\")\n\
         - Use real route handler names for actions when matching routes exist\n\
         - Use data_path bindings for form fields when matching model fields exist\n\
         - Return ONLY Rust source code, no explanation\n\n\
         {COMPONENT_CATALOG}\n\n\
         <example>\n\
         Input: user_list view showing all users in a table with edit and delete actions\n\
         Output:\n\
         //! User List JSON-UI view\n\n\
         use ferro::{{\n\
             Action, Component, ComponentNode, JsonUiView, TableColumn, TableProps, TextElement, \
         TextProps,\n\
         }};\n\n\
         pub fn view() -> JsonUiView {{\n\
             JsonUiView::new()\n\
                 .title(\"User List\")\n\
                 .layout(\"app\")\n\
                 .component(ComponentNode {{\n\
                     key: \"heading\".to_string(),\n\
                     component: Component::Text(TextProps {{\n\
                         content: \"User List\".to_string(),\n\
                         element: TextElement::H1,\n\
                     }}),\n\
                     action: None,\n\
                     visibility: None,\n\
                 }})\n\
                 .component(ComponentNode {{\n\
                     key: \"users_table\".to_string(),\n\
                     component: Component::Table(TableProps {{\n\
                         columns: vec![\n\
                             TableColumn {{ key: \"name\".to_string(), label: \"Name\".to_string(), \
         format: None }},\n\
                             TableColumn {{ key: \"email\".to_string(), label: \
         \"Email\".to_string(), format: None }},\n\
                         ],\n\
                         data_path: \"users\".to_string(),\n\
                         row_actions: Some(vec![\n\
                             Action::get(\"user_controller.edit\"),\n\
                             Action::delete(\"user_controller.destroy\").confirm_danger(\"Delete \
         user\"),\n\
                         ]),\n\
                         empty_message: Some(\"No users found.\".to_string()),\n\
                         sortable: None,\n\
                         sort_column: None,\n\
                         sort_direction: None,\n\
                     }}),\n\
                     action: None,\n\
                     visibility: None,\n\
                 }})\n\
         }}\n\
         </example>",
    );

    // User prompt: dynamic content that changes per request
    let mut user_prompt = String::new();

    let models = scan_models();
    if !models.is_empty() {
        user_prompt.push_str("## Project Models\n");
        user_prompt.push_str(&models);
        user_prompt.push('\n');
    }

    let routes = scan_routes();
    if !routes.is_empty() {
        user_prompt.push_str("## Project Routes\n");
        user_prompt.push_str(&routes);
        user_prompt.push('\n');
    }

    user_prompt.push_str(&format!(
        "Generate `src/views/{name}.rs`:\n\
         View name: {name}\n\
         Description: {description}",
    ));

    (system, user_prompt)
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
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        if path.file_name().is_some_and(|n| n == "mod.rs") {
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
