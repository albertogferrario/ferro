//! docker:init command - Generate production-ready Dockerfile

use console::style;
use std::fs;
use std::path::Path;
use toml::Value;

use crate::templates;

/// Generate Dockerfile and .dockerignore. Returns true if files were created.
pub fn generate() -> bool {
    let package_name = get_package_name();
    let dockerfile_path = Path::new("Dockerfile");
    let dockerignore_path = Path::new(".dockerignore");

    if dockerfile_path.exists() {
        return false;
    }

    let dockerfile_content = templates::dockerfile_template(&package_name);
    if let Err(e) = fs::write(dockerfile_path, dockerfile_content) {
        eprintln!(
            "{} Failed to write Dockerfile: {}",
            style("Error:").red().bold(),
            e
        );
        return false;
    }
    println!("{} Created Dockerfile", style("✓").green());

    if !dockerignore_path.exists() {
        let dockerignore_content = templates::dockerignore_template();
        if let Err(e) = fs::write(dockerignore_path, dockerignore_content) {
            eprintln!(
                "{} Failed to write .dockerignore: {}",
                style("Error:").red().bold(),
                e
            );
        } else {
            println!("{} Created .dockerignore", style("✓").green());
        }
    }

    true
}

pub fn run() {
    if !Path::new("Cargo.toml").exists() {
        eprintln!("{} Cargo.toml not found", style("Error:").red().bold());
        eprintln!(
            "{}",
            style("Make sure you're in a Ferro project root directory.").dim()
        );
        std::process::exit(1);
    }

    if !generate() {
        eprintln!(
            "{} Dockerfile already exists",
            style("Info:").yellow().bold()
        );
        eprintln!(
            "{}",
            style("Remove or rename the existing Dockerfile to generate a new one.").dim()
        );
        std::process::exit(0);
    }

    let package_name = get_package_name();
    println!();
    println!(
        "{}",
        style("Docker files created successfully!").cyan().bold()
    );
    println!();
    println!("Build your image:");
    println!(
        "  {}",
        style(format!("docker build -t {package_name} .")).cyan()
    );
    println!();
    println!("Run your container:");
    println!(
        "  {}",
        style(format!(
            "docker run -p 8080:8080 --env-file .env.production {package_name}"
        ))
        .cyan()
    );
    println!();
    println!(
        "{}",
        style("Tip: Create a .env.production file with your production environment variables.")
            .dim()
    );
    println!();
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
