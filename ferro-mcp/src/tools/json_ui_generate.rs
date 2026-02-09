//! JSON-UI generate tool - assembles context for creating new JSON-UI views
//!
//! This tool does NOT call any AI API. It provides structured context so the
//! consuming agent can write the view itself, avoiding double-LLM calls.

use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Complete context for generating a new JSON-UI view
#[derive(Debug, Serialize)]
pub struct JsonUiGenerationContext {
    /// Full component catalog text (all 20 components with props)
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

/// Concise reference of all 20 JSON-UI components with their props, types, and variants.
const COMPONENT_CATALOG: &str = r#"## Component Catalog

### Text
Props: content (String), element (h1|h2|h3|span|p)

### Button
Props: label (String), variant (default|secondary|destructive|outline|ghost|link), size (xs|sm|default|lg), disabled (Option<bool>), icon (Option<String>), icon_position (Option<left|right>)

### Card
Props: title (String), description (Option<String>), children (Vec<ComponentNode>), footer (Vec<ComponentNode>)

### Table
Props: columns (Vec<Column {key, label, format?}>), data_path (String), row_actions (Option<Vec<Action>>), empty_message (Option<String>), sortable (Option<bool>), sort_column (Option<String>), sort_direction (Option<asc|desc>)

### Form
Props: action (Action), fields (Vec<ComponentNode>), method (Option<GET|POST|PUT|PATCH|DELETE>)

### Input
Props: field (String), label (String), input_type (text|email|password|number|textarea|hidden|date|time|url|tel|search), placeholder (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>), description (Option<String>), default_value (Option<String>), data_path (Option<String>)

### Select
Props: field (String), label (String), options (Vec<SelectOption {value, label}>), placeholder (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>), description (Option<String>), default_value (Option<String>), data_path (Option<String>)

### Alert
Props: message (String), variant (info|success|warning|error), title (Option<String>)

### Badge
Props: label (String), variant (default|secondary|destructive|outline)

### Modal
Props: title (String), description (Option<String>), children (Vec<ComponentNode>), footer (Vec<ComponentNode>), trigger_label (Option<String>)

### Checkbox
Props: field (String), label (String), description (Option<String>), checked (Option<bool>), data_path (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>)

### Switch
Props: field (String), label (String), description (Option<String>), checked (Option<bool>), data_path (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>)

### Separator
Props: orientation (Option<horizontal|vertical>)

### DescriptionList
Props: items (Vec<DescriptionItem {label, value, format?}>), columns (Option<u8>)

### Tabs
Props: default_tab (String), tabs (Vec<Tab {value, label, children}>)

### Breadcrumb
Props: items (Vec<BreadcrumbItem {label, url?}>)

### Pagination
Props: current_page (u32), per_page (u32), total (u32), base_url (Option<String>)

### Progress
Props: value (u8 0-100), max (Option<u8>), label (Option<String>)

### Avatar
Props: src (Option<String>), alt (String), fallback (Option<String>), size (Option<xs|sm|default|lg>)

### Skeleton
Props: width (Option<String>), height (Option<String>), rounded (Option<bool>)

## Action
Props: handler (String "controller.method" format), method (GET|POST|PUT|PATCH|DELETE), confirm (Option<ConfirmDialog {title, message?, variant: default|danger}>), on_success (Option<ActionOutcome>), on_error (Option<ActionOutcome>)
Builders: Action::new("handler") (POST), Action::get("handler"), Action::delete("handler"), .confirm("title"), .confirm_danger("title")

## ComponentNode
Wraps every component: key (String), component (Component variant), action (Option<Action>), visibility (Option<Visibility>)

## JsonUiView Builder
JsonUiView::new().title("Title").layout("app").data(json).component(node).components(vec_of_nodes)
"#;

/// Complete example of a well-structured JSON-UI view
const VIEW_EXAMPLE: &str = r#"//! User List JSON-UI view

use ferro::{
    Action, Component, ComponentNode, JsonUiView, TableColumn, TableProps, TextElement, TextProps,
};

pub fn view() -> JsonUiView {
    JsonUiView::new()
        .title("User List")
        .layout("app")
        .component(ComponentNode {
            key: "heading".to_string(),
            component: Component::Text(TextProps {
                content: "User List".to_string(),
                element: TextElement::H1,
            }),
            action: None,
            visibility: None,
        })
        .component(ComponentNode {
            key: "users_table".to_string(),
            component: Component::Table(TableProps {
                columns: vec![
                    TableColumn { key: "name".to_string(), label: "Name".to_string(), format: None },
                    TableColumn { key: "email".to_string(), label: "Email".to_string(), format: None },
                ],
                data_path: "users".to_string(),
                row_actions: Some(vec![
                    Action::get("user_controller.edit"),
                    Action::delete("user_controller.destroy").confirm_danger("Delete user"),
                ]),
                empty_message: Some("No users found.".to_string()),
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        })
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
        component_catalog: COMPONENT_CATALOG.to_string(),
        models,
        routes,
        existing_views,
        example: VIEW_EXAMPLE.to_string(),
        conventions: ViewConventions {
            file_location: "src/views/{name}.rs".to_string(),
            function_signature: "pub fn view() -> JsonUiView".to_string(),
            import_pattern: "use ferro::{...};".to_string(),
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
            "pub fn view() -> JsonUiView"
        );
        assert_eq!(result.conventions.import_pattern, "use ferro::{...};");
        assert_eq!(result.conventions.layout_default, "app");
    }

    #[test]
    fn test_example_not_empty() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_generate_test");
        let result = execute(&non_existent, None, None);

        assert!(!result.example.is_empty());
        assert!(result.example.contains("JsonUiView"));
        assert!(result.example.contains("pub fn view()"));
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
                function_signature: "pub fn view() -> JsonUiView".to_string(),
                import_pattern: "use ferro::{...};".to_string(),
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
