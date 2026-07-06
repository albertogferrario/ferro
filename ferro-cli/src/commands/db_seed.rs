use console::style;
use std::path::Path;
use std::process::Command;

pub fn run(class: Option<String>) {
    // Check we're in a Ferro project
    if !Path::new("src/seeders").exists() {
        eprintln!(
            "{} No seeders directory found at src/seeders",
            style("Error:").red().bold()
        );
        eprintln!(
            "{}",
            style("Run 'ferro make:seeder <name>' to create your first seeder.").dim()
        );
        std::process::exit(1);
    }

    match &class {
        Some(name) => {
            println!(
                "{} Running seeder: {}...",
                style("->").cyan(),
                style(name).bold()
            );
        }
        None => {
            println!("{} Running all seeders...", style("->").cyan());
        }
    }

    // Build command arguments
    let mut args = vec!["run", "--quiet", "--", "db:seed"];

    let class_arg;
    if let Some(name) = &class {
        class_arg = format!("--class={name}");
        args.push(&class_arg);
    }

    // Run cargo run -- db:seed [--class=Name] (unified binary)
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("Failed to execute cargo command");

    if !status.success() {
        eprintln!("{} Seeding failed", style("Error:").red().bold());
        std::process::exit(1);
    }
}
