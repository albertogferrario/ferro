//! claude:install command - Install Ferro Claude Code skills

use console::style;
use std::fs;
use std::path::PathBuf;

/// Embedded skill files - these are compiled into the binary
const SKILLS: &[(&str, &str)] = &[
    ("help.md", include_str!("skills/help.md")),
    ("info.md", include_str!("skills/info.md")),
    ("routes.md", include_str!("skills/routes.md")),
    ("route-explain.md", include_str!("skills/route-explain.md")),
    ("model.md", include_str!("skills/model.md")),
    ("models.md", include_str!("skills/models.md")),
    ("controller.md", include_str!("skills/controller.md")),
    ("middleware.md", include_str!("skills/middleware.md")),
    ("db.md", include_str!("skills/db.md")),
    ("test.md", include_str!("skills/test.md")),
    ("serve.md", include_str!("skills/serve.md")),
    ("new.md", include_str!("skills/new.md")),
    ("tinker.md", include_str!("skills/tinker.md")),
    ("diagnose.md", include_str!("skills/diagnose.md")),
];

pub fn run(force: bool, list: bool) {
    if list {
        list_skills();
        return;
    }

    let target_dir = get_target_directory();

    println!(
        "{} Installing Ferro Claude Code skills...",
        style("🦀").cyan()
    );
    println!();

    // Create target directory
    if let Err(e) = fs::create_dir_all(&target_dir) {
        eprintln!(
            "{} Failed to create directory {}: {}",
            style("Error:").red().bold(),
            target_dir.display(),
            e
        );
        std::process::exit(1);
    }

    let mut installed = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for (filename, content) in SKILLS {
        let target_path = target_dir.join(filename);

        if target_path.exists() && !force {
            println!(
                "{} {} already exists, skipping (use --force to overwrite)",
                style("→").dim(),
                filename
            );
            skipped += 1;
            continue;
        }

        match fs::write(&target_path, content) {
            Ok(_) => {
                let action = if target_path.exists() && force {
                    "Updated"
                } else {
                    "Created"
                };
                println!("{} {} {}", style("✓").green(), action, filename);
                installed += 1;
            }
            Err(e) => {
                eprintln!("{} Failed to write {}: {}", style("✗").red(), filename, e);
                errors += 1;
            }
        }
    }

    println!();

    if errors > 0 {
        eprintln!(
            "{} Completed with errors: {} installed, {} skipped, {} failed",
            style("⚠").yellow(),
            installed,
            skipped,
            errors
        );
        std::process::exit(1);
    }

    println!(
        "{}",
        style("Ferro Claude Code skills installed successfully!")
            .green()
            .bold()
    );
    println!();
    println!("Location: {}", style(target_dir.display()).cyan());
    println!();
    println!("Available commands:");
    println!(
        "  {} - Show all available Ferro commands",
        style("/ferro:help").yellow()
    );
    println!("  {} - Project information", style("/ferro:info").yellow());
    println!("  {} - List all routes", style("/ferro:routes").yellow());
    println!("  {} - Generate a model", style("/ferro:model").yellow());
    println!("  {} - Database operations", style("/ferro:db").yellow());
    println!();
    println!(
        "{}",
        style("Tip: Run /ferro:help in Claude Code to see all commands").dim()
    );
}

fn list_skills() {
    println!("{} Ferro Claude Code Skills", style("🦀").cyan());
    println!();

    for (filename, _) in SKILLS {
        let name = filename.trim_end_matches(".md");
        println!("  {} /ferro:{}", style("•").dim(), style(name).yellow());
    }

    println!();
    println!("Total: {} skills", SKILLS.len());
}

fn get_target_directory() -> PathBuf {
    // Get home directory
    let home = dirs::home_dir().unwrap_or_else(|| {
        eprintln!(
            "{} Could not determine home directory",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    });

    home.join(".claude").join("commands").join("ferro")
}
