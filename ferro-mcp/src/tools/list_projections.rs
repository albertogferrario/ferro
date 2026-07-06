//! List service projections defined in a project's `src/projections/` directory.

use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// List of discovered service projections.
#[derive(Debug, Serialize)]
pub struct ProjectionList {
    pub projections: Vec<ProjectionInfo>,
    pub total: usize,
}

/// Metadata about a single projection function.
#[derive(Debug, Serialize)]
pub struct ProjectionInfo {
    /// Function name (e.g., "user_service").
    pub name: String,
    /// Relative path from project root (e.g., "src/projections/user.rs").
    pub file: String,
    /// Extracted from `ServiceDef::new("...")`.
    pub service_name: Option<String>,
    /// Extracted from `.display_name("...")`.
    pub display_name: Option<String>,
}

/// Scan the project's `src/projections/` directory for ServiceDef functions.
///
/// Returns an empty list when the directory does not exist.
pub fn execute(project_root: &Path, filter: Option<&str>) -> ProjectionList {
    let projections_dir = project_root.join("src/projections");
    if !projections_dir.exists() {
        return ProjectionList {
            projections: Vec::new(),
            total: 0,
        };
    }

    let fn_re = Regex::new(r"pub\s+fn\s+(\w+)\s*\(.*\).*->\s*ServiceDef").unwrap();
    let service_name_re = Regex::new(r#"ServiceDef::new\("([^"]+)"\)"#).unwrap();
    let display_name_re = Regex::new(r#"\.display_name\("([^"]+)"\)"#).unwrap();

    let mut projections = Vec::new();

    let entries: Vec<_> = match fs::read_dir(&projections_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => {
            return ProjectionList {
                projections: Vec::new(),
                total: 0,
            }
        }
    };

    for entry in entries {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        if path.file_name().is_some_and(|n| n == "mod.rs") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        for fn_cap in fn_re.captures_iter(&content) {
            let name = fn_cap[1].to_string();

            let service_name = service_name_re.captures(&content).map(|c| c[1].to_string());

            let display_name = display_name_re.captures(&content).map(|c| c[1].to_string());

            projections.push(ProjectionInfo {
                name,
                file: relative_path.clone(),
                service_name,
                display_name,
            });
        }
    }

    // Apply optional filter (case-insensitive substring on name)
    if let Some(filter_str) = filter {
        let filter_lower = filter_str.to_lowercase();
        projections.retain(|p| p.name.to_lowercase().contains(&filter_lower));
    }

    let total = projections.len();
    ProjectionList { projections, total }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_empty_project() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_projections_test");
        let result = execute(&non_existent, None);
        assert_eq!(result.total, 0);
        assert!(result.projections.is_empty());
    }

    #[test]
    fn test_serialization() {
        let list = ProjectionList {
            projections: vec![ProjectionInfo {
                name: "user_service".to_string(),
                file: "src/projections/user.rs".to_string(),
                service_name: Some("user".to_string()),
                display_name: Some("User".to_string()),
            }],
            total: 1,
        };

        let json = serde_json::to_string(&list);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("user_service"));
        assert!(json_str.contains("\"user\""));
        assert!(json_str.contains("User"));
        assert!(json_str.contains("projections"));
        assert!(json_str.contains("total"));
    }
}
