---
phase: 151-ferro-wallet-crate
plan: 04
subsystem: assets
tags: [ferro-wallet, images, qr, pkpass, google-wallet, png, base64]

requires:
  - phase: 151-01-scaffold
    provides: ferro-wallet crate scaffold + WalletError::Image / WalletError::Qr variants + images/qr module placeholders
provides:
  - "images::fit_to(bytes, w, h) — preserve-aspect Lanczos3 resize + centre-pad on transparent canvas, PNG output"
  - "images::apple_logo_set(bytes) — Vec<(name, bytes)> for logo.png 160x50, logo@2x.png 320x100, logo@3x.png 480x150"
  - "images::apple_icon_set(icon, logo_fallback) — Vec<(name, bytes)> for icon.png 29x29, icon@2x.png 58x58, icon@3x.png 87x87 with centre-square-crop derivation when icon is None"
  - "images::google_hero(bytes) — 1032x336 PNG (delegates to fit_to)"
  - "qr::png(data, size) — qrcode-generator to_png_to_vec with QrCodeEcc::Medium"
  - "qr::data_uri(data, size) — base64-encoded data:image/png;base64,... wrapper around qr::png"
affects:
  - 151-05-apple-builder (consumes apple_logo_set + apple_icon_set to populate the 6 image entries of the .pkpass ZIP)
  - 151-07-google-builder (consumes google_hero + qr::data_uri for eventTicketObject.heroImage + barcode)
  - downstream gestiscilo-it wallet integration (calls these helpers when staging input branding assets)

tech-stack:
  added: []
  patterns:
    - "DynamicImage::resize -> .into_rgba8() before imageops::overlay (RESEARCH.md Pitfall 4: pixel types must match for overlay; resize returns DynamicImage, not RgbaImage)"
    - "Centre-pad math uses i64 offsets (image 0.25 imageops::overlay signature) so the resize-larger-than-target edge case stays defensive"
    - "Test fixtures are constructed in-process via RgbaImage::from_pixel + PNG encode — no on-disk asset bytes, no test data files"
    - "QR data_uri composed as png + base64::engine::general_purpose::STANDARD.encode rather than via a separate qrcode-generator helper to keep the wrapping format explicit"

key-files:
  created: []
  modified:
    - "ferro-wallet/src/images.rs (placeholder -> 4 public functions, 1 private helper, 6 unit tests)"
    - "ferro-wallet/src/qr.rs (placeholder -> 2 public functions, 3 unit tests)"

key-decisions:
  - "[151-04] fit_to uses FilterType::Lanczos3 as the only resize filter — Lanczos3 is the highest-quality resampler in image 0.25 and matters at 1x/2x/3x logo scales where moiré is visible; the cost (vs Triangle/Catmull-Rom) is negligible at these dimensions"
  - "[151-04] apple_icon_set takes (icon: Option, logo_fallback) rather than (logo, icon: Option) to keep the most-common call site (\"derive from logo\") at apple_icon_set(None, logo) — caller does not have to construct an explicit Some()"
  - "[151-04] centre-square-crop helper re-encodes as PNG instead of holding the cropped DynamicImage — fit_to is the single decode/encode boundary, so the helper produces bytes the same shape (PNG) as every other input fit_to handles"
  - "[151-04] qr::data_uri builds the data URI as a format!(\"data:image/png;base64,{b64}\") rather than via the data-url crate to avoid pulling in a single-string-prefix dependency"

patterns-established:
  - "Pure-transform image helpers return Result<Vec<u8>, WalletError> with WalletError::Image variant for decode/encode failures"
  - "Apple multi-resolution sets returned as Vec<(String, Vec<u8>)> in fixed canonical order (1x, 2x, 3x) so downstream ZIP writers iterate in stable insertion order"
  - "QR PNG generation pinned to QrCodeEcc::Medium (~15% error correction) — sufficient redundancy for wallet barcode use cases without inflating module size"

requirements-completed: [ACC-1g, ACC-1h]

duration: 2m 46s
completed: 2026-05-11
---

# Phase 151 Plan 04: Pure-transform image + QR helpers

**Two stateless modules: `images` produces Apple Wallet logo/icon sets and the Google Wallet hero, `qr` produces PNG bytes and base64 data URIs — both wrapping `WalletError` and pinned to `FilterType::Lanczos3` / `QrCodeEcc::Medium`.**

## Performance

- **Duration:** 2m 46s
- **Started:** 2026-05-11T03:52:25Z
- **Completed:** 2026-05-11T03:55:11Z
- **Tasks:** 2 (both `auto` with `tdd="true"`)
- **Files modified:** 2 (`ferro-wallet/src/images.rs`, `ferro-wallet/src/qr.rs`)

## Accomplishments

- Implemented `images::fit_to` as the single decode/resize/centre-pad/encode primitive every other image helper builds on (Apple logo set, Apple icon set, Google hero).
- Wired the three-tier Apple resolution sets (`logo.png`/`@2x`/`@3x` at 160×50 / 320×100 / 480×150; `icon.png`/`@2x`/`@3x` at 29×29 / 58×58 / 87×87) per D-03.
- Implemented `apple_icon_set` with the derive-from-logo fallback: when `icon == None`, centre-square-crops the logo (`side = min(w, h)`) and resizes through `fit_to` at each icon dimension; when `icon == Some(b)`, those bytes drive all three sizes verbatim.
- Implemented `qr::png` + `qr::data_uri` with `QrCodeEcc::Medium` and the canonical `data:image/png;base64,...` wrapping.
- Shipped 9 unit tests (6 images + 3 qr); ACC-1g (`fit_to_exact_dims_transparent`) and ACC-1h (`png_starts_with_png_magic`) both green; 30/30 ferro-wallet tests pass.

## Dimensions Returned by Each Helper

| Helper | Output dimensions | Format | Naming |
|--------|-------------------|--------|--------|
| `fit_to(b, w, h)` | exactly `(w, h)` | PNG (RGBA) | n/a — raw bytes |
| `apple_logo_set` | (160, 50), (320, 100), (480, 150) | PNG | `logo.png`, `logo@2x.png`, `logo@3x.png` |
| `apple_icon_set` | (29, 29), (58, 58), (87, 87) | PNG | `icon.png`, `icon@2x.png`, `icon@3x.png` |
| `google_hero` | exactly (1032, 336) | PNG | n/a — raw bytes |
| `qr::png(data, size)` | square, `size × size` | PNG | n/a — raw bytes |
| `qr::data_uri(data, size)` | square, `size × size` | data URI | string `"data:image/png;base64,…"` |

## PNG Magic-Byte Assertion (ACC-1h)

`qr::tests::png_starts_with_png_magic` asserts that `qr::png("hello", 200)` returns bytes whose first 8 octets equal the PNG file signature:

```
[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
```

This is the canonical PNG magic per the PNG spec (§5.2 PNG signature); presence guarantees the bytes are at minimum a syntactically-valid PNG file header. The test additionally asserts `bytes.len() >= 8` to make the slice-index unambiguously safe.

## Acceptance Criteria → Test Mapping

| Criterion | Test Name | Behaviour |
|-----------|-----------|-----------|
| ACC-1g | `images::tests::fit_to_exact_dims_transparent` | 100×40 red rectangle ⇒ 160×50 output; corner pixel α=0 (transparent canvas honoured) |
| ACC-1h | `qr::tests::png_starts_with_png_magic` | `png("hello", 200)` starts with the 8-byte PNG magic prefix |

Supporting tests in the same modules (not directly mapped to ACC-IDs but locked in):
- `images::tests::apple_logo_set_returns_three_entries_with_correct_dims` — names + decoded dims for all three logo entries
- `images::tests::apple_icon_set_derives_from_logo_when_icon_absent` — `None` icon path; centre pixel of 29×29 icon dominantly green (came from the green logo fallback)
- `images::tests::apple_icon_set_uses_explicit_icon_when_present` — `Some(blue)` over `green` logo; centre pixel of 29×29 icon dominantly blue (explicit icon used, fallback ignored)
- `images::tests::google_hero_returns_1032_by_336` — decoded dims match exactly
- `images::tests::fit_to_rejects_malformed_bytes` — `b"not-an-image"` ⇒ `Err(WalletError::Image(_))`
- `qr::tests::data_uri_starts_with_data_image_png_base64` — prefix shape verified
- `qr::tests::data_uri_payload_decodes_to_png_bytes` — base64-decoded payload equals `qr::png` bytes (round-trip)

## Task Commits

1. **Task 1: Implement images.rs (fit_to + apple_logo_set + apple_icon_set + google_hero) + tests** — `c536fd61` (feat)
2. **Task 2: Implement qr.rs (png + data_uri) + tests** — `00007b9f` (feat)

## Files Created/Modified

- `ferro-wallet/src/images.rs` — Replaced `// placeholder` with crate-level doc-comment, four public functions (`fit_to`, `apple_logo_set`, `apple_icon_set`, `google_hero`), one private helper (`centre_square_crop_png`), and a `#[cfg(test)] mod tests` block containing four fixture helpers (`encode_png`, `red_100x40`, `green_200x200`, `blue_100x100`, `dims`) plus six acceptance / supporting tests.
- `ferro-wallet/src/qr.rs` — Replaced `// placeholder` with two public functions (`png`, `data_uri`) wrapping `qrcode-generator::to_png_to_vec` + `base64::engine::general_purpose::STANDARD.encode`, plus a `#[cfg(test)] mod tests` block with the canonical `PNG_MAGIC` constant and three acceptance / supporting tests.

No changes to `ferro-wallet/src/lib.rs` — both modules are already declared `pub mod images;` / `pub mod qr;` from PLAN-01.

## Decisions Made

- **Lanczos3 as the sole resize filter.** `image::imageops::FilterType` exposes Nearest / Triangle / CatmullRom / Gaussian / Lanczos3. At the 1×/2×/3× logo scales an Apple Wallet card renders, Triangle exhibits noticeable jaggies on diagonals and CatmullRom produces visible ringing on high-contrast edges. Lanczos3 is the established correct choice for downscaling photographic / illustrative content; the per-call cost is bounded by the resize dimensions (max 480×150 in this plan) and dominated by the surrounding PNG encode anyway.
- **`apple_icon_set` parameter order: `(icon, logo_fallback)` not `(logo, icon)`.** The most common downstream call site is "derive icons from the logo" — `apple_icon_set(None, logo)` reads naturally; `apple_icon_set(logo, None)` would be less obvious. The explicit-icon case (`Some(b), logo`) is a strict superset whose semantics the function signature documents via the parameter name.
- **Centre-square-crop helper round-trips through PNG.** The helper takes bytes and returns bytes (not a `DynamicImage`) so the calling site of `fit_to` always feeds it bytes from a single canonical path — no `DynamicImage` is constructed at the apple_icon_set call site, which keeps the public surface byte-oriented end to end. The small re-encode cost (one extra PNG round-trip for the icon-derived-from-logo case) is acceptable for the simplicity gain.
- **`qr::data_uri` composes the URI manually.** `format!("data:image/png;base64,{b64}")` is two lines; a hypothetical `data-url` crate dependency would add zero abstraction benefit at the cost of a transitive dependency. The base64 alphabet is the standard `STANDARD` engine (not `URL_SAFE`) — `data:` URIs use the standard alphabet per RFC 2397.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Lint] Inlined `format!` args to satisfy `clippy::uninlined_format_args`**

- **Found during:** Task 1 (`cargo clippy -p ferro-wallet --all-targets -- -D warnings`)
- **Issue:** Two `assert!` macros in the images tests used `{:?}` + positional argument (`"...got {:?}", centre`) which clippy's `uninlined_format_args` lint flags as -D warning under CI's pre-push command.
- **Fix:** Rewrote as `"...got {centre:?}"` interpolation. Behaviour unchanged.
- **Files modified:** `ferro-wallet/src/images.rs` (test block only)
- **Verification:** `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0 after the fix.
- **Committed in:** `c536fd61` (Task 1 commit — fixes integrated before the commit landed)

**2. [Rule 1 - Lint] Removed unused `Engine as _` import from qr::tests**

- **Found during:** Task 2 (`cargo test -p ferro-wallet --lib qr::tests` — compile warning, not yet a lint failure)
- **Issue:** The test module re-imported `base64::Engine as _` even though the test path (`general_purpose::STANDARD.decode(...)`) uses an inherent method on `GeneralPurpose` and therefore does not require the trait in scope. Anonymous-aliased imports (`Engine as _`) are not re-exported through `use super::*`, so the test module's own import was the source of the unused-trait warning.
- **Fix:** Reduced the test-module import to `use base64::engine::general_purpose;` only.
- **Files modified:** `ferro-wallet/src/qr.rs` (test block only)
- **Verification:** `cargo test -p ferro-wallet --lib qr::tests` exits 0 with no warnings; full `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0.
- **Committed in:** `00007b9f` (Task 2 commit — integrated before the commit landed)

---

**Total deviations:** 2 auto-fixed (both clippy / unused-import lint fixes, no production-surface change).
**Impact on plan:** None — both fixes were strictly internal to the test modules; public API matches the plan exactly.

## Issues Encountered

- `clippy::uninlined_format_args` is enforced as `-D warnings` per CLAUDE.md's lint command; the plan's reference code in the `<action>` block predates the lint and used the older positional `{:?}, centre` form. Future plans should use inlined-arg form in template code blocks.

## Threat Flags

None. The plan's `<threat_model>` flagged no security-relevant surface in this plan, and the implementation introduces none. `image::load_from_memory` returns `Result`, so malformed-input panics are not a vector; `qrcode-generator::to_png_to_vec` is similarly fallible. No secrets, no crypto, no network.

## Verification Gates

- [x] `cargo test -p ferro-wallet --lib images::tests::fit_to_exact_dims_transparent` — exits 0 (ACC-1g)
- [x] `cargo test -p ferro-wallet --lib qr::tests::png_starts_with_png_magic` — exits 0 (ACC-1h)
- [x] `cargo test -p ferro-wallet --lib images::tests` — 6/6 pass
- [x] `cargo test -p ferro-wallet --lib qr::tests` — 3/3 pass
- [x] `cargo test -p ferro-wallet` — 30/30 pass (9 new + 21 from PLAN-01 / PLAN-02 / PLAN-03)
- [x] `cargo build --workspace` — exits 0
- [x] `cargo clippy -p ferro-wallet --all-targets -- -D warnings` — exits 0
- [x] `cargo fmt --all -- --check` — exits 0
- [x] `grep -F 'pub fn fit_to' ferro-wallet/src/images.rs` — one match
- [x] `grep -F 'pub fn apple_logo_set' ferro-wallet/src/images.rs` — one match
- [x] `grep -F 'pub fn apple_icon_set' ferro-wallet/src/images.rs` — one match
- [x] `grep -F 'pub fn google_hero' ferro-wallet/src/images.rs` — one match
- [x] `grep -F 'FilterType::Lanczos3' ferro-wallet/src/images.rs` — one match
- [x] `grep -F 'pub fn png' ferro-wallet/src/qr.rs` — one match
- [x] `grep -F 'pub fn data_uri' ferro-wallet/src/qr.rs` — one match
- [x] `grep -F 'QrCodeEcc::Medium' ferro-wallet/src/qr.rs` — one match

## Next Phase Readiness

- PLAN-05 (apple builder) is unblocked — `apple/package.rs`'s ZIP-assembly loop can now consume `images::apple_logo_set(...)?` + `images::apple_icon_set(None, &logo)?` to produce the six image entries that go alongside `pass.json`, `manifest.json`, and `signature` in the `.pkpass`.
- PLAN-07 (google builder) is unblocked — `google/object.rs`'s `build_event_ticket_object` can now call `images::google_hero(...)?` to populate `heroImage.sourceUri.uri` (after a separate upload step out of scope here) and `qr::data_uri(...)?` to embed a barcode inline.
- The pure-transform discipline of `images` + `qr` (no I/O, no async, no env reads) means both builders can call these helpers from any context — Tokio task, blocking handler, CLI command — without contamination.

---
*Phase: 151-ferro-wallet-crate*
*Completed: 2026-05-11*

## Self-Check: PASSED

- `ferro-wallet/src/images.rs` — FOUND
- `ferro-wallet/src/qr.rs` — FOUND
- `.planning/phases/151-ferro-wallet-crate/151-04-SUMMARY.md` — FOUND
- Commit `c536fd61` (Task 1 — images.rs) — FOUND
- Commit `00007b9f` (Task 2 — qr.rs) — FOUND
