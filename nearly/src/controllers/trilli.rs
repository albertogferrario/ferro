//! Trilli — the wordless-ping inbox and the send / accept / decline actions.

use std::collections::HashMap;

use ferro::serde_json::json;
use ferro::{handler, Auth, Inertia, Redirect, Request, Response};
use serde::Deserialize;

use crate::models::profile::Profile;
use crate::models::trillo::{Trillo, STATUS_ACCEPTED, STATUS_DECLINED, STATUS_PENDING};

#[derive(Deserialize)]
struct SendInput {
    to_user_id: i32,
}

/// Human label for a trillo status.
fn status_label(status: &str) -> &'static str {
    match status {
        STATUS_ACCEPTED => "Accettato",
        STATUS_DECLINED => "Ignorato",
        _ => "In attesa",
    }
}

/// GET /trilli — trilli received by the current user (browse).
#[handler]
pub async fn index(req: Request) -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    let uid = uid as i32;

    let mut trilli = Trillo::inbox(uid).await.unwrap_or_default();
    trilli.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let names: HashMap<i32, String> = Profile::all_visible()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| (p.user_id, p.display_name))
        .collect();

    let rows: Vec<_> = trilli
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "from": names.get(&t.from_user_id).cloned().unwrap_or_else(|| "Qualcuno".to_string()),
                "status": t.status,
                "status_label": status_label(&t.status),
                "pending": t.status == STATUS_PENDING,
            })
        })
        .collect();

    Inertia::render(&req, "Trilli", json!({ "trilli": rows }))
}

/// POST /trilli — send a trillo to another user (no message body, by design).
#[handler]
pub async fn send(req: Request) -> Response {
    let Some(uid) = Auth::id() else {
        return Redirect::to("/login").into();
    };
    let from = uid as i32;

    let input: SendInput = req.input().await?;
    if input.to_user_id != from {
        Trillo::send(from, input.to_user_id).await?;
        // Live ping to just the recipient's private channel. Only the sender's
        // display name travels — no message body, so the no-chat rule holds.
        let from_name = Profile::find_by_user(from)
            .await
            .ok()
            .flatten()
            .map(|p| p.display_name)
            .unwrap_or_else(|| "Qualcuno".to_string());
        crate::realtime::emit(
            &format!("private-user.{}", input.to_user_id),
            "TrilloReceived",
            json!({ "from": from_name }),
        )
        .await;
    }
    Redirect::to("/trilli").into()
}

/// POST /trilli/:id/accept
#[handler]
pub async fn accept(id: i32) -> Response {
    if Auth::id().is_none() {
        return Redirect::to("/login").into();
    }
    Trillo::set_status(id, STATUS_ACCEPTED).await?;
    Redirect::to("/trilli").into()
}

/// POST /trilli/:id/decline
#[handler]
pub async fn decline(id: i32) -> Response {
    if Auth::id().is_none() {
        return Redirect::to("/login").into();
    }
    Trillo::set_status(id, STATUS_DECLINED).await?;
    Redirect::to("/trilli").into()
}
