//! Profile endpoints: show (optional auth, viewer-relative `following` flag) and
//! follow/unfollow (required auth, mutating the `follows` junction directly).
//!
//! `following` is computed by a direct junction query (RESEARCH §2,
//! `follower_id = viewer AND followed_id = target`), never via a SeaORM relation.
//! `follower_id` is taken only from the verified `UserId` (T-230-15), and the
//! composite PK on `follows` makes a duplicate follow a no-op (T-230-16).

use ferro::serde_json::json;
use ferro::{handler, HttpResponse, Request, Response, DB};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};

use crate::dto::error_envelope;
use crate::dto::responses::ProfileDto;
use crate::middleware::UserId;
use crate::models::{follow, user};

/// Read the required viewer id, or a 401 token envelope (required-auth routes).
fn require_viewer(req: &Request) -> Result<i64, HttpResponse> {
    req.get::<UserId>()
        .map(|u| u.0)
        .ok_or_else(|| error_envelope(401, "token", &["is missing"]))
}

/// Load the target user by username, or a 404 envelope.
async fn find_user(db: &DatabaseConnection, username: &str) -> Result<user::Model, HttpResponse> {
    user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(|e| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() })
        .and_then(|m| m.ok_or_else(|| error_envelope(404, "profile", &["not found"])))
}

/// Does `viewer` follow `target_id`? Direct junction query; `false` for a guest.
async fn is_following(
    db: &DatabaseConnection,
    viewer: Option<i64>,
    target_id: i32,
) -> Result<bool, HttpResponse> {
    let dberr = |e: sea_orm::DbErr| -> HttpResponse {
        ferro::FrameworkError::database(e.to_string()).into()
    };
    match viewer {
        Some(uid) => Ok(follow::Entity::find()
            .filter(follow::Column::FollowerId.eq(uid as i32))
            .filter(follow::Column::FollowedId.eq(target_id))
            .count(db)
            .await
            .map_err(dberr)?
            > 0),
        None => Ok(false),
    }
}

/// Build the profile envelope for `target` as seen by `viewer`.
async fn profile_response(
    db: &DatabaseConnection,
    target: user::Model,
    viewer: Option<i64>,
) -> Result<HttpResponse, HttpResponse> {
    let following = is_following(db, viewer, target.id).await?;
    Ok(HttpResponse::json(json!({
        "profile": ProfileDto {
            username: target.username,
            bio: target.bio,
            image: target.image,
            following,
        }
    })))
}

/// GET /api/profiles/{username} — fetch a profile (optional auth).
#[handler]
pub async fn show(req: Request) -> Response {
    let viewer = req.get::<UserId>().map(|u| u.0);
    let username = req.param("username")?.to_string();
    let db = DB::get()?;
    let target = find_user(&db, &username).await?;
    profile_response(&db, target, viewer).await
}

/// POST /api/profiles/{username}/follow — follow a user (required auth).
///
/// Inserts a `follows` row (follower_id = viewer, followed_id = target). The
/// composite PK makes a repeat follow a no-op, so a unique-violation insert error
/// is swallowed (idempotent). Returns the profile with `following = true`.
#[handler]
pub async fn follow(req: Request) -> Response {
    let viewer = require_viewer(&req)?;
    let username = req.param("username")?.to_string();
    let db = DB::get()?;
    let target = find_user(&db, &username).await?;

    // Idempotent: ignore the unique-violation on a duplicate (follows composite PK).
    let _ = (follow::ActiveModel {
        follower_id: Set(viewer as i32),
        followed_id: Set(target.id),
    })
    .insert(&*db)
    .await;

    profile_response(&db, target, Some(viewer)).await
}

/// DELETE /api/profiles/{username}/follow — unfollow a user (required auth).
///
/// Deletes the `follows` row if present (a no-op when absent). Returns the
/// profile with `following = false`.
#[handler]
pub async fn unfollow(req: Request) -> Response {
    let viewer = require_viewer(&req)?;
    let username = req.param("username")?.to_string();
    let db = DB::get()?;
    let target = find_user(&db, &username).await?;

    follow::Entity::delete_by_id((viewer as i32, target.id))
        .exec(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    profile_response(&db, target, Some(viewer)).await
}
