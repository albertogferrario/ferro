//! User pop-up — the focused profile card reached by tapping a map pin.

use ferro::serde_json::json;
use ferro::{handler, HttpResponse, Inertia, Request, Response};

use crate::models::profile::Profile;

/// GET /utenti/:id — a single person's card with the "invia un trillo" action.
#[handler]
pub async fn show(req: Request, id: i32) -> Response {
    let profile = Profile::find_by_user(id)
        .await?
        .ok_or_else(|| HttpResponse::text("Profilo non trovato").status(404))?;

    Inertia::render(
        &req,
        "User",
        json!({
            "user_id": id,
            "display_name": profile.display_name,
            "status": profile.status,
        }),
    )
}
