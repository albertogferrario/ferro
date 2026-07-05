//! Shared interactive-base class fragments. Single source of truth for the
//! focus-visible ring, disabled treatment, and frequency-tiered motion, composed
//! into each component's class string so the interactive feel cannot drift.
//!
//! Every constant is a complete class literal in crate source so the Tailwind
//! `@source` scanner picks the utilities up; compose via `concat!`/`format!`
//! of these fragments — never build class names dynamically.

/// focus-visible ring from the `--color-ring` token (D-14). ring-2 + offset-2 baseline.
pub(crate) const FOCUS_RING: &str =
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2";

/// Fast tier (D-03): hover, toggles, controls, nav — high-frequency interactions.
pub(crate) const MOTION_FAST: &str = "transition-colors duration-fast ease-base";

/// Base tier (D-03): dropdowns, modals, toasts.
pub(crate) const MOTION_BASE: &str = "transition-opacity duration-base ease-base";

/// Uniform disabled treatment for native controls (D-16).
pub(crate) const DISABLED_BASE: &str = "disabled:opacity-50 disabled:pointer-events-none";

/// Toast tone treatments: translucent tone tint (70% alpha) paired with a
/// `backdrop-blur-md` on the toast shell. Single source of truth shared by
/// the SSR `render_toast` and the JS runtime's `VARIANT_CLASSES` — lockstep
/// is asserted in `runtime::tests::toast_tone_classes_match_ssr`.
pub(crate) const TOAST_TONE_NEUTRAL: &str = "bg-primary/70 text-primary-foreground";
/// Success toast tone — see [`TOAST_TONE_NEUTRAL`].
pub(crate) const TOAST_TONE_SUCCESS: &str = "bg-success/70 text-primary-foreground";
/// Warning toast tone — see [`TOAST_TONE_NEUTRAL`].
pub(crate) const TOAST_TONE_WARNING: &str = "bg-warning/70 text-primary-foreground";
/// Destructive toast tone — see [`TOAST_TONE_NEUTRAL`].
pub(crate) const TOAST_TONE_DESTRUCTIVE: &str = "bg-destructive/70 text-primary-foreground";

/// Fast-tier interactive base = fast motion + focus ring, for buttons/links/nav.
pub(crate) const INTERACTIVE_BASE: &str = concat!(
    "transition-colors duration-fast ease-base ",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
);

/// Touch-manipulation scroll optimisation for tap targets. Full literal.
pub const TOUCH_ACTION: &str = "touch-manipulation";

/// Minimum 44px hit target for interactive elements (WCAG 2.5.5). Full literal.
pub const HIT_TARGET_MIN: &str = "min-h-[44px] min-w-[44px]";

/// Enlarged 56px hit target for Numpad keys (Phase 256 consumer). Full literal.
pub const HIT_TARGET_NUMPAD: &str = "min-h-[56px] min-w-[56px]";

/// Press-state feedback for tap surfaces: scale-down + border tint.
/// Token-compliant (`active:bg-border` = `--color-border`); no raw palette class.
pub const PRESS_ACTIVE: &str = "active:scale-95 active:bg-border";

/// Prevents scroll bounce from escaping a pane boundary. Full literal.
pub const OVERSCROLL_CONTAIN: &str = "overscroll-contain";

/// Removes the default iOS tap-highlight rectangle on touch targets.
/// Backed by `@utility pos-tap-highlight` in input.css (Path B — guaranteed CSS).
pub const TAP_HIGHLIGHT: &str = "pos-tap-highlight";

#[cfg(test)]
mod tests {
    use super::*;

    /// `INTERACTIVE_BASE` must be exactly the fast-tier motion + focus ring —
    /// editing one fragment without the other is drift.
    #[test]
    fn interactive_base_is_motion_fast_plus_focus_ring() {
        assert_eq!(INTERACTIVE_BASE, format!("{MOTION_FAST} {FOCUS_RING}"));
    }

    /// Every fragment sources the Phase 250 token vocabulary, never raw values.
    #[test]
    fn fragments_use_token_utilities() {
        assert!(FOCUS_RING.contains("focus-visible:ring-ring"));
        assert!(MOTION_FAST.contains("duration-fast") && MOTION_FAST.contains("ease-base"));
        assert!(MOTION_BASE.contains("duration-base") && MOTION_BASE.contains("ease-base"));
        assert!(DISABLED_BASE.contains("disabled:pointer-events-none"));
    }

    /// D-07: no render file (outside classes.rs) may inline the touch literals.
    /// Auto-covers Phase 256 render files via read_dir — no manual re-enrollment.
    #[test]
    fn render_functions_use_constants_not_literals() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let render_dir = std::path::Path::new(&manifest_dir).join("src/render");
        let guarded = [
            "touch-manipulation",
            "min-h-[44px] min-w-[44px]",
            "min-h-[56px] min-w-[56px]",
        ];
        for entry in std::fs::read_dir(&render_dir).expect("src/render readable") {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            if filename == "classes.rs" {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            for lit in &guarded {
                assert!(
                    !source.contains(lit),
                    "{filename}: raw POS literal {lit:?} — import from render::classes instead"
                );
            }
        }
    }

    #[test]
    fn touch_constants_are_full_literals_and_token_compliant() {
        assert_eq!(TOUCH_ACTION, "touch-manipulation");
        assert_eq!(HIT_TARGET_MIN, "min-h-[44px] min-w-[44px]");
        assert_eq!(HIT_TARGET_NUMPAD, "min-h-[56px] min-w-[56px]");
        assert_eq!(OVERSCROLL_CONTAIN, "overscroll-contain");
        assert_eq!(TAP_HIGHLIGHT, "pos-tap-highlight");
        assert!(PRESS_ACTIVE.contains("active:scale-95"));
        assert!(PRESS_ACTIVE.contains("active:bg-border"));
        for raw in ["red-", "blue-", "orange-", "zinc-", "gray-", "slate-"] {
            assert!(
                !PRESS_ACTIVE.contains(raw),
                "raw palette class in PRESS_ACTIVE"
            );
        }
    }
}
