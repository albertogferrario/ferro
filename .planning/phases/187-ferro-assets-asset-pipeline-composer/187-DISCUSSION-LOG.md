# Phase 187: ferro-assets — Asset Pipeline Composer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-07
**Phase:** 187-ferro-assets-asset-pipeline-composer
**Mode:** `--auto` (all gray areas auto-selected; recommended option chosen per area)
**Areas discussed:** Crate placement & deps, Asset/content-type model, Transform trait shape, Execution model & image concurrency, Image formats & responsive variants, HTML-minify inline safety, Injection & token substitution, Failure semantics

---

## Crate placement, dependencies & publish wave

| Option | Description | Selected |
|--------|-------------|----------|
| Pure leaf crate, zero ferro-* deps, Wave 1a | Operates only on bytes; no storage/HTTP/DB | ✓ |
| Depend on ferro-storage (like 186), Wave 1b | Would let pipeline upload directly | |
| Depend on ferro-rs/framework (like 183), Wave 3 | Would let it serve HTTP | |

**Auto-selected:** Pure leaf, Wave 1a. **Rationale:** Success criteria describe transforms over an in-memory file set only — no upload, no HTTP, no DB. Storage upload and promote are the consumer's job (gestiscilo Phase 189). Zero ferro-* deps is the correct dependency posture and makes it parallel-capable with Phase 186. Deps pinned per gestiscilo v7.1-STACK D-03; zero C system deps (libvips rejected).

---

## Asset representation & content-type model

| Option | Description | Selected |
|--------|-------------|----------|
| `Asset { path, content_type, bytes }`, type inferred from extension | Logical-path + bytes::Bytes + ext-inferred type w/ override | ✓ |
| Raw `(String, Vec<u8>)` tuples, type sniffed from content | Lighter, but no clean passthrough gating | |

**Auto-selected:** `Asset` struct, extension-inferred content type with override, unknown → passthrough. **Rationale:** A typed content-type with an "other" catch-all is what drives the byte-identical passthrough guarantee (criterion 1). `bytes::Bytes` for cheap clones across transforms.

---

## Transform trait shape & passthrough semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Whole-set: `run(Vec<Asset>) -> Result<Vec<Asset>>` + map helper | Later transforms cross-reference earlier outputs; stateless | ✓ |
| File-by-file: `apply(Asset) -> Result<Vec<Asset>>` | Simpler, but responsive_images can't see image variants | |
| File-by-file + shared mutable pipeline context | Enables cross-ref, but adds shared mutable state | |

**Auto-selected:** Whole-set model with a content-type-gated mapping helper. **Rationale:** `responsive_images` must reference variants `image_transcode` emitted; whole-set keeps that stateless. Helper keeps simple minifiers simple while declaring accepted types; non-matching files pass through byte-identical.

---

## Execution model & bounded image concurrency

| Option | Description | Selected |
|--------|-------------|----------|
| Synchronous `run()`, no tokio; image bound via thread pool | Caller wraps whole run in spawn_blocking | ✓ |
| Async `run()` with tokio Semaphore | Pulls a runtime into a Wave 1a leaf | |

**Auto-selected:** Synchronous `run()`, no async runtime; image_transcode concurrency bound configurable default ≤2 via a CPU thread pool. **Rationale:** gestiscilo PITFALLS A-04 — pipeline is synchronous CPU work the consumer wraps in `spawn_blocking`; C-03 — encodes must be bounded to avoid OOM on a 512MB instance. Keeps the leaf crate dependency-light.

---

## Image output formats & responsive variants

| Option | Description | Selected |
|--------|-------------|----------|
| AVIF (ravif) + JPEG fallback only | Pure Rust, no C deps; matches ASSET-F-03 | ✓ |
| AVIF + WebP + JPEG (gestiscilo original) | Lossy WebP needs C libwebp-sys; lossless redundant | |

**Auto-selected:** AVIF + JPEG only; widths configurable default [480,768,1200,1920], never upscale; deterministic variant naming `{stem}-{width}w.{ext}`; `<picture>` with `<source type=image/avif srcset>` + JPEG `<img>` fallback. **Rationale:** The ferro roadmap criterion narrowed gestiscilo's AVIF+WebP+JPEG to AVIF+JPEG; pure-Rust codecs keep zero C deps (criterion 3). WebP deferred.

---

## HTML minify inline-content safety

| Option | Description | Selected |
|--------|-------------|----------|
| Treat `<script>`/`<style>` bodies as opaque | lol_html handlers never touch inner text | ✓ |
| Full whitespace minification including script bodies | Smaller output, but corrupts inline JS | |

**Auto-selected:** Opaque `<script>`/`<style>` bodies; regression fixture from a real tenant site proves byte-correct preservation. **Rationale:** gestiscilo PITFALLS C-02 — inline scripts with template literals/JSON get corrupted by naive minification → `SyntaxError` live. This is criterion 2 and the single most failure-prone transform.

---

## Injection & token substitution built-ins

| Option | Description | Selected |
|--------|-------------|----------|
| `inject_before_tag` (lol_html) + `replace_tokens` (raw bytes) | Structural inject vs textual token replace, each correct | ✓ |
| Single lol_html transform doing both | Token replace via lol_html can't reach JS/text bodies | |

**Auto-selected:** Two built-ins — `inject_before_tag(tag, snippet)` via lol_html; `replace_tokens(map)` via byte-safe raw string substitution. SEO injection explicitly OUT (stays consumer serve-time, gestiscilo C-01). **Rationale:** `%%TOKEN%%` placeholders appear in attributes/JS/text, so token replace must be raw-byte; structural injection is correctly a lol_html pass (criterion 4).

---

## Failure semantics / atomicity

| Option | Description | Selected |
|--------|-------------|----------|
| All-or-nothing: any failure → structured Err, no partial set | Caller builds two-phase upload on this | ✓ |
| Best-effort: return successful files + per-file errors | Risks partial promote downstream | |

**Auto-selected:** `run()` all-or-nothing; one thiserror `Error` enum with per-file + per-transform context; no partial output set. **Rationale:** criterion 5; gestiscilo PUB-05 / PITFALLS C-04 — the consumer's two-phase WRITE→PROMOTE invariant requires an atomic in-memory result to build on. The crate never touches storage.

---

## Claude's Discretion

- Exact `Asset.content_type` representation and detection table.
- Exact `Transform` trait signature/ownership and convenience-helper API.
- Bounded-concurrency primitive (rayon vs std threads) — default 2, no tokio.
- Exact responsive variant naming format (must round-trip).
- Exact swc sub-crate minor versions.
- Builder shape for pipeline tuning (widths, concurrency, AVIF quality).

## Deferred Ideas

- Lossy/WebP output; oxc_minifier; critical-CSS extraction; Tier 2 Node pipeline;
  streaming/on-disk pipeline; ferro-mcp asset-pipeline introspection tool.
