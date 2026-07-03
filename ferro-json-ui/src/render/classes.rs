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

/// Fast-tier interactive base = fast motion + focus ring, for buttons/links/nav.
pub(crate) const INTERACTIVE_BASE: &str = concat!(
    "transition-colors duration-fast ease-base ",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
);

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
}
