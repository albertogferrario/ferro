//! Account — edit your public profile.

use ferro::database::ModelMut;
use ferro::serde_json::json;
use ferro::{handler, Auth, Inertia, Redirect, Request, Response};
use sea_orm::Set;
use serde::Deserialize;

use crate::models::profile::{Entity as ProfileEntity, Profile};

#[derive(Deserialize)]
struct AccountInput {
    display_name: String,
    status: String,
}

/// GET /account
#[handler]
pub async fn show(req: Request) -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    let profile = Profile::find_by_user(uid as i32).await?;
    let (display_name, status) = profile
        .map(|p| (p.display_name, p.status))
        .unwrap_or_default();

    Inertia::render(
        &req,
        "Account",
        json!({ "display_name": display_name, "status": status }),
    )
}

/// POST /account
#[handler]
pub async fn update(req: Request) -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    let input: AccountInput = req.input().await?;

    if let Some(profile) = Profile::find_by_user(uid as i32).await? {
        let mut active: crate::models::profile::ActiveModel = profile.into();
        active.display_name = Set(input.display_name.trim().to_string());
        active.status = Set(input.status.trim().to_string());
        active.updated_at = Set(crate::models::now());
        ProfileEntity::update_one(active).await?;
    }

    Redirect::to("/account").into()
}
