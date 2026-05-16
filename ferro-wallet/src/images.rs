//! Image normalisation for wallet passes.
//!
//! `fit_to` resizes preserve-aspect and centre-pads on a transparent PNG canvas
//! at the target size. `apple_logo_set` and `apple_icon_set` produce the 1x/2x/3x
//! resolution sets Apple Wallet requires; `google_hero` produces the 1032×336
//! hero image Google Wallet expects. Pure transforms — no I/O.

use image::{
    imageops, imageops::FilterType, DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage,
};
use std::io::Cursor;

use crate::WalletError;

/// Produce a 1×1 fully-transparent PNG. Used as a fallback icon source when a
/// pass has neither a logo nor an explicit icon — Apple still requires `icon.png`
/// to be present in the bundle.
pub fn transparent_1x1_png() -> Result<Vec<u8>, WalletError> {
    let img = RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 0]));
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img)
        .write_to(&mut out, ImageFormat::Png)
        .map_err(|e| WalletError::Image(format!("encode png: {e}")))?;
    Ok(out.into_inner())
}

/// Resize `bytes` to fit within `(w, h)` preserving aspect ratio, then centre-pad
/// onto a transparent canvas of exactly `(w, h)` and encode as PNG.
pub fn fit_to(bytes: &[u8], w: u32, h: u32) -> Result<Vec<u8>, WalletError> {
    let src =
        image::load_from_memory(bytes).map_err(|e| WalletError::Image(format!("decode: {e}")))?;
    let resized = src.resize(w, h, FilterType::Lanczos3).into_rgba8();
    let (rw, rh) = resized.dimensions();

    let mut canvas = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    let x = ((w as i64) - (rw as i64)) / 2;
    let y = ((h as i64) - (rh as i64)) / 2;
    imageops::overlay(&mut canvas, &resized, x, y);

    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(canvas)
        .write_to(&mut out, ImageFormat::Png)
        .map_err(|e| WalletError::Image(format!("encode png: {e}")))?;
    Ok(out.into_inner())
}

/// Produce the three Apple Wallet logo PNGs (1x/2x/3x) from a single input.
///
/// Returns entries in order: `("logo.png", 160×50)`, `("logo@2x.png", 320×100)`,
/// `("logo@3x.png", 480×150)`.
pub fn apple_logo_set(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError> {
    Ok(vec![
        ("logo.png".to_string(), fit_to(bytes, 160, 50)?),
        ("logo@2x.png".to_string(), fit_to(bytes, 320, 100)?),
        ("logo@3x.png".to_string(), fit_to(bytes, 480, 150)?),
    ])
}

/// Produce the three Apple Wallet icon PNGs (1x/2x/3x).
///
/// If `icon` is `Some`, those bytes are used directly. Otherwise the logo is
/// centre-square-cropped (`side = min(w, h)`) and that crop drives all three icon
/// dimensions. Returns `("icon.png", 29×29)`, `("icon@2x.png", 58×58)`,
/// `("icon@3x.png", 87×87)`.
pub fn apple_icon_set(
    icon: Option<&[u8]>,
    logo_fallback: &[u8],
) -> Result<Vec<(String, Vec<u8>)>, WalletError> {
    let source: Vec<u8> = match icon {
        Some(b) => b.to_vec(),
        None => centre_square_crop_png(logo_fallback)?,
    };
    Ok(vec![
        ("icon.png".to_string(), fit_to(&source, 29, 29)?),
        ("icon@2x.png".to_string(), fit_to(&source, 58, 58)?),
        ("icon@3x.png".to_string(), fit_to(&source, 87, 87)?),
    ])
}

/// Produce the Google Wallet hero image (1032×336 PNG).
pub fn google_hero(bytes: &[u8]) -> Result<Vec<u8>, WalletError> {
    fit_to(bytes, 1032, 336)
}

/// Produce the three Apple Wallet strip PNGs (1x/2x/3x).
///
/// The strip image is the banner-style background that fills the top of an
/// `eventTicket` pass — what gives passes their "ticket" look versus the flat
/// card layout without a strip.
///
/// Apple's recommended sizes for `eventTicket` with rectangular barcode:
/// `("strip.png", 320×84)`, `("strip@2x.png", 640×168)`, `("strip@3x.png", 960×252)`.
pub fn apple_strip_set(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError> {
    Ok(vec![
        ("strip.png".to_string(), fit_to(bytes, 320, 84)?),
        ("strip@2x.png".to_string(), fit_to(bytes, 640, 168)?),
        ("strip@3x.png".to_string(), fit_to(bytes, 960, 252)?),
    ])
}

/// Decode the input, centre-square-crop to `side = min(w, h)`, re-encode as PNG.
fn centre_square_crop_png(bytes: &[u8]) -> Result<Vec<u8>, WalletError> {
    let src =
        image::load_from_memory(bytes).map_err(|e| WalletError::Image(format!("decode: {e}")))?;
    let (w, h) = src.dimensions();
    let side = w.min(h);
    let x = (w - side) / 2;
    let y = (h - side) / 2;
    let cropped = src.crop_imm(x, y, side, side);
    let mut out = Cursor::new(Vec::new());
    cropped
        .write_to(&mut out, ImageFormat::Png)
        .map_err(|e| WalletError::Image(format!("encode png: {e}")))?;
    Ok(out.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    /// Encode an `RgbaImage` to PNG bytes (test fixture helper).
    fn encode_png(img: &RgbaImage) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(img.clone())
            .write_to(&mut out, ImageFormat::Png)
            .expect("encode png");
        out.into_inner()
    }

    /// 100×40 red rectangle — wider than tall, useful for centre-pad-vertical tests.
    fn red_100x40() -> Vec<u8> {
        let img = RgbaImage::from_pixel(100, 40, Rgba([255, 0, 0, 255]));
        encode_png(&img)
    }

    /// 200×200 green square — useful for icon-set tests.
    fn green_200x200() -> Vec<u8> {
        let img = RgbaImage::from_pixel(200, 200, Rgba([0, 255, 0, 255]));
        encode_png(&img)
    }

    /// 100×100 blue square — useful as an "explicit icon" distinguishable from green.
    fn blue_100x100() -> Vec<u8> {
        let img = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 255, 255]));
        encode_png(&img)
    }

    /// Decode PNG bytes back to (w, h) for dimension assertions.
    fn dims(bytes: &[u8]) -> (u32, u32) {
        let img = image::load_from_memory(bytes).expect("decode");
        img.dimensions()
    }

    #[test]
    fn fit_to_exact_dims_transparent() {
        // ACC-1g — 100×40 red into a 160×50 canvas:
        // preserve-aspect resize fills width, leaves transparent bands top/bottom.
        let src = red_100x40();
        let out = fit_to(&src, 160, 50).expect("fit_to");
        assert_eq!(dims(&out), (160, 50), "output must be exactly 160×50");

        // Top-left corner pixel must be transparent: the resized red rectangle is
        // centred horizontally but narrower than 160px at the same aspect ratio, so
        // (0, 0) sits in the padded region.
        let decoded = image::load_from_memory(&out).expect("decode").to_rgba8();
        let corner = decoded.get_pixel(0, 0);
        assert_eq!(corner[3], 0, "corner pixel must be fully transparent");
    }

    #[test]
    fn apple_logo_set_returns_three_entries_with_correct_dims() {
        let src = red_100x40();
        let set = apple_logo_set(&src).expect("apple_logo_set");
        assert_eq!(set.len(), 3, "must return exactly 3 entries");

        assert_eq!(set[0].0, "logo.png");
        assert_eq!(dims(&set[0].1), (160, 50));

        assert_eq!(set[1].0, "logo@2x.png");
        assert_eq!(dims(&set[1].1), (320, 100));

        assert_eq!(set[2].0, "logo@3x.png");
        assert_eq!(dims(&set[2].1), (480, 150));
    }

    #[test]
    fn apple_icon_set_derives_from_logo_when_icon_absent() {
        let logo = green_200x200();
        let set = apple_icon_set(None, &logo).expect("apple_icon_set None");
        assert_eq!(set.len(), 3);

        assert_eq!(set[0].0, "icon.png");
        assert_eq!(dims(&set[0].1), (29, 29));

        assert_eq!(set[1].0, "icon@2x.png");
        assert_eq!(dims(&set[1].1), (58, 58));

        assert_eq!(set[2].0, "icon@3x.png");
        assert_eq!(dims(&set[2].1), (87, 87));

        // Centre pixel of the 29×29 icon must be green (came from logo_fallback).
        let decoded = image::load_from_memory(&set[0].1)
            .expect("decode")
            .to_rgba8();
        let centre = decoded.get_pixel(14, 14);
        assert!(
            centre[1] > centre[0] && centre[1] > centre[2],
            "icon centre must be dominantly green (from logo_fallback), got {centre:?}",
        );
    }

    #[test]
    fn apple_icon_set_uses_explicit_icon_when_present() {
        // logo = green, explicit icon = blue. If the explicit icon is used,
        // the centre pixel of the produced icon must be blue, not green.
        let logo = green_200x200();
        let icon = blue_100x100();
        let set = apple_icon_set(Some(&icon), &logo).expect("apple_icon_set Some");
        assert_eq!(set.len(), 3);
        assert_eq!(dims(&set[0].1), (29, 29));

        let decoded = image::load_from_memory(&set[0].1)
            .expect("decode")
            .to_rgba8();
        let centre = decoded.get_pixel(14, 14);
        assert!(
            centre[2] > centre[0] && centre[2] > centre[1],
            "icon centre must be dominantly blue (from explicit icon), got {centre:?}",
        );
    }

    #[test]
    fn google_hero_returns_1032_by_336() {
        let src = red_100x40();
        let out = google_hero(&src).expect("google_hero");
        assert_eq!(dims(&out), (1032, 336));
    }

    #[test]
    fn fit_to_rejects_malformed_bytes() {
        let err = fit_to(b"not-an-image", 160, 50).expect_err("must fail on garbage");
        assert!(matches!(err, WalletError::Image(_)), "got {err:?}");
    }
}
