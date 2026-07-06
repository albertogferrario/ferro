//! `ferro make:json-view` command implementation.
//!
//! Generates a JSON-UI v2 spec file (`src/views/{name}.json`) from a `ServiceDef`
//! via the deterministic `Spec::from_service_def` renderer. Two ServiceDef sources
//! are supported:
//!
//! - NL description (`-d "<text>"`) → `scaffold_core` → `ServiceDef`
//! - Pre-serialized JSON file (`--from-service-json <path>`) → `ServiceDef`
//!
//! Handlers call `JsonUi::render_file("views/{name}.json", data)`.

use console::style;
use ferro_ai::AiConfig;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(
    name: String,
    description: Option<String>,
    no_ai: bool,
    layout: Option<String>,
    from_service_json: Option<String>,
) {
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
    } else if let Some(ref _path) = from_service_json {
        // --from-service-json path: deserialize ServiceDef from JSON file, render deterministically.
        #[cfg(feature = "projections")]
        {
            let path = _path;
            let json_content = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "{} Failed to read service JSON file '{}': {}",
                        style("Error:").red().bold(),
                        path,
                        e
                    );
                    std::process::exit(1);
                }
            };
            let service: ferro_projections::ServiceDef = match serde_json::from_str(&json_content) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "{} Failed to parse ServiceDef from '{}': {}",
                        style("Error:").red().bold(),
                        path,
                        e
                    );
                    std::process::exit(1);
                }
            };
            println!(
                "{} Rendering ServiceDef from {} ...",
                style("⏳").cyan(),
                path
            );
            render_service_def(&service, &file_name, &title, layout_name)
        }
        #[cfg(not(feature = "projections"))]
        {
            eprintln!(
                "{} make:json-view --from-service-json requires the `projections` feature",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    } else {
        match AiConfig::from_env() {
            Ok(_) => {
                let desc = description.as_deref().unwrap_or(&title);
                #[cfg(feature = "projections")]
                {
                    // NL → ServiceDef via scaffold_core, then deterministic render.
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!(
                                "{} Failed to create tokio runtime: {}",
                                style("Warning:").yellow().bold(),
                                e
                            );
                            eprintln!("{}", style("Falling back to static template.").dim());
                            return write_content(
                                &view_file,
                                templates::json_view_template(&file_name, &title, layout_name),
                                &file_name,
                            );
                        }
                    };
                    let cwd =
                        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    println!("{} Generating ServiceDef via AI...", style("⏳").cyan());
                    let desc_owned = desc.to_string();
                    match rt.block_on(ferro_mcp::tools::ai_scaffold::scaffold_core(
                        &desc_owned,
                        &cwd,
                    )) {
                        Ok(service) => {
                            println!("{} Rendering projection spec...", style("⏳").cyan());
                            render_service_def(&service, &file_name, &title, layout_name)
                        }
                        Err(e) => {
                            eprintln!(
                                "{} AI scaffold failed: {}",
                                style("Warning:").yellow().bold(),
                                e
                            );
                            eprintln!("{}", style("Falling back to static template.").dim());
                            templates::json_view_template(&file_name, &title, layout_name)
                        }
                    }
                }
                #[cfg(not(feature = "projections"))]
                {
                    // projections feature disabled — note and fall back.
                    let _ = desc;
                    eprintln!(
                        "{} AI generation requires the `projections` feature. Using static template.",
                        style("Info:").yellow().bold()
                    );
                    templates::json_view_template(&file_name, &title, layout_name)
                }
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

    write_content(&view_file, content, &file_name);
}

/// Write spec content to the view file and print usage guidance.
fn write_content(view_file: &Path, content: String, file_name: &str) {
    if let Err(e) = fs::write(view_file, content) {
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
        style(file_name).cyan().bold()
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

/// Render a `ServiceDef` deterministically to a JSON-UI v2 spec string.
///
/// Uses `derive_intents` + `Spec::from_service_def` (FieldMeaning-driven component
/// dispatch). On any render/serialize/parse failure, prints a yellow warning and
/// returns the static template fallback.
#[cfg(feature = "projections")]
fn render_service_def(
    service: &ferro_projections::ServiceDef,
    file_name: &str,
    title: &str,
    layout_name: &str,
) -> String {
    use ferro_json_ui::{Spec, VisualContext};
    use ferro_projections::derive_intents;

    let intents = derive_intents(service);
    let ctx = VisualContext::default();
    match Spec::from_service_def(service, &intents, &ctx) {
        Err(e) => {
            eprintln!(
                "{} Projection render failed: {e}",
                style("Warning:").yellow().bold()
            );
            eprintln!("{}", style("Falling back to static template.").dim());
            templates::json_view_template(file_name, title, layout_name)
        }
        Ok(spec) => match serde_json::to_string_pretty(&spec) {
            Err(e) => {
                eprintln!(
                    "{} Spec serialization failed: {e}",
                    style("Warning:").yellow().bold()
                );
                eprintln!("{}", style("Falling back to static template.").dim());
                templates::json_view_template(file_name, title, layout_name)
            }
            Ok(json_str) => {
                // Write-gate re-parse (D-02): validate the serialized JSON form.
                match Spec::from_json(&json_str) {
                    Err(e) => {
                        eprintln!(
                            "{} Spec parse failed: {e}",
                            style("Warning:").yellow().bold()
                        );
                        eprintln!("{}", style("Falling back to static template.").dim());
                        templates::json_view_template(file_name, title, layout_name)
                    }
                    Ok(_) => json_str,
                }
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
