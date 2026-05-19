use ferro::{get, group, post, resource, routes};
use ferro::{AuthMiddleware as SessionAuthMiddleware, GuestMiddleware};

use crate::api::docs::docs_routes;
use crate::api::routes::api_routes;
use crate::controllers;
use crate::middleware::AuthMiddleware;

routes! {
    get!("/", controllers::home::index).name("home"),
    get!("/redirect-example", controllers::user::redirect_example),
    get!("/config", controllers::config_example::show).name("config.show"),
    get!("/pagamenti", controllers::pagamenti::index).name("pagamenti.index"),

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

    // API CRUD routes - protected by API key middleware
    api_routes(),

    // API documentation and OpenAPI spec
    docs_routes(),

    // Broadcasting auth (uncomment when broadcasting is configured in bootstrap):
    // post!("/broadcasting/auth", ferro::broadcasting_auth),
}
