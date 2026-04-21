//! `ferro make:json-view` command implementation.
//!
//! Generates a JSON-UI v2 spec file (`src/views/{name}.json`), optionally using
//! the Anthropic API for AI-powered two-pass generation from a natural language
//! description. Handlers call `JsonUi::render_file("views/{name}.json", data)`.

use console::style;
use std::fs;
use std::path::Path;

use crate::ai;
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
        match std::env::var("ANTHROPIC_API_KEY") {
            Ok(_) => {
                let desc = description.as_deref().unwrap_or(&title);
                println!(
                    "{} Generating view with AI (two passes)...",
                    style("⏳").cyan()
                );
                generate_with_ai(&file_name, &title, layout_name, desc)
            }
            Err(_) => {
                if description.is_some() {
                    eprintln!(
                        "{} No ANTHROPIC_API_KEY found, using static template. \
                         Set the key or use --no-ai to suppress this message.",
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
/// Pass 1: plain-text component plan via `call_anthropic_plain`.
/// Pass 2: structured JSON via `call_anthropic_structured` + `catalog.json_schema()`.
/// On any failure (HTTP error, unparseable spec, catalog validation error), prints a
/// yellow warning to stderr and falls back to the static template.
fn generate_with_ai(
    file_name: &str,
    title: &str,
    layout_name: &str,
    description: &str,
) -> String {
    // ── Pass 1: plain-text plan ────────────────────────────────────────────
    let (sys1, usr1) = ai::build_json_view_pass1(file_name, description);
    let pass1_result = match ai::call_anthropic_plain(&sys1, &usr1) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{} AI Pass 1 failed: {}", style("Warning:").yellow().bold(), e);
            eprintln!("{}", style("Falling back to static template.").dim());
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    // ── Pass 2: structured spec ───────────────────────────────────────────
    let schema = ferro_json_ui::global_catalog().json_schema().clone();
    let (sys2, usr2) = ai::build_json_view_pass2(&pass1_result, &schema);
    let json_str = match ai::call_anthropic_structured(&sys2, &usr2, schema) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} AI Pass 2 failed: {}", style("Warning:").yellow().bold(), e);
            eprintln!("{}", style("Falling back to static template.").dim());
            return templates::json_view_template(file_name, title, layout_name);
        }
    };

    // ── Validation (D-03): Spec::from_json → global_catalog().validate ───
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
