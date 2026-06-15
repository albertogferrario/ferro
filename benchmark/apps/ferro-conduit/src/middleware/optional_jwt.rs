//! Optional JWT auth middleware: inserts `UserId` when a valid token is present,
//! never rejects. Used by Conduit's optional-auth routes (article list, article
//! detail, profile) where `following`/`favorited` differ for guests vs users.

use ferro::{async_trait, Middleware, Next, Request, Response};

use super::extract_user_id;

/// Inserts `UserId` if a valid token is present; proceeds as guest otherwise.
pub struct OptionalJwtMiddleware;

#[async_trait]
impl Middleware for OptionalJwtMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        if let Some(user_id) = extract_user_id(request.header("Authorization")) {
            request.insert(user_id);
        }
        next(request).await
    }
}
