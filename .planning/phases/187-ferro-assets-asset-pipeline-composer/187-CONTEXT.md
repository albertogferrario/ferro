# Phase 187: ferro-assets — Asset Pipeline Composer - Context

**Gathered:** 2026-06-07 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

New **pure leaf crate** `ferro-assets` providing a composable, content-type-aware asset
pipeline for publish-time optimization over an in-memory set of files: HTML/CSS/JS
minification, pure-Rust image transcoding with responsive variants, `<img>`→`<picture>`
rewriting, and generic tag injection / token substitution. This is the Tier 1 pipeline
gestiscilo's `PublishFrontendJob` composes (gestiscilo Phase 189).

**Killer feature:** the pipeline is a **content-type router with a byte-identical
passthrough guarantee** — one `Pipeline` runs over a heterogeneous artifact set (HTML,
CSS, JS, images, *and unknown files like JSON-UI spec bundles*) and each file only
touches the transforms that accept its content type; everything else passes through
byte-for-byte. This passthrough guarantee (criterion 1, proved by running a JSON file
through the full HTML/CSS/JS/image pipeline untouched) is what makes the crate
artifact-shape agnostic and lets the same pipeline serve static HTML sites, JSON-UI
bundles, and SSR manifests — the v12.3 milestone's consumer-agnostic design constraint.

**What this crate is NOT** (scope anchors — these belong to the consumer, not ferro-assets):
- No template rendering (MiniJinja `{% extends %}`/`{% include %}` resolution happens in
  the consumer *before* the pipeline runs).
- No storage upload — the pipeline returns an in-memory `Vec<Asset>`; the caller uploads.
  The two-phase WRITE→PROMOTE all-or-nothing upload invariant (gestiscilo PUB-05 /
  PITFALLS C-04) is the *consumer's* job, built on this crate's atomic in-memory result.
- No job orchestration, no CDN purge (Phase 188), no deployment row writes (Phase 186).
- No SEO/meta injection — SEO stays a consumer serve-time concern (gestiscilo PITFALLS
  C-01: moving it into the publish pipeline would double meta tags).

Requirements: ASSET-F-01, ASSET-F-02, ASSET-F-03, ASSET-F-04. Consumer: gestiscilo
Phase 189 (`PublishFrontendJob` composes the Tier 1 pipeline). Parallel-capable with
Phase 186 (no inter-dependency).

</domain>

<decisions>
## Implementation Decisions

### Crate placement, dependencies & publish wave
- **D-01:** `ferro-assets` is a **pure leaf crate with ZERO `ferro-*` dependencies** —
  it operates only on bytes. This is the key difference from sibling Phase 186
  (ferro-deployments depends on ferro-storage → Wave 1b) and Phase 183 (ferro-bundle
  depends on ferro-rs → Wave 3). ferro-assets has no storage, no HTTP, no DB — only
  external transform crates. **Publish Wave 1a** (alongside ferro-macros, ferro-events,
  ferro-storage, etc.). Add to `.github/workflows/publish.yml` `WAVE1A_CRATES` and to the
  workspace `members` list in `Cargo.toml`.
- **D-02:** Dependency pins (from gestiscilo v7.1-STACK D-03, the locked stack research):
  `lol_html = "2.6"`, `lightningcss = "=1.0.0-alpha.71"` (EXACT pin — alpha releases have
  breaking changes between minor bumps; criterion 2 names this version), `swc_ecma_minifier`
  (~0.203) + `swc_ecma_parser` + `swc_ecma_codegen` + `swc_common` (NOT the `swc_core`
  umbrella — bloats compile time), `image = "0.25"`, `ravif` (pure-Rust AVIF via rav1e),
  `thiserror = "2"`, `bytes = "1"`. Exact swc sub-crate minor versions are planner
  discretion (check docs.rs at plan time). **`cargo build` must introduce ZERO new C
  system dependencies** (criterion 3) — every codec is pure Rust; libvips is rejected
  (its `VipsImage` is documented thread-unsafe, fatal inside a tokio `spawn_blocking`
  worker, and it adds a C lib to the production image).
- **D-03:** **No `from_env()` config struct, no app-identity fields.** The project-agnostic
  crates rule (CLAUDE.md) requires `from_env()` only when a crate needs app identity
  (`APP_NAME`/`APP_URL`). ferro-assets needs none — SDK injection snippets and token
  maps are passed by the caller, never read from env. Reviewers should NOT flag the
  absence of a `from_env()` config here. (Pipeline tuning — widths, concurrency bound —
  lives on builder methods, not env.)
- **D-04:** No `tokio`/async-runtime dependency. See D-08 (synchronous API).

### Asset representation & content-type model
- **D-05:** Core data type `Asset { path: String, content_type: <type>, bytes: bytes::Bytes }`.
  `path` is a logical artifact path (e.g. `assets/hero.jpg`, `index.html`); `bytes` uses
  `bytes::Bytes` for cheap clones across transforms. Exact `content_type` representation
  (a small `ContentType` enum covering html/css/js/jpeg/png/avif/other vs a `mime` string)
  is planner discretion — must distinguish the transform-relevant types and have an
  "other/unknown" catch-all that drives passthrough.
- **D-06:** Content type is **inferred from the path extension** on ingest, with an
  explicit per-asset override available. Unknown/unmatched extensions → the "other"
  catch-all → byte-identical passthrough.

### Transform trait shape & passthrough semantics
- **D-07:** The `Transform` trait's core method operates over the **whole asset
  collection**, not file-by-file in isolation: `run(&self, assets: Vec<Asset>) ->
  Result<Vec<Asset>, Error>` (exact signature/ownership planner discretion). Rationale:
  `responsive_images` must cross-reference the image variants that `image_transcode`
  emitted earlier in the chain — a whole-set model makes this stateless (the later
  transform discovers variants already present in the set) instead of requiring a shared
  mutable pipeline context. Provide a convenience helper (e.g. `map_matching(types, fn)`)
  so simple per-file transforms (minifiers) stay simple while declaring their accepted
  content types; files outside a transform's accepted types pass through byte-identical
  (criterion 1). `Pipeline::new().add(transform)…run(files)` applies transforms in
  insertion order.

### Execution model & bounded image concurrency
- **D-08:** **`Pipeline::run()` is synchronous (blocking).** ferro-assets pulls in no
  async runtime; the consumer wraps the entire `pipeline.run()` call in
  `tokio::task::spawn_blocking` (gestiscilo PITFALLS A-04: pipeline transforms are
  synchronous CPU work; calling them on the async executor stalls every HTTP request).
  Keeping the crate sync also keeps a Wave 1a leaf dependency-light.
- **D-09:** `image_transcode` bounds concurrent encodes to a **configurable limit,
  default ≤2** (criterion 3; gestiscilo PITFALLS C-03 — unbounded encodes OOM a small
  instance). Because `run()` is sync, the bound is enforced with a CPU thread pool
  (e.g. `rayon` with a sized pool, or `std::thread` + a counting gate) — exact primitive
  is planner discretion, but it must NOT require a live tokio runtime. The peak-memory
  target: stay bounded on a 512 MB instance.

### Image transcode & responsive variants
- **D-10:** Output formats: **AVIF (`ravif`) + JPEG fallback only.** WebP is explicitly
  OUT of v1 — the ferro roadmap criterion ASSET-F-03 narrowed gestiscilo's original
  AVIF+WebP+JPEG to AVIF+JPEG (lossy WebP needs the C `libwebp-sys`; lossless WebP via
  `image-webp` is large and redundant once AVIF ships). WebP is a deferred idea.
- **D-11:** Responsive widths are **configurable, default `[480, 768, 1200, 1920]`**; only
  emit widths `<= source.width()` (never upscale). Resize via `image::imageops` Lanczos3.
- **D-12:** **Deterministic variant naming** so `responsive_images` can discover variants
  from the asset set without shared state: scheme `{stem}-{width}w.{ext}` (e.g.
  `hero-768w.avif`) — exact format planner discretion, but it must be parseable back into
  (stem, width, format) by the rewriter.
- **D-13:** `responsive_images` is a `lol_html` rewriter that transforms each `<img src>`
  into `<picture><source type="image/avif" srcset=…><img …(JPEG fallback)></picture>`,
  referencing the emitted variants discovered in the asset set. Runs AFTER
  `image_transcode` in the chain.

### HTML minify inline-content safety (the regression-critical transform)
- **D-14:** `html_minify` (lol_html) MUST treat the text content of `<script>` and
  `<style>` elements as **opaque** — never collapse or rewrite whitespace inside them
  (gestiscilo PITFALLS C-02: inline scripts with template literals / multi-line strings /
  JSON blobs get corrupted by naive minification, producing `SyntaxError` on the live
  site). Configure lol_html `ElementContentHandlers` to leave `<script>`/`<style>` bodies
  untouched. **Proof artifact (criterion 2):** a regression fixture from a real tenant
  site (jetskiadriatic inline `<script>` with template literals + an inline `<style>`)
  asserts the minified output has byte-correct script/style bodies.

### Injection & token substitution built-ins
- **D-15:** `inject_before_tag(tag, snippet)` — a `lol_html` transform that inserts a
  snippet immediately before a given tag (primary use: SDK `<script>` before `</body>`)
  (criterion 4 / ASSET-F-04).
- **D-16:** `replace_tokens(map)` — a separate **byte-safe raw string substitution**
  transform for `%%TOKEN%%`-style placeholders. Done on raw bytes (NOT via lol_html)
  because tokens can appear anywhere — attribute values, inline JS, text. Criterion 4
  calls injection "string-substitution safe (used for `%%TOKEN%%`-style replacement)";
  splitting inject (structural, lol_html) from token-replace (textual, raw) keeps each
  correct for its job.

### Failure semantics / atomicity
- **D-17:** `Pipeline::run()` is **all-or-nothing**: any transform/file failure returns a
  structured `Error` and produces **NO partial output set** (criterion 5). Because the
  pipeline is in-memory, "no partial output" is natural — on error you simply don't
  return the `Vec<Asset>`. One `thiserror` `Error` enum, carrying per-file + per-transform
  context so the caller can report which file failed in which stage. The consumer's
  two-phase upload (gestiscilo PUB-05) builds its all-or-nothing upload guarantee on top
  of this atomic result; the crate itself never touches storage.

### New-crate workspace chores
- **D-18:** Add to workspace `members` (Cargo.toml) and `.github/workflows/publish.yml`
  `WAVE1A_CRATES`. First publish requires a **one-time manual `cargo publish -p
  ferro-assets`** from a local terminal (CI publish token has `publish-update` only, not
  `publish-new` — same as Phase 183/186 new crates). Add a docs page in `docs/src/` +
  SUMMARY.md entry. Cargo.toml metadata mirrors sibling new crates (license/edition
  workspace inherit, repository, keywords, categories, README).

### Claude's Discretion
- Exact `Asset.content_type` representation (enum vs mime string) and detection table.
- Exact `Transform` trait signature/ownership (`Vec<Asset>` by value vs `&mut`), and the
  convenience-helper API surface for content-type-gated mapping.
- Bounded-concurrency primitive (rayon sized pool vs std threads + counting gate) — only
  constraint: default 2, configurable, no tokio runtime requirement.
- Exact responsive variant naming format (must round-trip to stem/width/format).
- Exact swc sub-crate minor versions (verify on docs.rs at plan time).
- Builder-method shape for pipeline tuning (widths, concurrency, AVIF quality).
- Whether `<picture>` rewrite includes width/sizes attributes or just type+srcset in v1.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Locked design & stack (gestiscilo v7.1 — source of this milestone)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-STACK.md` §"D-03 — Tier 1 Asset Pipeline" — the authoritative crate selection: lol_html 2.6, lightningcss =1.0.0-alpha.71 (API pattern + pinning rationale), swc_ecma_minifier (+ parser/codegen/common, NOT swc_core), **image crate + ravif instead of libvips** (§4, with the thread-safety + C-dep rationale), responsive-widths code sketch, Cargo.toml additions
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-PITFALLS.md` §"Section C — Asset Pipeline (Phase 189)" — C-01 (SEO injection stays serve-time, OUT of pipeline), **C-02 (html_minify must treat `<script>`/`<style>` as opaque)**, C-03 (memory/semaphore bound ≤2), **C-04 (all-or-nothing, no partial output)**; §A-04 (pipeline is sync CPU work → caller wraps in spawn_blocking)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-ARCHITECTURE.md` §"D-03 — Build pipeline: Tier 1" (the 8-step pipeline ordering), "Ferro-side primitives" table (`ferro-assets` deliverable definition), §"content-type aware" note (JSON-UI specs pass through unchanged)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-INTEGRATION.md` §"Phase 189" — how `PublishFrontendJob` composes the pipeline (the consumer contract this crate must satisfy)

### Ferro repo
- `.planning/ROADMAP.md` §"v12.3 Deployment Platform Primitives" → "Phase 187" — requirements ASSET-F-01..04, the 5 success criteria (the acceptance contract), build-order/parallelism notes
- `.planning/phases/186-ferro-deployments-immutable-deployments-atomic-promote/186-CONTEXT.md` — sibling new-leaf-crate precedent (publish wave registration, manual-first-publish, thiserror/serde/builder conventions, docs-page requirement)
- `ferro-bundle/Cargo.toml`, `ferro-deployments/Cargo.toml` — new-crate Cargo.toml templates (workspace inherit, metadata block)
- `.github/workflows/publish.yml` — Wave structure; ferro-assets goes in `WAVE1A_CRATES`
- `.planning/codebase/CONVENTIONS.md`, `.planning/codebase/TESTING.md` — workspace conventions and test patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-bundle` (Phase 183) and `ferro-deployments` (Phase 186) — the two most recent
  new-crate precedents: Cargo.toml metadata shape, README, docs page, publish.yml wave
  entry, one-time manual first publish.
- The Phase 185/186 cfg-gated test pattern (always-on default test + feature-gated
  heavier test) — applicable if any transcode test is too heavy for default CI.

### Established Patterns
- New crate: workspace `members` entry + publish.yml wave + `docs/src/` page + SUMMARY.md.
- Error types: `thiserror`, ONE `Error` enum per crate. Serde enums `snake_case` (if any
  serde types appear). Consuming `with_*`/builder methods.
- Project-agnostic rule: `from_env()` only when a crate needs app identity — **N/A here**
  (D-03); do not add a config-from-env surface.
- CPU-heavy work runs under the consumer's `spawn_blocking`; the library stays synchronous.

### Integration Points
- `Cargo.toml` workspace `members` — add `ferro-assets`.
- `.github/workflows/publish.yml` `WAVE1A_CRATES` — add `ferro-assets` (zero ferro-* deps).
- First publish: one-time manual `cargo publish -p ferro-assets` (CI token lacks publish-new).
- `docs/src/` — new feature page + SUMMARY.md entry (mandatory).
- Consumer: gestiscilo Phase 189 adds `ferro-assets = "…"` to its Cargo.toml and composes
  the pipeline inside `PublishFrontendJob::handle` (wrapped in `spawn_blocking`).

</code_context>

<specifics>
## Specific Ideas

- **Passthrough proof (criterion 1):** a JSON (or JSON-UI spec) file run through the full
  HTML/CSS/JS/image pipeline comes out byte-identical — this is the artifact-agnostic
  guarantee and the conceptual core of the crate.
- **Inline-script regression fixture (criterion 2):** lift a real fragment from the
  jetskiadriatic tenant site — an inline `<script>` with template literals + a multi-line
  string + a JSON blob, plus an inline `<style>` — and assert byte-correct bodies after
  `html_minify`. This is the single most failure-prone transform.
- **Zero-C-deps as a feature:** `cargo build` adding no system packages is an explicit
  acceptance criterion (criterion 3) and a deliberate improvement over gestiscilo's
  original libvips plan — worth stating in the docs page.
- **Pipeline ordering** the consumer composes (gestiscilo D-03):
  html_minify → css_minify → js_minify → image_transcode → responsive_images →
  inject_before_tag → replace_tokens. The crate doesn't hardcode this order — the
  consumer adds transforms in sequence — but the built-ins must compose correctly in it.

</specifics>

<deferred>
## Deferred Ideas

- **Lossy/WebP output** — AVIF+JPEG ships now; WebP (esp. lossy via `libwebp-sys`)
  reconsidered only if AVIF coverage proves insufficient (gestiscilo notes v7.2).
- **`oxc_minifier`** instead of swc — faster, but its Rust API is less stable as of 2026;
  revisit for a later version (gestiscilo STACK §3).
- **Critical-CSS extraction/inline** — gestiscilo D-03 step 2 mentions it, but the ferro
  ASSET-F-02 criterion is minify-only; critical-CSS is a future transform.
- **Tier 2 (Node sandbox) pipeline** for Vite/Astro/Next/SvelteKit — opens only when a
  tenant generation pipeline emits framework code (gestiscilo D-03); entirely out of scope.
- **Streaming / on-disk pipeline** for very large artifact sets — v1 is in-memory
  (`Vec<Asset>`); a streaming variant is a future concern if memory becomes a constraint
  beyond the image-encode semaphore.
- **ferro-mcp asset-pipeline introspection tool** — natural once the framework integrates
  the crate; not required for a leaf-crate phase.

</deferred>

---

*Phase: 187-ferro-assets-asset-pipeline-composer*
*Context gathered: 2026-06-07 (auto mode)*
