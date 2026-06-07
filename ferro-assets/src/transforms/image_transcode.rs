//! Image transcoding transform: AVIF + JPEG responsive variants, bounded by a rayon thread pool.
//!
//! Emits deterministic `{stem}-{width}w.{ext}` variant assets for each configured width that
//! is <= the source image width (never upscales). Non-image assets pass through unchanged.
//!
//! Concurrent encodes are bounded to a configurable limit (default 2) via a rayon
//! [`ThreadPool`](rayon::ThreadPool). No tokio runtime is required.

use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;
use ravif::{Encoder, Img, RGBA8};
use rayon::ThreadPoolBuilder;

use crate::asset::ContentType;
use crate::pipeline::Transform;
use crate::{Asset, Error};

/// Image transcoding transform.
///
/// Accepts `Jpeg`, `Png`, and `Avif` assets. For each source image, emits
/// AVIF and JPEG variants at each configured width that is ≤ the source width.
/// Source images with width smaller than all configured widths emit zero variants.
///
/// Non-image assets pass through byte-identical.
///
/// ## Naming scheme (D-12)
///
/// Variants follow `{stem}-{width}w.{ext}`:
///
/// - `assets/hero.jpg` at width 768 → `assets/hero-768w.avif` + `assets/hero-768w.jpg`
///
/// This scheme is parseable back to `(stem, width, format)` by [`super::ResponsiveImages`].
///
/// ## Concurrency (D-09)
///
/// Encodes run inside a rayon [`ThreadPool`](rayon::ThreadPool) with `num_threads` set to
/// `max_concurrent` (default 2). At most that many image encodes run in parallel, bounding
/// peak memory on small instances.
#[derive(Debug, Clone)]
pub struct ImageTranscode {
    /// Maximum number of concurrent image encodes. Default: 2.
    max_concurrent: usize,
    /// Responsive widths to emit. Only widths <= source width are emitted. Default: [480, 768, 1200, 1920].
    widths: Vec<u32>,
    /// AVIF quality (0–100). Default: 70.0.
    avif_quality: f32,
    /// AVIF encoding speed (1 = slowest/best, 10 = fastest/worst). Default: 4.
    avif_speed: u8,
    /// JPEG quality (0–100). Default: 80.
    jpeg_quality: u8,
}

impl ImageTranscode {
    /// Create a new `ImageTranscode` with default settings.
    ///
    /// Defaults: max_concurrent=2, widths=[480,768,1200,1920], avif_quality=70.0, avif_speed=4, jpeg_quality=80.
    ///
    /// Speed 4 is chosen deliberately over the ravif default (speed=1) to avoid 10–30 second
    /// encode times per image during publish jobs (see RESEARCH.md Pitfall 3).
    pub fn new() -> Self {
        Self {
            max_concurrent: 2,
            widths: vec![480, 768, 1200, 1920],
            avif_quality: 70.0,
            avif_speed: 4,
            jpeg_quality: 80,
        }
    }

    /// Set the maximum number of concurrent image encodes.
    pub fn with_max_concurrent(mut self, n: usize) -> Self {
        self.max_concurrent = n;
        self
    }

    /// Set the responsive widths to emit.
    ///
    /// Only widths <= the source image width are emitted (no upscaling).
    pub fn with_widths(mut self, widths: Vec<u32>) -> Self {
        self.widths = widths;
        self
    }

    /// Set the AVIF encode quality (0–100).
    pub fn with_avif_quality(mut self, q: f32) -> Self {
        self.avif_quality = q;
        self
    }

    /// Set the AVIF encode speed (1 = slowest/best quality, 10 = fastest).
    pub fn with_avif_speed(mut self, s: u8) -> Self {
        self.avif_speed = s;
        self
    }

    /// Set the JPEG encode quality (0–100).
    pub fn with_jpeg_quality(mut self, q: u8) -> Self {
        self.jpeg_quality = q;
        self
    }

    /// Transcode a single source image asset into AVIF+JPEG variants.
    ///
    /// Returns a list of emitted variant assets (may be empty if no configured width <=
    /// source width). The original source asset is also included in the returned list so
    /// the JPEG fallback `<img src>` still resolves.
    fn transcode_image(&self, asset: Asset) -> Result<Vec<Asset>, Error> {
        let src = image::load_from_memory(&asset.bytes)
            .map_err(|e| Error::transform("image_transcode", &asset.path, e.to_string()))?;

        let stem = stem_from_path(&asset.path);
        let dir_prefix = dir_prefix_from_path(&asset.path);

        let mut result = vec![asset.clone()]; // retain original so fallback <img src> resolves

        // D-11 / Pitfall 4: width filter runs FIRST — never upscale
        let src_width = src.width();
        for &width in self.widths.iter().filter(|&&w| w <= src_width) {
            let resized = resize_to_width(&src, width);

            // Emit AVIF variant
            let avif_bytes = encode_avif(&resized, self.avif_quality, self.avif_speed)
                .map_err(|e| Error::transform("image_transcode", &asset.path, e))?;
            let avif_path = format!("{dir_prefix}{stem}-{width}w.avif");
            result.push(Asset {
                path: avif_path,
                content_type: ContentType::Avif,
                bytes: bytes::Bytes::from(avif_bytes),
            });

            // Emit JPEG variant
            let jpeg_bytes = encode_jpeg(&resized, self.jpeg_quality)
                .map_err(|e| Error::transform("image_transcode", &asset.path, e))?;
            let jpeg_path = format!("{dir_prefix}{stem}-{width}w.jpg");
            result.push(Asset {
                path: jpeg_path,
                content_type: ContentType::Jpeg,
                bytes: bytes::Bytes::from(jpeg_bytes),
            });
        }

        Ok(result)
    }
}

impl Default for ImageTranscode {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform for ImageTranscode {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.max_concurrent)
            .build()
            .map_err(|e| Error::setup(e.to_string()))?;

        // Partition into images vs other assets
        let (images, others): (Vec<_>, Vec<_>) = assets.into_iter().partition(|a| {
            matches!(
                a.content_type,
                ContentType::Jpeg | ContentType::Png | ContentType::Avif
            )
        });

        // Non-image assets pass through unchanged
        let mut output: Vec<Asset> = others;

        // pool.install bounds parallelism to max_concurrent threads (D-09)
        let variants: Result<Vec<Vec<Asset>>, Error> = pool.install(|| {
            images
                .into_iter()
                .map(|a| self.transcode_image(a))
                .collect()
        });

        for group in variants? {
            output.extend(group);
        }

        Ok(output)
    }
}

/// Resize `src` to `width`, preserving aspect ratio via Lanczos3 (D-11).
fn resize_to_width(src: &DynamicImage, width: u32) -> DynamicImage {
    let height = ((src.height() as f64 * width as f64) / src.width() as f64).round() as u32;
    let height = height.max(1);
    src.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
}

/// Encode `img` to AVIF bytes using ravif.
fn encode_avif(img: &DynamicImage, quality: f32, speed: u8) -> Result<Vec<u8>, String> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    // Reinterpret the raw u8 RGBA buffer as &[RGBA8] by chunking into 4-byte groups.
    let raw = rgba.as_raw();
    let pixels: Vec<RGBA8> = raw
        .chunks_exact(4)
        .map(|c| RGBA8 {
            r: c[0],
            g: c[1],
            b: c[2],
            a: c[3],
        })
        .collect();
    let encoded = Encoder::new()
        .with_quality(quality)
        .with_speed(speed)
        .encode_rgba(Img::new(&pixels, width as usize, height as usize))
        .map_err(|e| e.to_string())?;
    Ok(encoded.avif_file)
}

/// Encode `img` to JPEG bytes at the given quality.
fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();
    let mut out: Vec<u8> = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut out, quality);
    encoder
        .encode(rgb.as_raw(), width, height, image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// Extract the file stem from a path (e.g. `assets/hero.jpg` → `hero`).
fn stem_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("asset")
        .to_string()
}

/// Extract the directory prefix from a path, with trailing slash (e.g. `assets/hero.jpg` → `assets/`).
/// Returns `""` if the file is at the root.
fn dir_prefix_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .parent()
        .and_then(|p| p.to_str())
        .filter(|s| !s.is_empty())
        .map(|s| format!("{s}/"))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn make_jpeg_asset(path: &str, width: u32, height: u32) -> Asset {
        // Generate a minimal JPEG in memory using the image crate.
        let img = image::DynamicImage::ImageRgb8(image::RgbImage::new(width, height));
        let mut out = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut out, 75);
        encoder
            .encode(
                img.to_rgb8().as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgb8,
            )
            .expect("test jpeg encode");
        Asset::new(path, Bytes::from(out))
    }

    fn make_png_asset(path: &str, width: u32, height: u32) -> Asset {
        let img = image::RgbaImage::new(width, height);
        let dyn_img = image::DynamicImage::ImageRgba8(img);
        let mut out = Vec::new();
        dyn_img
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .expect("test png encode");
        Asset::new(path, Bytes::from(out))
    }

    #[test]
    fn default_config_has_expected_values() {
        let t = ImageTranscode::new();
        assert_eq!(t.max_concurrent, 2);
        assert_eq!(t.widths, vec![480u32, 768, 1200, 1920]);
        assert!((t.avif_quality - 70.0_f32).abs() < f32::EPSILON);
        assert_eq!(t.avif_speed, 4);
        assert_eq!(t.jpeg_quality, 80);
    }

    #[test]
    fn builder_methods_override_defaults() {
        let t = ImageTranscode::new()
            .with_max_concurrent(4)
            .with_widths(vec![320, 640])
            .with_avif_quality(80.0)
            .with_avif_speed(6)
            .with_jpeg_quality(90);
        assert_eq!(t.max_concurrent, 4);
        assert_eq!(t.widths, vec![320u32, 640]);
        assert!((t.avif_quality - 80.0_f32).abs() < f32::EPSILON);
        assert_eq!(t.avif_speed, 6);
        assert_eq!(t.jpeg_quality, 90);
    }

    #[test]
    fn no_upscale_when_source_narrower_than_all_widths() {
        // 400px source with default widths [480,768,1200,1920] → ZERO variants (Pitfall 4)
        let asset = make_jpeg_asset("hero.jpg", 400, 300);
        let t = ImageTranscode::new(); // default widths all > 400
        let result = t.run(vec![asset]).expect("run must succeed");
        // Only the original asset should be present — no variants
        assert_eq!(result.len(), 1, "narrow source must emit zero variants");
        assert_eq!(result[0].path, "hero.jpg");
    }

    #[test]
    fn no_upscale_png_source() {
        let asset = make_png_asset("icon.png", 200, 200);
        let t = ImageTranscode::new();
        let result = t.run(vec![asset]).expect("run must succeed");
        assert_eq!(
            result.len(),
            1,
            "200px png with default widths must emit zero variants"
        );
    }

    #[test]
    fn variant_naming_follows_d12_scheme() {
        // Use a small custom width list so the source qualifies
        let asset = make_jpeg_asset("assets/hero.jpg", 800, 600);
        let t = ImageTranscode::new().with_widths(vec![400]);
        let result = t.run(vec![asset]).expect("run must succeed");
        // Should have original + 2 variants (AVIF + JPEG at 400)
        assert_eq!(
            result.len(),
            3,
            "800px source at width 400 should emit 2 variants"
        );
        let paths: Vec<&str> = result.iter().map(|a| a.path.as_str()).collect();
        assert!(
            paths.contains(&"assets/hero.jpg"),
            "original must be retained"
        );
        assert!(
            paths.contains(&"assets/hero-400w.avif"),
            "AVIF variant naming: {paths:?}"
        );
        assert!(
            paths.contains(&"assets/hero-400w.jpg"),
            "JPEG variant naming: {paths:?}"
        );
    }

    #[test]
    fn root_level_asset_naming() {
        let asset = make_jpeg_asset("photo.jpg", 800, 600);
        let t = ImageTranscode::new().with_widths(vec![400]);
        let result = t.run(vec![asset]).expect("run must succeed");
        let paths: Vec<&str> = result.iter().map(|a| a.path.as_str()).collect();
        assert!(
            paths.contains(&"photo-400w.avif"),
            "root asset avif: {paths:?}"
        );
        assert!(
            paths.contains(&"photo-400w.jpg"),
            "root asset jpeg: {paths:?}"
        );
    }

    #[test]
    fn non_image_assets_pass_through_unchanged() {
        let html = Asset::new("index.html", Bytes::from_static(b"<html></html>"));
        let css = Asset::new("style.css", Bytes::from_static(b"body{}"));
        let json = Asset::new("spec.json", Bytes::from_static(b"{}"));
        let t = ImageTranscode::new();
        let result = t
            .run(vec![html.clone(), css.clone(), json.clone()])
            .expect("run must succeed");
        assert_eq!(result.len(), 3, "non-image assets must all pass through");
        assert_eq!(result[0].bytes, html.bytes);
        assert_eq!(result[1].bytes, css.bytes);
        assert_eq!(result[2].bytes, json.bytes);
    }

    #[test]
    fn malformed_image_bytes_return_err() {
        let asset = Asset::new("bad.jpg", Bytes::from_static(b"not a jpeg"));
        let t = ImageTranscode::new();
        let result = t.run(vec![asset]);
        assert!(
            result.is_err(),
            "malformed image must return Err, not panic"
        );
    }

    #[test]
    fn max_concurrent_configuration_builds_pool() {
        // Verify that a custom max_concurrent doesn't fail pool construction
        let asset = Asset::new("spec.json", Bytes::from_static(b"{}"));
        let t = ImageTranscode::new().with_max_concurrent(1);
        let result = t.run(vec![asset]);
        assert!(result.is_ok(), "custom max_concurrent must succeed");
    }

    #[test]
    fn variant_content_types_are_correct() {
        let asset = make_jpeg_asset("hero.jpg", 800, 600);
        let t = ImageTranscode::new().with_widths(vec![400]);
        let result = t.run(vec![asset]).expect("run must succeed");
        for a in &result {
            if a.path.ends_with(".avif") {
                assert_eq!(
                    a.content_type,
                    ContentType::Avif,
                    "avif variant must have Avif content type"
                );
            } else if a.path.ends_with("-400w.jpg") {
                assert_eq!(
                    a.content_type,
                    ContentType::Jpeg,
                    "jpeg variant must have Jpeg content type"
                );
            }
        }
    }

    #[test]
    fn stem_extraction_works_correctly() {
        assert_eq!(stem_from_path("hero.jpg"), "hero");
        assert_eq!(stem_from_path("assets/hero.jpg"), "hero");
        assert_eq!(stem_from_path("a/b/c/image.png"), "image");
        assert_eq!(stem_from_path("noext"), "noext");
    }

    #[test]
    fn dir_prefix_extraction_works_correctly() {
        assert_eq!(dir_prefix_from_path("hero.jpg"), "");
        assert_eq!(dir_prefix_from_path("assets/hero.jpg"), "assets/");
        assert_eq!(dir_prefix_from_path("a/b/c/image.png"), "a/b/c/");
    }

    #[test]
    fn only_widths_lte_source_are_emitted() {
        // Source is 1000px wide; configured widths 500 and 1200; only 500 should be emitted
        let asset = make_jpeg_asset("hero.jpg", 1000, 500);
        let t = ImageTranscode::new().with_widths(vec![500, 1200]);
        let result = t.run(vec![asset]).expect("run must succeed");
        // original + AVIF at 500 + JPEG at 500 = 3; no 1200w variants
        assert_eq!(
            result.len(),
            3,
            "only widths <= source.width() should emit variants"
        );
        let paths: Vec<&str> = result.iter().map(|a| a.path.as_str()).collect();
        assert!(
            paths.contains(&"hero-500w.avif"),
            "500w avif should be present"
        );
        assert!(
            paths.contains(&"hero-500w.jpg"),
            "500w jpg should be present"
        );
        assert!(
            !paths.iter().any(|p| p.contains("1200w")),
            "1200w variants must not be emitted"
        );
    }
}
