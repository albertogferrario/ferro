//! `ferro make:json-view` command implementation.
//!
//! Generates a JSON-UI v2 spec file (`src/views/{name}.json`), optionally using
//! an AI provider for two-pass generation from a natural language description.
//! Handlers call `JsonUi::render_file("views/{name}.json", data)`.

use console::style;
use ferro_ai::client::{Message, Role};
use ferro_ai::{AiConfig, CompletionRequest};
use ferro_json_ui::global_catalog;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(name: String, description: Option<String>, no_ai: bool, layout: Option<String>) {
    let file_name = to_snake_case(&name);

    if !is_valid_identifier(&file_name) {
        eprintln!(
            "{} '{}' is not a valid view name",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    let views_dir = Path::new("src/views");
    let view_file = views_dir.join(format!("{file_name}.json"));

    // Create views/ directory if missing
    if !views_dir.exists() {
        if let Err(e) = fs::create_dir_all(views_dir) {
            eprintln!(
                "{} Failed to create src/views directory: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/views/", style("✓").green());
    }

    // If the JSON view already exists, skip (non-destructive)
    if view_file.exists() {
        eprintln!(
            "{} View '{}' already exists at {}",
            style("Info:").yellow().bold(),
            file_name,
            view_file.display()
        );
        std::process::exit(0);
    }

    let layout_name = layout.as_deref().unwrap_or("dashboard");
    let title = to_title_case(&file_name);

    let content = if no_ai {
        templates::json_view_template(&file_name, &title, layout_name)
    } else {
        match AiConfig::from_env() {
            Ok(client) => {
                let desc = description.as_deref().unwrap_or(&title);
                println!(
                    "{} Generating view with AI (two passes)...",
                    style("⏳").cyan()
                );
                generate_with_ai(client.as_ref(), &file_name, &title, layout_name, desc)
            }
            Err(_) => {
                if description.is_some() {
                    eprintln!(
                        "{} No AI provider configured. Set FERRO_AI_API_KEY (and optionally \
                         FERRO_AI_PROVIDER / FERRO_AI_MODEL), or use --no-ai to suppress this message.",
                        style("Info:").yellow().bold(),
                    );
                }
                templates::json_view_template(&file_name, &title, layout_name)
            }
        }
    };

    if let Err(e) = fs::write(&view_file, content) {
        eprintln!(
            "{} Failed to write view file: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!("{} Created {}", style("✓").green(), view_file.display());

    // Usage guidance — v2 handler pattern
    println!();
    println!(
        "View {} created successfully!",
        style(&file_name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!("  {} Serve the view from a handler:", style("1.").dim());
    println!();
    println!("     #[handler]");
    println!("     pub async fn {file_name}(req: Request) -> Response {{");
    println!("         let data = serde_json::json!({{}});");
    println!("         JsonUi::render_file(\"views/{file_name}.json\", data)");
    println!("     }}");
    println!();
}

/// Orchestrate two-pass AI generation with catalog validation and static fallback.
///
/// Pass 1: plain-text component plan via `client.complete` (schema: None).
/// Pass 2: structured JSON via `client.complete` + `global_catalog().json_schema()`.
/// On any failure (runtime, HTTP error, unparseable spec, catalog validation error),
/// prints a yellow warning to stderr and falls back to the static template.
fn generate_with_ai(
    client: &dyn ferro_ai::LlmClient,
    file_name: &str,
    title: &str,
    layout_name: &str,
    description: &str,
) -> String {
    // One runtime, reused across both passes (D-01). ferro-cli main() is sync (no #[tokio::main]),
    // so Runtime::new() is safe — no nested-runtime panic.
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "{} Failed to create tokio runtime: {}",
                style("Warning:").yellow().bold(),
                e
            );
            eprintln!("{}", style("Falling back to static template.").dim());
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    // ── Pass 1: plain-text plan (schema: None, max_tokens 1024) ──────────────
    let (sys1, usr1) = build_json_view_pass1(file_name, description);
    let req1 = CompletionRequest {
        system: Some(sys1),
        messages: vec![Message {
            role: Role::User,
            content: usr1,
            tool_call_id: None,
        }],
        max_tokens: 1024,
        model_override: None,
        schema: None,
        tools: None,
        tool_choice: None,
    };
    let pass1_result = match rt.block_on(client.complete(req1)) {
        Ok(text) => text,
        Err(e) => {
            eprintln!(
                "{} AI Pass 1 failed: {}",
                style("Warning:").yellow().bold(),
                e
            );
            eprintln!("{}", style("Falling back to static template.").dim());
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    // ── Pass 2: structured spec against the catalog schema (max_tokens 4096) ─
    let (sys2, usr2) = build_json_view_pass2(&pass1_result);
    let schema = ferro_json_ui::global_catalog().json_schema().clone();
    let req2 = CompletionRequest {
        system: Some(sys2),
        messages: vec![Message {
            role: Role::User,
            content: usr2,
            tool_call_id: None,
        }],
        max_tokens: 4096,
        model_override: None,
        // Catalog runtime schema is the validation source of truth (D-02) — NOT schemars.
        schema: Some(schema),
        tools: None,
        tool_choice: None,
    };
    let json_str = match rt.block_on(client.complete(req2)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{} AI Pass 2 failed: {}",
                style("Warning:").yellow().bold(),
                e
            );
            eprintln!("{}", style("Falling back to static template.").dim());
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    // ── Validation (D-03): unchanged from the current implementation ─────────
    match ferro_json_ui::Spec::from_json(&json_str) {
        Err(parse_err) => {
            eprintln!(
                "{} Generated spec failed structural parse: {}",
                style("Warning:").yellow().bold(),
                parse_err
            );
            eprintln!("{}", style("Falling back to static template.").dim());
            templates::json_view_template(file_name, title, layout_name)
        }
        Ok(spec) => match ferro_json_ui::global_catalog().validate(&spec) {
            Ok(()) => json_str,
            Err(errors) => {
                eprintln!(
                    "{} Generated spec failed catalog validation ({} error{}):",
                    style("Warning:").yellow().bold(),
                    errors.len(),
                    if errors.len() == 1 { "" } else { "s" }
                );
                for err in &errors {
                    eprintln!("  - {err}");
                }
                eprintln!("{}", style("Falling back to static template.").dim());
                templates::json_view_template(file_name, title, layout_name)
            }
        },
    }
}

/// Build Pass 1 prompts for JSON-UI v2 view generation (plain-text component plan).
///
/// Returns `(system_prompt, user_prompt)` ready for `client.complete` with `schema: None`.
fn build_json_view_pass1(name: &str, description: &str) -> (String, String) {
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
/// Returns `(system_prompt, user_prompt)` ready for `client.complete` with the catalog schema.
/// Pass 2 receives the plain-text plan from Pass 1 and produces a structured JSON spec.
fn build_json_view_pass2(pass1_result: &str) -> (String, String) {
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

fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }

    chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_snake_case_basic() {
        assert_eq!(to_snake_case("UserList"), "user_list");
        assert_eq!(to_snake_case("dashboard"), "dashboard");
    }

    #[test]
    fn to_title_case_basic() {
        assert_eq!(to_title_case("user_list"), "User List");
        assert_eq!(to_title_case("dashboard"), "Dashboard");
    }

    #[test]
    fn is_valid_identifier_accepts_snake_case() {
        assert!(is_valid_identifier("user_list"));
        assert!(is_valid_identifier("dashboard"));
    }

    #[test]
    fn is_valid_identifier_rejects_invalid() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("1bad"));
        assert!(!is_valid_identifier("has-dash"));
    }

    // Integration-ish: the fallback path writes a parseable spec.
    // We do NOT invoke `run` here because it calls std::process::exit.
    // Instead we exercise the static template path directly.
    #[test]
    fn static_fallback_produces_valid_spec() {
        let out = crate::templates::json_view_template("dashboard", "Dashboard", "dashboard");
        let spec = ferro_json_ui::Spec::from_json(&out);
        assert!(spec.is_ok(), "static fallback must parse: {spec:?}");
    }
}
