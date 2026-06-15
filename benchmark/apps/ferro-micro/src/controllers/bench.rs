//! Benchmark endpoint handlers — four micro-endpoints against the world table

use ferro::{handler, Request, Response, DB};
use ferro::http::HttpResponse;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use rand::Rng;
use serde_json::json;

use crate::models::world;

#[handler]
pub async fn json_handler() -> Response {
    Ok(HttpResponse::json(json!({ "message": "Hello, World!" })))
}

#[handler]
pub async fn db_handler() -> Response {
    let db = DB::get()?;
    let id = rand::thread_rng().gen_range(1i32..=10_000);
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
        let id = rand::thread_rng().gen_range(1i32..=10_000);
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
        let id = rand::thread_rng().gen_range(1i32..=10_000);
        let mut row: world::ActiveModel = world::Entity::find_by_id(id)
            .one(&*db)
            .await
            .map_err(|e| ferro::FrameworkError::database(e.to_string()))?
            .ok_or_else(|| ferro::FrameworkError::database("world row not found".to_string()))?
            .into();
        let new_n = rand::thread_rng().gen_range(1i32..=10_000);
        row.random_number = Set(new_n);
        let saved = row
            .update(&*db)
            .await
            .map_err(|e| ferro::FrameworkError::database(e.to_string()))?;
        out.push(json!({ "id": saved.id, "randomNumber": saved.random_number }));
    }
    Ok(HttpResponse::json(json!(out)))
}
