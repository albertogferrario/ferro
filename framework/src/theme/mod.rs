//! Semantic theme support for Ferro framework.
//!
//! Provides per-request theme selection via [`ThemeMiddleware`] and task-local
//! storage via [`current_theme()`].
//!
//! # Overview
//!
//! - [`ThemeMiddleware`] — resolver chain middleware, always falls back to default theme
//! - [`ThemeResolver`] — trait for pluggable theme resolution strategies
//! - [`TenantThemeResolver`] — reads theme from `TenantContext.plan` with moka cache
//! - [`HeaderThemeResolver`] — reads theme from `X-Theme` header with moka cache
//! - [`DefaultResolver`] — always returns the configured default theme
//! - [`current_theme()`] — reads the active theme from task-local storage

pub(crate) mod context;
mod middleware;
mod resolver;

pub use context::current_theme;
pub use middleware::ThemeMiddleware;
pub use resolver::{DefaultResolver, HeaderThemeResolver, TenantThemeResolver, ThemeResolver};
