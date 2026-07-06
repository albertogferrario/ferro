//! Tag endpoint: index (no auth) — the flat list of tag names.

use ferro::serde_json::json;
use ferro::{handler, HttpResponse, Request, Response, DB};
use sea_orm::EntityTrait;

use crate::models::tag;

/// GET /api/tags — `{"tags":[name,...]}` (no auth).
#[handler]
pub async fn index(_req: Request) -> Response {
    let db = DB::get()?;
    let names: Vec<String> = tag::Entity::find()
        .all(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
        .into_iter()
        .map(|t| t.name)
        .collect();
    Ok(HttpResponse::json(json!({ "tags": names })))
}
