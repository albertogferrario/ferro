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

pub mod init;
pub mod middleware;

pub use init::{choice as lang_choice, init as lang_init, t, trans};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_returns_default_outside_scope() {
        // Outside any middleware scope, locale() should return "en"
        let result = locale();
        assert_eq!(result, "en");
    }

    #[tokio::test]
    async fn locale_returns_value_within_scope() {
        let ctx = locale_scope();
        {
            let mut guard = ctx.write().await;
            *guard = Some("fr".to_string());
        }

        let result = with_locale_scope(ctx, async { locale() }).await;
        assert_eq!(result, "fr");
    }

    #[tokio::test]
    async fn set_locale_normalizes_before_storing() {
        let ctx = locale_scope();

        let result = with_locale_scope(ctx, async {
            set_locale("en_US");
            locale()
        })
        .await;

        assert_eq!(result, "en-us");
    }

    #[tokio::test]
    async fn set_locale_normalizes_uppercase() {
        let ctx = locale_scope();

        let result = with_locale_scope(ctx, async {
            set_locale("PT-BR");
            locale()
        })
        .await;

        assert_eq!(result, "pt-br");
    }

    #[tokio::test]
    async fn locale_returns_default_when_not_set_in_scope() {
        let ctx = locale_scope();

        let result = with_locale_scope(ctx, async { locale() }).await;

        // No locale was set, and no Config registered, so falls back to "en"
        assert_eq!(result, "en");
    }
}
