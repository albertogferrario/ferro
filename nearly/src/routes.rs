//! Application routes.

use ferro::{fallback, get, group, post, routes};

use crate::controllers;
use crate::middleware;

routes! {
    // ── Public ────────────────────────────────────────────────────────────
    get!("/", controllers::home::splash).name("home"),
    get!("/map", controllers::home::map).name("map"),
    get!("/utenti/:id", controllers::utenti::show).name("utenti.show"),
    get!("/places", controllers::places::index).name("places.index"),

    // ── Guest-only (redirect to /map if already signed in) ────────────────
    group!("/", {
        get!("/login", controllers::auth::login_page).name("login.page"),
        post!("/login", controllers::auth::login).name("login"),
        get!("/register", controllers::auth::register_page).name("register.page"),
        post!("/register", controllers::auth::register).name("register"),
    }).middleware(middleware::authenticate::guest()),

    // ── Authenticated ─────────────────────────────────────────────────────
    group!("/", {
        post!("/logout", controllers::auth::logout).name("logout"),

        get!("/account", controllers::account::show).name("account.show"),
        post!("/account", controllers::account::update).name("account.update"),

        get!("/settings", controllers::settings::show).name("settings.show"),
        post!("/settings", controllers::settings::update).name("settings.update"),

        post!("/presence", controllers::presence::update).name("presence.update"),
        post!("/presence/checkin", controllers::presence::checkin).name("presence.checkin"),

        // Real-time channel authorization (session-authenticated; signs the token).
        post!("/broadcasting/auth", ferro::broadcasting_auth).name("broadcasting.auth"),

        get!("/trilli", controllers::trilli::index).name("trilli.index"),
        post!("/trilli", controllers::trilli::send).name("trilli.send"),
        post!("/trilli/:id/accept", controllers::trilli::accept).name("trilli.accept"),
        post!("/trilli/:id/decline", controllers::trilli::decline).name("trilli.decline"),
    }).middleware(middleware::authenticate::auth()),

    // Friendly 404 for any unmatched route.
    fallback!(controllers::errors::not_found),
}
