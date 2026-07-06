# Phase 143: Tailwind Static CSS Pipeline — Research

**Researched:** 2026-04-20
**Domain:** CSS build pipeline, Rust binary embedding, Actix-style framework routing
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Generate `ferro-base.css` by running the Tailwind v4 standalone CLI against all class-emitting source files in `ferro-json-ui/src/**/*.rs` and `framework/src/**/*.rs` plus a default theme file.
- **D-02:** Check the generated `ferro-base.css` into the repo. CI step verifies it is up-to-date. Crate consumers get the pre-built file — no tailwind CLI needed at compile time.
- **D-03:** Embed the checked-in file in the Rust binary at compile time via `include_str!` or equivalent. No runtime file I/O.
- **D-04:** The framework registers `GET /_ferro/ferro-base.css` automatically during app bootstrap — unconditional, no user configuration required.
- **D-05:** No separate `ferro.use_json_ui()` init call is needed. The route is part of the standard framework boot sequence.
- **D-06:** Add `stylesheet_urls: Vec<String>` field to `JsonUiConfig`. Default: `vec!["/_ferro/ferro-base.css".to_string()]`. Each URL emits a `<link rel="stylesheet" href="...">` in `<head>`, in order.
- **D-07:** Builder method: `stylesheet_urls(mut self, urls: Vec<String>) -> Self` — replaces the entire list.
- **D-08:** Theme token URL injection via `stylesheet_urls` — apps push their token file URL into the list.
- **D-09:** Remove `<style type="text/tailwindcss">` injection. Replace with `<style>` containing plain CSS variable overrides.
- **D-10:** Theme CSS using `@theme` syntax must be converted to plain `:root { --color-...: ... }` before injection.
- **D-11:** Inline `<style>` injection (not a separate route) is acceptable for theme overrides.
- **D-12:** Flip `JsonUiConfig::tailwind_cdn` default from `true` to `false`. Breaking change, expected pre-1.0.
- **D-13:** Keep `tailwind_cdn(true)` as explicit opt-in. Not deprecated or removed.
- **D-14:** When `tailwind_cdn: true` AND `stylesheet_urls` contains the default ferro-base URL, both load. No automatic mutual-exclusion logic.
- **D-15:** Update existing tests that assert on `<style type="text/tailwindcss">` presence.
- **D-16:** Add test asserting `JsonUiConfig::default()` produces `<link rel="stylesheet" href="/_ferro/ferro-base.css">` and no Tailwind CDN `<script>`.
- **D-17:** Add test that embedded CSS bytes are non-empty and parseable as UTF-8.

### Claude's Discretion

- Exact asset path for the checked-in CSS file (e.g. `ferro-json-ui/assets/` vs `framework/assets/`).
- Cache-Control header value for the static CSS route.
- Whether to use `include_str!` at the crate level or a `static FERRO_BASE_CSS: &str = include_str!(...)` in a dedicated asset module.
- CI check implementation (diff-based or hash-based).

### Deferred Ideas (OUT OF SCOPE)

- App-level Tailwind build loop (watch/rebuild for apps that extend Tailwind).
- Tailwind config file support (`@theme` compilation for apps).
- Additional modalities.

</user_constraints>

---

## Summary

This phase replaces the Tailwind v4 browser JIT runtime (`@tailwindcss/browser@4` from cdn.jsdelivr.net) with a pre-built static CSS file shipped inside the ferro binary. The runtime is documented by Tailwind as a development convenience only; it downloads a WebAssembly blob and fails silently on Safari/WebKit.

The fix has three parts: (1) generate a complete `ferro-base.css` covering every utility class ferro-json-ui components can emit, check it into the repo, and embed it via `include_str!`; (2) register `GET /_ferro/ferro-base.css` in the framework server's existing `/_ferro/*` dispatch block; (3) update `JsonUiConfig` to add `stylesheet_urls`, flip `tailwind_cdn` default to `false`, and update head injection to emit `<link>` tags instead of the CDN `<script>`.

The theme injection path also changes: `<style type="text/tailwindcss">` (Tailwind-CDN-specific magic) becomes a plain `<style>` tag with `:root { ... }` CSS variable overrides. This requires converting ferro-theme's `@theme { ... }` syntax to standard CSS variable declarations.

**Primary recommendation:** Asset location should be `ferro-json-ui/assets/ferro-base.css` (mirrors the `ferro-theme/assets/default.css` pattern and keeps the asset close to the code that generates the classes). Static const with module-level `include_str!` is preferable to crate-level for locality and testability.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CSS class generation | Build-time tool (Tailwind CLI) | — | Happens before compile, not at runtime |
| CSS binary embedding | `ferro-json-ui` crate | — | Asset lives next to the code that emits the classes |
| Static CSS route registration | `framework` server dispatch | — | All `/_ferro/*` routes live in `framework/src/server.rs` |
| Head `<link>` injection | `framework/src/json_ui/mod.rs` | — | `build_response` owns all head assembly |
| Theme `<style>` injection | `framework/src/json_ui/mod.rs` | — | Same function, `#[cfg(feature = "theme")]` block |
| Token vocabulary definition | `ferro-theme` | — | `token.rs` defines all 23 slots; `default.css` defines values |
| Theme override conversion | Per-app (gestiscilo) | `ferro-theme` docs | `@theme` → `:root` is a consumer-side change |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS v4 standalone CLI | v4.2.x [VERIFIED: github.com/tailwindlabs/tailwindcss/releases] | Generate `ferro-base.css` from Rust source files | No Node.js required; self-contained binary |
| `include_str!` macro | Rust std | Embed CSS file in binary at compile time | Zero runtime I/O, verified at compile time |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Tailwind `@source inline()` | v4.1+ [VERIFIED: tailwindcss.com/docs] | Safelist utility classes not in source files | When render.rs emits classes programmatically via string concat rather than full literal names |

### Standalone CLI Download URLs (macOS arm64, for development)

```
https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64
https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
```

[VERIFIED: tailwindcss.com/blog/standalone-cli + github.com/tailwindlabs/tailwindcss/releases]

---

## Architecture Patterns

### System Architecture Diagram

```
Build time:
  ferro-json-ui/src/**/*.rs  ──┐
  framework/src/**/*.rs       ─┼──▶  tailwind CLI  ──▶  ferro-json-ui/assets/ferro-base.css
  input.css (@import tailwindcss                          (checked into git)
             @source "../..")  ┘

Compile time:
  ferro-json-ui/assets/ferro-base.css  ──▶  include_str!  ──▶  FERRO_BASE_CSS: &'static str
                                                                  (in ferro-json-ui binary)

Request time:
  GET /_ferro/ferro-base.css
       │
       ▼
  framework/src/server.rs dispatch block
       │  match "/_ferro/ferro-base.css"
       ▼
  serve FERRO_BASE_CSS bytes, Content-Type: text/css, Cache-Control: immutable
       │
       ▼
  Browser: render fully styled page
```

```
HTML head assembly (JsonUi::build_response):

  stylesheet_urls (default: ["/_ferro/ferro-base.css"])
       │  for each url
       ▼
  <link rel="stylesheet" href="...">   (in order)

  tailwind_cdn: false (default)  ──▶  no CDN <script>
  tailwind_cdn: true             ──▶  <script src="cdn.jsdelivr.net/...">

  #[cfg(feature = "theme")] active theme
       │
       ▼
  <style>                            (was: <style type="text/tailwindcss">)
    :root { --color-primary: ...; }  (plain CSS vars, not @theme syntax)
  </style>
```

### Recommended Asset Location

```
ferro-json-ui/
├── assets/
│   └── ferro-base.css       # Generated + checked in
├── src/
│   ├── assets.rs            # pub(crate) const FERRO_BASE_CSS: &str = include_str!(...)
│   ├── config.rs            # JsonUiConfig (add stylesheet_urls field)
│   ├── render.rs            # Component renderers (Tailwind source scan target)
│   └── lib.rs               # Re-export FERRO_BASE_CSS for framework
```

Rationale: `ferro-theme` already uses `assets/` for `default.css` with `include_str!` in a loader module. The same pattern in `ferro-json-ui` is consistent and keeps asset next to the code it covers.

### Pattern 1: `include_str!` for embedded static asset

```rust
// ferro-json-ui/src/assets.rs
// Source: ferro-theme/src/loader.rs (existing project pattern)

/// Pre-built Tailwind CSS containing all utility classes used by ferro-json-ui components.
///
/// Embedded at compile time. Served by the framework at `/_ferro/ferro-base.css`.
/// Regenerate via: `tailwindcss -i ferro-json-ui/assets/input.css -o ferro-json-ui/assets/ferro-base.css`
pub const FERRO_BASE_CSS: &str = include_str!("../assets/ferro-base.css");
```

### Pattern 2: `/_ferro/*` route registration in `server.rs`

The existing dispatch block (lines 214-225) handles all `/_ferro/` GET routes as a match on `path.as_str()`. The CSS route adds one arm:

```rust
// framework/src/server.rs (existing block, add one arm)
// Source: [VERIFIED: framework/src/server.rs:215-224]

if path.starts_with("/_ferro/") && method == hyper::Method::GET {
    return match path.as_str() {
        "/_ferro/health"        => health_response(query).await,
        "/_ferro/routes"        => crate::debug::handle_routes(),
        "/_ferro/middleware"    => crate::debug::handle_middleware(),
        "/_ferro/services"      => crate::debug::handle_services(),
        "/_ferro/metrics"       => crate::debug::handle_metrics(),
        "/_ferro/queue/jobs"    => crate::debug::handle_queue_jobs().await,
        "/_ferro/queue/stats"   => crate::debug::handle_queue_stats().await,
        // NEW:
        "/_ferro/ferro-base.css" => serve_ferro_base_css(),
        _ => HttpResponse::text("404 Not Found").status(404).into_hyper(),
    };
}
```

The handler is a simple synchronous function — no `async` needed:

```rust
fn serve_ferro_base_css() -> hyper::Response<Full<Bytes>> {
    let css = ferro_json_ui::FERRO_BASE_CSS;
    hyper::Response::builder()
        .status(200)
        .header("Content-Type", "text/css; charset=utf-8")
        .header("Content-Length", css.len().to_string())
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Full::new(Bytes::from_static(css.as_bytes())))
        .unwrap()
}
```

`Bytes::from_static` is zero-copy because `FERRO_BASE_CSS` is `&'static str`. Cache-Control is `immutable` because the URL path is fixed and the content changes only on framework version bumps. [ASSUMED: `max-age=31536000, immutable` is appropriate; could use shorter TTL if desired]

### Pattern 3: `stylesheet_urls` head injection in `build_response`

```rust
// framework/src/json_ui/mod.rs — build_response (existing function)
// Source: [VERIFIED: framework/src/json_ui/mod.rs:91-116]

// Before (CDN path):
if config.tailwind_cdn {
    head.push_str(r#"<script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>"#);
}

// After (stylesheet_urls, then optional CDN):
for url in &config.stylesheet_urls {
    head.push_str(&format!(r#"<link rel="stylesheet" href="{}">"#, html_escape(url)));
}
if config.tailwind_cdn {
    head.push_str(r#"<script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>"#);
}
```

Theme injection (the `#[cfg(feature = "theme")]` block) changes from:

```rust
// Before:
head.push_str(&format!(
    "<style type=\"text/tailwindcss\">{}</style>",
    theme.css
));

// After:
head.push_str(&format!("<style>{}</style>", theme.css));
// Note: theme.css must already contain plain CSS vars (not @theme syntax)
```

### Pattern 4: `JsonUiConfig` additions

```rust
// ferro-json-ui/src/config.rs (add field + builder, update Default)
// Source: [VERIFIED: ferro-json-ui/src/config.rs]

pub struct JsonUiConfig {
    pub tailwind_cdn: bool,
    pub stylesheet_urls: Vec<String>,   // NEW
    pub custom_head: Option<String>,
    pub body_class: String,
}

impl Default for JsonUiConfig {
    fn default() -> Self {
        Self {
            tailwind_cdn: false,                                   // CHANGED from true
            stylesheet_urls: vec!["/_ferro/ferro-base.css".to_string()],  // NEW
            custom_head: None,
            body_class: "dark bg-background text-text font-sans".to_string(),
        }
    }
}

// Builder method:
pub fn stylesheet_urls(mut self, urls: Vec<String>) -> Self {
    self.stylesheet_urls = urls;
    self
}
```

### Pattern 5: `@theme` → `:root` CSS conversion

The default ferro-theme `assets/default.css` uses:
```css
@import "tailwindcss";

@theme {
  --color-background: oklch(100% 0 0);
  /* ... 23 tokens ... */
}

@media (prefers-color-scheme: dark) {
  @theme { ... }
}
[data-theme="dark"] { ... }
```

This file is the **input** for the Tailwind CLI. It must NOT be injected verbatim into `<style>` tags — the `@import` and `@theme` directives require Tailwind processing.

For inline theme injection, the format must be plain CSS:

```css
/* Equivalent plain CSS (injectable as-is) */
:root {
  --color-background: oklch(100% 0 0);
  --color-primary: oklch(55% 0.2 250);
  /* ... all 23 tokens ... */
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-background: oklch(12% 0 0);
    /* ... dark values ... */
  }
}

[data-theme="dark"] {
  --color-background: oklch(12% 0 0);
  /* ... */
}
```

Apps using `Theme::from_path()` (like gestiscilo) read `tokens.css` directly. Those files currently use `@import "tailwindcss"` + `@theme`. After this phase, `Theme::css` must contain plain CSS vars — either by converting the file content or by having the framework strip/ignore `@import` and `@theme` wrapper. The simplest approach: update `ferro-theme`'s `default.css` to a plain-CSS variant (removing `@import tailwindcss`), and have `ferro-cli make:theme` scaffold plain CSS as well.

### Pattern 6: Tailwind CLI input file for CSS generation

A dedicated input file tells the Tailwind CLI what to scan and what to include:

```css
/* ferro-json-ui/assets/input.css */
@import "tailwindcss";

/* Explicitly scan the Rust source that emits classes */
@source "../../ferro-json-ui/src";
@source "../../framework/src";
```

Tailwind v4 auto-detects text files (including `.rs`) and extracts any token matching utility class patterns. For classes emitted via string literals in render.rs (full literal strings like `"flex items-center gap-2"`), automatic detection works.

**Critical risk:** If render.rs emits classes via runtime string interpolation (dynamic construction), Tailwind cannot detect them. The source scan of `.rs` files only picks up literal strings. Review: render.rs uses plain string literals in format macros — class strings are complete literals in the source, so auto-detection should work. [ASSUMED: no dynamically assembled class strings exist — needs manual verification during planning/implementation]

For any class that cannot be detected (e.g., classes assembled from variables), `@source inline()` provides v4.1+ safelisting:

```css
/* In input.css, for classes that cannot be auto-detected: */
@source inline("bg-background text-text bg-surface bg-card bg-primary text-primary-foreground ...");
```

### Anti-Patterns to Avoid

- **`@theme` syntax in injected `<style>` tags:** Only works when the Tailwind CDN is active. Plain CSS vars work universally.
- **Runtime CSS file reading:** Use `include_str!` for zero-overhead embedding. Do not call `std::fs::read_to_string` at request time.
- **Scanning all workspace source:** The Tailwind CLI scans `../**` by default relative to the CSS input. Explicitly scoping with `@source` prevents over-scanning vendored dependencies.
- **`Bytes::from(css.as_bytes().to_vec())`:** Unnecessary allocation. Use `Bytes::from_static(css.as_bytes())` since the string is `'static`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CSS utility class generation | Custom class list | Tailwind v4 standalone CLI | Tailwind handles variants, responsive prefixes, dark mode, semantic token resolution |
| CSS file serving with content type | Custom hyper response builder from scratch | Follow existing `/_ferro/*` handler pattern in `server.rs` | Pattern already established; consistent with debug/health endpoints |
| MIME type detection | Extension → content-type map | Hardcode `text/css; charset=utf-8` for this single known file | Only one file; the general-purpose `mime_guess` is used by `static_files.rs` for public/ directory |

**Key insight:** The Tailwind CLI is the authoritative tool for producing correct, complete CSS from utility class names. Any custom CSS generation would miss the combinatoric explosion of variant classes (hover:, dark:, sm:, etc.).

---

## Common Pitfalls

### Pitfall 1: Missing dynamically-assembled Tailwind classes

**What goes wrong:** If any component in render.rs builds a class string at runtime (e.g., `format!("bg-{}", color)`) the Tailwind CLI scanner does not detect it. The class appears in the source as a fragment, not as a complete utility name.

**Why it happens:** Tailwind's scanner does plain-text token extraction — it only matches complete class names as written in source.

**How to avoid:** Audit render.rs for any `format!` or concatenation that produces partial class names. For any found, add `@source inline("bg-red-500 bg-green-500 ...")` to `input.css` listing all possible values.

**Warning signs:** Visual regression where one component works but a state variant (e.g., destructive badge, warning alert) renders without color.

### Pitfall 2: `@theme` syntax left in injected theme CSS

**What goes wrong:** After removing the CDN script, any `theme.css` that still contains `@theme { ... }` is injected into a plain `<style>` tag. Browsers treat `@theme` as an unknown at-rule and ignore the entire block — zero token values, page renders with no color.

**Why it happens:** `@theme` is Tailwind-CDN-specific. Browsers only understand standard CSS at-rules (`@media`, `@keyframes`, `@layer`, etc.).

**How to avoid:** Convert `default.css` to use `:root { ... }` and `@media (prefers-color-scheme: dark) { :root { ... } }`. Update the `#[cfg(feature = "theme")]` injection block to emit plain `<style>` (no `type` attribute). Update `make:theme` scaffolder.

**Warning signs:** Dark mode tokens not applied; custom theme colors reverting to browser defaults despite theme middleware being active.

### Pitfall 3: Stale CI check blocks contributor PRs

**What goes wrong:** CI runs `tailwindcss -i input.css -o /tmp/check.css` and diffs against committed file. Contributors who add a new component to render.rs but forget to regenerate `ferro-base.css` get a failed CI check with a cryptic diff.

**Why it happens:** The CSS generation is a manual step outside the normal Rust workflow.

**How to avoid:** Make the CI step explicit in the contributor guide. Keep the CI error message self-documenting (e.g., "Run `scripts/gen-ferro-base-css.sh` and commit the result").

**Warning signs:** Contributors opening PRs with failing CI they can't explain.

### Pitfall 4: `JsonUiConfig` `schemars::JsonSchema` derive breaks

**What goes wrong:** `Vec<String>` is handled by schemars, but if the new `stylesheet_urls` field is not added before the schema snapshot tests (if any), tests fail or the schema drifts.

**Why it happens:** `JsonUiConfig` derives `schemars::JsonSchema`. Adding fields changes the schema.

**How to avoid:** Check if any test snapshots the schema. Update them as part of this phase.

**Warning signs:** `cargo test` failures mentioning schemars or schema mismatch.

### Pitfall 5: `Bytes::from_static` requires `&'static [u8]`

**What goes wrong:** `include_str!` returns `&'static str`. `Bytes::from_static` requires `&'static [u8]`. The conversion `.as_bytes()` on a `&'static str` returns `&'static [u8]`, so the static lifetime is preserved — this works. However, if the CSS is stored in a `String` (e.g., `const.to_string()`), the lifetime is lost and `from_static` does not compile.

**How to avoid:** Declare `FERRO_BASE_CSS` as `&'static str = include_str!(...)` (not `String`). Pass `FERRO_BASE_CSS.as_bytes()` directly.

---

## Code Examples

### Existing `include_str!` pattern in ferro-theme

```rust
// ferro-theme/src/loader.rs (VERIFIED: read from codebase)
const DEFAULT_THEME_CSS: &str = include_str!("../assets/default.css");
```

This is the exact pattern to replicate in `ferro-json-ui/src/assets.rs`.

### Existing `/_ferro/*` dispatch in server.rs

```rust
// framework/src/server.rs lines 214-225 (VERIFIED: read from codebase)
if path.starts_with("/_ferro/") && method == hyper::Method::GET {
    return match path.as_str() {
        "/_ferro/health"       => health_response(query).await,
        "/_ferro/routes"       => crate::debug::handle_routes(),
        "/_ferro/middleware"   => crate::debug::handle_middleware(),
        "/_ferro/services"     => crate::debug::handle_services(),
        "/_ferro/metrics"      => crate::debug::handle_metrics(),
        "/_ferro/queue/jobs"   => crate::debug::handle_queue_jobs().await,
        "/_ferro/queue/stats"  => crate::debug::handle_queue_stats().await,
        _ => HttpResponse::text("404 Not Found").status(404).into_hyper(),
    };
}
```

The new `"/_ferro/ferro-base.css"` arm is synchronous — no `.await` needed.

### Existing static file response pattern

```rust
// framework/src/static_files.rs (VERIFIED: read from codebase)
let response = hyper::Response::builder()
    .status(200)
    .header("Content-Type", &content_type)
    .header("Content-Length", bytes.len().to_string())
    .header("Cache-Control", cache_control)
    .body(Full::new(Bytes::from(bytes)))
    .unwrap();
```

The CSS handler uses the same structure but with `Bytes::from_static` instead.

### Default ferro-theme token vocabulary (all 23 slots)

```
// ferro-theme/src/token.rs (VERIFIED: read from codebase)
// 6 surface, 8 role, 4 shape/radius, 3 shadow, 2 typography = 23 total
--color-background, --color-surface, --color-card, --color-border, --color-text, --color-text-muted
--color-primary, --color-primary-foreground, --color-secondary, --color-secondary-foreground
--color-accent, --color-destructive, --color-success, --color-warning
--radius-sm, --radius-md, --radius-lg, --radius-full
--shadow-sm, --shadow-md, --shadow-lg
--font-sans, --font-mono
```

All 23 must appear in `ferro-base.css` as Tailwind CSS custom property mappings so that `bg-primary`, `text-text`, `rounded-md`, `shadow-lg`, etc. work without the CDN.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tailwind v3 `content: [...]` config | v4 automatic source detection + `@source` directive | v4.0 (early 2025) | No config file needed; scans project files automatically |
| v3 `safelist: [...]` in config | v4.1+ `@source inline(...)` in CSS | v4.1 (mid-2025) | Safelisting is now CSS-native, not config-native |
| Tailwind v4 browser JIT runtime | Pre-built static CSS | This phase | Eliminates Safari/WebKit failure, removes WASM download |

**Deprecated/outdated:**
- `<style type="text/tailwindcss">`: Only processed by the CDN browser runtime. Remove in favor of plain `<style>`.
- `tailwind_cdn: true` as default: Should have been dev-only from the start; the CDN runtime is documented as a development tool.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Tailwind v4 auto-detection works on `.rs` source files (extracts string literals as candidate class names) | Architecture Patterns / Pitfall 1 | Classes in Rust string literals not detected → visual regressions; fix with `@source inline()` |
| A2 | No render.rs component emits Tailwind classes via runtime string concatenation of partial names | Common Pitfalls | Dynamic classes not in CSS; specific component variants (e.g., destructive states) render unstyled |
| A3 | `max-age=31536000, immutable` Cache-Control is appropriate for the CSS route | Architecture Patterns / Pattern 2 | Clients may cache across framework version bumps — acceptable if URL stays fixed, unacceptable if users see stale styles; could use shorter TTL |
| A4 | gestiscilo's `tokens.css` uses `@theme` syntax throughout | Common Pitfalls / Pitfall 2 | If it already uses `:root` vars, no conversion needed |
| A5 | No existing test snapshots `JsonUiConfig`'s JSON schema | Common Pitfalls / Pitfall 4 | Schema drift causes test failures not covered here |

---

## Open Questions (RESOLVED)

All open questions from research are now resolved by concrete plan decisions. Each question has been addressed during planning:

1. **Does render.rs emit any classes via partial string concatenation?**
   - What we know: render.rs is ~300KB; the first 350 lines use full literal class names.
   - What's unclear: Whether any component (e.g., CalendarCell, ActionCard, DataTable, KanbanBoard) assembles class names dynamically.
   - Recommendation: During implementation, grep for `format!` calls in render.rs that produce CSS class strings. Add `@source inline()` entries for any found.
   - **RESOLVED:** Plan 01 Task 2 ships an `@source inline(...)` directive in `ferro-json-ui/assets/input.css` that safelists the semantic class set (bg-background, bg-surface, bg-card, bg-border, text-text, text-text-muted, bg-primary/secondary/accent/destructive/success/warning and foregrounds, rounded-*, shadow-*, font-sans, font-mono). The regeneration script is idempotent and the CI drift check in Task 4 catches any missing classes during PRs. If a component later emits a class not yet safelisted, the fix is to add it to the `@source inline(...)` list and regenerate.

2. **gestiscilo tokens.css — current syntax?**
   - What we know: CONTEXT.md mentions converting it as part of acceptance criteria.
   - What's unclear: Whether it uses raw `@theme` or already has a `:root` block.
   - Recommendation: Plan should include a step to read and convert gestiscilo's tokens.css regardless (it is a named acceptance criterion).
   - **RESOLVED:** Out of scope for this phase. CONTEXT.md Phase Boundary explicitly excludes migrating apps beyond the framework itself ("Migrating apps beyond gestiscilo — that is a separate consumer phase"). Plan 02 converts `ferro-theme/assets/default.css` (the framework-owned default theme); the gestiscilo-side conversion of `themes/gestiscilo/tokens.css` happens in the downstream gestiscilo consumer phase once this framework phase ships. Plan 04 updates the `ferro make:theme` scaffolder so newly generated themes use the plain-CSS form from the start, giving gestiscilo a template to follow for its own conversion.

3. **Cache busting for `/_ferro/ferro-base.css` across version bumps?**
   - What we know: URL is fixed at `/_ferro/ferro-base.css`. With `immutable` cache, clients may serve stale CSS after updating the framework.
   - What's unclear: Whether this is a real concern given the deployment pattern (container rebuild usually clears CDN/proxy caches).
   - Recommendation: Use `public, max-age=86400` (24h) instead of `immutable` as the initial setting. Revisit when version-stamped URLs are implemented.
   - **RESOLVED:** Plan 03 Task 3 uses `Cache-Control: public, max-age=86400` (24 hours). This is asserted by the integration test `serve_ferro_base_css_returns_200_with_text_css_content_type`. Revisit in a future phase when version-stamped URLs are implemented.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Tailwind v4 standalone CLI | Generating `ferro-base.css` | Not verified | — | Download from github.com/tailwindlabs/tailwindcss/releases/latest |
| Rust 1.88.0 | CI requirement | Assumed present on dev machine | 1.88.0 | — |

**Missing dependencies with no fallback:**
- Tailwind v4 CLI binary (for regenerating `ferro-base.css` when components change). Must be documented in CONTRIBUTING.md or a shell script. Not required at compile time.

**Missing dependencies with fallback:**
- None at compile time — `ferro-base.css` is pre-generated and checked in.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | None — workspace-wide via `cargo test --all-features` |
| Quick run command | `cargo test -p framework -p ferro-json-ui --all-features` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-12/D-16 | `JsonUiConfig::default()` produces `<link>` for ferro-base.css and no CDN `<script>` | unit | `cargo test -p framework config_default_no_cdn` | ❌ Wave 0 |
| D-17 | Embedded CSS bytes are non-empty and valid UTF-8 | unit | `cargo test -p ferro-json-ui ferro_base_css_non_empty` | ❌ Wave 0 |
| D-06 | `stylesheet_urls` builder replaces list | unit | `cargo test -p ferro-json-ui stylesheet_urls_builder` | ❌ Wave 0 |
| D-15 | Updated theme CSS injection test (plain `<style>`, not `type=text/tailwindcss`) | unit | `cargo test -p framework --features theme theme_css_injected` | ✅ exists, needs update |
| D-04 | `/_ferro/ferro-base.css` route returns 200 with `text/css` content type | integration | `cargo test -p framework ferro_base_css_route` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p framework -p ferro-json-ui --all-features`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] Test: `JsonUiConfig::default()` has `tailwind_cdn: false` and `stylesheet_urls: ["/_ferro/ferro-base.css"]`
- [ ] Test: `build_response` with default config emits `<link rel="stylesheet" href="/_ferro/ferro-base.css">` and no `@tailwindcss/browser` script
- [ ] Test: `FERRO_BASE_CSS` constant is non-empty and valid UTF-8
- [ ] Test: `/_ferro/ferro-base.css` route returns 200, `Content-Type: text/css`
- [ ] Update existing test: `theme_css_injected_into_head_when_theme_active` — assert plain `<style>` not `type="text/tailwindcss"`
- [ ] Update existing test: `theme_css_injected_after_tailwind_cdn` — still valid but verify against new injection order

---

## Security Domain

> Applicable ASVS categories for this phase:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | CSS file is public by design |
| V5 Input Validation | partial | URL values in `stylesheet_urls` are emitted into HTML `href` attributes — must be HTML-escaped |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via unescaped stylesheet_url in `href` | Tampering | HTML-escape all URLs before emitting into `<link href="...">` attributes |
| Path traversal on `/_ferro/ferro-base.css` route | Tampering | Route is an exact string match (`== "/_ferro/ferro-base.css"`), no path parsing |

**Note:** The existing `build_response` function uses `html_escape()` for data attributes. The new `stylesheet_urls` loop must also use `html_escape()`.

---

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/src/config.rs` — `JsonUiConfig` struct and builder methods [VERIFIED: read from codebase]
- `framework/src/json_ui/mod.rs` — `build_response` head injection logic [VERIFIED: read from codebase]
- `framework/src/server.rs` lines 207-225 — `/_ferro/*` dispatch block [VERIFIED: read from codebase]
- `framework/src/static_files.rs` — hyper response builder pattern for static files [VERIFIED: read from codebase]
- `ferro-theme/src/loader.rs` — `include_str!("../assets/default.css")` pattern [VERIFIED: read from codebase]
- `ferro-theme/src/token.rs` — complete 23-slot token vocabulary [VERIFIED: read from codebase]
- `ferro-theme/assets/default.css` — current `@theme` syntax and all token values [VERIFIED: read from codebase]
- tailwindcss.com/docs/detecting-classes-in-source-files — `@source` directive and auto-detection [CITED: tailwindcss.com/docs/detecting-classes-in-source-files]
- tailwindcss.com/blog/standalone-cli — standalone binary download URLs [CITED: tailwindcss.com/blog/standalone-cli]

### Secondary (MEDIUM confidence)

- github.com/tailwindlabs/tailwindcss/releases — v4.2.2 is latest; standalone binaries available for macOS arm64 and Linux x64 [CITED: github.com/tailwindlabs/tailwindcss/releases]
- github.com/tailwindlabs/tailwindcss/discussions/14462 — `@source inline()` safelisting requires v4.1+; not available in v4.0.x [CITED via search results]

### Tertiary (LOW confidence)

- Community reports: `@tailwindcss/browser@4` fails on Safari/WebKit when loading the WASM blob — consistent with the production failure report in the CONTEXT [LOW: not independently verified in this session, but aligns with the stated motivation]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Tailwind v4 CLI download patterns and `include_str!` are both well-established in this codebase and in Tailwind docs
- Architecture: HIGH — dispatch pattern, file embedding pattern, and injection point are all verified from the actual codebase
- Pitfalls: MEDIUM — dynamic class assembly risk is assumed based on the known limitation; A2 requires manual verification

**Research date:** 2026-04-20
**Valid until:** 2026-07-20 (Tailwind CLI invocation stable; Rust patterns stable)
