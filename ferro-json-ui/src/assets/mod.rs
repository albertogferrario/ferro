//! Embedded static assets for ferro-json-ui.
//!
//! Served by the framework via the automatically-registered
//! `GET /_ferro/ferro-base.css` route. Embedded at compile time —
//! no runtime file I/O.

pub(crate) mod quill;

pub mod leaflet;

/// Pre-built Tailwind CSS covering every utility class emitted by
/// ferro-json-ui components.
///
/// Regenerate with `scripts/gen-ferro-base-css.sh` after adding or
/// modifying components that introduce new utility classes.
///
/// In release builds (no `dev-css` feature): static compile-time constant.
/// In dev builds (`dev-css` feature): use `ferro_base_css()` instead (disk read).
#[cfg(not(feature = "dev-css"))]
pub const FERRO_BASE_CSS: &str = include_str!("../../assets/ferro-base.css");

/// Returns the pre-built ferro-json-ui base CSS.
///
/// In release builds (no `dev-css` feature): returns the compile-time embedded
/// constant as a `Cow::Borrowed` — zero allocation, zero I/O.
///
/// In dev builds (`dev-css` feature): reads `assets/ferro-base.css` from disk
/// at call time so that a running Tailwind `--watch` process can update the CSS
/// without a Rust recompile. Panics if the file is unreadable.
pub fn ferro_base_css() -> std::borrow::Cow<'static, str> {
    #[cfg(not(feature = "dev-css"))]
    {
        std::borrow::Cow::Borrowed(FERRO_BASE_CSS)
    }
    #[cfg(feature = "dev-css")]
    {
        let css = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ferro-base.css"
        ))
        .expect("ferro-base.css must be readable in dev-css mode");
        std::borrow::Cow::Owned(css)
    }
}

/// Geist Sans Variable woff2. Served at `/_ferro/fonts/Geist-Variable.woff2`.
///
/// Source: vercel/geist-font v1.7.2, OFL-1.1 license.
pub const GEIST_SANS_WOFF2: &[u8] =
    include_bytes!("../../assets/fonts/Geist-Variable.woff2");

/// Geist Mono Variable woff2. Served at `/_ferro/fonts/GeistMono-Variable.woff2`.
///
/// Source: vercel/geist-font v1.7.2, OFL-1.1 license.
pub const GEIST_MONO_WOFF2: &[u8] =
    include_bytes!("../../assets/fonts/GeistMono-Variable.woff2");

/// OFL-1.1 license text for the Geist fonts.
///
/// Served at `/_ferro/fonts/OFL.txt` alongside the font files.
pub const GEIST_OFL_TXT: &[u8] = include_bytes!("../../assets/fonts/OFL.txt");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn ferro_base_css_non_empty() {
        let css = ferro_base_css();
        assert!(!css.is_empty(), "embedded CSS must not be empty");
        // include_str! guarantees valid UTF-8 (compile error otherwise),
        // so runtime validation is unnecessary. Smoke-check a class that
        // every ferro-json-ui page relies on:
        assert!(
            css.contains("flex"),
            "expected `flex` utility in generated CSS"
        );
    }

    #[test]
    fn geist_sans_woff2_non_empty() {
        assert!(
            !GEIST_SANS_WOFF2.is_empty(),
            "Geist Sans woff2 must not be empty"
        );
    }

    #[test]
    fn geist_mono_woff2_non_empty() {
        assert!(
            !GEIST_MONO_WOFF2.is_empty(),
            "Geist Mono woff2 must not be empty"
        );
    }

    #[test]
    fn geist_sans_woff2_magic_bytes() {
        // WOFF2 magic number: 'w' 'O' 'F' '2' (0x77 0x4F 0x46 0x32)
        assert_eq!(
            &GEIST_SANS_WOFF2[..4],
            &[0x77, 0x4F, 0x46, 0x32],
            "Geist Sans woff2 must start with wOF2 magic bytes"
        );
    }

    #[test]
    fn geist_mono_woff2_magic_bytes() {
        assert_eq!(
            &GEIST_MONO_WOFF2[..4],
            &[0x77, 0x4F, 0x46, 0x32],
            "Geist Mono woff2 must start with wOF2 magic bytes"
        );
    }

    #[test]
    fn ferro_base_css_contains_motion_duration_fallback() {
        let css = ferro_base_css();
        // SC1: v1 themes that omit --motion-duration-fast resolve via the fallback.
        assert!(
            css.contains("var(--motion-duration-fast,"),
            "expected motion-duration-fast fallback in generated CSS; run scripts/gen-ferro-base-css.sh"
        );
        // The duration utilities must exist as class rules — a theme-layer
        // variable emission alone satisfies the var() substring above without
        // any consumable utility being generated.
        for class in [".duration-fast{", ".duration-base{", ".duration-slow{"] {
            assert!(
                css.contains(class),
                "expected `{class}` utility rule in generated CSS; run scripts/gen-ferro-base-css.sh"
            );
        }
        // SC3: reduced-motion collapse survives regeneration. The collapse
        // declarations need !important — theme <style> tags are injected
        // after this stylesheet and redeclare the tokens on :root at equal
        // specificity, so normal declarations would lose on source order.
        assert!(
            css.contains("prefers-reduced-motion"),
            "expected prefers-reduced-motion block in generated CSS"
        );
        assert!(
            css.contains("--motion-duration-fast:.01ms!important"),
            "expected !important on the reduced-motion collapse; run scripts/gen-ferro-base-css.sh"
        );
    }

    #[test]
    fn ferro_base_css_ring_falls_back_to_primary() {
        let css = ferro_base_css();
        // 23-slot v1 themes never define --color-ring; focus rings must
        // resolve to the theme's primary color instead of an undefined var.
        assert!(
            css.contains("var(--color-ring,var(--color-primary))"),
            "expected --color-ring fallback to --color-primary in generated CSS; run scripts/gen-ferro-base-css.sh"
        );
    }
}
