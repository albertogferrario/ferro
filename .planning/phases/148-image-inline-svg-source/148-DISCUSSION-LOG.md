# Phase 148: HtmlEmbed component for ferro-json-ui — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-24
**Phase:** 148-htmlembed-component-ferro-json-ui
**Mode:** `--auto` (single-pass; recommended default auto-selected for every gray area)
**Areas discussed:** Props shape, Wrapper element & styling, Resolver participation, Security framing, Factory constructor, MCP catalog & exhaustive-list assertion, TDD wave structure, Docs entry

---

## Props shape

| Option | Description | Selected |
|--------|-------------|----------|
| A. `HtmlEmbedProps` named struct | `pub struct HtmlEmbedProps { pub html: String }` — every Component variant uses a `*Props` struct; consistent uniformity; future extension (class, wrapper_tag, data_path) stays non-breaking | ✓ |
| B. Struct-style enum variant | `Component::HtmlEmbed { html: String }` — matches ROADMAP wording literally but breaks enum uniformity (42 other variants all use `*Props` named structs) | |
| C. Tuple variant | `Component::HtmlEmbed(String)` — most minimal but strongest break from the crate's pattern | |

**User's choice:** A (recommended default)
**Notes:** ROADMAP wording `Component::HtmlEmbed { html: String }` is conceptual; implementation mirrors the established Props-struct pattern to preserve uniformity and keep future extensions non-breaking.

---

## Wrapper element & styling

| Option | Description | Selected |
|--------|-------------|----------|
| A. Plain `<div>`, no classes | `<div>{html}</div>` — minimal surface, matches ROADMAP goal literally, callers style via parent containers | ✓ |
| B. `<div class="ferro-html-embed">` | Adds a default styling hook for projects that want one | |
| C. Configurable wrapper element + class props | `wrapper_tag: Option<String>` + `class: Option<String>` — flexibility today, API surface tomorrow | |

**User's choice:** A (recommended default)
**Notes:** Follows Separator/Image minimalism. Wrapper class/tag stay in `<deferred>` for future extension.

---

## Resolver participation

| Option | Description | Selected |
|--------|-------------|----------|
| A. Grouped leaf OR-chain | Add `\| Component::HtmlEmbed(_)` to each of the three leaf OR-chains ending in `=> {}` in resolve.rs — follows Separator/DescriptionList/Image precedent | ✓ |
| B. Standalone empty arms | Explicit `Component::HtmlEmbed(_) => {}` in each pass — more verbose, inconsistent with convention | |

**User's choice:** A (recommended default)
**Notes:** Matches the established leaf-grouping convention for components with no action/children/error surface.

---

## Security framing

| Option | Description | Selected |
|--------|-------------|----------|
| A. Loud & repeated safety messaging | Prominent warning on `HtmlEmbedProps` rustdoc + `COMPONENT_CATALOG` string + MCP catalog description + docs safety callout + inline comment in `render_html_embed` | ✓ |
| B. Minimal inline note | Single line on the props struct docstring only | |

**User's choice:** A (recommended default)
**Notes:** This is the one component that deliberately bypasses html_escape. The asymmetry is load-bearing and must be visible everywhere a human or agent encounters the type.

---

## Factory constructor shape

| Option | Description | Selected |
|--------|-------------|----------|
| A. `ComponentNode::html_embed(key, props)` + `HtmlEmbedProps::new(html)` helper | Matches Separator/Image/DetailForm `(key, props)` factory pattern; `HtmlEmbedProps::new` keeps call-sites compact | ✓ |
| B. `ComponentNode::html_embed(key, html)` shorthand | Skips the props wrapper; most ergonomic for a single-field type but breaks uniformity with every other factory | |

**User's choice:** A (recommended default)
**Notes:** Uniformity with the rest of the factory surface wins over a one-keystroke savings. `HtmlEmbedProps::new(html)` provides the compact call-site ergonomics.

---

## MCP catalog & exhaustive-list assertion

| Option | Description | Selected |
|--------|-------------|----------|
| A. Full catalog integration | Add `CatalogComponent { name: "HtmlEmbed", ... }` with safety-first description; bump exhaustive-list assertion from 41 → 42 and append `"HtmlEmbed"` to the `expected` array in `test_all_components_present`; verify `no_required` allowlist exclusion | ✓ |
| B. Minimal catalog entry, skip assertion bump | Would fail CI — the exhaustiveness test is what catches catalog drift | |

**User's choice:** A (recommended default)
**Notes:** Every component-addition phase touches this assertion. Phase 147 went 40 → 41; phase 148 goes 41 → 42.

---

## TDD wave structure

| Option | Description | Selected |
|--------|-------------|----------|
| A. Wave 0 RED + Wave 1 impl (phase 147 shape) | Wave 0: serde + render + resolver + MCP catalog assertion tests. Wave 1: types + render + resolver + MCP + docs + CI gate. Plan split per file area (component.rs, render.rs, resolve.rs, MCP+docs) | ✓ |
| B. Single wave | Combined tests + impl in one pass — loses the RED-GREEN discipline and parallelization opportunity | |

**User's choice:** A (recommended default)
**Notes:** Matches the crate's TDD convention established in phases 146 and 147. No runtime wave — `HtmlEmbed` has zero JS.

---

## Docs entry

| Option | Description | Selected |
|--------|-------------|----------|
| A. Full `### HtmlEmbed` section | Safety callout, props table, Rust example, JSON output, use-case list, escaped-text pointer to `Text` — matches KeyValueEditor/DetailForm density | ✓ |
| B. Minimal paragraph | One paragraph + one example — below the density of sibling component sections | |

**User's choice:** A (recommended default)
**Notes:** Follows the existing docs chapter density and ensures the safety callout is visible in the reference documentation.

---

## Claude's Discretion

- Exact Rustdoc prose for the safety warning on `HtmlEmbedProps`
- Exact wording of the MCP catalog and COMPONENT_CATALOG descriptions
- Local ordering of new entries (alphabetical vs. recency-grouped) — match existing local convention
- Whether to emit a `<script>alert('xss')</script>` pass-through test (recommended: yes — documents the intended-bypass contract)
- Docs safety-callout styling (blockquote vs. `⚠️ Warning` block) — match sibling files in `docs/src/json-ui/`
- Whether `HtmlEmbedProps` derives `Default` (recommended: no — explicit construction is safer)

## Deferred Ideas

- Wrapper `class` / `id` / `style` props
- Configurable wrapper tag (`<span>` for inline usage)
- `data_path` binding for HTML from data payload
- Built-in sanitization opt-in
- Plugin-style sandboxed variant (`HtmlEmbedIframe`)
- Markdown-aware sibling (`MarkdownEmbed`)
- Framework-level `#[warning("unescaped")]` marker / Clippy lint

---

## 2026-04-24 — Rework: scope changed from `Component::HtmlEmbed` to extending `Component::Image`

After the planning artifacts were authored, execution was paused mid-wave (no
production code had been touched yet — only the execution-start STATE.md update
was committed). Reason for pause: the proposed `Component::HtmlEmbed { html:
String }` shape was challenged.

### Challenge
A generic `HtmlEmbed` component:
1. Reads as a general HTML escape hatch — the name invites misuse with
   user-supplied content.
2. Weakens the projection/intent abstraction by adding a pixel-level escape
   hatch disjoint from any structural concept.
3. Ships a `<deferred>` list (`class`, `id`, `wrapper_tag`, `data_path`,
   `sanitize`, iframe sandbox, `MarkdownEmbed`) that reads as the roadmap of
   a swiss army knife.
4. Duplicates the "bounded visual asset rendered into a box" concept that
   `Image` already owns, and loses the compile-enforced `alt` text
   requirement (a11y regression vs. extending Image).

### Real requirement (from `../../gestiscilo-it/app/.planning/research/v6.1-ANALYTICS-ARCHITECTURE.md`)
Server-generated inline SVG bar chart for the Statistiche revenue-trend
chart (gestiscilo Phase 117). No other concrete call-site — "pre-rendered
markdown", "static HTML widgets", "third-party embed snippets" in the
original HtmlEmbed context were hypothetical.

### Decision
Extend `ImageProps` with an `ImageSource` serde-untagged enum:

```rust
#[serde(untagged)]
pub enum ImageSource {
    Url { src: String },
    InlineSvg { svg: String },
}
```

`ImageProps` flattens `source: ImageSource`; `alt: String` stays required
(a11y win). Wire format stays backward-compatible for the URL case.
`render_image` gains one branch for the SVG case; the URL branch keeps its
XSS-escape test verbatim.

Benefits:
- No new `Component` variant, no exhaustive-list bump, no new resolver arms,
  no new MCP entry.
- a11y-enforced: `alt: String` compile-required on both variants.
- Projection/intent story preserved — `Image` is already the
  "bounded visual asset" slot.
- Deferred-feature pressure collapses (`data_path`, `MarkdownEmbed`,
  iframe sandbox, sanitize) — Image is a bounded scope, not a swiss army knife.

### Archive
Original HtmlEmbed planning artifacts preserved under
`archive-htmlembed/` in this directory:
- `148-CONTEXT.md` — HtmlEmbed scope
- `148-RESEARCH.md` — HtmlEmbed research
- `148-PATTERNS.md` — HtmlEmbed patterns
- `148-UI-SPEC.md` — HtmlEmbed UI spec
- `148-VALIDATION.md` — HtmlEmbed validation
- `148-0{1..5}-PLAN.md` — five HtmlEmbed plans

These are kept for historical reference and so the decision trail is
traceable. They are NOT to be re-used for execution.

### Cross-repo follow-up
- `gestiscilo-it/app/.planning/research/v6.1-ANALYTICS-ARCHITECTURE.md`
  — update the "Rendering Strategy" section to reference
  `ImageProps::inline_svg(svg, alt)` instead of `Component::HtmlEmbed`.
- `gestiscilo-it/app/.planning/ROADMAP.md` Phases 115-117 — update the
  dependency note and Phase 117 success criterion #2 wording.

### Re-planning
`/gsd-plan-phase 148 --auto` will regenerate RESEARCH + PATTERNS + PLAN
artifacts from the new CONTEXT.md.
