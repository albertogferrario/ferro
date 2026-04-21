//! Embedded static assets for ferro-json-ui.
//!
//! Served by the framework via the automatically-registered
//! `GET /_ferro/ferro-base.css` route. Embedded at compile time —
//! no runtime file I/O.

/// Pre-built Tailwind CSS covering every utility class emitted by
/// ferro-json-ui components.
///
/// Regenerate with `scripts/gen-ferro-base-css.sh` after adding or
/// modifying components that introduce new utility classes.
pub const FERRO_BASE_CSS: &str = include_str!("../assets/ferro-base.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn ferro_base_css_non_empty() {
        assert!(!FERRO_BASE_CSS.is_empty(), "embedded CSS must not be empty");
        // include_str! guarantees valid UTF-8 (compile error otherwise),
        // so runtime validation is unnecessary. Smoke-check a class that
        // every ferro-json-ui page relies on:
        assert!(
            FERRO_BASE_CSS.contains("flex"),
            "expected `flex` utility in generated CSS"
        );
    }
}
