//! Per-request locale context for Ferro framework.
//!
//! Provides task-local storage for the current locale, allowing handlers
//! to call translation functions without passing locale explicitly.
//!
//! The [`LangMiddleware`] sets the locale per-request from `Accept-Language`
//! header or query parameter override. Handlers read via [`locale()`] and
//! can override with [`set_locale()`].
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{locale, set_locale, LangMiddleware};
//!
//! // In bootstrap.rs
//! global_middleware!(LangMiddleware);
//!
//! // In a handler — locale() returns the detected locale
//! let current = locale(); // e.g. "en-us"
//!
//! // Override for this request
//! set_locale("fr");
//! ```

pub mod middleware;

pub use middleware::LangMiddleware;

use crate::config::Config;
use ferro_lang::{normalize_locale, LangConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

tokio::task_local! {
    static LOCALE_CONTEXT: Arc<RwLock<Option<String>>>;
}

/// Get the current request locale.
///
/// Reads the task-local locale set by [`LangMiddleware`]. If called outside
/// middleware scope, falls back to `LangConfig` default locale, then `"en"`.
///
/// Always returns a value — there is always a reasonable default.
pub fn locale() -> String {
    LOCALE_CONTEXT
        .try_with(|ctx| ctx.try_read().ok().and_then(|guard| guard.clone()))
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            Config::get::<LangConfig>()
                .map(|c| c.locale)
                .unwrap_or_else(|| "en".to_string())
        })
}

/// Set the locale for the current request.
///
/// Normalizes the input (e.g. `"en_US"` becomes `"en-us"`) before storing.
/// Has no effect if called outside [`LangMiddleware`] scope.
pub fn set_locale(locale: impl Into<String>) {
    let normalized = normalize_locale(&locale.into());
    let result = LOCALE_CONTEXT.try_with(|ctx| {
        if let Ok(mut guard) = ctx.try_write() {
            *guard = Some(normalized);
        }
    });
    if result.is_err() {
        eprintln!("[ferro::lang] set_locale called outside LangMiddleware scope");
    }
}

/// Create a locale context for use with `LOCALE_CONTEXT.scope()`.
///
/// Returns the `Arc<RwLock<Option<String>>>` so the middleware controls
/// the scope lifetime.
pub(crate) fn locale_scope() -> Arc<RwLock<Option<String>>> {
    Arc::new(RwLock::new(None))
}

/// Run an async block within a locale context scope.
///
/// Used by [`LangMiddleware`] to make [`locale()`] and [`set_locale()`]
/// available during request processing.
pub(crate) async fn with_locale_scope<F, R>(ctx: Arc<RwLock<Option<String>>>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    LOCALE_CONTEXT.scope(ctx, f).await
}
