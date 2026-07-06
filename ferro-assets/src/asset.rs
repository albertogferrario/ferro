//! Asset struct, ContentType enum, and extension-based content type inference.

use std::path::Path;

/// Content types recognized by the asset pipeline.
///
/// Variants correspond to transform-relevant media types. All other extensions
/// map to [`ContentType::Other`], which passes through every transform unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    /// HTML document (`.html`, `.htm`).
    Html,
    /// CSS stylesheet (`.css`).
    Css,
    /// JavaScript module or script (`.js`, `.mjs`).
    Js,
    /// JPEG image (`.jpg`, `.jpeg`).
    Jpeg,
    /// PNG image (`.png`).
    Png,
    /// AVIF image (`.avif`).
    Avif,
    /// Catch-all: no transform touches this file; bytes pass through identically.
    Other,
}

/// A single in-memory artifact with a logical path and content-type tag.
#[derive(Debug, Clone)]
pub struct Asset {
    /// Logical artifact path (e.g. `assets/hero.jpg`, `index.html`).
    pub path: String,
    /// Content type, inferred from `path` extension or set explicitly.
    pub content_type: ContentType,
    /// File contents. Uses [`bytes::Bytes`] for cheap clone across transforms.
    pub bytes: bytes::Bytes,
}

impl Asset {
    /// Construct an asset, inferring content type from the path extension.
    pub fn new(path: impl Into<String>, bytes: bytes::Bytes) -> Self {
        let path = path.into();
        let content_type = infer_content_type(&path);
        Self {
            path,
            content_type,
            bytes,
        }
    }

    /// Override the content type after construction.
    ///
    /// Returns `self` for method chaining.
    pub fn with_content_type(mut self, ct: ContentType) -> Self {
        self.content_type = ct;
        self
    }
}

/// Infer content type from path extension.
///
/// Unknown or absent extensions return [`ContentType::Other`].
pub fn infer_content_type(path: &str) -> ContentType {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("html" | "htm") => ContentType::Html,
        Some("css") => ContentType::Css,
        Some("js" | "mjs") => ContentType::Js,
        Some("jpg" | "jpeg") => ContentType::Jpeg,
        Some("png") => ContentType::Png,
        Some("avif") => ContentType::Avif,
        _ => ContentType::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_extensions_infer_html() {
        assert_eq!(infer_content_type("index.html"), ContentType::Html);
        assert_eq!(infer_content_type("page.htm"), ContentType::Html);
    }

    #[test]
    fn css_infers_css() {
        assert_eq!(infer_content_type("a.css"), ContentType::Css);
    }

    #[test]
    fn js_and_mjs_infer_js() {
        assert_eq!(infer_content_type("a.js"), ContentType::Js);
        assert_eq!(infer_content_type("a.mjs"), ContentType::Js);
    }

    #[test]
    fn image_extensions_infer_correctly() {
        assert_eq!(infer_content_type("a.jpg"), ContentType::Jpeg);
        assert_eq!(infer_content_type("a.jpeg"), ContentType::Jpeg);
        assert_eq!(infer_content_type("a.png"), ContentType::Png);
        assert_eq!(infer_content_type("a.avif"), ContentType::Avif);
    }

    #[test]
    fn unknown_and_no_extension_infer_other() {
        assert_eq!(infer_content_type("spec.json"), ContentType::Other);
        assert_eq!(infer_content_type("noext"), ContentType::Other);
        assert_eq!(infer_content_type(""), ContentType::Other);
    }

    #[test]
    fn asset_new_infers_content_type_from_path() {
        let asset = Asset::new("x.css", bytes::Bytes::new());
        assert_eq!(asset.content_type, ContentType::Css);
    }

    #[test]
    fn asset_with_content_type_overrides_inferred() {
        let asset = Asset::new("x.json", bytes::Bytes::new()).with_content_type(ContentType::Html);
        assert_eq!(asset.content_type, ContentType::Html);
    }

    #[test]
    fn error_transform_to_string_contains_all_three_fields() {
        let e = crate::error::Error::transform("html_minify", "index.html", "boom");
        let s = e.to_string();
        assert!(s.contains("html_minify"), "must contain transform name");
        assert!(s.contains("index.html"), "must contain path");
        assert!(s.contains("boom"), "must contain cause");
    }
}
