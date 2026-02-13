//! make:lang command — scaffold translation files for a new locale

use console::style;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(name: String) {
    // Validate locale code: lowercase letters, optionally hyphen + more lowercase letters
    if !is_valid_locale(&name) {
        eprintln!(
            "{} '{}' is not a valid locale code (expected format: en, fr, pt-br, zh-hans)",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    let lang_dir = Path::new("lang").join(&name);

    // Check if locale directory already exists
    if lang_dir.exists() {
        eprintln!(
            "{} Language directory '{}' already exists",
            style("Error:").red().bold(),
            lang_dir.display()
        );
        std::process::exit(1);
    }

    // Create locale directory
    if let Err(e) = fs::create_dir_all(&lang_dir) {
        eprintln!(
            "{} Failed to create directory {}: {}",
            style("Error:").red().bold(),
            lang_dir.display(),
            e
        );
        std::process::exit(1);
    }

    // Write validation.json
    let validation_path = lang_dir.join("validation.json");
    if let Err(e) = fs::write(&validation_path, templates::lang_validation_json()) {
        eprintln!(
            "{} Failed to write {}: {}",
            style("Error:").red().bold(),
            validation_path.display(),
            e
        );
        std::process::exit(1);
    }
    println!(
        "{} Created {}",
        style("✓").green(),
        validation_path.display()
    );

    // Write app.json
    let app_path = lang_dir.join("app.json");
    if let Err(e) = fs::write(&app_path, templates::lang_app_json()) {
        eprintln!(
            "{} Failed to write {}: {}",
            style("Error:").red().bold(),
            app_path.display(),
            e
        );
        std::process::exit(1);
    }
    println!("{} Created {}", style("✓").green(), app_path.display());

    println!();
    println!(
        "Language files for '{}' created successfully!",
        style(&name).cyan().bold()
    );
    println!();
    println!("Usage:");
    println!(
        "  {} Set APP_LOCALE={} in .env to use as default",
        style("1.").dim(),
        name
    );
    println!(
        "  {} Translate the strings in lang/{}/*.json",
        style("2.").dim(),
        name
    );
    println!(
        "  {} Use t(\"app.welcome\", &[(\"name\", \"Ferro\")]) in handlers",
        style("3.").dim()
    );
    println!();
}

/// Validate locale code format: lowercase letters, optionally followed by hyphen
/// and more lowercase letters (e.g., en, fr, pt-br, zh-hans).
fn is_valid_locale(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let parts: Vec<&str> = s.split('-').collect();

    // First part must be exactly 2 lowercase letters
    if parts[0].len() != 2 || !parts[0].chars().all(|c| c.is_ascii_lowercase()) {
        return false;
    }

    // Optional subsequent parts must be 2+ lowercase letters
    for part in &parts[1..] {
        if part.len() < 2 || !part.chars().all(|c| c.is_ascii_lowercase()) {
            return false;
        }
    }

    true
}
