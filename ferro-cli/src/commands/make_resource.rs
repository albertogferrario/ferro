use console::style;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(name: String, model: Option<String>) {
    // Ensure name ends with "Resource"
    let name = if name.ends_with("Resource") {
        name
    } else {
        format!("{}Resource", name)
    };

    // Convert to snake_case for file name
    let file_name = to_snake_case(&name);

    // Validate the resulting name is a valid Rust identifier
    if !is_valid_identifier(&file_name) {
        eprintln!(
            "{} '{}' is not a valid resource name",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    let resources_dir = Path::new("src/resources");
    let resource_file = resources_dir.join(format!("{}.rs", file_name));

    // Check if resources directory exists; create if not
    if !resources_dir.exists() {
        if let Err(e) = fs::create_dir_all(resources_dir) {
            eprintln!(
                "{} Failed to create resources directory: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/resources/", style("✓").green());
    }

    // Check if resource file already exists
    if resource_file.exists() {
        eprintln!(
            "{} Resource '{}' already exists at {}",
            style("Info:").yellow().bold(),
            file_name,
            resource_file.display()
        );
        std::process::exit(0);
    }

    // Generate resource file content
    let content = templates::resource_template(&name, model.as_deref());

    // Write resource file
    if let Err(e) = fs::write(&resource_file, content) {
        eprintln!(
            "{} Failed to write resource file: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!(
        "{} Created {}",
        style("✓").green(),
        resource_file.display()
    );

    println!();
    println!(
        "Resource {} created successfully!",
        style(&name).cyan().bold()
    );
    println!();
    println!(
        "  {} Add `pub mod {};` to src/resources/mod.rs",
        style("→").dim(),
        file_name
    );
    println!();
}

fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Rest must be alphanumeric or underscore
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
