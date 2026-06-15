//! Comment endpoints: store (required auth), index (optional auth), destroy
//! (required auth, author-only).
//!
//! `store`/`destroy` take the author/actor id only from the verified `UserId`
//! (T-230-17): `destroy` asserts `comment.author_id == UserId` and returns 403
//! otherwise. `index` is viewer-relative for each comment author's `following`.
//! All queries are parameterized through SeaORM — no raw SQL.

use ferro::serde_json::json;
use ferro::{handler, HttpResponse, Request, Response, DB};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use crate::dto::requests::CreateCommentEnvelope;
use crate::dto::responses::{CommentDto, ProfileDto};
use crate::dto::{error_envelope, validation_error_envelope};
use crate::middleware::UserId;
use crate::models::{article, comment, follow, user};

/// Read the required viewer id, or a 401 token envelope (required-auth routes).
fn require_viewer(req: &Request) -> Result<i64, HttpResponse> {
    req.get::<UserId>()
        .map(|u| u.0)
        .ok_or_else(|| error_envelope(401, "token", &["is missing"]))
}

/// Find an article by slug, or 404.
async fn find_by_slug(
    db: &DatabaseConnection,
    slug: &str,
) -> Result<article::Model, HttpResponse> {
    article::Entity::find()
        .filter(article::Column::Slug.eq(slug))
        .one(db)
        .await
        .map_err(|e| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() })
        .and_then(|m| m.ok_or_else(|| error_envelope(404, "article", &["not found"])))
}

/// Build the comment envelope inner object for `c`, authored by `author`, as seen
/// by `viewer` (drives the author's `following` flag).
async fn to_comment_dto(
    db: &DatabaseConnection,
    c: comment::Model,
    author: user::Model,
    viewer: Option<i64>,
) -> Result<CommentDto, HttpResponse> {
    let dberr =
        |e: sea_orm::DbErr| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() };
    let following = match viewer {
        Some(uid) => {
            follow::Entity::find()
                .filter(follow::Column::FollowerId.eq(uid as i32))
                .filter(follow::Column::FollowedId.eq(author.id))
                .count(db)
                .await
                .map_err(dberr)?
                > 0
        }
        None => false,
    };
    Ok(CommentDto {
        id: c.id as i64,
        created_at: c.created_at.to_rfc3339(),
        updated_at: c.updated_at.to_rfc3339(),
        body: c.body,
        author: ProfileDto {
            username: author.username,
            bio: author.bio,
            image: author.image,
            following,
        },
    })
}

/// POST /api/articles/{slug}/comments — add a comment (required auth).
#[handler]
pub async fn store(req: Request) -> Response {
    let uid = require_viewer(&req)?;
    let slug = req.param("slug")?.to_string();
    let env = req.input::<CreateCommentEnvelope>().await?;
    let body = env.comment.body;
    if body.trim().is_empty() {
        return Err(validation_error_envelope("body", &["can't be blank"]));
    }

    let db = DB::get()?;
    let art = find_by_slug(&db, &slug).await?;

    let created = comment::ActiveModel {
        body: Set(body),
        article_id: Set(art.id),
        author_id: Set(uid as i32),
        ..Default::default()
    }
    .insert(&*db)
    .await
    .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    let author = user::Entity::find_by_id(uid as i32)
        .one(&*db)
        .await
        .map_err(|e| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() })?
        .ok_or_else(|| error_envelope(404, "author", &["not found"]))?;

    let dto = to_comment_dto(&db, created, author, Some(uid)).await?;
    Ok(HttpResponse::json(json!({ "comment": dto })))
}

/// GET /api/articles/{slug}/comments — list comments (optional auth), oldest first.
#[handler]
pub async fn index(req: Request) -> Response {
    let viewer = req.get::<UserId>().map(|u| u.0);
    let slug = req.param("slug")?.to_string();
    let db = DB::get()?;
    let art = find_by_slug(&db, &slug).await?;

    let comments = comment::Entity::find()
        .filter(comment::Column::ArticleId.eq(art.id))
        .order_by_asc(comment::Column::CreatedAt)
        .all(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    let mut dtos = Vec::with_capacity(comments.len());
    for c in comments {
        let author = user::Entity::find_by_id(c.author_id)
            .one(&*db)
            .await
            .map_err(|e| -> HttpResponse {
                ferro::FrameworkError::database(e.to_string()).into()
            })?
            .ok_or_else(|| error_envelope(404, "author", &["not found"]))?;
        dtos.push(to_comment_dto(&db, c, author, viewer).await?);
    }

    Ok(HttpResponse::json(json!({ "comments": dtos })))
}

/// DELETE /api/articles/{slug}/comments/{id} — delete a comment (required auth,
/// author-only). Asserts `comment.author_id == uid` (403 otherwise; T-230-17).
#[handler]
pub async fn destroy(req: Request) -> Response {
    let uid = require_viewer(&req)?;
    let id: i64 = req.param_as("id")?;
    let db = DB::get()?;

    let c = comment::Entity::find_by_id(id as i32)
        .one(&*db)
        .await
        .map_err(|e| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() })?
        .ok_or_else(|| error_envelope(404, "comment", &["not found"]))?;

    if c.author_id != uid as i32 {
        return Err(error_envelope(403, "comment", &["forbidden"]));
    }

    comment::Entity::delete_by_id(c.id)
        .exec(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    Ok(HttpResponse::new().status(200))
}
