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

/// Separator orientation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
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
    /// Data path for pre-filling from handler data (e.g., "/data/user/name").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
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
    /// Data path for pre-filling from handler data (e.g., "/data/user/name").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
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

/// Props for Checkbox component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxProps {
    /// Form field name for data binding.
    pub field: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// Data path for pre-filling from handler data (e.g., "/data/user/name").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Props for Switch component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchProps {
    /// Form field name for data binding.
    pub field: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// Data path for pre-filling from handler data (e.g., "/data/user/name").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Props for Separator component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeparatorProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
}

/// A single item in a description list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescriptionItem {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ColumnFormat>,
}

/// Props for DescriptionList component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescriptionListProps {
    pub items: Vec<DescriptionItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u8>,
}

/// A single tab within a Tabs component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    pub value: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ComponentNode>,
}

/// Props for Tabs component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabsProps {
    pub default_tab: String,
    pub tabs: Vec<Tab>,
}

/// A single item in a breadcrumb trail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Props for Breadcrumb component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreadcrumbProps {
    pub items: Vec<BreadcrumbItem>,
}

/// Props for Pagination component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginationProps {
    pub current_page: u32,
    pub per_page: u32,
    pub total: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

/// Props for Progress component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressProps {
    /// Percentage value (0-100).
    pub value: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Props for Avatar component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvatarProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Size>,
}

/// Props for Skeleton loading placeholder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkeletonProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rounded: Option<bool>,
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
    Checkbox(CheckboxProps),
    Switch(SwitchProps),
    Separator(SeparatorProps),
    DescriptionList(DescriptionListProps),
    Tabs(TabsProps),
    Breadcrumb(BreadcrumbProps),
    Pagination(PaginationProps),
    Progress(ProgressProps),
    Avatar(AvatarProps),
    Skeleton(SkeletonProps),
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
            data_path: None,
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
                url: None,
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
                    data_path: None,
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
                url: None,
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
    fn all_component_variants_serialize() {
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
                    url: None,
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
                data_path: None,
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
                data_path: None,
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
            Component::Checkbox(CheckboxProps {
                field: "f".to_string(),
                label: "l".to_string(),
                description: None,
                checked: None,
                data_path: None,
                required: None,
                disabled: None,
                error: None,
            }),
            Component::Switch(SwitchProps {
                field: "f".to_string(),
                label: "l".to_string(),
                description: None,
                checked: None,
                data_path: None,
                required: None,
                disabled: None,
                error: None,
            }),
            Component::Separator(SeparatorProps { orientation: None }),
            Component::DescriptionList(DescriptionListProps {
                items: vec![DescriptionItem {
                    label: "k".to_string(),
                    value: "v".to_string(),
                    format: None,
                }],
                columns: None,
            }),
            Component::Tabs(TabsProps {
                default_tab: "t1".to_string(),
                tabs: vec![Tab {
                    value: "t1".to_string(),
                    label: "Tab 1".to_string(),
                    children: vec![],
                }],
            }),
            Component::Breadcrumb(BreadcrumbProps {
                items: vec![BreadcrumbItem {
                    label: "Home".to_string(),
                    url: Some("/".to_string()),
                }],
            }),
            Component::Pagination(PaginationProps {
                current_page: 1,
                per_page: 10,
                total: 100,
                base_url: None,
            }),
            Component::Progress(ProgressProps {
                value: 50,
                max: None,
                label: None,
            }),
            Component::Avatar(AvatarProps {
                src: None,
                alt: "User".to_string(),
                fallback: Some("U".to_string()),
                size: None,
            }),
            Component::Skeleton(SkeletonProps {
                width: None,
                height: None,
                rounded: None,
            }),
        ];
        assert_eq!(components.len(), 20, "should have 20 component variants");
        let expected_types = [
            "Card",
            "Table",
            "Form",
            "Button",
            "Input",
            "Select",
            "Alert",
            "Badge",
            "Modal",
            "Text",
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

    #[test]
    fn size_enum_serialization() {
        let cases = [
            (Size::Xs, "xs"),
            (Size::Sm, "sm"),
            (Size::Default, "default"),
            (Size::Lg, "lg"),
        ];
        for (size, expected) in &cases {
            let json = serde_json::to_value(size).unwrap();
            assert_eq!(json, *expected);
            let parsed: Size = serde_json::from_value(json).unwrap();
            assert_eq!(&parsed, size);
        }
    }

    #[test]
    fn icon_position_serialization() {
        let cases = [(IconPosition::Left, "left"), (IconPosition::Right, "right")];
        for (pos, expected) in &cases {
            let json = serde_json::to_value(pos).unwrap();
            assert_eq!(json, *expected);
            let parsed: IconPosition = serde_json::from_value(json).unwrap();
            assert_eq!(&parsed, pos);
        }
    }

    #[test]
    fn sort_direction_serialization() {
        let cases = [(SortDirection::Asc, "asc"), (SortDirection::Desc, "desc")];
        for (dir, expected) in &cases {
            let json = serde_json::to_value(dir).unwrap();
            assert_eq!(json, *expected);
            let parsed: SortDirection = serde_json::from_value(json).unwrap();
            assert_eq!(&parsed, dir);
        }
    }

    #[test]
    fn button_with_size_and_icon() {
        let button = Component::Button(ButtonProps {
            label: "Save".to_string(),
            variant: ButtonVariant::Default,
            size: Size::Lg,
            disabled: None,
            icon: Some("save".to_string()),
            icon_position: Some(IconPosition::Left),
        });
        let json = serde_json::to_value(&button).unwrap();
        assert_eq!(json["size"], "lg");
        assert_eq!(json["icon"], "save");
        assert_eq!(json["icon_position"], "left");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, button);
    }

    #[test]
    fn card_with_footer() {
        let card = Component::Card(CardProps {
            title: "Actions".to_string(),
            description: None,
            children: vec![],
            footer: vec![ComponentNode {
                key: "cancel".to_string(),
                component: Component::Button(ButtonProps {
                    label: "Cancel".to_string(),
                    variant: ButtonVariant::Outline,
                    size: Size::Default,
                    disabled: None,
                    icon: None,
                    icon_position: None,
                }),
                action: None,
                visibility: None,
            }],
        });
        let json = serde_json::to_value(&card).unwrap();
        assert!(json["footer"].is_array());
        assert_eq!(json["footer"][0]["label"], "Cancel");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, card);
    }

    #[test]
    fn input_with_error_and_description() {
        let input = Component::Input(InputProps {
            field: "email".to_string(),
            label: "Email".to_string(),
            input_type: InputType::Email,
            placeholder: None,
            required: Some(true),
            disabled: Some(false),
            error: Some("Invalid email".to_string()),
            description: Some("Your work email".to_string()),
            default_value: Some("user@example.com".to_string()),
            data_path: None,
        });
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(json["error"], "Invalid email");
        assert_eq!(json["description"], "Your work email");
        assert_eq!(json["default_value"], "user@example.com");
        assert_eq!(json["disabled"], false);
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, input);
    }

    #[test]
    fn select_with_default_value() {
        let select = Component::Select(SelectProps {
            field: "role".to_string(),
            label: "Role".to_string(),
            options: vec![SelectOption {
                value: "admin".to_string(),
                label: "Admin".to_string(),
            }],
            placeholder: None,
            required: None,
            disabled: Some(true),
            error: Some("Required field".to_string()),
            description: Some("User role".to_string()),
            default_value: Some("admin".to_string()),
            data_path: None,
        });
        let json = serde_json::to_value(&select).unwrap();
        assert_eq!(json["default_value"], "admin");
        assert_eq!(json["error"], "Required field");
        assert_eq!(json["description"], "User role");
        assert_eq!(json["disabled"], true);
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, select);
    }

    #[test]
    fn alert_with_title() {
        let alert = Component::Alert(AlertProps {
            message: "Something happened".to_string(),
            variant: AlertVariant::Warning,
            title: Some("Warning".to_string()),
        });
        let json = serde_json::to_value(&alert).unwrap();
        assert_eq!(json["title"], "Warning");
        assert_eq!(json["message"], "Something happened");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, alert);
    }

    #[test]
    fn modal_with_footer_and_description() {
        let modal = Component::Modal(ModalProps {
            title: "Delete Item".to_string(),
            description: Some("This action cannot be undone.".to_string()),
            children: vec![],
            footer: vec![ComponentNode {
                key: "confirm".to_string(),
                component: Component::Button(ButtonProps {
                    label: "Delete".to_string(),
                    variant: ButtonVariant::Destructive,
                    size: Size::Default,
                    disabled: None,
                    icon: None,
                    icon_position: None,
                }),
                action: None,
                visibility: None,
            }],
            trigger_label: Some("Delete".to_string()),
        });
        let json = serde_json::to_value(&modal).unwrap();
        assert_eq!(json["description"], "This action cannot be undone.");
        assert!(json["footer"].is_array());
        assert_eq!(json["footer"][0]["label"], "Delete");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, modal);
    }

    #[test]
    fn table_with_sort_props() {
        let table = Component::Table(TableProps {
            columns: vec![Column {
                key: "name".to_string(),
                label: "Name".to_string(),
                format: None,
            }],
            data_path: "/data/users".to_string(),
            row_actions: None,
            empty_message: None,
            sortable: Some(true),
            sort_column: Some("name".to_string()),
            sort_direction: Some(SortDirection::Desc),
        });
        let json = serde_json::to_value(&table).unwrap();
        assert_eq!(json["sortable"], true);
        assert_eq!(json["sort_column"], "name");
        assert_eq!(json["sort_direction"], "desc");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, table);
    }

    #[test]
    fn aligned_button_variants_serialize() {
        let cases = [
            (ButtonVariant::Default, "default"),
            (ButtonVariant::Secondary, "secondary"),
            (ButtonVariant::Destructive, "destructive"),
            (ButtonVariant::Outline, "outline"),
            (ButtonVariant::Ghost, "ghost"),
            (ButtonVariant::Link, "link"),
        ];
        for (variant, expected) in &cases {
            let json = serde_json::to_value(variant).unwrap();
            assert_eq!(
                json, *expected,
                "ButtonVariant::{:?} should serialize as {}",
                variant, expected
            );
            let parsed: ButtonVariant = serde_json::from_value(json).unwrap();
            assert_eq!(&parsed, variant);
        }
    }

    #[test]
    fn aligned_badge_variants_serialize() {
        let cases = [
            (BadgeVariant::Default, "default"),
            (BadgeVariant::Secondary, "secondary"),
            (BadgeVariant::Destructive, "destructive"),
            (BadgeVariant::Outline, "outline"),
        ];
        for (variant, expected) in &cases {
            let json = serde_json::to_value(variant).unwrap();
            assert_eq!(
                json, *expected,
                "BadgeVariant::{:?} should serialize as {}",
                variant, expected
            );
            let parsed: BadgeVariant = serde_json::from_value(json).unwrap();
            assert_eq!(&parsed, variant);
        }
    }

    #[test]
    fn checkbox_round_trips() {
        let checkbox = Component::Checkbox(CheckboxProps {
            field: "terms".to_string(),
            label: "Accept Terms".to_string(),
            description: Some("You must accept the terms".to_string()),
            checked: Some(true),
            data_path: None,
            required: Some(true),
            disabled: Some(false),
            error: None,
        });
        let json = serde_json::to_value(&checkbox).unwrap();
        assert_eq!(json["type"], "Checkbox");
        assert_eq!(json["field"], "terms");
        assert_eq!(json["checked"], true);
        assert_eq!(json["description"], "You must accept the terms");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, checkbox);
    }

    #[test]
    fn switch_round_trips() {
        let switch = Component::Switch(SwitchProps {
            field: "notifications".to_string(),
            label: "Enable Notifications".to_string(),
            description: Some("Receive email notifications".to_string()),
            checked: Some(false),
            data_path: None,
            required: None,
            disabled: Some(false),
            error: None,
        });
        let json = serde_json::to_value(&switch).unwrap();
        assert_eq!(json["type"], "Switch");
        assert_eq!(json["field"], "notifications");
        assert_eq!(json["checked"], false);
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, switch);
    }

    #[test]
    fn separator_defaults_to_horizontal() {
        let json = r#"{"type": "Separator"}"#;
        let component: Component = serde_json::from_str(json).unwrap();
        match component {
            Component::Separator(props) => {
                assert_eq!(props.orientation, None);
                // When orientation is None, frontend defaults to horizontal.
                // Explicit horizontal also round-trips correctly:
                let explicit = Component::Separator(SeparatorProps {
                    orientation: Some(Orientation::Horizontal),
                });
                let v = serde_json::to_value(&explicit).unwrap();
                assert_eq!(v["orientation"], "horizontal");
                let parsed: Component = serde_json::from_value(v).unwrap();
                assert_eq!(parsed, explicit);
            }
            _ => panic!("expected Separator"),
        }
    }

    #[test]
    fn description_list_with_format() {
        let dl = Component::DescriptionList(DescriptionListProps {
            items: vec![
                DescriptionItem {
                    label: "Created".to_string(),
                    value: "2026-01-15".to_string(),
                    format: Some(ColumnFormat::Date),
                },
                DescriptionItem {
                    label: "Name".to_string(),
                    value: "Alice".to_string(),
                    format: None,
                },
            ],
            columns: Some(2),
        });
        let json = serde_json::to_value(&dl).unwrap();
        assert_eq!(json["type"], "DescriptionList");
        assert_eq!(json["columns"], 2);
        assert_eq!(json["items"][0]["format"], "date");
        assert!(json["items"][1].get("format").is_none());
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, dl);
    }

    #[test]
    fn checkbox_with_error() {
        let checkbox = Component::Checkbox(CheckboxProps {
            field: "agree".to_string(),
            label: "I agree".to_string(),
            description: None,
            checked: None,
            data_path: None,
            required: Some(true),
            disabled: None,
            error: Some("You must agree".to_string()),
        });
        let json = serde_json::to_value(&checkbox).unwrap();
        assert_eq!(json["error"], "You must agree");
        assert!(json.get("description").is_none());
        assert!(json.get("checked").is_none());
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, checkbox);
    }

    #[test]
    fn tabs_round_trips() {
        let tabs = Component::Tabs(TabsProps {
            default_tab: "general".to_string(),
            tabs: vec![
                Tab {
                    value: "general".to_string(),
                    label: "General".to_string(),
                    children: vec![ComponentNode {
                        key: "name-input".to_string(),
                        component: Component::Input(InputProps {
                            field: "name".to_string(),
                            label: "Name".to_string(),
                            input_type: InputType::Text,
                            placeholder: None,
                            required: None,
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: None,
                        }),
                        action: None,
                        visibility: None,
                    }],
                },
                Tab {
                    value: "security".to_string(),
                    label: "Security".to_string(),
                    children: vec![ComponentNode {
                        key: "password-input".to_string(),
                        component: Component::Input(InputProps {
                            field: "password".to_string(),
                            label: "Password".to_string(),
                            input_type: InputType::Password,
                            placeholder: None,
                            required: None,
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: None,
                        }),
                        action: None,
                        visibility: None,
                    }],
                },
            ],
        });
        let json = serde_json::to_string(&tabs).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, tabs);
    }

    #[test]
    fn breadcrumb_round_trips() {
        let breadcrumb = Component::Breadcrumb(BreadcrumbProps {
            items: vec![
                BreadcrumbItem {
                    label: "Home".to_string(),
                    url: Some("/".to_string()),
                },
                BreadcrumbItem {
                    label: "Users".to_string(),
                    url: Some("/users".to_string()),
                },
                BreadcrumbItem {
                    label: "Edit User".to_string(),
                    url: None,
                },
            ],
        });
        let json = serde_json::to_string(&breadcrumb).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, breadcrumb);

        // Verify last item has no URL serialized
        let value = serde_json::to_value(&breadcrumb).unwrap();
        assert!(value["items"][2].get("url").is_none());
    }

    #[test]
    fn pagination_round_trips() {
        let pagination = Component::Pagination(PaginationProps {
            current_page: 3,
            per_page: 25,
            total: 150,
            base_url: None,
        });
        let json = serde_json::to_string(&pagination).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pagination);
    }

    #[test]
    fn progress_round_trips() {
        let progress = Component::Progress(ProgressProps {
            value: 75,
            max: Some(100),
            label: Some("Uploading...".to_string()),
        });
        let json = serde_json::to_string(&progress).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, progress);

        let value = serde_json::to_value(&progress).unwrap();
        assert_eq!(value["value"], 75);
        assert_eq!(value["max"], 100);
        assert_eq!(value["label"], "Uploading...");
    }

    #[test]
    fn avatar_with_fallback() {
        let avatar = Component::Avatar(AvatarProps {
            src: None,
            alt: "John Doe".to_string(),
            fallback: Some("JD".to_string()),
            size: Some(Size::Lg),
        });
        let json = serde_json::to_string(&avatar).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, avatar);

        let value = serde_json::to_value(&avatar).unwrap();
        assert!(value.get("src").is_none());
        assert_eq!(value["fallback"], "JD");
        assert_eq!(value["size"], "lg");
    }

    #[test]
    fn skeleton_round_trips() {
        let skeleton = Component::Skeleton(SkeletonProps {
            width: Some("100%".to_string()),
            height: Some("40px".to_string()),
            rounded: Some(true),
        });
        let json = serde_json::to_string(&skeleton).unwrap();
        let parsed: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, skeleton);

        let value = serde_json::to_value(&skeleton).unwrap();
        assert_eq!(value["width"], "100%");
        assert_eq!(value["height"], "40px");
        assert_eq!(value["rounded"], true);
    }

    #[test]
    fn tabs_deserializes_from_json() {
        let json = r#"{
            "type": "Tabs",
            "default_tab": "general",
            "tabs": [
                {
                    "value": "general",
                    "label": "General",
                    "children": [
                        {
                            "key": "name-input",
                            "type": "Input",
                            "field": "name",
                            "label": "Name"
                        }
                    ]
                },
                {
                    "value": "security",
                    "label": "Security"
                }
            ]
        }"#;
        let component: Component = serde_json::from_str(json).unwrap();
        match component {
            Component::Tabs(props) => {
                assert_eq!(props.default_tab, "general");
                assert_eq!(props.tabs.len(), 2);
                assert_eq!(props.tabs[0].value, "general");
                assert_eq!(props.tabs[0].children.len(), 1);
                assert_eq!(props.tabs[1].value, "security");
                assert!(props.tabs[1].children.is_empty());
            }
            _ => panic!("expected Tabs"),
        }
    }

    #[test]
    fn input_data_path_round_trips() {
        let input = Component::Input(InputProps {
            field: "name".to_string(),
            label: "Name".to_string(),
            input_type: InputType::Text,
            placeholder: None,
            required: None,
            disabled: None,
            error: None,
            description: None,
            default_value: None,
            data_path: Some("/data/user/name".to_string()),
        });
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(json["data_path"], "/data/user/name");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, input);
    }

    #[test]
    fn select_data_path_round_trips() {
        let select = Component::Select(SelectProps {
            field: "role".to_string(),
            label: "Role".to_string(),
            options: vec![SelectOption {
                value: "admin".to_string(),
                label: "Admin".to_string(),
            }],
            placeholder: None,
            required: None,
            disabled: None,
            error: None,
            description: None,
            default_value: None,
            data_path: Some("/data/user/role".to_string()),
        });
        let json = serde_json::to_value(&select).unwrap();
        assert_eq!(json["data_path"], "/data/user/role");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, select);
    }

    #[test]
    fn checkbox_data_path_round_trips() {
        let checkbox = Component::Checkbox(CheckboxProps {
            field: "terms".to_string(),
            label: "Accept Terms".to_string(),
            description: None,
            checked: None,
            data_path: Some("/data/user/accepted_terms".to_string()),
            required: None,
            disabled: None,
            error: None,
        });
        let json = serde_json::to_value(&checkbox).unwrap();
        assert_eq!(json["data_path"], "/data/user/accepted_terms");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, checkbox);
    }

    #[test]
    fn switch_data_path_round_trips() {
        let switch = Component::Switch(SwitchProps {
            field: "notifications".to_string(),
            label: "Enable Notifications".to_string(),
            description: None,
            checked: None,
            data_path: Some("/data/user/notifications_enabled".to_string()),
            required: None,
            disabled: None,
            error: None,
        });
        let json = serde_json::to_value(&switch).unwrap();
        assert_eq!(json["data_path"], "/data/user/notifications_enabled");
        let parsed: Component = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, switch);
    }
}
