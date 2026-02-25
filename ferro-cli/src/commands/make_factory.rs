use console::style;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(name: String) {
    // Convert to PascalCase for struct name
    let struct_name = to_pascal_case(&name);

    // Append "Factory" suffix if not already present
    let struct_name = if struct_name.ends_with("Factory") {
        struct_name
    } else {
        format!("{struct_name}Factory")
    };

    // Extract model name (remove Factory suffix)
    let model_name = struct_name
        .strip_suffix("Factory")
        .unwrap_or(&struct_name)
        .to_string();

    // Convert to snake_case for file name
    let file_name = to_snake_case(&struct_name);

    // Validate the resulting name is a valid Rust identifier
    if !is_valid_identifier(&file_name) {
        eprintln!(
            "{} '{}' is not a valid factory name",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    let factories_dir = Path::new("src/factories");
    let factory_file = factories_dir.join(format!("{file_name}.rs"));
    let mod_file = factories_dir.join("mod.rs");

    // Create factories directory if it doesn't exist
    if !factories_dir.exists() {
        if let Err(e) = fs::create_dir_all(factories_dir) {
            eprintln!(
                "{} Failed to create factories directory: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/factories directory", style("✓").green());
    }

    // Check if factory file already exists
    if factory_file.exists() {
        eprintln!(
            "{} Factory '{}' already exists at {}",
            style("Info:").yellow().bold(),
            struct_name,
            factory_file.display()
        );
        std::process::exit(0);
    }

    // Check if module is already declared in mod.rs
    if mod_file.exists() {
        let mod_content = fs::read_to_string(&mod_file).unwrap_or_default();
        let mod_decl = format!("mod {file_name};");
        let pub_mod_decl = format!("pub mod {file_name};");
        if mod_content.contains(&mod_decl) || mod_content.contains(&pub_mod_decl) {
            eprintln!(
                "{} Module '{}' is already declared in src/factories/mod.rs",
                style("Info:").yellow().bold(),
                file_name
            );
            std::process::exit(0);
        }
    }

    // Generate factory file content
    let factory_content = templates::factory_template(&file_name, &struct_name, &model_name);

    // Write factory file
    if let Err(e) = fs::write(&factory_file, factory_content) {
        eprintln!(
            "{} Failed to write factory file: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!("{} Created {}", style("✓").green(), factory_file.display());

    // Update or create mod.rs
    if mod_file.exists() {
        if let Err(e) = update_mod_file(&mod_file, &file_name) {
            eprintln!(
                "{} Failed to update mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Updated src/factories/mod.rs", style("✓").green());
    } else {
        // Create mod.rs with template content
        let mut mod_content = templates::factories_mod().to_string();
        mod_content.push_str(&format!("pub mod {file_name};\n"));
        if let Err(e) = fs::write(&mod_file, mod_content) {
            eprintln!(
                "{} Failed to create mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/factories/mod.rs", style("✓").green());
    }

    println!();
    println!(
        "Factory {} created successfully!",
        style(&struct_name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!(
        "  {} Make without persisting (in tests):",
        style("1.").dim()
    );
    println!("     let model = {struct_name}::factory().make();");
    println!();
    println!("  {} Create with database persistence:", style("2.").dim());
    println!("     let model = {struct_name}::factory().create().await?;");
    println!();
    println!("  {} Apply named traits:", style("3.").dim());
    println!("     let admin = {struct_name}::factory().trait_(\"admin\").create().await?;");
    println!();
    println!("{}", style("Note:").yellow().bold());
    println!("  Update the factory struct to match your {model_name} model fields,");
    println!("  then uncomment the DatabaseFactory impl for database persistence.");
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

fn update_mod_file(mod_file: &Path, file_name: &str) -> Result<(), String> {
    let content =
        fs::read_to_string(mod_file).map_err(|e| format!("Failed to read mod.rs: {e}"))?;

    let pub_mod_decl = format!("pub mod {file_name};");

    // Find position to insert pub mod declaration (after other pub mod declarations)
    let mut lines: Vec<&str> = content.lines().collect();

    // Find the last pub mod declaration line
    let mut last_pub_mod_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("pub mod ") {
            last_pub_mod_idx = Some(i);
        }
    }

    // Insert pub mod declaration
    let insert_idx = match last_pub_mod_idx {
        Some(idx) => idx + 1,
        None => {
            // If no pub mod declarations, insert at the end (after any doc comments)
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
    fs::write(mod_file, new_content).map_err(|e| format!("Failed to write mod.rs: {e}"))?;

    Ok(())
}
