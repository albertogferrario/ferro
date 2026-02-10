//! Broadcasting channel authorization handler.
//!
//! Provides the [`broadcasting_auth`] handler that validates user authentication
//! and channel authorization for private and presence channels.

use crate::auth::Auth;
use crate::container::App;
use crate::error::FrameworkError;
use crate::http::{HttpResponse, Request, Response};
use ferro_broadcast::{AuthData, Broadcaster, ChannelType};
use serde::Deserialize;

#[derive(Deserialize)]
struct BroadcastAuthInput {
    channel_name: String,
    socket_id: String,
}

/// Broadcasting channel authorization handler.
///
/// Register this as a POST route in your routes file. It must run through
/// session middleware so that `Auth::id()` works.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::broadcasting_auth;
///
/// Route::post("/broadcasting/auth", broadcasting_auth)
///     .middleware(SessionAuthMiddleware);
/// ```
///
/// # Protocol
///
/// Clients send POST with `channel_name` and `socket_id` fields (form or JSON).
/// Returns 200 with auth confirmation if authorized, 403 otherwise.
///
/// For presence channels, the response includes `channel_data` with user info
/// for member tracking.
pub async fn broadcasting_auth(req: Request) -> Response {
    // Verify authenticated
    let user_id = Auth::id().ok_or_else(|| FrameworkError::domain("Unauthenticated", 401))?;

    // Parse body (supports both JSON and form-urlencoded)
    let input: BroadcastAuthInput = req.input().await?;

    // Get broadcaster from container
    let broadcaster = App::get::<Broadcaster>()
        .ok_or_else(|| FrameworkError::internal("Broadcasting not configured"))?;

    // Build auth data with user_id as the auth token
    let auth_data = AuthData {
        socket_id: input.socket_id.clone(),
        channel: input.channel_name.clone(),
        auth_token: Some(user_id.to_string()),
    };

    // Check authorization via registered authorizer
    let authorized = broadcaster.check_auth(&auth_data).await;
    if !authorized {
        return Err(HttpResponse::json(serde_json::json!({
            "error": "Forbidden"
        }))
        .status(403));
    }

    // Build response
    let mut response = serde_json::json!({
        "auth": "ok",
        "socket_id": input.socket_id,
        "channel": input.channel_name,
    });

    // For presence channels, include user info
    let channel_type = ChannelType::from_name(&input.channel_name);
    if channel_type == ChannelType::Presence {
        response["channel_data"] = serde_json::json!({
            "user_id": user_id.to_string(),
        });
    }

    Ok(HttpResponse::json(response))
}
