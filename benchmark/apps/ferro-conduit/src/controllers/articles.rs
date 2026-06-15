//! Article endpoints: index (list + filters + pagination), store, show, update,
//! destroy, plus a `feed_placeholder` (Plan 06 replaces it with the real feed).
//!
//! Reads use optional auth (`req.get::<UserId>()` may be None → guest); writes
//! require auth and enforce author-ownership before mutating (T-230-12).
//! All filter params are parameterized through SeaORM — no raw SQL (T-230-13).

use ferro::serde_json::json;
use ferro::{handler, HttpResponse, Request, Response, DB};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use crate::dto::requests::{CreateArticleEnvelope, UpdateArticleEnvelope};
use crate::dto::responses::{ArticleDto, ProfileDto};
use crate::dto::{error_envelope, validation_error_envelope};
use crate::middleware::UserId;
use crate::models::{article, article_tag, favorite, follow, tag, user};

/// Max page size for the article list (T-230-14: bound DoS via unbounded limit).
const MAX_LIMIT: u64 = 100;

/// Read the optional viewer id (guest when absent on optional-auth routes).
fn viewer(req: &Request) -> Option<i64> {
    req.get::<UserId>().map(|u| u.0)
}

/// Read the required viewer id, or a 401 token envelope (required-auth routes).
fn require_viewer(req: &Request) -> Result<i64, HttpResponse> {
    req.get::<UserId>()
        .map(|u| u.0)
        .ok_or_else(|| error_envelope(401, "token", &["is missing"]))
}

/// `true` if `s` is None or blank (Conduit "can't be blank").
fn blank(s: &Option<String>) -> bool {
    s.as_deref().map(str::trim).unwrap_or("").is_empty()
}

/// Assemble the full Conduit article envelope for `article`, as seen by `viewer`.
///
/// Uses count queries (not per-field N+1 in a loop) for `favoritesCount`,
/// `favorited`, and `author.following`. `tagList` is a single junction→tag query.
async fn to_article_dto(
    db: &DatabaseConnection,
    a: article::Model,
    viewer: Option<i64>,
) -> Result<ArticleDto, HttpResponse> {
    let dberr = |e: sea_orm::DbErr| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() };

    // tagList: tag names linked via the article_tags junction.
    let tag_ids: Vec<i32> = article_tag::Entity::find()
        .filter(article_tag::Column::ArticleId.eq(a.id))
        .all(db)
        .await
        .map_err(dberr)?
        .into_iter()
        .map(|at| at.tag_id)
        .collect();
    let mut tag_list: Vec<String> = if tag_ids.is_empty() {
        Vec::new()
    } else {
        tag::Entity::find()
            .filter(tag::Column::Id.is_in(tag_ids))
            .all(db)
            .await
            .map_err(dberr)?
            .into_iter()
            .map(|t| t.name)
            .collect()
    };
    tag_list.sort();

    // favoritesCount: rows in favorites for this article.
    let favorites_count = favorite::Entity::find()
        .filter(favorite::Column::ArticleId.eq(a.id))
        .count(db)
        .await
        .map_err(dberr)? as i64;

    // favorited: does the viewer favorite this article?
    let favorited = match viewer {
        Some(uid) => {
            favorite::Entity::find()
                .filter(favorite::Column::ArticleId.eq(a.id))
                .filter(favorite::Column::UserId.eq(uid as i32))
                .count(db)
                .await
                .map_err(dberr)?
                > 0
        }
        None => false,
    };

    // author profile + following.
    let author = user::Entity::find_by_id(a.author_id)
        .one(db)
        .await
        .map_err(dberr)?
        .ok_or_else(|| error_envelope(404, "author", &["not found"]))?;
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

    Ok(ArticleDto {
        slug: a.slug,
        title: a.title,
        description: a.description,
        body: a.body,
        tag_list,
        created_at: a.created_at.to_rfc3339(),
        updated_at: a.updated_at.to_rfc3339(),
        favorited,
        favorites_count,
        author: ProfileDto {
            username: author.username,
            bio: author.bio,
            image: author.image,
            following,
        },
    })
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

/// Find-or-create a tag by name, returning its id.
async fn upsert_tag(db: &DatabaseConnection, name: &str) -> Result<i32, HttpResponse> {
    let dberr = |e: sea_orm::DbErr| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() };
    if let Some(existing) = tag::Entity::find()
        .filter(tag::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(dberr)?
    {
        return Ok(existing.id);
    }
    let created = tag::ActiveModel {
        name: Set(name.to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(dberr)?;
    Ok(created.id)
}

/// POST /api/articles — create an article (required auth), associating tags.
#[handler]
pub async fn store(req: Request) -> Response {
    let author_id = require_viewer(&req)?;
    let env = req.input::<CreateArticleEnvelope>().await?;
    let r = env.article;

    if blank(&Some(r.title.clone())) {
        return Err(validation_error_envelope("title", &["can't be blank"]));
    }
    if blank(&Some(r.description.clone())) {
        return Err(validation_error_envelope("description", &["can't be blank"]));
    }
    if blank(&Some(r.body.clone())) {
        return Err(validation_error_envelope("body", &["can't be blank"]));
    }

    let db = DB::get()?;

    // Insert with slug; retry on UNIQUE conflict with a fresh random suffix.
    let mut inserted: Option<article::Model> = None;
    for _ in 0..3 {
        let slug = article::generate_slug(&r.title);
        let active = article::ActiveModel {
            slug: Set(slug),
            title: Set(r.title.clone()),
            description: Set(r.description.clone()),
            body: Set(r.body.clone()),
            author_id: Set(author_id as i32),
            ..Default::default()
        };
        match active.insert(&*db).await {
            Ok(m) => {
                inserted = Some(m);
                break;
            }
            Err(_) => continue, // slug collision (or transient) → retry
        }
    }
    let model = inserted
        .ok_or_else(|| error_envelope(409, "slug", &["could not generate a unique slug"]))?;

    // Associate tags: find-or-create each, then link via the junction.
    if let Some(tags) = r.tag_list {
        for name in tags {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let tag_id = upsert_tag(&db, name).await?;
            // Ignore duplicate-link errors (same tag twice in the request).
            let _ = (article_tag::ActiveModel {
                article_id: Set(model.id),
                tag_id: Set(tag_id),
            })
            .insert(&*db)
            .await;
        }
    }

    let dto = to_article_dto(&db, model, Some(author_id)).await?;
    Ok(HttpResponse::json(json!({ "article": dto })).status(201))
}

/// GET /api/articles/{slug} — get a single article (optional auth).
#[handler]
pub async fn show(req: Request) -> Response {
    let viewer = viewer(&req);
    let slug = req.param("slug")?.to_string();
    let db = DB::get()?;
    let model = find_by_slug(&db, &slug).await?;
    let dto = to_article_dto(&db, model, viewer).await?;
    Ok(HttpResponse::json(json!({ "article": dto })))
}

/// PUT /api/articles/{slug} — update present fields (required auth, author-only).
#[handler]
pub async fn update(req: Request) -> Response {
    let viewer_id = require_viewer(&req)?;
    let slug = req.param("slug")?.to_string();
    let env = req.input::<UpdateArticleEnvelope>().await?;
    let u = env.article;

    let db = DB::get()?;
    let model = find_by_slug(&db, &slug).await?;
    if model.author_id != viewer_id as i32 {
        return Err(error_envelope(403, "article", &["forbidden"]));
    }

    let mut active: article::ActiveModel = model.into();
    if let Some(title) = u.title {
        active.title = Set(title);
    }
    if let Some(description) = u.description {
        active.description = Set(description);
    }
    if let Some(body) = u.body {
        active.body = Set(body);
    }
    let saved = active
        .update(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    let dto = to_article_dto(&db, saved, Some(viewer_id)).await?;
    Ok(HttpResponse::json(json!({ "article": dto })))
}

/// DELETE /api/articles/{slug} — delete an article (required auth, author-only).
///
/// FK ON DELETE CASCADE removes article_tags/favorites/comments rows.
#[handler]
pub async fn destroy(req: Request) -> Response {
    let viewer_id = require_viewer(&req)?;
    let slug = req.param("slug")?.to_string();
    let db = DB::get()?;
    let model = find_by_slug(&db, &slug).await?;
    if model.author_id != viewer_id as i32 {
        return Err(error_envelope(403, "article", &["forbidden"]));
    }
    article::Entity::delete_by_id(model.id)
        .exec(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;
    Ok(HttpResponse::json(json!({})))
}

/// GET /api/articles — list articles with tag/author/favorited filters,
/// limit/offset pagination, and a pre-pagination `articlesCount` (optional auth).
#[handler]
pub async fn index(req: Request) -> Response {
    let viewer = viewer(&req);
    let db = DB::get()?;
    let dberr = |e: sea_orm::DbErr| ferro::FrameworkError::database(e.to_string());

    let limit = req.query_as_or("limit", 20u64).min(MAX_LIMIT);
    let offset = req.query_as_or("offset", 0u64);

    // Build a base query, narrowing by each present filter via parameterized
    // `is_in` over article ids (M:N) or a direct column eq (author).
    let mut query = article::Entity::find();

    if let Some(tag_name) = req.query("tag") {
        let ids = article_ids_for_tag(&db, &tag_name).await?;
        query = query.filter(article::Column::Id.is_in(ids));
    }
    if let Some(author_name) = req.query("author") {
        match user::Entity::find()
            .filter(user::Column::Username.eq(&author_name))
            .one(&*db)
            .await
            .map_err(dberr)?
        {
            Some(u) => query = query.filter(article::Column::AuthorId.eq(u.id)),
            // Unknown author → empty result set.
            None => query = query.filter(article::Column::Id.eq(-1)),
        }
    }
    if let Some(fav_name) = req.query("favorited") {
        let ids = article_ids_favorited_by(&db, &fav_name).await?;
        query = query.filter(article::Column::Id.is_in(ids));
    }

    // articlesCount is the total over the filtered query, before limit/offset.
    let articles_count = query.clone().count(&*db).await.map_err(dberr)? as i64;

    let models = query
        .order_by_desc(article::Column::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(&*db)
        .await
        .map_err(dberr)?;

    let mut dtos = Vec::with_capacity(models.len());
    for m in models {
        dtos.push(to_article_dto(&db, m, viewer).await?);
    }

    Ok(HttpResponse::json(json!({
        "articles": dtos,
        "articlesCount": articles_count,
    })))
}

/// GET /api/articles/feed — placeholder until Plan 06 implements the real feed.
///
/// Returns the empty multiple-articles envelope so the route resolves and shape
/// assertions pass minimally. PLAN 06 MUST REPLACE THIS with the followed-author feed.
#[handler]
pub async fn feed_placeholder(req: Request) -> Response {
    // Required auth: a missing token is a 401, matching the real feed contract.
    let _ = require_viewer(&req)?;
    Ok(HttpResponse::json(json!({
        "articles": [],
        "articlesCount": 0,
    })))
}

/// Article ids tagged with `tag_name` (empty when the tag does not exist).
async fn article_ids_for_tag(
    db: &DatabaseConnection,
    tag_name: &str,
) -> Result<Vec<i32>, HttpResponse> {
    let dberr = |e: sea_orm::DbErr| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() };
    let Some(t) = tag::Entity::find()
        .filter(tag::Column::Name.eq(tag_name))
        .one(db)
        .await
        .map_err(dberr)?
    else {
        return Ok(Vec::new());
    };
    Ok(article_tag::Entity::find()
        .filter(article_tag::Column::TagId.eq(t.id))
        .all(db)
        .await
        .map_err(dberr)?
        .into_iter()
        .map(|at| at.article_id)
        .collect())
}

/// Article ids favorited by user `username` (empty when the user does not exist).
async fn article_ids_favorited_by(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Vec<i32>, HttpResponse> {
    let dberr = |e: sea_orm::DbErr| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() };
    let Some(u) = user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(dberr)?
    else {
        return Ok(Vec::new());
    };
    Ok(favorite::Entity::find()
        .filter(favorite::Column::UserId.eq(u.id))
        .all(db)
        .await
        .map_err(dberr)?
        .into_iter()
        .map(|f| f.article_id)
        .collect())
}
