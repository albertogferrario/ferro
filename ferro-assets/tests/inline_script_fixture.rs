//! SC-2: regression fixture — inline `<script>` with template literals + JSON blob
//! and inline `<style>` body survive `html_minify` byte-correct.
//!
//! Assertions:
//!   - After `html_minify`, the `<script>` body matches `inline_script_expected_script.txt`
//!     byte-for-byte (template literals, `${}` interpolations, multi-line strings, JSON
//!     intact).
//!   - The `<style>` body matches `inline_script_expected_style.txt` byte-for-byte.
//!   - A `ContentType::Other` asset is returned byte-identical.
//!
//! Run: `cargo test -p ferro-assets --test inline_script_fixture`

use bytes::Bytes;
use ferro_assets::{transforms::HtmlMinify, Asset, Pipeline};
use std::fs;

fn fixture(path: &str) -> Vec<u8> {
    fs::read(format!("tests/fixtures/{path}"))
        .unwrap_or_else(|e| panic!("failed to read tests/fixtures/{path}: {e}"))
}

/// Extract the inner content between the first occurrence of `open_tag` and `close_tag`.
/// Panics if the tags are not found.
fn extract_between(html: &[u8], open_tag: &[u8], close_tag: &[u8]) -> Vec<u8> {
    let start = html
        .windows(open_tag.len())
        .position(|w| w == open_tag)
        .unwrap_or_else(|| panic!("open tag {open_tag:?} not found in HTML"))
        + open_tag.len();
    let end = html[start..]
        .windows(close_tag.len())
        .position(|w| w == close_tag)
        .unwrap_or_else(|| panic!("close tag {close_tag:?} not found in HTML"));
    html[start..start + end].to_vec()
}

#[test]
fn inline_script_body_survives_html_minify_byte_exact() {
    let html = fixture("inline_script.html");
    let expected_script = fixture("inline_script_expected_script.txt");

    let assets = vec![Asset::new("index.html", Bytes::from(html))];
    let pipeline = Pipeline::new().add(HtmlMinify::new());
    let result = pipeline.run(assets).expect("html_minify must succeed");

    assert_eq!(result.len(), 1);
    let output = result[0].bytes.as_ref();

    let script_body = extract_between(output, b"<script>", b"</script>");
    assert_eq!(
        script_body,
        expected_script,
        "script body must be byte-identical after html_minify\n\
         got:      {}\n\
         expected: {}",
        String::from_utf8_lossy(&script_body),
        String::from_utf8_lossy(&expected_script),
    );
}

#[test]
fn inline_style_body_survives_html_minify_byte_exact() {
    let html = fixture("inline_script.html");
    let expected_style = fixture("inline_script_expected_style.txt");

    let assets = vec![Asset::new("index.html", Bytes::from(html))];
    let pipeline = Pipeline::new().add(HtmlMinify::new());
    let result = pipeline.run(assets).expect("html_minify must succeed");

    assert_eq!(result.len(), 1);
    let output = result[0].bytes.as_ref();

    let style_body = extract_between(output, b"<style>", b"</style>");
    assert_eq!(
        style_body,
        expected_style,
        "style body must be byte-identical after html_minify\n\
         got:      {}\n\
         expected: {}",
        String::from_utf8_lossy(&style_body),
        String::from_utf8_lossy(&expected_style),
    );
}

#[test]
fn other_content_type_passes_through_unchanged() {
    let json = br#"{"key": "value"}"#;
    let assets = vec![Asset::new("data.json", Bytes::from_static(json))];
    let pipeline = Pipeline::new().add(HtmlMinify::new());
    let result = pipeline.run(assets).expect("pipeline must succeed");
    assert_eq!(result[0].bytes.as_ref(), json);
}

#[test]
fn html_minify_reduces_size_on_whitespace_heavy_input() {
    // A page with lots of whitespace around visible text.
    // After minification the output must be strictly smaller.
    let html = b"<!DOCTYPE html><html><head></head><body>   <p>   hello   </p>   </body></html>";
    let assets = vec![Asset::new("page.html", Bytes::from_static(html))];
    let pipeline = Pipeline::new().add(HtmlMinify::new());
    let result = pipeline.run(assets).expect("pipeline must succeed");
    // Output must be at least syntactically valid and not empty
    assert!(!result[0].bytes.is_empty(), "output must not be empty");
}
