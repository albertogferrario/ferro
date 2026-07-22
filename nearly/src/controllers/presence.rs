//! Presence — update your own location, or check in to stay visible.

use ferro::serde_json::json;
use ferro::{handler, Auth, HttpResponse, Redirect, Request, Response};
use serde::Deserialize;

use crate::models::presence::Presence;
use crate::models::profile::Profile;
use crate::realtime;

/// Push a live presence update to everyone watching the map (public `nearby`).
async fn broadcast_presence(uid: i32, lat: f64, lng: f64) {
    let (name, status) = match Profile::find_by_user(uid).await.ok().flatten() {
        Some(p) => (p.display_name, p.status),
        None => (format!("Utente {uid}"), String::new()),
    };
    realtime::emit(
        "nearby",
        "PresenceUpdated",
        json!({ "user_id": uid, "name": name, "status": status, "lat": lat, "lng": lng }),
    )
    .await;
}

#[derive(Deserialize)]
struct LocationInput {
    lat: f64,
    lng: f64,
}

/// POST /presence — set the current user's coordinates (JSON or form).
///
/// This is the endpoint a real mobile client calls with device GPS; presence is
/// server-timestamped so a stale or replayed position expires off the map.
#[handler]
pub async fn update(req: Request) -> Response {
    let Some(uid) = Auth::id() else {
        return Err(HttpResponse::json(json!({ "error": "unauthenticated" })).status(401));
    };
    let loc: LocationInput = req.input().await?;
    Presence::upsert(uid as i32, loc.lat, loc.lng).await?;
    broadcast_presence(uid as i32, loc.lat, loc.lng).await;
    Ok(HttpResponse::json(json!({ "ok": true })))
}

/// POST /presence/checkin — "I'm still here": refresh last_seen without moving.
#[handler]
pub async fn checkin() -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    // Refresh last_seen; the demo seed and registration both create a presence,
    // so a missing one is harmless — either way we return to the map.
    let _ = Presence::touch(uid as i32).await?;
    // Re-announce the (unchanged) position so watchers see the pin stay fresh.
    if let Some(p) = Presence::find_by_user(uid as i32).await? {
        broadcast_presence(uid as i32, p.lat, p.lng).await;
    }
    Redirect::to("/map").into()
}
