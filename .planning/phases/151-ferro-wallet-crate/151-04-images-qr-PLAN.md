---
phase: 151
plan: 151-04
slug: images-qr
wave: 2
depends_on: [151-01]
files_modified:
  - ferro-wallet/src/images.rs
  - ferro-wallet/src/qr.rs
autonomous: true
requirements: [ACC-1g, ACC-1h]
must_haves:
  truths:
    - "`fit_to(bytes, w, h)` resizes preserve-aspect and centre-pads onto a transparent canvas, returning PNG bytes of exact (w, h)"
    - "`apple_logo_set` returns three entries (logo.png 160×50, logo@2x.png 320×100, logo@3x.png 480×150)"
    - "`apple_icon_set` returns three entries (icon.png 29×29, icon@2x.png 58×58, icon@3x.png 87×87); derives from logo via centre-square-crop if no explicit icon"
    - "`qr::png(data, size)` returns valid PNG bytes (8-byte magic prefix)"
    - "`qr::data_uri(data, size)` returns `data:image/png;base64,<b64>`"
  artifacts:
    - path: "ferro-wallet/src/images.rs"
      provides: "fit_to + apple_logo_set + apple_icon_set + google_hero"
      contains: "pub fn fit_to"
    - path: "ferro-wallet/src/qr.rs"
      provides: "png + data_uri"
      contains: "pub fn png"
  key_links:
    - from: "fit_to"
      to: "DynamicImage::resize + imageops::overlay"
      via: "image 0.25 crate"
      pattern: "FilterType::Lanczos3"
    - from: "qr::png"
      to: "qrcode_generator::to_png_to_vec"
      via: "qrcode-generator 5"
      pattern: "QrCodeEcc::Medium"
---

<objective>
Implement the pure-transform `images` and `qr` modules. Two independent helpers (no shared state) so both fit into one plan as two atomic-commit tasks.
</objective>

<context>
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md
@ferro-wallet/src/error.rs

<interfaces>
Public API per spec §3.4:
```rust
// images.rs — pure transformation, no I/O
pub fn fit_to(bytes: &[u8], w: u32, h: u32) -> Result<Vec<u8>, WalletError>;
pub fn apple_logo_set(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError>;
pub fn apple_icon_set(icon: Option<&[u8]>, logo_fallback: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError>;
pub fn google_hero(bytes: &[u8]) -> Result<Vec<u8>, WalletError>;

// qr.rs
pub fn png(data: &str, size: u32) -> Result<Vec<u8>, WalletError>;
pub fn data_uri(data: &str, size: u32) -> Result<String, WalletError>;
```

Apple dimensions (D-03):
- logo set: `("logo.png", 160×50)`, `("logo@2x.png", 320×100)`, `("logo@3x.png", 480×150)`
- icon set: `("icon.png", 29×29)`, `("icon@2x.png", 58×58)`, `("icon@3x.png", 87×87)`
- Icon fallback from logo: centre-square-crop the logo (square of `min(w, h)` from centre), then `fit_to` each icon dimension.

Google hero: `google_hero(bytes)` = `fit_to(bytes, 1032, 336)`.

Reference code: 151-RESEARCH.md §"Code Examples" — `image 0.25 fit + centre-pad`, `qrcode-generator 5.0.0 PNG output`. RESEARCH.md Pitfall 4: `DynamicImage::resize` returns `DynamicImage`; always `.into_rgba8()` before `imageops::overlay`.
</interfaces>
</context>

<must_haves>
- `fit_to` returns PNG bytes whose decoded dimensions are exactly `(w, h)` regardless of input aspect ratio.
- `apple_logo_set` returns exactly 3 entries with the names and dimensions above.
- `apple_icon_set` returns exactly 3 entries; when `icon == None`, derives via centre-square-crop of the logo.
- `google_hero` returns PNG bytes whose decoded dimensions are exactly 1032×336.
- `qr::png(data, size)` returns bytes starting with the 8-byte PNG magic `[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]`.
- `qr::data_uri(data, size)` returns a string starting with `"data:image/png;base64,"`.
- Tests cover ACC-1g (`fit_to_exact_dims_transparent`) and ACC-1h (`png_starts_with_png_magic`).
</must_haves>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement images.rs (fit_to + apple_logo_set + apple_icon_set + google_hero) + tests</name>
  <files>ferro-wallet/src/images.rs</files>
  <read_first>
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §3.4 (image module API)
    - 151-RESEARCH.md §"Code Examples" → "image 0.25 fit + centre-pad on transparent canvas" (full body)
    - 151-RESEARCH.md §"Common Pitfalls" Pitfall 4 (DynamicImage::resize → .into_rgba8())
    - 151-PATTERNS.md §"ferro-wallet/src/images.rs"
    - 151-CONTEXT.md D-03 (dimensions for logo / icon / hero, centre-square-crop fallback)
    - 151-VALIDATION.md ACC-1g row (test name)
  </read_first>
  <behavior>
    - For any input PNG/JPEG bytes that decode successfully: `fit_to(bytes, 160, 50)` returns PNG bytes whose decoded dimensions are exactly 160×50.
    - For any input PNG/JPEG bytes: `apple_logo_set(bytes).unwrap().len() == 3`; entry names in order are `"logo.png"`, `"logo@2x.png"`, `"logo@3x.png"`; each entry's bytes decode to exactly (160,50), (320,100), (480,150) respectively.
    - With `icon: Some(bytes)`, `apple_icon_set(Some(bytes), logo).unwrap()` uses `bytes` for all three icon sizes (ignores `logo`).
    - With `icon: None`, `apple_icon_set(None, logo).unwrap()` derives via centre-square-crop of `logo` then `fit_to` each icon dimension; produces 3 entries decoding to 29×29, 58×58, 87×87.
    - `google_hero(bytes).unwrap()` decodes to exactly 1032×336.
    - Malformed input bytes return `Err(WalletError::Image(_))`.
  </behavior>
  <action>
    Replace the `// placeholder` line. Implement:

    ```rust
    //! Image normalisation for wallet passes.
    //!
    //! `fit_to` resizes preserve-aspect and centre-pads on a transparent PNG canvas
    //! at the target size. `apple_logo_set` and `apple_icon_set` produce the 1x/2x/3x
    //! resolution sets Apple Wallet requires; `google_hero` produces the 1032×336
    //! hero image Google Wallet expects. Pure transforms — no I/O.

    use image::{imageops, imageops::FilterType, DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    use crate::WalletError;

    pub fn fit_to(bytes: &[u8], w: u32, h: u32) -> Result<Vec<u8>, WalletError> {
        let src = image::load_from_memory(bytes)
            .map_err(|e| WalletError::Image(format!("decode: {e}")))?;
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

    pub fn apple_logo_set(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError> {
        Ok(vec![
            ("logo.png".to_string(),    fit_to(bytes, 160, 50)?),
            ("logo@2x.png".to_string(), fit_to(bytes, 320, 100)?),
            ("logo@3x.png".to_string(), fit_to(bytes, 480, 150)?),
        ])
    }

    pub fn apple_icon_set(
        icon: Option<&[u8]>,
        logo_fallback: &[u8],
    ) -> Result<Vec<(String, Vec<u8>)>, WalletError> {
        // If no explicit icon, centre-square-crop the logo for icon use.
        let source: Vec<u8> = match icon {
            Some(b) => b.to_vec(),
            None => centre_square_crop_png(logo_fallback)?,
        };
        Ok(vec![
            ("icon.png".to_string(),    fit_to(&source, 29, 29)?),
            ("icon@2x.png".to_string(), fit_to(&source, 58, 58)?),
            ("icon@3x.png".to_string(), fit_to(&source, 87, 87)?),
        ])
    }

    pub fn google_hero(bytes: &[u8]) -> Result<Vec<u8>, WalletError> {
        fit_to(bytes, 1032, 336)
    }

    fn centre_square_crop_png(bytes: &[u8]) -> Result<Vec<u8>, WalletError> {
        let src = image::load_from_memory(bytes)
            .map_err(|e| WalletError::Image(format!("decode: {e}")))?;
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
    ```

    Append `#[cfg(test)] mod tests` block. Build a small test fixture in-test by creating a 100×40 red `RgbaImage` and encoding to PNG bytes, then exercise:

    - `fit_to_exact_dims_transparent` (ACC-1g) — call `fit_to(&fixture, 160, 50)`, decode result, assert dimensions == (160, 50). Sample the (0, 0) pixel and assert it is fully transparent (alpha 0) because the resized image is centred — outside the resized rectangle the canvas remains transparent.
    - `apple_logo_set_returns_three_entries_with_correct_dims` — assert names match exactly and decoded dimensions are (160,50), (320,100), (480,150).
    - `apple_icon_set_derives_from_logo_when_icon_absent` — pass `None` for icon and a logo fixture; assert 3 entries, decoded dims (29,29), (58,58), (87,87), names `icon.png`, `icon@2x.png`, `icon@3x.png`.
    - `apple_icon_set_uses_explicit_icon_when_present` — pass `Some(icon_bytes)`. Use distinguishable colour so a pixel check can confirm `logo_fallback` was ignored.
    - `google_hero_returns_1032_by_336` — decode and assert dims.
    - `fit_to_rejects_malformed_bytes` — pass `b"not-an-image"`, assert `Err(WalletError::Image(_))`.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib images::tests::fit_to_exact_dims_transparent &amp;&amp; cargo test -p ferro-wallet --lib images::tests &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub fn fit_to' ferro-wallet/src/images.rs &amp;&amp; grep -F 'pub fn apple_logo_set' ferro-wallet/src/images.rs &amp;&amp; grep -F 'pub fn apple_icon_set' ferro-wallet/src/images.rs &amp;&amp; grep -F 'pub fn google_hero' ferro-wallet/src/images.rs &amp;&amp; grep -F 'FilterType::Lanczos3' ferro-wallet/src/images.rs</automated>
  </verify>
  <done>All 4 public functions land. 6 tests pass. ACC-1g test name exists. No clippy warnings.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement qr.rs (png + data_uri) + tests</name>
  <files>ferro-wallet/src/qr.rs</files>
  <read_first>
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §3.4 (qr API)
    - 151-RESEARCH.md §"Code Examples" → "qrcode-generator 5.0.0 PNG output" (full body)
    - 151-PATTERNS.md §"ferro-wallet/src/qr.rs"
    - 151-VALIDATION.md ACC-1h row (test name)
  </read_first>
  <behavior>
    - `qr::png("hello", 200).unwrap()` returns a `Vec<u8>` starting with the 8-byte PNG magic `[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]` (ACC-1h).
    - `qr::data_uri("hello", 200).unwrap()` returns a string starting with `"data:image/png;base64,"`.
    - The base64 portion of `data_uri` decodes to the same bytes as `png(...)`.
    - `qr::png("", 200)` either succeeds (qrcode-generator accepts empty) or fails with `WalletError::Qr(_)` — accept either behaviour but test it deterministically (call `.unwrap_err()` if the implementation rejects, `.unwrap()` if it accepts).
  </behavior>
  <action>
    Replace the `// placeholder` line. Implement:

    ```rust
    //! QR code generation — PNG bytes + base64 data URI helper.

    use base64::{engine::general_purpose, Engine as _};
    use qrcode_generator::QrCodeEcc;

    use crate::WalletError;

    pub fn png(data: &str, size: u32) -> Result<Vec<u8>, WalletError> {
        qrcode_generator::to_png_to_vec(data, QrCodeEcc::Medium, size as usize)
            .map_err(|e| WalletError::Qr(format!("png generate: {e}")))
    }

    pub fn data_uri(data: &str, size: u32) -> Result<String, WalletError> {
        let bytes = png(data, size)?;
        let b64 = general_purpose::STANDARD.encode(&bytes);
        Ok(format!("data:image/png;base64,{b64}"))
    }
    ```

    Append `#[cfg(test)] mod tests` block:

    - `png_starts_with_png_magic` (ACC-1h) — call `png("hello", 200).unwrap()`; assert bytes start with `[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]`.
    - `data_uri_starts_with_data_image_png_base64` — call `data_uri("hello", 200).unwrap()`; assert it `.starts_with("data:image/png;base64,")`.
    - `data_uri_payload_decodes_to_png_bytes` — strip the prefix, base64-decode, assert the decoded bytes equal `png("hello", 200).unwrap()`.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib qr::tests::png_starts_with_png_magic &amp;&amp; cargo test -p ferro-wallet --lib qr::tests &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub fn png' ferro-wallet/src/qr.rs &amp;&amp; grep -F 'pub fn data_uri' ferro-wallet/src/qr.rs &amp;&amp; grep -F 'QrCodeEcc::Medium' ferro-wallet/src/qr.rs</automated>
  </verify>
  <done>Two public functions land. ACC-1h test name exists and passes. PNG magic prefix verified. Data URI shape verified.</done>
</task>

</tasks>

<threat_model>
This plan contains no security-relevant code. `images.rs` decodes/encodes images via the `image` crate; `qr.rs` generates QR codes via `qrcode-generator`. No secrets, no crypto, no network.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| (none) | — | — | — | No security-relevant code in this plan. Image decode is provided by the `image` crate; any malformed-input panic risk is bounded by `image::load_from_memory` returning `Result`. |
</threat_model>

<verification>
- `cargo test -p ferro-wallet --lib images::tests` runs ≥6 tests, all pass.
- `cargo test -p ferro-wallet --lib qr::tests` runs ≥3 tests, all pass.
- `cargo build -p ferro-wallet` exits 0.
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0.
- `cargo fmt -p ferro-wallet -- --check` exits 0.
- ACC-1g and ACC-1h test names exist and are referenced by the exact commands in VALIDATION.md.
</verification>

<success_criteria>
PLAN-05's apple builder can call `images::apple_logo_set` and `images::apple_icon_set` to produce the 6 image entries that go into the `.pkpass` ZIP. PLAN-07's google builder (or downstream callers) can call `images::google_hero` and `qr::data_uri` to populate the eventTicketObject.
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-04-SUMMARY.md` documenting the exact dimensions returned by each set and the PNG magic-byte assertion used in `qr::tests`.
</output>

## PLANNING COMPLETE
