//! Shares common data with every Inertia response: the authenticated user and
//! a CSRF token. React reads these from the page's shared props (e.g. the app
//! shell shows the nav + logout only when `auth` is present).

use ferro::serde_json::json;
use ferro::{async_trait, csrf_token, Auth, InertiaShared, Middleware, Next, Request, Response};

use crate::models::profile::Profile;
use crate::models::user::User;

pub struct ShareInertiaData;

#[async_trait]
impl Middleware for ShareInertiaData {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        let mut shared = InertiaShared::new();

        if let Some(token) = csrf_token() {
            shared = shared.csrf(token);
        }

        // Share a compact auth object when signed in. `Auth::user_as` works
        // because `User` derives `Authenticatable` and bootstrap registered a
        // `ModelUserProvider` — no manual session-id + find_by_pk plumbing.
        if let Ok(Some(user)) = Auth::user_as::<User>().await {
            let display = Profile::find_by_user(user.id)
                .await
                .ok()
                .flatten()
                .map(|p| p.display_name)
                .unwrap_or_else(|| user.name.clone());
            shared = shared.auth(json!({
                "id": user.id,
                "name": user.name,
                "display_name": display,
            }));
        }

        request.insert(shared);
        next(request).await
    }
}
