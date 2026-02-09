//! Action resolver for JSON-UI component trees.
//!
//! Walks a `JsonUiView`'s component tree and resolves each `Action.handler`
//! reference to a URL using a caller-provided callback. This keeps
//! ferro-json-ui decoupled from the framework's route registry.

use crate::action::Action;
use crate::component::{Component, ComponentNode};
use crate::view::JsonUiView;

/// Resolve a single action using the callback.
fn resolve_action(action: &mut Action, resolver: &impl Fn(&str) -> Option<String>) {
    if let Some(url) = resolver(&action.handler) {
        action.url = Some(url);
    }
}

/// Recursively resolve all actions within a component node.
fn resolve_component_node(node: &mut ComponentNode, resolver: &impl Fn(&str) -> Option<String>) {
    // Resolve the node-level action (any component can have one).
    if let Some(ref mut action) = node.action {
        resolve_action(action, resolver);
    }

    // Recurse into component-specific children.
    match &mut node.component {
        Component::Card(props) => {
            for child in &mut props.children {
                resolve_component_node(child, resolver);
            }
            for child in &mut props.footer {
                resolve_component_node(child, resolver);
            }
        }
        Component::Form(props) => {
            resolve_action(&mut props.action, resolver);
            for field in &mut props.fields {
                resolve_component_node(field, resolver);
            }
        }
        Component::Modal(props) => {
            for child in &mut props.children {
                resolve_component_node(child, resolver);
            }
            for child in &mut props.footer {
                resolve_component_node(child, resolver);
            }
        }
        Component::Tabs(props) => {
            for tab in &mut props.tabs {
                for child in &mut tab.children {
                    resolve_component_node(child, resolver);
                }
            }
        }
        Component::Table(props) => {
            if let Some(ref mut row_actions) = props.row_actions {
                for action in row_actions {
                    resolve_action(action, resolver);
                }
            }
        }
        // Leaf components with no children or actions to resolve.
        Component::Button(_)
        | Component::Input(_)
        | Component::Select(_)
        | Component::Alert(_)
        | Component::Badge(_)
        | Component::Text(_)
        | Component::Checkbox(_)
        | Component::Switch(_)
        | Component::Separator(_)
        | Component::DescriptionList(_)
        | Component::Breadcrumb(_)
        | Component::Pagination(_)
        | Component::Progress(_)
        | Component::Avatar(_)
        | Component::Skeleton(_) => {}
    }
}

/// Walk the entire component tree and resolve all action handler names to URLs.
///
/// The resolver callback maps a handler name (e.g. `"users.store"`) to an
/// optional URL (e.g. `Some("/users")`). Actions whose handler cannot be
/// resolved are left with `url: None`.
pub fn resolve_actions(view: &mut JsonUiView, resolver: impl Fn(&str) -> Option<String>) {
    for node in &mut view.components {
        resolve_component_node(node, &resolver);
    }
}

/// Walk the entire component tree and resolve all actions, returning an error
/// for any handler that cannot be resolved.
///
/// Returns `Ok(())` if all handlers resolve successfully, or `Err(Vec<String>)`
/// containing the names of all unresolvable handlers.
pub fn resolve_actions_strict(
    view: &mut JsonUiView,
    resolver: impl Fn(&str) -> Option<String>,
) -> Result<(), Vec<String>> {
    let mut unresolved: Vec<String> = Vec::new();

    let collecting_resolver = |handler: &str| -> Option<String> {
        resolver(handler)
    };

    // First resolve everything.
    resolve_actions(view, collecting_resolver);

    // Then collect unresolved handlers by walking the tree again.
    for node in &view.components {
        collect_unresolved_node(node, &mut unresolved);
    }

    if unresolved.is_empty() {
        Ok(())
    } else {
        Err(unresolved)
    }
}

/// Collect handler names from actions that have no resolved URL.
fn collect_unresolved_action(action: &Action, unresolved: &mut Vec<String>) {
    if action.url.is_none() {
        unresolved.push(action.handler.clone());
    }
}

/// Recursively collect unresolved actions from a component node.
fn collect_unresolved_node(node: &ComponentNode, unresolved: &mut Vec<String>) {
    if let Some(ref action) = node.action {
        collect_unresolved_action(action, unresolved);
    }

    match &node.component {
        Component::Card(props) => {
            for child in &props.children {
                collect_unresolved_node(child, unresolved);
            }
            for child in &props.footer {
                collect_unresolved_node(child, unresolved);
            }
        }
        Component::Form(props) => {
            collect_unresolved_action(&props.action, unresolved);
            for field in &props.fields {
                collect_unresolved_node(field, unresolved);
            }
        }
        Component::Modal(props) => {
            for child in &props.children {
                collect_unresolved_node(child, unresolved);
            }
            for child in &props.footer {
                collect_unresolved_node(child, unresolved);
            }
        }
        Component::Tabs(props) => {
            for tab in &props.tabs {
                for child in &tab.children {
                    collect_unresolved_node(child, unresolved);
                }
            }
        }
        Component::Table(props) => {
            if let Some(ref row_actions) = props.row_actions {
                for action in row_actions {
                    collect_unresolved_action(action, unresolved);
                }
            }
        }
        Component::Button(_)
        | Component::Input(_)
        | Component::Select(_)
        | Component::Alert(_)
        | Component::Badge(_)
        | Component::Text(_)
        | Component::Checkbox(_)
        | Component::Switch(_)
        | Component::Separator(_)
        | Component::DescriptionList(_)
        | Component::Breadcrumb(_)
        | Component::Pagination(_)
        | Component::Progress(_)
        | Component::Avatar(_)
        | Component::Skeleton(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::HttpMethod;
    use crate::component::*;

    /// Helper to build a simple action.
    fn make_action(handler: &str) -> Action {
        Action {
            handler: handler.to_string(),
            url: None,
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
        }
    }

    /// Helper resolver that maps known handlers to URLs.
    fn test_resolver(handler: &str) -> Option<String> {
        match handler {
            "users.store" => Some("/users".to_string()),
            "users.show" => Some("/users/{id}".to_string()),
            "users.destroy" => Some("/users/{id}".to_string()),
            "users.create" => Some("/users/create".to_string()),
            "posts.index" => Some("/posts".to_string()),
            _ => None,
        }
    }

    #[test]
    fn resolve_button_with_action() {
        let mut view = JsonUiView::new().component(ComponentNode {
            key: "btn".to_string(),
            component: Component::Button(ButtonProps {
                label: "Create".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
            }),
            action: Some(make_action("users.store")),
            visibility: None,
        });

        resolve_actions(&mut view, test_resolver);

        assert_eq!(
            view.components[0].action.as_ref().unwrap().url,
            Some("/users".to_string())
        );
    }

    #[test]
    fn resolve_nested_card_children() {
        let mut view = JsonUiView::new().component(ComponentNode {
            key: "card".to_string(),
            component: Component::Card(CardProps {
                title: "Users".to_string(),
                description: None,
                children: vec![ComponentNode {
                    key: "btn".to_string(),
                    component: Component::Button(ButtonProps {
                        label: "Create".to_string(),
                        variant: ButtonVariant::Default,
                        size: Size::Default,
                        disabled: None,
                        icon: None,
                        icon_position: None,
                    }),
                    action: Some(make_action("users.create")),
                    visibility: None,
                }],
                footer: vec![ComponentNode {
                    key: "footer-btn".to_string(),
                    component: Component::Button(ButtonProps {
                        label: "Save".to_string(),
                        variant: ButtonVariant::Default,
                        size: Size::Default,
                        disabled: None,
                        icon: None,
                        icon_position: None,
                    }),
                    action: Some(make_action("users.store")),
                    visibility: None,
                }],
            }),
            action: None,
            visibility: None,
        });

        resolve_actions(&mut view, test_resolver);

        match &view.components[0].component {
            Component::Card(props) => {
                assert_eq!(
                    props.children[0].action.as_ref().unwrap().url,
                    Some("/users/create".to_string())
                );
                assert_eq!(
                    props.footer[0].action.as_ref().unwrap().url,
                    Some("/users".to_string())
                );
            }
            _ => panic!("expected Card"),
        }
    }

    #[test]
    fn resolve_form_action() {
        let mut view = JsonUiView::new().component(ComponentNode {
            key: "form".to_string(),
            component: Component::Form(FormProps {
                action: make_action("users.store"),
                fields: vec![ComponentNode {
                    key: "name".to_string(),
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
                method: None,
            }),
            action: None,
            visibility: None,
        });

        resolve_actions(&mut view, test_resolver);

        match &view.components[0].component {
            Component::Form(props) => {
                assert_eq!(props.action.url, Some("/users".to_string()));
            }
            _ => panic!("expected Form"),
        }
    }

    #[test]
    fn resolve_table_row_actions() {
        let mut view = JsonUiView::new().component(ComponentNode {
            key: "table".to_string(),
            component: Component::Table(TableProps {
                columns: vec![Column {
                    key: "name".to_string(),
                    label: "Name".to_string(),
                    format: None,
                }],
                data_path: "/data/users".to_string(),
                row_actions: Some(vec![
                    make_action("users.show"),
                    make_action("users.destroy"),
                ]),
                empty_message: None,
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        });

        resolve_actions(&mut view, test_resolver);

        match &view.components[0].component {
            Component::Table(props) => {
                let row_actions = props.row_actions.as_ref().unwrap();
                assert_eq!(row_actions[0].url, Some("/users/{id}".to_string()));
                assert_eq!(row_actions[1].url, Some("/users/{id}".to_string()));
            }
            _ => panic!("expected Table"),
        }
    }

    #[test]
    fn resolve_tabs_children() {
        let mut view = JsonUiView::new().component(ComponentNode {
            key: "tabs".to_string(),
            component: Component::Tabs(TabsProps {
                default_tab: "general".to_string(),
                tabs: vec![
                    Tab {
                        value: "general".to_string(),
                        label: "General".to_string(),
                        children: vec![ComponentNode {
                            key: "btn1".to_string(),
                            component: Component::Button(ButtonProps {
                                label: "Save".to_string(),
                                variant: ButtonVariant::Default,
                                size: Size::Default,
                                disabled: None,
                                icon: None,
                                icon_position: None,
                            }),
                            action: Some(make_action("users.store")),
                            visibility: None,
                        }],
                    },
                    Tab {
                        value: "posts".to_string(),
                        label: "Posts".to_string(),
                        children: vec![ComponentNode {
                            key: "btn2".to_string(),
                            component: Component::Button(ButtonProps {
                                label: "View Posts".to_string(),
                                variant: ButtonVariant::Default,
                                size: Size::Default,
                                disabled: None,
                                icon: None,
                                icon_position: None,
                            }),
                            action: Some(make_action("posts.index")),
                            visibility: None,
                        }],
                    },
                ],
            }),
            action: None,
            visibility: None,
        });

        resolve_actions(&mut view, test_resolver);

        match &view.components[0].component {
            Component::Tabs(props) => {
                assert_eq!(
                    props.tabs[0].children[0].action.as_ref().unwrap().url,
                    Some("/users".to_string())
                );
                assert_eq!(
                    props.tabs[1].children[0].action.as_ref().unwrap().url,
                    Some("/posts".to_string())
                );
            }
            _ => panic!("expected Tabs"),
        }
    }

    #[test]
    fn resolve_modal_children_and_footer() {
        let mut view = JsonUiView::new().component(ComponentNode {
            key: "modal".to_string(),
            component: Component::Modal(ModalProps {
                title: "Confirm".to_string(),
                description: None,
                children: vec![ComponentNode {
                    key: "info".to_string(),
                    component: Component::Text(TextProps {
                        content: "Are you sure?".to_string(),
                        element: TextElement::P,
                    }),
                    action: None,
                    visibility: None,
                }],
                footer: vec![ComponentNode {
                    key: "confirm-btn".to_string(),
                    component: Component::Button(ButtonProps {
                        label: "Delete".to_string(),
                        variant: ButtonVariant::Destructive,
                        size: Size::Default,
                        disabled: None,
                        icon: None,
                        icon_position: None,
                    }),
                    action: Some(make_action("users.destroy")),
                    visibility: None,
                }],
                trigger_label: Some("Open".to_string()),
            }),
            action: None,
            visibility: None,
        });

        resolve_actions(&mut view, test_resolver);

        match &view.components[0].component {
            Component::Modal(props) => {
                assert_eq!(
                    props.footer[0].action.as_ref().unwrap().url,
                    Some("/users/{id}".to_string())
                );
            }
            _ => panic!("expected Modal"),
        }
    }

    #[test]
    fn unresolvable_handler_leaves_url_none() {
        let mut view = JsonUiView::new().component(ComponentNode {
            key: "btn".to_string(),
            component: Component::Button(ButtonProps {
                label: "Unknown".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
            }),
            action: Some(make_action("nonexistent.handler")),
            visibility: None,
        });

        resolve_actions(&mut view, test_resolver);

        assert_eq!(view.components[0].action.as_ref().unwrap().url, None);
    }

    #[test]
    fn strict_with_missing_handler_returns_error() {
        let mut view = JsonUiView::new()
            .component(ComponentNode {
                key: "btn1".to_string(),
                component: Component::Button(ButtonProps {
                    label: "OK".to_string(),
                    variant: ButtonVariant::Default,
                    size: Size::Default,
                    disabled: None,
                    icon: None,
                    icon_position: None,
                }),
                action: Some(make_action("users.store")),
                visibility: None,
            })
            .component(ComponentNode {
                key: "btn2".to_string(),
                component: Component::Button(ButtonProps {
                    label: "Bad".to_string(),
                    variant: ButtonVariant::Default,
                    size: Size::Default,
                    disabled: None,
                    icon: None,
                    icon_position: None,
                }),
                action: Some(make_action("unknown.handler")),
                visibility: None,
            });

        let result = resolve_actions_strict(&mut view, test_resolver);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors, vec!["unknown.handler"]);

        // The known handler should still be resolved.
        assert_eq!(
            view.components[0].action.as_ref().unwrap().url,
            Some("/users".to_string())
        );
    }

    #[test]
    fn strict_with_all_resolved_returns_ok() {
        let mut view = JsonUiView::new().component(ComponentNode {
            key: "btn".to_string(),
            component: Component::Button(ButtonProps {
                label: "Create".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
            }),
            action: Some(make_action("users.store")),
            visibility: None,
        });

        let result = resolve_actions_strict(&mut view, test_resolver);
        assert!(result.is_ok());
    }
}
