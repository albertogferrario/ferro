//! Required JWT auth middleware: rejects with a Conduit 401 envelope when no
//! valid `Authorization: Token <jwt>` is present.

use ferro::serde_json::json;
use ferro::{async_trait, HttpResponse, Middleware, Next, Request, Response};

use super::extract_user_id;

/// Rejects unauthenticated requests with `401 {"errors":{"token":["is missing"]}}`
/// (Conduit "Error Cases - Auth" contract). On a valid token, inserts `UserId`
/// into the request extension map and proceeds.
pub struct JwtAuthMiddleware;

#[async_trait]
impl Middleware for JwtAuthMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        match extract_user_id(request.header("Authorization")) {
            Some(user_id) => {
                request.insert(user_id);
                next(request).await
            }
            None => Err(HttpResponse::json(json!({"errors": {"token": ["is missing"]}}))
                .status(401)),
        }
    }
}
