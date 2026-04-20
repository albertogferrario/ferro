---
phase: 143
milestone: v11.7
status: ready-for-planning
created: 2026-04-20
updated: 2026-04-20
---

# Phase 143: Tailwind Static CSS Pipeline — Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the Tailwind CDN browser runtime (`@tailwindcss/browser@4`) with a pre-built static CSS file shipped inside the ferro binary. Apps serve this file from a framework-registered route and override theme tokens via plain CSS variable declarations. No Node toolchain required at any stage.

This phase does NOT include:
- A watch/rebuild dev loop for app-level Tailwind extensions
- A tailwind config file format / `@theme` compilation for apps
- Migrating apps beyond gestiscilo (that is a separate consumer phase)

</domain>

<decisions>
## Implementation Decisions

### CSS Production Mechanism
- **D-01:** Generate `ferro-base.css` by running the Tailwind v4 standalone CLI against all class-emitting source files in `ferro-json-ui/src/**/*.rs` and `framework/src/**/*.rs` plus a default theme file.
- **D-02:** Check the generated `ferro-base.css` into the repo (e.g. `ferro-json-ui/assets/ferro-base.css` or `framework/assets/ferro-base.css`). A CI step verifies the file is up-to-date (runs tailwind, diffs output). Users pulling the crate get the pre-built file — no tailwind CLI needed at compile time.
- **D-03:** Embed the checked-in file in the Rust binary at compile time via `include_str!("../assets/ferro-base.css")` or equivalent. No runtime file I/O.

### Static Route Registration
- **D-04:** The framework registers `GET /_ferro/ferro-base.css` automatically during app bootstrap — unconditional, no user configuration required. The embedded bytes are served with `Content-Type: text/css` and a long-lived `Cache-Control` header.
- **D-05:** No separate `ferro.use_json_ui()` init call is needed. The route is part of the standard framework boot sequence.

### Config API Shape
- **D-06:** Add `stylesheet_urls: Vec<String>` field to `JsonUiConfig`. Default: `vec!["/_ferro/ferro-base.css".to_string()]`. Each URL emits a `<link rel="stylesheet" href="...">` in `<head>`, in order.
- **D-07:** Builder method: `stylesheet_urls(mut self, urls: Vec<String>) -> Self` — replaces the entire list. This allows apps to inject their own stylesheet and optionally drop the default if desired.
- **D-08:** Theme token URL injection (app-level tokens.css) is done via `stylesheet_urls` — apps push their token file URL into the list. No separate field for theme CSS URLs.

### Theme Injection Replacement
- **D-09:** Remove `<style type="text/tailwindcss">` injection (currently done for active themes in the `#[cfg(feature = "theme")]` block). Replace with `<style>` containing plain CSS variable overrides.
- **D-10:** Theme CSS that uses Tailwind v4 `@theme` syntax must be converted to plain `:root { --color-background: ...; }` declarations before injection (or apps convert their tokens.css and pass it via `stylesheet_urls`). The runtime magic (`@theme` processing) is no longer available.
- **D-11:** Inline `<style>` injection (not a separate route) is acceptable for theme overrides — keeps the existing `theme.css` → `<style>` injection path, but with a plain `<style>` tag instead of `type="text/tailwindcss"`.

### CDN Fallback Handling
- **D-12:** Flip `JsonUiConfig::tailwind_cdn` default from `true` to `false`. This is a breaking change (expected pre-1.0).
- **D-13:** Keep `tailwind_cdn(true)` as an explicit opt-in. The CDN path remains functional — it is not deprecated or removed. Dev scaffolding or quick prototypes may still enable it deliberately.
- **D-14:** When `tailwind_cdn` is `true` AND `stylesheet_urls` contains the default ferro-base URL, both load. Callers that enable the CDN explicitly should clear `stylesheet_urls` if they want CDN-only mode. No automatic mutual-exclusion logic — KISS.

### Test Coverage
- **D-15:** Update existing tests that assert on `<style type="text/tailwindcss">` presence to match the new plain `<style>` tag.
- **D-16:** Add a test asserting `JsonUiConfig::default()` produces `<link rel="stylesheet" href="/_ferro/ferro-base.css">` and no Tailwind CDN `<script>` tag.
- **D-17:** Add a test that the embedded CSS bytes are non-empty and parseable as UTF-8.

### Claude's Discretion
- Exact asset path for the checked-in CSS file (e.g. `ferro-json-ui/assets/` vs `framework/assets/`)
- Cache-Control header value for the static CSS route
- Whether to use `include_str!` at the crate level or a `static FERRO_BASE_CSS: &str = include_str!(...)` in a dedicated asset module
- CI check implementation (diff-based or hash-based)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Implementation
- `ferro-json-ui/src/config.rs` — `JsonUiConfig` struct and builder methods; starting point for config changes
- `framework/src/json_ui/mod.rs` — `build_response` head injection logic; where CDN script and theme style tags are emitted (lines 93–115)
- `ferro-json-ui/src/render.rs` — component emitters that emit Tailwind utility classes (source input for tailwind CLI scan)

### Upstream References
- Tailwind v4 standalone binary: https://tailwindcss.com/blog/standalone-cli
- Tailwind v4 browser runtime (dev-only disclaimer): https://tailwindcss.com/docs/installation/play-cdn
- Field report: gestiscilo.it Safari rendering failure, 2026-04-20 (iPhone Safari + desktop Safari, login page unstyled)

### Affected Crates
- `ferro-json-ui` — config default, `stylesheet_urls` field, CSS asset
- `framework` — head injection logic, static route registration for `/_ferro/ferro-base.css`
- `ferro-theme` — default theme must declare full variable vocabulary as plain CSS vars

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `JsonUiConfig` builder pattern (`with_*`-style methods) — follow same pattern for `stylesheet_urls`
- Existing `include_str!`/`include_bytes!` usage in the codebase (check for prior examples in framework crates)

### Established Patterns
- `tailwind_cdn(bool)` builder method — pattern for the new `stylesheet_urls(Vec<String>)` builder
- `#[cfg(feature = "theme")]` block at line ~109 in `framework/src/json_ui/mod.rs` — where theme CSS injection lives; this block needs updating
- Framework static file handling — check if framework already has infrastructure for serving embedded static files (e.g. favicon, or other `/_ferro/*` routes)

### Integration Points
- `framework/src/json_ui/mod.rs:build_response` — the single function that assembles the `<head>` string; all changes are localized here plus the route registration
- App-level gestiscilo consumer — after this phase ships, gestiscilo bumps ferro dep, removes `tailwind_cdn(true)` config, and points its tokens.css at the stylesheet_urls list

</code_context>

<specifics>
## Specific Ideas

- The "production failure in Safari" is the concrete motivation. The success signal is: gestiscilo.it login page renders fully styled in iPhone Safari with zero CDN requests.
- Tailwind v4 standalone binary is Rust-native — no Node. The same binary used to generate the checked-in CSS can be documented for contributors who add new utility classes to ferro-json-ui.
- gestiscilo's `themes/gestiscilo/tokens.css` currently uses `@theme` syntax. Converting it to plain `:root { --color-...: ... }` is part of this phase's acceptance criteria.

</specifics>

<deferred>
## Deferred Ideas

- **App-level Tailwind build loop** — watch/rebuild for apps that want to extend Tailwind beyond ferro-base.css. Out of scope for this phase.
- **Tailwind config file support** (`@theme` compilation for apps) — apps can only override vars in v1 of this feature.
- **Additional modalities** — no impact on this phase.

None of the above were requested in this phase's discussion — all deferred by the existing CONTEXT scope definition.

</deferred>

---

*Phase: 143-tailwind-static-css-pipeline*
*Context gathered: 2026-04-20*
