//! Home: the splash landing and the full-screen map.

use std::collections::HashMap;

use ferro::serde_json::{json, Value};
use ferro::{handler, JsonUi, Response};

use crate::models::place::Place;
use crate::models::presence::Presence;
use crate::models::profile::Profile;

/// Central Milan — the demo city center.
const CENTER: [f64; 2] = [45.4642, 9.19];

/// Person pin (blue) and place pins (premium gold / trend green).
const COLOR_PERSON: &str = "#2563eb";
const COLOR_PREMIUM: &str = "#f59e0b";
const COLOR_PLACE: &str = "#16a34a";

/// GET / — splash / landing.
#[handler]
pub async fn splash() -> Response {
    JsonUi::render_file("src/views/splash.json", json!({}))
}

/// GET /map — the full-screen map of nearby people and places.
#[handler]
pub async fn map() -> Response {
    let profiles = Profile::all_visible().await.unwrap_or_default();
    let presences = Presence::all().await.unwrap_or_default();
    let places = Place::all().await.unwrap_or_default();

    // Index presences by user for a cheap join.
    let by_user: HashMap<i32, &Presence> = presences.iter().map(|p| (p.user_id, p)).collect();

    let mut markers: Vec<Value> = Vec::new();

    for profile in &profiles {
        if let Some(p) = by_user.get(&profile.user_id) {
            let popup = format!(
                "<strong>{}</strong><br>{}<br><a href=\"/utenti/{}\">Vedi profilo →</a>",
                esc(&profile.display_name),
                esc(&profile.status),
                profile.user_id
            );
            markers.push(json!({
                "lat": p.lat,
                "lng": p.lng,
                "color": COLOR_PERSON,
                "popup_html": popup,
            }));
        }
    }

    for place in &places {
        let color = if place.premium {
            COLOR_PREMIUM
        } else {
            COLOR_PLACE
        };
        let badge = if place.premium { " · ⭐ Premium" } else { "" };
        let popup = format!(
            "<strong>{}</strong><br>{}{}",
            esc(&place.name),
            esc(&place.category),
            badge
        );
        markers.push(json!({
            "lat": place.lat,
            "lng": place.lng,
            "color": color,
            "popup_html": popup,
        }));
    }

    JsonUi::render_file(
        "src/views/map.json",
        json!({
            "center": CENTER,
            "markers": markers,
            "people_count": profiles.len(),
        }),
    )
}

/// Minimal HTML escaping for values embedded in marker popup HTML.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
