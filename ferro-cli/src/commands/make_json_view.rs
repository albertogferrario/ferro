//! `ferro make:json-view` command implementation.
//!
//! Generates a JSON-UI view file, optionally using the Anthropic API
//! for AI-powered generation from a natural language description.

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
    let view_file = views_dir.join(format!("{}.rs", file_name));
    let mod_file = views_dir.join("mod.rs");

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

    // Check if module is already declared in mod.rs
    if mod_file.exists() {
        let mod_content = fs::read_to_string(&mod_file).unwrap_or_default();
        let mod_decl = format!("mod {};", file_name);
        let pub_mod_decl = format!("pub mod {};", file_name);
        if mod_content.contains(&mod_decl) || mod_content.contains(&pub_mod_decl) {
            eprintln!(
                "{} Module '{}' is already declared in src/views/mod.rs",
                style("Info:").yellow().bold(),
                file_name
            );
            std::process::exit(0);
        }
    }

    let layout_name = layout.as_deref().unwrap_or("app");
    let title = to_title_case(&file_name);

    // Determine content: AI or static template
    let content = if no_ai {
        templates::json_view_template(&file_name, &title, layout_name)
    } else {
        match std::env::var("ANTHROPIC_API_KEY") {
            Ok(_) => {
                let desc = description.as_deref().unwrap_or(&title);
                println!("{} Generating view with AI...", style("⏳").cyan());

                let prompt = ai::build_view_context(&file_name, desc);

                match ai::call_anthropic(&prompt) {
                    Ok(response) => strip_markdown_fences(&response),
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

    // Update mod.rs
    if mod_file.exists() {
        if let Err(e) = update_mod_file(&mod_file, &file_name) {
            eprintln!(
                "{} Failed to update mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Updated src/views/mod.rs", style("✓").green());
    } else {
        let mod_content = format!("pub mod {};\n", file_name);
        if let Err(e) = fs::write(&mod_file, mod_content) {
            eprintln!(
                "{} Failed to create mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/views/mod.rs", style("✓").green());
    }

    println!();
    println!(
        "View {} created successfully!",
        style(&file_name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!("  {} Use the view in a handler:", style("1.").dim());
    println!("     use crate::views::{};", file_name);
    println!();
    println!("     pub async fn index() -> Response {{");
    println!(
        "         JsonUi::render(&{}::view(), &json!({{}}))",
        file_name
    );
    println!("     }}");
    println!();
}

/// Strip markdown code fences if present in AI response.
fn strip_markdown_fences(s: &str) -> String {
    let trimmed = s.trim();

    // Check for ```rust or ``` at the start and remove trailing ```
    let stripped = if let Some(rest) = trimmed.strip_prefix("```rust") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest
    } else {
        return trimmed.to_string();
    };

    let stripped = stripped.trim();
    if let Some(inner) = stripped.strip_suffix("```") {
        inner.trim().to_string()
    } else {
        stripped.to_string()
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

fn update_mod_file(mod_file: &Path, file_name: &str) -> Result<(), String> {
    let content =
        fs::read_to_string(mod_file).map_err(|e| format!("Failed to read mod.rs: {}", e))?;

    let pub_mod_decl = format!("pub mod {};", file_name);

    let mut lines: Vec<&str> = content.lines().collect();

    // Find the last pub mod declaration line
    let mut last_pub_mod_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("pub mod ") {
            last_pub_mod_idx = Some(i);
        }
    }

    let insert_idx = match last_pub_mod_idx {
        Some(idx) => idx + 1,
        None => {
            let mut insert_idx = 0;
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with("//!") || line.is_empty() {
                    insert_idx = i + 1;
                } else {
                    break;
                }
            }
            insert_idx
        }
    };
    lines.insert(insert_idx, &pub_mod_decl);

    let new_content = lines.join("\n");
    fs::write(mod_file, new_content).map_err(|e| format!("Failed to write mod.rs: {}", e))?;

    Ok(())
}
