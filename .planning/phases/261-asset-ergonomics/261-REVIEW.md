---
phase: 261-asset-ergonomics
reviewed: 2026-07-26T00:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - ferro-macros/src/asset.rs
  - ferro-macros/src/lib.rs
  - ferro-macros/tests/asset_macro.rs
  - ferro-macros/tests/ui/asset/pass/minimal.rs
  - ferro-bundle/src/lib.rs
  - ferro-bundle/tests/serve_cold.rs
  - ferro-bundle/tests/serve_304.rs
  - ferro-bundle/tests/alias_redirect.rs
  - framework/src/bundle.rs
  - framework/src/lib.rs
  - ferro-cli/src/commands/assets.rs
  - ferro-cli/src/commands/mod.rs
  - ferro-cli/src/main.rs
  - framework/tests/bundle_serve.rs
  - .github/workflows/publish.yml
findings:
  critical: 1
  warning: 3
  info: 2
  total: 6
status: resolved
fixed_at: 2026-07-26
---

# Phase 261: Code Review Report

**Reviewed:** 2026-07-26
**Fixed:** 2026-07-26
**Depth:** standard
**Files Reviewed:** 15
**Status:** resolved (all critical and warning findings fixed)

## Summary

Phase 261 adds the `asset!()` proc-macro in `ferro-macros`, decouples `ferro-bundle` as a leaf crate, adds a `framework::bundle::serve` adapter, and introduces `ferro assets fetch` for downloading Iconify icons and Fontsource fonts. The decouple is clean: `ferro-bundle` has no `ferro-rs` dependency, the adapter faithfully maps all three response fields, and the 304/301/404 dispatch order is correct. The publish.yml wave placement is clean — `ferro-bundle` appears once in Wave 1a.

One critical issue: the `woff2_url` extracted from the Fontsource API response is used directly as an HTTP client argument without any host or scheme validation, creating a Server-Side Request Forgery (SSRF) vector. Three warnings cover the `bundle_name` collision space, the unvalidated `icon_body` in SVG reconstruction, and a missing `Content-Length` header on the 404 body. Two info items cover minor style concerns.

## Critical Issues

### CR-01: SSRF via unvalidated Fontsource woff2 URL ✓ FIXED (d6aca914, c059b02f)

**File:** `ferro-cli/src/commands/assets.rs:197`
**Issue:** The woff2 download URL is taken verbatim from the Fontsource API JSON response and passed directly to `client.get(woff2_url)`. A compromised or malicious Fontsource API response (or a DNS rebinding / BGP-hijack scenario) could direct the CLI to fetch from an arbitrary host, including internal network addresses (`http://169.254.169.254/latest/meta-data/`, `file://`, SMB paths on Windows, etc.). The `validate_segment` guard protects filesystem paths but is never applied to this URL. The Fontsource documentation says woff2 URLs should be CDN-hosted under `cdn.fontsource.com`, but that invariant is not enforced in code.

**Fix:**
```rust
// Before client.get(woff2_url)... at line 197, add:
fn validate_woff2_url(url: &str) -> anyhow::Result<()> {
    let parsed = url::Url::parse(url)
        .map_err(|_| anyhow::anyhow!("invalid woff2 URL: {url:?}"))?;
    if parsed.scheme() != "https" {
        anyhow::bail!("woff2 URL must use HTTPS; got scheme {:?}", parsed.scheme());
    }
    let host = parsed.host_str().unwrap_or("");
    // Fontsource CDN and API hosts only
    if !matches!(host, "cdn.fontsource.com" | "api.fontsource.org") {
        anyhow::bail!(
            "woff2 URL host {host:?} is not an allowed Fontsource host"
        );
    }
    Ok(())
}

// In fetch_fontsource, before the client.get call:
validate_woff2_url(woff2_url)?;
let bytes = client.get(woff2_url).send()?.error_for_status()?.bytes()?;
```

## Warnings

### WR-01: `bundle_name` collision space is small for common path patterns ✓ FIXED (96d52739)

**File:** `ferro-macros/src/asset.rs:32-41`
**Issue:** The bundle name is derived by mapping every character outside `[a-zA-Z0-9-]` to `'_'`. This produces collisions for paths that differ only in their separator or extension: `"assets/app.css"` and `"assets_app_css"` (a hypothetical file) both produce the bundle name `"assets_app_css"`. In practice, asset paths within a single application are unlikely to collide, but the collision is a silent boot-time `panic!` (from `Bundle::new`), not a compile-time error. The current scheme also downcases uppercase letters at the extension stage (`ext.to_ascii_lowercase()`) but does NOT downcase the bundle name itself, so `"assets/App.js"` produces bundle name `"assets_App_js"` — uppercase in the name — which is inconsistent with the comment in the macro ("D-04: keep [a-z0-9-]").

**Fix:** Downcase the entire bundle-name string so the comment matches the implementation, and consider appending a short hash of the original path to guarantee uniqueness:
```rust
let bundle_name: String = path_str
    .chars()
    .map(|c| {
        if c.is_ascii_alphanumeric() || c == '-' {
            c.to_ascii_lowercase()
        } else {
            '_'
        }
    })
    .collect();
```

### WR-02: Unvalidated `icon_body` inserted into reconstructed SVG ✓ FIXED (4d5c6435, c059b02f)

**File:** `ferro-cli/src/commands/assets.rs:114-121`
**Issue:** When fetching a full icon set, `icon_body` is taken from the `"body"` field of the Iconify JSON response and embedded verbatim into a synthesized `<svg>` string. A malicious API response (or a MITM attack — the Iconify API is fetched over HTTPS, but TLS termination is not verified beyond the default `rustls` configuration) could inject arbitrary SVG content, including `<script>` elements or `<foreignObject>` with HTML. The written `.svg` files are later embedded into browser pages by the application. This is a stored-XSS vector in the generated asset files.

Note: single-icon fetches (`GET /{prefix}/{icon}.svg`) return the full SVG document verbatim and are not affected by this specific issue, as the file is written unmodified. The problem is specific to the set-mode reconstruction path.

**Fix:** Strip the `icon_body` of any element tags before embedding, or use an allowlist of safe SVG body content. At minimum, refuse to write icons whose body contains `<script`, `<foreignObject`, `on` event attributes, or `javascript:` URIs:
```rust
fn is_safe_svg_body(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    !lower.contains("<script")
        && !lower.contains("<foreignobject")
        && !lower.contains("javascript:")
        && !lower.contains(" on")  // event handler attributes (onclick, onload, etc.)
}

// Before write_icon call:
if !is_safe_svg_body(icon_body) {
    eprintln!("warning: skipping icon {name:?} — body contains potentially unsafe content");
    continue;
}
```

### WR-03: 404 response includes a `Content-Type: text/plain` header but no body content ✓ FIXED (86a543bc)

**File:** `ferro-bundle/src/lib.rs:314`
**Issue:** The 404 path adds `Content-Type: text/plain` but `with_body` is never called, so the body is empty (`Bytes::new()`). The `Content-Type` header on a zero-byte body is misleading and may cause some HTTP clients or proxies to wait for content they will never receive or to log spurious errors. The HTTP/1.1 spec (RFC 7230 §3.3) permits a 404 with no body, but the `Content-Type` should be omitted when there is no body, or a minimal body should be included.

**Fix:** Either remove the `Content-Type` header from the 404 path, or add a short plain-text body:
```rust
// Option A: no content-type on empty body
BundleResponse::new(404)

// Option B: include a body
BundleResponse::new(404)
    .header("Content-Type", "text/plain")
    .with_body(Bytes::from_static(b"not found"))
```

## Info

### IN-01: Trybuild pass-fixture has no corresponding `fail/` tests (not fixed — informational)

**File:** `ferro-macros/tests/asset_macro.rs:11`
**Issue:** The trybuild test harness only has a `pass/` directory. There are no `fail/` fixtures to verify that `asset!()` rejects invalid inputs (wrong argument count, non-string-literal argument, etc.). Other macro tests in the project follow the same pattern, so this is consistent — but it leaves the error-path surface of the macro unverified.

**Fix:** Add `t.compile_fail("tests/ui/asset/fail/*.rs")` with at least one fixture (e.g., `asset!(123)` expecting a type-error diagnostic). Not blocking.

### IN-02: `woff2_dest` does not validate `family` or `subset` before use as path components (not fixed — informational)

**File:** `ferro-cli/src/commands/assets.rs:78-82`
**Issue:** `woff2_dest` is a public function (used in tests) that constructs a path from `family` and `subset` without calling `validate_segment`. In `fetch_fontsource`, `validate_segment` is called on `family` and each `subset` before `woff2_dest` is invoked, so the production path is safe. However, the public function itself contains no guard. A future caller invoking `woff2_dest` directly with unvalidated input (e.g., in a test or a new code path) would silently produce an unsafe path. The existing tests confirm the expected shape but do not test the traversal-rejection case.

**Fix:** Add `validate_segment` calls inside `woff2_dest` and make it return `anyhow::Result<PathBuf>`, or document the precondition clearly and keep it `pub(crate)` so external callers cannot misuse it.

---

_Reviewed: 2026-07-26_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
