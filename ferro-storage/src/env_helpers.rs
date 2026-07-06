//! Crate-local env-var helpers shared across `config`, `cdn`, `drivers`, and `facade`.
//!
//! These keep the deprecation cushion uniform across every `ferro-storage` surface
//! that reads provider-specific env vars: primary name first, then ordered legacy
//! aliases, with one `tracing::warn!` naming the deprecated var (never its value)
//! on the first hit.

/// Read `primary`; if unset, try each `alias` in order, emitting one
/// `tracing::warn!` naming the deprecated var (never its value) on first hit.
pub(crate) fn env_with_fallback(primary: &str, aliases: &[&str]) -> Option<String> {
    if let Ok(val) = std::env::var(primary) {
        return Some(val);
    }
    for alias in aliases {
        if let Ok(val) = std::env::var(alias) {
            tracing::warn!("{alias} is deprecated; use {primary} instead");
            return Some(val);
        }
    }
    None
}
