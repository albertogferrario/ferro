# Phase 72: Binary Response Type - Research

**Researched:** 2026-02-26
**Domain:** Rust web framework binary HTTP responses
**Confidence:** HIGH

<research_summary>
## Summary

Researched how to add binary response support to Ferro's `HttpResponse` type. Currently `HttpResponse` stores the body as `String`, forcing apps to use `unsafe { String::from_utf8_unchecked }` to serve non-UTF-8 content (images, PDFs, QR codes). This is undefined behavior when the bytes aren't valid UTF-8.

The standard approach in Rust web frameworks is to use `Bytes` (or a body enum) as the internal representation. Axum uses a generic body type with `IntoResponse` trait implementations for `String`, `Bytes`, `Vec<u8>`, etc. Actix-web uses a `Body` enum with `None`, `Bytes`, and `Stream` variants. Ferro's architecture is simpler (no generics on response type), so the cleanest approach is changing the internal body to `Bytes` with convenience constructors.

The existing static file system (`static_files.rs`) already bypasses `HttpResponse` entirely, returning `hyper::Response<Full<Bytes>>` directly. Phase 72 brings this capability into the user-facing API so controller handlers can serve binary data safely.

**Primary recommendation:** Change `HttpResponse.body` from `String` to `Bytes`, add `HttpResponse::bytes()` constructor, add `HttpResponse::download()` helper for file downloads with `Content-Disposition`.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already in Ferro)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bytes | 1.x | Zero-copy byte buffer | Already used throughout Ferro (hyper, storage, body parsing) |
| http-body-util | 0.1.x | Body utilities for hyper | Already used for `Full<Bytes>` response construction |
| mime_guess | 2.x | MIME type detection from file extensions | Already used in `static_files.rs` |

### Supporting (no new dependencies needed)
No new crates required. Ferro already has everything needed:
- `bytes::Bytes` for the body type
- `mime_guess` for content-type detection
- `hyper::Response<Full<Bytes>>` for the final HTTP response

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Bytes` body | `enum { Text(String), Binary(Bytes) }` | Enum adds complexity with no benefit — `Bytes::from(String)` is zero-copy |
| `mime_guess` for content-type | Manual match on extensions | `mime_guess` covers 800+ types, hand-rolling is error-prone |
| Sync download | Streaming `Body` | Streaming needed only for very large files; out of scope for this phase |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Current Architecture (the problem)

```rust
pub struct HttpResponse {
    status: u16,
    body: String,         // <-- Forces UTF-8, no binary support
    headers: Vec<(String, String)>,
}

// Conversion to hyper
pub fn into_hyper(self) -> hyper::Response<Full<Bytes>> {
    // ...
    builder.body(Full::new(Bytes::from(self.body))).unwrap()  // String → Bytes (safe)
}
```

Apps must use `unsafe` to serve binary:
```rust
// UB: arbitrary file bytes may not be valid UTF-8
let body = unsafe { String::from_utf8_unchecked(file_bytes) };
HttpResponse::text(body).header("Content-Type", "image/png")
```

### Pattern 1: Bytes-Based Body (recommended)

Change the internal representation to `Bytes` and adjust constructors:

```rust
pub struct HttpResponse {
    status: u16,
    body: Bytes,          // <-- Binary-safe
    headers: Vec<(String, String)>,
}

impl HttpResponse {
    pub fn text(body: impl Into<String>) -> Self {
        let s: String = body.into();
        Self {
            status: 200,
            body: Bytes::from(s),  // String → Bytes is zero-copy
            headers: vec![("Content-Type".into(), "text/plain".into())],
        }
    }

    pub fn bytes(body: impl Into<Bytes>) -> Self {
        Self {
            status: 200,
            body: body.into(),
            headers: vec![],  // Caller sets Content-Type
        }
    }

    pub fn download(body: impl Into<Bytes>, filename: &str) -> Self {
        Self {
            status: 200,
            body: body.into(),
            headers: vec![
                ("Content-Disposition".into(),
                 format!("attachment; filename=\"{}\"", filename)),
            ],
        }
    }
}
```

**Why this works:** `Bytes::from(String)` is zero-copy (reuses the String's allocation). So existing `text()` and `json()` constructors lose nothing. New `bytes()` constructor handles arbitrary data.

### Pattern 2: Body Accessor Changes

The `body()` method currently returns `&str`. With `Bytes` body this needs adjustment:

```rust
impl HttpResponse {
    // New: returns raw bytes
    pub fn body_bytes(&self) -> &Bytes {
        &self.body
    }

    // Preserved: returns text (lossy for binary)
    pub fn body(&self) -> &str {
        // This is used in tests and by the framework internally.
        // For binary responses, this won't make sense.
        // Option A: panic if not UTF-8 (breaking)
        // Option B: return lossy (existing behavior of TestResponse::text())
        // Option C: change return type to Cow<str> or &[u8]
        std::str::from_utf8(&self.body).unwrap_or("")
    }
}
```

**Decision needed:** Whether `body()` should return `&[u8]` (breaking but correct) or stay as `&str` with fallback (compatible but lossy). Since Ferro is pre-1.0 and this is a feature branch, changing to `&[u8]` is fine.

### Pattern 3: `into_hyper()` stays unchanged

```rust
pub fn into_hyper(self) -> hyper::Response<Full<Bytes>> {
    let mut builder = hyper::Response::builder().status(self.status);
    for (name, value) in self.headers {
        builder = builder.header(name, value);
    }
    builder.body(Full::new(self.body)).unwrap()  // Already Bytes, no conversion
}
```

### Anti-Patterns to Avoid
- **Adding a body enum (Text/Binary):** Unnecessary complexity. `Bytes` handles both cases. `String → Bytes` is zero-copy.
- **Streaming body support in this phase:** Scope creep. `Full<Bytes>` buffers the entire response. Streaming is a separate concern for very large files.
- **Changing the return type of `handle_request()`:** It already returns `hyper::Response<Full<Bytes>>`. No change needed at the server level.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MIME detection | Manual content-type mapping | `mime_guess::from_path()` | Already used in `static_files.rs`, covers 800+ types |
| Content-Disposition encoding | Manual header formatting | RFC 6266 format with filename quoting | Special chars in filenames need proper quoting |
| Zero-copy byte conversion | Custom buffer types | `bytes::Bytes` | Already in the dependency tree, battle-tested |
| Body type abstraction | Custom enum or trait | Direct `Bytes` field | `Bytes::from(String)` is zero-copy; no abstraction needed |

**Key insight:** The `bytes` crate already provides the right abstraction. `Bytes::from(String)` doesn't copy data — it reuses the String's heap allocation. So switching from `String` to `Bytes` has zero performance cost for existing text responses.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Content-Type Not Set for Binary Responses
**What goes wrong:** Binary data served without `Content-Type` header; browser tries to render as text
**Why it happens:** `bytes()` constructor can't infer content type without a file extension
**How to avoid:** Document that `bytes()` requires explicit `.header("Content-Type", ...)`. Consider a `bytes_with_type(data, mime)` convenience method.
**Warning signs:** Browser showing garbled text instead of downloading/displaying binary content

### Pitfall 2: Content-Disposition Filename Injection
**What goes wrong:** User-controlled filenames in `Content-Disposition` header allow header injection
**Why it happens:** Filenames containing `"` or `\r\n` break the header format
**How to avoid:** Sanitize filenames in `download()` — strip or escape `"`, remove control characters
**Warning signs:** Download filenames looking wrong, or extra headers appearing

### Pitfall 3: Breaking `body()` Return Type
**What goes wrong:** Existing code calls `response.body()` expecting `&str`, gets compilation errors
**Why it happens:** Changing internal type from `String` to `Bytes` invalidates `&str` return
**How to avoid:** Either keep `body()` as `&str` (with fallback for non-UTF-8) or audit all callers. Since Ferro is pre-1.0, a clean break is acceptable. `body_bytes()` provides the new accessor.
**Warning signs:** Compilation errors in user code and tests

### Pitfall 4: Large File Memory Usage
**What goes wrong:** Serving a 500MB file loads it entirely into memory via `Vec<u8>`
**Why it happens:** `Full<Bytes>` is a buffered body — no streaming
**How to avoid:** Document the limitation. For this phase, binary responses are for reasonable-sized content (images, PDFs, QR codes — KB to low MB). Streaming is a future enhancement.
**Warning signs:** Memory spikes when serving large files
</common_pitfalls>

<code_examples>
## Code Examples

### Serving an Image from Storage
```rust
#[handler]
pub async fn show_avatar(req: Request, user: User) -> Response {
    let storage = req.app::<Storage>();
    let bytes = storage.get(&user.avatar_path).await
        .map_err(|_| HttpResponse::text("Not found").status(404))?;

    Ok(HttpResponse::bytes(bytes)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=3600"))
}
```

### File Download with Content-Disposition
```rust
#[handler]
pub async fn download_report(req: Request) -> Response {
    let pdf_bytes = generate_pdf_report().await?;

    Ok(HttpResponse::download(pdf_bytes, "report.pdf")
        .header("Content-Type", "application/pdf"))
}
```

### QR Code Generation (the mkmenu use case)
```rust
#[handler]
pub async fn qr_code(req: Request) -> Response {
    let data = req.param::<String>("data")?;
    let png_bytes = qrcode_generator::to_png_to_vec(&data, QrCodeEcc::Medium, 256)
        .map_err(|e| HttpResponse::text(e.to_string()).status(500))?;

    Ok(HttpResponse::bytes(png_bytes)
        .header("Content-Type", "image/png"))
}
```

### Backward Compatibility — Existing Text/JSON Usage Unchanged
```rust
// These continue to work exactly as before:
HttpResponse::text("Hello")           // String → Bytes (zero-copy)
HttpResponse::json(json!({"ok": 1}))  // String → Bytes (zero-copy)
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| String body only | `Bytes` body supporting both text and binary | This phase | Eliminates unsafe code for binary responses |
| Bypass HttpResponse for static files | Unified response type for all content | This phase | static_files.rs pattern becomes unnecessary workaround |

**New patterns enabled by this change:**
- Safe file downloads from storage
- In-memory image/PDF generation served directly
- QR codes, charts, and other generated binary content
- Proper `Content-Disposition` for file downloads

**Not in scope (future):**
- Streaming responses for large files (would need `http_body::Body` trait impl)
- Range requests (206 Partial Content)
- Chunked transfer encoding
</sota_updates>

<open_questions>
## Open Questions

1. **`body()` return type: `&str` or `&[u8]`?**
   - What we know: Currently returns `&str`. Changing to `&[u8]` is cleaner but breaks callers.
   - What's unclear: How many external consumers depend on `body()` returning `&str`
   - Recommendation: Keep `body()` as `&str` with `from_utf8` + empty fallback for non-UTF-8. Add `body_bytes()` returning `&Bytes`. Since `body()` is mostly used in tests and middleware, audit callers during implementation.

2. **Should `download()` auto-detect Content-Type from filename?**
   - What we know: `mime_guess::from_path()` is already available and used in `static_files.rs`
   - What's unclear: Whether auto-detection is always desired (might want to force `application/octet-stream` for security)
   - Recommendation: Auto-detect from filename extension, allow override via `.header()`. Use `application/octet-stream` as fallback.

3. **Should `into_hyper()` set `Content-Length` automatically?**
   - What we know: Static file serving already sets `Content-Length`. Current `into_hyper()` does not.
   - What's unclear: Whether adding `Content-Length` could break any existing behavior
   - Recommendation: Not in scope for this phase. Hyper sets it automatically for `Full<Bytes>` bodies.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Ferro codebase analysis: `framework/src/http/response.rs` (current HttpResponse implementation)
- Ferro codebase analysis: `framework/src/static_files.rs` (existing binary response pattern using hyper directly)
- Ferro codebase analysis: `framework/src/server.rs` (request dispatch and into_hyper conversion)
- `bytes` crate documentation: `Bytes::from(String)` is zero-copy (reuses allocation)
- Ferro codebase analysis: `ferro-storage/src/facade.rs` (storage returns `Bytes`)

### Secondary (MEDIUM confidence)
- Axum `IntoResponse` trait pattern: `Bytes`, `Vec<u8>`, `String` all implement it with appropriate content-types
- Actix-web `Body` enum pattern: `None`, `Bytes`, `Stream` variants
- RFC 6266: Content-Disposition header format for file downloads

### Tertiary (LOW confidence - needs validation)
- None — all findings verified against codebase
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust web framework response body types
- Ecosystem: bytes, hyper, http-body-util (all already in dependency tree)
- Patterns: Binary response, file download, content-type detection
- Pitfalls: Content-type omission, filename injection, memory usage, API breakage

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, all already available
- Architecture: HIGH — direct analysis of current codebase; `String → Bytes` is proven zero-copy
- Pitfalls: HIGH — standard web framework concerns, well-documented
- Code examples: HIGH — based on existing Ferro patterns and current API

**Research date:** 2026-02-26
**Valid until:** 2026-03-28 (30 days — stable domain, no ecosystem churn)
</metadata>

---

*Phase: 72-binary-response-type*
*Research completed: 2026-02-26*
*Ready for planning: yes*
