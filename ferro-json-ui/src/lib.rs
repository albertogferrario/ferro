//! # Ferro JSON-UI
//!
//! **Experimental:** The component schema and plugin interface may evolve.
//!
//! JSON-based server-driven UI schema types for the Ferro framework.
//!
//! This crate defines the typed foundation for JSON-UI: a declarative
//! component system where the server sends JSON descriptions that are
//! rendered to HTML. Components, actions, and visibility rules are all
//! defined as Rust types with serde serialization.
//!
//! ## Schema Structure
//!
//! A JSON-UI view consists of:
//! - **Components** - UI elements (Card, Table, Form, Button, etc.)
//! - **Actions** - Handler references with confirmations and outcomes
//! - **Visibility** - Conditional rendering based on data conditions
//! - **View** - Top-level container with layout and title
//!
//! ## Example
//!
//! ```rust
//! use ferro_json_ui::{JsonUiView, ComponentNode, Component, CardProps};
//!
//! let view = JsonUiView::new()
//!     .title("Users")
//!     .component(ComponentNode {
//!         key: "header".to_string(),
//!         component: Component::Card(CardProps {
//!             title: "User Management".to_string(),
//!             description: None,
//!             children: vec![],
//!             footer: vec![],
//!         }),
//!         action: None,
//!         visibility: None,
//!     });
//!
//! let json = view.to_json().unwrap();
//! assert!(json.contains("\"$schema\":\"ferro-json-ui/v1\""));
//! ```

pub mod action;
pub mod component;
pub mod config;
pub mod data;
pub mod layout;
pub mod plugin;
pub mod plugins;
pub mod render;
pub mod resolve;
pub mod view;
pub mod visibility;

pub(crate) mod runtime;

pub use action::{Action, ActionOutcome, ConfirmDialog, DialogVariant, HttpMethod, NotifyVariant};
pub use component::{
    AlertProps, AlertVariant, AvatarProps, BadgeProps, BadgeVariant, BreadcrumbItem,
    BreadcrumbProps, ButtonProps, ButtonVariant, CardProps, CheckboxProps, ChecklistItem,
    ChecklistProps, Column, ColumnFormat, Component, ComponentNode, DescriptionItem,
    DescriptionListProps, FormProps, HeaderProps, IconPosition, InputProps, InputType, ModalProps,
    NotificationDropdownProps, NotificationItem, Orientation, PaginationProps, PluginProps,
    ProgressProps, SelectOption, SelectProps, SeparatorProps, SidebarGroup, SidebarNavItem,
    SidebarProps, Size, SkeletonProps, SortDirection, StatCardProps, SwitchProps, Tab, TableProps,
    TabsProps, TextElement, TextProps, ToastProps, ToastVariant,
};
pub use config::JsonUiConfig;
pub use data::{resolve_path, resolve_path_string};
pub use layout::{
    footer, global_registry, navigation, register_layout, render_layout, sidebar, AppLayout,
    AuthLayout, DefaultLayout, Layout, LayoutContext, LayoutRegistry, NavItem, SidebarSection,
};
pub use plugin::{
    collect_plugin_assets, global_plugin_registry, register_plugin, registered_plugin_types,
    with_plugin, Asset, CollectedAssets, JsonUiPlugin, PluginRegistry,
};
pub use plugins::{register_built_in_plugins, MapPlugin};
pub use render::{collect_plugin_types, render_to_html, render_to_html_with_plugins, RenderResult};
pub use resolve::{resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all};
pub use view::{JsonUiView, SCHEMA_VERSION};
pub use visibility::{Visibility, VisibilityCondition, VisibilityOperator};

// Re-export serde_json for convenience
pub use serde_json;
