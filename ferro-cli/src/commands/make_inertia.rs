use console::style;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(name: String) {
    // Convert to page name (PascalCase with "Page" suffix)
    let page_name = to_page_name(&name);

    // Validate the resulting name
    if !is_valid_component_name(&page_name) {
        eprintln!(
            "{} '{}' is not a valid page name",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    let pages_dir = Path::new("frontend/src/pages");
    let page_file = pages_dir.join(format!("{page_name}.tsx"));

    // Check if frontend/src/pages directory exists
    if !pages_dir.exists() {
        eprintln!(
            "{} Pages directory not found at frontend/src/pages",
            style("Error:").red().bold()
        );
        eprintln!(
            "{}",
            style("Make sure you're in a Ferro project root directory.").dim()
        );
        std::process::exit(1);
    }

    // Check if page file already exists
    if page_file.exists() {
        eprintln!(
            "{} Page '{}' already exists at {}",
            style("Info:").yellow().bold(),
            page_name,
            page_file.display()
        );
        std::process::exit(0);
    }

    // Generate page file content
    let page_content = templates::inertia_page_template(&page_name);

    // Write page file
    if let Err(e) = fs::write(&page_file, page_content) {
        eprintln!(
            "{} Failed to write page file: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!("{} Created {}", style("✓").green(), page_file.display());

    println!();
    println!(
        "Page {} created successfully!",
        style(&page_name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!("  {} Use the page in a controller:", style("1.").dim());
    println!("     inertia_response!(\"{page_name}\", props)");
    println!();
}

fn is_valid_component_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be uppercase letter
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {}
        _ => return false,
    }

    // Rest must be alphanumeric
    chars.all(|c| c.is_alphanumeric())
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn to_page_name(input: &str) -> String {
    // Convert to PascalCase
    let pascal = to_pascal_case(input);

    // Append "Page" if not already present
    if pascal.ends_with("Page") {
        pascal
    } else {
        format!("{pascal}Page")
    }
}
