//! Action declarations for JSON-UI components.
//!
//! Actions map user interactions (button clicks, form submissions) to
//! backend Ferro handlers. Each action references a handler in
//! `"controller.method"` format and can include confirmation dialogs
//! and outcome behaviors.

use serde::{Deserialize, Serialize};

/// Variant for confirmation dialogs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DialogVariant {
    #[default]
    Default,
    Danger,
}

/// HTTP method for action requests.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    #[default]
    Post,
    Put,
    Patch,
    Delete,
}

/// Confirmation dialog shown before executing an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfirmDialog {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default)]
    pub variant: DialogVariant,
}

/// Notification variant for action outcomes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyVariant {
    #[default]
    Success,
    Info,
    Warning,
    Error,
}

/// Outcome after an action completes (success or error).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionOutcome {
    Redirect {
        url: String,
    },
    ShowErrors,
    Refresh,
    Notify {
        message: String,
        variant: NotifyVariant,
    },
}

/// An action declaration mapping a user interaction to a backend handler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    /// Handler reference in "controller.method" format.
    pub handler: String,
    #[serde(default)]
    pub method: HttpMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmDialog>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_success: Option<ActionOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_error: Option<ActionOutcome>,
}
