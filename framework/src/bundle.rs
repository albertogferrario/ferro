//! Content-hashed static asset bundles.
//!
//! Re-exports the leaf `ferro-bundle` crate and adds the framework-aware
//! [`bundle::serve`](crate::bundle::serve) adapter that maps a [`crate::Request`] to an [`crate::HttpResponse`].
//! Register bundles at boot (or lazily via the `asset!()` macro), then mount
//! [`bundle::serve`](crate::bundle::serve) on `/bundles/{filename}` and on each registered alias path.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro::bundle::{Bundle, serve as bundle_serve};
//!
//! // Boot-time registration.
//! Bundle::new("app_js", include_bytes!("../assets/app.js"))
//!     .content_type("application/javascript");
//!
//! // In your route handler:
//! pub async fn bundle_handler(req: Request) -> Response {
//!     Ok(bundle_serve(&req))
//! }
//! ```

pub use ferro_bundle::{mime_from_ext, Bundle, BundleResponse};

use crate::{HttpResponse, Request};

/// Dispatch a request to the bundle registry, returning a framework [`HttpResponse`].
///
/// Reads `req.path()` and the `if-none-match` header, delegates to
/// [`ferro_bundle::serve_path`], then maps the returned [`BundleResponse`]
/// (status / headers / body) into a framework [`HttpResponse`]. Mount this
/// function as the handler for `/bundles/{filename}` and each registered alias path.
pub fn serve(req: &Request) -> HttpResponse {
    let path = req.path().to_string();
    let if_none_match = req.header("if-none-match").map(|s| s.to_string());
    let resp = ferro_bundle::serve_path(&path, if_none_match.as_deref());

    let mut out = HttpResponse::bytes(resp.body_bytes().clone()).status(resp.status_code());
    for (name, value) in resp.headers() {
        out = out.header(name.clone(), value.clone());
    }
    out
}
