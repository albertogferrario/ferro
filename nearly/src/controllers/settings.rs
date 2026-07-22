//! Settings — visibility toggle and about / credits.

use ferro::database::ModelMut;
use ferro::serde_json::json;
use ferro::{handler, Auth, Inertia, Redirect, Request, Response};
use sea_orm::Set;
use serde::Deserialize;

use crate::models::profile::{Entity as ProfileEntity, Profile};

#[derive(Deserialize)]
struct SettingsInput {
    /// The React toggle always submits an explicit boolean.
    visible: bool,
}

/// GET /settings
#[handler]
pub async fn show(req: Request) -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    let visible = Profile::find_by_user(uid as i32)
        .await?
        .map(|p| p.visible)
        .unwrap_or(true);

    Inertia::render(&req, "Settings", json!({ "visible": visible }))
}

/// POST /settings — update visibility.
#[handler]
pub async fn update(req: Request) -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    let input: SettingsInput = req.input().await?;

    if let Some(profile) = Profile::find_by_user(uid as i32).await? {
        let mut active: crate::models::profile::ActiveModel = profile.into();
        active.visible = Set(input.visible);
        active.updated_at = Set(crate::models::now());
        ProfileEntity::update_one(active).await?;
    }

    Redirect::to("/settings").into()
}
