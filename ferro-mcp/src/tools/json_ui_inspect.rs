//! JSON-UI inspect tool — discovers v2 JSON-UI views in a project
//! and inspects component schemas (including plugin components like Map).
//!
//! v2 scanner — reads `src/views/**/*.json` and parses flat `elements` maps.

use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// List of discovered JSON-UI views
#[derive(Debug, Serialize)]
pub struct JsonUiViewList {
    pub views: Vec<ViewInfo>,
    pub total: usize,
}

/// Metadata about a single JSON-UI view file
#[derive(Debug, Serialize)]
pub struct ViewInfo {
    /// File stem (e.g., "user_list" from "src/views/user_list.json")
    pub name: String,
    /// Relative path from project root (e.g., "src/views/user_list.json")
    pub file: String,
    /// Title from `spec["title"]`
    pub title: Option<String>,
    /// Layout from `spec["layout"]`
    pub layout: Option<String>,
    /// Component type names found in `elements[*].type` (deduplicated, sorted)
    pub components_used: Vec<String>,
    /// Action handler references found in `elements[*].action` fields
    pub actions: Vec<String>,
}

/// Schema information for a JSON-UI component (built-in or plugin).
#[derive(Debug, Serialize)]
pub struct ComponentSchemaInfo {
    /// Component type name (e.g., "Map").
    pub name: String,
    /// Whether this is a plugin component or built-in.
    pub is_plugin: bool,
    /// JSON Schema describing accepted props (plugin components only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props_schema: Option<serde_json::Value>,
    /// Prop descriptions from the catalog (built-in components).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_entry: Option<super::json_ui_catalog::CatalogComponent>,
}

/// Inspect a specific component type and return its schema.
///
/// For built-in components, returns the catalog entry from `json_ui_catalog`.
/// For plugin components (e.g., "Map"), queries the plugin registry and
/// returns the `props_schema` JSON Schema.
pub fn inspect_component(component_type: &str) -> ComponentSchemaInfo {
    use ferro_json_ui::global_catalog;
    let cat = global_catalog();

    // Check built-in catalog first
    let builtin = cat
        .components_sorted()
        .find(|spec| spec.name.eq_ignore_ascii_case(component_type));

    if builtin.is_some() {
        let catalog = super::json_ui_catalog::execute(Some(component_type));
        let entry = catalog.components.into_iter().next();
        ComponentSchemaInfo {
            name: component_type.to_string(),
            is_plugin: false,
            props_schema: None,
            catalog_entry: entry,
        }
    } else {
        // Check plugin registry
        let catalog = super::json_ui_catalog::execute(Some(component_type));
        let plugin_entry = catalog.plugin_components.into_iter().next();

        let schema = ferro_json_ui::with_plugin(component_type, |plugin| plugin.props_schema());

        ComponentSchemaInfo {
            name: component_type.to_string(),
            is_plugin: true,
            props_schema: schema,
            catalog_entry: plugin_entry,
        }
    }
}

/// Scan the project's `src/views/` directory for v2 JSON-UI spec files.
///
/// Reads each `*.json` file, parses the flat element map, and extracts
/// title, layout, component types, and action handler references.
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

    let mut views = scan_json_views(project_root, &views_dir);

    // Apply optional filter (case-insensitive substring match on name)
    if let Some(filter_str) = filter {
        let filter_lower = filter_str.to_lowercase();
        views.retain(|v| v.name.to_lowercase().contains(&filter_lower));
    }

    let total = views.len();
    JsonUiViewList { views, total }
}

/// Walk `views_dir` for `*.json` files and parse each as a v2 flat spec.
fn scan_json_views(project_root: &Path, views_dir: &Path) -> Vec<ViewInfo> {
    let entries = match fs::read_dir(views_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut views = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            // Recurse one level into subdirectories
            let sub_views = scan_json_views(project_root, &path);
            views.extend(sub_views);
            continue;
        }

        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let spec: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let title = spec
            .get("title")
            .and_then(|v| v.as_str())
            .map(String::from);

        let layout = spec
            .get("layout")
            .and_then(|v| v.as_str())
            .map(String::from);

        let (components_used, actions) = extract_elements_info(&spec);

        views.push(ViewInfo {
            name,
            file: relative_path,
            title,
            layout,
            components_used,
            actions,
        });
    }

    views.sort_by(|a, b| a.name.cmp(&b.name));
    views
}

/// Extract component types and action handlers from the flat `elements` map.
fn extract_elements_info(spec: &serde_json::Value) -> (Vec<String>, Vec<String>) {
    let Some(elements) = spec.get("elements").and_then(|v| v.as_object()) else {
        return (Vec::new(), Vec::new());
    };

    let mut component_types: HashSet<String> = HashSet::new();
    let mut actions: Vec<String> = Vec::new();

    for (_id, element) in elements {
        // Collect component type
        if let Some(type_name) = element.get("type").and_then(|v| v.as_str()) {
            component_types.insert(type_name.to_string());
        }

        // Collect action handler references
        if let Some(action) = element.get("action") {
            if let Some(handler) = action.get("handler").and_then(|v| v.as_str()) {
                let method = action
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("POST");
                actions.push(format!("{method} {handler}"));
            }
        }
    }

    let mut sorted_types: Vec<String> = component_types.into_iter().collect();
    sorted_types.sort();

    (sorted_types, actions)
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
                file: "src/views/user_list.json".to_string(),
                title: Some("User List".to_string()),
                layout: Some("dashboard".to_string()),
                components_used: vec!["Card".to_string(), "DataTable".to_string()],
                actions: vec!["GET users.index".to_string()],
            }],
            total: 1,
        };

        let json = serde_json::to_string(&list);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("user_list"));
        assert!(json_str.contains("User List"));
        assert!(json_str.contains("DataTable"));
        assert!(json_str.contains("views"));
        assert!(json_str.contains("total"));
    }

    #[test]
    fn test_parse_v2_spec() {
        use std::io::Write;
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let views_dir = tmp.path().join("src/views");
        fs::create_dir_all(&views_dir).unwrap();

        let spec = serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "title": "User List",
            "layout": "dashboard",
            "root": "root",
            "elements": {
                "root": {
                    "type": "Card",
                    "props": { "title": "User List" },
                    "children": ["table"]
                },
                "table": {
                    "type": "DataTable",
                    "props": { "data_path": "/data/users" },
                    "action": {
                        "handler": "users.store",
                        "method": "POST"
                    }
                }
            }
        });

        let spec_path = views_dir.join("user_list.json");
        let mut file = fs::File::create(&spec_path).unwrap();
        write!(file, "{}", serde_json::to_string(&spec).unwrap()).unwrap();

        let result = execute(tmp.path(), None);
        assert_eq!(result.total, 1);

        let view = &result.views[0];
        assert_eq!(view.name, "user_list");
        assert_eq!(view.title.as_deref(), Some("User List"));
        assert_eq!(view.layout.as_deref(), Some("dashboard"));
        assert!(view.components_used.contains(&"Card".to_string()));
        assert!(view.components_used.contains(&"DataTable".to_string()));
        assert!(view.actions.contains(&"POST users.store".to_string()));
    }

    #[test]
    fn test_filter_by_name() {
        use std::io::Write;
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let views_dir = tmp.path().join("src/views");
        fs::create_dir_all(&views_dir).unwrap();

        for name in &["user_list", "product_list", "dashboard"] {
            let spec = serde_json::json!({
                "$schema": "ferro-json-ui/v2",
                "title": name,
                "layout": "dashboard",
                "root": "root",
                "elements": {
                    "root": { "type": "Card", "props": {}, "children": [] }
                }
            });
            let path = views_dir.join(format!("{name}.json"));
            let mut file = fs::File::create(&path).unwrap();
            write!(file, "{}", serde_json::to_string(&spec).unwrap()).unwrap();
        }

        let result = execute(tmp.path(), Some("list"));
        assert_eq!(result.total, 2);
        let names: Vec<&str> = result.views.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"user_list"));
        assert!(names.contains(&"product_list"));
    }

    #[test]
    fn test_skips_non_json_files() {
        use std::io::Write;
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let views_dir = tmp.path().join("src/views");
        fs::create_dir_all(&views_dir).unwrap();

        // Write a .rs file that should be ignored
        let rs_path = views_dir.join("old_view.rs");
        let mut file = fs::File::create(&rs_path).unwrap();
        write!(file, "pub fn view() {{}}").unwrap();

        // Write a valid json spec
        let spec = serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "title": "Dashboard",
            "layout": "dashboard",
            "root": "root",
            "elements": {
                "root": { "type": "Card", "props": {}, "children": [] }
            }
        });
        let json_path = views_dir.join("dashboard.json");
        let mut file = fs::File::create(&json_path).unwrap();
        write!(file, "{}", serde_json::to_string(&spec).unwrap()).unwrap();

        let result = execute(tmp.path(), None);
        assert_eq!(result.total, 1);
        assert_eq!(result.views[0].name, "dashboard");
    }

    #[test]
    fn test_inspect_builtin_component() {
        let info = inspect_component("Button");
        assert_eq!(info.name, "Button");
        assert!(!info.is_plugin);
        assert!(info.props_schema.is_none());
        assert!(info.catalog_entry.is_some());
        assert_eq!(info.catalog_entry.unwrap().name, "Button");
    }

    #[test]
    fn test_inspect_plugin_component_map() {
        let info = inspect_component("Map");
        assert_eq!(info.name, "Map");
        assert!(info.is_plugin);
        assert!(info.props_schema.is_some());

        let schema = info.props_schema.unwrap();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["center"].is_object());
    }

    #[test]
    fn test_inspect_unknown_component() {
        let info = inspect_component("NonExistent");
        assert_eq!(info.name, "NonExistent");
        assert!(info.is_plugin);
        assert!(info.props_schema.is_none());
        assert!(info.catalog_entry.is_none());
    }

    #[test]
    fn test_inspect_component_serialization() {
        let info = inspect_component("Map");
        let json = serde_json::to_string(&info);
        assert!(json.is_ok(), "ComponentSchemaInfo should serialize to JSON");
        let json_str = json.unwrap();
        assert!(json_str.contains("Map"));
        assert!(json_str.contains("is_plugin"));
        assert!(json_str.contains("props_schema"));
    }
}
