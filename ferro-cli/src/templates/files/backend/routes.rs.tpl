use ferro::{delete, get, group, post, put, routes};

use crate::controllers;
use crate::middleware;

routes! {
    // Public routes
    get!("/", controllers::home::index),

    // Guest-only routes (redirect to dashboard if logged in)
    group!("/", {
        get!("/login", controllers::auth::show_login),
        post!("/login", controllers::auth::login),
        get!("/register", controllers::auth::show_register),
        post!("/register", controllers::auth::register),
        get!("/forgot-password", controllers::auth::show_forgot_password),
        post!("/forgot-password", controllers::auth::forgot_password),
        get!("/reset-password", controllers::auth::show_reset_password),
        post!("/reset-password", controllers::auth::reset_password),
    }).middleware(middleware::authenticate::guest()),

    // Protected routes (require authentication)
    group!("/", {
        get!("/dashboard", controllers::dashboard::index),
        post!("/logout", controllers::auth::logout),

        // Profile routes
        get!("/profile", controllers::profile::show),
        put!("/profile", controllers::profile::update),
        put!("/profile/password", controllers::profile::update_password),
        delete!("/profile", controllers::profile::destroy),

        // Settings routes
        get!("/settings", controllers::settings::show),
        put!("/settings", controllers::settings::update),
    }).middleware(middleware::authenticate::auth()),
}
