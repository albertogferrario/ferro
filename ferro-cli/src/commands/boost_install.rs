//! boost:install command - Generate MCP configuration and AI guidelines

use console::style;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(editor: Option<String>) {
    // Verify we're in a Ferro project directory
    if !Path::new("Cargo.toml").exists() {
        eprintln!("{} Cargo.toml not found", style("Error:").red().bold());
        eprintln!(
            "{}",
            style("Make sure you're in a Ferro project root directory.").dim()
        );
        std::process::exit(1);
    }

    // Detect or use specified editor
    let target_editor = editor.unwrap_or_else(detect_editor);

    println!(
        "{} Installing AI development boost for {}...",
        style("⚡").cyan(),
        style(&target_editor).yellow()
    );
    println!();

    // Generate MCP configuration
    generate_mcp_config(&target_editor);

    // Generate AI guidelines
    generate_ai_guidelines(&target_editor);

    // Print success message
    println!();
    println!(
        "{}",
        style("AI development boost installed successfully!")
            .green()
            .bold()
    );
    println!();

    // Print editor-specific instructions
    match target_editor.as_str() {
        "cursor" => {
            println!("To activate MCP in Cursor:");
            println!("  1. Open Command Palette (Cmd+Shift+P / Ctrl+Shift+P)");
            println!("  2. Search for 'Reload Window'");
            println!("  3. The Ferro MCP tools will now be available");
        }
        "claude" => {
            println!("MCP configuration written to {}", style(".mcp.json").cyan());
            println!("CLAUDE.md updated with Ferro framework guidelines.");
            println!();
            println!("Claude Code will automatically use these configurations.");
        }
        "vscode" => {
            println!(
                "AI guidelines written to {}",
                style(".ai/guidelines/").cyan()
            );
            println!("GitHub Copilot will use these guidelines for context.");
        }
        _ => {
            println!("Configuration files have been generated.");
        }
    }

    println!();
}

fn detect_editor() -> String {
    // Check for editor-specific files/directories
    if Path::new(".cursor").exists() {
        return "cursor".to_string();
    }

    if Path::new("CLAUDE.md").exists() || std::env::var("CLAUDE_CODE").is_ok() {
        return "claude".to_string();
    }

    if Path::new(".vscode").exists() {
        return "vscode".to_string();
    }

    // Default to claude as it's the most common for MCP
    "claude".to_string()
}

fn generate_mcp_config(editor: &str) {
    let config_path = match editor {
        "cursor" => {
            // Cursor uses .cursor/mcp.json
            fs::create_dir_all(".cursor").ok();
            ".cursor/mcp.json"
        }
        _ => {
            // Claude and others use .mcp.json at root
            ".mcp.json"
        }
    };

    // Try to find the ferro binary path
    let ferro_command = find_ferro_binary();
    let config_content = format!(
        r#"{{
  "mcpServers": {{
    "ferro": {{
      "command": "{}",
      "args": ["mcp"],
      "env": {{}}
    }}
  }}
}}
"#,
        ferro_command.replace('\\', "\\\\").replace('"', "\\\"")
    );

    if Path::new(config_path).exists() {
        println!(
            "{} {} already exists, skipping",
            style("→").dim(),
            config_path
        );
    } else {
        if let Err(e) = fs::write(config_path, &config_content) {
            eprintln!(
                "{} Failed to write {}: {}",
                style("Error:").red().bold(),
                config_path,
                e
            );
            return;
        }
        println!("{} Created {}", style("✓").green(), config_path);
    }
}

fn find_ferro_binary() -> String {
    // First, check if ferro is in PATH
    if let Ok(output) = std::process::Command::new("which").arg("ferro").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                let path = path.trim();
                if !path.is_empty() {
                    return path.to_string();
                }
            }
        }
    }

    // On Windows, try where instead
    #[cfg(windows)]
    if let Ok(output) = std::process::Command::new("where").arg("ferro").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                if let Some(first_line) = path.lines().next() {
                    return first_line.to_string();
                }
            }
        }
    }

    // Try to get the current executable's directory
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let ferro_in_same_dir = exe_dir.join("ferro");
            if ferro_in_same_dir.exists() {
                return ferro_in_same_dir.to_string_lossy().to_string();
            }
        }
        // If this IS the ferro binary, use its path
        if current_exe
            .file_name()
            .map(|n| n == "ferro")
            .unwrap_or(false)
        {
            return current_exe.to_string_lossy().to_string();
        }
    }

    // Fall back to just "ferro" and hope it's in PATH
    "ferro".to_string()
}

fn generate_ai_guidelines(editor: &str) {
    // Create .ai/guidelines directory
    let guidelines_dir = Path::new(".ai/guidelines");
    if let Err(e) = fs::create_dir_all(guidelines_dir) {
        eprintln!(
            "{} Failed to create .ai/guidelines: {}",
            style("Error:").red().bold(),
            e
        );
        return;
    }

    // Generate Ferro framework guidelines
    let ferro_md_path = guidelines_dir.join("ferro.md");
    if !ferro_md_path.exists() {
        let content = templates::ferro_guidelines_template();
        if let Err(e) = fs::write(&ferro_md_path, content) {
            eprintln!(
                "{} Failed to write ferro.md: {}",
                style("Error:").red().bold(),
                e
            );
        } else {
            println!("{} Created .ai/guidelines/ferro.md", style("✓").green());
        }
    } else {
        println!(
            "{} .ai/guidelines/ferro.md already exists, skipping",
            style("→").dim()
        );
    }

    // Generate editor-specific rules
    match editor {
        "cursor" => {
            let cursor_rules_path = Path::new(".cursorrules");
            if !cursor_rules_path.exists() {
                let content = templates::cursor_rules_template();
                if let Err(e) = fs::write(cursor_rules_path, content) {
                    eprintln!(
                        "{} Failed to write .cursorrules: {}",
                        style("Error:").red().bold(),
                        e
                    );
                } else {
                    println!("{} Created .cursorrules", style("✓").green());
                }
            } else {
                println!("{} .cursorrules already exists, skipping", style("→").dim());
            }
        }
        "claude" => {
            let claude_md_path = Path::new("CLAUDE.md");
            if !claude_md_path.exists() {
                let content = templates::claude_md_template();
                if let Err(e) = fs::write(claude_md_path, content) {
                    eprintln!(
                        "{} Failed to write CLAUDE.md: {}",
                        style("Error:").red().bold(),
                        e
                    );
                } else {
                    println!("{} Created CLAUDE.md", style("✓").green());
                }
            } else {
                // Append Ferro-specific instructions if not already present
                let existing = fs::read_to_string(claude_md_path).unwrap_or_default();
                if !existing.contains("Ferro Framework") {
                    let ferro_section = templates::claude_md_ferro_section();
                    if let Err(e) = fs::write(
                        claude_md_path,
                        format!("{}\n\n{}", existing.trim(), ferro_section),
                    ) {
                        eprintln!(
                            "{} Failed to update CLAUDE.md: {}",
                            style("Error:").red().bold(),
                            e
                        );
                    } else {
                        println!(
                            "{} Updated CLAUDE.md with Ferro guidelines",
                            style("✓").green()
                        );
                    }
                } else {
                    println!(
                        "{} CLAUDE.md already contains Ferro guidelines, skipping",
                        style("→").dim()
                    );
                }
            }
        }
        "vscode" => {
            let copilot_path = guidelines_dir.join("copilot.md");
            if !copilot_path.exists() {
                let content = templates::copilot_instructions_template();
                if let Err(e) = fs::write(&copilot_path, content) {
                    eprintln!(
                        "{} Failed to write copilot.md: {}",
                        style("Error:").red().bold(),
                        e
                    );
                } else {
                    println!("{} Created .ai/guidelines/copilot.md", style("✓").green());
                }
            } else {
                println!(
                    "{} .ai/guidelines/copilot.md already exists, skipping",
                    style("→").dim()
                );
            }
        }
        _ => {}
    }
}
