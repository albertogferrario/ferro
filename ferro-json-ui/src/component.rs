//! Component catalog for JSON-UI.
//!
//! Defines the available UI components with typed props. Each component
//! uses serde's tagged enum representation so JSON includes `"type": "Card"`.

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::visibility::Visibility;

/// Shared size enum for components (Button, Badge, Avatar, Input).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    Xs,
    Sm,
    #[default]
    Default,
    Lg,
}

/// Icon placement relative to button label.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IconPosition {
    #[default]
    Left,
    Right,
}

/// Sort direction for table columns.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

/// Button visual variants (aligned to shadcn/ui).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

/// Input field types.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    #[default]
    Text,
    Email,
    Password,
    Number,
    Textarea,
    Hidden,
    Date,
    Time,
    Url,
    Tel,
    Search,
}

/// Alert visual variants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Badge visual variants (aligned to shadcn/ui).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
}

/// Text element types for semantic HTML rendering.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextElement {
    #[default]
    P,
    H1,
    H2,
    H3,
    Span,
}

/// Column display format for tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnFormat {
    Date,
    DateTime,
    Currency,
    Boolean,
}

/// Table column definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub key: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ColumnFormat>,
}

/// Select option (value + label pair).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// Props for Card component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ComponentNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<ComponentNode>,
}

/// Props for Table component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableProps {
    pub columns: Vec<Column>,
    pub data_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_actions: Option<Vec<Action>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_column: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<SortDirection>,
}

/// Props for Form component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormProps {
    pub action: Action,
    pub fields: Vec<ComponentNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<crate::action::HttpMethod>,
}

/// Props for Button component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonProps {
    pub label: String,
    #[serde(default)]
    pub variant: ButtonVariant,
    #[serde(default)]
    pub size: Size,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_position: Option<IconPosition>,
}

/// Props for Input component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputProps {
    /// Form field name for data binding.
    pub field: String,
    pub label: String,
    #[serde(default)]
    pub input_type: InputType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

/// Props for Select component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectProps {
    /// Form field name for data binding.
    pub field: String,
    pub label: String,
    pub options: Vec<SelectOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

/// Props for Alert component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertProps {
    pub message: String,
    #[serde(default)]
    pub variant: AlertVariant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Props for Badge component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BadgeProps {
    pub label: String,
    #[serde(default)]
    pub variant: BadgeVariant,
}

/// Props for Modal component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModalProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ComponentNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<ComponentNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_label: Option<String>,
}

/// Props for Text component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextProps {
    pub content: String,
    #[serde(default)]
    pub element: TextElement,
}

/// Tagged component enum. Serializes with `"type": "Card"` etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Component {
    Card(CardProps),
    Table(TableProps),
    Form(FormProps),
    Button(ButtonProps),
    Input(InputProps),
    Select(SelectProps),
    Alert(AlertProps),
    Badge(BadgeProps),
    Modal(ModalProps),
    Text(TextProps),
}

/// A component node wrapping a component with shared fields.
///
/// Every component in a view tree is wrapped in a `ComponentNode` that
/// provides a unique key, optional action binding, and optional visibility
/// rules. The component itself is flattened into the node's JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentNode {
    pub key: String,
    #[serde(flatten)]
    pub component: Component,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::HttpMethod;
    use crate::visibility::{VisibilityCondition, VisibilityOperator};

    #[test]
    fn card_component_tagged_serialization() {
        let card = Component::Card(CardProps {
            title: "Test Card".to_string(),
            description: Some("A description".to_string()),
            children: vec![],
            footer: vec![],
        });
        let json = serde_json::to_value(&card).unwrap();
        assert_eq!(json["type"], "Card");
        assert_eq!(json["title"], "Test Card");
        assert_eq!(json["description"], "A description");
    }

    #[test]
    fn button_variant_defaults_to_default() {
        let json = r#"{"type": "Button", "label": "Click me"}"#;
        let component: Component = serde_json::from_str(json).unwrap();
        match component {
            Component::Button(props) => {
                assert_eq!(props.variant, ButtonVariant::Default);
                assert_eq!(props.label, "Click me");
            }
            _ => panic!("expected Button"),
        }
    }

    #[test]
    fn input_type_defaults_to_text() {
        let json = r#"{"type": "Input", "field": "email", "label": "Email"}"#;
        let component: Component = serde_json::from_str(json).unwrap();
        match component {
            Component::Input(props) => {
                assert_eq!(props.input_type, InputType::Text);
                assert_eq!(props.field, "email");
            }
            _ => panic!("expected Input"),
        }
    }

    #[test]
    fn alert_variant_defaults_to_info() {
        let json = r#"{"type": "Alert", "message": "Hello"}"#;
        let component: Component = serde_json::from_str(json).unwrap();
        match component {
            Component::Alert(props) => assert_eq!(props.variant, AlertVariant::Info),
            _ => panic!("expected Alert"),
        }
    }

    #[test]
    fn badge_variant_defaults_to_default() {
        let json = r#"{"type": "Badge", "label": "New"}"#;
        let component: Component = serde_json::from_str(json).unwrap();
        match component {
            Component::Badge(props) => assert_eq!(props.variant, BadgeVariant::Default),
            _ => panic!("expected Badge"),
        }
    }

    #[test]
    fn text_element_defaults_to_p() {
        let json = r#"{"type": "Text", "content": "Hello world"}"#;
        let component: Component = serde_json::from_str(json).unwrap();
        match component {
            Component::Text(props) => {
                assert_eq!(props.element, TextElement::P);
                assert_eq!(props.content, "Hello world");
            }
            _ => panic!("expected Text"),
        }
    }

    #[test]
    fn table_component_round_trips() {
        let table = Component::Table(TableProps {
            columns: vec![
                Column {
                    key: "name".to_string(),
                    label: "Name".to_string(),
                    format: None,
                },
                Column {
                    key: "created_at".to_string(),
                    label: "Created".to_string(),
                    format: Some(ColumnFormat::Date),
                },
            ],
            data_path: "/data/users".to_string(),
            row_actions: None,
            empty_message: Some("No users found".to_string()),
            sortable: None,
            sort_column: None,
            sort_direction: None,
        });
        let json = serde_json::to_string(&table).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, table);
    }

    #[test]
    fn select_component_round_trips() {
        let select = Component::Select(SelectProps {
            field: "role".to_string(),
            label: "Role".to_string(),
            options: vec![
                SelectOption {
                    value: "admin".to_string(),
                    label: "Administrator".to_string(),
                },
                SelectOption {
                    value: "user".to_string(),
                    label: "User".to_string(),
                },
            ],
            placeholder: Some("Select a role".to_string()),
            required: Some(true),
            disabled: None,
            error: None,
            description: None,
            default_value: None,
        });
        let json = serde_json::to_string(&select).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, select);
    }

    #[test]
    fn modal_component_round_trips() {
        let modal = Component::Modal(ModalProps {
            title: "Confirm".to_string(),
            description: None,
            children: vec![ComponentNode {
                key: "msg".to_string(),
                component: Component::Text(TextProps {
                    content: "Are you sure?".to_string(),
                    element: TextElement::P,
                }),
                action: None,
                visibility: None,
            }],
            footer: vec![],
            trigger_label: Some("Open".to_string()),
        });
        let json = serde_json::to_string(&modal).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modal);
    }

    #[test]
    fn form_component_round_trips() {
        let form = Component::Form(FormProps {
            action: Action {
                handler: "users.store".to_string(),
                method: HttpMethod::Post,
                confirm: None,
                on_success: None,
                on_error: None,
            },
            fields: vec![ComponentNode {
                key: "email-input".to_string(),
                component: Component::Input(InputProps {
                    field: "email".to_string(),
                    label: "Email".to_string(),
                    input_type: InputType::Email,
                    placeholder: Some("user@example.com".to_string()),
                    required: Some(true),
                    disabled: None,
                    error: None,
                    description: None,
                    default_value: None,
                }),
                action: None,
                visibility: None,
            }],
            method: None,
        });
        let json = serde_json::to_string(&form).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, form);
    }

    #[test]
    fn component_node_with_action_and_visibility() {
        let node = ComponentNode {
            key: "create-btn".to_string(),
            component: Component::Button(ButtonProps {
                label: "Create User".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
            }),
            action: Some(Action {
                handler: "users.create".to_string(),
                method: HttpMethod::Post,
                confirm: None,
                on_success: None,
                on_error: None,
            }),
            visibility: Some(Visibility::Condition(VisibilityCondition {
                path: "/auth/user/role".to_string(),
                operator: VisibilityOperator::Eq,
                value: Some(serde_json::Value::String("admin".to_string())),
            })),
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: ComponentNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, node);

        // Verify flattened structure includes type
        let value = serde_json::to_value(&node).unwrap();
        assert_eq!(value["type"], "Button");
        assert_eq!(value["key"], "create-btn");
        assert!(value.get("action").is_some());
        assert!(value.get("visibility").is_some());
    }

    #[test]
    fn all_ten_component_variants_serialize() {
        let components: Vec<Component> = vec![
            Component::Card(CardProps {
                title: "t".to_string(),
                description: None,
                children: vec![],
                footer: vec![],
            }),
            Component::Table(TableProps {
                columns: vec![],
                data_path: "/d".to_string(),
                row_actions: None,
                empty_message: None,
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            Component::Form(FormProps {
                action: Action {
                    handler: "h.m".to_string(),
                    method: HttpMethod::Post,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                },
                fields: vec![],
                method: None,
            }),
            Component::Button(ButtonProps {
                label: "b".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
            }),
            Component::Input(InputProps {
                field: "f".to_string(),
                label: "l".to_string(),
                input_type: InputType::Text,
                placeholder: None,
                required: None,
                disabled: None,
                error: None,
                description: None,
                default_value: None,
            }),
            Component::Select(SelectProps {
                field: "f".to_string(),
                label: "l".to_string(),
                options: vec![],
                placeholder: None,
                required: None,
                disabled: None,
                error: None,
                description: None,
                default_value: None,
            }),
            Component::Alert(AlertProps {
                message: "m".to_string(),
                variant: AlertVariant::Info,
                title: None,
            }),
            Component::Badge(BadgeProps {
                label: "b".to_string(),
                variant: BadgeVariant::Default,
            }),
            Component::Modal(ModalProps {
                title: "t".to_string(),
                description: None,
                children: vec![],
                footer: vec![],
                trigger_label: None,
            }),
            Component::Text(TextProps {
                content: "c".to_string(),
                element: TextElement::P,
            }),
        ];
        let expected_types = [
            "Card", "Table", "Form", "Button", "Input", "Select", "Alert", "Badge", "Modal", "Text",
        ];
        for (component, expected_type) in components.iter().zip(expected_types.iter()) {
            let json = serde_json::to_value(component).unwrap();
            assert_eq!(
                json["type"], *expected_type,
                "component should serialize with type={}",
                expected_type
            );
            let roundtripped: Component = serde_json::from_value(json).unwrap();
            assert_eq!(&roundtripped, component);
        }
    }
}
