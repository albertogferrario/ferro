//! Benchmark endpoint handlers — four micro-endpoints against the world table

use ferro::{handler, Request, Response, DB};
use ferro::http::HttpResponse;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;

use crate::models::world;

/// Generate a random world id in [1, 10_000].
/// Uses rand::random to avoid holding a ThreadRng (!Send) across .await.
fn rand_id() -> i32 {
    // random::<u16>() fits in [0, 65535]; modulo gives [0, 9999], shift to [1, 10000].
    (rand::random::<u16>() as i32 % 10_000) + 1
}

#[handler]
pub async fn json_handler() -> Response {
    Ok(HttpResponse::json(json!({ "message": "Hello, World!" })))
}

#[handler]
pub async fn db_handler() -> Response {
    let db = DB::get()?;
    let id = rand_id();
    let row = world::Entity::find_by_id(id)
        .one(&*db)
        .await
        .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
        .ok_or_else(|| ferro::FrameworkError::database("world row not found".to_string()))?;
    Ok(HttpResponse::json(json!({ "id": row.id, "randomNumber": row.random_number })))
}

fn clamp_n(n: Option<String>) -> i32 {
    n.and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1)
        .clamp(1, 500)
}

#[handler]
pub async fn queries(req: Request) -> Response {
    let k = clamp_n(req.query("n"));
    let db = DB::get()?;
    let mut out = Vec::with_capacity(k as usize);
    for _ in 0..k {
        let id = rand_id();
        let row = world::Entity::find_by_id(id)
            .one(&*db)
            .await
            .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
            .ok_or_else(|| ferro::FrameworkError::database("world row not found".to_string()))?;
        out.push(json!({ "id": row.id, "randomNumber": row.random_number }));
    }
    Ok(HttpResponse::json(json!(out)))
}

#[handler]
pub async fn updates(req: Request) -> Response {
    let k = clamp_n(req.query("n"));
    let db = DB::get()?;
    let mut out = Vec::with_capacity(k as usize);
    for _ in 0..k {
        let id = rand_id();
        let mut row: world::ActiveModel = world::Entity::find_by_id(id)
            .one(&*db)
            .await
            .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
            .ok_or_else(|| ferro::FrameworkError::database("world row not found".to_string()))?
            .into();
        let new_n = rand_id();
        row.random_number = Set(new_n);
        let saved = row
            .update(&*db)
            .await
            .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;
        out.push(json!({ "id": saved.id, "randomNumber": saved.random_number }));
    }
    Ok(HttpResponse::json(json!(out)))
}
