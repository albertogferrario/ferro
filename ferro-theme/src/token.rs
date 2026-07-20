//! Fixed semantic token vocabulary for ferro-theme/v2.
//!
//! Defines ~30 semantic slots that every theme must provide. Tokens are
//! CSS custom properties resolved by the Tailwind v4 `@theme` directive.
//! Components reference them as utility classes (`bg-primary`, `text-surface`, etc.).

// Surface tokens — structural background hierarchy
/// Background of the page.
pub const TOKEN_BACKGROUND: &str = "--color-background";
/// Raised surface above background (panels, sidebars).
pub const TOKEN_SURFACE: &str = "--color-surface";
/// Card background (further raised above surface).
pub const TOKEN_CARD: &str = "--color-card";
/// Default border color.
pub const TOKEN_BORDER: &str = "--color-border";
/// Primary text color.
pub const TOKEN_TEXT: &str = "--color-text";
/// Muted/secondary text color.
pub const TOKEN_TEXT_MUTED: &str = "--color-text-muted";

// Role tokens — semantic color roles
/// Primary action color (buttons, links, highlights).
pub const TOKEN_PRIMARY: &str = "--color-primary";
/// Foreground color on primary backgrounds.
pub const TOKEN_PRIMARY_FOREGROUND: &str = "--color-primary-foreground";
/// Secondary action color.
pub const TOKEN_SECONDARY: &str = "--color-secondary";
/// Foreground color on secondary backgrounds.
pub const TOKEN_SECONDARY_FOREGROUND: &str = "--color-secondary-foreground";
/// Accent color for decorative highlights.
pub const TOKEN_ACCENT: &str = "--color-accent";
/// Destructive / danger actions.
pub const TOKEN_DESTRUCTIVE: &str = "--color-destructive";
/// Success / confirmation state.
pub const TOKEN_SUCCESS: &str = "--color-success";
/// Warning / caution state.
pub const TOKEN_WARNING: &str = "--color-warning";

// Shape tokens — border radius scale
/// Extra-small border radius.
pub const TOKEN_RADIUS_SM: &str = "--radius-sm";
/// Medium border radius (default for most elements).
pub const TOKEN_RADIUS_MD: &str = "--radius-md";
/// Large border radius (cards, modals).
pub const TOKEN_RADIUS_LG: &str = "--radius-lg";
/// Full (pill) border radius.
pub const TOKEN_RADIUS_FULL: &str = "--radius-full";

// Shadow tokens — elevation scale
/// Subtle shadow (inputs, small cards).
pub const TOKEN_SHADOW_SM: &str = "--shadow-sm";
/// Medium shadow (floating panels).
pub const TOKEN_SHADOW_MD: &str = "--shadow-md";
/// Large shadow (modals, popovers).
pub const TOKEN_SHADOW_LG: &str = "--shadow-lg";

// Typography tokens — font family scale only; Tailwind size scale stays as-is
/// Sans-serif font stack.
pub const TOKEN_FONT_SANS: &str = "--font-sans";
/// Monospace font stack.
pub const TOKEN_FONT_MONO: &str = "--font-mono";

// Density token — base spacing unit
/// Base spacing unit; all spacing utilities resolve as calc(var(--spacing) * N).
pub const TOKEN_SPACING: &str = "--spacing";

// Motion tokens — frequency-tiered transition discipline
/// Fast transition duration (micro-interactions: hover, toggles). Default: 120ms.
pub const TOKEN_MOTION_DURATION_FAST: &str = "--motion-duration-fast";
/// Base transition duration (dropdowns, modals, toasts). Default: 220ms.
pub const TOKEN_MOTION_DURATION_BASE: &str = "--motion-duration-base";
/// Slow transition duration (drawers, page-level reveals). Default: 320ms.
pub const TOKEN_MOTION_DURATION_SLOW: &str = "--motion-duration-slow";
/// Standard easing curve (calm, settled, no bounce).
pub const TOKEN_MOTION_EASE: &str = "--motion-ease";

// Focus token — uniform keyboard-navigation ring
/// Focus-visible ring color for interactive components.
pub const TOKEN_COLOR_RING: &str = "--color-ring";

// Display font token
/// Display/heading font family; defaults to var(--font-sans).
pub const TOKEN_FONT_DISPLAY: &str = "--font-display";

// Type-scale tokens — v2 semantic size and weight scale
/// Display text size (headings, page titles). Default: 1.75rem (28px).
pub const TOKEN_TEXT_DISPLAY_SIZE: &str = "--text-display-size";
/// Display text weight. Default: 600.
pub const TOKEN_TEXT_DISPLAY_WEIGHT: &str = "--text-display-weight";
/// Section label text size (group headers, card titles). Default: 0.9375rem (15px).
pub const TOKEN_TEXT_SECTION_SIZE: &str = "--text-section-size";
/// Section label text weight. Default: 600.
pub const TOKEN_TEXT_SECTION_WEIGHT: &str = "--text-section-weight";
/// Body text size (default content). Default: 0.875rem (14px).
pub const TOKEN_TEXT_BODY_SIZE: &str = "--text-body-size";
/// Body text weight. Default: 400.
pub const TOKEN_TEXT_BODY_WEIGHT: &str = "--text-body-weight";
/// Meta text size (secondary labels, timestamps). Default: 0.8125rem (13px).
pub const TOKEN_TEXT_META_SIZE: &str = "--text-meta-size";
/// Meta text weight. Default: 400.
pub const TOKEN_TEXT_META_WEIGHT: &str = "--text-meta-weight";
/// Micro text size (badges, chips, footnotes). Default: 0.75rem (12px).
pub const TOKEN_TEXT_MICRO_SIZE: &str = "--text-micro-size";
/// Micro text weight. Default: 500.
pub const TOKEN_TEXT_MICRO_WEIGHT: &str = "--text-micro-weight";

/// All token names in the ferro-theme/v2 vocabulary (40 slots).
pub const ALL_TOKENS: &[&str] = &[
    TOKEN_BACKGROUND,
    TOKEN_SURFACE,
    TOKEN_CARD,
    TOKEN_BORDER,
    TOKEN_TEXT,
    TOKEN_TEXT_MUTED,
    TOKEN_PRIMARY,
    TOKEN_PRIMARY_FOREGROUND,
    TOKEN_SECONDARY,
    TOKEN_SECONDARY_FOREGROUND,
    TOKEN_ACCENT,
    TOKEN_DESTRUCTIVE,
    TOKEN_SUCCESS,
    TOKEN_WARNING,
    TOKEN_RADIUS_SM,
    TOKEN_RADIUS_MD,
    TOKEN_RADIUS_LG,
    TOKEN_RADIUS_FULL,
    TOKEN_SHADOW_SM,
    TOKEN_SHADOW_MD,
    TOKEN_SHADOW_LG,
    TOKEN_FONT_SANS,
    TOKEN_FONT_MONO,
    TOKEN_SPACING,
    TOKEN_MOTION_DURATION_FAST,
    TOKEN_MOTION_DURATION_BASE,
    TOKEN_MOTION_DURATION_SLOW,
    TOKEN_MOTION_EASE,
    TOKEN_COLOR_RING,
    TOKEN_FONT_DISPLAY,
    // Type-scale tokens (v2)
    TOKEN_TEXT_DISPLAY_SIZE,
    TOKEN_TEXT_DISPLAY_WEIGHT,
    TOKEN_TEXT_SECTION_SIZE,
    TOKEN_TEXT_SECTION_WEIGHT,
    TOKEN_TEXT_BODY_SIZE,
    TOKEN_TEXT_BODY_WEIGHT,
    TOKEN_TEXT_META_SIZE,
    TOKEN_TEXT_META_WEIGHT,
    TOKEN_TEXT_MICRO_SIZE,
    TOKEN_TEXT_MICRO_WEIGHT,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_tokens_len_is_40() {
        assert_eq!(
            ALL_TOKENS.len(),
            40,
            "ALL_TOKENS must have exactly 40 slots"
        );
    }

    #[test]
    fn all_tokens_contains_text_display_size() {
        assert!(
            ALL_TOKENS.contains(&"--text-display-size"),
            "ALL_TOKENS must contain --text-display-size"
        );
    }

    #[test]
    fn all_tokens_contains_text_body_size() {
        assert!(
            ALL_TOKENS.contains(&"--text-body-size"),
            "ALL_TOKENS must contain --text-body-size"
        );
    }

    #[test]
    fn all_tokens_contains_text_micro_weight() {
        assert!(
            ALL_TOKENS.contains(&"--text-micro-weight"),
            "ALL_TOKENS must contain --text-micro-weight"
        );
    }

    #[test]
    fn all_tokens_contains_font_display() {
        assert!(
            ALL_TOKENS.contains(&"--font-display"),
            "ALL_TOKENS must contain --font-display"
        );
    }

    #[test]
    fn all_tokens_no_duplicates() {
        let set: HashSet<&&str> = ALL_TOKENS.iter().collect();
        assert_eq!(
            set.len(),
            ALL_TOKENS.len(),
            "ALL_TOKENS must not contain duplicate slots"
        );
    }
}
