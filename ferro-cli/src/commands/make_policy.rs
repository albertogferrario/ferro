use console::style;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(name: String, model: Option<String>) {
    // Validate name is a valid Rust identifier
    if !is_valid_identifier(&name) {
        eprintln!(
            "{} '{}' is not a valid Rust identifier",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    // Convert name to struct name and file name
    // e.g., "Post" -> "PostPolicy", "post_policy"
    // e.g., "PostPolicy" -> "PostPolicy", "post_policy"
    let struct_name = if name.ends_with("Policy") {
        name.clone()
    } else {
        format!("{name}Policy")
    };
    let file_name = to_snake_case(name.trim_end_matches("Policy"));

    // Derive model name from policy name if not provided
    // e.g., "PostPolicy" -> "Post"
    let model_name = model.unwrap_or_else(|| {
        let base = name.trim_end_matches("Policy");
        to_pascal_case(base)
    });

    let policies_dir = Path::new("src/policies");
    let policy_file = policies_dir.join(format!("{file_name}_policy.rs"));
    let mod_file = policies_dir.join("mod.rs");

    // Check if policies directory exists
    if !policies_dir.exists() {
        if let Err(e) = fs::create_dir_all(policies_dir) {
            eprintln!(
                "{} Failed to create policies directory: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/policies directory", style("✓").green());
    }

    // Check if policy file already exists
    if policy_file.exists() {
        eprintln!(
            "{} Policy '{}' already exists at {}",
            style("Error:").red().bold(),
            struct_name,
            policy_file.display()
        );
        std::process::exit(1);
    }

    // Generate policy file content
    let policy_content = templates::policy_template(&file_name, &struct_name, &model_name);

    // Write policy file
    if let Err(e) = fs::write(&policy_file, policy_content) {
        eprintln!(
            "{} Failed to write policy file: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!("{} Created {}", style("✓").green(), policy_file.display());

    // Update mod.rs
    let module_name = format!("{file_name}_policy");
    if mod_file.exists() {
        if let Err(e) = update_mod_file(&mod_file, &module_name, &struct_name) {
            eprintln!(
                "{} Failed to update mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Updated src/policies/mod.rs", style("✓").green());
    } else {
        // Create mod.rs if it doesn't exist
        let mod_content = format!(
            "{}mod {};\n\npub use {}::{};\n",
            templates::policies_mod(),
            module_name,
            module_name,
            struct_name
        );
        if let Err(e) = fs::write(&mod_file, mod_content) {
            eprintln!(
                "{} Failed to create mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/policies/mod.rs", style("✓").green());
    }

    println!();
    println!(
        "Policy {} created successfully!",
        style(&struct_name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!(
        "  {} Import your model and user types in the policy file",
        style("1.").dim()
    );
    println!(
        "  {} Implement the authorization logic in each method",
        style("2.").dim()
    );
    println!();
    println!("Example:");
    println!("  use crate::policies::{struct_name};");
    println!("  use ferro::authorization::Policy;");
    println!();
    println!("  let policy = {struct_name};");
    println!("  if policy.update(&user, &model).allowed() {{");
    println!("      // Proceed with update");
    println!("  }}");
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

fn update_mod_file(mod_file: &Path, file_name: &str, struct_name: &str) -> Result<(), String> {
    let content =
        fs::read_to_string(mod_file).map_err(|e| format!("Failed to read mod.rs: {e}"))?;

    // Check if module already declared
    let mod_decl = format!("mod {file_name};");
    if content.contains(&mod_decl) {
        return Err(format!("Module '{file_name}' already declared in mod.rs"));
    }

    // Find position to insert mod declaration (after other mod declarations)
    let mut lines: Vec<&str> = content.lines().collect();

    // Find the last mod declaration line
    let mut last_mod_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("mod ") {
            last_mod_idx = Some(i);
        }
    }

    // Insert mod declaration
    let mod_insert_idx = match last_mod_idx {
        Some(idx) => idx + 1,
        None => {
            // If no mod declarations, insert after doc comments
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
    lines.insert(mod_insert_idx, &mod_decl);

    // Find position to insert pub use (after other pub use declarations)
    let pub_use_decl = format!("pub use {file_name}::{struct_name};");
    let mut last_pub_use_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("pub use ") {
            last_pub_use_idx = Some(i);
        }
    }

    // Insert pub use declaration
    match last_pub_use_idx {
        Some(idx) => {
            lines.insert(idx + 1, &pub_use_decl);
        }
        None => {
            // If no pub use declarations, add after mod declarations with empty line
            let mut insert_idx = mod_insert_idx + 1;
            // Skip past remaining mod declarations
            while insert_idx < lines.len() && lines[insert_idx].trim().starts_with("mod ") {
                insert_idx += 1;
            }
            // Add empty line if needed
            if insert_idx < lines.len() && !lines[insert_idx].is_empty() {
                lines.insert(insert_idx, "");
                insert_idx += 1;
            }
            lines.insert(insert_idx, &pub_use_decl);
        }
    }

    let new_content = lines.join("\n");
    fs::write(mod_file, new_content).map_err(|e| format!("Failed to write mod.rs: {e}"))?;

    Ok(())
}
