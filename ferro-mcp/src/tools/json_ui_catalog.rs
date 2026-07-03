//! JSON-UI component catalog tool — structured reference of all built-in
//! components plus plugin components (Map, etc.).
//!
//! Component data is derived from [`ferro_json_ui::global_catalog()`] (Phase 117).
//! The public [`JsonUiCatalog`] / [`CatalogComponent`] struct shape is preserved
//! (CONTEXT D-24). The hand-maintained `BUILDER_API` and `ACTION_API` strings stay.

use serde::Serialize;

/// Complete catalog of JSON-UI components with builder and action API.
#[derive(Debug, Serialize)]
pub struct JsonUiCatalog {
    pub components: Vec<CatalogComponent>,
    pub plugin_components: Vec<CatalogComponent>,
    pub builder_api: String,
    pub action_api: String,
    /// Full spec JSON Schema (suitable for schema-based validation).
    pub json_schema: serde_json::Value,
    /// Per-component props JSON Schema, keyed by type name.
    pub component_schemas: std::collections::HashMap<String, serde_json::Value>,
    /// Spec-level directives recognized by `ferro-json-ui` resolve pipeline
    /// (Phase 163: `$each`, `$if`).
    pub directives: Vec<DirectiveInfo>,
}

/// A spec-level directive (e.g., `$each`, `$if`) discoverable by agents.
///
/// Directives are JSON keys placed on an [`ferro_json_ui::Element`] object
/// alongside `type`, `props`, etc. The resolve pipeline (Phase 163) expands
/// directives at request time before render.
#[derive(Debug, Serialize)]
pub struct DirectiveInfo {
    /// Wire-format directive name including the `$` prefix (e.g. `"$each"`).
    pub name: String,
    /// Short one-line description (agent-facing).
    pub description: String,
    /// JSON snippet showing the directive's wire-format usage.
    pub syntax_example: String,
    /// Names of `SpecError` variants that can fire when the directive is
    /// malformed (cross-reference for diagnostic output).
    pub validation_errors: Vec<String>,
}

/// A single component in the catalog.
#[derive(Debug, Serialize)]
pub struct CatalogComponent {
    pub name: String,
    pub description: String,
    pub props: Vec<PropInfo>,
    pub variants: Option<Vec<String>>,
}

/// A prop on a component.
#[derive(Debug, Serialize)]
pub struct PropInfo {
    pub name: String,
    pub type_name: String,
    pub required: bool,
    pub description: String,
}

/// Execute the JSON-UI catalog tool.
///
/// When `component` is Some, returns only the matching component (case-insensitive).
/// When None, returns the full catalog.
///
/// Component data is sourced from [`ferro_json_ui::global_catalog()`] (Phase 117).
/// The public struct shape (`components`, `plugin_components`, `builder_api`,
/// `action_api`) is preserved (CONTEXT D-24).
pub fn execute(component: Option<&str>) -> JsonUiCatalog {
    use ferro_json_ui::global_catalog;
    let cat = global_catalog();

    let to_catalog_component = |spec: &ferro_json_ui::ComponentSpec| -> CatalogComponent {
        CatalogComponent {
            name: spec.name.clone(),
            description: spec.description.clone(),
            props: derive_prop_infos(&spec.props_schema),
            variants: derive_variants(&spec.props_schema),
        }
    };

    let components: Vec<CatalogComponent> = cat
        .components_sorted()
        .map(to_catalog_component)
        .filter(|c| {
            component
                .map(|needle| c.name.to_lowercase() == needle.to_lowercase())
                .unwrap_or(true)
        })
        .collect();

    let plugin_components: Vec<CatalogComponent> = cat
        .plugin_components_sorted()
        .map(to_catalog_component)
        .filter(|c| {
            component
                .map(|needle| c.name.to_lowercase() == needle.to_lowercase())
                .unwrap_or(true)
        })
        .collect();

    // Build component_schemas from all built-in and plugin component specs
    let component_schemas: std::collections::HashMap<String, serde_json::Value> = cat
        .components_sorted()
        .chain(cat.plugin_components_sorted())
        .filter_map(|spec| {
            cat.component_schema(&spec.name)
                .cloned()
                .map(|schema| (spec.name.clone(), schema))
        })
        .collect();

    let directives = vec![
        DirectiveInfo {
            name: "$each".to_string(),
            description:
                "Iterate over a JSON array in spec.data, producing one element per row \
                 with auto-suffixed IDs `{id}-0`, `{id}-1`, ... Loop variable bound by \
                 `as` scopes `$data` paths starting with `/{as}/...` to the current row."
                    .to_string(),
            syntax_example: r#"{"type":"Card","$each":{"path":"/orders","as":"order"},"props":{"title":{"$data":"/order/order_number"}}}"#.to_string(),
            validation_errors: vec![
                "EachPathNotArray".to_string(),
                "EachAsReservedName".to_string(),
                "NestedEach".to_string(),
                "MismatchedEach".to_string(),
            ],
        },
        DirectiveInfo {
            name: "$if".to_string(),
            description:
                "Conditional element emission. Falsy predicates REMOVE the element from \
                 the spec at resolve time (no hidden DOM, distinct from `visible` which \
                 renders hidden). Predicate reuses the `Visibility` evaluator (And/Or/Not \
                 supported)."
                    .to_string(),
            syntax_example: r#"{"type":"Button","$if":{"path":"/can_advance","operator":"eq","value":true},"props":{"label":"Advance"}}"#.to_string(),
            validation_errors: vec!["IfPathMissing".to_string()],
        },
    ];

    JsonUiCatalog {
        components,
        plugin_components,
        builder_api: BUILDER_API.to_string(),
        action_api: ACTION_API.to_string(),
        json_schema: cat.json_schema().clone(),
        component_schemas,
        directives,
    }
}

/// Derive [`PropInfo`] entries from a schemars-generated Props JSON Schema.
fn derive_prop_infos(schema: &serde_json::Value) -> Vec<PropInfo> {
    let Some(obj) = schema.as_object() else {
        return Vec::new();
    };
    let Some(props) = obj.get("properties").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let required: std::collections::HashSet<&str> = obj
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    props
        .iter()
        .map(|(name, field)| PropInfo {
            name: name.clone(),
            type_name: schema_type_hint(field),
            required: required.contains(name.as_str()),
            description: field
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect()
}

/// Derive a type-hint string from a field's JSON Schema fragment.
fn schema_type_hint(field: &serde_json::Value) -> String {
    // Prefer enum → pipe-joined variants
    if let Some(variants) = field.get("enum").and_then(|v| v.as_array()) {
        let names: Vec<String> = variants
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        if !names.is_empty() {
            return names.join("|");
        }
    }
    // anyOf / oneOf with null → Option<T>
    for key in ["anyOf", "oneOf"] {
        if let Some(arr) = field.get(key).and_then(|v| v.as_array()) {
            let has_null = arr
                .iter()
                .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("null"));
            let non_null: Vec<&serde_json::Value> = arr
                .iter()
                .filter(|v| v.get("type").and_then(|t| t.as_str()) != Some("null"))
                .collect();
            if has_null && non_null.len() == 1 {
                return format!("Option<{}>", schema_type_hint(non_null[0]));
            }
        }
    }
    // Plain type string
    if let Some(t) = field.get("type").and_then(|v| v.as_str()) {
        return t.to_string();
    }
    // Fallback
    serde_json::to_string(field).unwrap_or_default()
}

/// Derive top-level enum variants from a Props schema (e.g. a pure-enum Props type).
///
/// Returns `None` for struct-shaped Props (the common case). Struct Props components
/// like Button expose their enum fields via [`PropInfo::type_name`] instead.
fn derive_variants(schema: &serde_json::Value) -> Option<Vec<String>> {
    // oneOf of const strings (schemars enum pattern at the top level)
    if let Some(arr) = schema.get("oneOf").and_then(|v| v.as_array()) {
        let names: Vec<String> = arr
            .iter()
            .filter_map(|v| v.get("const").and_then(|c| c.as_str()).map(String::from))
            .collect();
        if !names.is_empty() {
            return Some(names);
        }
    }
    None
}

const BUILDER_API: &str = "\
Spec::builder() -> SpecBuilder
  .title(impl Into<String>) -> Self
  .layout(impl Into<String>) -> Self
  .data(serde_json::Value) -> Self
  .element(id, Element) -> Self
  .build() -> Result<Spec, SpecError>

Element::new(type_name: impl Into<String>) -> ElementBuilder
  .prop(key, value) -> Self (accumulates into props: serde_json::Value)
  .child(id: impl Into<String>) -> Self (child element id reference)
  .action(Action) -> Self (click/submit handler)
  .visible(Visibility) -> Self (show/hide based on data path)

Spec { $schema, root, elements: HashMap<String, Element>, title?, layout?, data? }
  - $schema: \"ferro-json-ui/v2\"
  - root: id of the root element
  - elements: flat map of element id -> Element
Element { type: String, props: Value, children: Vec<String>, action?, visible? }
  - type: component type name (e.g. \"Card\", \"DataTable\", \"Map\")
  - props: component-specific properties as a JSON value
  - children: element id references (no nested structures — flat lookup)
  - action: optional Action binding
  - visible: optional Visibility rule";

const ACTION_API: &str = "\
Action::new(handler) -> Action (POST)
Action::get(handler) -> Action (GET)
Action::delete(handler) -> Action (DELETE)
  .method(HttpMethod) -> Self
  .confirm(title) -> Self (neutral dialog)
  .confirm_danger(title) -> Self (destructive dialog)
  .on_success(ActionOutcome) -> Self
  .on_error(ActionOutcome) -> Self

Handler format: \"controller.method\" (e.g., \"users.store\")

ActionOutcome variants:
  Redirect { url: String }
  ShowErrors
  Refresh
  Notify { message: String, tone: Tone (neutral|success|warning|destructive) }

ConfirmDialog { title: String, message: Option<String>, tone: Tone (neutral|destructive) }";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_components_present() {
        let catalog = execute(None);
        // Cross-crate mirror of the builtin count. The canonical source-of-truth
        // tripwire is ferro_json_ui::catalog::tests::builtin_types_count_drift_guard;
        // bump this in step with it when the builtin set changes (BUILTIN_TYPES is
        // pub(crate) in ferro-json-ui, so this side can't assert it relationally).
        assert_eq!(
            catalog.components.len(),
            47,
            "Catalog should contain all 47 built-in components (incl. SegmentedControl, SidebarLayout, ActionGroup), got {}",
            catalog.components.len()
        );

        let names: Vec<&str> = catalog.components.iter().map(|c| c.name.as_str()).collect();
        let expected = [
            "Text",
            "Button",
            "Card",
            "Table",
            "Form",
            "Input",
            "Select",
            "Alert",
            "Badge",
            "Modal",
            "Checkbox",
            "CheckboxList",
            "CheckboxGroup",
            "Switch",
            "Separator",
            "DescriptionList",
            "Tabs",
            "Breadcrumb",
            "Pagination",
            "Progress",
            "Avatar",
            "Skeleton",
            "StatCard",
            "Checklist",
            "Toast",
            "NotificationDropdown",
            "Sidebar",
            "Header",
            "Grid",
            "Collapsible",
            "EmptyState",
            "FormSection",
            "PageHeader",
            "ButtonGroup",
            "ActionGroup",
            "DataTable",
            "KanbanBoard",
            "CalendarCell",
            "ActionCard",
            "ProductTile",
            "RawHtml",
            "StreamText",
            "Image",
            "DetailPage",
            "MediaCardGrid",
            "SegmentedControl",
            "SidebarLayout",
        ];
        for name in &expected {
            assert!(names.contains(name), "Missing component: {name}");
        }
    }

    #[test]
    fn test_plugin_components_present() {
        let catalog = execute(None);
        assert_eq!(
            catalog.plugin_components.len(),
            2,
            "Catalog should contain 2 plugin components (Map + RichTextEditor), got {}",
            catalog.plugin_components.len()
        );
        let plugin_names: Vec<&str> = catalog
            .plugin_components
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert!(
            plugin_names.contains(&"Map"),
            "Plugin components should include Map"
        );
        assert!(
            plugin_names.contains(&"RichTextEditor"),
            "Plugin components should include RichTextEditor"
        );
    }

    #[test]
    fn test_filter_by_component() {
        let catalog = execute(Some("Button"));
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].name, "Button");
        assert!(catalog.plugin_components.is_empty());
    }

    #[test]
    fn test_filter_by_plugin_component() {
        let catalog = execute(Some("Map"));
        assert!(catalog.components.is_empty());
        assert_eq!(catalog.plugin_components.len(), 1);
        assert_eq!(catalog.plugin_components[0].name, "Map");
    }

    #[test]
    fn test_filter_case_insensitive() {
        let catalog = execute(Some("button"));
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].name, "Button");

        let catalog = execute(Some("CARD"));
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].name, "Card");
    }

    #[test]
    fn test_unknown_component_returns_empty() {
        let catalog = execute(Some("NonExistent"));
        assert!(
            catalog.components.is_empty(),
            "Unknown component should return empty list"
        );
    }

    #[test]
    fn test_serialization() {
        let catalog = execute(None);
        let json = serde_json::to_string(&catalog);
        assert!(json.is_ok(), "Catalog should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("components"));
        assert!(json_str.contains("plugin_components"));
        assert!(json_str.contains("builder_api"));
        assert!(json_str.contains("action_api"));
        assert!(json_str.contains("json_schema"));
        assert!(json_str.contains("component_schemas"));
        assert!(json_str.contains("Button"));
        assert!(json_str.contains("Map"));
        assert!(json_str.contains("props"));
    }

    #[test]
    fn test_json_schema_present() {
        let catalog = execute(None);
        // Full spec schema must be a non-null JSON object
        assert!(
            catalog.json_schema.is_object(),
            "json_schema should be a JSON object"
        );
    }

    #[test]
    fn test_component_schemas_present() {
        let catalog = execute(None);
        // Should have at least one entry per built-in component
        assert!(
            !catalog.component_schemas.is_empty(),
            "component_schemas should not be empty"
        );
        assert!(
            catalog.component_schemas.contains_key("Button"),
            "component_schemas should contain Button"
        );
    }

    #[test]
    fn test_builder_api_present() {
        let catalog = execute(None);
        assert!(
            catalog.builder_api.contains("Spec::builder()"),
            "Builder API should document Spec::builder()"
        );
        assert!(
            catalog.builder_api.contains("Element::new"),
            "Builder API should document Element::new"
        );
    }

    #[test]
    fn test_action_api_present() {
        let catalog = execute(None);
        assert!(
            catalog.action_api.contains("Action::new"),
            "Action API should document Action::new"
        );
        assert!(
            catalog.action_api.contains("Action::get"),
            "Action API should document Action::get"
        );
        assert!(
            catalog.action_api.contains("Action::delete"),
            "Action API should document Action::delete"
        );
        assert!(
            catalog.action_api.contains("ActionOutcome"),
            "Action API should document ActionOutcome"
        );
    }

    #[test]
    fn test_components_have_props() {
        let catalog = execute(None);
        for component in &catalog.components {
            assert!(
                !component.description.is_empty(),
                "{} should have a description",
                component.name
            );
            // Components derived from global_catalog() carry props from JSON Schema.
            // Leaf components with all-optional props (Separator, Skeleton, etc.) may
            // have no required props — that is correct schema-driven behavior.
            let no_required = [
                "Separator",
                "Skeleton",
                "Sidebar",
                "Grid",
                "ButtonGroup",
                "CalendarCell",
                "Collapsible",
                "Toast",
                "Checklist",
                // D-15: items is now #[serde(default)] so data_path can be the sole source.
                "DescriptionList",
                // D-17a: html is #[serde(default)] — empty HTML is a valid no-op.
                "RawHtml",
                // sse_url is #[serde(default)] — matches RawHtml pattern; stream renders
                // empty when url is absent rather than failing.
                "StreamText",
                // D-13a: columns is now #[serde(default)] — data_path can be the sole source.
                "KanbanBoard",
                // items and data_path are both #[serde(default)] — either is a valid
                // sole source (same pattern as KanbanBoard); no single prop is required.
                "SegmentedControl",
            ];
            if !no_required.contains(&component.name.as_str()) {
                assert!(
                    component.props.iter().any(|p| p.required),
                    "{} should have at least one required prop",
                    component.name
                );
            }
        }
    }

    #[test]
    fn test_button_has_props() {
        // Button props are now derived from ButtonProps JSON Schema.
        // The `variants` field is None for struct Props (variants appear in prop type_name).
        let catalog = execute(Some("Button"));
        let button = &catalog.components[0];
        assert!(!button.props.is_empty(), "Button should have props");
        let has_label = button.props.iter().any(|p| p.name == "label");
        assert!(has_label, "Button should have a 'label' prop");
    }

    #[test]
    fn test_filter_returns_all_fields() {
        let catalog = execute(Some("Table"));
        assert_eq!(catalog.components.len(), 1);
        // Builder and action API still present even when filtering
        assert!(!catalog.builder_api.is_empty());
        assert!(!catalog.action_api.is_empty());
    }

    #[test]
    fn json_ui_catalog_includes_each_directive() {
        let catalog = execute(None);
        let each = catalog
            .directives
            .iter()
            .find(|d| d.name == "$each")
            .expect("$each directive present");
        assert!(!each.description.is_empty());
        assert!(each.syntax_example.contains("$each"));
        assert!(each
            .validation_errors
            .contains(&"EachPathNotArray".to_string()));
        assert!(each
            .validation_errors
            .contains(&"EachAsReservedName".to_string()));
    }

    #[test]
    fn json_ui_catalog_includes_if_directive() {
        let catalog = execute(None);
        let if_dir = catalog
            .directives
            .iter()
            .find(|d| d.name == "$if")
            .expect("$if directive present");
        assert!(!if_dir.description.is_empty());
        assert!(if_dir.syntax_example.contains("$if"));
        assert!(if_dir
            .validation_errors
            .contains(&"IfPathMissing".to_string()));
    }

    #[test]
    fn json_ui_catalog_directives_serialize_to_json() {
        let catalog = execute(None);
        let json = serde_json::to_value(&catalog).expect("serialize");
        let directives = json
            .get("directives")
            .and_then(|v| v.as_array())
            .expect("directives is an array");
        assert_eq!(directives.len(), 2);
    }
}
