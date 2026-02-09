//! JSON-UI component catalog tool — structured reference of all 20 components.

use serde::Serialize;

/// Complete catalog of JSON-UI components with builder and action API.
#[derive(Debug, Serialize)]
pub struct JsonUiCatalog {
    pub components: Vec<CatalogComponent>,
    pub builder_api: String,
    pub action_api: String,
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
pub fn execute(component: Option<&str>) -> JsonUiCatalog {
    let all = build_catalog();

    let components = match component {
        Some(name) => {
            let lower = name.to_lowercase();
            all.into_iter()
                .filter(|c| c.name.to_lowercase() == lower)
                .collect()
        }
        None => all,
    };

    JsonUiCatalog {
        components,
        builder_api: BUILDER_API.to_string(),
        action_api: ACTION_API.to_string(),
    }
}

const BUILDER_API: &str = "\
JsonUiView::new() -> JsonUiView
  .title(impl Into<String>) -> Self
  .layout(impl Into<String>) -> Self
  .data(serde_json::Value) -> Self
  .errors(HashMap<String, Vec<String>>) -> Self
  .component(ComponentNode) -> Self
  .components(Vec<ComponentNode>) -> Self

ComponentNode { key: String, component: Component, action: Option<Action>, visibility: Option<Visibility> }
  - key: Unique identifier for this node in the view tree
  - component: One of the 20 Component enum variants
  - action: Optional Action binding (click/submit handler)
  - visibility: Optional Visibility rule (show/hide based on data path)";

const ACTION_API: &str = "\
Action::new(handler) -> Action (POST)
Action::get(handler) -> Action (GET)
Action::delete(handler) -> Action (DELETE)
  .method(HttpMethod) -> Self
  .confirm(title) -> Self (default dialog)
  .confirm_danger(title) -> Self (danger dialog)
  .on_success(ActionOutcome) -> Self
  .on_error(ActionOutcome) -> Self

Handler format: \"controller.method\" (e.g., \"users.store\")

ActionOutcome variants:
  Redirect { url: String }
  ShowErrors
  Refresh
  Notify { message: String, variant: NotifyVariant }

ConfirmDialog { title: String, message: Option<String>, variant: DialogVariant (default|danger) }";

fn build_catalog() -> Vec<CatalogComponent> {
    vec![
        CatalogComponent {
            name: "Text".to_string(),
            description: "Renders text content with semantic HTML element selection.".to_string(),
            props: vec![
                prop("content", "String", true, "Text content to display"),
                prop(
                    "element",
                    "TextElement",
                    false,
                    "HTML element: h1, h2, h3, span, p (default: p)",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Button".to_string(),
            description: "Interactive button with visual variants and optional icon.".to_string(),
            props: vec![
                prop("label", "String", true, "Button label text"),
                prop(
                    "variant",
                    "ButtonVariant",
                    false,
                    "Visual style (default: default)",
                ),
                prop("size", "Size", false, "Button size (default: default)"),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether button is disabled",
                ),
                prop("icon", "Option<String>", false, "Icon name"),
                prop(
                    "icon_position",
                    "Option<IconPosition>",
                    false,
                    "Icon placement: left or right (default: left)",
                ),
            ],
            variants: Some(vec![
                "default".to_string(),
                "secondary".to_string(),
                "destructive".to_string(),
                "outline".to_string(),
                "ghost".to_string(),
                "link".to_string(),
            ]),
        },
        CatalogComponent {
            name: "Card".to_string(),
            description: "Container with title, optional description, children, and footer."
                .to_string(),
            props: vec![
                prop("title", "String", true, "Card title"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Card description below title",
                ),
                prop(
                    "children",
                    "Vec<ComponentNode>",
                    false,
                    "Nested components inside the card body",
                ),
                prop(
                    "footer",
                    "Vec<ComponentNode>",
                    false,
                    "Components in the card footer",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Table".to_string(),
            description: "Data table with columns, row actions, sorting, and empty state."
                .to_string(),
            props: vec![
                prop(
                    "columns",
                    "Vec<Column>",
                    true,
                    "Column definitions: { key, label, format? }",
                ),
                prop(
                    "data_path",
                    "String",
                    true,
                    "Data path to the array of rows (e.g., \"/data/users\")",
                ),
                prop(
                    "row_actions",
                    "Option<Vec<Action>>",
                    false,
                    "Actions available for each row",
                ),
                prop(
                    "empty_message",
                    "Option<String>",
                    false,
                    "Message when table has no data",
                ),
                prop(
                    "sortable",
                    "Option<bool>",
                    false,
                    "Whether columns are sortable",
                ),
                prop(
                    "sort_column",
                    "Option<String>",
                    false,
                    "Currently sorted column key",
                ),
                prop(
                    "sort_direction",
                    "Option<SortDirection>",
                    false,
                    "Sort direction: asc or desc",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Form".to_string(),
            description: "Form container with action binding and field components.".to_string(),
            props: vec![
                prop("action", "Action", true, "Action to execute on form submit"),
                prop(
                    "fields",
                    "Vec<ComponentNode>",
                    true,
                    "Form field components (Input, Select, Checkbox, etc.)",
                ),
                prop(
                    "method",
                    "Option<HttpMethod>",
                    false,
                    "HTTP method override (GET, POST, PUT, PATCH, DELETE)",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Input".to_string(),
            description: "Text input field with type variants, validation error, and data binding."
                .to_string(),
            props: vec![
                prop("field", "String", true, "Form field name for data binding"),
                prop("label", "String", true, "Input label text"),
                prop(
                    "input_type",
                    "InputType",
                    false,
                    "Input type (default: text)",
                ),
                prop("placeholder", "Option<String>", false, "Placeholder text"),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "Whether field is required",
                ),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether field is disabled",
                ),
                prop("error", "Option<String>", false, "Validation error message"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Help text below the input",
                ),
                prop("default_value", "Option<String>", false, "Pre-filled value"),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "Data path for pre-filling from handler data",
                ),
            ],
            variants: Some(vec![
                "text".to_string(),
                "email".to_string(),
                "password".to_string(),
                "number".to_string(),
                "textarea".to_string(),
                "hidden".to_string(),
                "date".to_string(),
                "time".to_string(),
                "url".to_string(),
                "tel".to_string(),
                "search".to_string(),
            ]),
        },
        CatalogComponent {
            name: "Select".to_string(),
            description: "Dropdown select field with options, validation error, and data binding."
                .to_string(),
            props: vec![
                prop("field", "String", true, "Form field name for data binding"),
                prop("label", "String", true, "Select label text"),
                prop(
                    "options",
                    "Vec<SelectOption>",
                    true,
                    "Options: { value, label }",
                ),
                prop("placeholder", "Option<String>", false, "Placeholder text"),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "Whether field is required",
                ),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether field is disabled",
                ),
                prop("error", "Option<String>", false, "Validation error message"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Help text below the select",
                ),
                prop(
                    "default_value",
                    "Option<String>",
                    false,
                    "Pre-selected value",
                ),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "Data path for pre-filling from handler data",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Alert".to_string(),
            description: "Alert message with variant-based styling.".to_string(),
            props: vec![
                prop("message", "String", true, "Alert message content"),
                prop(
                    "variant",
                    "AlertVariant",
                    false,
                    "Visual style (default: info)",
                ),
                prop("title", "Option<String>", false, "Alert title"),
            ],
            variants: Some(vec![
                "info".to_string(),
                "success".to_string(),
                "warning".to_string(),
                "error".to_string(),
            ]),
        },
        CatalogComponent {
            name: "Badge".to_string(),
            description: "Small label with variant-based styling.".to_string(),
            props: vec![
                prop("label", "String", true, "Badge text"),
                prop(
                    "variant",
                    "BadgeVariant",
                    false,
                    "Visual style (default: default)",
                ),
            ],
            variants: Some(vec![
                "default".to_string(),
                "secondary".to_string(),
                "destructive".to_string(),
                "outline".to_string(),
            ]),
        },
        CatalogComponent {
            name: "Modal".to_string(),
            description: "Dialog overlay with title, content, footer, and trigger button."
                .to_string(),
            props: vec![
                prop("title", "String", true, "Modal title"),
                prop("description", "Option<String>", false, "Modal description"),
                prop(
                    "children",
                    "Vec<ComponentNode>",
                    false,
                    "Content components inside the modal body",
                ),
                prop(
                    "footer",
                    "Vec<ComponentNode>",
                    false,
                    "Components in the modal footer",
                ),
                prop(
                    "trigger_label",
                    "Option<String>",
                    false,
                    "Label for the trigger button",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Checkbox".to_string(),
            description: "Boolean checkbox field with label, description, and data binding."
                .to_string(),
            props: vec![
                prop("field", "String", true, "Form field name for data binding"),
                prop("label", "String", true, "Checkbox label text"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Help text below the checkbox",
                ),
                prop("checked", "Option<bool>", false, "Default checked state"),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "Data path for pre-filling from handler data",
                ),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "Whether field is required",
                ),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether field is disabled",
                ),
                prop("error", "Option<String>", false, "Validation error message"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Switch".to_string(),
            description: "Toggle switch (visual alternative to Checkbox, same props).".to_string(),
            props: vec![
                prop("field", "String", true, "Form field name for data binding"),
                prop("label", "String", true, "Switch label text"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Help text below the switch",
                ),
                prop("checked", "Option<bool>", false, "Default checked state"),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "Data path for pre-filling from handler data",
                ),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "Whether field is required",
                ),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether field is disabled",
                ),
                prop("error", "Option<String>", false, "Validation error message"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Separator".to_string(),
            description: "Visual divider between content sections.".to_string(),
            props: vec![prop(
                "orientation",
                "Option<Orientation>",
                false,
                "Direction: horizontal (default) or vertical",
            )],
            variants: None,
        },
        CatalogComponent {
            name: "DescriptionList".to_string(),
            description: "Key-value pairs displayed as a description list.".to_string(),
            props: vec![
                prop(
                    "items",
                    "Vec<DescriptionItem>",
                    true,
                    "Items: { label, value, format? }",
                ),
                prop(
                    "columns",
                    "Option<u8>",
                    false,
                    "Number of columns for layout",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Tabs".to_string(),
            description: "Tabbed content with multiple panels.".to_string(),
            props: vec![
                prop(
                    "default_tab",
                    "String",
                    true,
                    "Value of the initially active tab",
                ),
                prop(
                    "tabs",
                    "Vec<Tab>",
                    true,
                    "Tab definitions: { value, label, children }",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Breadcrumb".to_string(),
            description: "Navigation breadcrumb trail.".to_string(),
            props: vec![prop(
                "items",
                "Vec<BreadcrumbItem>",
                true,
                "Breadcrumb items: { label, url? }",
            )],
            variants: None,
        },
        CatalogComponent {
            name: "Pagination".to_string(),
            description: "Page navigation for paginated data.".to_string(),
            props: vec![
                prop("current_page", "u32", true, "Current page number"),
                prop("per_page", "u32", true, "Items per page"),
                prop("total", "u32", true, "Total number of items"),
                prop(
                    "base_url",
                    "Option<String>",
                    false,
                    "Base URL for page links",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Progress".to_string(),
            description: "Progress bar with percentage value.".to_string(),
            props: vec![
                prop("value", "u8", true, "Percentage value (0-100)"),
                prop("max", "Option<u8>", false, "Maximum value"),
                prop("label", "Option<String>", false, "Label text above the bar"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Avatar".to_string(),
            description: "User avatar with image, fallback text, and size variants.".to_string(),
            props: vec![
                prop("src", "Option<String>", false, "Image URL"),
                prop(
                    "alt",
                    "String",
                    true,
                    "Alt text (required for accessibility)",
                ),
                prop(
                    "fallback",
                    "Option<String>",
                    false,
                    "Fallback initials when no image",
                ),
                prop(
                    "size",
                    "Option<Size>",
                    false,
                    "Avatar size: xs, sm, default, lg",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Skeleton".to_string(),
            description: "Loading placeholder with configurable dimensions.".to_string(),
            props: vec![
                prop(
                    "width",
                    "Option<String>",
                    false,
                    "CSS width (e.g., \"100%\", \"200px\")",
                ),
                prop(
                    "height",
                    "Option<String>",
                    false,
                    "CSS height (e.g., \"40px\")",
                ),
                prop(
                    "rounded",
                    "Option<bool>",
                    false,
                    "Whether to use rounded corners",
                ),
            ],
            variants: None,
        },
    ]
}

fn prop(name: &str, type_name: &str, required: bool, description: &str) -> PropInfo {
    PropInfo {
        name: name.to_string(),
        type_name: type_name.to_string(),
        required,
        description: description.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_components_present() {
        let catalog = execute(None);
        assert_eq!(
            catalog.components.len(),
            20,
            "Catalog should contain all 20 components, got {}",
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
            "Switch",
            "Separator",
            "DescriptionList",
            "Tabs",
            "Breadcrumb",
            "Pagination",
            "Progress",
            "Avatar",
            "Skeleton",
        ];
        for name in &expected {
            assert!(names.contains(name), "Missing component: {}", name);
        }
    }

    #[test]
    fn test_filter_by_component() {
        let catalog = execute(Some("Button"));
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].name, "Button");
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
        assert!(json_str.contains("builder_api"));
        assert!(json_str.contains("action_api"));
        assert!(json_str.contains("Button"));
        assert!(json_str.contains("props"));
    }

    #[test]
    fn test_builder_api_present() {
        let catalog = execute(None);
        assert!(
            catalog.builder_api.contains("JsonUiView::new()"),
            "Builder API should document JsonUiView::new()"
        );
        assert!(
            catalog.builder_api.contains("ComponentNode"),
            "Builder API should document ComponentNode"
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
            // All components have at least one prop except Skeleton (all optional) and Separator
            // But even those have props defined
            if component.name != "Separator" && component.name != "Skeleton" {
                assert!(
                    component.props.iter().any(|p| p.required),
                    "{} should have at least one required prop",
                    component.name
                );
            }
        }
    }

    #[test]
    fn test_button_has_variants() {
        let catalog = execute(Some("Button"));
        let button = &catalog.components[0];
        let variants = button
            .variants
            .as_ref()
            .expect("Button should have variants");
        assert_eq!(variants.len(), 6);
        assert!(variants.contains(&"default".to_string()));
        assert!(variants.contains(&"destructive".to_string()));
    }

    #[test]
    fn test_filter_returns_all_fields() {
        let catalog = execute(Some("Table"));
        assert_eq!(catalog.components.len(), 1);
        // Builder and action API still present even when filtering
        assert!(!catalog.builder_api.is_empty());
        assert!(!catalog.action_api.is_empty());
    }
}
