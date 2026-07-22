//! Friendly error pages rendered through Inertia.

use ferro::serde_json::json;
use ferro::{handler, Inertia, Request, Response};

/// Render the `Error` React page with a status + copy.
pub fn render(req: &Request, status: u16, title: &str, message: &str) -> Response {
    Inertia::render(
        req,
        "Error",
        json!({ "status": status, "title": title, "message": message }),
    )
    .map(|r| r.status(status))
}

/// Fallback handler for unmatched routes → a 404 page.
#[handler]
pub async fn not_found(req: Request) -> Response {
    render(
        &req,
        404,
        "Pagina non trovata",
        "La pagina che cerchi non esiste o è stata spostata.",
    )
}
