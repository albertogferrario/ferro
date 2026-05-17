//! JSON-UI inspect tool — discovers existing JSON-UI v2 spec files in a project
//! and inspects component schemas (including plugin components like Map).
//!
//! Scans `src/views/*.json` files (v2 flat spec format). Returns an empty list
//! for projects with no `src/views/` directory or no `.json` files (not an error).

use serde::Serialize;
use std::fs;
use std::path::Path;

/// List of discovered JSON-UI views
#[derive(Debug, Serialize)]
pub struct JsonUiViewList {
    pub views: Vec<ViewInfo>,
    pub total: usize,
}

/// Metadata about a single JSON-UI v2 spec file
#[derive(Debug, Serialize)]
pub struct ViewInfo {
    /// File stem (e.g., "user_list" from "user_list.json")
    pub name: String,
    /// Relative path from project root (e.g., "src/views/user_list.json")
    pub file: String,
    /// Value of the top-level `title` field in the spec
    pub title: Option<String>,
    /// Value of the top-level `layout` field in the spec
    pub layout: Option<String>,
    /// Deduplicated, sorted list of `type` values from all elements
    pub components_used: Vec<String>,
    /// Action handler references found in element `action` fields
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

    // Check if it's a built-in (non-plugin) component by iterating components_sorted
    let is_builtin = cat
        .components_sorted()
        .any(|spec| spec.name.eq_ignore_ascii_case(component_type));

    if is_builtin {
        let catalog = super::json_ui_catalog::execute(Some(component_type));
        let entry = catalog.components.into_iter().next();
        return ComponentSchemaInfo {
            name: component_type.to_string(),
            is_plugin: false,
            props_schema: None,
            catalog_entry: entry,
        };
    }

    // Check plugin registry
    let catalog = super::json_ui_catalog::execute(Some(component_type));
    let plugin_entry = catalog.plugin_components.into_iter().next();
    if plugin_entry.is_some() {
        let schema = ferro_json_ui::with_plugin(component_type, |plugin| plugin.props_schema());
        return ComponentSchemaInfo {
            name: component_type.to_string(),
            is_plugin: true,
            props_schema: schema,
            catalog_entry: plugin_entry,
        };
    }

    // Unknown — no matching built-in or plugin
    ComponentSchemaInfo {
        name: component_type.to_string(),
        is_plugin: false,
        props_schema: None,
        catalog_entry: None,
    }
}

/// Scan the project's `src/views/` directory for JSON-UI v2 spec files.
///
/// Reads each `.json` file, parses it as a JSON value, and extracts:
/// - `title` and `layout` from top-level fields
/// - `components_used` from `elements[*].type` values (deduplicated, sorted)
/// - `actions` from `elements[*].action.handler` values
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

/// Walk `views_dir` and parse each `.json` file into a [`ViewInfo`].
fn scan_json_views(project_root: &Path, views_dir: &Path) -> Vec<ViewInfo> {
    let entries: Vec<_> = match fs::read_dir(views_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return Vec::new(),
    };

    let mut views = Vec::new();

    for entry in entries {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }

        let name = match path.file_stem() {
            Some(stem) => stem.to_string_lossy().to_string(),
            None => continue,
        };

        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let spec: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let title = spec["title"].as_str().map(String::from);
        let layout = spec["layout"].as_str().map(String::from);

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

/// Extract component type names and action handlers from a spec's `elements` map.
fn extract_elements_info(spec: &serde_json::Value) -> (Vec<String>, Vec<String>) {
    let Some(elements) = spec["elements"].as_object() else {
        return (Vec::new(), Vec::new());
    };

    let mut component_types = std::collections::BTreeSet::new();
    let mut actions = Vec::new();

    for element in elements.values() {
        if let Some(type_name) = element["type"].as_str() {
            component_types.insert(type_name.to_string());
        }

        // action.handler field (string)
        if let Some(handler) = element["action"]["handler"].as_str() {
            actions.push(handler.to_string());
        }
    }

    let components_used: Vec<String> = component_types.into_iter().collect();
    actions.sort();
    actions.dedup();

    (components_used, actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        let mut f = fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_empty_project() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_test");
        let result = execute(&non_existent, None);
        assert_eq!(result.total, 0);
        assert!(result.views.is_empty());
    }

    #[test]
    fn test_scan_json_views() {
        let tmp = TempDir::new().unwrap();
        let views_dir = tmp.path().join("src/views");
        fs::create_dir_all(&views_dir).unwrap();

        write_file(
            &views_dir,
            "user_list.json",
            r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "User List",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "User List" },
      "children": ["heading", "table"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "Users", "element": "h1" }
    },
    "table": {
      "type": "DataTable",
      "props": { "data_path": "/data/users" },
      "action": { "handler": "users.index", "method": "GET" }
    }
  }
}"#,
        );

        let result = execute(tmp.path(), None);
        assert_eq!(result.total, 1);

        let view = &result.views[0];
        assert_eq!(view.name, "user_list");
        assert_eq!(view.title.as_deref(), Some("User List"));
        assert_eq!(view.layout.as_deref(), Some("dashboard"));

        let mut expected_components = vec!["Card", "DataTable", "Text"];
        expected_components.sort();
        let mut actual = view.components_used.clone();
        actual.sort();
        assert_eq!(actual, expected_components);

        assert!(view.actions.contains(&"users.index".to_string()));
    }

    #[test]
    fn test_filter_by_name() {
        let tmp = TempDir::new().unwrap();
        let views_dir = tmp.path().join("src/views");
        fs::create_dir_all(&views_dir).unwrap();

        write_file(
            &views_dir,
            "user_list.json",
            r#"{"$schema":"ferro-json-ui/v2","title":"Users","layout":"dashboard","root":"root","elements":{"root":{"type":"Card","props":{}}}}"#,
        );
        write_file(
            &views_dir,
            "product_list.json",
            r#"{"$schema":"ferro-json-ui/v2","title":"Products","layout":"dashboard","root":"root","elements":{"root":{"type":"Card","props":{}}}}"#,
        );

        let result = execute(tmp.path(), Some("user"));
        assert_eq!(result.total, 1);
        assert_eq!(result.views[0].name, "user_list");
    }

    #[test]
    fn test_ignores_non_json_files() {
        let tmp = TempDir::new().unwrap();
        let views_dir = tmp.path().join("src/views");
        fs::create_dir_all(&views_dir).unwrap();

        write_file(&views_dir, "stale_artifact.rs", "// non-JSON artifact");
        write_file(&views_dir, "mod.rs", "pub mod stale_artifact;");

        let result = execute(tmp.path(), None);
        assert_eq!(result.total, 0);
    }

    #[test]
    fn test_serialization() {
        let list = JsonUiViewList {
            views: vec![ViewInfo {
                name: "user_list".to_string(),
                file: "src/views/user_list.json".to_string(),
                title: Some("User List".to_string()),
                layout: Some("dashboard".to_string()),
                components_used: vec!["DataTable".to_string(), "Text".to_string()],
                actions: vec!["users.index".to_string()],
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
        assert!(!info.is_plugin);
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
