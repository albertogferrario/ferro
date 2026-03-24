//! Fixed semantic token vocabulary for ferro-theme/v1.
//!
//! Defines ~23 semantic slots that every theme must provide. Tokens are
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

/// All token names in the ferro-theme/v1 vocabulary (23 slots).
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
];
