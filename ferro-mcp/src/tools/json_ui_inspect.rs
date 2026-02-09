//! JSON-UI inspect tool - discovers existing JSON-UI views in a project

use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// List of discovered JSON-UI views
#[derive(Debug, Serialize)]
pub struct JsonUiViewList {
    pub views: Vec<ViewInfo>,
    pub total: usize,
}

/// Metadata about a single JSON-UI view function
#[derive(Debug, Serialize)]
pub struct ViewInfo {
    /// Function name (e.g., "view", "user_list")
    pub name: String,
    /// Relative path from project root (e.g., "src/views/user_list.rs")
    pub file: String,
    /// Extracted .title() value
    pub title: Option<String>,
    /// Extracted .layout() value
    pub layout: Option<String>,
    /// Component:: variant names found in the view
    pub components_used: Vec<String>,
    /// Action handler references found (e.g., "Action::get(\"users.show\")")
    pub actions: Vec<String>,
}

/// Scan the project's src/views/ directory for JSON-UI view functions.
///
/// When `src/views/` does not exist, returns an empty list (not an error).
pub fn execute(project_root: &Path, filter: Option<&str>) -> JsonUiViewList {
    let views_dir = project_root.join("src/views");
    if !views_dir.exists() {
        return JsonUiViewList {
            views: Vec::new(),
            total: 0,
        };
    }

    let fn_re = Regex::new(r"pub\s+fn\s+(\w+)\s*\(.*\).*->\s*JsonUiView").unwrap();
    let title_re = Regex::new(r#"\.title\("([^"]+)"\)"#).unwrap();
    let layout_re = Regex::new(r#"\.layout\("([^"]+)"\)"#).unwrap();
    let component_re = Regex::new(r"Component::(\w+)").unwrap();
    let action_re = Regex::new(r#"Action::(get|new|delete)\("([^"]+)"\)"#).unwrap();

    let mut views = Vec::new();

    let entries: Vec<_> = match fs::read_dir(&views_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => {
            return JsonUiViewList {
                views: Vec::new(),
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

            let title = title_re
                .captures(&content)
                .map(|c| c[1].to_string());

            let layout = layout_re
                .captures(&content)
                .map(|c| c[1].to_string());

            let components_used: Vec<String> = component_re
                .captures_iter(&content)
                .map(|c| c[1].to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let actions: Vec<String> = action_re
                .captures_iter(&content)
                .map(|c| format!("Action::{}(\"{}\")", &c[1], &c[2]))
                .collect();

            views.push(ViewInfo {
                name,
                file: relative_path.clone(),
                title,
                layout,
                components_used,
                actions,
            });
        }
    }

    // Apply optional filter (case-insensitive substring match on name)
    if let Some(filter_str) = filter {
        let filter_lower = filter_str.to_lowercase();
        views.retain(|v| v.name.to_lowercase().contains(&filter_lower));
    }

    let total = views.len();
    JsonUiViewList { views, total }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_empty_project() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_test");
        let result = execute(&non_existent, None);
        assert_eq!(result.total, 0);
        assert!(result.views.is_empty());
    }

    #[test]
    fn test_serialization() {
        let list = JsonUiViewList {
            views: vec![ViewInfo {
                name: "user_list".to_string(),
                file: "src/views/user_list.rs".to_string(),
                title: Some("User List".to_string()),
                layout: Some("app".to_string()),
                components_used: vec!["Table".to_string(), "Text".to_string()],
                actions: vec!["Action::get(\"users.show\")".to_string()],
            }],
            total: 1,
        };

        let json = serde_json::to_string(&list);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("user_list"));
        assert!(json_str.contains("User List"));
        assert!(json_str.contains("Table"));
        assert!(json_str.contains("views"));
        assert!(json_str.contains("total"));
    }
}
