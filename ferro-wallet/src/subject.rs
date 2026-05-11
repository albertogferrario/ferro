//! `WalletSubject` — the content contract every downstream domain object implements
//! to be issued as either an Apple `.pkpass` or a Google Wallet save-JWT.
//!
//! See the design spec at `docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md`
//! §3.1 for the authoritative public surface. Value types stay deliberately small:
//! they are the input shape both `ApplePassBuilder::build` and
//! `GoogleWalletBuilder::save_jwt` accept.

use crate::WalletError;
use chrono::{DateTime, Utc};

/// Top-level pass category. v1 only exercises [`PassKind::EventTicket`] in integration
/// tests; [`PassKind::Generic`] and [`PassKind::Coupon`] are declared but not test-covered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassKind {
    EventTicket,
    Generic,
    Coupon,
}

/// Field alignment hint surfaced to the wallet renderer. Maps directly to Apple's
/// `PKTextAlignment*` values; Google ignores it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldAlignment {
    Left,
    Center,
    Right,
    Natural,
}

/// Foreground colour derivation mode. [`TextColorMode::Auto`] uses BT.601 luminance
/// (see [`auto_foreground`]). [`TextColorMode::Light`] / [`TextColorMode::Dark`]
/// force white / black respectively.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextColorMode {
    Auto,
    Light,
    Dark,
}

/// A single labelled value on a pass (primary, secondary, auxiliary, or back row).
#[derive(Debug, Clone)]
pub struct Field {
    pub key: String,
    pub label: String,
    pub value: String,
    pub alignment: FieldAlignment,
}

/// Visual + identity branding applied to the pass. All image fields are raw PNG bytes —
/// the `images` module produces them in the resolutions Apple / Google require.
#[derive(Debug, Clone)]
pub struct Branding {
    pub organization_name: Option<String>,
    pub logo_text: Option<String>,
    pub background_color: RgbColor,
    pub text_color_mode: TextColorMode,
    pub logo_png_bytes: Vec<u8>,
    pub icon_png_bytes: Option<Vec<u8>>,
    pub hero_png_bytes: Option<Vec<u8>>,
}

/// Optional geographic relevance hint — Apple surfaces the pass on the lock screen when
/// the device is near the coordinate.
#[derive(Debug, Clone)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub relevant_text: Option<String>,
}

/// 24-bit RGB colour. Constructed from a `#RRGGBB` hex string via [`RgbColor::from_hex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    /// Parse a 6-digit hex colour. Accepts `"#RRGGBB"` or `"RRGGBB"`. Case-insensitive.
    /// Short forms (`#fff`) and 8-digit forms (`#RRGGBBAA`) are rejected.
    pub fn from_hex(s: &str) -> Result<Self, WalletError> {
        let hex = s.strip_prefix('#').unwrap_or(s);
        if hex.len() != 6 {
            return Err(WalletError::InvalidInput(format!(
                "rgb hex must be 6 chars (with optional leading '#'): got {s:?}"
            )));
        }
        let parse = |range: std::ops::Range<usize>| -> Result<u8, WalletError> {
            u8::from_str_radix(&hex[range], 16)
                .map_err(|e| WalletError::InvalidInput(format!("rgb hex parse: {e}")))
        };
        Ok(RgbColor {
            r: parse(0..2)?,
            g: parse(2..4)?,
            b: parse(4..6)?,
        })
    }

    /// CSS-style `rgb(r,g,b)` literal — used by Apple `pass.json` colour fields.
    pub fn css_rgb(&self) -> String {
        format!("rgb({},{},{})", self.r, self.g, self.b)
    }
}

/// Derives a readable foreground colour from a background using ITU-R BT.601 luminance
/// (D-06). Normalised luminance `Y = 0.299*R + 0.587*G + 0.114*B`, all channels in `0..1`.
///
/// - `Y < 0.5` ⇒ white `rgb(255,255,255)`
/// - `Y >= 0.5` ⇒ dark slate `rgb(17,24,39)`
///
/// The mid-grey case `rgb(128,128,128)` resolves to dark slate (luminance ≈ 0.502).
pub fn auto_foreground(bg: RgbColor) -> RgbColor {
    let r = bg.r as f64 / 255.0;
    let g = bg.g as f64 / 255.0;
    let b = bg.b as f64 / 255.0;
    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
    if lum < 0.5 {
        RgbColor {
            r: 255,
            g: 255,
            b: 255,
        }
    } else {
        RgbColor {
            r: 17,
            g: 24,
            b: 39,
        }
    }
}

/// The content contract every domain object implements to be issued as a wallet pass.
///
/// Implementors describe what the pass _means_ (identity, fields, branding, barcode,
/// timing). The builders ([`crate::apple::ApplePassBuilder`],
/// [`crate::google::GoogleWalletBuilder`]) translate this into the appropriate wire
/// format.
pub trait WalletSubject {
    /// Top-level pass category.
    fn pass_kind(&self) -> PassKind;

    /// Unique-per-pass identifier (becomes Apple `serialNumber` and the suffix of the
    /// Google `object.id`).
    fn serial(&self) -> String;

    /// The single most prominent field on the pass.
    fn primary(&self) -> Field;

    /// Secondary row — usually 1–4 fields directly under the primary.
    fn secondary(&self) -> Vec<Field>;

    /// Auxiliary row — additional context fields below secondary.
    fn auxiliary(&self) -> Vec<Field>;

    /// Back-of-pass fields — long-form details exposed when the user flips the pass.
    fn back(&self) -> Vec<Field>;

    /// Opaque token encoded into the pass's QR / barcode.
    fn barcode_token(&self) -> String;

    /// Optional timestamp at which the pass is most relevant (Apple surfaces it on the
    /// lock screen near this time).
    fn relevant_at(&self) -> Option<DateTime<Utc>>;

    /// Optional expiry timestamp.
    fn expires_at(&self) -> Option<DateTime<Utc>>;

    /// Optional geographic relevance hints.
    fn locations(&self) -> Vec<GeoPoint>;

    /// Visual + identity branding.
    fn branding(&self) -> Branding;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_from_hex() {
        assert_eq!(
            RgbColor::from_hex("#ffffff").unwrap(),
            RgbColor {
                r: 255,
                g: 255,
                b: 255
            }
        );
        assert_eq!(
            RgbColor::from_hex("000000").unwrap(),
            RgbColor { r: 0, g: 0, b: 0 }
        );
        assert_eq!(
            RgbColor::from_hex("#FF8000").unwrap(),
            RgbColor {
                r: 255,
                g: 128,
                b: 0
            }
        );
        // Mixed case accepted.
        assert_eq!(
            RgbColor::from_hex("#aAbBcC").unwrap(),
            RgbColor {
                r: 0xaa,
                g: 0xbb,
                b: 0xcc
            }
        );
    }

    #[test]
    fn rgb_from_hex_rejects_malformed() {
        // Free-form non-hex string.
        let err = RgbColor::from_hex("not-a-color").unwrap_err();
        assert!(matches!(err, WalletError::InvalidInput(_)));

        // 3-digit short form not supported.
        let err = RgbColor::from_hex("#fff").unwrap_err();
        assert!(matches!(err, WalletError::InvalidInput(_)));

        // 7 chars total (one too many).
        let err = RgbColor::from_hex("#fffffff").unwrap_err();
        assert!(matches!(err, WalletError::InvalidInput(_)));

        // 6 chars but non-hex.
        let err = RgbColor::from_hex("#zzzzzz").unwrap_err();
        assert!(matches!(err, WalletError::InvalidInput(_)));

        // Empty.
        let err = RgbColor::from_hex("").unwrap_err();
        assert!(matches!(err, WalletError::InvalidInput(_)));
    }

    #[test]
    fn auto_foreground_dark_bg_is_white() {
        assert_eq!(
            auto_foreground(RgbColor { r: 0, g: 0, b: 0 }),
            RgbColor {
                r: 255,
                g: 255,
                b: 255
            }
        );
    }

    #[test]
    fn auto_foreground_light_bg_is_dark_slate() {
        assert_eq!(
            auto_foreground(RgbColor {
                r: 255,
                g: 255,
                b: 255
            }),
            RgbColor {
                r: 17,
                g: 24,
                b: 39
            }
        );
    }

    #[test]
    fn rgb_css_rgb_format() {
        assert_eq!(
            RgbColor {
                r: 17,
                g: 24,
                b: 39
            }
            .css_rgb(),
            "rgb(17,24,39)"
        );
    }
}
