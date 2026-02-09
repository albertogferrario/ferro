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
    /// Resolved URL for this action. Populated by the resolver at render time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub method: HttpMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmDialog>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_success: Option<ActionOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_error: Option<ActionOutcome>,
}

impl Action {
    /// Create an action with Post method (the default for form submissions).
    pub fn new(handler: impl Into<String>) -> Self {
        Self {
            handler: handler.into(),
            url: None,
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
        }
    }

    /// Create a navigation action with Get method.
    pub fn get(handler: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Get,
            ..Self::new(handler)
        }
    }

    /// Create a deletion action with Delete method.
    pub fn delete(handler: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Delete,
            ..Self::new(handler)
        }
    }

    /// Override the HTTP method.
    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    /// Add a default confirmation dialog.
    pub fn confirm(mut self, title: impl Into<String>) -> Self {
        self.confirm = Some(ConfirmDialog {
            title: title.into(),
            message: None,
            variant: DialogVariant::Default,
        });
        self
    }

    /// Add a danger confirmation dialog.
    pub fn confirm_danger(mut self, title: impl Into<String>) -> Self {
        self.confirm = Some(ConfirmDialog {
            title: title.into(),
            message: None,
            variant: DialogVariant::Danger,
        });
        self
    }

    /// Set the success outcome.
    pub fn on_success(mut self, outcome: ActionOutcome) -> Self {
        self.on_success = Some(outcome);
        self
    }

    /// Set the error outcome.
    pub fn on_error(mut self, outcome: ActionOutcome) -> Self {
        self.on_error = Some(outcome);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_action_serializes() {
        let action = Action {
            handler: "users.store".to_string(),
            url: None,
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
        };
        let json = serde_json::to_value(&action).unwrap();
        assert_eq!(json["handler"], "users.store");
        assert_eq!(json["method"], "POST");
        assert!(json.get("confirm").is_none());
        assert!(json.get("on_success").is_none());
    }

    #[test]
    fn action_with_confirm_dialog() {
        let action = Action {
            handler: "users.destroy".to_string(),
            url: None,
            method: HttpMethod::Delete,
            confirm: Some(ConfirmDialog {
                title: "Delete user?".to_string(),
                message: Some("This cannot be undone.".to_string()),
                variant: DialogVariant::Danger,
            }),
            on_success: Some(ActionOutcome::Redirect {
                url: "/users".to_string(),
            }),
            on_error: Some(ActionOutcome::ShowErrors),
        };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, action);
    }

    #[test]
    fn action_outcome_variants_serialize() {
        let redirect = ActionOutcome::Redirect {
            url: "/dashboard".to_string(),
        };
        let json = serde_json::to_value(&redirect).unwrap();
        assert_eq!(json["type"], "redirect");
        assert_eq!(json["url"], "/dashboard");

        let show_errors = ActionOutcome::ShowErrors;
        let json = serde_json::to_value(&show_errors).unwrap();
        assert_eq!(json["type"], "show_errors");

        let refresh = ActionOutcome::Refresh;
        let json = serde_json::to_value(&refresh).unwrap();
        assert_eq!(json["type"], "refresh");

        let notify = ActionOutcome::Notify {
            message: "Saved!".to_string(),
            variant: NotifyVariant::Success,
        };
        let json = serde_json::to_value(&notify).unwrap();
        assert_eq!(json["type"], "notify");
        assert_eq!(json["message"], "Saved!");
        assert_eq!(json["variant"], "success");
    }

    #[test]
    fn http_method_defaults_to_post() {
        let json = r#"{"handler": "posts.store"}"#;
        let action: Action = serde_json::from_str(json).unwrap();
        assert_eq!(action.method, HttpMethod::Post);
    }

    #[test]
    fn dialog_variant_defaults_to_default() {
        let json = r#"{"title": "Confirm?"}"#;
        let dialog: ConfirmDialog = serde_json::from_str(json).unwrap();
        assert_eq!(dialog.variant, DialogVariant::Default);
    }

    #[test]
    fn action_without_url_omits_url_field() {
        let action = Action {
            handler: "users.index".to_string(),
            url: None,
            method: HttpMethod::Get,
            confirm: None,
            on_success: None,
            on_error: None,
        };
        let json = serde_json::to_value(&action).unwrap();
        assert!(json.get("url").is_none(), "url should be omitted when None");
    }

    #[test]
    fn action_with_url_includes_url_field() {
        let action = Action {
            handler: "users.store".to_string(),
            url: Some("/users".to_string()),
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
        };
        let json = serde_json::to_value(&action).unwrap();
        assert_eq!(json["url"], "/users");
    }

    #[test]
    fn action_url_round_trips() {
        let action = Action {
            handler: "users.show".to_string(),
            url: Some("/users/42".to_string()),
            method: HttpMethod::Get,
            confirm: None,
            on_success: None,
            on_error: None,
        };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.url, Some("/users/42".to_string()));
        assert_eq!(parsed, action);
    }

    // ── Builder method tests ──────────────────────────────────────────

    #[test]
    fn builder_new_creates_post_action() {
        let action = Action::new("users.store");
        assert_eq!(action.handler, "users.store");
        assert_eq!(action.method, HttpMethod::Post);
        assert_eq!(action.url, None);
        assert_eq!(action.confirm, None);
        assert_eq!(action.on_success, None);
        assert_eq!(action.on_error, None);
    }

    #[test]
    fn builder_get_creates_get_action() {
        let action = Action::get("users.index");
        assert_eq!(action.handler, "users.index");
        assert_eq!(action.method, HttpMethod::Get);
    }

    #[test]
    fn builder_delete_creates_delete_action() {
        let action = Action::delete("users.destroy");
        assert_eq!(action.handler, "users.destroy");
        assert_eq!(action.method, HttpMethod::Delete);
    }

    #[test]
    fn builder_confirm_adds_default_dialog() {
        let action = Action::new("users.store").confirm("Save changes?");
        let dialog = action.confirm.unwrap();
        assert_eq!(dialog.title, "Save changes?");
        assert_eq!(dialog.variant, DialogVariant::Default);
        assert_eq!(dialog.message, None);
    }

    #[test]
    fn builder_confirm_danger_adds_danger_dialog() {
        let action = Action::delete("users.destroy").confirm_danger("Delete user?");
        let dialog = action.confirm.unwrap();
        assert_eq!(dialog.title, "Delete user?");
        assert_eq!(dialog.variant, DialogVariant::Danger);
    }

    #[test]
    fn builder_on_success_sets_outcome() {
        let action = Action::new("users.store").on_success(ActionOutcome::Refresh);
        assert_eq!(action.on_success, Some(ActionOutcome::Refresh));
    }

    #[test]
    fn builder_on_error_sets_outcome() {
        let action = Action::new("users.store").on_error(ActionOutcome::ShowErrors);
        assert_eq!(action.on_error, Some(ActionOutcome::ShowErrors));
    }

    #[test]
    fn builder_chain_produces_expected_json() {
        let action = Action::delete("users.destroy")
            .confirm_danger("Delete user?")
            .on_success(ActionOutcome::Refresh);

        let json = serde_json::to_value(&action).unwrap();
        assert_eq!(json["handler"], "users.destroy");
        assert_eq!(json["method"], "DELETE");
        assert_eq!(json["confirm"]["title"], "Delete user?");
        assert_eq!(json["confirm"]["variant"], "danger");
        assert_eq!(json["on_success"]["type"], "refresh");
    }

    #[test]
    fn builder_method_overrides() {
        let action = Action::new("users.update").method(HttpMethod::Put);
        assert_eq!(action.method, HttpMethod::Put);
    }
}
