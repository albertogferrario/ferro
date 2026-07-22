//! User pop-up — the focused profile card reached by tapping a map pin.

use ferro::serde_json::json;
use ferro::{handler, HttpResponse, JsonUi, Response};

use crate::models::profile::Profile;

/// GET /utenti/:id — a single person's card with the "invia un trillo" action.
#[handler]
pub async fn show(id: i32) -> Response {
    let profile = Profile::find_by_user(id)
        .await?
        .ok_or_else(|| HttpResponse::text("Profilo non trovato").status(404))?;

    JsonUi::render_file(
        "src/views/utente.json",
        json!({
            "display_name": profile.display_name,
            "status": profile.status,
            "to_user_id": id.to_string(),
        }),
    )
}
