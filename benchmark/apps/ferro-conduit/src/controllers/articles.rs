//! Article endpoints: index (list + filters + pagination), store, show, update,
//! destroy, plus a `feed_placeholder` (Plan 06 replaces it with the real feed).
//!
//! Reads use optional auth (`req.get::<UserId>()` may be None → guest); writes
//! require auth and enforce author-ownership before mutating (T-230-12).
//! All filter params are parameterized through SeaORM — no raw SQL (T-230-13).

use std::collections::{HashMap, HashSet};

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

/// Assemble article envelopes for a *page* of articles with a fixed number of
/// batched queries (no per-article N+1). For `n` articles this runs at most 5
/// queries total — tag links, tag names, favorites counts, viewer favorites,
/// author profiles, viewer follows — instead of ~6n. Output is byte-for-byte
/// identical to mapping `to_article_dto` over each article.
async fn to_article_dtos(
    db: &DatabaseConnection,
    articles: Vec<article::Model>,
    viewer: Option<i64>,
) -> Result<Vec<ArticleDto>, HttpResponse> {
    let dberr =
        |e: sea_orm::DbErr| -> HttpResponse { ferro::FrameworkError::database(e.to_string()).into() };

    if articles.is_empty() {
        return Ok(Vec::new());
    }

    let article_ids: Vec<i32> = articles.iter().map(|a| a.id).collect();
    let author_ids: Vec<i32> = articles.iter().map(|a| a.author_id).collect();

    // tagList: one junction query for all articles, then one query for the tag names.
    let links = article_tag::Entity::find()
        .filter(article_tag::Column::ArticleId.is_in(article_ids.clone()))
        .all(db)
        .await
        .map_err(dberr)?;
    let needed_tag_ids: HashSet<i32> = links.iter().map(|l| l.tag_id).collect();
    let tag_names: HashMap<i32, String> = if needed_tag_ids.is_empty() {
        HashMap::new()
    } else {
        tag::Entity::find()
            .filter(tag::Column::Id.is_in(needed_tag_ids.into_iter().collect::<Vec<_>>()))
            .all(db)
            .await
            .map_err(dberr)?
            .into_iter()
            .map(|t| (t.id, t.name))
            .collect()
    };
    let mut tags_by_article: HashMap<i32, Vec<String>> = HashMap::new();
    for l in links {
        if let Some(name) = tag_names.get(&l.tag_id) {
            tags_by_article
                .entry(l.article_id)
                .or_default()
                .push(name.clone());
        }
    }
    for list in tags_by_article.values_mut() {
        list.sort();
    }

    // favoritesCount: one grouped query over all article favorites.
    let mut favorites_count: HashMap<i32, i64> = HashMap::new();
    for f in favorite::Entity::find()
        .filter(favorite::Column::ArticleId.is_in(article_ids.clone()))
        .all(db)
        .await
        .map_err(dberr)?
    {
        *favorites_count.entry(f.article_id).or_default() += 1;
    }

    // favorited: the viewer's favorites among this page (single filtered query).
    let viewer_favorites: HashSet<i32> = match viewer {
        Some(uid) => favorite::Entity::find()
            .filter(favorite::Column::ArticleId.is_in(article_ids.clone()))
            .filter(favorite::Column::UserId.eq(uid as i32))
            .all(db)
            .await
            .map_err(dberr)?
            .into_iter()
            .map(|f| f.article_id)
            .collect(),
        None => HashSet::new(),
    };

    // author profiles: one query for all distinct authors.
    let distinct_authors: Vec<i32> = author_ids
        .iter()
        .copied()
        .collect::<HashSet<i32>>()
        .into_iter()
        .collect();
    let authors: HashMap<i32, user::Model> = user::Entity::find()
        .filter(user::Column::Id.is_in(distinct_authors.clone()))
        .all(db)
        .await
        .map_err(dberr)?
        .into_iter()
        .map(|u| (u.id, u))
        .collect();

    // following: the viewer's followed authors among this page (single filtered query).
    let viewer_following: HashSet<i32> = match viewer {
        Some(uid) => follow::Entity::find()
            .filter(follow::Column::FollowerId.eq(uid as i32))
            .filter(follow::Column::FollowedId.is_in(distinct_authors))
            .all(db)
            .await
            .map_err(dberr)?
            .into_iter()
            .map(|f| f.followed_id)
            .collect(),
        None => HashSet::new(),
    };

    let mut dtos = Vec::with_capacity(articles.len());
    for a in articles {
        let author = authors
            .get(&a.author_id)
            .ok_or_else(|| error_envelope(404, "author", &["not found"]))?;
        dtos.push(ArticleDto {
            slug: a.slug,
            title: a.title,
            description: a.description,
            body: a.body,
            tag_list: tags_by_article.remove(&a.id).unwrap_or_default(),
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
            favorited: viewer_favorites.contains(&a.id),
            favorites_count: *favorites_count.get(&a.id).unwrap_or(&0),
            author: ProfileDto {
                username: author.username.clone(),
                bio: author.bio.clone(),
                image: author.image.clone(),
                following: viewer_following.contains(&author.id),
            },
        });
    }
    Ok(dtos)
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

    // Title uniqueness (Conduit contract): a duplicate title is a 409 with
    // `errors.title = ["has already been taken"]`. The reference Conduit derives
    // the slug from the title and treats the title as unique; we enforce that
    // invariant explicitly here (the slug itself keeps a random suffix to avoid
    // collisions between genuinely distinct titles that slugify the same).
    let existing_title = article::Entity::find()
        .filter(article::Column::Title.eq(&r.title))
        .count(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;
    if existing_title > 0 {
        return Err(error_envelope(409, "title", &["has already been taken"]));
    }

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
    // 204 No Content — the Conduit collection asserts 204 on successful delete.
    Ok(HttpResponse::new().status(204))
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

    let dtos = to_article_dtos(&db, models, viewer).await?;

    Ok(HttpResponse::json(json!({
        "articles": dtos,
        "articlesCount": articles_count,
    })))
}

/// POST /api/articles/{slug}/favorite — favorite an article (required auth).
///
/// Inserts a `favorites` row (user_id = viewer, article_id). The composite PK
/// makes a repeat favorite a no-op (T-230-18), so a unique-violation insert error
/// is swallowed (idempotent). Returns the article with `favorited = true` and the
/// recomputed `favoritesCount`.
#[handler]
pub async fn favorite(req: Request) -> Response {
    let uid = require_viewer(&req)?;
    let slug = req.param("slug")?.to_string();
    let db = DB::get()?;
    let model = find_by_slug(&db, &slug).await?;

    // Idempotent: ignore the unique-violation on a duplicate (favorites composite PK).
    let _ = (favorite::ActiveModel {
        user_id: Set(uid as i32),
        article_id: Set(model.id),
    })
    .insert(&*db)
    .await;

    let dto = to_article_dto(&db, model, Some(uid)).await?;
    Ok(HttpResponse::json(json!({ "article": dto })))
}

/// DELETE /api/articles/{slug}/favorite — unfavorite (required auth).
///
/// Deletes the `favorites` row if present (a no-op when absent). Returns the
/// article with `favorited = false` and the recomputed `favoritesCount`.
#[handler]
pub async fn unfavorite(req: Request) -> Response {
    let uid = require_viewer(&req)?;
    let slug = req.param("slug")?.to_string();
    let db = DB::get()?;
    let model = find_by_slug(&db, &slug).await?;

    favorite::Entity::delete_by_id((uid as i32, model.id))
        .exec(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;

    let dto = to_article_dto(&db, model, Some(uid)).await?;
    Ok(HttpResponse::json(json!({ "article": dto })))
}

/// GET /api/articles/feed — articles by authors the viewer follows (required auth).
///
/// `followed_ids` come from the `follows` junction (`follower_id = viewer`); the
/// query filters `article.author_id IN followed_ids`, computes `articlesCount`
/// before pagination, then orders by `created_at` desc with limit/offset.
#[handler]
pub async fn feed(req: Request) -> Response {
    let uid = require_viewer(&req)?;
    let db = DB::get()?;
    let dberr = |e: sea_orm::DbErr| ferro::FrameworkError::database(e.to_string());

    let limit = req.query_as_or("limit", 20u64).min(MAX_LIMIT);
    let offset = req.query_as_or("offset", 0u64);

    let followed_ids: Vec<i32> = follow::Entity::find()
        .filter(follow::Column::FollowerId.eq(uid as i32))
        .all(&*db)
        .await
        .map_err(dberr)?
        .into_iter()
        .map(|f| f.followed_id)
        .collect();

    let query = article::Entity::find().filter(article::Column::AuthorId.is_in(followed_ids));

    let articles_count = query.clone().count(&*db).await.map_err(dberr)? as i64;

    let models = query
        .order_by_desc(article::Column::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(&*db)
        .await
        .map_err(dberr)?;

    let dtos = to_article_dtos(&db, models, Some(uid)).await?;

    Ok(HttpResponse::json(json!({
        "articles": dtos,
        "articlesCount": articles_count,
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
