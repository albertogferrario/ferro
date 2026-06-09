//! Personal library controller.
//!
//! The library lets you search every book that exists (via Open Library and
//! Project Gutenberg), save the ones you care about to your own collection,
//! and — for public-domain titles — download the real EPUB file into local
//! storage so you can keep and read it.
//!
//! Endpoints:
//! * `GET  /`                      — the single-page UI
//! * `GET  /library/search?q=...`  — search external catalogs
//! * `GET  /library/books`         — list the saved collection
//! * `POST /library/books`         — save a search result to the collection
//! * `DELETE /library/books/:book` — remove a book from the collection
//! * `POST /library/books/:book/download` — fetch a public-domain file

use crate::catalog;
use crate::models::books;
use ferro::serde::Deserialize;
use ferro::serde_json::json;
use ferro::{handler, FormRequest, HttpResponse, Request, Response, Storage};
use sea_orm::EntityTrait;

/// Helper: 500 response from any displayable error.
fn server_error(e: impl std::fmt::Display) -> HttpResponse {
    HttpResponse::json(json!({ "error": e.to_string() })).status(500)
}

/// Serve the single-page library UI.
#[handler]
pub async fn page() -> Response {
    Ok(HttpResponse::text(PAGE_HTML).header("Content-Type", "text/html; charset=utf-8"))
}

/// Search external catalogs for any book.
///
/// GET /library/search?q=...
#[handler]
pub async fn search(req: Request) -> Response {
    let query: String = req.query_as_or("q", String::new());
    let results = catalog::search(&query).await;
    Ok(HttpResponse::json(json!({ "results": results })))
}

/// List the saved collection (newest first).
///
/// GET /library/books
#[handler]
pub async fn index() -> Response {
    let db = ferro::DB::connection().map_err(server_error)?;
    let mut books = books::Entity::find()
        .all(db.inner())
        .await
        .map_err(server_error)?;
    // Newest first by id (creation order).
    books.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(HttpResponse::json(json!({ "books": books })))
}

/// Request body for saving a search result. Mirrors `catalog::SearchResult`.
#[derive(Deserialize)]
pub struct SaveBookRequest {
    pub title: String,
    pub author: Option<String>,
    pub year: Option<i32>,
    pub isbn: Option<String>,
    pub cover_url: Option<String>,
    pub source: String,
    pub source_id: String,
    #[serde(default)]
    pub public_domain: bool,
    pub download_url: Option<String>,
    pub description: Option<String>,
}

impl ferro::Validate for SaveBookRequest {
    fn validate(&self) -> Result<(), ferro::validator::ValidationErrors> {
        Ok(())
    }
}

impl FormRequest for SaveBookRequest {}

/// Save a search result to the collection.
///
/// Idempotent on `(source, source_id)`: saving the same catalog entry twice
/// returns the book that already exists instead of erroring.
///
/// POST /library/books
#[handler]
pub async fn store(form: SaveBookRequest) -> Response {
    if let Some(existing) = books::Model::find_by_source(&form.source, &form.source_id)
        .await
        .map_err(server_error)?
    {
        return Ok(HttpResponse::json(json!({ "book": existing, "created": false })));
    }

    let mut builder = books::Model::create()
        .set_title(form.title.clone())
        .set_source(form.source.clone())
        .set_source_id(form.source_id.clone())
        .set_public_domain(form.public_domain)
        .set_status("wanted");

    if let Some(v) = &form.author {
        builder = builder.set_author(v.clone());
    }
    if let Some(v) = form.year {
        builder = builder.set_year(v);
    }
    if let Some(v) = &form.isbn {
        builder = builder.set_isbn(v.clone());
    }
    if let Some(v) = &form.cover_url {
        builder = builder.set_cover_url(v.clone());
    }
    if let Some(v) = &form.description {
        builder = builder.set_description(v.clone());
    }
    if let Some(v) = &form.download_url {
        builder = builder.set_download_url(v.clone());
    }

    let book = builder.insert().await.map_err(server_error)?;
    Ok(HttpResponse::json(json!({ "book": book, "created": true })).status(201))
}

/// Remove a book from the collection.
///
/// DELETE /library/books/:book
#[handler]
pub async fn destroy(book: books::Model) -> Response {
    book.delete().await.map_err(server_error)?;
    Ok(HttpResponse::json(json!({ "deleted": true })))
}

/// Download the file for a public-domain book into local storage.
///
/// Only public-domain titles with a `download_url` are eligible — anything else
/// is rejected, because downloading copyrighted files is not something this app
/// does.
///
/// POST /library/books/:book/download
#[handler]
pub async fn download(book: books::Model) -> Response {
    if !book.public_domain {
        return Ok(HttpResponse::json(json!({
            "error": "This book is not public domain; its file cannot be downloaded."
        }))
        .status(403));
    }

    let url = match &book.download_url {
        Some(u) => u.clone(),
        None => {
            return Ok(HttpResponse::json(json!({
                "error": "No download URL available for this book."
            }))
            .status(422))
        }
    };

    let bytes = catalog::download_file(&url).await.map_err(server_error)?;

    let path = format!("books/{}.epub", book.id);
    Storage::new().put(&path, bytes).await.map_err(server_error)?;

    let updated = book
        .update()
        .set_local_path(path.clone())
        .set_status("owned")
        .save()
        .await
        .map_err(server_error)?;

    Ok(HttpResponse::json(json!({ "book": updated, "stored_at": path })))
}

/// Self-contained single-page UI (no build step, no JS framework).
const PAGE_HTML: &str = include_str!("library_page.html");
