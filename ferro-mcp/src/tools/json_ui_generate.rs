//! JSON-UI generate tool - assembles context for creating new JSON-UI views
//!
//! This tool does NOT call any AI API. It provides structured context so the
//! consuming agent can write the view itself, avoiding double-LLM calls.

use ferro_json_ui::global_catalog;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Complete context for generating a new JSON-UI view
#[derive(Debug, Serialize)]
pub struct JsonUiGenerationContext {
    /// Full component catalog text (20 built-in + plugin components with props)
    pub component_catalog: String,
    /// Models discovered in the project
    pub models: Vec<ModelContext>,
    /// Routes discovered in the project
    pub routes: Vec<RouteContext>,
    /// File names of existing views in src/views/
    pub existing_views: Vec<String>,
    /// Complete example of a well-structured view
    pub example: String,
    /// Naming and structure conventions
    pub conventions: ViewConventions,
    /// Optional view description passed through from input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A model with its fields for context
#[derive(Debug, Serialize)]
pub struct ModelContext {
    pub name: String,
    pub fields: Vec<FieldContext>,
}

/// A single model field
#[derive(Debug, Serialize)]
pub struct FieldContext {
    pub name: String,
    pub type_name: String,
}

/// A route definition for context
#[derive(Debug, Serialize)]
pub struct RouteContext {
    pub method: String,
    pub path: String,
    pub handler: String,
}

/// Conventions for JSON-UI view files
#[derive(Debug, Serialize)]
pub struct ViewConventions {
    /// Where view files go
    pub file_location: String,
    /// Standard function signature
    pub function_signature: String,
    /// Standard import pattern
    pub import_pattern: String,
    /// Default layout name
    pub layout_default: String,
}

/// Complete example of a well-structured JSON-UI view
const VIEW_EXAMPLE: &str = r#"//! User List JSON-UI view

use ferro::{Action, Spec, Element, JsonUi, Response};

pub async fn view() -> Response {
    let spec = Spec::builder()
        .title("User List")
        .layout("dashboard")
        .element(
            "root",
            Element::new("Card")
                .prop("title", "User List")
                .child("heading")
                .child("users_table"),
        )
        .element(
            "heading",
            Element::new("Text")
                .prop("content", "User List")
                .prop("element", "h1"),
        )
        .element(
            "users_table",
            Element::new("DataTable")
                .prop(
                    "columns",
                    serde_json::json!([
                        {"key": "name", "label": "Name"},
                        {"key": "email", "label": "Email"},
                    ]),
                )
                .prop("data_path", "/data/users")
                .prop("empty_message", "No users found."),
        )
        .build()
        .expect("spec is valid");

    JsonUi::render(&spec, &serde_json::json!({}))
}
"#;

/// Assemble generation context for creating a new JSON-UI view.
///
/// Scans the project for models and routes, then bundles them with the
/// component catalog, a working example, and naming conventions.
pub fn execute(
    project_root: &Path,
    model: Option<&str>,
    description: Option<&str>,
) -> JsonUiGenerationContext {
    let models = scan_models(project_root, model);
    let routes = scan_routes(project_root);
    let existing_views = list_existing_views(project_root);

    JsonUiGenerationContext {
        component_catalog: global_catalog().prompt(),
        models,
        routes,
        existing_views,
        example: VIEW_EXAMPLE.to_string(),
        conventions: ViewConventions {
            file_location: "src/views/{name}.rs".to_string(),
            function_signature: "pub async fn view() -> Response".to_string(),
            import_pattern: "use ferro::{Spec, Element, JsonUi, Response, ...};".to_string(),
            layout_default: "app".to_string(),
        },
        description: description.map(|s| s.to_string()),
    }
}

/// Scan `src/models/*.rs` and extract struct fields using regex.
fn scan_models(project_root: &Path, filter_model: Option<&str>) -> Vec<ModelContext> {
    let models_dir = project_root.join("src/models");
    if !models_dir.exists() {
        return Vec::new();
    }

    let struct_re = Regex::new(r"pub\s+struct\s+(\w+)\s*\{").unwrap();
    let field_re = Regex::new(r"pub\s+(\w+)\s*:\s*([^,\n]+)").unwrap();

    let mut models = Vec::new();

    let entries: Vec<_> = match fs::read_dir(&models_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return Vec::new(),
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

        for struct_cap in struct_re.captures_iter(&content) {
            let struct_name = struct_cap[1].to_string();

            // If a model filter is provided, only include that model
            if let Some(filter) = filter_model {
                if !struct_name.eq_ignore_ascii_case(filter) {
                    continue;
                }
            }

            let struct_start = struct_cap.get(0).unwrap().end();
            let rest = &content[struct_start..];

            // Find closing brace
            let mut depth = 1;
            let mut struct_end = rest.len();
            for (i, ch) in rest.chars().enumerate() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            struct_end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let struct_body = &rest[..struct_end];
            let fields: Vec<FieldContext> = field_re
                .captures_iter(struct_body)
                .map(|cap| FieldContext {
                    name: cap[1].trim().to_string(),
                    type_name: cap[2].trim().trim_end_matches(',').to_string(),
                })
                .collect();

            if !fields.is_empty() {
                models.push(ModelContext {
                    name: struct_name,
                    fields,
                });
            }
        }
    }

    models
}

/// Scan `src/routes.rs` and extract route definitions using regex.
fn scan_routes(project_root: &Path) -> Vec<RouteContext> {
    let routes_file = project_root.join("src/routes.rs");
    if !routes_file.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(routes_file) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let route_re =
        Regex::new(r#"\.(get|post|put|patch|delete)\("([^"]+)".*?(\w+)::(\w+)\)"#).unwrap();

    let mut routes = Vec::new();

    for cap in route_re.captures_iter(&content) {
        routes.push(RouteContext {
            method: cap[1].to_uppercase(),
            path: cap[2].to_string(),
            handler: format!("{}::{}", &cap[3], &cap[4]),
        });
    }

    routes
}

/// List existing view file names in src/views/ (just names, not full inspection).
fn list_existing_views(project_root: &Path) -> Vec<String> {
    let views_dir = project_root.join("src/views");
    if !views_dir.exists() {
        return Vec::new();
    }

    let mut views = Vec::new();

    let entries: Vec<_> = match fs::read_dir(&views_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return Vec::new(),
    };

    for entry in entries {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        if path.file_name().is_some_and(|n| n == "mod.rs") {
            continue;
        }
        if let Some(name) = path.file_name() {
            views.push(name.to_string_lossy().to_string());
        }
    }

    views.sort();
    views
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_conventions_populated() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_generate_test");
        let result = execute(&non_existent, None, None);

        assert_eq!(result.conventions.file_location, "src/views/{name}.rs");
        assert_eq!(
            result.conventions.function_signature,
            "pub async fn view() -> Response"
        );
        assert_eq!(
            result.conventions.import_pattern,
            "use ferro::{Spec, Element, JsonUi, Response, ...};"
        );
        assert_eq!(result.conventions.layout_default, "app");
    }

    #[test]
    fn test_example_not_empty() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_generate_test");
        let result = execute(&non_existent, None, None);

        assert!(!result.example.is_empty());
        assert!(result.example.contains("Spec::builder()"));
        assert!(result.example.contains("pub async fn view()"));
    }

    #[test]
    fn test_serialization() {
        let context = JsonUiGenerationContext {
            component_catalog: "test catalog".to_string(),
            models: vec![ModelContext {
                name: "User".to_string(),
                fields: vec![FieldContext {
                    name: "email".to_string(),
                    type_name: "String".to_string(),
                }],
            }],
            routes: vec![RouteContext {
                method: "GET".to_string(),
                path: "/users".to_string(),
                handler: "users::index".to_string(),
            }],
            existing_views: vec!["user_list.rs".to_string()],
            example: "example code".to_string(),
            conventions: ViewConventions {
                file_location: "src/views/{name}.rs".to_string(),
                function_signature: "pub async fn view() -> Response".to_string(),
                import_pattern: "use ferro::{Spec, Element, JsonUi, Response, ...};".to_string(),
                layout_default: "app".to_string(),
            },
            description: Some("A user management view".to_string()),
        };

        let json = serde_json::to_string(&context);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("component_catalog"));
        assert!(json_str.contains("models"));
        assert!(json_str.contains("routes"));
        assert!(json_str.contains("existing_views"));
        assert!(json_str.contains("conventions"));
        assert!(json_str.contains("description"));
    }

    #[test]
    fn test_description_omitted_when_none() {
        let context = JsonUiGenerationContext {
            component_catalog: String::new(),
            models: Vec::new(),
            routes: Vec::new(),
            existing_views: Vec::new(),
            example: String::new(),
            conventions: ViewConventions {
                file_location: String::new(),
                function_signature: String::new(),
                import_pattern: String::new(),
                layout_default: String::new(),
            },
            description: None,
        };

        let json_str = serde_json::to_string(&context).unwrap();
        assert!(
            !json_str.contains("description"),
            "description should be omitted when None"
        );
    }

    #[test]
    fn test_component_catalog_not_empty() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_generate_test");
        let result = execute(&non_existent, None, None);

        assert!(!result.component_catalog.is_empty());
        assert!(result.component_catalog.contains("Text"));
        assert!(result.component_catalog.contains("Button"));
        assert!(result.component_catalog.contains("Table"));
        assert!(result.component_catalog.contains("Form"));
        assert!(result.component_catalog.contains("Action"));
    }
}
