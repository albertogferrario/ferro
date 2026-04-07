//! MCP server command - start the Model Context Protocol server for AI-assisted development

use console::style;
use std::path::PathBuf;
use std::process::Command;

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

    // Spawn the `ferro-mcp` binary as a subprocess. This avoids a library-level
    // dependency on the ferro-mcp crate (which depends on ferro-cli for shared
    // deploy helpers per D-12), breaking what would otherwise be a cyclic
    // package dependency.
    let status = Command::new("ferro-mcp")
        .arg(project_root.as_os_str())
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!(
                "{} ferro-mcp exited with status {}",
                style("[ERROR]").red().bold(),
                s
            );
            std::process::exit(s.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!(
                "{} Failed to spawn ferro-mcp binary: {} (is it installed and on PATH?)",
                style("[ERROR]").red().bold(),
                e
            );
            std::process::exit(1);
        }
    }
}
