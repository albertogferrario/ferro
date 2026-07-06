# Phase 73: Security Headers Middleware - Research

**Researched:** 2026-02-26
**Domain:** HTTP security response headers for Rust web framework
**Confidence:** HIGH

<research_summary>
## Summary

Researched OWASP-recommended HTTP security headers and how to implement them as middleware in the Ferro framework. The domain is well-established — OWASP Secure Headers Project (updated Jan 2026) provides authoritative guidance, and the implementation pattern (post-processing middleware that appends headers to responses) maps cleanly to Ferro's existing middleware architecture.

Key finding: CSP is the only complex header due to Inertia.js/Vite SPA compatibility concerns. Nonce-based CSP requires per-request nonce generation and coordination with Vite's script tags. The recommended approach is to ship a safe, permissive default that works out-of-the-box, with builder methods for tightening.

HSTS needs environment-awareness — it must NOT be sent in development (breaks `localhost` over HTTP) and should default to conservative values (no `preload` by default, as preload submission is permanent).

**Primary recommendation:** Implement as a single `SecurityHeaders` middleware struct with builder pattern for customization. Ship sensible defaults that work for both Inertia.js and JSON-UI apps without breaking development workflows.
</research_summary>

<standard_stack>
## Standard Stack

### Core

No external crates needed. The implementation is straightforward string header injection on responses — the same pattern as `CsrfMiddleware` but applied post-response.

| Component | Source | Purpose | Why |
|-----------|--------|---------|-----|
| `SecurityHeaders` middleware | Hand-roll | Set security headers on all responses | ~100 lines; no crate matches Ferro's middleware trait |
| OWASP Secure Headers Project | Reference | Authoritative header values | Updated Jan 2026; industry standard |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-roll | `http-security-headers` crate | Crate targets tower/axum middleware; Ferro uses its own `Middleware` trait. Adding a dependency for string constants is unnecessary |
| Hand-roll | `rust-helmet` crate | Same issue — framework-specific (axum/actix/ntex). Also unclear maintenance status (20 total commits) |
| Single middleware | Per-header middlewares | Single middleware is simpler to register, configure, and reason about |

**No installation needed** — pure Rust implementation using Ferro's existing `Middleware` trait.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: Post-Processing Response Middleware

**What:** Middleware that calls `next(request).await` first, then appends headers to the response before returning.
**When to use:** Any middleware that modifies responses (security headers, CORS, cache headers).
**How it maps to Ferro:**

```rust
#[async_trait]
impl Middleware for SecurityHeaders {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let response = next(request).await;
        // Add headers to both Ok and Err responses
        match response {
            Ok(resp) => Ok(self.apply_headers(resp)),
            Err(resp) => Err(self.apply_headers(resp)),
        }
    }
}
```

This matches the existing pattern in `MetricsMiddleware` (which also processes the response after `next()`).

### Pattern 2: Builder Configuration (like CsrfMiddleware)

**What:** Struct with sensible defaults, customizable via builder methods.
**When to use:** Middleware that needs app-specific configuration.

```rust
let headers = SecurityHeaders::new()           // Sensible defaults
    .x_frame_options("SAMEORIGIN")             // Override specific headers
    .content_security_policy("default-src 'self'; script-src 'self' 'unsafe-inline'")
    .without_hsts();                           // Disable specific headers

global_middleware!(headers);
```

This follows `CsrfMiddleware::new().except(vec![...])` pattern already established.

### Pattern 3: Environment-Aware Defaults

**What:** Different defaults for development vs production.
**When to use:** Headers like HSTS that break local development.

HSTS must NOT be sent over HTTP (browsers ignore it, but it signals misconfiguration). In development (`localhost`, no TLS), HSTS should be disabled by default.

Approach: Check `APP_ENV` or provide `SecurityHeaders::production()` vs `SecurityHeaders::development()` constructors. Simpler: just skip HSTS unless explicitly enabled or environment is production.

### Anti-Patterns to Avoid

- **X-XSS-Protection: 1**: OWASP recommends `0` (disabled). The header "can create XSS vulnerabilities in otherwise safe websites." Modern browsers use CSP instead.
- **HSTS with preload by default**: Preload submission to browser lists is permanent and affects all subdomains. Never default to `preload`.
- **Overly strict CSP by default**: Will break Inertia.js, Vite HMR, inline styles. Start permissive, let users tighten.
- **Setting headers on every response including static files**: Security headers should apply to all responses (including error pages), but CSP might need different values for API JSON responses vs HTML pages.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Nothing for this phase | — | — | Security headers are simple string constants. No complex logic, algorithms, or edge cases that warrant a library. The entire implementation is ~100 lines of Rust. |

**Key insight:** This is one of the rare cases where hand-rolling IS the right approach. External crates (`http-security-headers`, `rust-helmet`) target different framework middleware systems (tower, actix). Ferro's `Middleware` trait is custom, so we'd need an adapter layer that's more code than just setting the headers directly.

The OWASP recommended values are stable, well-documented string constants. There's no algorithmic complexity to get wrong.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: X-XSS-Protection Set to 1
**What goes wrong:** Enables XSS Auditor in older browsers, which can itself create XSS vulnerabilities
**Why it happens:** Intuitive to think "1 = enabled = safer". Older guides recommend `1; mode=block`.
**How to avoid:** Always set to `0`. OWASP explicitly recommends disabling it. Use CSP instead.
**Warning signs:** Security scanner still flags it — some scanners haven't updated their rules.

### Pitfall 2: HSTS Breaking Development
**What goes wrong:** `Strict-Transport-Security` sent over HTTP on localhost, or HSTS cached by browser makes localhost inaccessible
**Why it happens:** Middleware applies globally including development environment
**How to avoid:** Skip HSTS when not running over TLS or when APP_ENV is not production. Provide `.without_hsts()` builder method.
**Warning signs:** "This site can't be reached" errors on localhost after testing with HSTS enabled.

### Pitfall 3: CSP Breaking Inertia.js/Vite
**What goes wrong:** Strict CSP blocks Vite HMR WebSocket, inline scripts from Vite, or React's runtime
**Why it happens:** Default CSP `script-src 'self'` blocks inline scripts. Vite injects inline scripts for HMR.
**How to avoid:** Default CSP should include `'unsafe-inline'` for scripts/styles, or use nonces. In development, CSP should be very permissive or disabled.
**Warning signs:** Blank page in browser, console errors about "refused to execute inline script".

### Pitfall 4: HSTS Preload as Default
**What goes wrong:** Submitting domain to HSTS preload list is permanent — removing it takes months
**Why it happens:** Copy-pasting "recommended" HSTS header which includes `preload`
**How to avoid:** Default to `max-age=31536000; includeSubDomains` WITHOUT preload. Preload must be explicitly opted into.
**Warning signs:** None until you need to serve HTTP for a subdomain and can't.

### Pitfall 5: Duplicate Headers
**What goes wrong:** Security headers appear twice if both middleware and reverse proxy (nginx) set them
**Why it happens:** Common deployment pattern has nginx adding headers, plus application middleware
**How to avoid:** Document that SecurityHeaders should be used in application OR reverse proxy, not both. Consider checking if header already exists before setting it.
**Warning signs:** Double values in response headers.

### Pitfall 6: Not Setting Headers on Error Responses
**What goes wrong:** 4xx/5xx error pages don't get security headers
**Why it happens:** Middleware only processes `Ok(response)`, not `Err(response)`
**How to avoid:** Apply headers to BOTH Ok and Err variants of the Response type.
**Warning signs:** Security scanner reports missing headers on error pages.
</common_pitfalls>

<code_examples>
## Code Examples

### Middleware Structure (following CsrfMiddleware pattern)

```rust
// Source: Ferro framework patterns (csrf/middleware.rs, middleware/metrics.rs)
pub struct SecurityHeaders {
    x_content_type_options: Option<String>,
    x_frame_options: Option<String>,
    strict_transport_security: Option<String>,
    content_security_policy: Option<String>,
    referrer_policy: Option<String>,
    permissions_policy: Option<String>,
    cross_origin_opener_policy: Option<String>,
    x_xss_protection: Option<String>,
}

impl SecurityHeaders {
    pub fn new() -> Self {
        Self {
            x_content_type_options: Some("nosniff".to_string()),
            x_frame_options: Some("DENY".to_string()),
            strict_transport_security: None, // Off by default (breaks localhost)
            content_security_policy: Some(
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' ws: wss:; frame-ancestors 'none'".to_string()
            ),
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some(
                "geolocation=(), camera=(), microphone=()".to_string()
            ),
            cross_origin_opener_policy: Some("same-origin".to_string()),
            x_xss_protection: Some("0".to_string()),
        }
    }

    /// Enable HSTS for production (1 year, includeSubDomains, no preload)
    pub fn with_hsts(mut self) -> Self {
        self.strict_transport_security =
            Some("max-age=31536000; includeSubDomains".to_string());
        self
    }

    fn apply_headers(&self, mut resp: HttpResponse) -> HttpResponse {
        if let Some(ref v) = self.x_content_type_options {
            resp = resp.header("X-Content-Type-Options", v.as_str());
        }
        if let Some(ref v) = self.x_frame_options {
            resp = resp.header("X-Frame-Options", v.as_str());
        }
        // ... repeat for each header
        resp
    }
}
```

### Bootstrap.rs Registration

```rust
// In bootstrap.rs — generated by `ferro new`
use ferro::{global_middleware, SecurityHeaders};

// Security headers (runs on every response)
global_middleware!(SecurityHeaders::new());

// Or for production with HSTS:
// global_middleware!(SecurityHeaders::new().with_hsts());
```

### Response Header Application (both Ok and Err)

```rust
// Source: Ferro middleware pattern
#[async_trait]
impl Middleware for SecurityHeaders {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let response = next(request).await;
        match response {
            Ok(resp) => Ok(self.apply_headers(resp)),
            Err(resp) => Err(self.apply_headers(resp)),
        }
    }
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| X-XSS-Protection: 1; mode=block | X-XSS-Protection: 0 | OWASP 2023+ | Old value can CREATE XSS vulnerabilities |
| Feature-Policy header | Permissions-Policy header | 2021+ | Name and syntax changed |
| Expect-CT header | Deprecated, remove | 2023+ | No longer needed |
| Public-Key-Pins (HPKP) | Deprecated, remove | 2018+ | Was dangerous, never widely adopted |
| CSP Level 2 | CSP Level 3 (with strict-dynamic, nonces) | 2023+ | Nonce-based CSP is preferred over allowlists |
| X-Frame-Options only | CSP frame-ancestors preferred | 2022+ | X-Frame-Options still needed for IE11; both recommended |

**New headers to consider:**
- **Cross-Origin-Opener-Policy (COOP):** `same-origin` — isolates browsing context, prevents Spectre attacks
- **Cross-Origin-Embedder-Policy (COEP):** `require-corp` — but can break third-party resources (skip by default)
- **Cross-Origin-Resource-Policy (CORP):** `same-site` — but can affect API responses to cross-origin clients (skip by default)

**Decisions for Ferro defaults:**
- Include COOP (`same-origin`) — minimal breakage, good protection
- Skip COEP/CORP by default — too likely to break legitimate cross-origin requests
- Provide builder methods to enable them when needed
</sota_updates>

<open_questions>
## Open Questions

1. **CSP nonce support for Inertia.js/Vite**
   - What we know: Laravel implements `Vite::useCspNonce()` which generates per-request nonces and injects them into script/style tags. Nonce-based CSP is the modern best practice.
   - What's unclear: Ferro's Inertia integration renders HTML server-side — can we inject nonces into the Vite script tags? How does Ferro's Inertia adapter generate the HTML that includes Vite assets?
   - Recommendation: Ship without nonce support in v1. Use `'unsafe-inline'` for scripts/styles as a safe default. Add nonce support as a follow-up if demand exists. The current phase goal is headers middleware, not full CSP nonce infrastructure.

2. **Static file response headers**
   - What we know: Static files are served via `handle_request()` bypassing the middleware chain (they use `hyper::Response<Full<Bytes>>` directly, per Phase 72 decisions).
   - What's unclear: Should security headers apply to static file responses too?
   - Recommendation: Check if static files go through the middleware chain. If not, this is a known gap to document. Security headers on static files matter less (they're images/CSS/JS, not HTML), but it's still best practice.

3. **JSON-UI vs Inertia.js CSP differences**
   - What we know: JSON-UI renders HTML server-side (more CSP-friendly). Inertia.js uses client-side React (needs looser CSP for inline scripts).
   - What's unclear: Should the default CSP differ based on which rendering mode is in use?
   - Recommendation: Use a single permissive default that works for both. Users can tighten CSP for their specific setup.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [OWASP HTTP Headers Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet.html) — Complete header recommendations with values
- [OWASP Secure Headers Project](https://owasp.org/www-project-secure-headers/) — Updated Jan 2026
- [MDN Permissions-Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Permissions-Policy) — Directive reference with defaults
- [MDN Strict-Transport-Security](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Strict-Transport-Security) — HSTS specification
- [OWASP HSTS Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Strict_Transport_Security_Cheat_Sheet.html) — HSTS best practices
- [Laravel Vite CSP Nonce docs](https://laravel.com/docs/12.x/vite) — Laravel's nonce implementation approach

### Secondary (MEDIUM confidence)
- [Auth0: Deploying CSP in SPAs](https://auth0.com/blog/deploying-csp-in-spa/) — SPA CSP challenges, verified against MDN
- [HSTS Preload](https://hstspreload.org/) — Preload list requirements
- [Spatie Laravel-CSP](https://github.com/spatie/laravel-csp) — Laravel CSP middleware patterns (reference for API design)

### Tertiary (LOW confidence - needs validation)
- [rust-helmet](https://github.com/danielkov/rust-helmet) — Rust security headers library, unclear maintenance status (20 commits total)
- [http-security-headers crate](https://crates.io/crates/http-security-headers) — Couldn't load full details (crates.io requires JS)
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: HTTP security response headers
- Ecosystem: OWASP standards, Rust crates (evaluated, not needed)
- Patterns: Post-processing middleware, builder configuration, environment-aware defaults
- Pitfalls: XSS-Protection, HSTS in dev, CSP + SPA, duplicate headers, error responses

**Confidence breakdown:**
- Standard stack: HIGH — OWASP recommendations are authoritative and stable
- Architecture: HIGH — follows existing Ferro middleware patterns (CsrfMiddleware, MetricsMiddleware)
- Pitfalls: HIGH — well-documented in OWASP guides and MDN
- Code examples: HIGH — based on existing Ferro codebase patterns

**Research date:** 2026-02-26
**Valid until:** 2026-03-26 (30 days — security header standards are stable)
</metadata>

---

*Phase: 73-security-headers*
*Research completed: 2026-02-26*
*Ready for planning: yes*
