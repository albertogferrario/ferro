//! Book catalog client.
//!
//! Searches external, legally-usable book catalogs and returns a unified list
//! of results that can be saved to the personal library:
//!
//! * **Open Library** (`openlibrary.org`) — universal metadata for essentially
//!   every book that exists (~40M editions). Metadata only; no file download.
//! * **Project Gutenberg** via the **Gutendex** API (`gutendex.com`) —
//!   public-domain titles that carry a real, downloadable EPUB.
//!
//! Both are queried in parallel and merged. If one source is unavailable the
//! other still returns, so a transient outage never fails the whole search.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// A single search hit, normalized across catalog sources.
///
/// This is the shape the frontend renders and the shape the "add to library"
/// endpoint accepts back, so the client never has to re-query to import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub author: Option<String>,
    pub year: Option<i32>,
    pub isbn: Option<String>,
    pub cover_url: Option<String>,
    /// Originating catalog: `"openlibrary"` or `"gutenberg"`.
    pub source: String,
    /// The source's own identifier, used for de-duplication on import.
    pub source_id: String,
    /// True when `download_url` points at a file we are allowed to fetch.
    pub public_domain: bool,
    pub download_url: Option<String>,
    pub description: Option<String>,
}

/// Shared HTTP client, created once on first use.
fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            // Open Library asks callers to identify themselves with a UA.
            .user_agent("LaMiaLibreria/0.1 (personal book collection)")
            .build()
            .expect("failed to build HTTP client")
    })
}

/// Download a public-domain file (e.g. an EPUB) from a catalog URL.
///
/// Returns the raw bytes so the caller can hand them straight to storage.
pub async fn download_file(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let bytes = client()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(bytes.to_vec())
}

/// Search every configured catalog and return the merged results.
///
/// Public-domain (Gutenberg) hits are listed first because they are the ones
/// the user can actually download and read.
pub async fn search(query: &str) -> Vec<SearchResult> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    // Query both sources concurrently; each tolerates the other's failure.
    let (gutenberg, open_library) =
        tokio::join!(search_gutenberg(query), search_open_library(query));

    let mut results = gutenberg.unwrap_or_default();
    results.extend(open_library.unwrap_or_default());
    results
}

// ---------------------------------------------------------------------------
// Open Library
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct OpenLibraryResponse {
    #[serde(default)]
    docs: Vec<OpenLibraryDoc>,
}

#[derive(Deserialize)]
struct OpenLibraryDoc {
    key: Option<String>,
    title: Option<String>,
    #[serde(default)]
    author_name: Vec<String>,
    first_publish_year: Option<i32>,
    cover_i: Option<i64>,
    #[serde(default)]
    isbn: Vec<String>,
}

async fn search_open_library(query: &str) -> Result<Vec<SearchResult>, reqwest::Error> {
    let url = "https://openlibrary.org/search.json";
    let resp: OpenLibraryResponse = client()
        .get(url)
        .query(&[
            ("q", query),
            ("limit", "15"),
            ("fields", "key,title,author_name,first_publish_year,cover_i,isbn"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let results = resp
        .docs
        .into_iter()
        .filter_map(|doc| {
            let title = doc.title?;
            let source_id = doc.key?;
            let cover_url = doc
                .cover_i
                .map(|id| format!("https://covers.openlibrary.org/b/id/{id}-M.jpg"));
            Some(SearchResult {
                title,
                author: doc.author_name.into_iter().next(),
                year: doc.first_publish_year,
                isbn: doc.isbn.into_iter().next(),
                cover_url,
                source: "openlibrary".to_string(),
                source_id,
                public_domain: false,
                download_url: None,
                description: None,
            })
        })
        .collect();

    Ok(results)
}

// ---------------------------------------------------------------------------
// Project Gutenberg (via Gutendex)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GutendexResponse {
    #[serde(default)]
    results: Vec<GutendexBook>,
}

#[derive(Deserialize)]
struct GutendexBook {
    id: i64,
    title: Option<String>,
    #[serde(default)]
    authors: Vec<GutendexAuthor>,
    /// MIME type -> URL (e.g. "application/epub+zip", "image/jpeg").
    #[serde(default)]
    formats: std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
struct GutendexAuthor {
    name: Option<String>,
}

async fn search_gutenberg(query: &str) -> Result<Vec<SearchResult>, reqwest::Error> {
    let url = "https://gutendex.com/books";
    let resp: GutendexResponse = client()
        .get(url)
        .query(&[("search", query)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let results = resp
        .results
        .into_iter()
        .filter_map(|book| {
            let title = book.title?;
            // Prefer a plain EPUB; fall back to any epub variant.
            let download_url = book
                .formats
                .iter()
                .find(|(mime, _)| mime.starts_with("application/epub+zip"))
                .map(|(_, url)| url.clone());
            let cover_url = book
                .formats
                .iter()
                .find(|(mime, _)| mime.starts_with("image/jpeg"))
                .map(|(_, url)| url.clone());
            Some(SearchResult {
                title,
                author: book.authors.into_iter().find_map(|a| a.name),
                year: None,
                isbn: None,
                cover_url,
                source: "gutenberg".to_string(),
                source_id: book.id.to_string(),
                public_domain: true,
                download_url,
                description: None,
            })
        })
        .collect();

    Ok(results)
}
