# Phase 143: Tailwind Static CSS Pipeline — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 143-tailwind-static-css-pipeline
**Mode:** --auto (all options auto-selected at recommended defaults)
**Areas discussed:** CSS Production Mechanism, Static Route Registration, Config API Shape, Theme Injection Replacement, CDN Fallback Handling

---

## CSS Production Mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Check in generated CSS + CI verify | Run tailwind CLI in CI, commit output, embed via include_str! | ✓ |
| build.rs runtime generation | Run tailwind CLI during cargo build (requires CLI on PATH for all contributors) | |
| Runtime file serving | Read CSS from disk at runtime (no embedding) | |

**Auto-selected:** Check in generated CSS + CI verify (recommended — no build-time dep for users)
**Notes:** CI step diffs the output to detect drift. Contributors adding new utility classes regenerate manually and commit.

---

## Static Route Registration

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-register at framework boot | `/_ferro/ferro-base.css` always available, no user config | ✓ |
| Register via explicit init call | User calls `ferro.use_json_ui(config)` to mount the route | |
| User-configured static serving | User adds the route themselves | |

**Auto-selected:** Auto-register at framework boot (recommended — zero friction)

---

## Config API Shape

| Option | Description | Selected |
|--------|-------------|----------|
| `stylesheet_urls(Vec<String>)` | Replaces the full list; default `["/_ferro/ferro-base.css"]` | ✓ |
| `add_stylesheet(url)` | Additive chainable method; harder to remove the default | |
| Single `stylesheet_url` field | Only one URL supported; too limiting | |

**Auto-selected:** `stylesheet_urls(Vec<String>)` (recommended — flexible, consistent with builder pattern)

---

## Theme Injection Replacement

| Option | Description | Selected |
|--------|-------------|----------|
| Inline `<style>` with plain CSS vars | Replace `type="text/tailwindcss"` with plain `<style>`; vars resolve natively | ✓ |
| Separate `/_ferro/theme.css` route | Dynamic route serving theme vars; adds complexity | |
| Remove theme injection entirely | Apps must include their tokens.css via `stylesheet_urls` | |

**Auto-selected:** Inline `<style>` with plain CSS vars (recommended — minimal change, preserves existing injection path)

---

## CDN Fallback Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Flip default to false, keep opt-in | `tailwind_cdn` default false; `tailwind_cdn(true)` still works | ✓ |
| Deprecate `tailwind_cdn` field | Mark deprecated but keep functional | |
| Remove field entirely | Breaking change, remove the CDN path | |

**Auto-selected:** Flip default to false, keep opt-in (recommended — breaking change acceptable pre-1.0; CDN path useful for quick prototyping)

---

## Claude's Discretion

- Exact asset path for checked-in CSS (`ferro-json-ui/assets/` vs `framework/assets/`)
- Cache-Control header value for static CSS route
- `include_str!` placement (module-level vs dedicated asset module)
- CI diff-based vs hash-based staleness check

## Deferred Ideas

- App-level Tailwind watch/rebuild loop
- `@theme` compilation support for apps (beyond var overrides)
