//! User pop-up — the focused profile card reached by tapping a map pin.

use ferro::serde_json::json;
use ferro::{handler, Inertia, Request, Response};

use crate::controllers::errors;
use crate::models::profile::Profile;

/// GET /utenti/:id — a single person's card with the "invia un trillo" action.
#[handler]
pub async fn show(req: Request, id: i32) -> Response {
    let profile = match Profile::find_by_user(id).await? {
        Some(p) => p,
        None => {
            return errors::render(
                &req,
                404,
                "Profilo non trovato",
                "Questa persona non è più su Nearly.",
            )
        }
    };

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
