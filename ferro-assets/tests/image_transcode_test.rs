//! SC-3: image_transcode integration tests — variant emission, no-upscale, bounded concurrency.
//!
//! - Always-on light test: a 400px-wide source emits zero variants (no upscale guarantee).
//! - Heavy test (cfg-gated `slow-tests` feature): a real source image through ImageTranscode
//!   emits the expected AVIF+JPEG variant count and the AVIF variants decode as valid AVIF.
//!
//! Run: `cargo test -p ferro-assets --test image_transcode_test`
//! Heavy: `cargo test -p ferro-assets --test image_transcode_test --features slow-tests`

use bytes::Bytes;
use ferro_assets::{
    transforms::{ImageTranscode, Transform},
    Asset, ContentType,
};

/// Create a minimal in-memory JPEG asset of the given dimensions.
fn make_jpeg_asset(path: &str, width: u32, height: u32) -> Asset {
    use image::codecs::jpeg::JpegEncoder;
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

// ── Always-on light tests ────────────────────────────────────────────────────

#[test]
fn no_upscale_when_source_narrower_than_all_widths() {
    // SC-3 light: 400px source with default widths [480,768,1200,1920] → ZERO variants.
    // This is the no-upscale guard (Pitfall 4 / D-11 / T-187-12).
    let asset = make_jpeg_asset("hero.jpg", 400, 300);
    let t = ferro_assets::transforms::ImageTranscode::new(); // default widths all > 400

    let result = t
        .run(vec![asset])
        .expect("run must succeed on a narrow source");

    assert_eq!(
        result.len(),
        1,
        "a 400px source with default widths must emit ZERO variants (no upscaling)"
    );
    assert_eq!(
        result[0].path, "hero.jpg",
        "original asset must be retained"
    );
}

#[test]
fn exact_source_width_emits_one_width() {
    // Source is exactly 480px — the first default width — should emit variants at 480 only.
    let asset = make_jpeg_asset("banner.jpg", 480, 320);
    let t = ImageTranscode::new(); // default widths [480,768,1200,1920]

    let result = t.run(vec![asset]).expect("run must succeed");

    // original + AVIF@480 + JPEG@480 = 3
    assert_eq!(
        result.len(),
        3,
        "480px source must emit exactly one width (480w): {:#?}",
        result.iter().map(|a| &a.path).collect::<Vec<_>>()
    );
    let paths: Vec<&str> = result.iter().map(|a| a.path.as_str()).collect();
    assert!(
        paths.contains(&"banner-480w.avif"),
        "avif 480w must be present"
    );
    assert!(
        paths.contains(&"banner-480w.jpg"),
        "jpeg 480w must be present"
    );
    // No wider variants
    assert!(
        !paths
            .iter()
            .any(|p| p.contains("768w") || p.contains("1200w") || p.contains("1920w")),
        "no wider variants must be emitted: {paths:?}"
    );
}

#[test]
fn variant_names_follow_d12_scheme() {
    let asset = make_jpeg_asset("assets/hero.jpg", 800, 600);
    let t = ImageTranscode::new().with_widths(vec![400]);

    let result = t.run(vec![asset]).expect("run must succeed");
    let paths: Vec<&str> = result.iter().map(|a| a.path.as_str()).collect();

    // D-12: {stem}-{width}w.{ext}
    assert!(
        paths.contains(&"assets/hero-400w.avif"),
        "AVIF D-12 name: {paths:?}"
    );
    assert!(
        paths.contains(&"assets/hero-400w.jpg"),
        "JPEG D-12 name: {paths:?}"
    );
}

#[test]
fn variant_assets_have_correct_content_types() {
    let asset = make_jpeg_asset("hero.jpg", 800, 600);
    let t = ImageTranscode::new().with_widths(vec![400]);

    let result = t.run(vec![asset]).expect("run must succeed");

    for a in &result {
        if a.path.ends_with(".avif") {
            assert_eq!(
                a.content_type,
                ContentType::Avif,
                "avif must have Avif type"
            );
        }
        if a.path.ends_with("-400w.jpg") {
            assert_eq!(
                a.content_type,
                ContentType::Jpeg,
                "jpeg variant must have Jpeg type"
            );
        }
    }
}

// ── Heavy encode tests (cfg-gated behind `slow-tests` feature) ───────────────

#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore)]
fn avif_and_jpeg_variants_emitted_for_all_configured_widths() {
    // SC-3 heavy: a real 2000×1500 source through ImageTranscode at default widths.
    // All four default widths (480, 768, 1200, 1920) are <= 2000, so 8 variants emitted.
    // Source image is generated in-test — no binary fixture committed.
    let width: u32 = 2000;
    let height: u32 = 1500;
    let asset = make_jpeg_asset("landscape.jpg", width, height);

    let t = ImageTranscode::new(); // default widths [480,768,1200,1920], speed=4

    let result = t.run(vec![asset]).expect("heavy transcode must succeed");

    // 1 original + 4 AVIF + 4 JPEG = 9
    assert_eq!(
        result.len(),
        9,
        "2000px source must emit 4 AVIF + 4 JPEG variants: {:#?}",
        result.iter().map(|a| &a.path).collect::<Vec<_>>()
    );

    // Verify every AVIF variant decodes as a valid image (T-187-10 mitigation check)
    for a in result
        .iter()
        .filter(|a| a.content_type == ContentType::Avif)
    {
        image::load_from_memory(&a.bytes)
            .unwrap_or_else(|e| panic!("AVIF variant {} must decode as valid image: {e}", a.path));
    }

    // Verify expected variant names exist
    let paths: Vec<&str> = result.iter().map(|a| a.path.as_str()).collect();
    for w in [480u32, 768, 1200, 1920] {
        assert!(
            paths.contains(&format!("landscape-{w}w.avif").as_str()),
            "avif {w}w must be present: {paths:?}"
        );
        assert!(
            paths.contains(&format!("landscape-{w}w.jpg").as_str()),
            "jpeg {w}w must be present: {paths:?}"
        );
    }
}

#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore)]
fn bounded_concurrency_with_multiple_images() {
    // SC-3 heavy: process 4 images with max_concurrent=2 — verifies the pool does not panic
    // and all variants are returned correctly when two images process simultaneously.
    let images: Vec<Asset> = (0..4)
        .map(|i| make_jpeg_asset(&format!("img{i}.jpg"), 600, 400))
        .collect();

    let t = ImageTranscode::new()
        .with_max_concurrent(2)
        .with_widths(vec![300]); // only one width per image to keep test fast

    let result = t
        .run(images)
        .expect("bounded pool must complete all images");

    // 4 originals + 4 AVIF@300 + 4 JPEG@300 = 12
    assert_eq!(
        result.len(),
        12,
        "4 images × (1 orig + 1 avif + 1 jpeg) = 12: {:#?}",
        result.iter().map(|a| &a.path).collect::<Vec<_>>()
    );
}
