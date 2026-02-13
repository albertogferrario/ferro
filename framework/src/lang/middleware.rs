//! Locale detection middleware.

use crate::config::Config;
use crate::http::Response;
use crate::middleware::{Middleware, Next};
use crate::Request;
use async_trait::async_trait;
use ferro_lang::{normalize_locale, LangConfig};

use super::{locale_scope, with_locale_scope};

/// Middleware that detects the request locale and sets it in task-local context.
///
/// Detection priority:
/// 1. `?locale=xx` query parameter (explicit override)
/// 2. `Accept-Language` header (first language tag)
/// 3. `LangConfig.locale` default
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::{global_middleware, LangMiddleware};
///
/// pub fn register() {
///     global_middleware!(LangMiddleware);
/// }
/// ```
pub struct LangMiddleware;

#[async_trait]
impl Middleware for LangMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let config = Config::get::<LangConfig>().unwrap_or_else(LangConfig::default);

        let detected = detect_locale(&request, &config);

        let ctx = locale_scope();
        {
            let mut guard = ctx.write().await;
            *guard = Some(detected);
        }

        with_locale_scope(ctx, async { next(request).await }).await
    }
}

/// Detect locale from request with priority: query param > Accept-Language > config default.
fn detect_locale(request: &Request, config: &LangConfig) -> String {
    // 1. Explicit query parameter override
    if let Some(locale) = request.query("locale") {
        if !locale.is_empty() {
            return normalize_locale(&locale);
        }
    }

    // 2. Accept-Language header (first language tag)
    if let Some(accept) = request.header("accept-language") {
        if let Some(locale) = parse_accept_language(accept) {
            return locale;
        }
    }

    // 3. Config default
    normalize_locale(&config.locale)
}

/// Parse Accept-Language header and return the first (highest priority) language tag.
///
/// Takes the first entry before any comma, strips quality suffix (`;q=...`),
/// and normalizes via `normalize_locale()`.
///
/// Example: `"en-US,en;q=0.9,fr;q=0.8"` returns `"en-us"`.
fn parse_accept_language(header: &str) -> Option<String> {
    let first = header.split(',').next()?.trim();
    if first.is_empty() {
        return None;
    }
    // Strip quality value suffix (e.g. ";q=0.9")
    let lang = first.split(';').next()?.trim();
    if lang.is_empty() || lang == "*" {
        return None;
    }
    Some(normalize_locale(lang))
}
