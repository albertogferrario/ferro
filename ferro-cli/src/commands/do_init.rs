//! do:init command - Generate DigitalOcean App Platform deployment spec

use crate::templates;
use console::style;
use std::fs;
use std::path::Path;
use toml::Value;

pub fn run(github_repo: Option<String>) {
    // Verify we're in a Ferro project
    if !Path::new("Cargo.toml").exists() {
        eprintln!(
            "{} Not a Ferro project directory (Cargo.toml not found)",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    }

    // Check if .do/app.yaml already exists
    let do_dir = Path::new(".do");
    let app_yaml_path = do_dir.join("app.yaml");

    if app_yaml_path.exists() {
        eprintln!(
            "{} .do/app.yaml already exists. Remove it first to regenerate.",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    }

    // Get package name from Cargo.toml
    let package_name = get_package_name();

    // Get GitHub repo - use provided or error
    let repo = match github_repo {
        Some(r) => r,
        None => {
            eprintln!(
                "{} GitHub repository not specified. Use --repo owner/repo",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    // Create .do directory
    if let Err(e) = fs::create_dir_all(do_dir) {
        eprintln!(
            "{} Failed to create .do directory: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }

    // Generate and write app.yaml
    let content = templates::do_app_yaml_template(&package_name, &repo);

    if let Err(e) = fs::write(&app_yaml_path, content) {
        eprintln!(
            "{} Failed to write app.yaml: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }

    println!(
        "{} Created .do/app.yaml for DigitalOcean App Platform",
        style("✓").green()
    );

    // Generate Dockerfile and .dockerignore if they don't exist
    if !super::docker_init::generate() {
        println!(
            "{} Dockerfile already exists, skipping generation",
            style("✓").green()
        );
    }

    println!();
    println!("Next steps:");
    println!("  1. Review and customize .do/app.yaml");
    println!("  2. Push to GitHub and connect to DigitalOcean App Platform");
    println!();
    println!(
        "{}",
        style("Tip: Add a /health endpoint for health checks").dim()
    );
}

fn get_package_name() -> String {
    let cargo_toml = match fs::read_to_string("Cargo.toml") {
        Ok(content) => content,
        Err(_) => return "app".to_string(),
    };

    let parsed: Value = match cargo_toml.parse() {
        Ok(v) => v,
        Err(_) => return "app".to_string(),
    };

    parsed["package"]["name"]
        .as_str()
        .unwrap_or("app")
        .to_string()
}
