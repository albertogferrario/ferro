//! In-memory immutable byte blobs with content-hashed URLs and one-year immutable caching.
//!
//! See the crate README for the bundle-vs-filesystem split: ferro-bundle handles
//! compile-time-embedded immutable assets; the framework's filesystem static-file
//! handler at `ferro_rs::static_files` handles mutable on-disk tenant assets.
//!
//! # Usage
//!
//! Register bundles at boot, then dispatch via [`Bundle::serve`] in a handler mounted
//! on `/bundles/{filename}` and on each registered alias path.
//!
//! ```rust,ignore
//! use ferro_bundle::Bundle;
//!
//! // Boot-time registration. Builder order matters: content_type BEFORE with_alias.
//! Bundle::new("embed-v1", include_bytes!("../assets/embed-v1.js"))
//!     .content_type("application/javascript")
//!     .with_alias("/embed/v1.js");
//!
//! async fn handler(req: ferro_rs::Request) -> ferro_rs::HttpResponse {
//!     Bundle::serve(req)
//! }
//! ```
//!
//! # Builder order
//!
//! Boot-time builder chain: `Bundle::new(name, bytes).content_type(ct).with_alias(path)`.
//! Call `.content_type(...)` before `.with_alias(...)` — `.content_type` re-keys the
//! bundle's URL (the extension is appended), and `.with_alias` captures the current
//! hashed URL at the time it is called.

use bytes::Bytes;
use dashmap::DashMap;
use ferro_rs::{HttpResponse, Request};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

// ── Error type ─────────────────────────────────────────────────────────

/// Single error type for the ferro-bundle crate.
///
/// `Bundle::serve` returns `HttpResponse` directly (not `Result`), so this enum is
/// primarily an internal/registration-time signal. The `DuplicateName` variant is
/// produced as a `panic!` message per D-06 (developer error, caught at boot).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bundle not found at path: {0}")]
    NotFound(String),
    #[error("duplicate bundle name: {0} already registered")]
    DuplicateName(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, Error>;

// ── Process-global registries (D-02) ───────────────────────────────────

#[derive(Debug, Clone)]
struct BundleEntry {
    name: String,
    bytes: &'static [u8],
    content_type: String,
    sha256_full_hex: String,
    sha256_short_hex: String,
    ext: String,
    hashed_url: String,
}

static BUNDLE_REGISTRY: OnceLock<DashMap<String, BundleEntry>> = OnceLock::new();
static ALIAS_REGISTRY: OnceLock<DashMap<String, String>> = OnceLock::new();
// Secondary index: bundle name -> current hashed_url. Lets `.content_type` and
// `.hashed_url` find the entry in O(1) without scanning the registry.
static NAME_INDEX: OnceLock<DashMap<String, String>> = OnceLock::new();

fn bundle_registry() -> &'static DashMap<String, BundleEntry> {
    BUNDLE_REGISTRY.get_or_init(DashMap::new)
}

fn alias_registry() -> &'static DashMap<String, String> {
    ALIAS_REGISTRY.get_or_init(DashMap::new)
}

fn name_index() -> &'static DashMap<String, String> {
    NAME_INDEX.get_or_init(DashMap::new)
}

// ── Content-type to extension mapping ──────────────────────────────────

fn ext_from_content_type(ct: &str) -> &'static str {
    match ct.split(';').next().unwrap_or(ct).trim() {
        "application/javascript" | "text/javascript" => "js",
        "text/css" => "css",
        "text/html" => "html",
        "text/plain" => "txt",
        "application/json" => "json",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/svg+xml" => "svg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "font/woff2" => "woff2",
        "font/woff" => "woff",
        "application/wasm" => "wasm",
        _ => "",
    }
}

fn hashed_url_for(name: &str, sha8: &str, ext: &str) -> String {
    if ext.is_empty() {
        format!("/bundles/{name}.{sha8}")
    } else {
        format!("/bundles/{name}.{sha8}.{ext}")
    }
}

// ── Public API ─────────────────────────────────────────────────────────

/// In-memory immutable byte blob registered at boot.
///
/// See the crate-level docs for the builder chain and ordering.
pub struct Bundle {
    name: String,
}

impl Bundle {
    /// Register a new bundle. Hashes the bytes (SHA-256), inserts an entry into the
    /// process-global registry keyed by the hashed URL, and returns a `Bundle` handle.
    ///
    /// # Panics
    ///
    /// Panics if a bundle with the same `name` is already registered (D-06). Duplicate
    /// registration is developer error caught at boot.
    pub fn new(name: &str, bytes: &'static [u8]) -> Self {
        if name_index().contains_key(name) {
            panic!("ferro-bundle: duplicate registration for bundle name {name:?}");
        }

        let digest = Sha256::digest(bytes);
        let sha256_full_hex = hex::encode(digest);
        let sha256_short_hex = sha256_full_hex[..8].to_string();
        let content_type = "application/octet-stream".to_string();
        let ext = ext_from_content_type(&content_type).to_string();
        let hashed_url = hashed_url_for(name, &sha256_short_hex, &ext);

        let entry = BundleEntry {
            name: name.to_string(),
            bytes,
            content_type,
            sha256_full_hex,
            sha256_short_hex,
            ext,
            hashed_url: hashed_url.clone(),
        };

        bundle_registry().insert(hashed_url.clone(), entry);
        name_index().insert(name.to_string(), hashed_url);

        Bundle {
            name: name.to_string(),
        }
    }

    /// Set the content-type. Re-keys the bundle's hashed URL (appends the extension
    /// derived from the content-type). Call BEFORE `.with_alias(...)` so aliases
    /// capture the final hashed URL.
    pub fn content_type(self, ct: &str) -> Self {
        let ext = ext_from_content_type(ct).to_string();

        let old_url = match name_index().get(&self.name) {
            Some(v) => v.value().clone(),
            None => return self, // unreachable; new() always inserts
        };

        // Remove the entry under the old key, mutate, reinsert under the new key.
        let mut entry = match bundle_registry().remove(&old_url) {
            Some((_, e)) => e,
            None => return self, // unreachable
        };
        entry.content_type = ct.to_string();
        entry.ext = ext.clone();
        let new_url = hashed_url_for(&entry.name, &entry.sha256_short_hex, &ext);
        entry.hashed_url = new_url.clone();
        bundle_registry().insert(new_url.clone(), entry);
        name_index().insert(self.name.clone(), new_url);

        self
    }

    /// Register a stable plain URL that 301-redirects to the current hashed URL.
    /// Multiple aliases per bundle are allowed; each call adds one entry.
    pub fn with_alias(self, alias_path: &str) -> Self {
        let target = match name_index().get(&self.name) {
            Some(v) => v.value().clone(),
            None => return self, // unreachable
        };
        alias_registry().insert(alias_path.to_string(), target);
        self
    }

    /// Return the current hashed URL (`/bundles/{name}.{sha8}.{ext}` or
    /// `/bundles/{name}.{sha8}` if the content-type has no known extension).
    pub fn hashed_url(&self) -> String {
        name_index()
            .get(&self.name)
            .map(|v| v.value().clone())
            .unwrap_or_default()
    }

    /// Dispatch a request to the bundle registry. Mount this as the handler for
    /// `/bundles/{filename}` and for each registered alias path.
    ///
    /// Order of checks (D-03):
    /// 1. Alias check → 301 redirect.
    /// 2. Bundle check → 304 fast-path on `If-None-Match` match, else 200 with bytes.
    /// 3. Otherwise → 404.
    pub fn serve(req: Request) -> HttpResponse {
        let path = req.path().to_string();
        let if_none_match = req.header("if-none-match").map(|s| s.to_string());
        serve_inner(&path, if_none_match.as_deref())
    }
}

// ── Dispatcher (private; pub(crate) so integration tests can bypass Request) ────

/// Dispatch by path + optional If-None-Match. Exposed at crate-visibility so
/// integration tests can call it directly without constructing a synthetic
/// `Request` (RESEARCH OQ #3 resolution).
pub(crate) fn serve_inner(path: &str, if_none_match: Option<&str>) -> HttpResponse {
    // Alias check first (D-03 ordering).
    if let Some(target) = alias_registry().get(path) {
        return HttpResponse::new()
            .status(301)
            .header("Location", target.value().clone());
    }

    // Bundle check.
    if let Some(entry) = bundle_registry().get(path) {
        let etag = format!("\"{}\"", entry.sha256_full_hex);
        if let Some(inm) = if_none_match {
            if inm == etag {
                return HttpResponse::new()
                    .status(304)
                    .header("ETag", etag)
                    .header("Cache-Control", "public, max-age=31536000, immutable");
            }
        }
        return HttpResponse::bytes(Bytes::from_static(entry.bytes))
            .header("Content-Type", entry.content_type.clone())
            .header("Cache-Control", "public, max-age=31536000, immutable")
            .header("ETag", etag);
    }

    // 404 fallback (defensive — caller is expected to route only /bundles/... here).
    HttpResponse::new()
        .status(404)
        .header("Content-Type", "text/plain")
}

// ── Integration-test access shim ───────────────────────────────────────

/// Doc-hidden wrapper around the crate-private `serve_inner` dispatcher for
/// integration tests.
///
/// Each `tests/*.rs` file is its own compilation unit and cannot see `pub(crate)`
/// items. This shim provides reachability without polluting the public crate API.
/// The `__test_internals` name signals "do not call from production code."
///
/// We expose a thin `pub fn` wrapper (rather than a `pub use`) because Rust's
/// visibility rules forbid `pub use` of a `pub(crate)` item — the re-export
/// cannot exceed the imported item's visibility. The wrapper is inlined.
#[doc(hidden)]
pub mod __test_internals {
    use ferro_rs::HttpResponse;

    /// Doc-hidden bridge to the crate-private dispatcher. Integration tests only.
    #[inline]
    pub fn serve_inner(path: &str, if_none_match: Option<&str>) -> HttpResponse {
        crate::serve_inner(path, if_none_match)
    }
}

// ── Test isolation helper (D-13) ───────────────────────────────────────

/// Clear all registries. Visible only under `#[cfg(test)]`. Call at the top of every
/// test that registers bundles to prevent process-global state leakage between tests
/// in the same binary.
#[cfg(test)]
pub(crate) fn reset() {
    if let Some(r) = BUNDLE_REGISTRY.get() {
        r.clear();
    }
    if let Some(r) = ALIAS_REGISTRY.get() {
        r.clear();
    }
    if let Some(r) = NAME_INDEX.get() {
        r.clear();
    }
}

// ── Unit tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        reset();
        let b = Bundle::new("test1", b"hello").content_type("text/plain");
        // SHA-256 of "hello" = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        // First 8 chars = 2cf24dba
        assert_eq!(b.hashed_url(), "/bundles/test1.2cf24dba.txt");
    }

    #[test]
    fn default_content_type_is_octet_stream() {
        reset();
        let b = Bundle::new("test2", b"x");
        let url = b.hashed_url();
        assert!(
            url.starts_with("/bundles/test2."),
            "expected /bundles/test2. prefix, got {url}"
        );
        assert!(
            !url.ends_with(".txt") && !url.ends_with(".js") && !url.ends_with(".css"),
            "default URL should not have a known extension; got {url}"
        );
        let suffix = url.strip_prefix("/bundles/test2.").unwrap();
        assert_eq!(suffix.len(), 8, "expected 8-char short hash; got {suffix}");
    }

    #[test]
    #[should_panic(expected = "duplicate")]
    fn duplicate_name_panics() {
        reset();
        Bundle::new("dup", b"a");
        Bundle::new("dup", b"a");
    }

    #[test]
    fn error_not_found_displays_message() {
        let e = Error::NotFound("/x".to_string());
        assert_eq!(e.to_string(), "bundle not found at path: /x");
    }

    #[test]
    fn error_duplicate_name_displays_message() {
        let e = Error::DuplicateName("dup".to_string());
        assert_eq!(
            e.to_string(),
            "duplicate bundle name: dup already registered"
        );
    }
}
