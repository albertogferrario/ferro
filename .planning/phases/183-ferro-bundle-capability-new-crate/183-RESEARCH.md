# Phase 183: `ferro-bundle` capability (new crate) — Research

**Researched:** 2026-06-06
**Domain:** Workspace new-crate scaffolding + HTTP byte-blob serving with content-hashed URLs
**Confidence:** HIGH — every source-of-truth claim verified by direct file read

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01: Bundle storage type — `&'static [u8]` only.** The roadmap's locked API signature is `Bundle::new(name: &str, bytes: &'static [u8])`. Runtime-loaded bytes (`Cow<'static, [u8]>`, `Arc<[u8]>`) are deferred.
- **D-02: Registry — process-global `OnceLock<DashMap<String, BundleEntry>>` keyed by URL path; separate `OnceLock<DashMap<String, String>>` for alias → hashed-URL.** The locked `Bundle::serve(req)` signature (no `&self`, no registry parameter) forces a process-global.
- **D-03: URL routing — `Bundle::serve(req)` dispatches via the registry.** Alias check first (301), then bundle check (200/304), then 404. Consumer wires `Bundle::serve` as handler for `/bundles/{filename}` and each alias path. ferro-bundle does NOT own routing.
- **D-04: Hash truncation — first 8 hex chars of SHA-256.** URL format: `/bundles/{name}.{sha8}.{ext}`. 32 bits of entropy. Documented in README.
- **D-05: ETag format — strong, full SHA-256 hex, quoted.** `ETag: "{64-hex-chars}"`. Per RFC 7232 §2.3. `If-None-Match` comparison is exact string match.
- **D-06: Bundle registration — eager at `Bundle::new()` call site; panic on duplicate name.** Identical re-registration (same name, same bytes) is also developer error — panic uniformly.
- **D-07: Hash algorithm — SHA-256 (locked by roadmap).** Crate dep: `sha2`. Hex encoding: `hex` crate.
- **D-08: Alias mechanism — stored on the Bundle, queried by `Bundle::serve`, 301 redirect.** Multiple aliases per bundle allowed.
- **D-09: Crate dependencies — minimal.** `sha2`, `hex`, `dashmap`, `framework` (published as `ferro-rs`). **Wave reassignment:** see Risks §1 below — `framework` is published in **Wave 2** (not Wave 1A), so ferro-bundle goes in a NEW publish wave after Wave 2 (call it Wave 2.5 or Wave 3.5), NOT in `WAVE1B_CRATES`.
- **D-10: README required — documents bundle-vs-filesystem split.** "Do not fold these" wording is load-bearing.
- **D-11: Workspace + publish.yml integration.** `Cargo.toml` `workspace.members` grows by one. Publish wave assignment revised — see D-09 note above.
- **D-12: First publish bootstrap — manual `cargo publish -p ferro-bundle` from local terminal.** CI token has `publish-update` only.
- **D-13: Test isolation via `#[cfg(test)] reset()` helper.** `pub(crate) fn reset()` visible only in `#[cfg(test)]` clears both registries.

### Claude's Discretion

- Exact crate metadata fields (`keywords`, `categories`, `description`) — follow sibling-crate template. Closest analog: `ferro-storage/Cargo.toml` (leaf-ish addon-crate shape).
- File layout under `ferro-bundle/src/` — single `lib.rs` acceptable; split into `bundle.rs` + `registry.rs` + `serve.rs` if planner judges readability gain.
- Specific error type shape — `thiserror`-derived `Error` enum, name-prefixed `Display` per sibling convention (`ferro-wallet/src/error.rs` is closest analog).
- Exact integration-test layout under `ferro-bundle/tests/` — follow `ferro-wallet/tests/` pattern (one file per integration scenario).

### Deferred Ideas (OUT OF SCOPE)

- **Runtime-mutable bundle bytes.** `Cow<'static, [u8]>` / `Arc<[u8]>` deferred. Compile-time `include_bytes!` is the v1 idiom.
- **Pre-deflated `Accept-Encoding` variants** (gzip/br).
- **Composite bundles / manifests.**
- **Streaming serve for large bundles.** Bytes-in-memory by design.
- **Content-type sniffing.** Caller provides at registration per locked SC-4.
- **Re-export `Bundle` through `framework`.** Future phase; not required to ship 183.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

Phase 183 is not enumerated in `.planning/REQUIREMENTS.md` (that document scopes the v12.1 AI milestone, REQ-IDs `AISDK-*` / `AISSE-*` / `AICLI-*`). Phase 183's requirements are derived from the roadmap's 6 success criteria, restated below with synthetic REQ-IDs for traceability:

| ID | Description (verbatim from ROADMAP.md §1982-1987) | Research Support |
|----|---|---|
| BUNDLE-01 | `Bundle::new("embed-v1", BYTES).content_type("application/javascript").hashed_url()` returns `/bundles/embed-v1.{8hex}.js` deterministically derived from SHA-256 of `BYTES`. | SHA-256 via `sha2::Sha256::digest` (workspace version `0.10`); first 8 chars of `hex::encode(digest)` (`hex 0.4`); content-type→ext mapping table in lib.rs. Determinism is a property of SHA-256. |
| BUNDLE-02 | `Bundle::serve(req)` returns 200 + `Cache-Control: public, max-age=31536000, immutable` + `ETag` on cold; 304 on `If-None-Match` exact match. | `HttpResponse::bytes(...)` + `.header("Content-Type", ct)` + `.header("Cache-Control", "...")` + `.header("ETag", "\"…\"")` (response.rs:48-54, 121-126). 304 = `HttpResponse::new().status(304)` (response.rs:18-24, 93-97). Header comparison via `req.header("if-none-match")` (request.rs:309-311). |
| BUNDLE-03 | `.with_alias("/embed/v1.js")` registers a plain-URL alias returning 301 to the current hashed URL. | `Redirect::to(hashed).permanent().into()` (response.rs:212-217, 257-260, 278-284) → emits `HttpResponse` with status 301 and `Location` header. Alternative: `HttpResponse::new().status(301).header("Location", hashed_url)` for a one-call inline path. |
| BUNDLE-04 | Content-type caller-provided at registration; default `application/octet-stream` if unspecified. | `BundleEntry.content_type: Option<String>` default `None` → serves `application/octet-stream` and URL has no extension. |
| BUNDLE-05 | Crate README documents bundle-vs-filesystem split. | Sibling reference: `framework/src/static_files.rs:55-59` shows the filesystem path's `bust_asset_urls` cache-control logic (`/assets/*` gets `max-age=31536000, immutable`; everything else gets `max-age=0, must-revalidate`). README contrasts both. |
| BUNDLE-06 | `ferro-bundle` publishes to crates.io via existing GH Actions workflow. | `.github/workflows/publish.yml` Wave 2.5 (new wave — see Risks §1) post-Wave-2 to allow framework's `ferro-rs` to be published first. First-version bootstrap via D-12 manual publish. |
</phase_requirements>

---

## Summary

Phase 183 adds a new top-level workspace crate `ferro-bundle/` (Cargo.toml + README.md + src/lib.rs [+ optional split into bundle.rs/registry.rs/serve.rs] + tests/), registers it in `Cargo.toml` `workspace.members`, adds it to a NEW publish wave in `.github/workflows/publish.yml` (the existing Wave 1B is wrong for this crate — see Risks §1), and bumps `workspace.package.version` so the next merge to master triggers a release. The crate ships a single public type `Bundle` with a 5-method API surface and a process-global registry for `Bundle::serve(req)` lookup.

The implementation is shaped by three sources of pattern compliance:

1. **Crate scaffolding pattern** — `ferro-storage/Cargo.toml` (leaf-ish addon) and `ferro-wallet/Cargo.toml` (newer shape with `homepage`, `workspace.version = true` inheritance) define the manifest layout. `ferro-wallet/src/error.rs` defines the `thiserror`-derived single-Error-enum-per-crate convention with name-prefixed `Display` strings.
2. **HTTP types pattern** — `framework/src/http/response.rs:48-54` is the `HttpResponse::bytes(impl Into<Bytes>) -> Self` constructor (verified: sets no default headers — caller must add Content-Type via `.header()`). `framework/src/http/response.rs:121-126` is the case-insensitive replace-semantics `.header()` builder. `framework/src/http/request.rs:121-123` is `req.path()`. `framework/src/http/request.rs:309-311` is `req.header(name) -> Option<&str>`. 304 = `HttpResponse::new().status(304)`. 301 via `Redirect::to(url).permanent()` (response.rs:204-260) is the idiomatic builder, but inline `HttpResponse::new().status(301).header("Location", url)` is acceptable for a single-call serve path.
3. **Process-global registry pattern** — `ferro-json-ui/src/plugin.rs:14, 147` shows the exact `static GLOBAL: OnceLock<RwLock<…>> = OnceLock::new()` + `fn global() -> &'static …` accessor pattern. D-02 swaps `RwLock<PluginRegistry>` for `DashMap<String, BundleEntry>` (concurrent reads cheaper for the hot serve path) but the surrounding `OnceLock` + `get_or_init` shape is identical.

**Primary recommendation:** Single `src/lib.rs` (~300 lines). Public surface = `Bundle` struct + `Error` enum. Internals = two module-private `static OnceLock<DashMap<…>>` registries + private `BundleEntry` struct + content-type→extension table. Integration tests under `ferro-bundle/tests/` (one file per scenario: hash determinism, 304 fast-path, 301 alias). Unit tests inline under `#[cfg(test)] mod tests` in `lib.rs`. Workspace version bump triggers publish; first publish bootstrapped manually from local terminal per D-12.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Byte storage + SHA-256 hashing | ferro-bundle (new crate, Rust library) | — | Pure compute; no I/O. SHA-256 via `sha2::Sha256::digest`. |
| Global registry of registered bundles | ferro-bundle (process-static OnceLock) | — | The locked `Bundle::serve(req)` signature forbids a per-app registry. |
| URL dispatch (`/bundles/…` → `Bundle::serve`) | Consumer app routing (NOT ferro-bundle) | framework router | ferro-bundle exposes a handler; the consumer wires it under `/bundles/{filename}` and any alias paths via standard ferro route macros. |
| Cache-Control + ETag + 304 fast-path | ferro-bundle (response shaping) | framework `HttpResponse` builder | All HTTP shaping uses `framework::HttpResponse` methods — no direct hyper. |
| Persisting bytes across deploys | None — bytes are compile-time `include_bytes!` | — | The `&'static [u8]` API guarantees no I/O at registration. |
| Compression / `Accept-Encoding` selection | Deferred (downstream CDN) | — | Phase 183 ships identity bytes only. |
| Publish to crates.io | CI (GH Actions) | Manual bootstrap (first version per D-12) | After bootstrap, CI Wave 2.5 publishes patch updates. |

---

## Source-of-Truth File Map

| Path | Line ranges (key) | Purpose for this phase |
|------|-------------------|------------------------|
| `framework/src/http/response.rs` | 1-15 (type def + `Response` alias), 18-24 (`new`), 27-43 (`text`/`json`), 45-54 (**`bytes` — primary constructor for binary blobs**), 87-97 (`status`), 117-136 (**`header`/`append_header` — replace vs append semantics**), 167-176 (`into_hyper`), 203-284 (**`Redirect` + `permanent()` + `From<Redirect> for Response`**) | All HTTP response shaping for `Bundle::serve`. Planner uses verbatim. |
| `framework/src/http/request.rs` | 11-26 (struct def), 105-123 (`method`/`uri`/`headers`/`path`), 309-316 (**`header(name) -> Option<&str>`**) | Request-side accessors for `If-None-Match` lookup and path dispatch. |
| `framework/src/http/mod.rs` | (full file — short) | Confirms `HttpResponse` and `Request` are re-exported at `framework::http::*`. Consumer-side import path. |
| `framework/src/lib.rs` | 76 (`pub use error::…`), 105-114 (**`pub use http::{HttpResponse, …, Request, …}`** at crate root) | Confirms ferro-bundle's dep on `framework` (= crate name `ferro-rs`) exposes `HttpResponse` and `Request` at the crate root: `use ferro_rs::{HttpResponse, Request};`. |
| `framework/src/static_files.rs` | 5-69 (`try_serve_from_dir` — existing filesystem handler), 52-59 (**`Cache-Control` selection on `/assets/*` vs everything else**) | Reference for the filesystem-vs-bundle split documented in ferro-bundle README. Phase 183 does NOT modify this file. |
| `framework/Cargo.toml` | 1-2 (`name = "ferro-rs"`), 30-77 (deps incl. `sha2 = "0.10"` line 71, `dashmap = "6"` line 63) | Confirms (a) crate name on crates.io is `ferro-rs`, (b) workspace already pins `sha2 = "0.10"` and `dashmap = "6"` — ferro-bundle matches both. |
| `ferro-wallet/Cargo.toml` | 1-25 (full file) | Newest crate-manifest template — `version.workspace = true` + `homepage` + `categories = ["web-programming"]`. Closest match for ferro-bundle's shape. |
| `ferro-storage/Cargo.toml` | 1-34 (full file) | Earlier sibling — leaf-ish addon with `dashmap = "6"` (line 21), `bytes = "1"` (line 19), `thiserror = "1.0"` (line 16). Confirms dashmap + bytes versions. |
| `ferro-ai/Cargo.toml` | 1-26 (full file) | Wave 1B reference — internal ferro-* dep `ferro-events = { path = "../ferro-events", version = "0.2" }` (line 20). Note: ferro-bundle's `framework` dep has a DIFFERENT shape because the crate name `ferro-rs` ≠ directory name `framework` — see Code Examples §1. |
| `ferro-macros/Cargo.toml` | 25 (`ferro-rs = { path = "../framework" }`) | Only existing in-repo example of depending on the framework crate. Shows the `ferro-rs` ↔ `framework/` path mapping. Note: ferro-macros is a `dev-dependencies` consumer; ferro-bundle is a `dependencies` consumer and must include `version = "0.2"` for publish-time resolution. |
| `ferro-wallet/src/error.rs` | 1-40 (enum + name-prefixed `Display`), 43-100 (per-variant tests) | Canonical `thiserror` + per-variant tests pattern. ferro-bundle's `Error` enum mirrors. |
| `ferro-wallet/src/lib.rs` | 1-23 (full file — module declarations + re-exports) | Crate-root pattern. ferro-bundle's `lib.rs` is similar but single-file (no submodule split needed for Phase 183). |
| `ferro-json-ui/src/plugin.rs` | 14 (`use std::sync::{OnceLock, RwLock};`), 147 (**`static GLOBAL_PLUGIN_REGISTRY: OnceLock<RwLock<…>> = OnceLock::new();`**), 148-159 (`global_plugin_registry() -> &'static …`) | **The OnceLock global-registry pattern.** ferro-bundle replaces `RwLock<PluginRegistry>` with `DashMap<String, BundleEntry>`; the surrounding shape is identical. |
| `Cargo.toml` (workspace root) | 1-30 (workspace.members), 32-37 (workspace.package) | Append `"ferro-bundle",` to members list at line 29-30. Bump `version = "0.2.42"` (line 33) to `"0.2.43"` to trigger publish. |
| `.github/workflows/publish.yml` | 206-229 (Wave 1a block), 236-262 (**Wave 1b block — note `framework` is NOT in this wave**), 269-290 (**Wave 2 — `ferro-rs` published HERE**), 297-310 (Wave 3 — `ferro-cli`) | **Critical correction:** `framework` (published as `ferro-rs`) lands in Wave 2, NOT Wave 1A as CONTEXT.md §D-09 states. Crates depending on framework cannot publish in Wave 1B. See Risks §1 for the resolution. |
| `.planning/phases/151-ferro-wallet-crate/151-01-scaffold-PLAN.md` | 1-367 (full file) | Reference plan structure for "scaffold new crate" — manifest, README, lib.rs stubs, workspace member registration, publish.yml entry in one atomic plan. ferro-bundle follows the same shape. |
| `.planning/phases/182-ferro-json-ui-data-lazy-hero-runtime-primitive/182-03-PLAN.md` | (full file — short) | Reference for the "single publish at end of phase" workspace-version-bump pattern (per memory `feedback_friction_loop_release_cadence.md`). |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sha2` | `0.10` | SHA-256 hashing of bundle bytes (D-07) | Workspace-pinned at `framework/Cargo.toml:71` and `ferro-stripe/Cargo.toml:29`. Verified via Cargo.lock: `sha2 0.10.9`. |
| `hex` | `0.4` | Hex encoding of SHA-256 output for URL hash + ETag | Workspace convention at `ferro-stripe/Cargo.toml:30` and `ferro-whatsapp/Cargo.toml:19`. Verified via Cargo.lock: `hex 0.4.3`. |
| `dashmap` | `6` | Concurrent map for process-global bundle + alias registries (D-02) | Workspace convention at `framework/Cargo.toml:63`, `ferro-storage/Cargo.toml:21`, `ferro-cache/Cargo.toml:21`, `ferro-ai/Cargo.toml:21`. Verified via Cargo.lock: `dashmap 6.1.0`. |
| `ferro-rs` (= `framework`) | `0.2` (path + version) | Source of `HttpResponse`, `Request`, `Response` type alias | Only in-repo precedent: `ferro-macros/Cargo.toml:25` declares `ferro-rs = { path = "../framework" }` (dev-dep — no version needed). ferro-bundle is a non-dev dep and MUST include `version = "0.2"` for publish-time resolution. **CRITICAL: the workspace directory is `framework/` but the crate name on crates.io is `ferro-rs`.** Dep declaration: `ferro-rs = { path = "../framework", version = "0.2" }`. |
| `thiserror` | `2` | Error enum derive | Newer crates use `thiserror = "2"` (ferro-ai line 18, ferro-wallet line 23). Older crates (`ferro-storage`, `ferro-cache`) still on `1.0`. ferro-bundle should use `2`. |
| `bytes` | `1` | `Bytes` type for `HttpResponse::bytes(impl Into<Bytes>)` constructor and zero-copy body reuse | Workspace convention. Required because `HttpResponse::bytes` takes `impl Into<Bytes>` and we want to convert `&'static [u8]` cheaply. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tracing` | `0.1` | Structured warnings on duplicate registration (before panic) | Optional — if added, follow `ferro-storage/Cargo.toml:15` and `ferro-ai/Cargo.toml:19`. Not strictly required; `panic!` carries the message inline. Planner's discretion. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `sha2 = "0.10"` | `blake3` | BLAKE3 is faster but adds a non-workspace dep; SHA-256 is canonical content-addressing and locked by roadmap. |
| `dashmap = "6"` | `RwLock<HashMap<…>>` | RwLock works (mirrors `ferro-json-ui/plugin.rs` exactly) but the bundle-serve path is read-only on the hot path; DashMap's bucket-level locking gives finer concurrency. Both are valid; planner's discretion. |
| `OnceLock` (std) | `once_cell::Lazy` / `lazy_static!` | `std::sync::OnceLock` is stabilized Rust 1.70+; ferro requires `1.88.0` (Cargo.toml `workspace.package.rust-version`). No need for `once_cell`. |
| `Redirect::to(url).permanent()` builder | Inline `HttpResponse::new().status(301).header("Location", url)` | Builder is idiomatic and tested in framework's response tests. Inline is one less import. Either works for `Bundle::serve`'s 301 path. |

**Installation (planner copies verbatim into ferro-bundle/Cargo.toml):**

```toml
[dependencies]
sha2 = "0.10"
hex = "0.4"
dashmap = "6"
bytes = "1"
thiserror = "2"
ferro-rs = { path = "../framework", version = "0.2" }

[dev-dependencies]
# Integration tests construct Request/HttpResponse via the public framework API.
# No additional test crates needed.
```

**Version verification (already executed in this research session):**

```bash
grep "name = \"sha2\"" Cargo.lock -A 1   # sha2 0.10.9
grep "name = \"hex\""  Cargo.lock -A 1   # hex  0.4.3
grep "name = \"dashmap\"" Cargo.lock -A 1  # dashmap 6.1.0
```

All three are already in the dep graph; ferro-bundle adds no NEW transitive deps to the workspace.

---

## Framework HTTP Types Reference

This section gives the planner verbatim-embeddable call shapes. Every line is verified against the source files cited.

### Constructing a 200 response with raw bytes + cache headers + ETag

```rust
// Verified against: framework/src/http/response.rs:48-54, 121-126
use ferro_rs::HttpResponse;

let resp = HttpResponse::bytes(bytes_static)              // &'static [u8] -> Bytes (via impl Into<Bytes>)
    .header("Content-Type", content_type_or_octet_stream)
    .header("Content-Length", body_len.to_string())       // optional — into_hyper does not auto-set
    .header("Cache-Control", "public, max-age=31536000, immutable")
    .header("ETag", format!("\"{}\"", full_sha256_hex));  // quoted per RFC 7232
```

**Notes:**
- `HttpResponse::bytes` (response.rs:48-54) sets **no** default headers. Caller MUST add Content-Type.
- `.header()` (response.rs:121-126) uses **case-insensitive replace semantics** — setting `Content-Type` twice yields one header. Use `.append_header()` (response.rs:133-136) only for legitimately multi-value headers like `Set-Cookie`. Bundle response uses `.header()` exclusively.
- For `&'static [u8]`: `HttpResponse::bytes(byte_slice)` works directly because `bytes::Bytes` has `From<&'static [u8]>` (zero-copy).

### Constructing a 304 Not Modified

```rust
// Verified against: framework/src/http/response.rs:18-24 (`new`) + 93-97 (`status`)
let resp = HttpResponse::new()
    .status(304)
    .header("ETag", format!("\"{}\"", full_sha256_hex))
    .header("Cache-Control", "public, max-age=31536000, immutable");
```

**Notes:**
- 304 responses MUST include the same `Cache-Control` and `ETag` headers as the 200 (per RFC 7232 §4.1 — strong validators and selected representation header fields must match).
- 304 responses MUST NOT include a body. `HttpResponse::new()` produces an empty `Bytes::new()` body (response.rs:21).

### Constructing a 301 Moved Permanently (alias redirect)

**Builder form (idiomatic):**
```rust
// Verified against: framework/src/http/response.rs:212-217, 257-260, 278-284
use ferro_rs::Redirect;

let resp_or_err: ferro_rs::Response = Redirect::to(hashed_url)
    .permanent()
    .into();
// resp_or_err is `Result<HttpResponse, HttpResponse>` — unwrap with `?` or `.unwrap_or_else`.
```

**Inline form (one less import, single-step):**
```rust
let resp = HttpResponse::new()
    .status(301)
    .header("Location", hashed_url);
```

Both produce equivalent wire output. Planner picks based on call-site readability. The inline form matches the pattern already used internally by `Redirect`'s `From` impl (response.rs:280-283).

### Reading `If-None-Match` and dispatching by request path

```rust
// Verified against: framework/src/http/request.rs:121-123, 309-311
use ferro_rs::Request;

pub fn serve(req: Request) -> HttpResponse {
    let path = req.path();                          // -> &str, e.g. "/bundles/embed-v1.a4b8c2d1.js"
    let if_none_match = req.header("if-none-match"); // Option<&str>, header lookup is case-insensitive (HeaderMap)

    // Registry lookup (D-02), 304 short-circuit, etc.
    // ...
}
```

**Notes:**
- `req.path()` returns only the path component (no query, no fragment). Matches the registry key shape.
- `req.header(name)` (request.rs:309-311) lowercases name lookup via `HeaderMap` semantics and returns `None` on absent or non-ASCII values.
- Header name `"if-none-match"` is the canonical lowercased form. The framework's `HeaderMap` lookup is case-insensitive but lowercased is conventional in ferro code.

### Public import paths (consumer's `use` line)

```rust
// Verified against: framework/src/lib.rs:108-113
use ferro_rs::{HttpResponse, Request, Response};
// All three are re-exported at crate root from `framework/src/http/mod.rs` via lib.rs line 108-113.
```

ferro-bundle's `Cargo.toml` declares `ferro-rs = { path = "../framework", version = "0.2" }`, and ferro-bundle's source imports as `use ferro_rs::{HttpResponse, Request};`.

---

## Sibling Crate Template Excerpts

### `ferro-wallet/Cargo.toml` (newest leaf-crate template — closest to ferro-bundle shape)

```toml
[package]
name = "ferro-wallet"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Digital wallet pass issuance (Apple .pkpass + Google Wallet) for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["wallet", "pkpass", "google-wallet", "apple-wallet", "ferro"]
categories = ["web-programming"]
readme = "README.md"
homepage = "https://ferro-rs.dev"

[dependencies]
openssl = "0.10"
# ... domain-specific deps ...
thiserror = "2"
```

### `ferro-storage/Cargo.toml` (earlier sibling, shows `dashmap` + `bytes` line items)

```toml
[package]
name = "ferro-storage"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "File storage abstraction for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["storage", "files", "s3", "ferro", "web"]
categories = ["web-programming", "filesystem"]
readme = "README.md"

[dependencies]
# ...
bytes = "1"
dashmap = "6"
thiserror = "1.0"  # older convention — ferro-bundle uses "2" per ferro-wallet
```

### `ferro-wallet/src/error.rs` (canonical `Error` enum + name-prefixed `Display` pattern)

```rust
//! `WalletError` — the single error type for the ferro-wallet crate.
//!
//! Each variant's `Display` impl prefixes its name (`"config: …"`, `"apple sign: …"`)
//! so production log greps stay surgical.

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("config: {0}")]
    Config(String),

    #[error("apple sign: {0}")]
    AppleSign(String),

    // ... 7 more variants ...
}
```

ferro-bundle's `Error` enum follows the same shape. Suggested variants (planner refines):

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bundle not found at path: {0}")]
    NotFound(String),
    #[error("duplicate bundle name: {0} already registered")]
    DuplicateName(String),
    // serve() returns HttpResponse directly (not Result), so Error is mostly an internal/registration-time signal.
    // Per D-06, `DuplicateName` is conveyed via panic, not Result — keep the variant for symmetry but the public surface uses panic.
}
```

### `ferro-json-ui/src/plugin.rs` (the OnceLock global registry pattern, verbatim)

```rust
use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};

// ── Global registry ────────────────────────────────────────────────────

static GLOBAL_PLUGIN_REGISTRY: OnceLock<RwLock<PluginRegistry>> = OnceLock::new();

/// Access the global plugin registry.
///
/// Lazily initialized on first call with built-in plugins registered.
pub fn global_plugin_registry() -> &'static RwLock<PluginRegistry> {
    GLOBAL_PLUGIN_REGISTRY.get_or_init(|| {
        let mut registry = PluginRegistry::new();
        registry.register(crate::plugins::MapPlugin);
        // ... more registrations ...
        registry
    })
}
```

ferro-bundle's translation (DashMap variant, no built-in registrations needed):

```rust
use dashmap::DashMap;
use std::sync::OnceLock;

static BUNDLE_REGISTRY: OnceLock<DashMap<String, BundleEntry>> = OnceLock::new();
static ALIAS_REGISTRY: OnceLock<DashMap<String, String>> = OnceLock::new();

fn bundle_registry() -> &'static DashMap<String, BundleEntry> {
    BUNDLE_REGISTRY.get_or_init(DashMap::new)
}

fn alias_registry() -> &'static DashMap<String, String> {
    ALIAS_REGISTRY.get_or_init(DashMap::new)
}
```

### `ferro-wallet/src/lib.rs` (crate-root re-export pattern)

```rust
//! Digital wallet pass issuance (Apple .pkpass + Google Wallet) for the Ferro framework.

pub mod apple;
pub mod config;
pub mod error;
pub mod google;
// ...

pub use apple::ApplePassBuilder;
pub use error::WalletError;
// ...
```

ferro-bundle's translation (single-file lib — no submodules unless planner splits):

```rust
//! In-memory immutable byte blobs with content-hashed URLs and one-year immutable caching.
//!
//! See README for the bundle-vs-filesystem split: ferro-bundle handles compile-time-embedded
//! immutable assets; the framework's filesystem static-file handler at
//! `framework::static_files` handles mutable on-disk tenant assets.

mod error;        // or inline if single-file
mod registry;     // or inline if single-file
mod bundle;       // or inline if single-file

pub use bundle::Bundle;
pub use error::Error;
```

---

## Workspace Integration

### Workspace root `Cargo.toml` edit

Current (lines 1-30):
```toml
[workspace]
resolver = "2"
members = [
    "framework",
    "app",
    # ... 27 more ...
    "ferro-projection",
]
```

Append `"ferro-bundle",` after `"ferro-projection",` (line 29). The members list does NOT follow alphabetical order — it grows by introduction phase, matching the convention established by `feedback_alphabetical_workspace.md`-style memory (ferro-wallet was appended at line 24 after ferro-whatsapp, not alphabetised).

Workspace version bump: change `version = "0.2.42"` (line 33) to `version = "0.2.43"` in `Cargo.toml` `[workspace.package]`. This is the ONLY version-bump action — `version.workspace = true` in ferro-bundle's Cargo.toml inherits.

### `.github/workflows/publish.yml` edit — **CRITICAL: re-wave assignment**

CONTEXT.md §D-09 specifies `WAVE1B_CRATES` (line 246). **This is wrong.** Verification against `.github/workflows/publish.yml`:

| Wave | Crates | Line range | ferro-rs status |
|------|--------|------------|-----------------|
| **1a** | `ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit ferro-migration` | 211 | Not present |
| **1b** | `ferro-ai ferro-projections ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection` | 246 | Not present |
| **2**  | `ferro-rs ferro-mcp` | 274 | **Published HERE** |
| **3**  | `ferro-cli` | 297-310 | Not present |

`ferro-bundle` depends on `ferro-rs` (= the `framework` crate). It cannot publish in Wave 1B because `ferro-rs` has not yet been published at that point — `cargo publish` would fail with "no matching package found" against crates.io for `ferro-rs = "0.2"`.

**Required: introduce a new wave after Wave 2.** Three viable shapes:

1. **Insert a new "Wave 2.5"** after the Wave 2 publish step (line 290) and before "Wait for crates.io index update" (line 292). The new wave publishes `ferro-bundle` after `ferro-rs` is live. This is the most surgical edit.
2. **Move `ferro-cli`'s Wave 3 to handle both** — rename `Wave 3` to "post-framework" and add `ferro-bundle` alongside `ferro-cli`. This works because both depend on `ferro-rs`. Cleaner conceptually but renames an existing step.
3. **Defer ferro-bundle into the Wave 3 block** — append `ferro-bundle` to the existing Wave 3 (currently single-crate `ferro-cli`). Smallest diff; semantic mismatch (Wave 3 was historically "CLI only").

**Recommendation: Option 1 or 2.** The planner picks based on workflow-file diff minimization. Either way, CONTEXT.md's `WAVE1B_CRATES` assignment must be revised before the publish.yml edit lands.

The "Wait for crates.io index update" sleep (line 292-294) is needed AFTER the new ferro-bundle publish step so that any downstream consumer (none today, but future ferro crates might) can resolve it.

### First-publish bootstrap command (D-12)

The CI workflow will fail to publish `ferro-bundle` on the first run because the `CARGO_REGISTRY_TOKEN` (line 9 of publish.yml) has scope `publish-update` only, not `publish-new` (per memory `project_ferro_publish_token_scoping.md`). The phase plan must include a manual-bootstrap task:

```bash
# Run from local terminal (not CI). Local cargo login token has full permissions.
cd /Users/alberto/repositories/albertogferrario/ferro
cargo publish -p ferro-bundle
# Verify on crates.io: https://crates.io/crates/ferro-bundle
```

This is a one-time action. After the crate exists on crates.io, subsequent versions ship via the workflow's `cargo publish -p ferro-bundle --no-verify` line automatically (mirroring the existing per-crate publish loop at lines 213-229 / 248-262 / 276-290).

**Important caveat for the planner:** the workspace version is bumped BEFORE the first manual publish. The sequence is:
1. Land Phase 183 plans (crate scaffold + workspace member + publish.yml entry + version bump 0.2.42 → 0.2.43).
2. Merge to master.
3. CI runs publish.yml. Wave 2 publishes `ferro-rs 0.2.43`. New "Wave 2.5" attempts `cargo publish -p ferro-bundle` → fails with "crate not found" (the token cannot create the crate on first call).
4. **Manual bootstrap from local terminal:** `cargo publish -p ferro-bundle`. The local cargo token has `publish-new` rights and creates the crate at version 0.2.43.
5. Future merges that bump the workspace version → CI Wave 2.5 publishes `ferro-bundle 0.2.44`, etc.

The planner should include a documentation task that captures the bootstrap step in `.planning/phases/183-…/183-XX-SUMMARY.md` so the workflow is traceable.

---

## Code Examples (verified patterns the planner embeds verbatim)

### Example 1: Bundle struct + builder methods + global registration (D-06)

```rust
// File: ferro-bundle/src/lib.rs (or src/bundle.rs if planner splits)
// Sources: framework/src/http/response.rs:48-54 (HttpResponse::bytes),
//          ferro-json-ui/src/plugin.rs:147 (OnceLock pattern),
//          ferro-wallet/src/lib.rs (re-export pattern)

use bytes::Bytes;
use dashmap::DashMap;
use ferro_rs::{HttpResponse, Request};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

static BUNDLE_REGISTRY: OnceLock<DashMap<String, BundleEntry>> = OnceLock::new();
static ALIAS_REGISTRY: OnceLock<DashMap<String, String>> = OnceLock::new();

fn bundle_registry() -> &'static DashMap<String, BundleEntry> {
    BUNDLE_REGISTRY.get_or_init(DashMap::new)
}

fn alias_registry() -> &'static DashMap<String, String> {
    ALIAS_REGISTRY.get_or_init(DashMap::new)
}

struct BundleEntry {
    name: String,
    bytes: &'static [u8],
    content_type: String,    // default "application/octet-stream"
    sha256_full_hex: String, // 64 chars
    sha256_short_hex: String, // first 8 chars
    ext: String,             // derived from content_type; empty for unknown
    hashed_url: String,      // "/bundles/{name}.{sha8}.{ext}" or "/bundles/{name}.{sha8}" if ext empty
}

/// In-memory immutable byte blob registered at boot.
pub struct Bundle {
    name: String,
}

impl Bundle {
    pub fn new(name: &str, bytes: &'static [u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let sha256_full_hex = hex::encode(digest);
        let sha256_short_hex = sha256_full_hex[..8].to_string();
        let entry = BundleEntry {
            name: name.to_string(),
            bytes,
            content_type: "application/octet-stream".to_string(),
            sha256_short_hex: sha256_short_hex.clone(),
            ext: String::new(),
            hashed_url: format!("/bundles/{}.{}", name, sha256_short_hex),
            sha256_full_hex,
        };
        let registry = bundle_registry();
        if let Some(existing) = registry.get(&entry.hashed_url) {
            // Per D-06: panic on duplicate name regardless of byte equality.
            // Identical re-registration is still developer error (forgotten call-site, hot-reload bug).
            panic!(
                "ferro-bundle: duplicate registration for bundle name {:?} (existing url: {})",
                name, existing.hashed_url
            );
        }
        registry.insert(entry.hashed_url.clone(), entry);
        Bundle { name: name.to_string() }
    }

    pub fn content_type(self, ct: &str) -> Self {
        let registry = bundle_registry();
        // Look up by current hashed_url (which doesn't yet have an ext). Update entry.
        // ... mutation logic — see Open Questions §1 for the OQ on Dashmap entry shape.
        self
    }

    pub fn with_alias(self, alias_path: &str) -> Self {
        let registry = bundle_registry();
        // Find this bundle's current hashed_url and insert alias_path -> hashed_url mapping.
        // ... see Open Questions §2 for the alias-tracking discussion.
        self
    }

    pub fn hashed_url(&self) -> String {
        // Look up current entry in registry, return its `hashed_url`.
        bundle_registry()
            .iter()
            .find(|e| e.value().name == self.name)
            .map(|e| e.value().hashed_url.clone())
            .unwrap_or_default()  // unreachable in practice — new() always inserts
    }

    pub fn serve(req: Request) -> HttpResponse {
        let path = req.path();

        // Alias check first (D-03 ordering).
        if let Some(target) = alias_registry().get(path) {
            return HttpResponse::new()
                .status(301)
                .header("Location", target.value().clone());
        }

        // Bundle check.
        if let Some(entry) = bundle_registry().get(path) {
            let etag = format!("\"{}\"", entry.sha256_full_hex);
            if let Some(if_none_match) = req.header("if-none-match") {
                if if_none_match == etag {
                    return HttpResponse::new()
                        .status(304)
                        .header("ETag", &etag)
                        .header("Cache-Control", "public, max-age=31536000, immutable");
                }
            }
            return HttpResponse::bytes(Bytes::from_static(entry.bytes))
                .header("Content-Type", &entry.content_type)
                .header("Cache-Control", "public, max-age=31536000, immutable")
                .header("ETag", &etag);
        }

        // 404 fallback (defensive).
        HttpResponse::new().status(404).header("Content-Type", "text/plain")
    }
}

#[cfg(test)]
pub(crate) fn reset() {
    if let Some(r) = BUNDLE_REGISTRY.get() { r.clear(); }
    if let Some(r) = ALIAS_REGISTRY.get() { r.clear(); }
}
```

**Caveats for the planner:**
- The `content_type` and `with_alias` builder methods must MUTATE the registry entry (re-insert under a new `hashed_url` key when `ext` changes). DashMap supports in-place `.get_mut()` so this is workable. The exact mutation shape needs design — see Open Questions §1 and §2.
- The `hashed_url()` lookup is O(n) in the snippet above; consider a secondary `name -> hashed_url` index for O(1) lookup if performance matters (it likely does not — `hashed_url()` is called once per bundle at boot, not per request).
- `Bytes::from_static(&'static [u8])` is zero-copy — the bytes are not copied on every request.

### Example 2: Content-type → extension table

```rust
fn ext_from_content_type(ct: &str) -> &'static str {
    match ct.split(';').next().unwrap_or(ct).trim() {
        "application/javascript" | "text/javascript" => "js",
        "text/css" => "css",
        "text/html" => "html",
        "text/plain" => "txt",
        "application/json" => "json",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/svg+xml" => "svg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "font/woff2" => "woff2",
        "font/woff" => "woff",
        "application/wasm" => "wasm",
        _ => "",  // unknown → no extension; URL becomes /bundles/{name}.{sha8}
    }
}
```

**Note:** `mime_guess` (used at `framework/src/http/response.rs:68-71` and `framework/src/static_files.rs:47-50` for the inverse direction `path → mime`) does not provide `mime → ext` cheaply. A hand-rolled match table is simpler and covers all common cases. Listed extensions are derived from gestiscilo Phase 185's likely bundle types (SDK JS, fonts, icons).

### Example 3: Test scaffolding (D-13)

```rust
// File: ferro-bundle/src/lib.rs (or tests/registry.rs)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        reset();
        let b = Bundle::new("test1", b"hello").content_type("text/plain");
        // SHA-256 of "hello" = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        // First 8 chars = 2cf24dba
        assert_eq!(b.hashed_url(), "/bundles/test1.2cf24dba.txt");
    }

    #[test]
    fn etag_is_full_sha256_quoted() {
        reset();
        let _b = Bundle::new("test2", b"hello").content_type("text/plain");
        // ... build a Request synthetically (see Open Questions §3), call Bundle::serve, assert ETag.
    }

    #[test]
    #[should_panic(expected = "duplicate registration")]
    fn duplicate_name_panics() {
        reset();
        Bundle::new("dup", b"a");
        Bundle::new("dup", b"a");  // identical bytes — still panics per D-06
    }
}
```

The `reset()` call at the top of every test is the test-isolation convention from D-13. Without it, the second test that registers `"hello"` (or any name colliding with a prior test) panics on the duplicate-registration check.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SHA-256 hashing | Custom hash impl | `sha2::Sha256::digest` | Constant-time, audited, workspace-pinned. |
| Hex encoding | Custom `format!("{:02x}", b)` loop | `hex::encode` | Zero-alloc reuses, handles edge cases. |
| Concurrent map | `RwLock<HashMap<…>>` | `dashmap::DashMap` | Bucket-level locking; read-mostly workload (serve path). RwLock works too — see Alternatives. |
| Process-global initialization | `lazy_static!` / `once_cell` | `std::sync::OnceLock` (Rust 1.70+) | Stable std. Ferro's MSRV is `1.88.0`. |
| HTTP response shaping | Raw `hyper::Response::builder()` | `framework::HttpResponse::bytes(…).header(…)` | Idiomatic for ferro consumers; same wire output via `into_hyper()`. Direct hyper bypasses the case-insensitive header replace semantics (response.rs:117-126) the framework relies on. |
| 301 redirect builder | Custom `.status(301).header("Location", …)` | `Redirect::to(url).permanent().into()` | Both work; builder is idiomatic. Picker's discretion. |
| Content-type sniffing | Inferring from filename | Caller provides at registration (D-04 / SC-4) | Explicitly excluded by success criterion. `mime_guess` is wrong direction (path → mime); we need mime → ext, which has no clean library. |
| Test isolation between integration tests | `serial_test` crate or test mutex | OS-level process isolation (default cargo test behavior per binary) | Cargo runs each `tests/*.rs` file as a separate binary by default. Single-binary unit tests use `reset()` per D-13. |

**Key insight:** every primitive ferro-bundle needs already exists in the workspace dep graph. No new transitive deps are introduced.

---

## Common Pitfalls

### Pitfall 1: Wrong publish wave breaks CI on first merge

**What goes wrong:** Following CONTEXT.md §D-09 verbatim — adding `ferro-bundle` to `WAVE1B_CRATES` — produces a CI failure on the first publish because `ferro-rs` is published in Wave 2, which runs AFTER Wave 1B.
**Why it happens:** CONTEXT.md misidentifies framework's wave. The `framework` crate (= `ferro-rs` on crates.io) is at line 274 of publish.yml, in `WAVE2_CRATES`, NOT Wave 1A.
**How to avoid:** Introduce a new wave (recommended: insert "Wave 2.5" between line 290 and line 292 of publish.yml). Or append `ferro-bundle` to Wave 3 alongside `ferro-cli`.
**Warning signs:** CI publish step output reads `error: failed to select a version for the requirement 'ferro-rs = "^0.2.X"': no matching package named 'ferro-rs' found`.

### Pitfall 2: `.content_type()` mutates the URL but registry key still matches old hashed_url

**What goes wrong:** `Bundle::new("x", b"…")` inserts under key `/bundles/x.{sha8}` (no ext). Then `.content_type("text/css")` should change the URL to `/bundles/x.{sha8}.css`. If the registry key isn't updated, `Bundle::serve` looks up the wrong path and returns 404.
**Why it happens:** DashMap's key cannot be mutated in place — the entry must be removed and reinserted under the new key.
**How to avoid:** Builder semantics. `content_type` removes the old entry from the registry, applies the new ext, and reinserts under the new key. Alternatively: defer registry insertion until the builder chain completes (terminal method like `.register()` or implicit drop). Phase 183 picks one — see Open Questions §1.
**Warning signs:** Test `Bundle::new("x", b"…").content_type("text/css").hashed_url()` returns a URL but `Bundle::serve(req with that path)` returns 404.

### Pitfall 3: ETag without surrounding quotes silently breaks 304 fast-path

**What goes wrong:** Browser sends `If-None-Match: "abc…"` (RFC 7232 §2.3 mandates quotes for strong ETags). Server stored ETag as `abc…` (unquoted). String comparison fails. 304 never fires; cache effectively disabled.
**Why it happens:** Rust developers tend to format the hash directly: `.header("ETag", hash)` instead of `.header("ETag", format!("\"{}\"", hash))`.
**How to avoid:** Code example §1 above uses `format!("\"{}\"", entry.sha256_full_hex)`. Test asserts the quoted form.
**Warning signs:** Tests using a mocked `If-None-Match: "abc…"` header against an unquoted-stored ETag return 200 instead of 304.

### Pitfall 4: 304 response missing `Cache-Control` causes browsers to re-revalidate immediately

**What goes wrong:** Per RFC 7232 §4.1, the 304 response should include the same `Cache-Control` and `ETag` as the 200 it short-circuits. Without `Cache-Control` on 304, browsers may treat the cache entry as stale and immediately re-request.
**Why it happens:** "Empty body" is conflated with "empty headers." 304 has no body but must keep cache-control headers.
**How to avoid:** Code example §1 above includes `Cache-Control` on the 304 path explicitly.
**Warning signs:** Browser DevTools network panel shows back-to-back requests to the same URL despite 304 responses.

### Pitfall 5: Process-global state leaks across unit tests in the same binary

**What goes wrong:** Test A registers `Bundle::new("shared")`. Test B also registers `Bundle::new("shared")` and panics on duplicate. Tests interfere.
**Why it happens:** Cargo runs unit tests inside ONE binary by default; the `OnceLock` registry persists across tests.
**How to avoid:** D-13's `reset()` helper called at the top of every unit test. OR push the affected scenarios into separate `tests/*.rs` integration binaries (one binary per file = OS-level process isolation).
**Warning signs:** Tests pass individually but fail when run together (`cargo test -p ferro-bundle` fails; `cargo test -p ferro-bundle -- hash_is_deterministic` passes).

### Pitfall 6: Constructing a synthetic `Request` for unit tests requires careful setup

**What goes wrong:** The `Request::new(hyper::Request<hyper::body::Incoming>)` constructor (request.rs:55-65) requires a `hyper::body::Incoming` which is hard to construct outside an actual server context.
**Why it happens:** `Incoming` is hyper-internal and not normally exposed for synthetic test construction.
**How to avoid:** Look at `framework/tests/routing_group_trailing_slash.rs` and `framework/Cargo.toml:80-83` ("Integration test helpers (action_handler.rs uses TCP-loopback Request constructor)") — the framework's own integration tests use a TCP-loopback helper. ferro-bundle's integration tests can mirror this; OR use `framework::testing::*` (lib.rs line 40 declares `pub mod testing`) if it exposes a Request constructor. The planner verifies during plan phase. See Open Questions §3.
**Warning signs:** Planning `Bundle::serve(req)` tests and hitting a type wall constructing `Request`.

### Pitfall 7: `Bytes::from_static(&'static [u8])` vs `Bytes::from(Vec<u8>)`

**What goes wrong:** Using `Bytes::from(vec.clone())` instead of `Bytes::from_static(static_slice)` causes a copy on every serve.
**Why it happens:** Familiarity bias — Rust developers reach for `Vec<u8>` patterns.
**How to avoid:** `Bytes::from_static` is zero-copy and exactly matches the `&'static [u8]` lifetime contract D-01 locks in.
**Warning signs:** `cargo bench` shows allocations on every serve call; or a code reviewer flags the `.clone()`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `lazy_static!` macro | `std::sync::OnceLock` | Rust 1.70 (June 2023) | Std-stable, no macro magic; ferro requires 1.88.0 so no compatibility constraint. |
| `weak ETag` (W/"…") | strong ETag for content-immutable bundles | RFC 7232 (always) | Immutable bytes = byte-for-byte identical = strong ETag (D-05). |
| `max-age=3600` | `max-age=31536000, immutable` | Cache-Control `immutable` directive (RFC 8246, 2017) | The `immutable` directive prevents revalidation during the resource's freshness lifetime. Critical for the discovery use case (current `max-age=300, stale-while-revalidate=86400` revalidates after 5 minutes). |
| `If-Modified-Since` validator | `If-None-Match` validator | RFC 7232 (current spec) | Both are valid; `If-None-Match` works with strong ETags and is required for immutable content. ferro-bundle implements `If-None-Match` only. |
| `application/octet-stream` default for unknown types | Same — no change | — | Per D-04 / SC-4. |

**Deprecated/outdated (in this domain):**
- ServiceWorker-based caching (`Cache API`) — orthogonal to immutable HTTP caching; the browser's HTTP cache handles immutable content without any JS.
- ETag-only validation (no `Cache-Control: immutable`) — modern best practice combines both.

---

## Project Constraints (from CLAUDE.md)

1. **"When adding a new crate to the workspace, always add it to `.github/workflows/publish.yml` in the correct wave."** ferro-bundle adds an entry in publish.yml (Wave 2.5 per Risks §1) and a new line in workspace.members.
2. **"Project-agnostic crates."** ferro-bundle must not hardcode app identity. The crate is generic infrastructure for serving immutable byte blobs; no `APP_NAME` / `APP_URL` reading needed (the URL prefix `/bundles/` is a path convention, not a tenant-specific value).
3. **"Run fmt + clippy + tests before every commit."** Specifically the CI-matching command per memory `feedback_ci_clippy_command_match.md`: `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features`.
4. **"Always update docs when framework changes."** Per CONTEXT.md (§canonical_refs), ferro-bundle's README satisfies this; no separate `docs/src/` page is required (bundle is a Rust API, not a JSON-UI authoring surface).
5. **"Never add co-author attribution to commits."** Standard.
6. **"Repository documents must read as neutral."** ferro-bundle's README documents the bundle-vs-filesystem split in neutral architectural voice, not internal-strategy framing.
7. **"Prefer editing existing files over creating new ones."** Phase 183 is irreducibly creating new files (new crate). The constraint applies to keeping the file count minimal: single `lib.rs` preferred over a multi-module split unless the planner judges readability gain.

---

## Runtime State Inventory

Not applicable. Phase 183 is purely additive (new crate, no rename / refactor / migration / data store interaction). No stored data, live service config, OS-registered state, secrets, or build artifacts are affected.

---

## Environment Availability

Not applicable. Phase 183 has no external runtime dependencies beyond standard Rust toolchain (already in use). All deps (`sha2`, `hex`, `dashmap`, `bytes`, `thiserror`) are pure Rust crates in the existing workspace dep graph. No databases, services, or CLI utilities are introduced.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in Rust test harness) |
| Config file | None — convention-driven via `[cfg(test)]` blocks and `tests/*.rs` integration binaries |
| Quick run command | `cargo test -p ferro-bundle` (runs only new crate, fast — single crate, no transitive recompile) |
| Full suite command | `cargo test --all-features` (gates the merge per CLAUDE.md) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BUNDLE-01 | Hashed URL is deterministically derived from SHA-256 of bytes | unit | `cargo test -p ferro-bundle hash_is_deterministic` | ❌ Wave 0 (lib.rs `#[cfg(test)]`) |
| BUNDLE-02 cold | 200 response with `Cache-Control: public, max-age=31536000, immutable` + quoted-SHA256 ETag | integration | `cargo test -p ferro-bundle --test serve_cold` | ❌ Wave 0 (`tests/serve_cold.rs`) |
| BUNDLE-02 304 | 304 fast-path on `If-None-Match` exact match | integration | `cargo test -p ferro-bundle --test serve_304` | ❌ Wave 0 (`tests/serve_304.rs`) |
| BUNDLE-03 | 301 redirect on alias path | integration | `cargo test -p ferro-bundle --test alias_redirect` | ❌ Wave 0 (`tests/alias_redirect.rs`) |
| BUNDLE-04 | Default `application/octet-stream` content-type when unspecified | unit | `cargo test -p ferro-bundle default_content_type_is_octet_stream` | ❌ Wave 0 (lib.rs) |
| BUNDLE-04 | Duplicate name panics | unit | `cargo test -p ferro-bundle duplicate_name_panics` (uses `#[should_panic]`) | ❌ Wave 0 (lib.rs) |
| BUNDLE-05 | README documents bundle-vs-filesystem split | manual / grep | `grep -F 'do not fold' ferro-bundle/README.md` | ❌ Wave 0 (`README.md`) |
| BUNDLE-06 | Publish wave correctness | manual — verified at merge | (no automated test; planner adds grep gate: `grep -F 'ferro-bundle' .github/workflows/publish.yml`) | ❌ Wave 0 (`publish.yml`) |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-bundle` (fast — single crate scope).
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` (CLAUDE.md / CI-matching gate).
- **Phase gate:** Full workspace test suite green; `cargo publish -p ferro-bundle --dry-run` exits 0 from local terminal (verifies metadata + version + dep resolution before D-12 bootstrap).

### Wave 0 Gaps

All test files are new — Phase 183 builds the test suite from scratch. The planner SHOULD add a Wave 0 task that creates:

- [ ] `ferro-bundle/src/lib.rs` `#[cfg(test)] mod tests` block (hash determinism, default content-type, duplicate-name panic).
- [ ] `ferro-bundle/tests/serve_cold.rs` (synthetic Request + asserts 200 headers; depends on Open Questions §3 — Request construction shape).
- [ ] `ferro-bundle/tests/serve_304.rs` (synthetic Request with `If-None-Match` header + asserts 304).
- [ ] `ferro-bundle/tests/alias_redirect.rs` (synthetic Request to alias path + asserts 301 + `Location`).
- [ ] No additional framework install command needed — `cargo test` is built-in.

If Open Questions §3 (Request constructor) is resolved against the integration-test path being too heavy, the integration tests fold into `lib.rs` unit tests calling internal helper functions that take `&str path, Option<&str> if_none_match` and bypass the Request construction.

---

## Security Domain

ferro-bundle exposes byte-blob serving over HTTP. Security threat surface is narrow but real.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | The `/bundles/*` namespace is intentionally public (SDK bundle, fonts, icons). No auth required. |
| V3 Session Management | no | Stateless GETs. |
| V4 Access Control | no | All bundles publicly readable by design. |
| V5 Input Validation | yes | `req.path()` is the registry key. DashMap lookup is exact-match (no path traversal possible). `If-None-Match` header is string-compared (no parsing, no injection vector). |
| V6 Cryptography | yes | SHA-256 via `sha2` crate. Never hand-rolled. |
| V14 Configuration | yes | Caller MUST provide `content_type` for any bundle exposing user-controlled rendering surface (e.g., HTML). Default `application/octet-stream` is safe. |

### Known Threat Patterns for static-bundle serving

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via `req.path()` | Tampering | DashMap key is exact match. `/bundles/../etc/passwd` is a key that won't match any registered entry → 404. No filesystem access. |
| ETag-based timing oracle on byte equality | Information disclosure | All bundle bytes are public; ETag value is the public hash. No secret comparison. |
| Cache poisoning via `Vary` mismatch | Tampering | ferro-bundle responses do NOT vary by request header. No `Vary` needed. If future versions add `Accept-Encoding`-selected variants, `Vary: Accept-Encoding` becomes mandatory. |
| Content-type confusion attack (XSS via `text/html` bundle) | Tampering / EoP | Caller-provided content-type is a deliberate API choice. Documentation must warn that user-controlled `name` + `text/html` could be exploited if the URL is reachable in a same-origin context. Recommendation: README adds a security note. |
| Resource exhaustion via many bundles | DoS | Each bundle's bytes are `&'static [u8]` — the size is bounded at compile time. Number of bundles is bounded by code (each `Bundle::new` call is explicit). No runtime growth vector. |
| Duplicate-name panic as DoS | DoS | Per D-06, duplicate name panics. Panic happens at registration (boot), not at serve. A boot-time panic crashes the server BEFORE accepting requests, which is the correct behavior — better to fail loudly than serve the wrong bytes. |

**Documentation requirement:** README should include a brief security note: "Content-type is caller-provided; serving bundles with `text/html` content-type from a domain with auth cookies is a known XSS vector. Use only for static assets that would not benefit from being on a CDN sandbox." This satisfies V14's intent without overengineering.

---

## Risks and Edge Cases

### Risk 1: CONTEXT.md `WAVE1B_CRATES` assignment is incorrect (HIGH priority — blocks first publish)

**Detail:** CONTEXT.md §D-09 and §D-11 specify `WAVE1B_CRATES`. Direct read of `.github/workflows/publish.yml` shows:
- Wave 1a (line 211): leaf crates with no internal ferro deps. `framework` NOT present.
- Wave 1b (line 246): depend on Wave 1a only. `framework` NOT present.
- **Wave 2 (line 274): `ferro-rs ferro-mcp` — `framework` published HERE.**
- Wave 3 (line 297): `ferro-cli`.

ferro-bundle depends on `ferro-rs`. If placed in Wave 1B, `cargo publish` errors with "no matching package for ferro-rs = ^0.2.X" because Wave 2 hasn't run yet.

**Resolution:** Introduce a "Wave 2.5" after Wave 2 (line 290) and before the index-wait sleep (line 292), OR append `ferro-bundle` to Wave 3 alongside `ferro-cli` (both are post-framework consumers). Planner decides during plan phase.

**Where to update CONTEXT.md:** D-09 line "Wave1B publish" should read "post-`ferro-rs` publish wave (new wave between Wave 2 and Wave 3, OR appended to Wave 3)." Same for D-11.

### Risk 2: `Bundle::new()` builder mutation across multiple registry entries

**Detail:** The locked builder chain `.content_type(ct).with_alias(path)` registers eagerly at `Bundle::new`. But `content_type` changes the URL extension, which is part of the registry key. Mutation requires removing the old key and reinserting under the new key. With DashMap, this is doable but introduces an instant in which the bundle is not findable. For a single-threaded boot sequence this is fine; for hypothetical concurrent boot it is a race. Recommendation: register under the FULL final URL (post-content_type) by deferring insertion until the builder chain completes via consuming `self` — OR accept the brief gap during boot (no requests in flight at boot).

**Recommended resolution:** Defer insertion until the user calls `.hashed_url()` OR until the `Bundle` is dropped. Actually simpler: eagerly insert at `Bundle::new`, and on `content_type`'s call, remove the old key and reinsert under the new key. Boot is single-threaded by convention; this is acceptable. Document the constraint: `Bundle::new` and builder calls MUST complete before the server starts accepting requests.

### Risk 3: `.with_alias()` requires knowing the bundle's current hashed_url

**Detail:** `with_alias("/embed/v1.js")` needs to map `/embed/v1.js → /bundles/embed-v1.{sha8}.{ext}`. To do this, the builder must know its OWN current hashed_url. The current code example §1 stores the `name` on `Bundle` and looks up by name; lookup requires iterating the registry. Faster: store the `hashed_url` on `Bundle` itself.

**Recommended resolution:** `Bundle` carries `name: String` AND `hashed_url: String` (last-computed). Builders update both fields in lockstep with the registry mutation.

### Risk 4: Default content-type with unknown extension produces URL with no extension

**Detail:** Per D-04, default content-type is `application/octet-stream`. The content-type→ext table doesn't map `application/octet-stream` to any extension (it would be `.bin` but that's not in the table). Result: URL is `/bundles/{name}.{sha8}` (no ext suffix).

**Recommended resolution:** Document this in README. The URL is still a valid path; the browser uses the Content-Type header to determine handling, not the URL extension. No functional impact.

### Risk 5: Test ergonomics — synthetic Request construction

**Detail:** `Request::new` takes a `hyper::Request<hyper::body::Incoming>` (request.rs:55-65). `Incoming` is hyper-internal and not trivially constructed in tests. The framework's own tests use TCP loopback (Cargo.toml line 81-83 comment).

**Recommended resolution:** Either (a) inspect `framework::testing` module (lib.rs:40 `pub mod testing`) for an exposed `Request` constructor — planner verifies during plan phase, OR (b) write integration tests using TCP loopback (heavy but reliable — matches framework's pattern), OR (c) extract a helper function `dispatch(path: &str, if_none_match: Option<&str>) -> HttpResponse` that bypasses `Request` and have `Bundle::serve` delegate to it; unit-test the helper, integration-test the wrapper end-to-end.

**Researcher recommendation:** Option (c) — extract a private helper for unit-test ergonomics. Integration tests use a single TCP-loopback fixture to verify end-to-end Request → HttpResponse path on one happy-path case.

### Risk 6: gestiscilo Phase 185 friction-loop coordination

**Detail:** Per memory `feedback_friction_loop_release_cadence.md`, single publish at end of release loop. Phase 183 publishes ONCE at merge to master. gestiscilo Phase 185 consumes via Cargo.toml bump AFTER ferro-bundle is live on crates.io. If gestiscilo tries to consume via `ferro-bundle = { path = "../ferro/ferro-bundle" }` during development (which is fine for local iteration), the bump to crates.io version is the load-bearing step at the end.

**Recommended resolution:** Phase 183's plan includes a "verify on crates.io" task that's manually checked after D-12 bootstrap completes.

### Risk 7: Edge case — `Bundle::serve` called with a path that has no entry in either registry

**Detail:** D-03 specifies "404 fallback (defensive)." But the consumer wires `Bundle::serve` only to `/bundles/{filename}` (catch-all). What if the consumer accidentally wires it to `/` ?

**Recommended resolution:** Code example §1 covers this — 404 with `Content-Type: text/plain`. Optionally return a body like `"Bundle not found"`. Defensive, not load-bearing.

### Risk 8: Multiple bundles with the same SHORT hash but different names

**Detail:** D-04 uses 8 hex chars (32 bits). Collision risk at 100 bundles is ~0.0001 (birthday-paradox approximation). At 10,000 bundles it rises to ~1%. For typical use (dozens of bundles per app), risk is negligible.

**Recommended resolution:** Document the collision space in README. Note that even with a hash collision, the registry key includes the bundle NAME prefix (`/bundles/{name}.{sha8}.{ext}`), so two bundles named differently with the same hash do NOT collide on the registry. Hash collision only matters if it produces the same final URL — and the name component disambiguates.

---

## Cross-References to Sibling Phases

| Sibling Phase | Relationship | What to Borrow / Avoid |
|---------------|--------------|-------------------------|
| **Phase 151 (`ferro-wallet`)** — Shipped 2026-05-11 | Closest analog. New top-level workspace crate with a public-API surface; multi-plan rollout. | Borrow: `151-01-scaffold-PLAN.md` structure (manifest + README + lib.rs + workspace member + publish.yml in one atomic plan). README pattern (short, neutral). Per-variant Error tests pattern. AVOID: ferro-wallet has zero internal ferro deps and went into Wave 1A; ferro-bundle has a framework dep and CANNOT use Wave 1A. |
| **Phase 152-155 (`ferro-orm`, `ferro-audit`, `ferro-reservation`, `ferro-projection`)** — Shipped 2026-05-13 | Multiple new crates landed in one milestone. | Borrow: workspace.members append-not-alphabetise convention (D-10 in 151 CONTEXT). AVOID: those crates landed in Wave 1A (no framework dep); same wave caveat as Phase 151. |
| **Phase 182 (`ferro-json-ui` data-lazy-hero)** — Shipped 2026-06-06 | Sibling phase in the same v12.2 milestone. Friction-loop coordination with gestiscilo Phase 186. | Borrow: single-publish-at-end-of-phase pattern (memory `feedback_friction_loop_release_cadence.md`). Workspace version bump as the LAST plan (`182-03-PLAN.md` pattern). |
| **Phase 145 (`ferro serve` watch supervisor)** — Shipped 2026-04-22 | Different domain (CLI tooling), but multi-plan structure with Wave 0 test infrastructure first. | Borrow: Wave 0 test-fixture + pure-function-contract scaffolding pattern (`145-01-PLAN.md`'s "minimal-serve fixture + integration-test scaffold + pure-function contracts" approach). Inspired the Wave 0 Gaps section above. |
| **Phase 184 (`ferro::InlineBudget` + `RequestTelemetry`)** — Same milestone, NOT YET PLANNED | Next phase in v12.2 milestone. Will land after 183. | Awareness: 184's request-scoped primitives may interact with ferro-bundle response shaping (e.g., logging serve latencies). Not a blocking dependency. No coordination needed at this time. |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `framework::testing` module exposes a Request constructor usable from integration tests in sibling crates | Risk 5, Common Pitfalls Pitfall 6 | Planner discovers during plan phase that `framework::testing` is not visible to `ferro-bundle` (it would need a re-export at framework's crate root); fallback to TCP-loopback test pattern from `framework/tests/routing_group_trailing_slash.rs`. Manageable — no plan-phase blocker. |
| A2 | `Bytes::from_static(&'static [u8])` is part of `bytes 1.x` public API and is zero-copy | Standard Stack, Pitfall 7 | LOW — `Bytes::from_static` is documented in `bytes 1.0+` (https://docs.rs/bytes/latest/bytes/struct.Bytes.html#method.from_static). Verified during plan via `cargo doc --no-deps -p bytes` if needed. |
| A3 | gestiscilo Phase 185 will consume `ferro-bundle` via `Cargo.toml` bump to the published crates.io version (not a path dep) | Risk 6, BUNDLE-06 | LOW — established pattern per memory `feedback_friction_loop_release_cadence.md`. |
| A4 | First-publish manual bootstrap remains necessary (CI token scope unchanged since memory entry) | D-12, Workspace Integration §First-publish bootstrap | LOW — verifiable by checking CARGO_REGISTRY_TOKEN scope on the GH org; researcher did not verify directly this session but the memory entry is authoritative. |
| A5 | DashMap's read path (`registry.get(path)`) is cheap enough for the serve hot path (every request to `/bundles/*`) | Code Example §1, Don't Hand-Roll | LOW — DashMap is bucket-locked and used by `ferro-cache` and `ferro-storage` already in serve paths. |

---

## Open Questions

1. **`Bundle::content_type(ct)` builder mutation semantics — re-insert under new key, or defer to terminal call?**
   - What we know: D-06 says registration is eager at `Bundle::new`. D-04 says URL includes ext derived from content_type.
   - What's unclear: how `content_type` mutates the registry key. Two valid implementations (re-insert vs deferred terminal-method).
   - Recommendation: re-insert under new key. Boot is single-threaded; the brief gap is acceptable. Document the constraint in lib.rs rustdoc.

2. **`Bundle::with_alias(alias_path)` — should the alias track changes to the hashed_url?**
   - What we know: If `content_type` is called AFTER `with_alias`, the alias maps to a stale URL.
   - What's unclear: should builder order matter (caller must call `content_type` before `with_alias`)? Or should the registry resolve aliases lazily at serve time?
   - Recommendation: enforce builder order at runtime. `with_alias` snapshots the current hashed_url at call time. Document in rustdoc: "Call `.content_type(...)` before `.with_alias(...)`. The alias points to the URL at the moment of registration."

3. **Synthetic `Request` construction in `ferro-bundle` unit tests — what's the supported pattern?**
   - What we know: `Request::new` requires `hyper::Request<hyper::body::Incoming>` which is hyper-internal.
   - What's unclear: whether `framework::testing` module exposes a public Request constructor for cross-crate test use.
   - Recommendation: planner checks `framework::testing::*` exports during plan phase. If absent: extract a private serve helper `fn serve_inner(path: &str, if_none_match: Option<&str>) -> HttpResponse` that doesn't take a Request, have `Bundle::serve` delegate; unit-test the helper. Integration tests stay simple.

4. **`Bundle::new(name, bytes)` panic strategy — match D-06 strictly?**
   - What we know: D-06 says "panic on duplicate name." Includes the identical-bytes case (re-registration with same bytes).
   - What's unclear: this is correct for production code but unfriendly to hot-reload or repeated module init scenarios.
   - Recommendation: hold the line on D-06's strict panic. Mention in the rustdoc that `Bundle::new` is intended for boot-time registration only; re-registration is a bug. If hot-reload becomes a real use case, a future phase adds a `Bundle::reregister_or_new()` variant.

5. **Wave numbering for the new publish step — Wave 2.5? Wave 3? Subsume into Wave 3?**
   - What we know: ferro-bundle depends on `ferro-rs` (published in Wave 2). It cannot publish in Wave 1A or 1B.
   - What's unclear: the planner's preference for diff minimization.
   - Recommendation: insert a new publish step right after Wave 2's index-wait (line 295) and before Wave 3 (line 297) — call it "Wave 2.5: ferro-bundle". Smallest semantic stretch. Alternative: append to Wave 3 (already has `ferro-cli`, another `ferro-rs` consumer).

---

## Sources

### Primary (HIGH confidence — direct file read this session)

- `/Users/alberto/repositories/albertogferrario/ferro/framework/src/http/response.rs` — full file read (702 lines). `HttpResponse::bytes` (line 48), `.header()` replace semantics (line 121), `Redirect::to/permanent` (line 212, 257), `From<Redirect> for Response` (line 278). All HTTP shaping primitives verified.
- `/Users/alberto/repositories/albertogferrario/ferro/framework/src/http/request.rs` — read lines 1-100, 100-160, 300-316. `Request::new` constructor (line 55), `req.path()` (line 121), `req.header(name)` (line 309).
- `/Users/alberto/repositories/albertogferrario/ferro/framework/src/static_files.rs` — full file read (211 lines). Filesystem static handler with `bust_asset_urls` cache differentiation (line 55).
- `/Users/alberto/repositories/albertogferrario/ferro/framework/Cargo.toml` — read lines 1-77. Crate name `ferro-rs` (line 2), `sha2 = "0.10"` (71), `dashmap = "6"` (63).
- `/Users/alberto/repositories/albertogferrario/ferro/framework/src/lib.rs` — read lines 1-50, grep for `HttpResponse|Request` (line 108-114 confirms re-exports).
- `/Users/alberto/repositories/albertogferrario/ferro/.github/workflows/publish.yml` — full file read (321 lines). Wave 1a (line 211), 1b (line 246), 2 (line 274, `ferro-rs` present), 3 (line 297).
- `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml` (workspace root) — full file read (30 members listed, version `0.2.42`).
- `/Users/alberto/repositories/albertogferrario/ferro/Cargo.lock` — grep for sha2, hex, dashmap. Versions verified: `sha2 0.10.9`, `hex 0.4.3`, `dashmap 6.1.0`.
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-wallet/Cargo.toml` — full file read.
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-wallet/src/error.rs` — full file read (canonical `thiserror` + per-variant tests pattern).
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-wallet/src/lib.rs` — read lines 1-23.
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-storage/Cargo.toml` — full file read (sibling-crate template).
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-storage/src/lib.rs` — read lines 1-80.
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-ai/Cargo.toml` — full file read.
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cache/Cargo.toml` — full file read.
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-macros/Cargo.toml` — full file read (only existing in-repo consumer of `ferro-rs = { path = "../framework" }`).
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/plugin.rs` — read lines 1-30, grep for `OnceLock`. Pattern at line 147.
- `/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/151-ferro-wallet-crate/151-01-scaffold-PLAN.md` — full file read (367 lines). Reference scaffold-plan structure.
- `/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/182-ferro-json-ui-data-lazy-hero-runtime-primitive/182-RESEARCH.md` — read lines 1-80. Style/voice reference.
- `/Users/alberto/repositories/albertogferrario/ferro/.planning/ROADMAP.md` — read lines 1-100, 1970-2010. Phase 183 + v12.2 milestone section.
- `/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/183-ferro-bundle-capability-new-crate/183-CONTEXT.md` — full file read.

### Secondary (MEDIUM confidence)

- Project memory `feedback_friction_loop_release_cadence.md` (referenced via CONTEXT.md canonical_refs) — single-publish-at-end-of-phase pattern.
- Project memory `project_ferro_publish_token_scoping.md` (referenced via CONTEXT.md canonical_refs) — CI publish token scope.
- `RFC 7232 §2.3` (Strong ETag format), `§4.1` (304 response header requirements) — cited but not re-fetched this session.
- `RFC 8246` (Cache-Control `immutable` directive) — cited but not re-fetched.

### Tertiary (LOW confidence — none in this research)

(No claims rely on unverified sources. All HIGH-MEDIUM categories cover the research surface.)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every version verified against `Cargo.lock` this session.
- Architecture (registry pattern + HTTP shaping): HIGH — `ferro-json-ui/src/plugin.rs` OnceLock pattern and `framework/src/http/response.rs` bytes-builder both read directly.
- Pitfalls: HIGH (Pitfalls 1, 3, 4 are RFC-derived); MEDIUM (Pitfall 6 — Request construction depends on Open Question 3 resolution).
- Wave assignment correction (Risks §1): HIGH — direct read of publish.yml confirms `ferro-rs` is in Wave 2.

**Research date:** 2026-06-06
**Valid until:** 2026-07-06 (30 days — stable domain, single-purpose new crate)

## RESEARCH COMPLETE
