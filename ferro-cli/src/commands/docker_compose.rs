//! docker:compose command - Generate docker-compose.yml for local development

use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::fs;
use std::path::Path;
use toml::Value;

use crate::templates;

pub fn run(with_mailpit: bool, with_minio: bool) {
    // Verify we're in a Ferro project directory
    if !Path::new("Cargo.toml").exists() {
        eprintln!("{} Cargo.toml not found", style("Error:").red().bold());
        eprintln!(
            "{}",
            style("Make sure you're in a Ferro project root directory.").dim()
        );
        std::process::exit(1);
    }

    // Get project name
    let project_name = get_project_name();

    let compose_path = Path::new("docker-compose.yml");

    // Check if docker-compose.yml already exists
    if compose_path.exists() {
        eprintln!(
            "{} docker-compose.yml already exists",
            style("Info:").yellow().bold()
        );
        eprintln!(
            "{}",
            style("Remove or rename the existing docker-compose.yml to generate a new one.").dim()
        );
        std::process::exit(0);
    }

    // Prompt for optional services
    let (include_mailpit, include_minio) = prompt_for_services(with_mailpit, with_minio);

    // Generate docker-compose.yml
    let compose_content =
        templates::docker_compose_template(&project_name, include_mailpit, include_minio);
    if let Err(e) = fs::write(compose_path, compose_content) {
        eprintln!(
            "{} Failed to write docker-compose.yml: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!("{} Created docker-compose.yml", style("✓").green());

    // Update .gitignore if needed
    update_gitignore();

    // Print usage instructions
    print_instructions(&project_name, include_mailpit, include_minio);
}

fn get_project_name() -> String {
    let cargo_toml = match fs::read_to_string("Cargo.toml") {
        Ok(content) => content,
        Err(_) => {
            // Fallback to directory name
            return std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
                .unwrap_or_else(|| "ferro_app".to_string());
        }
    };

    let parsed: Value = match cargo_toml.parse() {
        Ok(v) => v,
        Err(_) => {
            return std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
                .unwrap_or_else(|| "ferro_app".to_string());
        }
    };

    parsed["package"]["name"]
        .as_str()
        .unwrap_or("ferro_app")
        .to_string()
}

fn prompt_for_services(with_mailpit: bool, with_minio: bool) -> (bool, bool) {
    // If flags are provided, use them directly
    if with_mailpit || with_minio {
        return (with_mailpit, with_minio);
    }

    println!();
    println!("{}", style("Optional Services").cyan().bold());
    println!(
        "{}",
        style("PostgreSQL and Redis are included by default.").dim()
    );
    println!();

    let include_mailpit = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Include Mailpit (email testing)?")
        .default(false)
        .interact()
        .unwrap_or(false);

    let include_minio = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Include MinIO (S3-compatible storage)?")
        .default(false)
        .interact()
        .unwrap_or(false);

    println!();

    (include_mailpit, include_minio)
}

fn update_gitignore() {
    let gitignore_path = Path::new(".gitignore");
    if !gitignore_path.exists() {
        return;
    }

    let content = match fs::read_to_string(gitignore_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Check if docker-compose.override.yml is already ignored
    if content.contains("docker-compose.override.yml") {
        return;
    }

    // Append to .gitignore
    let new_content = format!(
        "{}\n# Local Docker overrides\ndocker-compose.override.yml\n",
        content.trim_end()
    );

    if fs::write(gitignore_path, new_content).is_ok() {
        println!("{} Updated .gitignore", style("✓").green());
    }
}

fn print_instructions(project_name: &str, has_mailpit: bool, has_minio: bool) {
    println!();
    println!(
        "{}",
        style("Docker Compose created successfully!").cyan().bold()
    );
    println!();
    println!("Start services:");
    println!("  {}", style("docker compose up -d").cyan());
    println!();
    println!("Stop services:");
    println!("  {}", style("docker compose down").cyan());
    println!();
    println!("Services:");
    println!(
        "  {} PostgreSQL: {}",
        style("•").dim(),
        style("localhost:5432").underlined()
    );
    println!(
        "  {} Redis: {}",
        style("•").dim(),
        style("localhost:6379").underlined()
    );
    if has_mailpit {
        println!(
            "  {} Mailpit SMTP: {}",
            style("•").dim(),
            style("localhost:1025").underlined()
        );
        println!(
            "  {} Mailpit UI: {}",
            style("•").dim(),
            style("http://localhost:8025").underlined()
        );
    }
    if has_minio {
        println!(
            "  {} MinIO API: {}",
            style("•").dim(),
            style("localhost:9000").underlined()
        );
        println!(
            "  {} MinIO Console: {}",
            style("•").dim(),
            style("http://localhost:9001").underlined()
        );
    }
    println!();
    println!("Update your .env:");
    println!(
        "  {}",
        style("DATABASE_URL=postgres://ferro:ferro_secret@localhost:5432/ferro_db").dim()
    );
    if has_mailpit {
        println!("  {}", style("MAIL_HOST=localhost").dim());
        println!("  {}", style("MAIL_PORT=1025").dim());
    }
    if has_minio {
        println!("  {}", style("S3_ENDPOINT=http://localhost:9000").dim());
        println!("  {}", style("S3_ACCESS_KEY=minioadmin").dim());
        println!("  {}", style("S3_SECRET_KEY=minioadmin").dim());
    }
    println!();
    println!(
        "{}",
        style(format!("Network: {project_name}_network")).dim()
    );
    println!();
}
