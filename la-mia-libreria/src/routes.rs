//! Route registration.
//!
//! The `routes!` macro generates `pub fn register()`, called from `main.rs`.

use ferro::{delete, get, group, post, routes};

use crate::controllers;

routes! {
    // Single-page UI
    get!("/", controllers::library::page).name("library.page"),

    // JSON API backing the UI
    group!("/library", {
        get!("/search", controllers::library::search).name("library.search"),
        get!("/books", controllers::library::index).name("library.books.index"),
        post!("/books", controllers::library::store).name("library.books.store"),
        delete!("/books/:book", controllers::library::destroy).name("library.books.destroy"),
        post!("/books/:book/download", controllers::library::download).name("library.books.download"),
    }),
}
