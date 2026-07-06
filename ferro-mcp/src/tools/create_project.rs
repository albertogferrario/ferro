//! Create project MCP tool
//!
//! This tool allows AI assistants to scaffold new Ferro projects by
//! calling the CLI's `ferro new` command.

use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct CreateProjectOutput {
    pub success: bool,
    pub project_path: String,
    pub message: String,
    pub next_steps: Vec<String>,
}

/// Execute the create_project tool by shelling out to `ferro new`
///
/// # Arguments
/// * `target_dir` - Directory where to create the project
/// * `name` - Project name
/// * `_description` - Optional project description (not used, CLI uses default)
/// * `no_git` - Whether to skip git initialization
pub fn execute(
    target_dir: &Path,
    name: &str,
    _description: Option<&str>,
    no_git: bool,
) -> Result<CreateProjectOutput, String> {
    let mut cmd = Command::new("ferro");
    cmd.arg("new");
    cmd.arg(name);
    cmd.arg("--no-interaction");

    if no_git {
        cmd.arg("--no-git");
    }

    // Set working directory if specified
    if target_dir != Path::new(".") {
        if !target_dir.exists() {
            return Err(format!(
                "Target directory '{}' does not exist",
                target_dir.display()
            ));
        }
        cmd.current_dir(target_dir);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute ferro new: {e}"))?;

    if output.status.success() {
        let project_path = if target_dir == Path::new(".") {
            name.to_string()
        } else {
            target_dir.join(name).to_string_lossy().to_string()
        };

        Ok(CreateProjectOutput {
            success: true,
            project_path: project_path.clone(),
            message: "Project created successfully".to_string(),
            next_steps: vec![format!("cd {}", project_path), "ferro serve".to_string()],
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "Failed to create project: {}{}",
            stderr,
            if stderr.is_empty() { &stdout } else { "" }
        ))
    }
}
