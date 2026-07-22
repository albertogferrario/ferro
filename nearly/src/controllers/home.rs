//! Home: the splash landing and the full-screen map.

use std::collections::HashMap;

use ferro::serde_json::{json, Value};
use ferro::{handler, Inertia, Request, Response};

use crate::models::place::Place;
use crate::models::presence::{Presence, FRESH_TTL_MINUTES};
use crate::models::profile::Profile;

/// Central Milan — the demo city center.
const CENTER: [f64; 2] = [45.4642, 9.19];

/// GET / — splash / landing.
#[handler]
pub async fn splash(req: Request) -> Response {
    Inertia::render(&req, "Splash", json!({}))
}

/// GET /map — the full-screen map of nearby people and places.
///
/// Emits structured markers; the React map decides how to draw pins and
/// pop-ups. Stale presences expire off the map (the brief's "coarse and
/// expiring" presence).
#[handler]
pub async fn map(req: Request) -> Response {
    let profiles = Profile::all_visible().await.unwrap_or_default();
    let presences = Presence::all().await.unwrap_or_default();
    let places = Place::all().await.unwrap_or_default();

    let by_user: HashMap<i32, &Presence> = presences
        .iter()
        .filter(|p| p.is_fresh(FRESH_TTL_MINUTES))
        .map(|p| (p.user_id, p))
        .collect();

    let mut people: Vec<Value> = Vec::new();
    for profile in &profiles {
        if let Some(p) = by_user.get(&profile.user_id) {
            people.push(json!({
                "user_id": profile.user_id,
                "name": profile.display_name,
                "status": profile.status,
                "lat": p.lat,
                "lng": p.lng,
            }));
        }
    }

    let places: Vec<Value> = places
        .iter()
        .map(|pl| {
            json!({
                "id": pl.id,
                "name": pl.name,
                "category": pl.category,
                "premium": pl.premium,
                "lat": pl.lat,
                "lng": pl.lng,
            })
        })
        .collect();

    Inertia::render(
        &req,
        "Map",
        json!({
            "center": CENTER,
            "people": people,
            "places": places,
        }),
    )
}
