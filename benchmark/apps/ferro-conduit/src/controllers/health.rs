use ferro::serde_json::json;
use ferro::{handler, HttpResponse, Response};

/// Liveness probe — returns `{"status":"ok"}`.
#[handler]
pub async fn show() -> Response {
    Ok(HttpResponse::json(json!({ "status": "ok" })))
}
