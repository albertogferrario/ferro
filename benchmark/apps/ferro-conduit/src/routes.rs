use ferro::{get, group, post, put, routes};

use crate::controllers;
use crate::middleware::jwt_auth::JwtAuthMiddleware;

// Route-ordering constraint (RESEARCH Pitfall 2): when the article routes land
// in Plan 04/05, declare the literal `/api/articles/feed` BEFORE the
// parameterized `/api/articles/{slug}`. Ferro's matchit router gives literal
// segments priority over wildcards, but declaring literal-first keeps the
// ordering explicit. See tests/route_ordering.rs for the registration guard.
//
// `routes!` expands to `pub fn register() -> Router`.
routes! {
    get!("/health", controllers::health::show),

    // Public auth
    post!("/api/users", controllers::auth::register),
    post!("/api/users/login", controllers::auth::login),

    // Required-auth (JWT)
    group!("/api", {
        get!("/user", controllers::auth::current_user),
        put!("/user", controllers::auth::update_user),
    }).middleware(JwtAuthMiddleware),
}
