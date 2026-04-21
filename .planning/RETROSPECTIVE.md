# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v11.7 — Tailwind Static CSS Pipeline

**Shipped:** 2026-04-21
**Phases:** 1 (143) | **Plans:** 4

### What Was Built
- Pre-built `ferro-base.css` (36 KB) embedded at compile time via `include_str!`, eliminating in-browser Tailwind JIT
- `/_ferro/ferro-base.css` static route with zero-copy `Bytes::from_static` and 24h `Cache-Control`
- CI drift check (`ferro-base-css-drift` job) enforcing committed CSS stays in sync with Tailwind CLI output
- `JsonUiConfig::stylesheet_urls: Vec<String>` field + `tailwind_cdn` default flipped to `false`
- Theme injection migrated from `<style type="text/tailwindcss">` magic MIME to plain `<style>` CSS variable overrides
- `ferro make:theme` scaffolder updated to emit plain `:root { }` CSS (not `@theme { }`)

### What Worked
- Urgent insertion pattern (single-phase milestone) worked well — clear scope, fast execution, all 4 plans done in one day
- XSS-safe href emission via `html_escape()` on `stylesheet_urls` was caught and added proactively during Plan 03
- Exact-string-match route dispatch (`/_ferro/ferro-base.css`) makes path traversal structurally impossible without extra logic
- Bootstrapping with a placeholder CSS (then replacing via CLI) unblocked the compile-time embedding before CLI install

### What Was Inefficient
- VERIFICATION.md not generated at milestone close — required a post-hoc gsd-verifier run; process gap, not substance gap
- Roadmap tool couldn't parse phase 143's detail section format — `missing_phase_details: ["143"]` throughout

### Patterns Established
- `include_str!` for static assets: committed binary/text asset + `pub const` re-exported from `lib.rs`
- Exact-string-match for framework-owned routes (no path parsing → no path traversal)
- CI drift pattern: generate → diff → fail with actionable error message

### Key Lessons
1. `@tailwindcss/browser@4` is documented as dev-only — production CDN CSS must always be pre-built. Verify CDN script limitations against the vendor docs.
2. A single urgent phase milestone (v11.7 = just Phase 143) is a valid pattern for production hotfixes; don't force it into a larger milestone.
3. Run `/gsd-verify-work` before closing any phase, even when UAT passed manually — VERIFICATION.md is the formal artifact that audit checks.

---

## Milestone: v10.0 — JSON-UI Visual Overhaul

**Shipped:** 2026-03-26
**Phases:** 6 | **Plans:** 8

### What Was Built
- Inter Variable font loading via Bunny Fonts CDN with correct Tailwind v4 token namespace
- Three-tier surface elevation hierarchy (background/surface/card) with WCAG dark mode contrast
- Typography scale system (heading rhythm + body line-height)
- Form polish (SVG chevron, destructive error rings, transitions, disabled states, DOM reorder)
- Focus-visible rings and hover states across all interactive elements
- SVG icons replacing emoji throughout (alerts, bell, breadcrumb, collapsible) + shimmer animation

### What Worked
- CSS dependency chain ordering (token fix → surfaces → typography → forms → interactive → details) prevented rework
- has_class() test helper established in Phase 102 prevented test avalanche across remaining 5 phases
- Structural vs cosmetic test separation kept tests stable through 6 phases of class string changes
- concat! macro pattern for inline SVG kept components self-contained without external dependencies

### What Was Inefficient
- Dark mode contrast verification was manual (oklch calculations by hand) — could benefit from automated WCAG checker
- BELL_SVG duplication between render.rs and layout.rs — visibility constraint (private const) caused copy instead of share

### Patterns Established
- `focus-visible:` over `focus:` for all keyboard-only focus rings (no mouse click noise)
- Canonical interactive element class triple: transition-colors + duration-150 + motion-reduce:transition-none + focus-visible ring quad
- Three-tier surface hierarchy: background (page/persistent frames) < surface (panels/hover) < card (floating components)
- Inline SVG via concat! macro for CDN-safe icon embedding

### Key Lessons
1. Tailwind v4 changed token namespaces silently — v3 `--font-family-*` tokens are completely ignored by v4. Always verify CDN token names match the Tailwind version.
2. Test infrastructure investment (has_class helper) in the first phase pays off exponentially across subsequent phases that modify the same class strings.
3. WCAG contrast ratios in oklch are non-linear — small L value changes can swing contrast ratios significantly. Design trade-offs need explicit documentation (pair 6 at 4.45:1).

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v10.0 | 6 | 8 | CSS dependency chain ordering; test infrastructure first |

### Cumulative Quality

| Milestone | Tests | Notable |
|-----------|-------|---------|
| v10.0 | 426 ferro-json-ui | WCAG 4.5:1 dark mode compliance (7/8 pairs, 1 accepted trade-off) |

### Top Lessons (Verified Across Milestones)

1. Test infrastructure investment in early phases prevents cascading failures in later phases
2. CSS token namespace changes between framework versions are silent breaking changes — verify empirically
