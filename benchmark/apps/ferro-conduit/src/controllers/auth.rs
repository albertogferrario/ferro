//! Auth endpoints: register, login, current user, update user.
//!
//! JWT is stateless: handlers mint a token and return it; they never call
//! `Auth::login()` (session — RESEARCH Anti-Pattern line 437). The protected
//! handlers read `req.get::<UserId>()` populated by `JwtAuthMiddleware`
//! (never `AuthUser<T>`, which is session-bound — Pitfall 1).

use ferro::serde_json::json;
use ferro::{handler, HttpResponse, Request, Response, DB};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::dto::requests::{LoginEnvelope, RegisterEnvelope, UpdateUserEnvelope};
use crate::dto::responses::UserDto;
use crate::dto::{error_envelope, validation_error_envelope};
use crate::middleware::UserId;
use crate::models::user;

/// Mint a fresh token for a user model and build the `{"user":{...}}` envelope.
fn user_envelope(model: &user::Model) -> HttpResponse {
    let token = crate::jwt::mint_token(model.id as i64, &model.email, &crate::jwt::jwt_secret());
    HttpResponse::json(json!({
        "user": UserDto {
            email: model.email.clone(),
            token,
            username: model.username.clone(),
            bio: model.bio.clone(),
            image: model.image.clone(),
        }
    }))
}

/// `true` if `s` is None or blank (Conduit "can't be blank").
fn blank(s: &Option<String>) -> bool {
    s.as_deref().map(str::trim).unwrap_or("").is_empty()
}

/// POST /api/users — register a new user, return user envelope + JWT.
#[handler]
pub async fn register(req: Request) -> Response {
    let env = req.input::<RegisterEnvelope>().await?;
    let r = env.user;

    // Validation envelopes (422) match the "Error Cases - Auth" assertions.
    if blank(&r.username) {
        return Err(validation_error_envelope("username", &["can't be blank"]));
    }
    if blank(&r.email) {
        return Err(validation_error_envelope("email", &["can't be blank"]));
    }
    if blank(&r.password) {
        return Err(validation_error_envelope("password", &["can't be blank"]));
    }
    let username = r.username.unwrap();
    let email = r.email.unwrap();
    let password = r.password.unwrap();

    let db = DB::get()?;

    // Uniqueness → 409 "has already been taken".
    if user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
        .is_some()
    {
        return Err(error_envelope(409, "email", &["has already been taken"]));
    }
    if user::Entity::find()
        .filter(user::Column::Username.eq(&username))
        .one(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
        .is_some()
    {
        return Err(error_envelope(409, "username", &["has already been taken"]));
    }

    let hashed = ferro::hashing::hash(&password)?;
    let active = user::ActiveModel {
        email: Set(email),
        username: Set(username),
        bio: Set(None),
        image: Set(None),
        password: Set(hashed),
        ..Default::default()
    };
    let model = active
        .insert(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    Ok(user_envelope(&model).status(201))
}

/// POST /api/users/login — verify credentials, return user envelope + JWT.
#[handler]
pub async fn login(req: Request) -> Response {
    let env = req.input::<LoginEnvelope>().await?;
    let r = env.user;

    if blank(&r.email) {
        return Err(validation_error_envelope("email", &["can't be blank"]));
    }
    if blank(&r.password) {
        return Err(validation_error_envelope("password", &["can't be blank"]));
    }
    let email = r.email.unwrap();
    let password = r.password.unwrap();

    let db = DB::get()?;
    let model = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    match model {
        Some(m) if m.verify_password(&password) => Ok(user_envelope(&m)),
        // No user-enumeration: same 401 for unknown email and wrong password.
        _ => Err(error_envelope(401, "credentials", &["invalid"])),
    }
}

/// GET /api/user — current user (re-mints token; Conduit returns token here too).
#[handler]
pub async fn current_user(req: Request) -> Response {
    let uid = req
        .get::<UserId>()
        .copied()
        .ok_or_else(|| error_envelope(401, "token", &["is missing"]))?;

    let db = DB::get()?;
    let model = user::Entity::find_by_id(uid.0 as i32)
        .one(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
        .ok_or_else(|| error_envelope(401, "token", &["is missing"]))?;

    Ok(user_envelope(&model))
}

/// PUT /api/user — apply present fields, return user envelope + JWT.
#[handler]
pub async fn update_user(req: Request) -> Response {
    let uid = req
        .get::<UserId>()
        .copied()
        .ok_or_else(|| error_envelope(401, "token", &["is missing"]))?;

    let env = req.input::<UpdateUserEnvelope>().await?;
    let u = env.user;

    let db = DB::get()?;
    let model = user::Entity::find_by_id(uid.0 as i32)
        .one(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
        .ok_or_else(|| error_envelope(401, "token", &["is missing"]))?;

    let mut active: user::ActiveModel = model.into();
    if let Some(email) = u.email {
        active.email = Set(email);
    }
    if let Some(username) = u.username {
        active.username = Set(username);
    }
    if let Some(bio) = u.bio {
        active.bio = Set(Some(bio));
    }
    if let Some(image) = u.image {
        active.image = Set(Some(image));
    }
    if let Some(password) = u.password {
        active.password = Set(ferro::hashing::hash(&password)?);
    }
    let saved = active
        .update(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    Ok(user_envelope(&saved))
}
