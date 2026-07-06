//! MCP server command - start the Model Context Protocol server for AI-assisted development

use console::style;
use std::path::PathBuf;

pub fn run(cwd: Option<String>) {
    eprintln!(
        "{} Starting Ferro MCP server...",
        style("[MCP]").cyan().bold()
    );

    let project_root = cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    eprintln!(
        "{} Project root: {}",
        style("[MCP]").cyan().bold(),
        project_root.display()
    );

    if let Err(e) = std::env::set_current_dir(&project_root) {
        eprintln!(
            "{} Failed to chdir to project root: {}",
            style("[ERROR]").red().bold(),
            e
        );
        std::process::exit(1);
    }

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!(
                "{} Failed to create tokio runtime: {}",
                style("[ERROR]").red().bold(),
                e
            );
            std::process::exit(1);
        }
    };

    if let Err(e) = runtime.block_on(ferro_mcp::run()) {
        eprintln!("{} ferro-mcp failed: {}", style("[ERROR]").red().bold(), e);
        std::process::exit(1);
    }
}
