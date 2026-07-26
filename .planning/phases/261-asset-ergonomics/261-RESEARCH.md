# Phase 261: `asset!()` ergonomics — Research

**Researched:** 2026-07-26
**Domain:** Proc-macro ergonomics, bundle registration, CLI fetch
**Confidence:** HIGH (all critical paths verified from source code)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `asset!("relative/path.ext")` expands to code containing `include_bytes!("relative/path.ext")` — call-site-source-relative path resolution. The macro does NOT read the file at proc-macro expansion time.
- **D-02:** Lazy register-once at the use site via a private `static OnceLock<String>` that, on first evaluation, registers the Bundle and caches its hashed URL. Safe inside per-request/hot render paths.
- **D-03:** Return type `&'static str` (via `static OnceLock<String>` → `get_or_init(...).as_str()`).
- **D-04:** Bundle name derived deterministically from the sanitized asset path (separators/dots → underscores).
- **D-05:** Add a `pub fn` ext→MIME helper in `ferro-bundle`. Cover extensions it already recognizes plus unknown → `application/octet-stream`. Do NOT merge with `ferro-assets::infer_content_type` (different return type and purpose).
- **D-06:** Re-export `ferro-bundle` from `framework` as `ferro::bundle`. Macro emits `::ferro::bundle::Bundle`. Reuses `crate::utils::ferro` root-path helper.
- **D-07:** `assets` command group with `fetch` subcommand — `ferro assets fetch iconify <set>[/<icon>…]` and `ferro assets fetch fontsource <family>`.
- **D-08:** Reuse existing `reqwest` blocking + rustls-tls dep in `ferro-cli/Cargo.toml`.
- **D-09:** Write individual servable files into `assets/` at project root (create if missing; `--out` override). Fetch only downloads; does NOT generate `asset!()` calls.

### Claude's Discretion

- Exact Iconify + Fontsource endpoint URLs and response formats.
- `OnceLock` vs `LazyLock` for use-site cache.
- Exact name-sanitization function.
- Whether macro accepts optional stable alias argument (recommend NO for v17.0).

### Deferred Ideas (OUT OF SCOPE)

- Macro-emitted stable alias (`asset!("path", alias = "/app.js")`).
- Auto-wiring fetched assets into `asset!()` calls / route generation.
- Delta-granular / list-diffing directions.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LIVE-03 | `asset!()` macro (embed + content-type infer + bundle registration) + opt-in `ferro assets fetch` for Iconify/Fontsource on Rust toolchain alone | D-01..D-09 locked; substrate (ferro-bundle, ferro-macros, ferro-cli) fully verified in source; API endpoints confirmed |
</phase_requirements>

---

## Summary

Phase 261 collapses the current three-call boot-time `Bundle::new(name, bytes).content_type(ct).with_alias(path)` chain into a single `asset!("path")` expression at the use site, plus adds an opt-in author-time CLI command that fetches Iconify icon sets and Fontsource families using only the Rust toolchain.

The substrate for all three deliverables is already present and stable:
- `ferro-bundle` owns `Bundle::new`, content-type management, and the process-global SHA-256 content-hash registry.
- `ferro-macros` has the `#[memoize]` proc-macro as a direct structural precedent, including the `crate::utils::ferro()` root-path helper and the `trybuild`-based UI test infrastructure.
- `ferro-cli` already has `reqwest 0.12` with `blocking + rustls-tls` features; the clap-derive subcommand pattern is well-established.

**Critical architectural blocker (D-06):** `ferro-bundle/Cargo.toml` declares `ferro-rs` (the `framework` crate) as a dependency. `framework` is Wave 2 in `publish.yml`; `ferro-bundle` is Wave 3. This means `framework` **cannot** add `ferro-bundle` as a dependency without creating a circular dependency cycle. The re-export path `::ferro::bundle::Bundle` as described in D-06 is impossible as stated. The planner MUST resolve this before assigning tasks. The recommended resolution is described in the Architecture Patterns section below.

**Primary recommendation:** Implement D-01..D-05 and D-07..D-09 as specified. For D-06, break the circular dependency by having `ferro-bundle` NOT import from `ferro-rs`, or by adding a thin `ferro-bundle-types` crate (violates no-new-crates rule), or — most practically — by having the macro emit `::ferro_bundle::Bundle` directly (i.e., `ferro-bundle` is a direct dependency of `ferro-macros` tests and the consumer app, not re-exported through `framework`). The research section documents this resolution in detail.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Proc-macro expansion (`asset!()`) | `ferro-macros` (compile time) | — | Proc-macro crates are compile-time only; no runtime tier |
| Bundle registration | `ferro-bundle` (library, process-global) | framework app boot | ferro-bundle owns the OnceLock DashMap registry |
| Content-type ext→MIME | `ferro-bundle` (new `pub fn` helper) | — | Single source of truth collocated with `ext_from_content_type` |
| Framework re-export | `framework` (feature-gated module) | — | All other crates re-exported here; but cycle risk blocks it |
| CLI fetch (Iconify/Fontsource) | `ferro-cli` (binary, author-time) | — | Never part of a library build; pure CLI side effect |
| HTTP download | `ferro-cli` via `reqwest` blocking | — | Already present dep, pure-Rust TLS |
| File output | `ferro-cli` → `assets/` dir | — | Author-time convention establishment |

---

## Standard Stack

### Core (already in-tree, verified)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `proc-macro2` | `1` | Token stream manipulation in proc-macros | Required for `quote!` / `syn` ecosystem |
| `quote` | `1` | Code generation from Rust token trees | Used in all existing ferro-macros |
| `syn` | `2` (features = "full") | Rust syntax parsing in proc-macros | Used in all existing ferro-macros |
| `ferro-bundle` | workspace `0.2` | `Bundle::new`, content-hash registry | The exact substrate `asset!()` delegates to |
| `reqwest` | `0.12` blocking + rustls-tls | HTTP download in ferro-cli | Already present; pure-Rust TLS; no nasm/OpenSSL |
| `clap` | `4` derive | CLI subcommand structure | Already used in ferro-cli |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::sync::OnceLock` | stdlib | Per-call-site lazy-init cache (D-02) | Stable since Rust 1.70; preferred over `once_cell` |
| `std::sync::LazyLock` | stdlib | Alternative to OnceLock for the static | Stable since Rust 1.80; requires init closure at definition; OnceLock preferred when init needs call-time data |

**Version verification:** All packages are already workspace dependencies; no new package installations needed.

---

## Architecture Patterns

### System Architecture Diagram

```
 COMPILE TIME                                RUNTIME
 ─────────────────────────────────          ─────────────────────────────
 asset!("assets/app.js")                    First call to asset!() site:
     │                                          │
     │  proc-macro expansion                    │  static OnceLock<String>
     ▼                                          │    get_or_init {
 include_bytes!("assets/app.js")  ─────────────▶    Bundle::new(name, bytes)
 (call-site-source-relative)                   │      .content_type(mime)
                                               │    .hashed_url()  ◀── &'static str
                                               ▼ }
                                           "/bundles/assets_app.a1b2c3d4.js"
                                               │
                                               │  HTTP GET /bundles/assets_app.a1b2c3d4.js
                                               ▼
                                           Bundle::serve(req)
                                           (mounted by app at boot on /bundles/*)
```

```
 AUTHOR TIME (ferro assets fetch)
 ────────────────────────────────────────────────────────────
 $ ferro assets fetch iconify heroicons
     │
     │  reqwest blocking GET https://api.iconify.design/heroicons/{icon}.svg
     │    (one request per icon, or full set via /heroicons.json)
     ▼
 assets/heroicons/check.svg
 assets/heroicons/x.svg
 ...
     │
     │  author writes: let url = asset!("assets/heroicons/check.svg");
     ▼
 flows through Bundle::new → .content_type("image/svg+xml")

 $ ferro assets fetch fontsource inter
     │
     │  reqwest blocking GET https://api.fontsource.org/v1/fonts/inter
     │    (get metadata: weights, subsets, woff2 URLs)
     │  reqwest blocking GET {woff2_url} for each (weight=400, subset=latin, style=normal)
     ▼
 assets/inter/latin-400-normal.woff2
 assets/inter/inter.css  (generated @font-face)
     │
     │  author writes: let _url = asset!("assets/inter/latin-400-normal.woff2");
     ▼
 flows through Bundle::new → .content_type("font/woff2")
```

### Recommended Project Structure (delta — no new crates)

```
ferro-bundle/src/
└── lib.rs              # ADD: pub fn mime_from_ext(ext: &str) -> &'static str

ferro-macros/src/
├── lib.rs              # ADD: #[proc_macro] pub fn asset + doc comment + re-export
├── asset.rs            # NEW: asset_impl() — the macro implementation
└── utils.rs            # unchanged — ferro() helper reused as-is

ferro-cli/src/
├── main.rs             # ADD: Assets { subcommand: AssetsCommand } variant
│                       # ADD: match arm → commands::assets::run(subcmd, out_dir)
└── commands/
    ├── mod.rs          # ADD: pub mod assets;
    └── assets.rs       # NEW: AssetsCommand enum + fetch_iconify() + fetch_fontsource()
```

### Pattern 1: `asset!()` macro expansion (D-01, D-02, D-03, D-04)

**What:** A `proc_macro` function in `ferro-macros/src/asset.rs`, registered in `lib.rs` and re-exported as `ferro::asset!`.

**Exact expansion emitted:**

```rust
// Source: ferro-bundle/src/lib.rs (Bundle::new, OnceLock pattern)
// Source: ferro-macros/src/memoize.rs:44 (ferro() root-path helper pattern)

// Input: asset!("assets/app.js")
// Expands to:
{
    static __FERRO_ASSET_URL: ::std::sync::OnceLock<::std::string::String> = ::std::sync::OnceLock::new();
    __FERRO_ASSET_URL.get_or_init(|| {
        static __FERRO_ASSET_BYTES: &[u8] = include_bytes!("assets/app.js");
        ::ferro_bundle::Bundle::new("assets_app_js", __FERRO_ASSET_BYTES)
            .content_type(::ferro_bundle::mime_from_ext("js"))
            .hashed_url()
    }).as_str()
}
```

**Key implementation notes:**
- The `static __FERRO_ASSET_BYTES` inside `get_or_init` is valid: `include_bytes!` produces a `&'static [u8]` even when lexically inside a closure, because `include_bytes!` is resolved at compile time.
- `OnceLock::get_or_init` returns `&T` where `T = String`; `.as_str()` gives `&str` with lifetime `'_` tied to the `OnceLock`. The static binding means this is effectively `&'static str` — the `OnceLock` itself is `'static`, so its `&String` ref is also `'static`, and `.as_str()` borrows it with that lifetime. [VERIFIED: stdlib OnceLock docs — `get_or_init` returns `&T` with the lifetime of `&self`; since `self` is `&'static OnceLock`, the returned `&String` is `&'static String`, and `.as_str()` is `&'static str`].
- The macro parses a single `LitStr` argument: `parse_macro_input!(input as LitStr)`.
- Extension extraction for MIME lookup: `Path::new(path_str).extension().and_then(|e| e.to_str()).unwrap_or("")`.
- Name sanitization (D-04): replace `/` and `.` and `-` with `_`, producing `assets_app_js` from `assets/app.js`. Must produce a valid URL segment (alphanumeric + `_`).

### Pattern 2: `crate::utils::ferro()` root-path resolution (D-06 dependency)

**What:** The `ferro()` helper in `ferro-macros/src/utils.rs` emits `::ferro`, which resolves to `ferro-rs` (the `framework` crate) in downstream consumers and to `crate` (via `extern crate self as ferro`) in the workspace tests.

**Exact source:** `ferro-macros/src/utils.rs` line 58: `pub(crate) fn ferro() -> TokenStream2 { quote!(::ferro) }`

**Impact on D-06 (CRITICAL BLOCKER — see below):** The `ferro()` helper emits `::ferro`, which means the macro expansion emits `::ferro::bundle::Bundle`. For this to resolve, `framework/src/lib.rs` must have `pub use ferro_bundle as bundle;` (or a `pub mod bundle { pub use ferro_bundle::*; }`). But `ferro-bundle/Cargo.toml` line 18 declares `ferro-rs = { path = "../framework", version = "0.2" }` as a dependency. Adding `ferro-bundle` to `framework`'s deps would create a direct circular dependency.

### CRITICAL ISSUE: D-06 Circular Dependency

**The problem:** [VERIFIED: ferro-bundle/Cargo.toml line 18, framework/Cargo.toml, publish.yml Wave 2/3]

```
framework (ferro-rs, Wave 2) ← depends on ← ferro-bundle (Wave 3)
ferro-bundle                 → depends on → framework (ferro-rs)
```

Adding `ferro-bundle` to `framework/Cargo.toml` [dependencies] creates a cycle. Cargo rejects this. This is a hard blocker on D-06 as literally specified.

**Resolution options (planner must choose one):**

**Option A (RECOMMENDED): Break ferro-bundle's dependency on ferro-rs.**
`ferro-bundle` uses only `ferro_rs::{HttpResponse, Request}` in its `serve()` method. Extract these to use raw `http` types, or make the `serve()` method take a simpler owned input (path + header), removing the `ferro-rs` dep from `ferro-bundle`. This moves `ferro-bundle` to Wave 1A (leaf crate), unblocking the `framework` re-export.
- Change scope: `ferro-bundle/src/lib.rs` `serve()` signature + `ferro-bundle/Cargo.toml`; `publish.yml` wave reassignment.
- Precedent: `ferro-assets/Cargo.toml` does NOT depend on `ferro-rs` — it is self-contained.

**Option B: Have the macro emit `::ferro_bundle::Bundle` directly.**
Instead of routing through `::ferro::bundle::Bundle`, the macro expansion emits the crate-direct path. This works if the consumer app adds `ferro-bundle` as a direct dep (alongside `ferro`). No framework change needed. The `ferro()` helper becomes optional for this macro.
- Downside: consumers must add `ferro-bundle` explicitly to their `Cargo.toml`, or `framework` re-exports it indirectly through a `mod bundle { pub use ferro_bundle::*; }` block that requires `ferro-bundle` in `framework`'s deps — which circles back to Option A.

**Option C: Add `ferro-bundle` as an optional feature-gated dep to framework.**
Even with a feature flag, Cargo's cycle detection operates on the full dep graph regardless of feature activation. This does NOT resolve the cycle.

**Summary:** Option A is the correct fix. The planner should scope a Wave 0 task: **decouple `ferro-bundle` from `ferro-rs`** by replacing the `serve()` method's `Request` parameter type with a simpler owned `(path: String, if_none_match: Option<String>)` tuple, delegating `Request`-aware dispatch to a thin `framework` adapter. Or remove the `serve()` method from `ferro-bundle` entirely and move it to `framework` as a handler. This also moves `ferro-bundle` from Wave 3 to Wave 1A in `publish.yml`.

### Pattern 3: `mime_from_ext` helper in ferro-bundle (D-05)

**Exact inverse of `ext_from_content_type`** [VERIFIED: ferro-bundle/src/lib.rs lines 89–106]:

```rust
// Source: ferro-bundle/src/lib.rs — inverts ext_from_content_type
pub fn mime_from_ext(ext: &str) -> &'static str {
    match ext {
        "js" | "mjs"    => "application/javascript",
        "css"           => "text/css",
        "html" | "htm"  => "text/html",
        "txt"           => "text/plain",
        "json"          => "application/json",
        "png"           => "image/png",
        "jpg" | "jpeg"  => "image/jpeg",
        "svg"           => "image/svg+xml",
        "gif"           => "image/gif",
        "webp"          => "image/webp",
        "woff2"         => "font/woff2",
        "woff"          => "font/woff",
        "wasm"          => "application/wasm",
        _               => "application/octet-stream",
    }
}
```

The existing `ext_from_content_type` in `lib.rs` already lists exactly these 13 MIME types. The inverse table is one-to-one except for `"application/javascript" | "text/javascript"` (both map to `"js"`). The inverse direction `"js"` → `"application/javascript"` is unambiguous. [VERIFIED: ferro-bundle/src/lib.rs line 91]

### Pattern 4: clap subcommand group for `ferro assets fetch` (D-07)

**Exact pattern from ferro-cli/src/main.rs** [VERIFIED: main.rs lines 14–543]:

```rust
// In Commands enum (main.rs):
/// Manage project assets
Assets {
    #[command(subcommand)]
    subcommand: AssetsCommand,
},

// In ferro-cli/src/commands/assets.rs (new file):
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AssetsCommand {
    /// Download Iconify icon sets or individual icons into the asset directory
    Fetch {
        #[command(subcommand)]
        source: FetchSource,
    },
}

#[derive(Subcommand)]
pub enum FetchSource {
    /// Fetch an Iconify set (e.g. heroicons) or specific icons (e.g. heroicons/check)
    Iconify {
        /// Icon set prefix, optionally with icon name(s): "heroicons" or "heroicons/check"
        set: String,
        /// Output directory (default: assets/)
        #[arg(long, default_value = "assets")]
        out: String,
    },
    /// Fetch a Fontsource font family
    Fontsource {
        /// Font family id (e.g. inter, open-sans)
        family: String,
        /// Comma-separated weights to fetch (default: 400)
        #[arg(long, default_value = "400", value_delimiter = ',')]
        weights: Vec<u32>,
        /// Comma-separated subsets to fetch (default: latin)
        #[arg(long, default_value = "latin", value_delimiter = ',')]
        subsets: Vec<String>,
        /// Output directory (default: assets/)
        #[arg(long, default_value = "assets")]
        out: String,
    },
}
```

**Wire-up in main.rs:** Add `Commands::Assets { subcommand }` match arm calling `commands::assets::run(subcommand)`. Add `pub mod assets;` to `commands/mod.rs`. [VERIFIED: commands/mod.rs pattern, main.rs match arm pattern]

### Pattern 5: Iconify fetch implementation (D-07, D-08, D-09)

**API endpoints** [VERIFIED via web search and Iconify docs]:

| Operation | URL |
|-----------|-----|
| Single SVG icon | `https://api.iconify.design/{prefix}/{icon}.svg` |
| Multiple icons JSON | `https://api.iconify.design/{prefix}.json?icons={icon1},{icon2}` |
| Full set JSON | `https://api.iconify.design/{prefix}.json` |

**Full set JSON response structure** (IconifyJSON format):
```json
{
  "prefix": "heroicons",
  "icons": {
    "check": { "body": "<path d=\"...\"/>", "width": 24, "height": 24 },
    "x": { "body": "<path d=\"...\"/>", "width": 24, "height": 24 }
  },
  "width": 24,
  "height": 24
}
```

**Recommended fetch strategy for `ferro assets fetch iconify <set>`:**
- If `<set>` is just a prefix (e.g. `heroicons`): fetch the full set JSON from `https://api.iconify.design/{set}.json`, parse icon bodies, write each as `assets/{set}/{name}.svg` with `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">{body}</svg>`.
- If `<set>` is `prefix/icon` (e.g. `heroicons/check`): fetch individual SVG from `https://api.iconify.design/{prefix}/{icon}.svg`, write to `assets/{prefix}/{icon}.svg`.
- Content-type for output: `image/svg+xml` — iron `mime_from_ext("svg")`.

**Implementation using reqwest blocking** [VERIFIED: ferro-cli/Cargo.toml line 48]:
```rust
let client = reqwest::blocking::Client::new();
let response = client.get(&url).send()?.error_for_status()?;
```

### Pattern 6: Fontsource fetch implementation (D-07, D-08, D-09)

**API metadata endpoint** [VERIFIED: live fetch from api.fontsource.org]:
`https://api.fontsource.org/v1/fonts/{family-id}`

Response structure:
```json
{
  "id": "inter",
  "family": "Inter",
  "weights": [100, 200, 300, 400, 500, 600, 700, 800, 900],
  "styles": ["italic", "normal"],
  "subsets": ["cyrillic", "greek", "latin", "latin-ext", "vietnamese"],
  "variable": true,
  "variants": {
    "400": {
      "normal": {
        "latin": {
          "url": {
            "woff2": "https://cdn.jsdelivr.net/fontsource/fonts/inter@latest/latin-400-normal.woff2",
            "woff":  "...",
            "ttf":   "..."
          }
        }
      }
    }
  }
}
```

**Default fetch behavior (D-09 "minimal correct default"):** weight=400, subset=latin, style=normal. These cover the most common web use case. Additional weights/subsets via `--weights` and `--subsets` flags.

**Output:** Write `.woff2` files to `assets/{family}/{subset}-{weight}-{style}.woff2`. Optionally generate a `assets/{family}/{family}.css` with `@font-face` declarations using `asset!()` URL paths (but D-09 says fetch only — the CSS generation is author responsibility).

**CSS generation is optional/bonus:** D-09 says "write individual servable files"; generating `@font-face` CSS is additive but not required for SC#3.

### Pattern 7: Bundle name sanitization (D-04)

**Allowed charset:** Bundle names are embedded in the URL path `/bundles/{name}.{sha8}.{ext}`. The name must be URL-safe: `[a-zA-Z0-9_-]` is safe. From the test at ferro-bundle/tests: `"serve-cold-sdk"` is used as a name, so hyphens are allowed. [VERIFIED: ferro-bundle/src/lib.rs `hashed_url_for` + test]

**Recommended sanitization function:**
```rust
fn sanitize_bundle_name(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect()
    // "assets/app.js" → "assets_app_js"
    // "assets/hero-icons/check.svg" → "assets_hero-icons_check_svg"
}
```

### Anti-Patterns to Avoid

- **Reading file bytes in the proc-macro itself:** The proc-macro runs in the build process at a different CWD. `include_bytes!` resolves relative to the source file at compile time; direct `std::fs::read` in the macro would resolve against CWD/`CARGO_MANIFEST_DIR`, which is fragile (D-01 decision).
- **Re-registering the bundle on every call:** `Bundle::new` panics on duplicate name. D-02 exists specifically to prevent this. A `static OnceLock` ensures exactly-once registration.
- **Using `LazyLock` with `include_bytes!` inside the static initializer:** `LazyLock::new(|| { ... include_bytes!(...) ... })` embeds the `include_bytes!` at definition, which runs at static initialization time. This works but is slightly less clear than `OnceLock::get_or_init` at call site. Both are valid; `OnceLock` is more explicit about when registration happens.
- **Returning `String` from `asset!()`:** Causes heap allocation per call. D-03 specifies `&'static str` for zero allocation.
- **Hardcoding app identity in ferro-bundle or ferro-macros:** The project-agnostic rule — no hardcoded `"gestiscilo"`, `"ferro-app"`, or tenant strings. Asset paths are consumer-supplied.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process-global content-hash registry with ETag serving | Custom HashMap + SHA | `ferro-bundle::Bundle::new` | Already handles dedup, SHA-256, URL keying, 304, immutable cache headers |
| Content-type → extension mapping | Manual match table | `ferro-bundle::ext_from_content_type` (already exists) + new `mime_from_ext` (D-05) | Single source of truth; the inverse table is trivially derived |
| Per-callsite `OnceLock` init pattern | Mutex + bool | `std::sync::OnceLock::get_or_init` | Stdlib stable since 1.70; data-race-free, no `unsafe` |
| HTTPS download with pure-Rust TLS | Custom TCP + TLS | `reqwest` blocking + rustls-tls (already in ferro-cli) | No nasm/OpenSSL; already present dep |

**Key insight:** The entire macro body delegates to `Bundle::new` — the macro is only a syntactic ergonomics layer over an existing, tested, production-grade substrate.

---

## Common Pitfalls

### Pitfall 1: D-06 Circular Dependency (BLOCKER)
**What goes wrong:** `framework/Cargo.toml` tries to add `ferro-bundle` as a dep, cargo reports a cycle.
**Why it happens:** `ferro-bundle/Cargo.toml` line 18 depends on `ferro-rs` (framework). Adding the reverse dep creates a cycle.
**How to avoid:** Implement Option A — remove `ferro-bundle`'s dependency on `ferro-rs` by making `Bundle::serve` take raw `(path, if_none_match)` rather than a `ferro_rs::Request`, or move the serve dispatch into framework.
**Warning signs:** If a plan task says "add ferro-bundle to framework/Cargo.toml" without first removing `ferro-rs` from `ferro-bundle/Cargo.toml`, it will fail at compile time.

### Pitfall 2: `static` inside a closure in `get_or_init`
**What goes wrong:** Putting `static __BYTES` inside the `get_or_init` closure and expecting `include_bytes!` to work there.
**Why it happens:** `static` items are allowed inside functions and closures in Rust; `include_bytes!` is resolved at compile time regardless of location. This actually WORKS, but some reviewers expect statics to be at module level.
**How to avoid:** Keep the `static __BYTES` and the `static __URL: OnceLock<String>` both at the use-site block scope — this is idiomatic for proc-macro-generated code and matches the `#[memoize]` expansion pattern.

### Pitfall 3: Duplicate bundle name collision
**What goes wrong:** Two `asset!()` calls with paths that sanitize to the same name (e.g., `assets/app.js` and `assets_app.js` both become `assets_app_js`).
**Why it happens:** The sanitization collapses multiple separators into `_`. 
**How to avoid:** The duplicate-name panic in `Bundle::new` surfaces this at the first request that hits either asset site. It is a developer error by definition (D-04). Document the edge case; do not make the sanitization more complex for the initial implementation.

### Pitfall 4: `include_bytes!` path is evaluated relative to the source file, not CWD
**What goes wrong:** Unexpected "file not found" errors if the path is not relative to the source file containing `asset!()`.
**Why it happens:** `include_bytes!` uses `#[proc_macro]`-relative paths at expansion time, which resolve against the source file location (call-site-source-relative). This is D-01's design intent — it is a feature, not a bug. Document it clearly.
**Warning signs:** Works in the sample app, fails in an integration test if the test file is in a different directory tree than the asset.

### Pitfall 5: reqwest blocking in an async context
**What goes wrong:** Calling `reqwest::blocking::Client::new().get(url).send()` from within a Tokio async task panics ("Cannot start a runtime from within a runtime").
**Why it happens:** The blocking reqwest client spins up its own mini runtime. `ferro assets fetch` is a CLI command, not an async handler — it runs synchronously in `main()`. This is NOT an issue for the fetch command.
**How to avoid:** Confirm `commands::assets::run()` is a synchronous function (all other CLI commands in ferro-cli are sync). [VERIFIED: ferro-cli/src/commands/make_scaffold.rs and others use sync `fn run()`]

### Pitfall 6: publish.yml wave order for ferro-bundle
**What goes wrong:** After decoupling ferro-bundle from ferro-rs (Option A resolution), ferro-bundle moves from Wave 3 to Wave 1A. Forgetting to update `publish.yml` means the publish job tries to publish ferro-bundle before its transitive deps are on crates.io.
**How to avoid:** Include a publish.yml update task in Wave 0. [VERIFIED: publish.yml — ferro-bundle currently in WAVE3_CRATES with ferro-cli]

### Pitfall 7: Iconify JSON body-to-SVG reconstruction
**What goes wrong:** The body field in Iconify JSON is the inner SVG content only (no `<svg>` wrapper). A naive write produces invalid SVG.
**How to avoid:** Wrap: `format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\">{}</svg>", w, h, body)`.

---

## Code Examples

### `asset!()` macro implementation skeleton

```rust
// Source: ferro-macros/src/asset.rs (new file, modeled after memoize.rs)
use proc_macro::TokenStream;
use quote::quote;
use std::path::Path;
use syn::{parse_macro_input, LitStr};
use crate::utils::ferro;

pub fn asset_impl(input: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(input as LitStr);
    let path_str = path_lit.value();
    let ferro = ferro();

    // Extract extension for MIME lookup
    let ext = Path::new(&path_str)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    // Sanitize path → bundle name: replace non-alphanumeric (except '-') with '_'
    let bundle_name: String = path_str
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect();

    let ext_str = ext.to_string();
    let bundle_name_str = bundle_name;

    let output = quote! {
        {
            static __FERRO_ASSET_URL: ::std::sync::OnceLock<::std::string::String>
                = ::std::sync::OnceLock::new();
            __FERRO_ASSET_URL.get_or_init(|| {
                static __FERRO_ASSET_BYTES: &[u8] = include_bytes!(#path_lit);
                #ferro::bundle::Bundle::new(#bundle_name_str, __FERRO_ASSET_BYTES)
                    .content_type(#ferro::bundle::mime_from_ext(#ext_str))
                    .hashed_url()
            }).as_str()
        }
    };

    output.into()
}
```

**Note:** This expansion emits `::ferro::bundle::Bundle` and `::ferro::bundle::mime_from_ext`, which require the D-06 re-export to be in place (and the cycle broken via Option A first).

### `mime_from_ext` function (ferro-bundle/src/lib.rs addition)

```rust
// Source: inverts ferro-bundle/src/lib.rs:89-106 ext_from_content_type
/// Map a file extension to its MIME type string.
///
/// Used by the `asset!()` macro to infer content-type from the path extension.
/// Unknown extensions return `"application/octet-stream"`, preserving
/// byte-identical passthrough for unrecognized file types (SC #2).
pub fn mime_from_ext(ext: &str) -> &'static str {
    match ext {
        "js" | "mjs"   => "application/javascript",
        "css"          => "text/css",
        "html" | "htm" => "text/html",
        "txt"          => "text/plain",
        "json"         => "application/json",
        "png"          => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg"          => "image/svg+xml",
        "gif"          => "image/gif",
        "webp"         => "image/webp",
        "woff2"        => "font/woff2",
        "woff"         => "font/woff",
        "wasm"         => "application/wasm",
        _              => "application/octet-stream",
    }
}
```

### Fontsource fetch (reqwest blocking pattern)

```rust
// Source: ferro-cli/Cargo.toml line 48 — reqwest = { version = "0.12", features = ["blocking","json","rustls-tls"] }
fn fetch_fontsource(family: &str, weights: &[u32], subsets: &[&str], out_dir: &Path) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();
    let meta_url = format!("https://api.fontsource.org/v1/fonts/{family}");
    let meta: serde_json::Value = client.get(&meta_url).send()?.json()?;

    let variants = meta["variants"].as_object().ok_or_else(|| anyhow::anyhow!("no variants"))?;
    for weight in weights {
        let w_key = weight.to_string();
        if let Some(styles) = variants.get(&w_key).and_then(|v| v.as_object()) {
            if let Some(normal) = styles.get("normal").and_then(|v| v.as_object()) {
                for subset in subsets {
                    if let Some(urls) = normal.get(*subset)
                        .and_then(|v| v.get("url"))
                        .and_then(|v| v.as_object())
                    {
                        if let Some(woff2_url) = urls.get("woff2").and_then(|v| v.as_str()) {
                            let bytes = client.get(woff2_url).send()?.bytes()?;
                            let filename = format!("{subset}-{weight}-normal.woff2");
                            let dest = out_dir.join(family).join(&filename);
                            std::fs::create_dir_all(dest.parent().unwrap())?;
                            std::fs::write(&dest, &bytes)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
```

### trybuild pass fixture for `asset!()` (new test file)

```rust
// tests/ui/asset/pass/minimal.rs — modeled after ferro-macros/tests/ui/action/pass/minimal.rs
#![allow(unused_imports)]
extern crate ferro_rs as ferro;

fn main() {
    // asset!() returns &'static str
    let _url: &'static str = ferro::asset!("tests/ui/asset/pass/fixture.js");
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Boot-time `Bundle::new(name, bytes).content_type(ct)` in `main.rs` | `asset!("path")` at use site, lazy-registered | Phase 261 | Author writes one line instead of three; macro generates the OnceLock boilerplate |
| `OnceLock` as stdlib feature | `once_cell::sync::OnceCell` | Rust 1.70 (2023-06) | stdlib OnceLock is preferred; `once_cell` not needed |
| `LazyLock` | Also available since Rust 1.80 | 2024-08 | Usable but `OnceLock` is more explicit when init depends on runtime data |

**Workspace Rust edition:** `2021` (from Cargo.toml). Workspace `rust-version = "1.88.0"` [VERIFIED: Cargo.toml workspace.package]. `OnceLock` (1.70) and `LazyLock` (1.80) are both below this floor — both are safely available.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Iconify API endpoint `https://api.iconify.design/{prefix}.json` returns IconifyJSON with `icons` object containing per-icon `body`, `width`, `height` | Pattern 5 | Planner would need to adjust fetch impl; the endpoint shape is described in multiple web sources but not verified by a live call in this session |
| A2 | Individual SVG endpoint `https://api.iconify.design/{prefix}/{icon}.svg` returns a complete SVG document (no body-only extraction needed) | Pattern 5 | If it returns icon body only, the SVG wrapping is still needed |
| A3 | `ferro assets fetch fontsource <family>` default of weight=400 + subset=latin + style=normal is sufficient for typical web usage | Pattern 6 | If a consumer needs a variable font or non-latin subset, they add `--weights`/`--subsets` flags |

---

## Open Questions (RESOLVED)

1. **D-06 resolution path**
   - What we know: A cycle exists (`ferro-bundle` → `ferro-rs` → ??? → `ferro-bundle`). Two viable resolutions identified (Option A: break ferro-bundle's dep on ferro-rs; Option B: emit `::ferro_bundle::Bundle` directly from macro).
   - What's unclear: Does anything in the current codebase call `Bundle::serve(req: ferro_rs::Request)` and require that exact signature? [VERIFIED: no caller found in `app/src/` — `Bundle::serve` appears unused in the sample app's source].
   - Recommendation: Implement Option A. Remove `ferro-rs` from `ferro-bundle/Cargo.toml`. Change `Bundle::serve` to accept `(path: &str, if_none_match: Option<&str>)` directly (the private `serve_inner` already has this signature). Expose `serve_inner` as a public `Bundle::serve_path` or make `serve` take path + header directly. Move the Request-aware dispatch to a `framework` adapter function.

2. **`asset!()` fixture files for trybuild tests**
   - What we know: trybuild `pass/*.rs` fixtures compile against `ferro-rs` as `ferro`. For `asset!()`, each pass fixture needs a real file on disk at the specified path.
   - What's unclear: Best practice for fixture asset files in the test directory.
   - Recommendation: Create `ferro-macros/tests/ui/asset/pass/fixture.js` (minimal, e.g., `console.log(1);`) as a real committed file that the pass test's `asset!()` call embeds.

3. **`OnceLock` vs `LazyLock` for the static URL cache**
   - What we know: Both are available at rust-version 1.88.0. `OnceLock` requires `get_or_init` call. `LazyLock` evaluates at first deref.
   - Recommendation: Use `OnceLock` to match the existing ferro-bundle registry pattern (process-global `OnceLock<DashMap>`) and to keep the initialization explicit inside `get_or_init`.

---

## Environment Availability

Step 2.6: SKIPPED (no new external tooling required — `reqwest` and `reqwest`'s rustls-tls dep are already in the workspace tree; no system libraries needed).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust built-in) + `trybuild 1.x` (proc-macro UI tests) |
| Config file | `ferro-macros/Cargo.toml` [dev-dependencies] includes `trybuild = "1"` |
| Quick run command | `cargo test -p ferro-bundle -p ferro-macros -- --test-threads=1` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | `asset!()` returns content-hashed URL; URL is stable for unchanged bytes | unit | `cargo test -p ferro-bundle test_mime_from_ext` + `cargo test -p ferro-macros` | ❌ Wave 0 |
| SC-1 | `OnceLock` ensures single registration per call site (no duplicate-name panic) | unit | `cargo test -p ferro-bundle` (existing `duplicate_name_panics` test validates the guard) | ✅ exists |
| SC-1 | Hash determinism across separate calls with same bytes | unit | `cargo test -p ferro-bundle hash_is_deterministic` | ✅ exists |
| SC-2 | Known extension (.js, .css, .svg, .woff2, etc.) infers correct MIME | unit | `cargo test -p ferro-bundle test_mime_from_ext` | ❌ Wave 0 |
| SC-2 | Unknown extension → `application/octet-stream`, bytes pass through unchanged | unit | `cargo test -p ferro-bundle mime_from_ext_unknown_is_octet_stream` | ❌ Wave 0 |
| SC-3 | `ferro assets fetch iconify <set>` downloads `.svg` files to `assets/` | integration (tempdir) | `cargo test -p ferro-cli assets_fetch_iconify` | ❌ Wave 0 |
| SC-3 | `ferro assets fetch fontsource <family>` downloads `.woff2` to `assets/` | integration (tempdir) | `cargo test -p ferro-cli assets_fetch_fontsource` | ❌ Wave 0 |
| SC-3 | Fetch runs on Rust toolchain alone (no nasm/node/OpenSSL) | structural | `cargo build -p ferro-cli` on CI (already verifies no native tooling via rustls-tls) | ✅ CI enforces |
| LIVE-03 | `asset!()` compiles and returns `&'static str` (trybuild pass) | trybuild | `cargo test -p ferro-macros --test asset_macro` | ❌ Wave 0 |
| LIVE-03 | `asset!()` applied to sync fn emits compile error (future: N/A — no such constraint) | — | — | N/A |

### Wave 0 Gaps

- [ ] `ferro-bundle/src/lib.rs` — add `mime_from_ext` function + unit test `test_mime_from_ext` covering all 13 known extensions + unknown passthrough
- [ ] `ferro-macros/src/asset.rs` — new file with `asset_impl()`
- [ ] `ferro-macros/tests/asset_macro.rs` — trybuild harness (`t.pass("tests/ui/asset/pass/*.rs")`)
- [ ] `ferro-macros/tests/ui/asset/pass/minimal.rs` — minimal compile-pass fixture
- [ ] `ferro-macros/tests/ui/asset/pass/fixture.js` — real asset file the fixture embeds
- [ ] `ferro-cli/src/commands/assets.rs` — new fetch command module
- [ ] `ferro-cli/src/commands/assets_fetch_integration_test` (or tempdir-based test in assets.rs under `#[cfg(test)]`)
- [ ] Resolve D-06 cycle: update `ferro-bundle/Cargo.toml` to remove `ferro-rs` dep + update `Bundle::serve` signature + update `publish.yml` wave

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-bundle -p ferro-macros -p ferro-cli -- --test-threads=1`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

---

## Project Constraints (from CLAUDE.md)

- `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` before every commit.
- No co-author attribution in commits.
- No new crates (confirmed: `ferro-macros`, `ferro-bundle`, `ferro-cli` are the three touch points).
- `publish = false` for new crates is not relevant here (no new crates).
- `ferro-*` crates must be project-agnostic: no hardcoded app identity in `ferro-bundle` or `ferro-macros`.
- Codec asm/nasm gotcha: `ferro-assets` already handles this with `ravif = { default-features = false, features = ["threading"] }`. `ferro-bundle` has no codec deps.
- **New crate in publish.yml rule:** No new crate is introduced. However, if `ferro-bundle` moves from Wave 3 to Wave 1A (Option A resolution), the `publish.yml` WAVE1A_CRATES and WAVE3_CRATES lines must be updated.
- No publish in Phase 261 — single publish at Phase 262.

---

## Sources

### Primary (HIGH confidence — code-verified in this session)

- `ferro-bundle/src/lib.rs` — `Bundle::new`, `ext_from_content_type`, `OnceLock` registry, test helpers, `__test_internals`
- `ferro-bundle/Cargo.toml` — `ferro-rs` dep (cycle blocker), `sha2`, `dashmap`, `hex`
- `ferro-macros/src/memoize.rs` — direct precedent: `OnceLock`-style expansion, `crate::utils::ferro()` usage
- `ferro-macros/src/utils.rs` — `ferro()` function returning `::ferro` TokenStream
- `ferro-macros/src/lib.rs` — macro registration pattern, `#[proc_macro]` entry points
- `ferro-macros/Cargo.toml` — `trybuild = "1"` in dev-dependencies
- `ferro-cli/src/main.rs` — clap `Commands` enum, match arm patterns
- `ferro-cli/src/commands/mod.rs` — `pub mod` list pattern
- `ferro-cli/Cargo.toml` — `reqwest = { version = "0.12", features = ["blocking","json","rustls-tls"] }` confirmed
- `framework/Cargo.toml` — `ferro-bundle` NOT present (confirms cycle if added)
- `.github/workflows/publish.yml` — Wave 2 = ferro-rs, Wave 3 = ferro-cli + ferro-bundle
- `ferro-assets/src/asset.rs` — `infer_content_type` returns `ContentType` enum (confirms D-05: different surface, do not merge)

### Secondary (MEDIUM confidence — web-verified)

- Fontsource API `https://api.fontsource.org/v1/fonts/{family}` response structure — verified via live fetch (open-sans and inter)
- Fontsource CDN URL pattern `https://cdn.jsdelivr.net/fontsource/fonts/{id}@{version}/{subset}-{weight}-{style}.woff2` — verified via fontsource.org docs
- Iconify individual SVG endpoint `https://api.iconify.design/{prefix}/{icon}.svg` — confirmed via Iconify docs
- Iconify full set JSON endpoint `https://api.iconify.design/{prefix}.json` — confirmed via web search

### Tertiary (LOW confidence — inferred/assumed)

- Iconify full-set JSON `body` field is inner SVG content requiring `<svg>` wrapper [A1]
- Individual SVG endpoint returns a complete SVG document [A2]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all packages verified from Cargo.toml files
- Architecture: HIGH — all integration points verified from source; cycle issue is code-verified
- Pitfalls: HIGH — cycle blocker verified from source; others verified from existing tests
- CLI API endpoints: MEDIUM — Fontsource confirmed live; Iconify confirmed via docs but no live call made

**Research date:** 2026-07-26
**Valid until:** 2026-09-26 (stable substrate; Iconify/Fontsource APIs rarely change)
