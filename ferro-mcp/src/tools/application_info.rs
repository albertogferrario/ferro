//! Application info tool - returns framework metadata

use crate::error::{McpError, Result};
use crate::introspection;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct ApplicationInfo {
    pub framework_version: String,
    pub rust_version: String,
    pub database_engine: Option<String>,
    pub environment: String,
    pub installed_crates: Vec<CrateInfo>,
    pub models: Vec<ModelInfo>,
    pub json_ui_views: JsonUiViewsStatus,
    pub broadcasting: BroadcastingStatus,
    pub claude_code_skills: ClaudeCodeSkillsStatus,
}

#[derive(Debug, Serialize)]
pub struct BroadcastingStatus {
    pub available: bool,
    pub ws_endpoint: String,
    pub hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClaudeCodeSkillsStatus {
    pub installed: bool,
    pub skill_count: usize,
    pub install_hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JsonUiViewsStatus {
    pub available: bool,
    pub view_count: usize,
    pub views_dir: String,
    pub hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub table: Option<String>,
    pub path: String,
}

pub fn execute(project_root: &Path) -> Result<ApplicationInfo> {
    // Get framework version from Cargo.toml
    let framework_version = get_framework_version(project_root)?;

    // Get Rust version
    let rust_version = get_rust_version();

    // Get database engine from .env
    let database_engine = get_database_engine(project_root);

    // Get environment from .env
    let environment = get_environment(project_root);

    // Get installed ferro-* crates
    let installed_crates = get_installed_crates(project_root)?;

    // Scan for models
    let models = introspection::models::scan_models(project_root);

    // Scan for JSON-UI views
    let json_ui_views = scan_json_ui_views(project_root);

    // Check broadcasting status
    let broadcasting = check_broadcasting(&installed_crates, project_root);

    // Check Claude Code skills installation
    let claude_code_skills = check_claude_code_skills();

    Ok(ApplicationInfo {
        framework_version,
        rust_version,
        database_engine,
        environment,
        installed_crates,
        models,
        json_ui_views,
        broadcasting,
        claude_code_skills,
    })
}

fn get_framework_version(project_root: &Path) -> Result<String> {
    let cargo_toml = project_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)
        .map_err(|_| McpError::FileNotFound("Cargo.toml".to_string()))?;

    // Parse Cargo.toml
    let parsed: toml::Value = content
        .parse()
        .map_err(|e| McpError::ParseError(format!("Failed to parse Cargo.toml: {}", e)))?;

    // Try to get version from package section
    if let Some(package) = parsed.get("package") {
        if let Some(version) = package.get("version") {
            return Ok(version.as_str().unwrap_or("unknown").to_string());
        }
    }

    // Try to get from workspace.package
    if let Some(workspace) = parsed.get("workspace") {
        if let Some(package) = workspace.get("package") {
            if let Some(version) = package.get("version") {
                return Ok(version.as_str().unwrap_or("unknown").to_string());
            }
        }
    }

    Ok("unknown".to_string())
}

fn get_rust_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_database_engine(project_root: &Path) -> Option<String> {
    let env_path = project_root.join(".env");
    if !env_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&env_path).ok()?;

    for line in content.lines() {
        if line.starts_with("DATABASE_URL=") {
            let url = line.trim_start_matches("DATABASE_URL=").trim_matches('"');
            if url.starts_with("sqlite:") {
                return Some("sqlite".to_string());
            } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
                return Some("postgresql".to_string());
            } else if url.starts_with("mysql:") {
                return Some("mysql".to_string());
            }
        }
    }

    None
}

fn get_environment(project_root: &Path) -> String {
    let env_path = project_root.join(".env");
    if !env_path.exists() {
        return "local".to_string();
    }

    let content = std::fs::read_to_string(&env_path).unwrap_or_default();

    for line in content.lines() {
        if line.starts_with("APP_ENV=") || line.starts_with("ENVIRONMENT=") {
            return line
                .split('=')
                .nth(1)
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_else(|| "local".to_string());
        }
    }

    "local".to_string()
}

fn get_installed_crates(project_root: &Path) -> Result<Vec<CrateInfo>> {
    let cargo_toml = project_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)
        .map_err(|_| McpError::FileNotFound("Cargo.toml".to_string()))?;

    let parsed: toml::Value = content
        .parse()
        .map_err(|e| McpError::ParseError(format!("Failed to parse Cargo.toml: {}", e)))?;

    let mut crates = Vec::new();

    // Check dependencies section
    if let Some(deps) = parsed.get("dependencies") {
        if let Some(table) = deps.as_table() {
            for (name, value) in table {
                // Filter for ferro-* crates
                if name.starts_with("ferro") {
                    let version = match value {
                        toml::Value::String(v) => v.clone(),
                        toml::Value::Table(t) => t
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("workspace")
                            .to_string(),
                        _ => "unknown".to_string(),
                    };
                    crates.push(CrateInfo {
                        name: name.clone(),
                        version,
                    });
                }
            }
        }
    }

    Ok(crates)
}

fn scan_json_ui_views(project_root: &Path) -> JsonUiViewsStatus {
    let views_dir = project_root.join("src").join("views");
    let views_dir_display = "src/views/".to_string();

    if !views_dir.exists() {
        return JsonUiViewsStatus {
            available: false,
            view_count: 0,
            views_dir: views_dir_display,
            hint: Some(
                "No src/views/ directory found. Create views with JSON-UI tools: json_ui_generate, json_ui_catalog"
                    .to_string(),
            ),
        };
    }

    // Count .rs files in src/views/ (excluding mod.rs)
    let view_count = std::fs::read_dir(&views_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.extension().map(|ext| ext == "rs").unwrap_or(false)
                        && path.file_name().map(|f| f != "mod.rs").unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0);

    JsonUiViewsStatus {
        available: true,
        view_count,
        views_dir: views_dir_display,
        hint: if view_count == 0 {
            Some(
                "Views directory exists but no views found. Use json_ui_generate to create one."
                    .to_string(),
            )
        } else {
            None
        },
    }
}

fn check_broadcasting(installed_crates: &[CrateInfo], project_root: &Path) -> BroadcastingStatus {
    let has_crate = installed_crates.iter().any(|c| c.name == "ferro-broadcast");

    if !has_crate {
        return BroadcastingStatus {
            available: false,
            ws_endpoint: "/_ferro/ws".to_string(),
            hint: Some(
                "Add ferro-broadcast to dependencies for real-time WebSocket broadcasting"
                    .to_string(),
            ),
        };
    }

    // Check if bootstrap.rs mentions Broadcaster registration
    let bootstrap_path = project_root.join("src").join("bootstrap.rs");
    let configured = bootstrap_path.exists()
        && std::fs::read_to_string(&bootstrap_path)
            .map(|c| c.contains("Broadcaster"))
            .unwrap_or(false);

    BroadcastingStatus {
        available: true,
        ws_endpoint: "/_ferro/ws".to_string(),
        hint: if configured {
            None
        } else {
            Some(
                "ferro-broadcast is available. Register a Broadcaster in bootstrap.rs to enable WebSocket connections at /_ferro/ws. Use code_templates category=broadcasting for setup examples."
                    .to_string(),
            )
        },
    }
}

fn check_claude_code_skills() -> ClaudeCodeSkillsStatus {
    // Get home directory and check for skills
    let skills_dir = dirs::home_dir().map(|h| h.join(".claude").join("commands").join("ferro"));

    match skills_dir {
        Some(dir) if dir.exists() => {
            // Count .md files in the directory
            let skill_count = std::fs::read_dir(&dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
                        .count()
                })
                .unwrap_or(0);

            ClaudeCodeSkillsStatus {
                installed: skill_count > 0,
                skill_count,
                install_hint: if skill_count == 0 {
                    Some("Run `ferro claude:install` to install Claude Code skills".to_string())
                } else {
                    None
                },
            }
        }
        _ => ClaudeCodeSkillsStatus {
            installed: false,
            skill_count: 0,
            install_hint: Some(
                "Run `ferro claude:install` to install Claude Code skills for enhanced DX"
                    .to_string(),
            ),
        },
    }
}
