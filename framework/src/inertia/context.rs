//! Inertia.js integration - async-safe implementation.
//!
//! This module provides the main `Inertia` struct for rendering Inertia responses.
//! It wraps the framework-agnostic `ferro-inertia` crate with Ferro-specific features.

use crate::csrf::csrf_token;
use crate::http::{HttpResponse, Request};
use crate::Response;
use ferro_inertia::{InertiaConfig, InertiaRequest as InertiaRequestTrait};
use serde::Serialize;
use std::collections::HashMap;

// Re-export InertiaShared from ferro-inertia
pub use ferro_inertia::InertiaShared;

/// Implement the framework-agnostic InertiaRequest trait for Ferro's Request type.
impl InertiaRequestTrait for Request {
    fn inertia_header(&self, name: &str) -> Option<&str> {
        self.header(name)
    }

    fn path(&self) -> &str {
        Request::path(self)
    }
}

/// Saved Inertia context for use after consuming the Request.
///
/// Use this when you need to call `req.input()` (which consumes the request)
/// but still need to render Inertia error responses.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::{Inertia, Request, Response, SavedInertiaContext};
///
/// pub async fn login(req: Request) -> Response {
///     // Save Inertia context before consuming request
///     let ctx = SavedInertiaContext::from(&req);
///
///     // This consumes the request
///     let form: LoginForm = req.input().await?;
///
///     // Use saved context for error responses
///     if let Err(errors) = form.validate() {
///         return Inertia::render(&ctx, "auth/Login", LoginProps { errors });
///     }
///
///     // ...
/// }
/// ```
#[derive(Clone, Debug)]
pub struct SavedInertiaContext {
    path: String,
    headers: HashMap<String, String>,
}

impl SavedInertiaContext {
    /// Create a new SavedInertiaContext by capturing data from a Request.
    pub fn new(req: &Request) -> Self {
        let mut headers = HashMap::new();

        // Capture Inertia-relevant headers
        for name in &[
            "X-Inertia",
            "X-Inertia-Version",
            "X-Inertia-Partial-Data",
            "X-Inertia-Partial-Component",
        ] {
            if let Some(value) = req.header(name) {
                headers.insert(name.to_string(), value.to_string());
            }
        }

        Self {
            path: req.path().to_string(),
            headers,
        }
    }
}

impl From<&Request> for SavedInertiaContext {
    fn from(req: &Request) -> Self {
        Self::new(req)
    }
}

impl InertiaRequestTrait for SavedInertiaContext {
    fn inertia_header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|s| s.as_str())
    }

    fn path(&self) -> &str {
        &self.path
    }
}

/// Main Inertia integration struct for Ferro framework.
///
/// Provides methods for rendering Inertia responses in an async-safe manner.
/// All state is derived from the Request, not thread-local storage.
pub struct Inertia;

impl Inertia {
    /// Render an Inertia response.
    ///
    /// This is the primary method for returning Inertia responses from controllers.
    /// It automatically:
    /// - Detects XHR vs initial page load
    /// - Merges shared props from middleware
    /// - Filters props for partial reloads
    /// - Includes CSRF token in HTML responses
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::{Inertia, Request, Response};
    ///
    /// pub async fn index(req: Request) -> Response {
    ///     Inertia::render(&req, "Home", HomeProps {
    ///         title: "Welcome".into(),
    ///     })
    /// }
    /// ```
    pub fn render<P: Serialize>(req: &Request, component: &str, props: P) -> Response {
        Self::render_with_config(
            req,
            component,
            props,
            crate::inertia::global::get_inertia_config(),
        )
    }

    /// Render an Inertia response with custom configuration.
    pub fn render_with_config<P: Serialize>(
        req: &Request,
        component: &str,
        props: P,
        config: InertiaConfig,
    ) -> Response {
        // Get shared props from middleware (if set)
        let shared = req.get::<InertiaShared>();

        // Get CSRF token for HTML responses
        let csrf = csrf_token().unwrap_or_default();

        // Build shared props with CSRF included
        let effective_shared = if let Some(existing) = shared {
            // Clone and add CSRF if not already set
            let mut shared_clone = existing.clone();
            if shared_clone.csrf.is_none() {
                shared_clone.csrf = Some(csrf.clone());
            }
            Some(shared_clone)
        } else {
            Some(InertiaShared::new().csrf(csrf.clone()))
        };

        // Use ferro-inertia for the core rendering logic
        let http_response = ferro_inertia::Inertia::render_with_options(
            req,
            component,
            props,
            effective_shared.as_ref(),
            config,
        );

        // Convert InertiaHttpResponse to Ferro's Response
        Ok(Self::convert_response(http_response))
    }

    /// Render an Inertia response using a saved context.
    ///
    /// Use this when you've already consumed the Request (e.g., via `req.input()`)
    /// but still need to render an Inertia response (typically for validation errors).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::{Inertia, Request, Response, SavedInertiaContext};
    ///
    /// pub async fn login(req: Request) -> Response {
    ///     let ctx = SavedInertiaContext::from(&req);
    ///     let form: LoginForm = req.input().await?;
    ///
    ///     if let Err(errors) = form.validate() {
    ///         return Inertia::render_ctx(&ctx, "auth/Login", LoginProps { errors });
    ///     }
    ///     // ...
    /// }
    /// ```
    pub fn render_ctx<P: Serialize>(
        ctx: &SavedInertiaContext,
        component: &str,
        props: P,
    ) -> Response {
        let csrf = csrf_token().unwrap_or_default();
        let shared = InertiaShared::new().csrf(csrf);

        let http_response = ferro_inertia::Inertia::render_with_options(
            ctx,
            component,
            props,
            Some(&shared),
            crate::inertia::global::get_inertia_config(),
        );

        Ok(Self::convert_response(http_response))
    }

    /// Convert an InertiaHttpResponse to Ferro's HttpResponse.
    fn convert_response(inertia_response: ferro_inertia::InertiaHttpResponse) -> HttpResponse {
        let mut response = HttpResponse::new()
            .header("Content-Type", inertia_response.content_type)
            .set_body(inertia_response.body)
            .status(inertia_response.status);

        for (name, value) in inertia_response.headers {
            response = response.header(name, value);
        }

        response
    }

    /// Check if the current request is an Inertia XHR request.
    pub fn is_inertia_request(req: &Request) -> bool {
        req.is_inertia()
    }

    /// Get the current URL from the request.
    pub fn current_url(req: &Request) -> String {
        req.path().to_string()
    }

    /// Check for version mismatch and return 409 Conflict if needed.
    ///
    /// Call this in middleware to handle asset version changes.
    pub fn check_version(
        req: &Request,
        current_version: &str,
        redirect_url: &str,
    ) -> Option<Response> {
        ferro_inertia::Inertia::check_version(req, current_version, redirect_url)
            .map(|http_response| Ok(Self::convert_response(http_response)))
    }

    /// Create an Inertia-aware redirect.
    ///
    /// This properly handles the Inertia protocol:
    /// - For POST/PUT/PATCH/DELETE requests, uses 303 status to force GET
    /// - Includes X-Inertia header for Inertia XHR requests
    /// - Falls back to standard 302 for non-Inertia requests
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::{Inertia, Request, Response};
    ///
    /// pub async fn login(req: Request) -> Response {
    ///     // ... validation and auth logic ...
    ///     Inertia::redirect(&req, "/dashboard")
    /// }
    /// ```
    pub fn redirect(req: &Request, path: impl Into<String>) -> Response {
        let url = path.into();
        let is_inertia = req.is_inertia();
        let is_post_like = matches!(req.method().as_str(), "POST" | "PUT" | "PATCH" | "DELETE");

        if is_inertia {
            // 303 See Other forces browser to GET the redirect location
            let status = if is_post_like { 303 } else { 302 };
            Ok(HttpResponse::new()
                .status(status)
                .header("X-Inertia", "true")
                .header("Location", url))
        } else {
            // Standard redirect for non-Inertia requests
            Ok(HttpResponse::new().status(302).header("Location", url))
        }
    }

    /// Create an Inertia-aware redirect using saved context.
    ///
    /// Use when you've consumed the Request but need to redirect.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::{Inertia, Request, Response, SavedInertiaContext};
    ///
    /// pub async fn store(req: Request) -> Response {
    ///     let ctx = SavedInertiaContext::from(&req);
    ///     let form: CreateForm = req.input().await?;
    ///
    ///     // ... create record ...
    ///
    ///     Inertia::redirect_ctx(&ctx, "/items")
    /// }
    /// ```
    pub fn redirect_ctx(ctx: &SavedInertiaContext, path: impl Into<String>) -> Response {
        let url = path.into();
        let is_inertia = ctx.headers.contains_key("X-Inertia");

        // When using saved context, we assume POST-like (form submissions)
        // because that's the common case for needing SavedInertiaContext
        if is_inertia {
            Ok(HttpResponse::new()
                .status(303)
                .header("X-Inertia", "true")
                .header("Location", url))
        } else {
            Ok(HttpResponse::new().status(302).header("Location", url))
        }
    }
}

// Keep deprecated InertiaContext for backward compatibility during migration
#[deprecated(
    since = "0.2.0",
    note = "Use Inertia::render() instead - thread-local storage is async-unsafe"
)]
/// Deprecated thread-local Inertia context (use `Inertia::render()` instead).
pub struct InertiaContext;

#[allow(deprecated)]
impl InertiaContext {
    /// No-op — kept for compilation compatibility.
    #[deprecated(note = "Use Inertia::render() instead")]
    pub fn set(_ctx: InertiaContextData) {
        // No-op - kept for compilation compatibility during migration
    }

    /// Always returns false — kept for compilation compatibility.
    #[deprecated(note = "Use Inertia::is_inertia_request(&req) instead")]
    pub fn is_inertia_request() -> bool {
        false
    }

    /// Returns empty string — kept for compilation compatibility.
    #[deprecated(note = "Use req.path() instead")]
    pub fn current_path() -> String {
        String::new()
    }

    /// No-op — kept for compilation compatibility.
    #[deprecated(note = "No longer needed")]
    pub fn clear() {
        // No-op
    }

    /// Always returns None — kept for compilation compatibility.
    #[deprecated(note = "Use req methods instead")]
    pub fn get() -> Option<InertiaContextData> {
        None
    }
}

/// Legacy context data - kept for migration compatibility.
#[deprecated(since = "0.2.0", note = "Use Request methods instead")]
#[derive(Clone, Default)]
pub struct InertiaContextData {
    /// Request path.
    pub path: String,
    /// Whether the request is an Inertia request.
    pub is_inertia: bool,
    /// Asset version for cache busting.
    pub version: Option<String>,
}
