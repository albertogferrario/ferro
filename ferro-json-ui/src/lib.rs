//! # Ferro JSON-UI
//!
//! Stable JSON-based server-driven UI schema types for the Ferro framework.
//!
//! This crate defines the typed foundation for JSON-UI: a declarative
//! component system where the server sends JSON descriptions that are
//! rendered to HTML. Components, actions, and visibility rules are all
//! defined as Rust types with serde serialization and JSON Schema generation.
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
    ChecklistProps, CollapsibleProps, Column, ColumnFormat, Component, ComponentNode,
    DescriptionItem, DescriptionListProps, EmptyStateProps, FormProps, FormSectionProps, GapSize,
    GridProps, HeaderProps, IconPosition, InputProps, InputType, ModalProps,
    NotificationDropdownProps, NotificationItem, Orientation, PaginationProps, PluginProps,
    ProgressProps, SelectOption, SelectProps, SeparatorProps, SidebarGroup, SidebarNavItem,
    SidebarProps, Size, SkeletonProps, SortDirection, StatCardProps, SwitchProps, Tab, TableProps,
    TabsProps, TextElement, TextProps, ToastProps, ToastVariant,
};
pub use config::JsonUiConfig;
// resolve_path and resolve_path_string are pub(crate) — internal render pipeline helpers
pub use layout::{
    register_layout, render_layout, DashboardLayout, DashboardLayoutConfig, Layout, LayoutContext,
    LayoutRegistry, NavItem, SidebarSection,
};
// AppLayout, AuthLayout, DefaultLayout are pub in layout.rs but not user-facing — users select
// layouts by name string ("dashboard", "app", "auth"), not by struct.
// navigation, sidebar, footer, global_registry are framework-internal.
pub use plugin::{
    collect_plugin_assets, global_plugin_registry, register_plugin, registered_plugin_types,
    with_plugin, Asset, CollectedAssets, JsonUiPlugin, PluginRegistry,
};
pub use plugins::{register_built_in_plugins, MapPlugin};
pub use render::{render_to_html, render_to_html_with_plugins, RenderResult};
// collect_plugin_types is pub(crate) — internal render pipeline helper
pub use resolve::{resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all};
pub use view::{JsonUiView, SCHEMA_VERSION};
pub use visibility::{Visibility, VisibilityCondition, VisibilityOperator};
