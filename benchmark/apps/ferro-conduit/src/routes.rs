use ferro::{delete, get, group, post, put, routes};

use crate::controllers;
use crate::middleware::jwt_auth::JwtAuthMiddleware;
use crate::middleware::optional_jwt::OptionalJwtMiddleware;

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

    // Public tags (no auth).
    get!("/api/tags", controllers::tags::index),

    // Required-auth user routes (no optional-auth sibling on the same path).
    group!("/api", {
        get!("/user", controllers::auth::current_user),
        put!("/user", controllers::auth::update_user),
    }).middleware(JwtAuthMiddleware),

    // All article routes share `OptionalJwtMiddleware`. Ferro's route middleware
    // is keyed by PATH, not by method, so GET `/api/articles` and POST
    // `/api/articles` resolve to the same middleware list — a per-method
    // optional-vs-required split is impossible at the middleware layer. Reads run
    // as guests; the mutation handlers (`store`/`update`/`destroy`/feed) enforce
    // required auth themselves via `require_viewer()` (401 when no UserId).
    //
    // `/articles/feed` (literal) is declared BEFORE `/articles/{slug}` so feed is
    // never shadowed (RESEARCH Pitfall 2; guarded by tests/route_ordering.rs).
    group!("/api", {
        get!("/articles/feed", controllers::articles::feed),
        get!("/articles", controllers::articles::index),
        get!("/articles/{slug}", controllers::articles::show),
        post!("/articles", controllers::articles::store),
        put!("/articles/{slug}", controllers::articles::update),
        delete!("/articles/{slug}", controllers::articles::destroy),
        // Comments share the `/articles/{slug}/comments` path between a public GET
        // (list) and an auth'd POST (add). Ferro route middleware is PATH-keyed, so
        // the GET and POST on this path MUST sit under the same (optional) middleware
        // — a required-auth POST here would force the public GET to 401. `store`
        // self-enforces `require_viewer()` (401 when no UserId), mirroring the
        // article mutation handlers. `destroy` likewise self-enforces.
        get!("/articles/{slug}/comments", controllers::comments::index),
        post!("/articles/{slug}/comments", controllers::comments::store),
        delete!("/articles/{slug}/comments/{id}", controllers::comments::destroy),
    }).middleware(OptionalJwtMiddleware),

    // Favorite/unfavorite are required-auth. Their paths (`/articles/{slug}/favorite`)
    // are distinct from any public article path, so they carry JwtAuthMiddleware
    // directly; the handlers also self-enforce `require_viewer()`.
    group!("/api", {
        post!("/articles/{slug}/favorite", controllers::articles::favorite),
        delete!("/articles/{slug}/favorite", controllers::articles::unfavorite),
    }).middleware(JwtAuthMiddleware),

    // Profile show is optional-auth (viewer-relative `following`). Follow/unfollow
    // require auth: Ferro route middleware is PATH-keyed, so `/profiles/{username}`
    // and `/profiles/{username}/follow` are distinct paths and CAN carry distinct
    // middleware. Show runs as guest-capable; follow/unfollow additionally
    // self-enforce `require_viewer()` (401 when no UserId).
    group!("/api", {
        get!("/profiles/{username}", controllers::profiles::show),
    }).middleware(OptionalJwtMiddleware),

    group!("/api", {
        post!("/profiles/{username}/follow", controllers::profiles::follow),
        delete!("/profiles/{username}/follow", controllers::profiles::unfollow),
    }).middleware(JwtAuthMiddleware),
}
