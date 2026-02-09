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
pub mod visibility;

pub use action::{Action, ActionOutcome, ConfirmDialog, DialogVariant, HttpMethod, NotifyVariant};
pub use component::{
    AlertProps, AlertVariant, BadgeProps, BadgeVariant, ButtonProps, ButtonVariant, CardProps,
    Column, ColumnFormat, Component, ComponentNode, FormProps, InputProps, InputType, ModalProps,
    SelectOption, SelectProps, TableProps, TextElement, TextProps,
};
pub use visibility::{Visibility, VisibilityCondition, VisibilityOperator};

// Re-export serde_json for convenience
pub use serde_json;
