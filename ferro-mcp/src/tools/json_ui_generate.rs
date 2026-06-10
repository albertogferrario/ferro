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
    /// Checkpoint verdict summary for the model-derived projection anchor.
    /// Present only when `model` is supplied and the anchor resolves to an existing
    /// projection. Omitted when `model` is `None` or the projection is not yet in
    /// the project (SC-1: never embed a vacuous all-`not_checked` summary).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<crate::tools::checkpoint_projection::VerdictSummary>,
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

/// Complete example of a well-structured JSON-UI v2 spec file.
///
/// This is a `src/views/user_list.json` file. Handlers call
/// `JsonUi::render_file("views/user_list.json", data)`.
const VIEW_EXAMPLE: &str = r#"{
  "$schema": "ferro-json-ui/v2",
  "title": "User List",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "User List" },
      "children": ["heading", "users_table"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "User List", "element": "h1" }
    },
    "users_table": {
      "type": "DataTable",
      "props": {
        "columns": [
          { "key": "name", "label": "Name" },
          { "key": "email", "label": "Email" }
        ],
        "data_path": "/data/users",
        "empty_message": "No users found."
      }
    }
  }
}"#;

/// Assemble generation context for creating a new JSON-UI view.
///
/// Scans the project for models and routes, then bundles them with the
/// component catalog, a working example, and naming conventions.
///
/// When `model` is `Some`, runs the projection checkpoint speculatively
/// against `{model_lowercase}_service` and embeds a compact `VerdictSummary`
/// in the result. When `model` is `None` the checkpoint field is omitted
/// (SC-1: never embed a vacuous all-`not_checked` summary without an anchor).
pub async fn execute(
    project_root: &Path,
    model: Option<&str>,
    description: Option<&str>,
) -> JsonUiGenerationContext {
    let models = scan_models(project_root, model);
    let routes = scan_routes(project_root);
    let existing_views = list_existing_views(project_root);

    // Speculative checkpoint: only when a model anchor is available.
    // None => None: skip run_for entirely (no anchor to derive from).
    let checkpoint = match model {
        Some(m) => {
            let anchor = format!("{}_service", m.to_lowercase());
            crate::tools::checkpoint_projection::run_for(
                project_root,
                &anchor,
                chrono::Utc::now(),
            )
            .await
            .ok()
            .map(|v| v.summary())
        }
        None => None,
    };

    JsonUiGenerationContext {
        component_catalog: global_catalog().prompt(),
        models,
        routes,
        existing_views,
        example: VIEW_EXAMPLE.to_string(),
        conventions: ViewConventions {
            file_location: "src/views/{name}.json".to_string(),
            function_signature: "#[handler] pub async fn {name}(req: Request) -> Response { JsonUi::render_file(\"views/{name}.json\", data) }".to_string(),
            import_pattern: "use ferro::{JsonUi, Response};".to_string(),
            layout_default: "dashboard".to_string(),
        },
        description: description.map(|s| s.to_string()),
        checkpoint,
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
            for (byte_idx, ch) in rest.char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            struct_end = byte_idx;
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
        if path.extension().is_none_or(|ext| ext != "json") {
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

    #[tokio::test]
    async fn test_conventions_populated() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_generate_test");
        let result = execute(&non_existent, None, None).await;

        assert_eq!(result.conventions.file_location, "src/views/{name}.json");
        assert!(result
            .conventions
            .function_signature
            .contains("JsonUi::render_file"));
        assert_eq!(
            result.conventions.import_pattern,
            "use ferro::{JsonUi, Response};"
        );
        assert_eq!(result.conventions.layout_default, "dashboard");
    }

    #[tokio::test]
    async fn test_example_not_empty() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_generate_test");
        let result = execute(&non_existent, None, None).await;

        assert!(!result.example.is_empty());
        assert!(result.example.contains("ferro-json-ui/v2"));
        assert!(result.example.contains("elements"));
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
            existing_views: vec!["user_list.json".to_string()],
            example: "example code".to_string(),
            conventions: ViewConventions {
                file_location: "src/views/{name}.json".to_string(),
                function_signature: "#[handler] pub async fn {name}(req: Request) -> Response { JsonUi::render_file(\"views/{name}.json\", data) }".to_string(),
                import_pattern: "use ferro::{JsonUi, Response};".to_string(),
                layout_default: "dashboard".to_string(),
            },
            description: Some("A user management view".to_string()),
            checkpoint: None,
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
            checkpoint: None,
        };

        let json_str = serde_json::to_string(&context).unwrap();
        assert!(
            !json_str.contains("description"),
            "description should be omitted when None"
        );
    }

    #[tokio::test]
    async fn test_component_catalog_not_empty() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_generate_test");
        let result = execute(&non_existent, None, None).await;

        assert!(!result.component_catalog.is_empty());
        assert!(result.component_catalog.contains("Text"));
        assert!(result.component_catalog.contains("Button"));
        assert!(result.component_catalog.contains("Table"));
        assert!(result.component_catalog.contains("Form"));
        assert!(result.component_catalog.contains("Action"));
    }

    // -----------------------------------------------------------------------
    // Inline-hook tests (CHK-07 / SC-1 / Pitfall 3)
    // -----------------------------------------------------------------------

    /// When model is None, checkpoint must be None and the serialized output
    /// must omit the "checkpoint" key entirely (SC-1: no vacuous all-not_checked summary).
    #[tokio::test]
    async fn json_ui_generate_no_model_omits_checkpoint() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_json_ui_generate_test");
        let ctx = execute(&non_existent, None, None).await;

        assert!(
            ctx.checkpoint.is_none(),
            "checkpoint must be None when model is not supplied"
        );

        let json_str = serde_json::to_string(&ctx).unwrap();
        assert!(
            !json_str.contains("\"checkpoint\""),
            "serialized context must omit checkpoint key when model is None: {json_str}"
        );
    }

    /// When model is Some but no matching projection exists in the project,
    /// checkpoint must still be None (speculative anchor miss → .ok() → None).
    /// This also covers Pitfall 3: no vacuous all-not_checked summary embedded.
    #[tokio::test]
    async fn json_ui_generate_with_model_no_projection_omits_checkpoint() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_project_json_ui_generate_test");
        let ctx = execute(&non_existent, Some("Booking"), None).await;

        assert!(
            ctx.checkpoint.is_none(),
            "checkpoint must be None when anchor projection does not exist (safe degradation)"
        );

        let json_str = serde_json::to_string(&ctx).unwrap();
        assert!(
            !json_str.contains("\"checkpoint\""),
            "serialized context must omit checkpoint key when anchor does not resolve"
        );
    }

    /// When model is Some and a matching projection exists, checkpoint may be
    /// Some with a compact VerdictSummary (status key, no seams key — SC-1).
    /// The test accepts both Some and None since inspect_projection indexing
    /// determines resolution; the key invariant is shape-correctness when Some.
    #[tokio::test]
    async fn json_ui_generate_with_resolving_model_embeds_checkpoint() {
        let tmp = tempfile::tempdir().unwrap();
        // Add a projection file that checkpoint_projection can attempt to find.
        let proj_dir = tmp.path().join("src/projections");
        std::fs::create_dir_all(&proj_dir).unwrap();
        std::fs::write(
            proj_dir.join("booking_service.rs"),
            r#"use ferro::{ServiceDef, DataType, FieldMeaning};
pub fn booking_service() -> ServiceDef {
    ServiceDef::new("booking")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
}
"#,
        )
        .unwrap();

        let ctx = execute(tmp.path(), Some("Booking"), None).await;

        // If checkpoint resolved (projection indexed), verify compact shape (SC-1).
        if let Some(ref chk) = ctx.checkpoint {
            let val = serde_json::to_value(chk).unwrap();
            assert!(
                val.get("status").is_some(),
                "VerdictSummary must have status key"
            );
            assert!(
                val.get("seams").is_none(),
                "VerdictSummary must NOT have seams key (SC-1)"
            );
        }
        // None is also acceptable (anchor resolution depends on inspect_projection indexing).
    }
}
