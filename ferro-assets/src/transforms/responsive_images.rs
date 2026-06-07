//! Responsive images transform: rewrites `<img src>` to `<picture><source>` using discovered variants.
//!
//! Scans the asset set for variants emitted by [`super::ImageTranscode`] using the deterministic
//! `{stem}-{width}w.{ext}` naming scheme (D-12), then rewrites each matching `<img>` element in
//! HTML assets to a `<picture>` wrapper with an AVIF `<source srcset>` and the original JPEG as
//! the fallback `<img>`. Assets with no discovered variants are left unchanged.

use std::collections::HashMap;

use bytes::Bytes;
use lol_html::{element, html_content::ContentType as LolContentType, HtmlRewriter, Settings};

use crate::pipeline::Transform;
use crate::{map_matching, Asset, ContentType, Error};

/// Rewrites `<img src>` elements to `<picture>` wrappers using AVIF variants discovered in the
/// asset set.
///
/// Run this transform **after** [`super::ImageTranscode`] in the pipeline so the variant assets
/// are already present in the set when `ResponsiveImages` scans for them.
///
/// ## How discovery works (D-12 round-trip)
///
/// For each non-HTML asset whose name matches `{stem}-{width}w.avif`, the stem and width are
/// parsed out. When an `<img src="hero.jpg">` is encountered and `hero` has AVIF variants in the
/// index, the element is wrapped:
///
/// ```html
/// <picture>
///   <source type="image/avif" srcset="hero-480w.avif 480w, hero-768w.avif 768w">
///   <img src="hero.jpg" ...>
/// </picture>
/// ```
///
/// If no variants exist for a given `<img src>`, the element is left unchanged.
///
/// Non-HTML assets pass through byte-identical.
#[derive(Debug, Clone)]
pub struct ResponsiveImages;

impl ResponsiveImages {
    /// Create a new `ResponsiveImages` transform.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResponsiveImages {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform for ResponsiveImages {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        // Build variant index: stem → sorted list of (width, avif_path)
        // Parse every asset that matches {stem}-{width}w.avif using D-12 naming.
        let avif_index = build_avif_index(&assets);

        map_matching(assets, &[ContentType::Html], |a| {
            let out = rewrite_img_to_picture(&a.bytes, &avif_index)
                .map_err(|e| Error::transform("responsive_images", &a.path, e))?;
            Ok(Asset {
                bytes: Bytes::from(out),
                ..a
            })
        })
    }
}

/// Parse variant assets and build a stem → Vec<(width, avif_path)> index.
///
/// Only AVIF variants are indexed (JPEG fallback is the original `<img src>`).
/// Variants are sorted by width ascending for deterministic srcset output.
fn build_avif_index(assets: &[Asset]) -> HashMap<String, Vec<(u32, String)>> {
    let mut index: HashMap<String, Vec<(u32, String)>> = HashMap::new();

    for asset in assets {
        if asset.content_type != ContentType::Avif {
            continue;
        }
        if let Some((stem, width)) = parse_variant_name(&asset.path) {
            index
                .entry(stem)
                .or_default()
                .push((width, asset.path.clone()));
        }
    }

    // Sort each stem's variants by width ascending for deterministic output
    for variants in index.values_mut() {
        variants.sort_by_key(|(w, _)| *w);
    }

    index
}

/// Parse `{stem}-{width}w.avif` from a path.
///
/// Returns `(stem, width)` if the path matches, or `None` otherwise.
/// The stem is extracted without directory prefix so it matches the logical
/// stem used when looking up `<img src="stem.jpg">`.
///
/// Examples:
/// - `hero-768w.avif` → `("hero", 768)`
/// - `assets/hero-480w.avif` → `("hero", 480)` — stripped of dir prefix
/// - `hero.avif` → `None` (no `-{n}w` suffix)
fn parse_variant_name(path: &str) -> Option<(String, u32)> {
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())?;

    // Must end in .avif
    let base = filename.strip_suffix(".avif")?;

    // Find the last `-` separator before the `{n}w` suffix
    let dash_pos = base.rfind('-')?;
    let suffix = &base[dash_pos + 1..];

    // suffix must be "{digits}w"
    let width_str = suffix.strip_suffix('w')?;
    let width: u32 = width_str.parse().ok()?;

    let stem = base[..dash_pos].to_string();
    if stem.is_empty() {
        return None;
    }

    Some((stem, width))
}

/// Rewrite `<img>` elements in HTML to `<picture>` wrappers using the AVIF index.
fn rewrite_img_to_picture(
    input: &[u8],
    avif_index: &HashMap<String, Vec<(u32, String)>>,
) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(input.len());

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("img", |el| {
                let src = match el.get_attribute("src") {
                    Some(s) => s,
                    None => return Ok(()),
                };

                // Derive stem from the src filename (strip dir + extension)
                let stem = match img_src_to_stem(&src) {
                    Some(s) => s,
                    None => return Ok(()),
                };

                // Look up AVIF variants for this stem
                let variants = match avif_index.get(&stem) {
                    Some(v) if !v.is_empty() => v,
                    _ => return Ok(()), // no variants — leave <img> unchanged
                };

                // Build srcset string: "hero-480w.avif 480w, hero-768w.avif 768w, ..."
                let srcset: String = variants
                    .iter()
                    .map(|(w, p)| format!("{p} {w}w"))
                    .collect::<Vec<_>>()
                    .join(", ");

                // Wrap: <picture><source type="image/avif" srcset="..."><img ...></picture>
                el.before("<picture>", LolContentType::Html);
                el.before(
                    &format!(r#"<source type="image/avif" srcset="{srcset}">"#),
                    LolContentType::Html,
                );
                el.after("</picture>", LolContentType::Html);

                Ok(())
            })],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(input).map_err(|e| e.to_string())?;
    rewriter.end().map_err(|e| e.to_string())?;

    Ok(output)
}

/// Extract the stem from an `<img src>` value.
///
/// `"hero.jpg"` → `"hero"`, `"assets/hero.jpg"` → `"hero"`, `"noext"` → `None`.
fn img_src_to_stem(src: &str) -> Option<String> {
    let stem = std::path::Path::new(src)
        .file_stem()
        .and_then(|s| s.to_str())?;
    if stem.is_empty() {
        return None;
    }
    Some(stem.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, ContentType};

    fn avif_asset(path: &str) -> Asset {
        Asset {
            path: path.to_string(),
            content_type: ContentType::Avif,
            bytes: Bytes::from_static(b"fake avif"),
        }
    }

    fn html_asset(path: &str, html: &'static [u8]) -> Asset {
        Asset::new(path, Bytes::from_static(html))
    }

    #[test]
    fn parse_variant_name_parses_d12_names() {
        assert_eq!(
            parse_variant_name("hero-768w.avif"),
            Some(("hero".to_string(), 768))
        );
        assert_eq!(
            parse_variant_name("assets/hero-480w.avif"),
            Some(("hero".to_string(), 480))
        );
        assert_eq!(
            parse_variant_name("photo-1200w.avif"),
            Some(("photo".to_string(), 1200))
        );
    }

    #[test]
    fn parse_variant_name_rejects_non_variant_names() {
        assert_eq!(parse_variant_name("hero.avif"), None);
        assert_eq!(parse_variant_name("hero-notnum.avif"), None);
        assert_eq!(parse_variant_name("hero.jpg"), None);
        assert_eq!(parse_variant_name("-768w.avif"), None); // empty stem
    }

    #[test]
    fn img_src_to_stem_extracts_stem() {
        assert_eq!(img_src_to_stem("hero.jpg"), Some("hero".to_string()));
        assert_eq!(img_src_to_stem("assets/hero.jpg"), Some("hero".to_string()));
        assert_eq!(img_src_to_stem("noext"), Some("noext".to_string()));
    }

    #[test]
    fn img_with_known_variants_is_rewritten_to_picture() {
        let avif_480 = avif_asset("hero-480w.avif");
        let avif_768 = avif_asset("hero-768w.avif");
        let html = html_asset(
            "index.html",
            b"<html><body><img src=\"hero.jpg\"></body></html>",
        );

        let t = ResponsiveImages::new();
        let result = t
            .run(vec![avif_480, avif_768, html])
            .expect("run must succeed");

        let html_out = result
            .iter()
            .find(|a| a.path == "index.html")
            .expect("html asset must be present");
        let text = std::str::from_utf8(&html_out.bytes).expect("valid utf8");

        assert!(text.contains("<picture>"), "must contain <picture>: {text}");
        assert!(
            text.contains("image/avif"),
            "must contain image/avif: {text}"
        );
        assert!(
            text.contains("hero-480w.avif 480w"),
            "must contain 480w entry: {text}"
        );
        assert!(
            text.contains("hero-768w.avif 768w"),
            "must contain 768w entry: {text}"
        );
        assert!(
            text.contains("</picture>"),
            "must contain </picture>: {text}"
        );
        // Original <img src> must still be present as fallback
        assert!(
            text.contains(r#"src="hero.jpg""#),
            "original img src must be retained: {text}"
        );
    }

    #[test]
    fn img_with_no_variants_is_left_unchanged() {
        let html = html_asset(
            "index.html",
            b"<html><body><img src=\"unknown.jpg\"></body></html>",
        );
        let t = ResponsiveImages::new();
        let result = t.run(vec![html]).expect("run must succeed");
        let text = std::str::from_utf8(&result[0].bytes).expect("valid utf8");
        assert!(
            !text.contains("<picture>"),
            "no variants → no picture wrapper: {text}"
        );
        assert!(
            text.contains(r#"src="unknown.jpg""#),
            "original img must be unchanged: {text}"
        );
    }

    #[test]
    fn non_html_assets_pass_through_unchanged() {
        let json = Asset::new("spec.json", Bytes::from_static(b"{}"));
        let css = Asset::new("style.css", Bytes::from_static(b"body{}"));
        let t = ResponsiveImages::new();
        let result = t
            .run(vec![json.clone(), css.clone()])
            .expect("run must succeed");
        assert_eq!(result[0].bytes, json.bytes);
        assert_eq!(result[1].bytes, css.bytes);
    }

    #[test]
    fn avif_variants_are_sorted_by_width_in_srcset() {
        // Add variants out-of-order to verify sorted output
        let avif_1200 = avif_asset("hero-1200w.avif");
        let avif_480 = avif_asset("hero-480w.avif");
        let avif_768 = avif_asset("hero-768w.avif");
        let html = html_asset(
            "index.html",
            b"<html><body><img src=\"hero.jpg\"></body></html>",
        );

        let t = ResponsiveImages::new();
        let result = t
            .run(vec![avif_1200, avif_480, avif_768, html])
            .expect("run must succeed");

        let html_out = result
            .iter()
            .find(|a| a.path == "index.html")
            .expect("html asset must be present");
        let text = std::str::from_utf8(&html_out.bytes).expect("valid utf8");

        // srcset must be sorted: 480w before 768w before 1200w
        let pos_480 = text.find("480w").expect("must contain 480w");
        let pos_768 = text.find("768w").expect("must contain 768w");
        let pos_1200 = text.find("1200w").expect("must contain 1200w");
        assert!(pos_480 < pos_768, "480w must appear before 768w");
        assert!(pos_768 < pos_1200, "768w must appear before 1200w");
    }

    #[test]
    fn build_avif_index_groups_by_stem() {
        let assets = vec![
            avif_asset("hero-480w.avif"),
            avif_asset("hero-768w.avif"),
            avif_asset("photo-480w.avif"),
        ];
        let index = build_avif_index(&assets);
        assert_eq!(index["hero"].len(), 2);
        assert_eq!(index["photo"].len(), 1);
    }
}
