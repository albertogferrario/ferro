//! # Ferro JSON-UI
//!
//! JSON-based server-driven UI schema types for the Ferro framework.
//!
//! This crate defines the v2 `Spec` foundation: a flat, ID-keyed element map
//! with parse-time structural validation. Typed `*Props` structs describe
//! per-component prop shape and feed the Phase 117 catalog via `JsonSchema`.
//!
//! ## Schema Structure
//!
//! A JSON-UI Spec consists of:
//! - **Spec** - Top-level container: `$schema`, `root`, `elements`, `title?`, `layout?`, `data?`
//! - **Element** - Single UI node: `type` (string), `props` (Value), `children` (Vec<String> of IDs), `action?`, `visible?`
//! - **Actions** - Handler references with confirmations and outcomes
//! - **Visibility** - Conditional rendering based on data conditions
//!
//! ## Example
//!
//! ```rust
//! use ferro_json_ui::{Spec, Element};
//!
//! let spec = Spec::builder()
//!     .title("Demo")
//!     .element("root", Element::new("Text").prop("content", "Hi"))
//!     .build()
//!     .unwrap();
//! ```

pub mod action;
pub mod assets;
pub mod catalog;
pub mod component;
pub mod config;
pub mod data;
pub mod expression;
pub mod layout;
pub mod loader;
pub mod plugin;
pub mod plugins;
pub mod render;
pub mod resolve;
pub mod spec;
pub mod visibility;

pub(crate) mod runtime;

pub use action::{Action, ActionOutcome, ConfirmDialog, DialogVariant, HttpMethod, NotifyVariant};
pub use assets::FERRO_BASE_CSS;
pub use component::{
    ActionCardProps, ActionCardVariant, AlertProps, AlertVariant, AvatarProps, BadgeProps,
    BadgeVariant, BreadcrumbItem, BreadcrumbProps, ButtonGroupProps, ButtonProps, ButtonType,
    ButtonVariant, CardProps, CardVariant, CheckboxListProps, CheckboxProps, ChecklistItem,
    ChecklistProps, CollapsibleProps, Column, ColumnFormat, DataTableProps, DescriptionItem,
    DescriptionListProps, DropdownMenuAction, DropdownMenuProps, EmptyStateProps, FormMaxWidth,
    FormProps, FormSectionProps, GapSize, GridProps, HeaderProps, IconPosition, ImageProps,
    InputProps, InputType, KanbanBoardProps, KanbanColumnProps, ModalProps,
    NotificationDropdownProps, NotificationItem, Orientation, PageHeaderProps, PaginationProps,
    ProductTileProps, ProgressProps, RawHtmlProps, RichTextEditorProps, SelectOption, SelectProps,
    SeparatorProps, SidebarGroup, SidebarNavItem, SidebarProps, Size, SkeletonProps, SortDirection,
    StatCardProps, SwitchProps, Tab, TableProps, TabsProps, TextElement, TextProps, ToastProps,
    ToastVariant,
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
pub use catalog::{global_catalog, Catalog, CatalogError, ComponentSpec};
pub use expression::resolve_expressions;
pub use loader::{load_cached, LoadError};
pub use plugin::{
    collect_plugin_assets, global_plugin_registry, register_plugin, registered_plugin_types,
    with_plugin, Asset, CollectedAssets, JsonUiPlugin, PluginRegistry,
};
pub use plugins::{register_built_in_plugins, MapPlugin, RichTextEditorPlugin};
pub use render::{render_spec_to_html, render_spec_to_html_with_plugins, RenderResult};
pub use resolve::{
    expand_directives, resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all,
};
pub use spec::{
    DataRef, Element, ElementBuilder, Spec, SpecBuilder, SpecError, TitleBinding,
    MAX_NESTING_DEPTH, SCHEMA_VERSION,
};
pub use visibility::{Visibility, VisibilityCondition, VisibilityOperator};

#[cfg(feature = "projections")]
pub mod projection;

#[cfg(feature = "projections")]
pub use projection::{JsonUiRenderer, ProjectionError, RenderMode, VisualContext};
