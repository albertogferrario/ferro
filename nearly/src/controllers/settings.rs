//! Settings — visibility toggle and about / credits.

use ferro::database::ModelMut;
use ferro::serde_json::json;
use ferro::{handler, Auth, JsonUi, Redirect, Request, Response};
use sea_orm::Set;
use serde::Deserialize;

use crate::models::profile::{Entity as ProfileEntity, Profile};

#[derive(Deserialize)]
struct SettingsInput {
    /// A checkbox/switch submits its value only in some states; treat any of
    /// "true"/"on"/"1" as visible, everything else (including absent) as hidden.
    visible: Option<String>,
}

/// GET /settings
#[handler]
pub async fn show() -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    let visible = Profile::find_by_user(uid as i32)
        .await?
        .map(|p| p.visible)
        .unwrap_or(true);

    JsonUi::render_file("src/views/settings.json", json!({ "visible": visible }))
}

/// POST /settings — update visibility.
#[handler]
pub async fn update(req: Request) -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    let input: SettingsInput = req.form().await?;
    let visible = matches!(input.visible.as_deref(), Some("true" | "on" | "1"));

    if let Some(profile) = Profile::find_by_user(uid as i32).await? {
        let mut active: crate::models::profile::ActiveModel = profile.into();
        active.visible = Set(visible);
        active.updated_at = Set(crate::models::now());
        ProfileEntity::update_one(active).await?;
    }

    Redirect::to("/settings").into()
}
