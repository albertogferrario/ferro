# ferro-bundle

In-memory immutable byte-blob serving with content-hashed URLs for the Ferro framework.

The crate exposes a single `Bundle` type that registers compile-time-included bytes (`include_bytes!`) at boot, exposes a content-hashed URL of the form `/bundles/{name}.{sha8}.{ext}`, and serves the bytes with a one-year `Cache-Control: public, max-age=31536000, immutable` header plus a strong SHA-256 `ETag` and 304 fast-path on `If-None-Match` exact match. A `.with_alias("/path")` mechanism registers stable plain URLs that 301-redirect to the current hashed URL.

## Features

- SHA-256 content hashing (first 8 hex chars in the URL, full 64-hex digest in the ETag)
- One-year immutable `Cache-Control` headers with `immutable` directive (RFC 8246)
- Strong, quoted ETag per RFC 7232 with 304 fast-path on `If-None-Match` exact match
- Stable alias URLs that 301-redirect to the current hashed URL

## Usage

```rust
use ferro_bundle::Bundle;

// At boot, register a compile-time-embedded SDK bundle.
let sdk = Bundle::new("embed-v1", include_bytes!("../assets/embed-v1.js"))
    .content_type("application/javascript")
    .with_alias("/embed/v1.js");

// The hashed URL is deterministic per byte contents.
assert!(sdk.hashed_url().starts_with("/bundles/embed-v1."));

// Inside a request handler mounted on `/bundles/{filename}` (and on each
// registered alias path), dispatch via Bundle::serve.
// ferro-bundle does not own routing; the consumer wires the handler.
async fn serve_bundle(req: ferro_rs::Request) -> ferro_rs::HttpResponse {
    Bundle::serve(req)
}
```

## Bundle vs filesystem static files

ferro-bundle and the framework's filesystem static-file handler at `ferro_rs::static_files` are two parallel asset-serving paths. They target different freshness models and are intentionally not folded into one.

| Path | Freshness model | Cache lifetime | Use for |
|------|-----------------|----------------|---------|
| `ferro-bundle` | content hash in URL | one year, `immutable` | SDK bundles, embedded fonts, versioned static assets included via `include_bytes!` |
| `ferro_rs::static_files` | `bust_asset_urls` timestamp query param | shorter, revalidated | tenant-customizable CSS, theme assets, on-disk user uploads |

**Do not fold these — they target different freshness models.** A content-hashed URL is a stable handle to an immutable blob; a timestamp-busted URL is a freshness marker on a mutable file. Collapsing the two paths erodes both contracts (do not fold these paths into one).

## Security note

The caller provides `content_type` at registration. Serving caller-supplied bytes as `text/html` from a cookie-authenticated origin can introduce an XSS vector if the bundle bytes are themselves user-controlled. The locked `&'static [u8]` API restricts bytes to compile-time inclusion, which mitigates this; downstream consumers should keep that invariant when extending the crate in future phases.

## License

MIT — see the [ferro](https://github.com/albertogferrario/ferro) workspace README for documentation links (`https://docs.rs/ferro-bundle`).
