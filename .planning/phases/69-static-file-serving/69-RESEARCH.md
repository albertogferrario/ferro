# Phase 69: Static File Serving - Research

**Researched:** 2026-02-25
**Domain:** Static file serving in a Rust/hyper web framework
**Confidence:** HIGH

<research_summary>
## Summary

Researched approaches for adding built-in static file serving to Ferro's raw hyper-based server. Three options evaluated: tower-http ServeDir, hyper-staticfile, and custom implementation using tokio::fs + mime_guess.

Key finding: A custom implementation is the best fit because Ferro uses raw hyper (not Axum), its HttpResponse stores body as String (unusable for binary files), and the requirements are simple enough that ~80-100 lines of focused code are preferable to adding a dependency with body type conversion overhead.

The integration point is clear: `handle_request()` in `framework/src/server.rs` lines 177-201, between "no route matched" and "return 404". Static file check happens before the existing fallback handler check.

**Primary recommendation:** Custom implementation using `tokio::fs::read()` + `mime_guess::from_path()` + `Path::canonicalize()` for security. Serve from `public/` with differentiated cache headers for hashed vs unhashed assets.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already available in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio::fs | 1.x | Async file reading | Already a dependency, production-grade async I/O |
| mime_guess | 2.x | MIME type detection from file extension | Already used in ferro-storage, standard in Rust ecosystem |
| std::path | stdlib | Path canonicalization for security | No external dependency, canonicalize() resolves symlinks + `..` |

### No New Dependencies Required
The framework already has all necessary building blocks:
- `tokio` (full features) for async filesystem operations
- `hyper` 1.x for HTTP response construction
- `bytes` for efficient byte handling
- `http-body-util` for `Full<Bytes>` response bodies

`mime_guess` needs to be added to `framework/Cargo.toml` (already in `ferro-storage/Cargo.toml`).

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom impl | tower-http ServeDir | ServeDir returns `ResponseBody` type, not `Full<Bytes>` — needs body conversion layer. Adds ~40 transitive deps. Overkill for serving from a single directory. |
| Custom impl | hyper-staticfile | Purpose-built for hyper 1.0 but had a security advisory (RUSTSEC-2022-0069, fixed). Adds external dependency for ~80 lines of code. |
| mime_guess | content sniffing | Never sniff content — extension-based detection is correct for web servers. Content sniffing causes security issues (MIME confusion attacks). |
| tokio::fs::read() | tokio::fs::File + streaming | Streaming is better for large files (>10MB), but Vite assets are typically <1MB. `read()` is simpler and sufficient. Can be upgraded later if needed. |

**Installation:**
```toml
# Add to framework/Cargo.toml [dependencies]
mime_guess = "2"
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Integration Point in server.rs

The static file check slots into `handle_request()` between route matching and 404:

```
Request comes in
  → WebSocket upgrade check (/_ferro/ws)
  → Framework endpoints (/_ferro/*)
  → Route matching (router.match_route)
  → **NEW: Static file serving (try_serve_static_file)**
  → Fallback handler (router.get_fallback)
  → 404 Not Found
```

Static files are checked BEFORE the fallback handler because:
1. The fallback is typically an Inertia SPA catch-all (serves the React app for any unmatched route)
2. Asset requests (/assets/main.js) must be intercepted before the SPA fallback

### Pattern 1: Direct hyper::Response Construction

**What:** Build `hyper::Response<Full<Bytes>>` directly, bypassing `HttpResponse`
**Why:** HttpResponse stores body as `String` — binary files (images, fonts, wasm) would be corrupted
**When to use:** Always, for static file responses

```rust
// Build response directly with hyper
fn static_file_response(bytes: Vec<u8>, content_type: &str, cache_control: &str) -> hyper::Response<Full<Bytes>> {
    hyper::Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Cache-Control", cache_control)
        .body(Full::new(Bytes::from(bytes)))
        .unwrap()
}
```

### Pattern 2: Differentiated Cache Headers

**What:** Different cache strategies for hashed vs unhashed assets
**Why:** Vite outputs hashed filenames (main-abc123.js) that are immutable. Root files (favicon.ico) may change.

```rust
fn cache_header_for_path(path: &str) -> &str {
    if path.starts_with("/assets/") {
        // Vite hashed assets: cache forever (hash changes on content change)
        "public, max-age=31536000, immutable"
    } else {
        // Root files (favicon.ico, robots.txt): revalidate each time
        "public, max-age=0, must-revalidate"
    }
}
```

### Pattern 3: Dev Mode Skip

**What:** Skip static file serving in development mode
**Why:** Vite dev server handles assets via HMR in development

```rust
// In handle_request, before static file check:
if Config::is_development() {
    // Skip — Vite dev server handles assets
}
```

Note: Actually, static file serving should work in ALL modes. In development, Vite's HMR proxy serves assets from its own server (e.g., localhost:5173). The HTML in dev mode references `http://localhost:5173/src/main.tsx`, not `/assets/main.js`. So the static file handler won't interfere — it simply won't find files in `public/assets/` (they don't exist until `vite build` runs). No special dev mode logic needed.

### Anti-Patterns to Avoid
- **Don't use HttpResponse for binary files:** Body is String, not bytes. Use hyper::Response directly.
- **Don't check files for every request:** Only check when no route matches. Route matching should be the fast path.
- **Don't stream large files with read():** For MVP, `tokio::fs::read()` loads the entire file into memory. This is fine for Vite assets (<1MB typically). If large file support is needed later, switch to `tokio::fs::File` + `ReaderStream`.
- **Don't serve dotfiles:** Never serve `.env`, `.git/`, `.planning/` etc. Reject paths starting with `.` or containing `/.`.
- **Don't follow symlinks outside public/:** Canonicalize and verify the resolved path is within `public/`.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MIME type detection | Custom extension→type mapping | `mime_guess::from_path()` | Covers 800+ types, maintained, already in workspace |
| Path traversal protection | Regex or string matching for `..` | `std::path::Path::canonicalize()` + `starts_with()` | Handles all edge cases: encoded paths, symlinks, Windows paths. String matching misses encoded `%2e%2e` |
| HTTP range requests | Custom byte range parsing | Skip for MVP | Complex spec (RFC 7233), only needed for video/large files. Add later if needed. |
| Compression | Gzip/brotli on the fly | Precompressed files or reverse proxy | On-the-fly compression adds latency. In production, use a CDN/reverse proxy for compression. Vite can be configured to output `.gz` files. |
| ETag/If-None-Match | Custom hash computation | Skip for MVP | Cache-Control with immutable for hashed assets makes ETags unnecessary. Add later for unhashed files if needed. |

**Key insight:** The scope is intentionally narrow — serve Vite build output and common root files. This is NOT a general-purpose file server. Production apps should use a CDN or reverse proxy for advanced features (compression, range requests, ETags). The framework just needs to "not 404" for assets.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Directory Traversal via Encoded Paths
**What goes wrong:** Paths like `/%2e%2e/etc/passwd` bypass naive `..` string checks
**Why it happens:** URL decoding happens before path checking, or checking happens on the raw URL
**How to avoid:** Decode the URL path first (hyper already does this), then canonicalize with `std::path::Path::canonicalize()`, then verify the canonical path `starts_with()` the public directory's canonical path
**Warning signs:** Any path resolution that doesn't use canonicalize()

### Pitfall 2: Binary File Corruption via String Body
**What goes wrong:** Images, fonts, and other binary files served as garbled data
**Why it happens:** Ferro's `HttpResponse` stores body as `String`. Binary data passed through `String` gets corrupted (invalid UTF-8 bytes replaced)
**How to avoid:** Build `hyper::Response<Full<Bytes>>` directly from `Vec<u8>`, never go through HttpResponse
**Warning signs:** Using `HttpResponse::text()` for file responses

### Pitfall 3: Serving Sensitive Files
**What goes wrong:** `.env`, `Cargo.toml`, `.git/config` served to the public
**Why it happens:** The `public/` directory is the document root, but developers might accidentally place sensitive files there, or symlinks might point outside
**How to avoid:** Only serve from canonicalized `public/` directory. Reject dotfiles. Log warnings for suspicious requests.
**Warning signs:** No dotfile filtering, no path validation after canonicalization

### Pitfall 4: Race Condition with Fallback Handler
**What goes wrong:** SPA fallback handler catches `/assets/main.js` requests, returns HTML instead of JS
**Why it happens:** Static file check runs after fallback handler (wrong order)
**How to avoid:** Static file check MUST run before the fallback handler in handle_request()
**Warning signs:** Inertia apps showing blank pages in production (HTML served instead of JS)

### Pitfall 5: Performance Death by Filesystem Calls
**What goes wrong:** Every request triggers a filesystem stat/read, even for API endpoints
**Why it happens:** Static file check runs for all requests, not just unmatched ones
**How to avoid:** Only attempt static file serving when no route matches. Route matching is in-memory and fast.
**Warning signs:** Increased latency on all endpoints, not just static file requests
</common_pitfalls>

<code_examples>
## Code Examples

### Core Static File Handler Function
```rust
// Source: Custom implementation for Ferro
use std::path::{Path, PathBuf};

/// Try to serve a static file from the public directory.
/// Returns None if the file doesn't exist or the path is invalid.
async fn try_serve_static_file(request_path: &str) -> Option<hyper::Response<Full<Bytes>>> {
    // Reject empty paths and paths with null bytes
    if request_path.is_empty() || request_path.contains('\0') {
        return None;
    }

    // Reject dotfiles and hidden directories
    if request_path.split('/').any(|segment| segment.starts_with('.')) {
        return None;
    }

    // Build the filesystem path
    let relative_path = request_path.trim_start_matches('/');
    let public_dir = Path::new("public");
    let file_path = public_dir.join(relative_path);

    // Canonicalize both paths for security comparison
    let canonical_public = match public_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return None, // public/ directory doesn't exist
    };
    let canonical_file = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return None, // file doesn't exist
    };

    // Directory traversal protection
    if !canonical_file.starts_with(&canonical_public) {
        return None;
    }

    // Don't serve directories
    if canonical_file.is_dir() {
        return None;
    }

    // Read the file
    let bytes = match tokio::fs::read(&canonical_file).await {
        Ok(b) => b,
        Err(_) => return None,
    };

    // Detect MIME type
    let content_type = mime_guess::from_path(&canonical_file)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Determine cache strategy
    let cache_control = if request_path.starts_with("/assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=0, must-revalidate"
    };

    let response = hyper::Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Cache-Control", cache_control)
        .header("Content-Length", bytes.len().to_string())
        .body(Full::new(Bytes::from(bytes)))
        .unwrap();

    Some(response)
}
```

### Integration in handle_request()
```rust
// In handle_request(), after route matching returns None:
None => {
    // Try static file serving first
    if method == hyper::Method::GET || method == hyper::Method::HEAD {
        if let Some(response) = try_serve_static_file(&path).await {
            return response;
        }
    }

    // Then try fallback handler
    if let Some((fallback_handler, fallback_middleware)) = router.get_fallback() {
        // ... existing fallback code ...
    } else {
        HttpResponse::text("404 Not Found").status(404).into_hyper()
    }
}
```

### Vite Config Reference
```typescript
// frontend/vite.config.ts — Ferro convention
export default defineConfig({
  build: {
    outDir: '../public/assets',  // Output to public/assets/
    manifest: true,
    rollupOptions: {
      input: 'src/main.tsx',
    },
  },
});
// Output: public/assets/main-[hash].js, public/assets/main-[hash].css
```

### Cache Header Strategy
```
Request path             → Cache-Control header
/assets/main-abc123.js   → public, max-age=31536000, immutable  (1 year, hashed)
/assets/main-def456.css  → public, max-age=31536000, immutable  (1 year, hashed)
/favicon.ico             → public, max-age=0, must-revalidate   (always check)
/robots.txt              → public, max-age=0, must-revalidate   (always check)
/storage/photos/1.jpg    → public, max-age=0, must-revalidate   (user uploads)
```
</code_examples>

<sota_updates>
## State of the Art (2024-2025)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tower-http 0.5 ServeDir | tower-http 0.6 ServeDir | 2024 | New features but still needs body type conversion for raw hyper |
| hyper 0.14 | hyper 1.0 | 2023 | Breaking change to body types; hyper-staticfile updated to match |
| ETag-based caching | Cache-Control: immutable | 2023+ | Immutable directive widely supported; ETags unnecessary for hashed assets |
| On-the-fly gzip | Precompressed files or CDN | Ongoing | CDN/reverse proxy handles compression better than app server |

**New tools/patterns to consider:**
- **`Cache-Control: immutable`**: Widely supported in 2025. Prevents unnecessary revalidation requests for hashed assets. Combined with max-age=1year, this is the gold standard.
- **Vite asset fingerprinting**: Vite's default content hashing means the server NEVER needs to invalidate caches — new content = new hash = new URL.

**Deprecated/outdated:**
- **ETag computation for hashed assets**: Unnecessary overhead. The hash IS the ETag conceptually.
- **On-the-fly compression in app server**: Use CDN or reverse proxy. App server should serve raw files.
</sota_updates>

<open_questions>
## Open Questions

1. **Should static file serving work in development too?**
   - What we know: In dev, Vite dev server serves assets via HMR proxy. HTML references `http://localhost:5173/...`, not `/assets/...`. So static serving in dev is a no-op (no files in public/assets/).
   - What's unclear: Should we explicitly disable it in dev for a tiny performance win?
   - Recommendation: Keep it always-on. Zero harm in dev (files don't exist), simpler code.

2. **Large file streaming support?**
   - What we know: Vite assets are typically <1MB. `tokio::fs::read()` loads entire file to memory.
   - What's unclear: Will users serve large files (video, datasets) from public/?
   - Recommendation: Ship with `read()` for simplicity. Document that large files should use ferro-storage + CDN. Can add streaming later if needed.

3. **HEAD request support?**
   - What we know: HTTP spec requires HEAD to return same headers as GET but no body.
   - What's unclear: How important is HEAD for static files in practice?
   - Recommendation: Support HEAD by checking the method and returning headers-only response. Cheap to implement.

4. **Should this be a middleware or built into server.rs?**
   - What we know: Integration point is clear (handle_request). Could be either.
   - What's unclear: Whether users will want to customize (e.g., different public dir, auth on static files).
   - Recommendation: Build into server.rs as a private function. Framework convention is `public/`. If customization is needed later, extract to configurable middleware.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- tower-http ServeDir docs (https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html) — API reference, security behavior, fallback patterns
- Ferro source code: `framework/src/server.rs` — handle_request() integration point, current request lifecycle
- Ferro source code: `framework/src/http/response.rs` — HttpResponse uses String body (critical constraint)
- Ferro source code: `ferro-inertia/src/response.rs` lines 440-441 — Production HTML references `/assets/main.js`, `/assets/main.css`
- Ferro source code: `app/frontend/vite.config.ts` — Vite outputs to `public/assets/`
- mime_guess docs (https://docs.rs/mime_guess/latest/mime_guess/) — Extension-based MIME detection

### Secondary (MEDIUM confidence)
- Vite build documentation (https://vite.dev/guide/build) — Asset hashing behavior, output structure
- Rust Path Traversal Guide (https://www.stackhawk.com/blog/rust-path-traversal-guide-example-and-prevention/) — canonicalize() + starts_with() pattern verified against stdlib docs
- Cache header best practices (https://blog.jonathanlau.io/posts/how-to-use-cache-control-headers/) — immutable + max-age strategy for hashed assets

### Tertiary (LOW confidence - needs validation)
- hyper-staticfile (https://docs.rs/hyper-staticfile) — Alternative approach, not recommended but documented for completeness
- RUSTSEC-2022-0069 (https://rustsec.org/advisories/RUSTSEC-2022-0069.html) — hyper-staticfile had Windows path traversal vulnerability (fixed, but illustrates why canonicalize() is essential)
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust static file serving over hyper 1.0
- Ecosystem: tower-http, hyper-staticfile, mime_guess, tokio::fs
- Patterns: Fallback-based serving, cache header differentiation, path security
- Pitfalls: Directory traversal, binary corruption, dotfile exposure, ordering

**Confidence breakdown:**
- Standard stack: HIGH — using existing workspace deps (tokio, mime_guess), well-understood patterns
- Architecture: HIGH — integration point is unambiguous (server.rs handle_request), code examples verified against codebase
- Pitfalls: HIGH — directory traversal is well-documented, binary/String issue confirmed by reading HttpResponse source
- Code examples: HIGH — written against actual Ferro types and patterns, verified against server.rs

**Research date:** 2026-02-25
**Valid until:** 2026-03-25 (30 days — static file serving is a stable, well-understood domain)
</metadata>

---

*Phase: 69-static-file-serving*
*Research completed: 2026-02-25*
*Ready for planning: yes*
