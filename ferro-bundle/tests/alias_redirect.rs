//! BUNDLE-03 alias 301 redirect integration test.
//!
//! Verifies that a request to a `.with_alias(...)`-registered plain URL returns:
//! - status 301
//! - Location header pointing at the bundle's current hashed URL

use ferro_bundle::Bundle;
use ferro_bundle::__test_internals::serve_inner;

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

#[test]
fn alias_path_redirects_301_to_hashed_url() {
    static BYTES: &[u8] = b"console.log('sdk');";

    // Builder order per crate-level docs: content_type BEFORE with_alias so the
    // alias captures the final hashed URL (which includes the extension).
    let bundle = Bundle::new("alias-redirect-sdk", BYTES)
        .content_type("application/javascript")
        .with_alias("/embed/v1.js");

    let hashed = bundle.hashed_url();

    let resp = serve_inner("/embed/v1.js", None);

    assert_eq!(resp.status_code(), 301);
    assert_eq!(
        header_value(resp.headers(), "Location"),
        Some(hashed.as_str())
    );
}
