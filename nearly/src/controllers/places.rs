//! Places — browse the trend + premium venues.

use ferro::serde_json::json;
use ferro::{handler, JsonUi, Response};

use crate::models::place::Place;

/// GET /places — the venue list (browse).
#[handler]
pub async fn index() -> Response {
    let mut places = Place::all().await.unwrap_or_default();
    // Premium venues first, then alphabetical.
    places.sort_by(|a, b| b.premium.cmp(&a.premium).then(a.name.cmp(&b.name)));

    let rows: Vec<_> = places
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "category": p.category,
                "badge": if p.premium { "⭐ Premium" } else { "Trend" },
            })
        })
        .collect();

    JsonUi::render_file("src/views/places.json", json!({ "places": rows }))
}
