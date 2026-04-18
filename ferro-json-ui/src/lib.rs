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
pub mod catalog;
pub mod component;
pub mod config;
pub mod data;
pub mod layout;
pub mod plugin;
pub mod plugins;
pub mod render;
pub mod resolve;
pub mod spec;
pub mod visibility;

pub(crate) mod runtime;

pub use action::{Action, ActionOutcome, ConfirmDialog, DialogVariant, HttpMethod, NotifyVariant};
pub use component::{
    ActionCardProps, ActionCardVariant, AlertProps, AlertVariant, AvatarProps, BadgeProps,
    BadgeVariant, BreadcrumbItem, BreadcrumbProps, ButtonGroupProps, ButtonProps, ButtonType,
    ButtonVariant, CardProps, CheckboxProps, ChecklistItem, ChecklistProps, CollapsibleProps,
    Column, ColumnFormat, DataTableProps, DescriptionItem, DescriptionListProps,
    DropdownMenuAction, DropdownMenuProps, EmptyStateProps, FormMaxWidth, FormProps,
    FormSectionProps, GapSize, GridProps, HeaderProps, IconPosition, ImageProps, InputProps,
    InputType, KanbanBoardProps, KanbanColumnProps, ModalProps, NotificationDropdownProps,
    NotificationItem, Orientation, PageHeaderProps, PaginationProps, ProductTileProps,
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
pub use catalog::{global_catalog, Catalog, CatalogError, ComponentSpec};
pub use plugin::{
    collect_plugin_assets, global_plugin_registry, register_plugin, registered_plugin_types,
    with_plugin, Asset, CollectedAssets, JsonUiPlugin, PluginRegistry,
};
pub use plugins::{register_built_in_plugins, MapPlugin};
pub use render::{render_spec_to_html, render_spec_to_html_with_plugins, RenderResult};
pub use resolve::{resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all};
pub use spec::{
    Element, ElementBuilder, Spec, SpecBuilder, SpecError, MAX_NESTING_DEPTH, SCHEMA_VERSION,
};
pub use visibility::{Visibility, VisibilityCondition, VisibilityOperator};

#[cfg(feature = "projections")]
pub mod projection;

#[cfg(feature = "projections")]
pub use projection::{JsonUiRenderer, RenderMode, VisualContext};

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
Props: title (String), description (Option<String>), max_width (Option<FormMaxWidth>), footer (Vec<String> of element IDs). Body children come from Element.children.

### Table
Props: columns (Vec<Column {key, label, format?}>), data_path (String), row_actions (Option<Vec<Action>>), empty_message (Option<String>), sortable (Option<bool>), sort_column (Option<String>), sort_direction (Option<asc|desc>)

### Form
Props: action (Action), method (Option<GET|POST|PUT|PATCH|DELETE>). Form fields come from Element.children.

### Input
Props: field (String), label (String), input_type (text|email|password|number|textarea|hidden|date|time|url|tel|search), placeholder (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>), description (Option<String>), default_value (Option<String>), data_path (Option<String>), step (Option<String>)

### Select
Props: field (String), label (String), options (Vec<SelectOption {value, label}>), placeholder (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>), description (Option<String>), default_value (Option<String>), data_path (Option<String>)

### Alert
Props: message (String), variant (info|success|warning|error), title (Option<String>)

### Badge
Props: label (String), variant (default|secondary|destructive|outline)

### Modal
Props: id (String), title (String), description (Option<String>), trigger_label (Option<String>), footer (Vec<String> of element IDs). Body children come from Element.children.

### Checkbox
Props: field (String), label (String), description (Option<String>), checked (Option<bool>), data_path (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>)

### Switch
Props: field (String), label (String), description (Option<String>), checked (Option<bool>), data_path (Option<String>), required (Option<bool>), disabled (Option<bool>), error (Option<String>)

### Separator
Props: orientation (Option<horizontal|vertical>)

### DescriptionList
Props: items (Vec<DescriptionItem {label, value, format?}>), columns (Option<u8>)

### Tabs
Props: default_tab (String), tabs (Vec<Tab {value, label, children: Vec<String> of element IDs}>)

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

### PageHeader
Props: title (String), breadcrumb (Vec<BreadcrumbItem>), actions (Vec<String> of element IDs)

### KanbanBoard
Props: columns (Vec<KanbanColumnProps {id, title, count, children: Vec<String> of element IDs}>)

## Plugin Components

Plugin components use the same JSON syntax as built-in components. Their JS/CSS assets are loaded automatically.

### Map
Props: center (Option<[f64; 2]>), zoom (u8 0-18, default 13), height (String, default "400px"), fit_bounds (Option<bool>), markers (Vec<{lat, lng, popup?, color?, popup_html?, href?}>), tile_url (Option<String>), attribution (Option<String>), max_zoom (Option<u8>)
Example JSON: {"type": "Map", "fit_bounds": true, "markers": [{"lat": 51.5, "lng": -0.09, "popup": "Hello"}]}
Note: Leaflet CSS/JS loaded via CDN automatically. Works inside Tabs/Modals (IntersectionObserver handles resize).

## Action
Props: handler (String "controller.method" format), method (GET|POST|PUT|PATCH|DELETE), confirm (Option<ConfirmDialog {title, message?, variant: default|danger}>), on_success (Option<ActionOutcome>), on_error (Option<ActionOutcome>)
Builders: Action::new("handler") (POST), Action::get("handler"), Action::delete("handler"), .confirm("title"), .confirm_danger("title")

## Element
Every Spec element: type (String), props (Value), children (Vec<String> of element IDs), action (Option<Action>), visible (Option<Visibility>)

## Spec Builder
Spec::builder().title("Title").layout("app").data(json).element("id", Element::new("Type").prop(k, v).child("child-id")).build()
"#;
