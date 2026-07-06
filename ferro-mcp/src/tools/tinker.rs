//! Tinker tool - Execute Rust code within the app context

use crate::error::{McpError, Result};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct TinkerResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Execute Rust code in a temporary context
///
/// Creates a temporary Rust project that imports the main app's types
/// and executes the provided code snippet.
pub fn execute(project_root: &Path, code: &str) -> Result<TinkerResult> {
    // Create temp directory for the tinker session
    let temp_dir = std::env::temp_dir().join(format!("ferro-tinker-{}", std::process::id()));

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
    }
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| McpError::ExecutionError(format!("Failed to create temp dir: {e}")))?;

    // Get the app crate name from Cargo.toml
    let app_name = get_app_name(project_root).unwrap_or_else(|| "app".to_string());

    // Create Cargo.toml for tinker
    let cargo_toml = format!(
        r#"[package]
name = "tinker-session"
version = "0.1.0"
edition = "2021"

[dependencies]
{app_name} = {{ path = "{project_path}/app" }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        app_name = app_name,
        project_path = project_root.display()
    );

    std::fs::write(temp_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| McpError::ExecutionError(format!("Failed to write Cargo.toml: {e}")))?;

    // Create src directory
    std::fs::create_dir_all(temp_dir.join("src"))
        .map_err(|e| McpError::ExecutionError(format!("Failed to create src dir: {e}")))?;

    // Wrap user code in a main function if it doesn't have one
    let wrapped_code = if code.contains("fn main") || code.contains("async fn main") {
        code.to_string()
    } else {
        format!(
            r#"use {app_name}::*;

#[tokio::main]
async fn main() {{
    let result = {{
        {code}
    }};
    println!("{{:?}}", result);
}}
"#
        )
    };

    std::fs::write(temp_dir.join("src/main.rs"), &wrapped_code)
        .map_err(|e| McpError::ExecutionError(format!("Failed to write main.rs: {e}")))?;

    // Run cargo run
    let output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .current_dir(&temp_dir)
        .output()
        .map_err(|e| McpError::ExecutionError(format!("Failed to run cargo: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Clean up temp directory
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(TinkerResult {
        success: output.status.success(),
        stdout,
        stderr,
        exit_code: output.status.code(),
    })
}

fn get_app_name(project_root: &Path) -> Option<String> {
    let cargo_toml = project_root.join("app/Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).ok()?;
    let parsed: toml::Value = content.parse().ok()?;

    parsed
        .get("package")?
        .get("name")?
        .as_str()
        .map(|s| s.to_string())
}
