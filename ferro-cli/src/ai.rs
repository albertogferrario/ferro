//! AI-powered view generation via the Anthropic API.
//!
//! Provides:
//! - `call_anthropic_plain`: Makes a blocking request to the Anthropic Messages API, returns plain text.
//! - `call_anthropic_structured`: Structured output via Anthropic tool_use, returns JSON string.
//! - `build_json_view_pass1`: Builds system + user prompts for Pass 1 (plain-text plan).
//! - `build_json_view_pass2`: Builds system + user prompts for Pass 2 (structured spec).
//! - `generate_json_view`: Two-pass JSON-UI v2 spec generation (higher-level orchestration).

use console::style;
use ferro_json_ui::{global_catalog, Spec};
use regex::Regex;
use std::fs;
use std::path::Path;

use crate::commands::generate_routes;

/// Call the Anthropic Messages API with separate system and user prompts.
///
/// Reads `ANTHROPIC_API_KEY` from environment. Model defaults to `claude-sonnet-4-5`
/// but can be overridden via `FERRO_AI_MODEL`.
///
/// Uses Anthropic best practices: system prompt with cache_control, temperature 0.2
/// for deterministic output, and 60-second HTTP timeout.
///
/// Alias: also exported as `call_anthropic_plain` (the canonical name in Plan 05 interfaces).
pub fn call_anthropic_plain(system: &str, user_prompt: &str) -> Result<String, String> {
    call_anthropic(system, user_prompt)
}

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
        "max_tokens": 1024,
        "temperature": 0.2,
        "system": [
            {
                "type": "text",
                "text": system,
                "cache_control": {"type": "ephemeral"}
            }
        ],
        "messages": [
            {"role": "user", "content": user_prompt}
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

    Ok(response_text.to_string())
}

/// Call the Anthropic Messages API with structured output via tool_use.
///
/// Constrains the model output to a JSON Schema by using Anthropic's tool_use
/// mechanism: `tools: [{ name: "emit_spec", input_schema: schema }]` with
/// `tool_choice: { type: "tool", name: "emit_spec" }`.
///
/// Returns the tool input serialized as a JSON string on success.
pub fn call_anthropic_structured(
    system: &str,
    user_prompt: &str,
    schema: serde_json::Value,
) -> Result<String, String> {
    let api_key =
        std::env::var("ANTHROPIC_API_KEY").map_err(|_| "ANTHROPIC_API_KEY not set.".to_string())?;

    let model = std::env::var("FERRO_AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".to_string());

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "temperature": 0.2,
        "system": [
            {
                "type": "text",
                "text": system,
                "cache_control": {"type": "ephemeral"}
            }
        ],
        "tools": [
            {
                "name": "emit_spec",
                "description": "Emit the complete JSON-UI v2 spec for the requested view.",
                "input_schema": schema
            }
        ],
        "tool_choice": { "type": "tool", "name": "emit_spec" },
        "messages": [
            {"role": "user", "content": user_prompt}
        ]
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
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

    // Extract tool_use block input
    let tool_input = json["content"]
        .as_array()
        .and_then(|arr| arr.iter().find(|item| item["type"] == "tool_use"))
        .and_then(|item| item.get("input"))
        .cloned()
        .ok_or_else(|| format!("No tool_use block in response: {text}"))?;

    serde_json::to_string_pretty(&tool_input)
        .map_err(|e| format!("Failed to serialize tool input: {e}"))
}

/// Build Pass 1 prompts for JSON-UI v2 view generation (plain-text component plan).
///
/// Returns `(system_prompt, user_prompt)` ready for `call_anthropic_plain`.
pub fn build_json_view_pass1(name: &str, description: &str) -> (String, String) {
    let catalog = global_catalog();
    let catalog_prompt = catalog.prompt();

    let system = format!(
        "You are a JSON-UI v2 view planner for the Ferro framework.\n\n\
         {catalog_prompt}\n\n\
         Given a view name and description, produce a concise plain-text component plan: \
         which components to use, what data each displays, what actions are present. \
         Do not emit any JSON or code — only a human-readable plan."
    );

    let user = format!(
        "View name: {name}\n\
         Description: {description}\n\n\
         Describe the component plan for this view."
    );

    (system, user)
}

/// Build Pass 2 prompts for JSON-UI v2 view generation (structured spec).
///
/// Returns `(system_prompt, user_prompt)` ready for `call_anthropic_structured`.
/// Pass 2 receives the plain-text plan from Pass 1 and produces a structured JSON spec.
pub fn build_json_view_pass2(pass1_result: &str, _schema: &serde_json::Value) -> (String, String) {
    let system = format!(
        "You are a JSON-UI v2 spec generator for the Ferro framework.\n\n\
         Component plan from previous step:\n{pass1_result}\n\n\
         Generate the complete v2 JSON spec matching this plan. \
         Root element id must be \"root\". \
         All element ids are unique strings. Use flat elements map — no nesting."
    );

    let user =
        "Generate the complete JSON-UI v2 spec for the view described in the component plan."
            .to_string();

    (system, user)
}

/// Two-pass AI generation of a JSON-UI v2 spec file.
///
/// Pass 1 (describe): asks the model for a plain-text component plan.
/// Pass 2 (structure): constrains output to the full spec JSON Schema via tool_use.
///
/// On validation failure, prints a yellow warning and returns an error so the
/// caller can fall back to the static template.
pub fn generate_json_view(name: &str, description: &str, layout: &str) -> Result<String, String> {
    let catalog = global_catalog();
    let catalog_prompt = catalog.prompt();

    // Collect project context for user prompt
    let models = scan_models();
    let routes = scan_routes();

    let mut project_context = String::new();
    if !models.is_empty() {
        project_context.push_str("## Project Models\n");
        project_context.push_str(&models);
        project_context.push('\n');
    }
    if !routes.is_empty() {
        project_context.push_str("## Project Routes\n");
        project_context.push_str(&routes);
        project_context.push('\n');
    }

    // --- Pass 1: describe ---
    let pass1_system = format!(
        "You are a JSON-UI v2 view planner for the Ferro framework.\n\n\
         {catalog_prompt}\n\n\
         Given a view name, description, and project context, produce a concise plain-text \
         component plan: which components to use, what data each displays, what actions are \
         present. Do not emit any JSON or code — only a human-readable plan."
    );

    let pass1_user = format!(
        "{project_context}\
         View name: {name}\n\
         Layout: {layout}\n\
         Description: {description}\n\n\
         Describe the component plan for this view."
    );

    let pass1_result = call_anthropic_plain(&pass1_system, &pass1_user)?;

    // --- Pass 2: structure ---
    let schema = catalog.json_schema().clone();

    let pass2_system = format!(
        "You are a JSON-UI v2 spec generator for the Ferro framework.\n\n\
         Component plan from previous step:\n{pass1_result}\n\n\
         Generate the complete v2 JSON spec matching this plan. \
         Use layout \"{layout}\". Root element id must be \"root\". \
         All element ids are unique strings. Use flat elements map — no nesting."
    );

    let pass2_user = format!("Generate the complete JSON-UI v2 spec for the \"{name}\" view.");

    let spec_json = call_anthropic_structured(&pass2_system, &pass2_user, schema)?;

    // Validate against catalog
    match Spec::from_json(&spec_json) {
        Ok(spec) => {
            if let Err(errors) = catalog.validate(&spec) {
                let msgs: Vec<String> = errors.iter().map(|e| format!("  - {e}")).collect();
                eprintln!(
                    "{} Generated spec has validation errors:\n{}",
                    style("Warning:").yellow().bold(),
                    msgs.join("\n")
                );
                return Err("Spec validation failed — using static template.".to_string());
            }
        }
        Err(e) => {
            eprintln!(
                "{} Generated spec failed structural parse: {e}",
                style("Warning:").yellow().bold(),
            );
            return Err("Spec parse failed — using static template.".to_string());
        }
    }

    Ok(spec_json)
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
            .map(|n| format!(" (name: \"{n}\")"))
            .unwrap_or_default();

        output.push_str(&format!(
            "{} {} -> {}::{}{}\n",
            method, route.path, route.handler_module, route.handler_fn, name_suffix
        ));
    }

    output
}
