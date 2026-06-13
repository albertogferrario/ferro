//! Profile controller

use ferro::{
    database::{Model as DatabaseModel, ModelMut},
    hashing,
    serde_json, Auth, Inertia, InertiaProps, Request, Response, SavedInertiaContext, Validate,
};
use sea_orm::Set;
use serde::Deserialize;

use crate::models::user::{self, User};

// ============================================================================
// Show Profile
// ============================================================================

#[derive(InertiaProps)]
pub struct ProfileProps {
    pub errors: Option<serde_json::Value>,
}

pub async fn show(req: Request) -> Response {
    Inertia::render(&req, "Profile", ProfileProps { errors: None })
}

// ============================================================================
// Update Profile
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 2, message = "Name must be at least 2 characters"))]
    pub name: String,
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,
}

pub async fn update(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let user_id = Auth::id().ok_or_else(|| ferro::FrameworkError::Unauthorized)?;
    let form: UpdateProfileRequest = req.input().await?;

    // Validate the form
    if let Err(errors) = form.validate() {
        return Inertia::render_ctx(
            &ctx,
            "Profile",
            ProfileProps {
                errors: Some(serde_json::json!(errors)),
            },
        )
        .map(|r| r.status(422));
    }

    // Get current user
    let current_user = user::Entity::find_by_pk(user_id).await?.ok_or_else(|| {
        ferro::FrameworkError::model_not_found("User")
    })?;

    // Check if email is already taken by another user
    if let Some(existing) = User::find_by_email(&form.email).await? {
        if existing.id != user_id {
            return Inertia::render_ctx(
                &ctx,
                "Profile",
                ProfileProps {
                    errors: Some(serde_json::json!({
                        "email": ["This email is already taken."]
                    })),
                },
            )
            .map(|r| r.status(422));
        }
    }

    // Update name and email
    let mut active: user::ActiveModel = current_user.into();
    active.name = Set(form.name.clone());
    active.email = Set(form.email.clone());
    user::Entity::update_one(active).await?;

    Inertia::redirect_ctx(&ctx, "/profile")
}

// ============================================================================
// Update Password
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct UpdatePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub password_confirmation: String,
}

pub async fn update_password(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let user_id = Auth::id().ok_or_else(|| ferro::FrameworkError::Unauthorized)?;
    let form: UpdatePasswordRequest = req.input().await?;

    // Validate the form
    if let Err(errors) = form.validate() {
        return Inertia::render_ctx(
            &ctx,
            "Profile",
            ProfileProps {
                errors: Some(serde_json::json!(errors)),
            },
        )
        .map(|r| r.status(422));
    }

    // Check password confirmation
    if form.password != form.password_confirmation {
        return Inertia::render_ctx(
            &ctx,
            "Profile",
            ProfileProps {
                errors: Some(serde_json::json!({
                    "password_confirmation": ["Passwords do not match."]
                })),
            },
        )
        .map(|r| r.status(422));
    }

    // Get current user
    let current_user = user::Entity::find_by_pk(user_id).await?.ok_or_else(|| {
        ferro::FrameworkError::model_not_found("User")
    })?;

    // Verify current password
    if !current_user.verify_password(&form.current_password)? {
        return Inertia::render_ctx(
            &ctx,
            "Profile",
            ProfileProps {
                errors: Some(serde_json::json!({
                    "current_password": ["Current password is incorrect."]
                })),
            },
        )
        .map(|r| r.status(422));
    }

    // Hash and update password
    let hashed = hashing::hash(&form.password)
        .map_err(|e| ferro::FrameworkError::internal(e.to_string()))?;
    let mut active: user::ActiveModel = current_user.into();
    active.password = Set(hashed);
    user::Entity::update_one(active).await?;

    Inertia::redirect_ctx(&ctx, "/profile")
}

// ============================================================================
// Delete Account
// ============================================================================

pub async fn destroy(req: Request) -> Response {
    let user_id = Auth::id().ok_or_else(|| ferro::FrameworkError::Unauthorized)?;

    // Get current user and delete
    let current_user = user::Entity::find_by_pk(user_id).await?.ok_or_else(|| {
        ferro::FrameworkError::model_not_found("User")
    })?;

    user::Entity::delete_by_pk(current_user.id).await?;

    // Logout
    Auth::logout();

    Inertia::redirect(&req, "/")
}
