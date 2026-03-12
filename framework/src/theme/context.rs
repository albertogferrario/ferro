//! Task-local theme context for Ferro framework.
//!
//! Provides task-local storage for the current theme, allowing handlers
//! and the JSON-UI render pipeline to call [`current_theme()`] without
//! passing theme data explicitly.
//!
//! The `ThemeMiddleware` sets the theme per-request. JSON-UI rendering
//! reads via [`current_theme()`] to inject the active theme's CSS.

use ferro_theme::Theme;
use std::sync::Arc;
use tokio::sync::RwLock;

tokio::task_local! {
    static CURRENT_THEME: Arc<RwLock<Option<Arc<Theme>>>>;
}

/// Get the current theme.
///
/// Returns the active theme when called within a `ThemeMiddleware` scope.
/// Returns `None` if called outside middleware scope or if no theme was
/// resolved (which should not happen — `ThemeMiddleware` always falls back
/// to the default theme).
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::theme::current_theme;
///
/// if let Some(theme) = current_theme() {
///     println!("css length: {}", theme.css.len());
/// }
/// ```
pub fn current_theme() -> Option<Arc<Theme>> {
    CURRENT_THEME
        .try_with(|ctx| ctx.try_read().ok().and_then(|guard| guard.clone()))
        .ok()
        .flatten()
}

/// Create a theme scope for use with `CURRENT_THEME.scope()`.
///
/// Returns `Arc<RwLock<Option<Arc<Theme>>>>` initialized to `None`.
/// The middleware controls the scope lifetime.
pub(crate) fn theme_scope() -> Arc<RwLock<Option<Arc<Theme>>>> {
    Arc::new(RwLock::new(None))
}

/// Run an async block within a theme context scope.
///
/// Used by `ThemeMiddleware` to make [`current_theme()`] available
/// during request processing.
pub(crate) async fn with_theme_scope<F, R>(ctx: Arc<RwLock<Option<Arc<Theme>>>>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    CURRENT_THEME.scope(ctx, f).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_theme::Theme;

    fn make_theme() -> Arc<Theme> {
        Arc::new(Theme::default_theme())
    }

    // Test: current_theme() returns None outside scope
    #[test]
    fn current_theme_returns_none_outside_scope() {
        let result = current_theme();
        assert!(result.is_none());
    }

    // Test: current_theme() returns Some(theme) within with_theme_scope
    #[tokio::test]
    async fn current_theme_returns_some_within_scope() {
        let scope = theme_scope();
        {
            let mut guard = scope.write().await;
            *guard = Some(make_theme());
        }
        let result = with_theme_scope(scope, async { current_theme() }).await;
        assert!(result.is_some());
        assert!(result.unwrap().css.contains("--color-primary"));
    }

    // Test: theme_scope() creates Arc<RwLock<None>>
    #[test]
    fn theme_scope_creates_arc_rwlock_initialized_to_none() {
        let scope = theme_scope();
        let guard = scope.try_read().unwrap();
        assert!(guard.is_none());
    }

    // Test: with_theme_scope provides access inside, None outside
    #[tokio::test]
    async fn with_theme_scope_returns_none_outside_and_some_inside() {
        let scope = theme_scope();
        {
            let mut guard = scope.write().await;
            *guard = Some(make_theme());
        }

        // Inside scope: Some
        let inside = with_theme_scope(scope, async { current_theme() }).await;
        assert!(inside.is_some());

        // Outside scope: None
        let outside = current_theme();
        assert!(outside.is_none());
    }
}
