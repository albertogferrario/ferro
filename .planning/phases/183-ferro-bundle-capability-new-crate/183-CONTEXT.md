---
phase: 183
name: ferro-bundle capability (new crate)
status: Ready for planning
gathered: 2026-06-06
discovered-by: gestiscilo /embed/v1.js SDK-10 caching audit (2026-06-06)
mode: auto
---

# Phase 183: `ferro-bundle` capability (new crate) — Context

<domain>
## Phase Boundary

Ship a new top-level workspace crate `ferro-bundle` for serving in-memory immutable byte blobs over HTTP with content-hashed URLs and one-year `max-age` caching. The crate provides a single `Bundle` type with the API shape locked by the roadmap:

```rust
Bundle::new("embed-v1", BYTES)
    .content_type("application/javascript")
    .with_alias("/embed/v1.js")
    .hashed_url();              // -> "/bundles/embed-v1.{8hex}.js"
// ...elsewhere, inside a request handler:
Bundle::serve(req)              // -> HttpResponse with cache headers + 304 + 301 alias support
```

The crate lives alongside the existing filesystem static-file handler — two parallel asset-serving paths, intentionally not folded. Filesystem path: mutable tenant assets, freshness via `bust_asset_urls` timestamp. Bundle path: symbolic immutable blobs, freshness via content hash. The split is documented in the crate README so a future contributor does not collapse them.

In scope:
- New workspace crate `ferro-bundle/` (Cargo.toml + src/lib.rs + README.md).
- Public API: `Bundle::new(name, bytes)`, `.content_type(ct)`, `.with_alias(path)`, `.hashed_url()`, `Bundle::serve(req) -> HttpResponse`.
- Content-hashed URL format `/bundles/{name}.{sha8}.{ext}` derived from SHA-256 of bytes.
- HTTP response: `Cache-Control: public, max-age=31536000, immutable`, strong `ETag` header (full SHA-256 hex, quoted), 304 fast-path on `If-None-Match` exact match.
- Alias mechanism: `.with_alias("/embed/v1.js")` registers a plain URL that 301-redirects to the current hashed URL.
- Process-global registry so `Bundle::serve(req)` can dispatch by request URL path (the API takes only `req`, so registry lookup is the only viable shape).
- Unit tests covering hash determinism, ETag format, 304 fast-path, 301 alias redirect.
- README that documents the bundle-vs-filesystem split.
- Cargo workspace integration: add to `Cargo.toml` `workspace.members`.
- `.github/workflows/publish.yml` integration: add to the appropriate publish wave.
- First publish bootstrap from local terminal (CI token cannot create new crates per project memory `project_ferro_publish_token_scoping.md`).

Out of scope:
- Runtime-mutable bundle bytes — the locked API signature is `bytes: &'static [u8]`, matching `include_bytes!` flow. Dynamic bytes deferred.
- Pre-deflated variants per `Accept-Encoding` (gzip/br). Deferred — bundles served as-is, downstream CDN handles compression.
- Stream serving for very large bundles — bundles fit in memory by design (SDK JS, fonts, etc.).
- Asset "manifests" or composite bundles. Deferred.
- Content-type sniffing from bytes or filename extension — caller provides at registration time per locked Success Criterion #4.
- Replacing the filesystem static-file handler. Two parallel paths is the design (Phase Boundary).

</domain>

<decisions>
## Implementation Decisions

### D-01: Bundle storage type — `&'static [u8]` only
The roadmap's locked API signature is `Bundle::new(name: &str, bytes: &'static [u8])`. The crate targets compile-time-included bytes via `include_bytes!("…")` — the natural Rust idiom for "ship this asset with the binary." Runtime-loaded bytes (file read at startup, downloaded at runtime, etc.) are out of scope. If a future consumer needs runtime bytes, a follow-up phase adds a `Cow<'static, [u8]>` or `Arc<[u8]>` variant; not Phase 183.

### D-02: Registry — process-global `OnceLock<DashMap<String, BundleEntry>>`
The public API `Bundle::serve(req) -> HttpResponse` takes only the request — no `&self`, no `&[Bundle]`. The only viable implementation is a process-global registry that `Bundle::serve` looks up by request URL path. Options considered:

| Option | API shape | Verdict |
|--------|-----------|---------|
| Process-global `OnceLock<DashMap<String, BundleEntry>>` keyed by URL path | `Bundle::serve(req)` looks up `req.path()` | ✅ Adopted — matches the locked API; DashMap allows concurrent reads cheaply |
| Per-`App` registration via `app.bundle(b)` | `Bundle::serve(req, &registry)` — but roadmap locks `serve(req)` only | ❌ Conflicts with locked API signature |
| `lazy_static`/`once_cell::Lazy` instead of `OnceLock` | Functionally equivalent | ❌ `std::sync::OnceLock` is std-stable (Rust 1.70+); avoids extra dep |

The registry stores: `path` (key) → `BundleEntry { name, bytes, content_type, sha256_hex, ext }`. Alias entries are stored in a parallel `OnceLock<DashMap<String, String>>` mapping `alias_path → current_hashed_path`.

### D-03: URL routing — `Bundle::serve(req)` dispatches via the registry
On each request, `Bundle::serve(req)`:

1. Extract `path = req.path()`.
2. If `path` is in the alias registry → return 301 redirect to the registered hashed URL.
3. If `path` is in the bundle registry → check `If-None-Match` header against the stored ETag; return 304 on match, else return 200 with bytes + cache headers + ETag.
4. Otherwise → return 404 (caller is responsible for routing only `/bundles/…` traffic to `Bundle::serve`, but defensive 404 covers misroutes).

The consumer is expected to wire `Bundle::serve` as the handler for `/bundles/{filename}` (catch-all) and for any registered alias paths. The framework provides standard route-registration; ferro-bundle does not own routing.

### D-04: Hash truncation — first 8 hex chars of SHA-256
The roadmap example `/bundles/embed-v1.{8hex}.js` locks the truncation length. 8 hex chars = 32 bits of entropy. Collision space ≈ 4.3 billion; for the practical range of bundles per application (dozens to low hundreds), collision risk is negligible. Documented in the README. Anyone needing more entropy can extend in v2 without breaking existing URLs.

### D-05: ETag format — strong, full SHA-256 hex, quoted
Per RFC 7232 §2.3: strong ETags are quoted opaque strings. Header value: `ETag: "{64-hex-chars}"`. Format:
- Strong (no `W/` prefix) — bytes are immutable, exact byte-for-byte equality is the relation.
- Full 64-hex-char SHA-256 (not the truncated 8-char URL hash). The URL hash is a cache-busting handle; the ETag is for content-integrity matching. They use the same algorithm but the ETag is full-precision.
- Quoted per spec: `"{hash}"`.

`If-None-Match` comparison is exact string match against the quoted ETag.

### D-06: Bundle registration — eager at `Bundle::new()` call site; panic on duplicate name
Bundle registration happens eagerly when `Bundle::new(name, bytes)` is called: the bytes are SHA-256-hashed, the entry is inserted into the registry. Subsequent `.content_type(ct)` and `.with_alias(path)` mutate the same registry entry. If the same `name` is registered twice with different bytes, `Bundle::new()` panics with a clear message — this is developer error caught at startup, not a runtime branch to handle silently.

Test isolation: provide a `#[cfg(test)] pub fn reset()` that clears the registry. Cargo runs each integration-test binary in its own process by default, so the global state is already isolated across test binaries; the `reset()` is for tests within the same binary that want clean state.

### D-07: Hash algorithm — SHA-256 (locked by roadmap)
SHA-256 is industry-standard for content addressing. Crate dep: `sha2`. Hex encoding: `hex` crate.

### D-08: Alias mechanism — stored on the Bundle, queried by `Bundle::serve`, 301 redirect
`.with_alias("/embed/v1.js")` adds the alias path to the global alias registry mapping `/embed/v1.js → /bundles/embed-v1.{sha8}.js` (the current hashed URL). On request, `Bundle::serve` checks alias registry FIRST (before bundle registry). 301 redirect is permanent — the alias path is a stable name that always resolves to the current hashed URL.

Multiple aliases per bundle are allowed (`.with_alias("/a.js").with_alias("/b.js")`). Each alias registers a separate entry pointing at the same hashed URL.

### D-09: Crate dependencies — minimal
- `sha2` — hashing (one of the most common crates; pinned to the workspace-shared version if one exists, otherwise `0.10`).
- `hex` — encoding (small, standard).
- `dashmap` — concurrent HashMap for the registry (already a transitive dep of multiple ferro crates).
- `framework` — for `HttpResponse` and `Request` types. This makes `ferro-bundle` a Wave1B publish (depends on `ferro-rs` which is `framework`'s published name).

Researcher confirms the wave assignment by reading `.github/workflows/publish.yml` and the dep graph. `framework` is the canonical ferro crate (`ferro-rs` on crates.io). Per CLAUDE.md "When adding a new crate to the workspace, always add it to `.github/workflows/publish.yml`" — Wave1B (after `ferro-rs` publishes in Wave1A).

### D-10: README required — documents bundle-vs-filesystem split
Per locked Success Criterion #5: the crate README documents the design split between `ferro-bundle` and the filesystem static-file handler:

- **`ferro-bundle`** — in-memory immutable byte blobs. Freshness model: content hash in URL. Cache lifetime: 1 year. Use for SDK bundles, embedded fonts/icons, versioned static assets.
- **filesystem static-file handler** — mutable assets on disk. Freshness model: `bust_asset_urls` timestamp query param. Cache lifetime: shorter, revalidated. Use for tenant-customizable CSS, theme assets, user uploads.

The README explicitly states "do not fold these — they target different freshness models" to prevent future contributors from refactoring them into one.

### D-11: Workspace + publish.yml integration
- `Cargo.toml` (root) `workspace.members` list grows by one: `"ferro-bundle"`.
- `.github/workflows/publish.yml` Wave1B (`WAVE1B_CRATES`) gets `ferro-bundle` appended.
- `framework/Cargo.toml` does NOT add a dep on ferro-bundle by default. Consumers opt into ferro-bundle explicitly. (Future phase may re-export through framework if the ergonomic case is strong.)

### D-12: First publish bootstrap — manual from local terminal
Per memory `project_ferro_publish_token_scoping.md`: the CI publish token has `publish-update` permission only, not `publish-new`. The CI workflow will fail to publish a brand-new crate on its first attempt. Bootstrap:

1. Author the crate, land Phase 183 on master.
2. From local terminal: `cargo publish -p ferro-bundle`. This creates the crate on crates.io under the maintainer's local token (which has full permissions).
3. Future versions ship via CI's Wave1B publish step automatically.

The phase plan includes a "manual bootstrap" task with explicit instructions; the executor does NOT attempt `cargo publish` in CI-mode.

### D-13: Test isolation via `#[cfg(test)] reset()` helper
Process-global state is the source-of-truth at runtime but a test-quality risk. Mitigation:

- `pub(crate) fn reset()` (visible in `#[cfg(test)]` only) clears both registries.
- Each unit test that registers bundles calls `reset()` at the top.
- Integration tests run in separate binaries (default `cargo test` behavior), so the OS already isolates them.

This keeps the runtime path simple (no per-context registry) while protecting test ergonomics.

### Claude's Discretion
- Exact crate metadata fields (`keywords`, `categories`, `description`) — follow sibling-crate template (`ferro-storage` Cargo.toml is the closest analog: leaf-ish crate with similar surface area).
- File layout under `ferro-bundle/src/` — single `lib.rs` is acceptable; can split into `bundle.rs` + `registry.rs` + `serve.rs` if the planner judges readability gain.
- Specific error types — `thiserror`-derived `Error` enum with variants like `DuplicateName`, `NotFound` (used internally; public surface is mostly infallible registration + `Result<HttpResponse, _>` on serve).
- Exact integration-test layout under `ferro-bundle/tests/` — follow the sibling pattern.

### Folded Todos
None — no GSD `todo match-phase 183` was run because the phase directory did not exist when `init phase-op` was invoked. If pending todos surface relevant to Phase 183 during planning, the planner can fold them into PLAN.md task descriptions at that point.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source — roadmap and prior decisions
- `.planning/ROADMAP.md` §`Phase 183: ferro-bundle capability (new crate)` — locked API shape, success criteria 1–6, discovery context, cross-tracked gestiscilo Phase 185.
- `.planning/PROJECT.md` — pre-1.0 status (breaking changes acceptable), v1.0 criteria (conceptual coherence, beauty in four dimensions).
- `.planning/phases/182-ferro-json-ui-data-lazy-hero-runtime-primitive/182-CONTEXT.md` — sibling phase that established the "ferro-* crate scope" framing (project-agnostic surface, no tenant identity, public-contract docs).

### Source — workspace integration patterns
- `Cargo.toml` (workspace root) — `[workspace.members]` is the list Phase 183 appends to.
- `Cargo.toml` `workspace.package.version` — currently `0.2.42` (from Phase 182). New crate inherits via `version.workspace = true`.
- `.github/workflows/publish.yml` `WAVE1A_CRATES` (leaf crates with no internal ferro deps) and `WAVE1B_CRATES` (crates with internal ferro deps). Phase 183's `ferro-bundle` depends on `framework` (published as `ferro-rs`) → Wave1B.
- `ferro-storage/Cargo.toml` — closest sibling template for a leaf-ish addon crate.
- `ferro-ai/Cargo.toml` — closest sibling template for a Wave1B crate with internal ferro deps (depends on `ferro-events`).

### Source — framework HTTP types
- `framework/src/lib.rs` — re-exports `HttpResponse`, `Request`, `FromRequest`. Public surface used by `Bundle::serve(req) -> HttpResponse`.
- `framework/src/http/` (directory) — request/response type definitions. `Bundle::serve` returns one of these.

### Source — contrasting freshness model
- Filesystem static-file handler — TBD location (researcher identifies). The contrast is documented in `ferro-bundle/README.md` per D-10. `bust_asset_urls` is the timestamp-based mutator on the filesystem path; ferro-bundle uses content hashes instead.

### Project conventions (CLAUDE.md)
- `CLAUDE.md` (project root) — "When adding a new crate to the workspace, always add it to `.github/workflows/publish.yml` in the correct wave (Wave 1 for leaf crates, Wave 2+ for crates with internal deps)." → Phase 183 adds `ferro-bundle` to Wave1B.
- `CLAUDE.md` (project root) — "Project-agnostic crates" rule. `ferro-bundle` is generic immutable-bundle infrastructure; no tenant identity, no hardcoded app strings.
- `CLAUDE.md` (project root) — "Always update docs when framework changes." README per D-10 satisfies this. No separate `docs/src/` page is required (bundle is a Rust API, not a JSON-UI authoring surface).
- `CLAUDE.md` (project root) — "Run fmt + clippy + tests before every commit."

### Project memory (referenced for behavior, not committed in the repo)
- Memory `project_ferro_publish_token_scoping.md` — CI publish token has publish-update only, not publish-new. First publish of `ferro-bundle` must be bootstrapped from local terminal. Drives D-12.
- Memory `feedback_friction_loop_release_cadence.md` — single publish at end of release loop. Phase 183 publishes once at merge; consumer (gestiscilo Phase 185) bumps after.
- Memory `feedback_breaking_changes_v12_ai.md` — pre-1.0, breaking changes acceptable. Not exercised in Phase 183 (purely additive).

### Discovery context
- Roadmap Phase 183 discovery note (2026-06-06): gestiscilo `/embed/v1.js` SDK bundle is forever-stable per SDK-10 contract but served today with `max-age=300, stale-while-revalidate=86400`. Content-hashed URL unlocks 1-year immutable caching. Cross-tracked as gestiscilo Phase 185 [FERRO REPO].

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Sibling Cargo.toml template** — `ferro-storage/Cargo.toml` shows the minimal-deps leaf-ish addon-crate shape. `ferro-ai/Cargo.toml` shows the Wave1B shape with an internal `ferro-events = { path = "../ferro-events", version = "0.2" }` dep. Phase 183 follows the Wave1B shape with `framework` as the internal dep.
- **Workspace inheritance** — `version.workspace = true`, `edition.workspace = true`, `license.workspace = true` are the standard inheritance pattern. Phase 183 follows verbatim.
- **`HttpResponse` builders** — `framework/src/lib.rs` macros `$crate::HttpResponse::json(...)`, `$crate::HttpResponse::text(...)` show the builder pattern. For ferro-bundle: `HttpResponse::bytes(bytes).content_type(ct).cache_control(...).etag(...).build()` or equivalent depending on framework's actual builder shape (researcher confirms).
- **`thiserror` for error types** — every ferro-* crate uses `thiserror::Error` derive. Phase 183 follows.
- **`DashMap` for concurrent registries** — already a transitive dep in multiple ferro-* crates (ferro-ai uses it). Direct dep on `dashmap = "6"` is consistent.

### Established Patterns
- **One Error enum per crate** — `pub enum Error { … }` with `thiserror::Error` derive, `pub type Result<T> = std::result::Result<T, Error>;`. Phase 183 follows.
- **Crate-level docs in `src/lib.rs`** — `//!`-style module-level documentation that mirrors the README's introduction. Future `cargo doc` reads from here.
- **`#[cfg(test)] mod tests` colocated with the implementation** — unit tests live alongside the code in `lib.rs` or submodule files. Integration tests live under `tests/`. Phase 183 follows.

### Integration Points
- **`framework::HttpResponse` and `framework::Request`** — `Bundle::serve(req: Request) -> HttpResponse` uses these. ferro-bundle's `Cargo.toml` adds `framework = { path = "../framework", version = "0.2" }` (or `ferro-rs` if the crates.io name is required for publish-time resolution — researcher confirms the convention used by sibling crates like ferro-ai which depends on `ferro-events`).
- **Consumer route registration** — the consumer wires `Bundle::serve` as the handler for `/bundles/{filename}` (catch-all) using ferro's standard route macros. Ferro-bundle does NOT own routing; the consumer chooses where to mount the bundle namespace.
- **Future re-export from `framework`** — out of scope for Phase 183 but worth noting: a future phase may re-export `Bundle` through `framework` for ergonomics, similar to how `framework` re-exports `ferro_json_ui` types today. Not required to ship Phase 183.
- **`ferro-mcp` introspection** — `ferro-bundle` is a runtime-only crate with no JSON-UI components, no MCP catalog entries. No `ferro-mcp` integration required. The crate's public Rust API is documented via `cargo doc`.

</code_context>

<specifics>
## Specific Ideas

- The roadmap example URL `/bundles/embed-v1.{8hex}.js` is the canonical shape: `{name}.{first-8-hex-of-sha256}.{ext}`. The `ext` is parsed from the content-type at registration: e.g., `"application/javascript"` → `js`; `"text/css"` → `css`; default → no extension. A small content-type-to-ext table inside `ferro-bundle` covers the common cases; unknown types fall back to no extension (URL becomes `/bundles/{name}.{8hex}`).
- The discovery context (gestiscilo SDK-10 contract + current `max-age=300, stale-while-revalidate=86400`) is the operational baseline `ferro-bundle` improves on. Concrete win: 1-year immutable caching for the SDK bundle, with 301 redirect from the plain URL ensuring backward-compat for sites that still hard-code the old `<script src="/embed/v1.js">`.
- Test ergonomics: pattern `assert_eq!(Bundle::new("x", b"hello").content_type("text/plain").hashed_url(), "/bundles/x.2cf24dba.txt");` — the hash is deterministic and citable in tests verbatim. SHA-256 of `"hello"` = `2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824`; first 8 chars = `2cf24dba`. Tests pin this so any future hash-algorithm drift fails loudly.
- The phrase "two parallel asset-serving paths intentional" in the roadmap is a load-bearing design assertion. The README must restate it. Future contributors with a tidying instinct may try to refactor into one path; the README is the architectural ward.
- The "publish bootstrap" friction (D-12) only happens once per crate. After the first `cargo publish` from local terminal, subsequent versions ship via CI Wave1B automatically. The bootstrap is a single command but easy to forget — the phase plan explicitly tasks it.

</specifics>

<deferred>
## Deferred Ideas

- **Runtime-mutable bundle bytes** — `Cow<'static, [u8]>` or `Arc<[u8]>` variant for bundles loaded at runtime (e.g., from a database or downloaded asset). Not Phase 183; the `&'static [u8]` API is sufficient for the discovery use case (compile-time `include_bytes!` for the SDK bundle).
- **Pre-deflated `Accept-Encoding` variants** — store gzip-/br-compressed copies alongside identity bytes and select based on the request's `Accept-Encoding` header. Future phase if downstream profiling shows the gain is worth the API expansion.
- **Composite bundles / manifests** — a `BundleManifest::new(...).add(bundle1).add(bundle2)` shape that serves multiple bundles under a versioned manifest. Speculative; ship when a real consumer asks.
- **Streaming serve for large bundles** — current design is bytes-in-memory. If a real consumer needs 100MB+ bundles, revisit. SDK JS / fonts / icons are well under this threshold.
- **Re-export `Bundle` through `framework`** — for ergonomic discoverability. Consumers today do `use ferro_bundle::Bundle;`. Future phase could add `framework::Bundle` re-export. Not required.
- **Content-type sniffing** — explicitly excluded by Success Criterion #4. Caller provides; default `application/octet-stream`.

### Reviewed Todos (not folded)
None — no GSD todo matching was performed for this phase (phase directory did not exist when `init phase-op` was invoked).

</deferred>

---

## Discovery Transcript (preserved from roadmap)

Roadmap Phase 183 discovery note, verbatim:

> Discovery: gestiscilo `/embed/v1.js` SDK bundle is forever-stable per the SDK-10 contract but served today with `max-age=300, stale-while-revalidate=86400` (adequate but not optimal). A content-hashed URL unlocks truly immutable caching with one-year `max-age`. Generic enough to live in ferro: any ferro app shipping versioned static asset bundles can reuse the same primitive. Cross-tracked as gestiscilo Phase 185 [FERRO REPO].

### Concrete consumer impact (gestiscilo Phase 185)

Pre-Phase-183 pattern:

```http
GET /embed/v1.js
Cache-Control: public, max-age=300, stale-while-revalidate=86400
```

Browsers revalidate after 5 minutes; CDN caches for ~1 day. SDK bundle bytes haven't changed but the URL is hit on every revalidation.

Post-Phase-183 pattern (gestiscilo Phase 185 adopts):

```http
GET /bundles/embed-v1.a4b8c2d1.js
Cache-Control: public, max-age=31536000, immutable
ETag: "a4b8c2d1f73e89..."

GET /embed/v1.js
HTTP/1.1 301 Moved Permanently
Location: /bundles/embed-v1.a4b8c2d1.js
```

The plain `/embed/v1.js` alias remains for backward-compat with tenant sites that hard-code the old URL. New tenants use the hashed URL directly and get year-long browser cache + CDN immutability.

---

*Phase: 183-ferro-bundle-capability-new-crate*
*Context gathered: 2026-06-06 (--auto)*
