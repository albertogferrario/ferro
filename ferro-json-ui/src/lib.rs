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
    BreadcrumbProps, ButtonGroupProps, ButtonProps, ButtonType, ButtonVariant, CardProps, CheckboxProps,
    ChecklistItem, ChecklistProps, CollapsibleProps, Column, ColumnFormat, Component,
    ComponentNode, DataTableProps, DescriptionItem, DescriptionListProps, DropdownMenuAction,
    DropdownMenuProps, EmptyStateProps, FormProps, FormSectionProps, GapSize, GridProps,
    HeaderProps, IconPosition, InputProps, InputType, KanbanBoardProps, KanbanColumnProps,
    ModalProps,
    NotificationDropdownProps, NotificationItem, Orientation, PageHeaderProps, PaginationProps,
    PluginProps, ProductTileProps, ProgressProps, SelectOption, SelectProps, SeparatorProps,
    SidebarGroup, SidebarNavItem, SidebarProps, Size, SkeletonProps, SortDirection, StatCardProps,
    SwitchProps, Tab, TableProps, TabsProps, TextElement, TextProps, ToastProps, ToastVariant,
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

/// Concise reference of all JSON-UI components for AI generation prompts.
///
/// Used by ferro-cli (AI view generation) and ferro-mcp (json_ui_generate tool)
/// as shared context for LLM-based code generation.
pub const COMPONENT_CATALOG: &str = r#"## Component Catalog

### Text
Props: content (String), element (h1|h2|h3|span|div|section|p)

### Button
Props: label (String), variant (default|secondary|destructive|outline|ghost|link), size (xs|sm|default|lg), disabled (Option<bool>), icon (Option<String>), icon_position (Option<left|right>)

### Card
Props: title (String), description (Option<String>), children (Vec<ComponentNode>), footer (Vec<ComponentNode>)

### Table
Props: columns (Vec<Column {key, label, format?}>), data_path (String), row_actions (Option<Vec<Action>>), empty_message (Option<String>), sortable (Option<bool>), sort_column (Option<String>), sort_direction (Option<asc|desc>)

### Form
Props: action (Action), fields (Vec<ComponentNode>), method (Option<GET|POST|PUT|PATCH|DELETE>)

### Input
Props: field (String), label (String), input_type (text|email|password|number|textarea|hidden|date|time|url|tel|search), placeholder (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>), description (Option<String>), default_value (Option<String>), data_path (Option<String>), step (Option<String>)

### Select
Props: field (String), label (String), options (Vec<SelectOption {value, label}>), placeholder (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>), description (Option<String>), default_value (Option<String>), data_path (Option<String>)

### Alert
Props: message (String), variant (info|success|warning|error), title (Option<String>)

### Badge
Props: label (String), variant (default|secondary|destructive|outline)

### Modal
Props: title (String), description (Option<String>), children (Vec<ComponentNode>), footer (Vec<ComponentNode>), trigger_label (Option<String>)

### Checkbox
Props: field (String), label (String), description (Option<String>), checked (Option<bool>), data_path (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>)

### Switch
Props: field (String), label (String), description (Option<String>), checked (Option<bool>), data_path (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>)

### Separator
Props: orientation (Option<horizontal|vertical>)

### DescriptionList
Props: items (Vec<DescriptionItem {label, value, format?}>), columns (Option<u8>)

### Tabs
Props: default_tab (String), tabs (Vec<Tab {value, label, children}>)

### Breadcrumb
Props: items (Vec<BreadcrumbItem {label, url?}>)

### Pagination
Props: current_page (u32), per_page (u32), total (u32), base_url (Option<String>)

### Progress
Props: value (u8 0-100), max (Option<u8>), label (Option<String>)

### Avatar
Props: src (Option<String>), alt (String), fallback (Option<String>), size (Option<xs|sm|default|lg>)

### Skeleton
Props: width (Option<String>), height (Option<String>), rounded (Option<bool>)

## Plugin Components

Plugin components use the same JSON syntax as built-in components. Their JS/CSS assets are loaded automatically.

### Map
Props: center (Option<[f64; 2]>), zoom (u8 0-18, default 13), height (String, default "400px"), fit_bounds (Option<bool>), markers (Vec<{lat, lng, popup?, color?, popup_html?, href?}>), tile_url (Option<String>), attribution (Option<String>), max_zoom (Option<u8>)
Example JSON: {"type": "Map", "fit_bounds": true, "markers": [{"lat": 51.5, "lng": -0.09, "popup": "Hello"}]}
Note: Leaflet CSS/JS loaded via CDN automatically. Works inside Tabs/Modals (IntersectionObserver handles resize).

## Action
Props: handler (String "controller.method" format), method (GET|POST|PUT|PATCH|DELETE), confirm (Option<ConfirmDialog {title, message?, variant: default|danger}>), on_success (Option<ActionOutcome>), on_error (Option<ActionOutcome>)
Builders: Action::new("handler") (POST), Action::get("handler"), Action::delete("handler"), .confirm("title"), .confirm_danger("title")

## ComponentNode
Wraps every component: key (String), component (Component variant), action (Option<Action>), visibility (Option<Visibility>)

## JsonUiView Builder
JsonUiView::new().title("Title").layout("app").data(json).component(node).components(vec_of_nodes)
"#;
