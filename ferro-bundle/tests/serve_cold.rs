//! BUNDLE-02 cold path integration test.
//!
//! Verifies that a registered bundle dispatched via `serve_inner` returns:
//! - status 200
//! - Content-Type matching the caller-supplied `.content_type(...)`
//! - Cache-Control: public, max-age=31536000, immutable
//! - ETag: quoted full SHA-256 hex (RFC 7232 §2.3)
//! - body == registered bytes
//!
//! Each integration test file is compiled into its own binary by cargo; OS-level
//! process isolation prevents registry leakage to other test files.

use ferro_bundle::Bundle;
use ferro_bundle::__test_internals::serve_inner;

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

#[test]
fn serve_cold_returns_200_with_cache_headers() {
    static BYTES: &[u8] = b"console.log('sdk');";

    let bundle = Bundle::new("serve-cold-sdk", BYTES).content_type("application/javascript");
    let hashed = bundle.hashed_url();
    assert!(
        hashed.starts_with("/bundles/serve-cold-sdk."),
        "unexpected hashed URL: {hashed}"
    );
    assert!(
        hashed.ends_with(".js"),
        "expected .js extension; got {hashed}"
    );

    let resp = serve_inner(&hashed, None);

    assert_eq!(resp.status_code(), 200);

    let headers = resp.headers();
    assert_eq!(
        header_value(headers, "Content-Type"),
        Some("application/javascript")
    );
    assert_eq!(
        header_value(headers, "Cache-Control"),
        Some("public, max-age=31536000, immutable")
    );

    let etag = header_value(headers, "ETag").expect("ETag header missing");
    assert!(
        etag.starts_with('"') && etag.ends_with('"'),
        "ETag must be quoted per RFC 7232 §2.3; got {etag}"
    );
    // Quoted full SHA-256 hex = 2 quotes + 64 hex chars = 66 chars.
    assert_eq!(etag.len(), 66, "ETag length unexpected: {etag}");

    assert_eq!(resp.body_bytes().as_ref(), BYTES);
}
