---
phase: 148
slug: htmlembed-component-ferro-json-ui
status: draft
shadcn_initialized: false
preset: none
created: 2026-04-24
precedent: 147-UI-SPEC.md
---

# Phase 148 — UI Design Contract (HtmlEmbed)

> This contract documents the DELIBERATE ABSENCE of a visual design surface.
> `HtmlEmbed` is a pure-backend Rust primitive in `ferro-json-ui`: the renderer
> wraps a caller-supplied HTML string in a bare `<div>` and emits the string
> **verbatim** (no `html_escape`). The component has no default class, no
> default id, no default attributes, no interactions, and no states. The
> caller owns every visual decision inside `{html}`.
>
> Most sections below are marked **N/A** with a brief rationale. This is
> correct for this phase. A bloated spec that invents tokens, palettes, or
> typography for a transparent wrapper would be dishonest.
>
> Authoritative source for the decisions referenced below: `148-CONTEXT.md`
> (D-01..D-24) and `148-RESEARCH.md`.

---

## 0. Scope of this contract

`HtmlEmbed` is a server-side Rust HTML emitter. It does not ship JavaScript,
does not own state, does not introduce design tokens, and does not compose
a visual surface of its own. Its entire rendered output is:

```
<div>{props.html}</div>
```

with `{props.html}` emitted **unescaped**. Everything the user sees is
produced by the caller-supplied HTML inside the wrapper. The framework's
only contribution to the pixels on screen is the opening and closing `<div>`
tags.

The contract this phase owns therefore reduces to four items:

1. The literal output shape: `<div>{html}</div>`, no attributes.
2. The no-escaping invariant (the reason `HtmlEmbed` exists).
3. The safety framing visible at every author surface (rustdoc,
   `COMPONENT_CATALOG`, `json_ui_catalog`, docs chapter, inline renderer
   comment).
4. The accessibility-responsibility boundary: because the HTML is opaque
   to the framework, a11y is the caller's responsibility.

Everything else — spacing, typography, color, copy, registries — is N/A for
this phase.

---

## 1. Component Contract

| Aspect | Value | Source |
|--------|-------|--------|
| Output shape | `<div>{props.html}</div>` | D-06 |
| Default class | none | D-06 |
| Default id | none | D-06 |
| Default attributes on `<div>` | none | D-06 |
| Wrapper element | `<div>` only (not `<span>`, `<section>`, etc.) | D-06, deferred list |
| Escaping | **NONE** — `props.html` emitted verbatim | D-06, EMBED-02 |
| Data binding | none — `html` is a static `String` | D-05, deferred list |
| Children | none — leaf component | D-10, D-11 |
| Action surface | none — not clickable, no form semantics | D-10, D-11 |
| Field-error surface | none — no validation participation | D-10, D-11 |
| Runtime JS | none | D-24 |
| Plugin hooks | none — built-in component | D-08, domain |

### Invariants the checker / auditor must verify

1. `render_html_embed(props)` returns exactly `format!("<div>{}</div>", props.html)`
   (or a byte-equivalent literal). No attributes. No classes. No
   whitespace variations between the `<div>` and `{html}`.
2. `render_html_embed` does **not** call `html_escape` on `props.html`.
   An inline code comment in the function body must document this
   omission so future audits do not "fix" it.
3. A test in `render.rs` asserts that `<script>alert('xss')</script>`
   passes through unescaped. This test is a **contract** (proving the
   bypass), not a smell.
4. Resolver passes (`resolve_component_node`, `collect_unresolved_node`,
   `resolve_errors_node`) all match `Component::HtmlEmbed(_)` in the
   leaf OR-chain ending in `=> {}`. No standalone arms.
5. The `<div>` has no `role`, no `aria-*`, no `tabindex`. Any a11y
   semantics live inside `{props.html}` and are the caller's problem.

---

## 2. Design System

**N/A — `HtmlEmbed` introduces no design system surface.**

| Property | Value | Rationale |
|----------|-------|-----------|
| Tool | not applicable | Server-side Rust HTML; no component library, no CSS framework consumption |
| Preset | not applicable | No shadcn / no tokens |
| Component library | not applicable | Bare `<div>` wrapper; no radix / base-ui / headless layer |
| Icon library | not applicable | Renderer emits no icons |
| Font | not applicable | Renderer emits no font rules; caller's HTML inherits from ancestor page |
| Theme bridge | not applicable | No `ferro-theme` tokens consumed — the wrapper has no classes |

---

## 3. Spacing Scale

**N/A — the wrapper emits no spacing.**

The `<div>` has no padding, no margin, no gap. Spacing within the embedded
content is owned entirely by the caller's HTML. Spacing around the
`HtmlEmbed` is owned by the surrounding component (e.g. `Card`, `Grid`,
`FormSection`). `HtmlEmbed` declares no spacing tokens.

---

## 4. Typography

**N/A — the wrapper emits no typography.**

Font family, size, weight, and line-height inside `{props.html}` are
whatever the caller's HTML sets (inline styles, inherited CSS, embedded
`<style>` blocks, or `<svg>` text elements). `HtmlEmbed` declares no
typography tokens and inherits nothing it would care about.

---

## 5. Color

**N/A — the wrapper emits no color.**

The `<div>` has no background, no border, no text color. All color inside
`{props.html}` is owned by the caller's content (SVG `fill`, inline
`style=`, markdown-generated classes, embedded stylesheets, etc.).
`HtmlEmbed` declares no color tokens and has no accent-reservation surface
to document.

---

## 6. Copywriting Contract

**N/A — the renderer emits no prose.**

The framework's contribution to the rendered output is `<div>` + `</div>`.
No labels, no CTAs, no empty states, no error banners, no confirmations.
All human-readable text inside `{props.html}` originates from the caller.

The one place copy **does** matter for this phase is the **author-facing
safety messaging** (§8). That is not runtime UI copy — it is documentation
visible to agents and humans who introspect the component.

---

## 7. Interaction & States

**N/A — `HtmlEmbed` has no interaction model.**

| State | Behaviour |
|-------|-----------|
| Loading | not applicable — server-rendered, no async |
| Hover | not applicable — the `<div>` has no hover affordance; any hover inside `{html}` is the caller's content |
| Focus | not applicable — the `<div>` is not focusable (no `tabindex`) |
| Validation error | not applicable — resolver skips this component in `resolve_errors_node` (D-10) |
| Disabled | not applicable — no interactive surface to disable |
| Empty | if `props.html == ""`, the renderer emits `<div></div>`. This is a valid degenerate output; no special UI. A Wave-0 test covers the empty case. |

Keyboard, mouse, pointer, and touch are all transparent — they affect
whatever the caller's HTML provides, not the wrapper.

---

## 8. Safety & Author-Facing Messaging (load-bearing)

This is the **only** section in this spec that carries substantive content,
because the no-escaping invariant is the whole point of the component and
the asymmetry must be visible at every surface an author can touch.

### Five surfaces where the safety contract MUST appear

| # | Surface | Required content | Source decision |
|---|---------|------------------|-----------------|
| 1 | `HtmlEmbedProps` rustdoc | (a) "HTML is emitted verbatim without escaping." (b) "Callers are responsible for XSS safety." (c) "Intended for server-generated content (inline SVG, pre-rendered markdown), not user input." (d) One-line pointer to `Component::Text` for escaped output. | D-15 |
| 2 | `COMPONENT_CATALOG` string entry in `ferro-json-ui/src/lib.rs` | Description foregrounds unescaped behavior + caller-owned safety + "do NOT pass user input." | D-16 |
| 3 | MCP `CatalogComponent` description in `ferro-mcp/src/tools/json_ui_catalog.rs` | Same safety-first phrasing; the `html` prop's description in the catalog `prop(...)` call also names the bypass. | D-17, D-18 |
| 4 | `### HtmlEmbed` section in `docs/src/json-ui/components.md` | Opening paragraph + dedicated safety callout (blockquote or warning block matching the docs site's existing warning convention) + explicit "for escaped text, use `Text`" pointer. | D-21 |
| 5 | Inline comment in the body of `render_html_embed` in `ferro-json-ui/src/render.rs` | One line flagging the deliberate `html_escape` omission so future audits do not "fix" it. | `148-CONTEXT.md` Code Insights, §Established Patterns |

### Intended vs. non-intended content types

Documented explicitly at surfaces 1, 2, 3, and 4 above:

- **Intended:** inline SVG charts, pre-rendered markdown, static HTML
  widgets, third-party embed snippets (e.g. tweet embeds).
- **Non-intended:** user input, anything not fully controlled by the
  server, any string that has passed through a user-writable field.

### Accessibility responsibility boundary

The `<div>` wrapper carries no `role`, no `aria-*`, and no `tabindex`.
Accessibility semantics (alt text, landmarks, labels, focus management)
are entirely the caller's responsibility because the framework cannot
inspect the opaque `{html}` payload. Documentation (surface 4) must state
this boundary so authors know not to expect the framework to add a11y
hooks around the wrapper.

This boundary is not a deficiency — it is the honest consequence of the
component's design. `HtmlEmbed`'s job is to stay out of the way; a11y
correctness of the injected fragment is a property of the fragment.

---

## 9. Component Inventory

Phase 148 adds **one** component to `ferro-json-ui`. There are no variants,
no sizes, no density modes, no composition presets.

| Component | Variant set | Composes | Owns rendering of |
|-----------|-------------|----------|-------------------|
| `HtmlEmbed` | single (no variants) | nothing — leaf | one `<div>` |

Ships:

- `HtmlEmbedProps` struct with a single `pub html: String` field (D-01).
- `Component::HtmlEmbed(HtmlEmbedProps)` enum variant (D-02).
- `HtmlEmbedProps::new(html: impl Into<String>) -> Self` convenience
  constructor (D-03).
- `ComponentNode::html_embed(key, props)` factory (D-04).
- `fn render_html_embed(props: &HtmlEmbedProps) -> String` renderer
  (D-05, D-06, D-09).
- Leaf-arm participation in all three resolver passes (D-10, D-11).
- Serde tagged-enum arms for round-trip (D-12, D-13, D-14).
- MCP catalog entry + exhaustive-list bump (D-18, D-19, D-20).
- Public re-export + `### HtmlEmbed` `COMPONENT_CATALOG` entry (D-16).
- User docs section with safety callout (D-21).

No new design tokens. No new classes. No new fonts. No new colors.

---

## 10. Registry Safety

**N/A — no shadcn registry, no third-party block consumption.**

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| none | none | not required |

`HtmlEmbed` is a built-in component implemented directly in `ferro-json-ui`
Rust source. No external block, no plugin registry, no `npx shadcn add`
surface. The safety-gate mechanism does not apply; the equivalent concern
(unsafe HTML injection by the caller) is addressed by the §8 safety
messaging contract.

---

## 11. Out of Scope (explicit fences — do not re-open in planning)

Every item below is deferred per `148-CONTEXT.md`. Re-introducing any of
them in planning or execution is a scope violation.

- `class` / `id` / `style` props on the wrapper `<div>`.
- Configurable wrapper element (`<span>`, `<section>`, `<iframe sandbox>`).
- `data_path: Option<String>` binding from the data payload.
- Built-in sanitization opt-in (`sanitize: Option<bool>` / DOMPurify).
- Plugin-style variants (`HtmlEmbedIframe`, etc.).
- Sibling markdown component (`MarkdownEmbed { source: String }`).
- Framework-level `#[warning("unescaped")]` clippy lint.
- `Default` derive on `HtmlEmbedProps` (empty `html` is a semantic
  foot-gun per Claude's Discretion resolution).

---

## 12. Acceptance for the UI Checker / Auditor

A pass requires all of:

1. Rendered output of `render_html_embed(&HtmlEmbedProps::new("<svg/>"))`
   is exactly `<div><svg/></div>` (bytewise), with no attributes on the
   wrapping `<div>`.
2. The renderer body does not call `html_escape` (or any equivalent
   escaping helper) on `props.html`, and carries an inline comment
   documenting this deliberate omission.
3. `render_html_embed(&HtmlEmbedProps::new("<script>alert('xss')</script>"))`
   returns a string containing the literal substring
   `<script>alert('xss')</script>` (proving the bypass contract).
4. Empty input: `render_html_embed(&HtmlEmbedProps::new(""))` returns
   exactly `<div></div>`.
5. All three resolver passes match `Component::HtmlEmbed(_)` inside the
   leaf OR-chain ending in `=> {}`. No standalone arms exist.
6. `HtmlEmbedProps` rustdoc contains all four D-15 content requirements:
   verbatim-emission statement, caller-responsibility statement,
   intended-use-cases statement, `Component::Text` pointer.
7. `COMPONENT_CATALOG` entry in `ferro-json-ui/src/lib.rs` foregrounds the
   safety contract (unescaped + caller-owned safety + do-not-pass-user-input).
8. `ferro-mcp/src/tools/json_ui_catalog.rs` gains a `CatalogComponent`
   entry with a safety-first description and a `prop("html", "String",
   true, ...)` entry whose description also names the bypass.
9. The exhaustive-list assertion is bumped 41 → 42 with an updated
   comment, and `"HtmlEmbed"` is present in the `expected` array.
10. `docs/src/json-ui/components.md` gains a `### HtmlEmbed` section with
    opening paragraph, safety callout, props table, Rust example, JSON
    example, use-case list, and the "for escaped text, use `Text`" pointer.
11. CI gate green:
    `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.

Items 1–5 are the visual / structural contract (such as it is). Items
6–10 are the author-facing safety contract. Item 11 is the completion
gate.

---

## 13. Pre-population Provenance

| Source | Decisions used |
|--------|----------------|
| `148-CONTEXT.md` D-01..D-24 | Output shape, no-escape invariant, factory shape, resolver posture, serde arms, safety messaging surfaces, wave structure, deferred list |
| `148-RESEARCH.md` | Line-number references, architectural responsibility map, standard-stack confirmation (no new deps) |
| `147-UI-SPEC.md` (precedent) | Spec structure and voice — scaled DOWN because phase 148's visual surface is materially smaller than DetailForm's |
| `/Users/alberto/.claude/CLAUDE.md` — "Design by subtraction" | Justification for the deliberate-absence framing of §§2–7 |
| `.planning/VISION.md` — agent-first philosophy | Justification for the load-bearing safety-messaging contract in §8 |

No interactive questions were asked. No decisions were invented. Every
`N/A` marker above is a direct consequence of the locked decisions in
`148-CONTEXT.md`, not a placeholder.

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS (N/A — no runtime prose; §8 author-facing messaging contract verified instead)
- [ ] Dimension 2 Visuals: PASS (output shape matches §1 invariants; renderer body matches §12 items 1–4)
- [ ] Dimension 3 Color: PASS (N/A — no color surface)
- [ ] Dimension 4 Typography: PASS (N/A — no typography surface)
- [ ] Dimension 5 Spacing: PASS (N/A — no spacing surface)
- [ ] Dimension 6 Registry Safety: PASS (N/A — no registry consumption; §8 safety-messaging contract verified instead)

**Approval:** pending
