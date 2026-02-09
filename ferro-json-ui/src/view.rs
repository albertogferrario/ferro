//! Top-level view container for JSON-UI.
//!
//! A `JsonUiView` is the root structure that defines a complete page.
//! It contains the schema version, optional layout and title, and the
//! component tree. Views can be built programmatically or parsed from JSON.

use serde::{Deserialize, Serialize};

use crate::component::ComponentNode;

/// Schema version identifier for JSON-UI views.
pub const SCHEMA_VERSION: &str = "ferro-json-ui/v1";

/// Top-level JSON-UI view container.
///
/// Every JSON-UI response is a `JsonUiView` containing a component tree.
/// The `$schema` field identifies the schema version for compatibility.
///
/// # Example
///
/// ```rust
/// use ferro_json_ui::JsonUiView;
///
/// let view = JsonUiView::new()
///     .title("Dashboard")
///     .layout("app");
///
/// let json = view.to_json().unwrap();
/// assert!(json.contains("ferro-json-ui/v1"));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonUiView {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub components: Vec<ComponentNode>,
}

impl JsonUiView {
    /// Create a new view with the current schema version and empty components.
    pub fn new() -> Self {
        Self {
            schema: SCHEMA_VERSION.to_string(),
            layout: None,
            title: None,
            components: vec![],
        }
    }

    /// Set the view title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the layout name.
    pub fn layout(mut self, layout: impl Into<String>) -> Self {
        self.layout = Some(layout.into());
        self
    }

    /// Add a single component to the view.
    pub fn component(mut self, node: ComponentNode) -> Self {
        self.components.push(node);
        self
    }

    /// Set all components at once, replacing any existing.
    pub fn components(mut self, nodes: Vec<ComponentNode>) -> Self {
        self.components = nodes;
        self
    }

    /// Parse a view from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize the view to a compact JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize the view to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for JsonUiView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, HttpMethod};
    use crate::component::*;
    use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};

    #[test]
    fn schema_field_serializes_as_dollar_schema() {
        let view = JsonUiView::new();
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["$schema"], "ferro-json-ui/v1");
        assert!(json.get("schema").is_none());
    }

    #[test]
    fn builder_produces_valid_json() {
        let view = JsonUiView::new()
            .title("Users")
            .layout("app")
            .component(ComponentNode {
                key: "header".to_string(),
                component: Component::Card(CardProps {
                    title: "User Management".to_string(),
                    description: None,
                    children: vec![],
                }),
                action: None,
                visibility: None,
            });

        let json = view.to_json().unwrap();
        assert!(json.contains("\"$schema\":\"ferro-json-ui/v1\""));
        assert!(json.contains("\"title\":\"Users\""));
        assert!(json.contains("\"layout\":\"app\""));
        assert!(json.contains("\"type\":\"Card\""));
    }

    #[test]
    fn round_trip_build_to_json_from_json() {
        let original = JsonUiView::new()
            .title("Dashboard")
            .layout("app")
            .component(ComponentNode {
                key: "alert".to_string(),
                component: Component::Alert(AlertProps {
                    message: "Welcome".to_string(),
                    variant: AlertVariant::Success,
                }),
                action: None,
                visibility: None,
            })
            .component(ComponentNode {
                key: "content".to_string(),
                component: Component::Text(TextProps {
                    content: "Hello world".to_string(),
                    element: TextElement::H1,
                }),
                action: None,
                visibility: None,
            });

        let json = original.to_json().unwrap();
        let parsed = JsonUiView::from_json(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn from_json_full_example() {
        // Based on the research doc example
        let json = r#"{
            "$schema": "ferro-json-ui/v1",
            "layout": "app",
            "title": "Users",
            "components": [
                {
                    "key": "header",
                    "type": "Card",
                    "title": "User Management",
                    "children": [
                        {
                            "key": "create-btn",
                            "type": "Button",
                            "label": "Create User",
                            "variant": "primary",
                            "action": {
                                "handler": "users.create",
                                "method": "POST"
                            }
                        }
                    ]
                },
                {
                    "key": "users-table",
                    "type": "Table",
                    "columns": [
                        {"key": "name", "label": "Name"},
                        {"key": "email", "label": "Email"},
                        {"key": "created_at", "label": "Created", "format": "date"}
                    ],
                    "data_path": "/data/users",
                    "visibility": {
                        "path": "/data/users",
                        "operator": "not_empty"
                    }
                }
            ]
        }"#;
        let view = JsonUiView::from_json(json).unwrap();
        assert_eq!(view.schema, "ferro-json-ui/v1");
        assert_eq!(view.title.as_deref(), Some("Users"));
        assert_eq!(view.layout.as_deref(), Some("app"));
        assert_eq!(view.components.len(), 2);

        // Verify first component is a Card
        assert_eq!(view.components[0].key, "header");
        match &view.components[0].component {
            Component::Card(props) => {
                assert_eq!(props.title, "User Management");
                assert_eq!(props.children.len(), 1);
                // Verify nested button
                match &props.children[0].component {
                    Component::Button(bp) => assert_eq!(bp.label, "Create User"),
                    _ => panic!("expected Button child"),
                }
            }
            _ => panic!("expected Card"),
        }

        // Verify second component is a Table with visibility
        assert_eq!(view.components[1].key, "users-table");
        match &view.components[1].component {
            Component::Table(props) => {
                assert_eq!(props.columns.len(), 3);
                assert_eq!(props.data_path, "/data/users");
            }
            _ => panic!("expected Table"),
        }
        assert!(view.components[1].visibility.is_some());
    }

    #[test]
    fn empty_view_serializes() {
        let view = JsonUiView::new();
        let json = view.to_json().unwrap();
        let parsed = JsonUiView::from_json(&json).unwrap();
        assert_eq!(parsed.schema, SCHEMA_VERSION);
        assert!(parsed.title.is_none());
        assert!(parsed.layout.is_none());
        assert!(parsed.components.is_empty());
    }

    #[test]
    fn to_json_pretty_is_readable() {
        let view = JsonUiView::new().title("Test");
        let pretty = view.to_json_pretty().unwrap();
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }

    #[test]
    fn components_method_replaces_existing() {
        let view = JsonUiView::new()
            .component(ComponentNode {
                key: "first".to_string(),
                component: Component::Text(TextProps {
                    content: "first".to_string(),
                    element: TextElement::P,
                }),
                action: None,
                visibility: None,
            })
            .components(vec![ComponentNode {
                key: "replaced".to_string(),
                component: Component::Text(TextProps {
                    content: "replaced".to_string(),
                    element: TextElement::P,
                }),
                action: None,
                visibility: None,
            }]);
        assert_eq!(view.components.len(), 1);
        assert_eq!(view.components[0].key, "replaced");
    }

    #[test]
    fn complex_view_with_action_and_visibility() {
        let view = JsonUiView::new()
            .title("Admin Panel")
            .component(ComponentNode {
                key: "delete-btn".to_string(),
                component: Component::Button(ButtonProps {
                    label: "Delete All".to_string(),
                    variant: ButtonVariant::Danger,
                    disabled: Some(false),
                }),
                action: Some(Action {
                    handler: "admin.delete_all".to_string(),
                    method: HttpMethod::Delete,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                }),
                visibility: Some(Visibility::Condition(VisibilityCondition {
                    path: "/auth/user/role".to_string(),
                    operator: VisibilityOperator::Eq,
                    value: Some(serde_json::Value::String("admin".to_string())),
                })),
            });

        let json = view.to_json().unwrap();
        let parsed = JsonUiView::from_json(&json).unwrap();
        assert_eq!(view, parsed);
    }
}
