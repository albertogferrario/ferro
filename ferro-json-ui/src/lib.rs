//! # Ferro JSON-UI
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
pub mod render;
pub mod resolve;
pub mod view;
pub mod visibility;

pub use action::{Action, ActionOutcome, ConfirmDialog, DialogVariant, HttpMethod, NotifyVariant};
pub use component::{
    AlertProps, AlertVariant, AvatarProps, BadgeProps, BadgeVariant, BreadcrumbItem,
    BreadcrumbProps, ButtonProps, ButtonVariant, CardProps, CheckboxProps, Column, ColumnFormat,
    Component, ComponentNode, DescriptionItem, DescriptionListProps, FormProps, IconPosition,
    InputProps, InputType, ModalProps, Orientation, PaginationProps, ProgressProps, SelectOption,
    SelectProps, SeparatorProps, Size, SkeletonProps, SortDirection, SwitchProps, Tab, TableProps,
    TabsProps, TextElement, TextProps,
};
pub use config::JsonUiConfig;
pub use data::{resolve_path, resolve_path_string};
pub use render::render_to_html;
pub use resolve::{resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all};
pub use view::{JsonUiView, SCHEMA_VERSION};
pub use visibility::{Visibility, VisibilityCondition, VisibilityOperator};

// Re-export serde_json for convenience
pub use serde_json;
