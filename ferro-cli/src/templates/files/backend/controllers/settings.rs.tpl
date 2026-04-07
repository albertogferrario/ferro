//! Settings controller

use ferro::{
    serde_json, Auth, Inertia, InertiaProps, Request, Response, SavedInertiaContext,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// Show Settings
// ============================================================================

#[derive(Serialize, Clone)]
pub struct UserSettings {
    pub email_notifications: bool,
    pub marketing_emails: bool,
    pub theme: String,
    pub language: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            email_notifications: true,
            marketing_emails: false,
            theme: "system".to_string(),
            language: "en".to_string(),
        }
    }
}

#[derive(InertiaProps)]
pub struct SettingsProps {
    pub settings: UserSettings,
    pub errors: Option<serde_json::Value>,
}

pub async fn show(req: Request) -> Response {
    // In a real application, you would load settings from database
    // For now, we use defaults
    let settings = UserSettings::default();

    Inertia::render(
        &req,
        "Settings",
        SettingsProps {
            settings,
            errors: None,
        },
    )
}

// ============================================================================
// Update Settings
// ============================================================================

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub email_notifications: bool,
    pub marketing_emails: bool,
    pub theme: String,
    pub language: String,
}

pub async fn update(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let _user_id = Auth::user_id().ok_or_else(|| ferro::FrameworkError::AuthenticationRequired)?;
    let form: UpdateSettingsRequest = req.input().await?;

    // In a real application, you would save settings to database
    // For now, we just return success
    tracing::info!(
        "Settings updated: notifications={}, marketing={}, theme={}, language={}",
        form.email_notifications,
        form.marketing_emails,
        form.theme,
        form.language
    );

    let settings = UserSettings {
        email_notifications: form.email_notifications,
        marketing_emails: form.marketing_emails,
        theme: form.theme,
        language: form.language,
    };

    Inertia::render_ctx(
        &ctx,
        "Settings",
        SettingsProps {
            settings,
            errors: None,
        },
    )
}
