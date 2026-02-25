//! Authorization middleware for route protection.
//!
//! Provides middleware to protect routes with authorization checks.

use super::gate::Gate;
use crate::http::{HttpResponse, Request, Response};
use crate::middleware::{Middleware, Next};
use async_trait::async_trait;

/// Middleware that checks authorization before allowing access.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::authorization::Authorize;
/// use ferro_rs::routing::Route;
///
/// Route::get("/admin", admin_dashboard)
///     .middleware(Authorize::ability("view-admin"));
///
/// Route::put("/posts/{id}", update_post)
///     .middleware(Authorize::ability("update-post"));
/// ```
pub struct Authorize {
    ability: String,
}

impl Authorize {
    /// Create middleware that checks a specific ability.
    ///
    /// # Arguments
    ///
    /// * `ability` - The ability to check (e.g., "view-admin", "update-post")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Route::get("/dashboard", dashboard)
    ///     .middleware(Authorize::ability("view-dashboard"));
    /// ```
    pub fn ability(ability: impl Into<String>) -> Self {
        Self {
            ability: ability.into(),
        }
    }
}

#[async_trait]
impl Middleware for Authorize {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // Check if user is authenticated
        let user = match crate::auth::Auth::user().await {
            Ok(Some(u)) => u,
            Ok(None) => {
                return Err(HttpResponse::json(serde_json::json!({
                    "message": "Unauthenticated."
                }))
                .status(401));
            }
            Err(e) => {
                eprintln!("Failed to retrieve user for authorization: {e}");
                return Err(HttpResponse::json(serde_json::json!({
                    "message": "Authentication error."
                }))
                .status(500));
            }
        };

        // Check authorization
        let response = Gate::inspect(user.as_ref(), &self.ability, None);

        if response.denied() {
            let message = response.message().unwrap_or("This action is unauthorized.");
            return Err(HttpResponse::json(serde_json::json!({
                "message": message
            }))
            .status(response.status()));
        }

        next(request).await
    }
}

/// Macro for creating authorization middleware.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::can;
///
/// Route::get("/admin", admin_dashboard)
///     .middleware(can!("view-admin"));
///
/// // Multiple abilities (all must pass)
/// Route::put("/posts/{id}", update_post)
///     .middleware(can!("authenticated"))
///     .middleware(can!("update-post"));
/// ```
#[macro_export]
macro_rules! can {
    ($ability:expr) => {
        $crate::authorization::Authorize::ability($ability)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorize_creation() {
        let middleware = Authorize::ability("test-ability");
        assert_eq!(middleware.ability, "test-ability");
    }

    #[test]
    fn test_can_macro() {
        let middleware = can!("view-dashboard");
        assert_eq!(middleware.ability, "view-dashboard");
    }
}
