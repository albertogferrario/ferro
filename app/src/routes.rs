use ferro::{get, group, post, resource, routes};
use ferro::{AuthMiddleware as SessionAuthMiddleware, GuestMiddleware, Throttle};

use crate::controllers;
use crate::middleware::AuthMiddleware;

routes! {
    get!("/", controllers::home::index).name("home"),
    get!("/redirect-example", controllers::user::redirect_example),
    get!("/config", controllers::config_example::show).name("config.show"),

    // User routes - all 7 RESTful endpoints from a single line
    resource!("/users", controllers::user),

    // Protected routes - requires Authorization header
    group!("/protected", {
        get!("/", controllers::home::index).name("protected.home"),
    }).middleware(AuthMiddleware),

    // Todo routes group
    group!("/todos", {
        get!("/", controllers::todo::list).name("todos.index"),
        post!("/random", controllers::todo::create_random).name("todos.create_random"),
    }),

    // Auth routes - guest only (redirects authenticated users)
    group!("/auth", {
        post!("/register", controllers::auth_controller::register).name("auth.register"),
        post!("/login", controllers::auth_controller::login).name("auth.login"),
    }).middleware(GuestMiddleware::redirect_to("/")),

    // Auth routes - authenticated only
    group!("/auth", {
        get!("/profile", controllers::auth_controller::profile).name("auth.profile"),
        post!("/logout", controllers::auth_controller::logout).name("auth.logout"),
    }).middleware(SessionAuthMiddleware::new()),

    // API routes - rate limited with named "api" limiter (60 req/min)
    group!("/api", {
        get!("/users", controllers::user::api_index).name("api.users.index"),
    }).middleware(Throttle::named("api")),

    // Broadcasting auth (uncomment when broadcasting is configured in bootstrap):
    // post!("/broadcasting/auth", ferro::broadcasting_auth),
}
