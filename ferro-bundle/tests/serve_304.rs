//! BUNDLE-02 304 fast-path integration test.
//!
//! Verifies that a request with `If-None-Match` exactly matching the stored ETag
//! returns:
//! - status 304
//! - same quoted ETag
//! - Cache-Control: public, max-age=31536000, immutable (RFC 7232 §4.1)
//! - empty body
//!
//! Pitfall 3 mitigation: ETag value MUST match exactly including the surrounding quotes.

use ferro_bundle::Bundle;
use ferro_bundle::__test_internals::serve_inner;

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

#[test]
fn serve_304_on_if_none_match_exact() {
    static BYTES: &[u8] = b"console.log('sdk');";

    let bundle = Bundle::new("serve-304-sdk", BYTES).content_type("application/javascript");
    let hashed = bundle.hashed_url();

    // First, cold dispatch to capture the ETag the server will send.
    let cold = serve_inner(&hashed, None);
    let etag = header_value(cold.headers(), "ETag")
        .expect("cold response missing ETag")
        .to_string();
    assert!(etag.starts_with('"') && etag.ends_with('"'));

    // Now dispatch with If-None-Match == that ETag → 304.
    let resp = serve_inner(&hashed, Some(&etag));

    assert_eq!(resp.status_code(), 304);

    let headers = resp.headers();
    assert_eq!(header_value(headers, "ETag"), Some(etag.as_str()));
    assert_eq!(
        header_value(headers, "Cache-Control"),
        Some("public, max-age=31536000, immutable")
    );

    // 304 MUST have an empty body.
    assert!(
        resp.body_bytes().is_empty(),
        "304 body should be empty; got {} bytes",
        resp.body_bytes().len()
    );
}
