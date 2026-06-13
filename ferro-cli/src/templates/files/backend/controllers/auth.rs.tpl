//! Authentication controller

use ferro::{
    database::{Model as DatabaseModel, ModelMut},
    hashing,
    serde_json, Auth, Inertia, InertiaProps, Request, Response, SavedInertiaContext, Validate,
};
use sea_orm::Set;
use serde::Deserialize;

use crate::models::user::{self, User};

// ============================================================================
// Login
// ============================================================================

#[derive(InertiaProps)]
pub struct LoginProps {
    pub errors: Option<serde_json::Value>,
}

pub async fn show_login(req: Request) -> Response {
    Inertia::render(&req, "auth/Login", LoginProps { errors: None })
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    #[serde(default)]
    pub remember: bool,
}

pub async fn login(req: Request) -> Response {
    // Save Inertia context before consuming request
    let ctx = SavedInertiaContext::from(&req);

    let form: LoginRequest = req.input().await?;

    // Validate the form
    if let Err(errors) = form.validate() {
        return Inertia::render_ctx(
            &ctx,
            "auth/Login",
            LoginProps {
                errors: Some(serde_json::json!(errors)),
            },
        )
        .map(|r| r.status(422));
    }

    // Find user by email
    let user = match User::find_by_email(&form.email).await? {
        Some(u) => u,
        None => {
            return Inertia::render_ctx(
                &ctx,
                "auth/Login",
                LoginProps {
                    errors: Some(serde_json::json!({
                        "email": ["These credentials do not match our records."]
                    })),
                },
            )
            .map(|r| r.status(422));
        }
    };

    // Verify password
    if !user.verify_password(&form.password)? {
        return Inertia::render_ctx(
            &ctx,
            "auth/Login",
            LoginProps {
                errors: Some(serde_json::json!({
                    "email": ["These credentials do not match our records."]
                })),
            },
        )
        .map(|r| r.status(422));
    }

    // Log in the user
    Auth::login(user.id);

    // Handle remember me
    if form.remember {
        // Generate and store remember token
        let token = ferro::session::generate_session_id();
        user.update_remember_token(Some(token)).await?;
    }

    Inertia::redirect_ctx(&ctx, "/dashboard")
}

// ============================================================================
// Registration
// ============================================================================

#[derive(InertiaProps)]
pub struct RegisterProps {
    pub errors: Option<serde_json::Value>,
}

pub async fn show_register(req: Request) -> Response {
    Inertia::render(&req, "auth/Register", RegisterProps { errors: None })
}

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, message = "Name must be at least 2 characters"))]
    pub name: String,
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub password_confirmation: String,
}

pub async fn register(req: Request) -> Response {
    // Save Inertia context before consuming request
    let ctx = SavedInertiaContext::from(&req);

    let form: RegisterRequest = req.input().await?;

    // Validate the form
    if let Err(errors) = form.validate() {
        return Inertia::render_ctx(
            &ctx,
            "auth/Register",
            RegisterProps {
                errors: Some(serde_json::json!(errors)),
            },
        )
        .map(|r| r.status(422));
    }

    // Check password confirmation
    if form.password != form.password_confirmation {
        return Inertia::render_ctx(
            &ctx,
            "auth/Register",
            RegisterProps {
                errors: Some(serde_json::json!({
                    "password_confirmation": ["Passwords do not match."]
                })),
            },
        )
        .map(|r| r.status(422));
    }

    // Check if email already exists
    if User::find_by_email(&form.email).await?.is_some() {
        return Inertia::render_ctx(
            &ctx,
            "auth/Register",
            RegisterProps {
                errors: Some(serde_json::json!({
                    "email": ["This email is already registered."]
                })),
            },
        )
        .map(|r| r.status(422));
    }

    // Create user
    let user = User::create(&form.name, &form.email, &form.password).await?;

    // Log in the new user
    Auth::login(user.id);

    Inertia::redirect_ctx(&ctx, "/dashboard")
}

// ============================================================================
// Logout
// ============================================================================

pub async fn logout(req: Request) -> Response {
    Auth::logout();
    Inertia::redirect(&req, "/")
}

// ============================================================================
// Forgot Password
// ============================================================================

#[derive(InertiaProps)]
pub struct ForgotPasswordProps {
    pub errors: Option<serde_json::Value>,
    pub status: Option<String>,
}

pub async fn show_forgot_password(req: Request) -> Response {
    Inertia::render(
        &req,
        "auth/ForgotPassword",
        ForgotPasswordProps {
            errors: None,
            status: None,
        },
    )
}

#[derive(Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,
}

pub async fn forgot_password(req: Request) -> Response {
    use crate::models::password_reset_tokens::PasswordResetToken;

    let ctx = SavedInertiaContext::from(&req);
    let form: ForgotPasswordRequest = req.input().await?;

    // Validate the form
    if let Err(errors) = form.validate() {
        return Inertia::render_ctx(
            &ctx,
            "auth/ForgotPassword",
            ForgotPasswordProps {
                errors: Some(serde_json::json!(errors)),
                status: None,
            },
        )
        .map(|r| r.status(422));
    }

    // Check if user exists
    if let Some(_user) = User::find_by_email(&form.email).await? {
        // Generate token
        let token = PasswordResetToken::create_for_email(&form.email).await?;

        // TODO: Send password reset email with link
        // For now, log the reset link for development
        tracing::info!(
            "Password reset link: /reset-password?email={}&token={}",
            form.email,
            token
        );
    }

    // Always show success to prevent email enumeration
    Inertia::render_ctx(
        &ctx,
        "auth/ForgotPassword",
        ForgotPasswordProps {
            errors: None,
            status: Some("If an account exists with that email, you will receive a password reset link.".to_string()),
        },
    )
}

// ============================================================================
// Reset Password
// ============================================================================

#[derive(InertiaProps)]
pub struct ResetPasswordProps {
    pub email: String,
    pub token: String,
    pub errors: Option<serde_json::Value>,
}

pub async fn show_reset_password(req: Request) -> Response {
    let email = req.query("email").unwrap_or_default();
    let token = req.query("token").unwrap_or_default();

    Inertia::render(
        &req,
        "auth/ResetPassword",
        ResetPasswordProps {
            email,
            token,
            errors: None,
        },
    )
}

#[derive(Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub password_confirmation: String,
}

pub async fn reset_password(req: Request) -> Response {
    use crate::models::password_reset_tokens::PasswordResetToken;

    let ctx = SavedInertiaContext::from(&req);
    let form: ResetPasswordRequest = req.input().await?;

    // Validate the form
    if let Err(errors) = form.validate() {
        return Inertia::render_ctx(
            &ctx,
            "auth/ResetPassword",
            ResetPasswordProps {
                email: form.email,
                token: form.token,
                errors: Some(serde_json::json!(errors)),
            },
        )
        .map(|r| r.status(422));
    }

    // Check password confirmation
    if form.password != form.password_confirmation {
        return Inertia::render_ctx(
            &ctx,
            "auth/ResetPassword",
            ResetPasswordProps {
                email: form.email,
                token: form.token,
                errors: Some(serde_json::json!({
                    "password_confirmation": ["Passwords do not match."]
                })),
            },
        )
        .map(|r| r.status(422));
    }

    // Verify the token
    if !PasswordResetToken::verify(&form.email, &form.token).await? {
        return Inertia::render_ctx(
            &ctx,
            "auth/ResetPassword",
            ResetPasswordProps {
                email: form.email,
                token: form.token,
                errors: Some(serde_json::json!({
                    "token": ["This password reset link is invalid or has expired."]
                })),
            },
        )
        .map(|r| r.status(422));
    }

    // Find user and update password
    if let Some(user) = User::find_by_email(&form.email).await? {
        let hashed = hashing::hash(&form.password)
            .map_err(|e| ferro::FrameworkError::internal(e.to_string()))?;
        let mut active: user::ActiveModel = user.into();
        active.password = Set(hashed);
        user::Entity::update_one(active).await?;
        PasswordResetToken::delete_for_email(&form.email).await?;
    }

    Inertia::redirect_ctx(&ctx, "/login")
}
