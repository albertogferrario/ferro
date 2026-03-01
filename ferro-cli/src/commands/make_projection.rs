use console::style;
use std::fs;
use std::path::Path;

pub fn execute(name: &str) {
    let file_name = to_snake_case(name);

    if !is_valid_identifier(&file_name) {
        eprintln!(
            "{} '{}' is not a valid projection name",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    let display_name = to_pascal_case(name);
    let projections_dir = Path::new("src/projections");
    let projection_file = projections_dir.join(format!("{file_name}.rs"));
    let mod_file = projections_dir.join("mod.rs");

    // Create projections directory if it doesn't exist
    if !projections_dir.exists() {
        if let Err(e) = fs::create_dir_all(projections_dir) {
            eprintln!(
                "{} Failed to create src/projections directory: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/projections/", style("✓").green());
    }

    // Check if projection file already exists
    if projection_file.exists() {
        eprintln!(
            "{} Projection '{}' already exists at {}",
            style("Info:").yellow().bold(),
            file_name,
            projection_file.display()
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
                "{} Module '{}' is already declared in src/projections/mod.rs",
                style("Info:").yellow().bold(),
                file_name
            );
            std::process::exit(0);
        }
    }

    // Generate projection file content
    let content = projection_template(&file_name, &display_name);

    // Write projection file
    if let Err(e) = fs::write(&projection_file, content) {
        eprintln!(
            "{} Failed to write projection file: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!(
        "{} Created {}",
        style("✓").green(),
        projection_file.display()
    );

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
        println!("{} Updated src/projections/mod.rs", style("✓").green());
    } else {
        let mod_content = format!("pub mod {file_name};\n");
        if let Err(e) = fs::write(&mod_file, mod_content) {
            eprintln!(
                "{} Failed to create mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
        println!("{} Created src/projections/mod.rs", style("✓").green());
    }

    println!();
    println!(
        "Projection {} created successfully!",
        style(&file_name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!(
        "  {} Define fields matching your model in src/projections/{file_name}.rs",
        style("1.").dim()
    );
    println!("  {} Use in a handler:", style("2.").dim());
    println!("     use crate::projections::{file_name};");
    println!();
    println!("     let service = {file_name}::{file_name}_service();");
    println!("     let intents = derive_intents(&service);");
    println!();
}

fn projection_template(name: &str, display_name: &str) -> String {
    format!(
        r#"use ferro::{{
    DataType, FieldMeaning, ServiceDef,
}};

/// Build the {display_name} service projection.
///
/// Describes the {display_name} entity's fields, relationships,
/// and behavioral semantics for intent derivation and UI rendering.
pub fn {name}_service() -> ServiceDef {{
    ServiceDef::new("{name}")
        .display_name("{display_name}")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        // Add fields matching your model:
        // .field("name", DataType::String, FieldMeaning::EntityName)
        // .field("email", DataType::String, FieldMeaning::Email)
        // .field("status", DataType::String, FieldMeaning::Status)
        // .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        // .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
}}
"#
    )
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

/// Generate projection files at the given base directory.
/// Returns (projection_file_path, mod_file_path) on success.
#[cfg(test)]
fn generate_in_dir(
    base_dir: &Path,
    name: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    let file_name = to_snake_case(name);
    let display_name = to_pascal_case(name);
    let projections_dir = base_dir.join("src/projections");
    let projection_file = projections_dir.join(format!("{file_name}.rs"));
    let mod_file = projections_dir.join("mod.rs");

    fs::create_dir_all(&projections_dir)
        .map_err(|e| format!("Failed to create projections directory: {e}"))?;

    let content = projection_template(&file_name, &display_name);
    fs::write(&projection_file, content)
        .map_err(|e| format!("Failed to write projection file: {e}"))?;

    if mod_file.exists() {
        let mod_content = fs::read_to_string(&mod_file).unwrap_or_default();
        let pub_mod_decl = format!("pub mod {file_name};");
        if !mod_content.contains(&pub_mod_decl) {
            update_mod_file(&mod_file, &file_name)?;
        }
    } else {
        let mod_content = format!("pub mod {file_name};\n");
        fs::write(&mod_file, mod_content).map_err(|e| format!("Failed to create mod.rs: {e}"))?;
    }

    Ok((projection_file, mod_file))
}

fn update_mod_file(mod_file: &Path, file_name: &str) -> Result<(), String> {
    let content =
        fs::read_to_string(mod_file).map_err(|e| format!("Failed to read mod.rs: {e}"))?;

    let pub_mod_decl = format!("pub mod {file_name};");

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
    fs::write(mod_file, new_content).map_err(|e| format!("Failed to write mod.rs: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_generation() {
        let template = projection_template("user", "User");

        assert!(template.contains("pub fn user_service() -> ServiceDef"));
        assert!(template.contains("ServiceDef::new(\"user\")"));
        assert!(template.contains(".display_name(\"User\")"));
        assert!(template.contains("DataType::Integer, FieldMeaning::Identifier"));
        assert!(template.contains("use ferro::{"));
        assert!(template.contains("/// Build the User service projection."));
    }

    #[test]
    fn test_creates_directory_and_file() {
        let tmp = TempDir::new().unwrap();
        let (proj_file, _mod_file) = generate_in_dir(tmp.path(), "order").unwrap();

        assert!(tmp.path().join("src/projections").exists());
        assert!(proj_file.exists());

        let content = fs::read_to_string(&proj_file).unwrap();
        assert!(content.contains("pub fn order_service() -> ServiceDef"));
        assert!(content.contains(".display_name(\"Order\")"));
    }

    #[test]
    fn test_mod_rs_creation() {
        let tmp = TempDir::new().unwrap();
        let (_proj_file, mod_file) = generate_in_dir(tmp.path(), "product").unwrap();

        assert!(mod_file.exists());
        let mod_content = fs::read_to_string(&mod_file).unwrap();
        assert!(mod_content.contains("pub mod product;"));
    }

    #[test]
    fn test_mod_rs_append() {
        let tmp = TempDir::new().unwrap();

        // First projection creates mod.rs
        generate_in_dir(tmp.path(), "user").unwrap();
        let mod_file = tmp.path().join("src/projections/mod.rs");
        let content = fs::read_to_string(&mod_file).unwrap();
        assert!(content.contains("pub mod user;"));

        // Second projection appends to mod.rs without duplicating
        generate_in_dir(tmp.path(), "order").unwrap();
        let content = fs::read_to_string(&mod_file).unwrap();
        assert!(content.contains("pub mod user;"));
        assert!(content.contains("pub mod order;"));

        // Third call with same name should not duplicate
        generate_in_dir(tmp.path(), "order").unwrap();
        let content = fs::read_to_string(&mod_file).unwrap();
        let count = content.matches("pub mod order;").count();
        assert_eq!(count, 1, "pub mod order; should appear exactly once");
    }
}
