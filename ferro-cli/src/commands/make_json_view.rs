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

    // Create views directory if it doesn't exist
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

    // Check if view file already exists
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

    // Determine content: AI (two-pass) or static template
    let content = if no_ai {
        templates::json_view_template(&file_name, &title, layout_name)
    } else {
        match std::env::var("ANTHROPIC_API_KEY") {
            Ok(_) => {
                let desc = description.as_deref().unwrap_or(&title);
                println!("{} Generating view with AI...", style("⏳").cyan());

                match ai::generate_json_view(&file_name, desc, layout_name) {
                    Ok(spec_json) => spec_json,
                    Err(e) => {
                        eprintln!(
                            "{} AI generation failed: {}",
                            style("Warning:").yellow().bold(),
                            e
                        );
                        eprintln!("{}", style("Falling back to static template.").dim());
                        templates::json_view_template(&file_name, &title, layout_name)
                    }
                }
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

    // Write view file
    if let Err(e) = fs::write(&view_file, content) {
        eprintln!(
            "{} Failed to write view file: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!("{} Created {}", style("✓").green(), view_file.display());

    println!();
    println!(
        "View {} created successfully!",
        style(&file_name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!("  {} Use the view in a handler:", style("1.").dim());
    println!();
    println!("     use ferro::{{JsonUi, Response}};");
    println!();
    println!("     #[handler]");
    println!("     pub async fn {file_name}(req: Request) -> Response {{");
    println!("         let data = serde_json::json!({{}});");
    println!("         JsonUi::render_file(\"views/{file_name}.json\", data)");
    println!("     }}");
    println!();
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
