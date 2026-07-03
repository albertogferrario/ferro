//! Embedded static assets for ferro-json-ui.
//!
//! Served by the framework via the automatically-registered
//! `GET /_ferro/ferro-base.css` route. Embedded at compile time —
//! no runtime file I/O.

pub(crate) mod quill;

/// Pre-built Tailwind CSS covering every utility class emitted by
/// ferro-json-ui components.
///
/// Regenerate with `scripts/gen-ferro-base-css.sh` after adding or
/// modifying components that introduce new utility classes.
pub const FERRO_BASE_CSS: &str = include_str!("../../assets/ferro-base.css");

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

    #[test]
    fn ferro_base_css_contains_motion_duration_fallback() {
        // SC1: v1 themes that omit --motion-duration-fast resolve via the fallback.
        assert!(
            FERRO_BASE_CSS.contains("var(--motion-duration-fast,"),
            "expected motion-duration-fast fallback in generated CSS; run scripts/gen-ferro-base-css.sh"
        );
        // The duration utilities must exist as class rules — a theme-layer
        // variable emission alone satisfies the var() substring above without
        // any consumable utility being generated.
        for class in [".duration-fast{", ".duration-base{", ".duration-slow{"] {
            assert!(
                FERRO_BASE_CSS.contains(class),
                "expected `{class}` utility rule in generated CSS; run scripts/gen-ferro-base-css.sh"
            );
        }
        // SC3: reduced-motion collapse survives regeneration. The collapse
        // declarations need !important — theme <style> tags are injected
        // after this stylesheet and redeclare the tokens on :root at equal
        // specificity, so normal declarations would lose on source order.
        assert!(
            FERRO_BASE_CSS.contains("prefers-reduced-motion"),
            "expected prefers-reduced-motion block in generated CSS"
        );
        assert!(
            FERRO_BASE_CSS.contains("--motion-duration-fast:.01ms!important"),
            "expected !important on the reduced-motion collapse; run scripts/gen-ferro-base-css.sh"
        );
    }
}
