# Phase 148: image-inline-svg-source — Research

**Researched:** 2026-04-24
**Domain:** ferro-json-ui component system — Rust serde untagged enum, HTML rendering, backward-compatible wire format extension
**Confidence:** HIGH

## Summary

Phase 148 extends `ImageProps` in `ferro-json-ui` with an `ImageSource` serde-untagged enum so `Component::Image` can carry either an external URL (`src: String`) or a server-constructed inline SVG string (`svg: String`). The URL wire format stays fully backward-compatible. The renderer gains one branch for the SVG case; everything else — the `Component` enum variant, all resolver arms, the MCP exhaustive-list count (41), and the `collect_plugin_types_node` leaf group — remains unchanged.

The design is self-contained within five files: `ferro-json-ui/src/component.rs`, `ferro-json-ui/src/render.rs`, `ferro-json-ui/src/lib.rs`, `ferro-mcp/src/tools/json_ui_catalog.rs`, and `docs/src/json-ui/components.md`. No new crates, no new dependencies, no resolver changes.

The safety framing is a first-class deliverable. The InlineSvg render branch is the one deliberate `html_escape` bypass in the entire codebase. That asymmetry must be visible at every level: rustdoc on `ImageSource::InlineSvg`, an inline comment in `render_image`, the `COMPONENT_CATALOG` string, the MCP catalog description, and the user-facing docs section. A load-bearing test (`inline_svg_with_script_passes_through`) asserts the bypass is working and doubles as executable documentation.

**Primary recommendation:** Follow the Phase 147 wave shape exactly: Wave 0 RED tests → Wave 1 implementation → Wave 2 surface updates (catalog + docs + CI gate). The five in-tree `ImageProps { src, alt, ... }` struct literals are rewritten in Wave 1; no deprecation shim.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Core shape:**
- D-00: HtmlEmbed rejected. `Component::HtmlEmbed` is out of scope. Extend `Component::Image` only.
- D-01: `ImageSource` is a serde-untagged enum: `Url { src: String }` + `InlineSvg { svg: String }`
- D-02: `ImageProps` flattens source via `#[serde(flatten)]`; `alt: String` stays required (compile-enforced a11y)
- D-03: `ImageProps::url(src: impl Into<String>, alt: impl Into<String>) -> Self` and `ImageProps::inline_svg(svg: impl Into<String>, alt: impl Into<String>) -> Self` convenience constructors; both default `aspect_ratio = None`, `placeholder_label = None`
- D-04: No deprecation shims — rewrite in-tree call-sites to new shape directly (pre-1.0)

**Rendering:**
- D-05: `render_image` branches on `props.source`. URL path unchanged. InlineSvg path: `<div role="img" aria-label="{escaped alt}">{svg verbatim}</div>` inside the existing aspect-ratio container. Placeholder NOT rendered for InlineSvg variant.
- D-06: Inline safety-contract comment in the InlineSvg branch flagging the deliberate `html_escape` omission. `alt` IS escaped on both branches.
- D-07: `render_component` dispatch arm unchanged.
- D-08: `collect_plugin_types_node` leaf group unchanged (Image already present).
- D-09: No new resolver arms.

**Serde:**
- D-10: Ambiguous input (both `src` + `svg`, or neither) rejected by serde untagged-enum discriminator. One test MUST assert this failure path.
- D-11: `Component::Serialize`/`Component::Deserialize` arms for `"Image"` unchanged; serde handles discrimination via `#[serde(flatten)]` + `#[serde(untagged)]`.

**Safety framing (all five sites required):**
- D-12: Rustdoc on `ImageSource::InlineSvg` AND `ImageProps::inline_svg` constructor: SVG emitted verbatim, intended for server-constructed SVG, not user input, callers sanitize if needed.
- D-13: `COMPONENT_CATALOG` `### Image` section (currently absent — new): both variants described, SVG safety note.
- D-14: MCP `CatalogComponent` for Image (exists — update): both variants in props/description, safety note. Catalog count stays at 41.
- D-15: `docs/src/json-ui/components.md` `### Image` section (currently absent — new): props table, Rust + JSON examples for both variants, safety callout blockquote.

**Wave structure:**
- D-16/D-17: 3–4 plans. Wave 0 RED, Wave 1 impl, Wave 2 surface updates (catalog + docs). Wave 2 may consolidate into one plan.

### Claude's Discretion

- Exact rustdoc wording for `ImageSource::InlineSvg` and constructors (must satisfy D-12 substance; prose follows existing `AvatarProps`/`ImageProps` doc style)
- Exact description text in MCP catalog and `COMPONENT_CATALOG` (must satisfy D-13/D-14)
- Whether to add `ComponentNode::image_svg(key, svg, alt)` sibling factory (recommended: yes, trivial)
- Whether ambiguous-input rejection test uses `serde_json::from_value` returning `Err` (recommended: yes, `.expect_err("ambiguous source")`)
- Whether `alt`-escape test for InlineSvg uses an injection string (recommended: yes, `" onload="alert(1)` — symmetry with `image_xss_src_escaped`)

### Deferred Ideas (OUT OF SCOPE)

- `Component::Chart` with structured data
- `Component::MarkdownEmbed`
- SVG sanitization helper in framework
- `figure`/`figcaption` wrapper variant
- `class`/`id`/`style` props on the SVG wrapper `<div>`
- `data_path` binding for SVG string
- Configurable wrapper element
- Cross-repo automation for consuming-repo updates
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| IMG-SRC-01 | `ImageSource` enum introduced (`Url {src}` / `InlineSvg {svg}`) with serde untagged discrimination | D-01: `#[serde(untagged)]` struct variants; serde stdlib handles discrimination by field presence |
| IMG-SRC-02 | `ImageProps` refactored to flatten `source: ImageSource`; `alt` stays required | D-02: `#[serde(flatten)]` on `source` field; backward-compat verified by existing `image_round_trips` and `all_known_types_round_trip` tests |
| IMG-SRC-03 | `render_image` branches on source; URL path unchanged (XSS escape test stays green); InlineSvg emits `<div role="img" aria-label="{escaped alt}">{svg verbatim}</div>` | D-05/D-06: `render_stat_card` precedent for verbatim SVG injection; `html_escape` already available |
| IMG-SRC-04 | `COMPONENT_CATALOG` `### Image` section added; MCP `CatalogComponent` for Image updated; `docs/src/json-ui/components.md` `### Image` section added with safety callout | D-13/D-14/D-15: pre-existing gaps closed; catalog count stays 41 |
| IMG-SRC-05 | CI gate green: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | All waves accumulate into this gate at Wave 2 completion |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ImageSource enum + ImageProps refactor | ferro-json-ui (component.rs) | — | Data model lives in the component definition file |
| Render branching (URL vs InlineSvg) | ferro-json-ui (render.rs) | — | HTML generation lives in the renderer |
| Runtime COMPONENT_CATALOG string | ferro-json-ui (lib.rs) | — | Catalog is a pub const in the crate root |
| MCP catalog description | ferro-mcp (json_ui_catalog.rs) | — | MCP introspection layer, separate crate |
| User-facing docs | docs/src/json-ui/components.md | — | Documentation site, not a Rust crate |

---

## Standard Stack

### Core (already present — no new deps)

| Item | Version | Purpose | Status |
|------|---------|---------|--------|
| `serde` with `#[serde(untagged)]` | workspace | Struct-variant enum discrimination by field presence | VERIFIED: in use throughout `component.rs` |
| `serde_json` | workspace | Round-trip testing, `from_value`/`to_value` | VERIFIED: used in all component tests |
| `html_escape` fn | internal | XSS-safe attribute emission | VERIFIED: `render.rs` uses it on every dynamic string |

No new crates. No `Cargo.toml` changes needed.

---

## Architecture Patterns

### System Architecture Diagram

```
Rust caller
    │  ImageProps::url(src, alt)
    │  ImageProps::inline_svg(svg, alt)
    ▼
Component::Image(ImageProps { source: ImageSource, alt, ... })
    │
    │  serde: #[serde(flatten)] + #[serde(untagged)]
    ▼
JSON wire format
  URL:  {"type":"Image","src":"...","alt":"..."}     ← backward-compat
  SVG:  {"type":"Image","svg":"...","alt":"..."}     ← new
    │
    ▼
render_image(props: &ImageProps)
    ├─ ImageSource::Url { src }
    │    → <div class="relative w-full">
    │         <img src="{escaped}" alt="{escaped}" loading="lazy" ...>
    │      </div>
    └─ ImageSource::InlineSvg { svg }
         → <div class="relative w-full">
              <div role="img" aria-label="{escaped alt}">{svg verbatim}</div>
           </div>
```

### Recommended Project Structure (files touched)

```
ferro-json-ui/src/
├── component.rs   ← ImageSource enum + ImageProps refactor + constructors + rustdoc
├── render.rs      ← render_image gains InlineSvg branch + inline safety comment
└── lib.rs         ← COMPONENT_CATALOG: add ### Image section

ferro-mcp/src/tools/
└── json_ui_catalog.rs  ← update existing CatalogComponent for "Image"

docs/src/json-ui/
└── components.md  ← add ### Image section with safety callout
```

### Pattern 1: Serde Untagged Enum with Flatten (the core design)

**What:** `#[serde(untagged)]` on `ImageSource` + `#[serde(flatten)]` on the `source` field in `ImageProps`. Serde tries each variant in declaration order; first match wins. Discrimination is by field presence: `src` key → `Url`, `svg` key → `InlineSvg`.

**When to use:** When you need a "one-of-two-shapes" at the top JSON level without a wrapper key, and the two shapes are distinguishable by which field is present.

**Wire format output verified:**
- `ImageProps { source: ImageSource::Url { src: "/img/s.png".into() }, alt: "Alt".into(), ... }` serializes to `{"src":"/img/s.png","alt":"Alt"}` (then `serialize_tagged` injects `"type":"Image"` to get `{"type":"Image","src":"/img/s.png","alt":"Alt"}`)
- `ImageProps { source: ImageSource::InlineSvg { svg: "<svg>…</svg>".into() }, alt: "Chart".into(), ... }` serializes to `{"type":"Image","svg":"<svg>…</svg>","alt":"Chart"}`

**Key serde subtlety:** With `#[serde(untagged)]` + `#[serde(flatten)]`, the two struct variants must have disjoint fields. `Url` has field `src`; `InlineSvg` has field `svg`. They are disjoint, so the discriminator is unambiguous. Input with both `src` and `svg` present will match `Url` (first variant wins in untagged), but the `svg` field will be ignored — NOT rejected. To enforce rejection of ambiguous input, the test must construct `from_value` with both fields and assert the overall `ImageProps::source` arm produces `Url` (src wins) while a separate "neither src nor svg" test asserts deserialization fails.

**Correction to CONTEXT.md D-10 framing:** CONTEXT.md says "both src + svg rejected." In serde's untagged enum, the first matching variant wins silently — it does not error on extra unknown fields by default. Exact ambiguous-input behavior depends on whether `#[serde(deny_unknown_fields)]` is used on `ImageSource` variants. Without it, "both src + svg" will silently deserialize as `Url` (src matched first). The test SHOULD verify: (a) `src`-only → `Url`, (b) `svg`-only → `InlineSvg`, (c) neither → `Err`. The "both" case deserializes as `Url` unless `deny_unknown_fields` is added. **The planner must decide: add `deny_unknown_fields` to make "both" an error, or document "first-variant-wins" as the specified behavior.** [ASSUMED: current plan is to treat "both fields present" as a `Url` match — the planner should confirm or add `deny_unknown_fields`.]

**Example (struct definition):**
```rust
// Source: CONTEXT.md D-01 (locked decision)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ImageSource {
    Url { src: String },
    InlineSvg { svg: String },
}

pub struct ImageProps {
    #[serde(flatten)]
    pub source: ImageSource,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder_label: Option<String>,
}
```

### Pattern 2: Verbatim SVG Injection (existing precedent)

**What:** Raw SVG string emitted directly into HTML without `html_escape`. This pattern already exists in the codebase.

**Precedents in ferro-json-ui/src/render.rs:**
- `render_stat_card` line 2753: `html.push_str(&format!("<span class=\"inline-block mb-2 w-6 h-6\">{icon}</span>"))` — `icon` is `props.icon` (a `String`), pushed verbatim with `// raw` comment [VERIFIED: read at lines 2751-2755]
- `BREADCRUMB_SEP` const (lines 2475-2479): inline SVG literal baked directly into rendered markup [VERIFIED]

**Phase 148 promotes this to a first-class source variant on Image.** The pattern is established; Phase 148 makes it explicit with rustdoc + render-site comment.

**Safety contract in render_image InlineSvg branch:**
```rust
// SAFETY: svg is emitted verbatim without html_escape. This is the one
// deliberate html_escape bypass in render_image. Intended for
// server-constructed SVG (charts, icons) — not user-supplied strings.
// alt IS html_escape'd on both branches.
```

### Pattern 3: Convenience Constructors (existing precedent)

**What:** `impl ImageProps` with `url(src, alt)` and `inline_svg(svg, alt)` constructors.

**Precedents in the codebase:** `ImageProps` currently has no constructors (struct literals only). `ComponentNode::image(key, props)` factory exists at `component.rs:1719`.

**New constructors:**
```rust
impl ImageProps {
    pub fn url(src: impl Into<String>, alt: impl Into<String>) -> Self { ... }
    pub fn inline_svg(svg: impl Into<String>, alt: impl Into<String>) -> Self { ... }
}
```

### Anti-Patterns to Avoid

- **Adding `deny_unknown_fields` without deciding policy:** If the planner wants strict "both src+svg = error", add `#[serde(deny_unknown_fields)]` to each variant. If not, document first-variant-wins in the test. Do not leave it ambiguous — a test that says "ambiguous source" while the code silently passes is misleading.
- **Escaping the svg in render_image:** Do not call `html_escape(svg)` on the InlineSvg branch. That would corrupt the SVG by escaping `<`, `>`, `&` inside tag content.
- **Removing the placeholder for URL variant:** The placeholder fallback div is part of the URL variant's render contract (existing tests verify it). The InlineSvg variant intentionally omits it (per D-05).
- **Forgetting `all_known_types_round_trip` update:** The test at `component.rs:3719` has `("Image", r#"{"type":"Image","src":"/img/s.png","alt":"a"}"#)`. After the refactor this still works (Url variant). Add an InlineSvg entry alongside it.
- **Forgetting the `all_variants_count` fixture in component.rs:** The test at `component.rs:2180` asserts `components.len() == 27`. The Image entry at line 2173 uses the old struct-literal form and must be updated to `ImageProps::url(...)` or the new struct form. The count stays 27 (no new variant).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON field-presence discrimination | Custom `from_value` match logic | `#[serde(untagged)]` on `ImageSource` | serde handles correctly and maintains JsonSchema compat |
| Backward-compat wire format | Version field or wrapper key | `#[serde(flatten)]` on `source: ImageSource` | Keeps `{"src":"...","alt":"..."}` shape identical to today |
| HTML attribute escaping on alt | Custom escaper | Existing `html_escape` fn in render.rs | Already imported, already used on both attribute slots |

---

## Common Pitfalls

### Pitfall 1: Serde Untagged "both fields" behavior

**What goes wrong:** A test is written expecting `from_value({"src":"x","svg":"y"})` to return `Err`. It doesn't — serde's untagged discriminator matches `Url` on `src` and the `svg` field is ignored.

**Why it happens:** `#[serde(untagged)]` tries each variant in order; first match wins. It does not look at extra fields unless `deny_unknown_fields` is set on the variant.

**How to avoid:** Either: (a) add `#[serde(deny_unknown_fields)]` to each `ImageSource` variant — then serde rejects input with fields not in the matched variant. Or (b) accept first-variant-wins semantics and write the test to verify `src`-wins-over-`svg`. Decide before Wave 0 tests are written; the RED test must know which behavior to assert.

**Warning signs:** A test named `ambiguous_source_rejected` that passes at Wave 0 without `deny_unknown_fields` is probably asserting the wrong thing.

### Pitfall 2: `#[serde(flatten)]` + `#[serde(untagged)]` + `JsonSchema`

**What goes wrong:** `JsonSchema` derive may not support `#[serde(flatten)]` with `#[serde(untagged)]` cleanly — the generated schema may be incorrect or the derive may fail to compile.

**Why it happens:** `schemars` has known limitations with `#[serde(flatten)]` on enum fields. Whether `ImageSource` should derive `JsonSchema` at all is a question — the parent `ImageProps` is the thing that needs a schema, and `JsonSchema` on `ImageSource` only matters if the schema for `ImageProps` references it.

**How to avoid:** Check whether existing `ImageProps` participates in JSON schema generation (it derives `JsonSchema` today). If `ImageSource` also needs `JsonSchema`, test the compile. If the derive fails, drop `JsonSchema` from `ImageSource` and keep it on `ImageProps` with a manual `#[schemars(schema_with = "...")]` annotation, or accept a less-precise schema. [ASSUMED: `JsonSchema` on `ImageSource` will compile — schemars does support untagged enums with struct variants. Verify at Wave 1.]

**Warning signs:** Compile error mentioning `JsonSchema` or `schemars` on `ImageSource`.

### Pitfall 3: Forgetting to update the five in-tree struct literals

**What goes wrong:** Wave 1 introduces `ImageSource` but forgets to update all existing `ImageProps { src, alt, ... }` struct literals, causing compile errors.

**Why it happens:** There are exactly 5 struct literals: `component.rs:2173`, `render.rs:3758`, `render.rs:3780`, `render.rs:3798`. Plus `all_known_types_round_trip` at `component.rs:3719` (JSON string, not struct literal — still works after refactor since the URL variant preserves the `src` wire format, so this one requires no change).

**How to avoid:** At Wave 1, grep for `ImageProps {` and `props.src` in the codebase before closing the plan.

**Warning signs:** `error[E0560]: struct ImageProps has no field named src` at compile.

### Pitfall 4: Placeholder rendered for InlineSvg

**What goes wrong:** The `render_image` InlineSvg branch accidentally renders the placeholder div (skeleton fallback), making the SVG container taller than intended.

**Why it happens:** Cut-paste from the URL branch without removing the placeholder block.

**How to avoid:** Per D-05, placeholder is NOT rendered for InlineSvg. The placeholder div is only meaningful for `<img>` tags that can `onerror` to hide themselves. SVG is always present.

**Warning signs:** InlineSvg render test contains the placeholder div class `absolute inset-0 flex items-center`.

### Pitfall 5: `image_round_trips` test not extended

**What goes wrong:** The existing `image_round_trips` test at `component.rs:3696` asserts the URL variant only. After the refactor it still passes (backward compat). But the InlineSvg variant is never tested in that test function.

**How to avoid:** Wave 0 RED test explicitly adds an InlineSvg round-trip assertion. The test is extended, not replaced.

---

## Code Examples

### Verified render_image current shape (URL path)

```rust
// Source: ferro-json-ui/src/render.rs:2420-2447 [VERIFIED]
fn render_image(props: &ImageProps) -> String {
    let container_style = match &props.aspect_ratio {
        Some(ratio) => format!(" style=\"aspect-ratio: {}\"", html_escape(ratio)),
        None => String::new(),
    };
    let placeholder = match &props.placeholder_label {
        Some(label) => format!(
            "<div class=\"absolute inset-0 flex items-center justify-center \
             rounded-md bg-surface text-xs text-text-muted\">{}</div>",
            html_escape(label)
        ),
        None => String::from("<div class=\"absolute inset-0 rounded-md bg-surface\"></div>"),
    };
    format!(
        "<div class=\"relative w-full\"{container_style}>\
            {placeholder}\
            <img src=\"{src}\" alt=\"{alt}\" \
                 class=\"relative w-full h-full rounded-md object-cover object-top\" \
                 loading=\"lazy\" onerror=\"this.style.display='none'\">\
         </div>",
        src = html_escape(&props.src),
        alt = html_escape(&props.alt),
    )
}
```

After Wave 1, `props.src` becomes `match &props.source { ImageSource::Url { src } => ..., ImageSource::InlineSvg { svg } => ... }`.

### Target render_image InlineSvg branch shape

```rust
// Based on: CONTEXT.md D-05, precedent render_stat_card:2751-2755 [VERIFIED]
ImageSource::InlineSvg { svg } => {
    // SAFETY: svg is emitted verbatim without html_escape. This is the one
    // deliberate html_escape bypass in render_image. Intended for
    // server-constructed SVG (charts, icons) — not user-supplied strings.
    // alt IS html_escape'd below.
    format!(
        "<div class=\"relative w-full\"{container_style}>\
            <div role=\"img\" aria-label=\"{alt}\">{svg}</div>\
         </div>",
        alt = html_escape(&props.alt),
    )
}
```

### Current ImageProps (pre-refactor)

```rust
// Source: ferro-json-ui/src/component.rs:601-614 [VERIFIED]
pub struct ImageProps {
    pub src: String,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder_label: Option<String>,
}
```

### Current ComponentNode::image factory

```rust
// Source: ferro-json-ui/src/component.rs:1718-1726 [VERIFIED]
pub fn image(key: impl Into<String>, props: ImageProps) -> Self {
    Self {
        key: key.into(),
        component: Component::Image(props),
        action: None,
        visibility: None,
    }
}
```

The factory signature (`key, props: ImageProps`) is unchanged. A `ComponentNode::image_svg(key, svg, alt)` convenience sibling may be added at Claude's discretion.

### Current MCP CatalogComponent for "Image"

```rust
// Source: ferro-mcp/src/tools/json_ui_catalog.rs:1113-1134 [VERIFIED]
CatalogComponent {
    name: "Image".to_string(),
    description: "Image element with optional aspect ratio and skeleton placeholder fallback on load error.".to_string(),
    props: vec![
        prop("src", "String", true, "Image source URL"),
        prop("alt", "String", true, "Alt text for accessibility"),
        prop("aspect_ratio", "Option<String>", false, "CSS aspect ratio (e.g., \"16/9\")"),
        prop("placeholder_label", "Option<String>", false, "Label shown in the skeleton placeholder behind the image"),
    ],
    variants: None,
},
```

Wave 2 replaces this with a description covering both source variants and safety note. `src` becomes `Optional<String>` (required only when using URL variant) and `svg` is added as an alternative.

---

## In-Tree Call-Sites Inventory

All `ImageProps` struct literal usages that require rewriting in Wave 1:

| File | Line | Current form | Action |
|------|------|-------------|--------|
| `ferro-json-ui/src/component.rs` | 2173 | `ImageProps { src: "/img/screenshot.png"..., alt: "Page screenshot"..., ... }` | Rewrite to `ImageProps::url(...)` or new struct form |
| `ferro-json-ui/src/render.rs` | 3758 | `ImageProps { src: "/img/page.png"..., aspect_ratio: Some("16/9"...)... }` | Rewrite (test — Wave 0 RED may pre-update these) |
| `ferro-json-ui/src/render.rs` | 3780 | `ImageProps { src: "/img/page.png"..., aspect_ratio: None... }` | Rewrite |
| `ferro-json-ui/src/render.rs` | 3798 | `ImageProps { src: "x\" onerror=\"alert(1)"..., alt: "Test"... }` | Rewrite |
| `ferro-json-ui/src/component.rs` | 3719 (JSON) | `r#"{"type":"Image","src":"/img/s.png","alt":"a"}"#` | No change — URL variant wire format preserved |

Note: `framework/src/lib.rs:85` re-exports `ImageProps` by name only — no struct literal there, no change needed.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ImageProps { src: String, alt: String, ... }` | `ImageProps { source: ImageSource, alt: String, ... }` with `#[serde(flatten)]` | Phase 148 | Backward-compat: URL wire format preserved; SVG variant gained |
| SVG injection via `StatCardProps.icon: Option<String>` (ad-hoc) | `ImageSource::InlineSvg { svg: String }` (first-class) | Phase 148 | Pattern promoted from "an icon prop on two components" to a named source variant with a11y enforcement |

**No deprecated items to remove** (no old `src` field to keep as deprecated alias — D-04 is a direct rewrite).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `#[serde(untagged)]` + `#[serde(flatten)]` + `JsonSchema` derive on `ImageSource` compiles without special handling | Standard Stack, Pitfall 2 | Compile failure at Wave 1; fallback: drop `JsonSchema` from `ImageSource` or add `#[schemars(schema_with = ...)]` |
| A2 | "Both src and svg present" input silently matches `Url` (first variant wins) rather than returning an error | Pitfall 1, Code Examples | Test written expecting `Err` will incorrectly pass or fail; planner must decide: add `deny_unknown_fields` or document first-variant-wins |

---

## Open Questions

1. **Ambiguous-input rejection policy (A2 above)**
   - What we know: serde's `#[serde(untagged)]` without `deny_unknown_fields` matches first variant; "both fields" silently matches `Url`.
   - What's unclear: CONTEXT.md D-10 says "ambiguous input rejected" — this requires `deny_unknown_fields` on each `ImageSource` variant, which is a small addition.
   - Recommendation: Add `#[serde(deny_unknown_fields)]` to both `Url` and `InlineSvg` variants. This makes the rejection behavior explicit and testable. The Wave 0 RED test then correctly asserts `from_value({"src":"x","svg":"y",...}).expect_err(...)`.

2. **`JsonSchema` on `ImageSource`**
   - What we know: `ImageProps` currently derives `JsonSchema`. After the refactor, `ImageProps` has `#[serde(flatten)] pub source: ImageSource`. For `ImageProps` to derive `JsonSchema`, `ImageSource` must also implement `JsonSchema` (or use a workaround).
   - What's unclear: Whether `schemars` produces correct output for `#[serde(flatten)]` + `#[serde(untagged)]` combination in this version.
   - Recommendation: Keep `JsonSchema` on `ImageSource` (as shown in CONTEXT.md D-01). If it fails, the Wave 1 implementer drops `JsonSchema` from the derive list on `ImageSource` and uses `#[schemars(schema_with = "...")]` on the `source` field, or removes `JsonSchema` from `ImageProps` entirely (it is not used for runtime validation).

---

## Environment Availability

Step 2.6: SKIPPED — Phase 148 is pure Rust code/documentation changes with no external tool dependencies beyond the workspace build environment.

---

## Validation Architecture

`workflow.nyquist_validation` key is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`#[test]`) |
| Config file | None — `cargo test` discovers tests in `#[cfg(test)]` modules |
| Quick run command | `cargo test -p ferro-json-ui image` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File | Wave |
|--------|----------|-----------|-------------------|------|------|
| IMG-SRC-01 | `ImageSource::Url { src }` deserializes from `{"src":"..."}` | unit | `cargo test -p ferro-json-ui image_source_url_roundtrip` | `ferro-json-ui/src/component.rs` (new) | Wave 0 RED |
| IMG-SRC-01 | `ImageSource::InlineSvg { svg }` deserializes from `{"svg":"..."}` | unit | `cargo test -p ferro-json-ui image_source_inline_svg_roundtrip` | `ferro-json-ui/src/component.rs` (new) | Wave 0 RED |
| IMG-SRC-01 | Input with neither `src` nor `svg` is rejected | unit | `cargo test -p ferro-json-ui image_source_neither_rejected` | `ferro-json-ui/src/component.rs` (new) | Wave 0 RED |
| IMG-SRC-02 | `ImageProps::url(src, alt)` constructor produces correct shape | unit | `cargo test -p ferro-json-ui image_props_url_constructor` | `ferro-json-ui/src/component.rs` (new) | Wave 0 RED |
| IMG-SRC-02 | `ImageProps::inline_svg(svg, alt)` constructor produces correct shape | unit | `cargo test -p ferro-json-ui image_props_inline_svg_constructor` | `ferro-json-ui/src/component.rs` (new) | Wave 0 RED |
| IMG-SRC-02 | Full `Component::Image` URL round-trip (`all_known_types_round_trip`) still green | unit | `cargo test -p ferro-json-ui all_known_types_round_trip` | `ferro-json-ui/src/component.rs` (extend) | Wave 0 RED |
| IMG-SRC-03 | URL variant renders `<img src="{escaped}" alt="{escaped}">` (existing) | unit | `cargo test -p ferro-json-ui image_with_aspect_ratio` | `ferro-json-ui/src/render.rs` (update struct literals) | Wave 0 RED |
| IMG-SRC-03 | XSS in `src` attribute is escaped (existing `image_xss_src_escaped`) | unit | `cargo test -p ferro-json-ui image_xss_src_escaped` | `ferro-json-ui/src/render.rs` (update struct literal) | Wave 0 RED |
| IMG-SRC-03 | InlineSvg variant renders `<div role="img" aria-label="...">` | unit | `cargo test -p ferro-json-ui inline_svg_renders_div_role_img` | `ferro-json-ui/src/render.rs` (new) | Wave 0 RED |
| IMG-SRC-03 | Inline SVG `<script>` tag passes through unescaped (bypass confirmed) | unit | `cargo test -p ferro-json-ui inline_svg_with_script_passes_through` | `ferro-json-ui/src/render.rs` (new) | Wave 0 RED |
| IMG-SRC-03 | `alt` text with injection chars is escaped in InlineSvg branch | unit | `cargo test -p ferro-json-ui inline_svg_alt_xss_escaped` | `ferro-json-ui/src/render.rs` (new) | Wave 0 RED |
| IMG-SRC-04 | MCP `test_all_components_present` still passes at 41 | unit | `cargo test -p ferro-mcp test_all_components_present` | `ferro-mcp/src/tools/json_ui_catalog.rs` (no count change) | Wave 2 |
| IMG-SRC-05 | CI gate: fmt + clippy + test all green | integration | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | all | Wave 2 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui image`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full CI command (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`) green before `/gsd-verify-work`

### Wave 0 Gaps

Tests that must be written RED in Wave 0 (these files exist; the test functions are new):

- [ ] `ferro-json-ui/src/component.rs` — add `image_source_url_roundtrip`, `image_source_inline_svg_roundtrip`, `image_source_neither_rejected`, `image_props_url_constructor`, `image_props_inline_svg_constructor` test functions in the existing `image_round_trips` test region
- [ ] `ferro-json-ui/src/render.rs` — add `inline_svg_renders_div_role_img`, `inline_svg_with_script_passes_through`, `inline_svg_alt_xss_escaped` test functions in the existing Image render tests section (lines 3752-3809)
- [ ] Update existing render test struct literals (lines 3758, 3780, 3798) to compile with the new `ImageProps` shape — these are Wave 0 changes too since the render tests call `ImageProps` directly

No new test files needed. No new test framework to install. Existing `#[cfg(test)]` modules in both files suffice.

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes — `alt` attribute | `html_escape` (existing, enforced on both branches) |
| V5 Input Validation | Yes — `svg` body | Deliberate bypass; documented in rustdoc + inline comment + docs + catalog |
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | No | — |
| V6 Cryptography | No | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via `alt` attribute | Tampering | `html_escape` on both URL and InlineSvg branches — enforced, unchanged |
| SVG-based XSS via caller-supplied SVG | Tampering | Deliberate bypass; scoped to server-constructed SVG only; documented at 5 sites (rustdoc × 2, inline comment, COMPONENT_CATALOG, MCP catalog, docs); callers responsible for sanitization if input contains user data |
| Script injection via `src` attribute | Tampering | `html_escape` on URL variant — enforced, unchanged; `image_xss_src_escaped` test stays green |

**Security posture of Phase 148 relative to existing code:** The InlineSvg bypass is not a new category of risk — `render_stat_card` already injects `props.icon` verbatim (lines 2751-2755). Phase 148 makes the bypass explicit, named, documented, and tested rather than ad-hoc.

---

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/src/component.rs:601-614` — current `ImageProps` struct [VERIFIED: read]
- `ferro-json-ui/src/component.rs:2173-2178` — only struct-literal in the `all_known_types` fixture [VERIFIED: read]
- `ferro-json-ui/src/component.rs:3696-3707` — `image_round_trips` test [VERIFIED: read]
- `ferro-json-ui/src/component.rs:3710-3749` — `all_known_types_round_trip` test fixture [VERIFIED: read]
- `ferro-json-ui/src/render.rs:2420-2447` — `render_image` current implementation [VERIFIED: read]
- `ferro-json-ui/src/render.rs:2748-2780` — `render_stat_card` SVG injection precedent [VERIFIED: read]
- `ferro-json-ui/src/render.rs:3754-3809` — all three existing Image render tests [VERIFIED: read]
- `ferro-json-ui/src/render.rs:188-194` — `collect_plugin_types_node` leaf OR-chain [VERIFIED: read]
- `ferro-json-ui/src/lib.rs:103-179` — `COMPONENT_CATALOG` string (Image section absent) [VERIFIED: read]
- `ferro-mcp/src/tools/json_ui_catalog.rs:1113-1134` — existing Image `CatalogComponent` [VERIFIED: read]
- `ferro-mcp/src/tools/json_ui_catalog.rs:1208-1264` — `test_all_components_present` at 41 [VERIFIED: read]
- `docs/src/json-ui/components.md:623-654` — Avatar section (density to mirror) [VERIFIED: read]
- `.planning/phases/148-image-inline-svg-source/148-CONTEXT.md` — all locked decisions [VERIFIED: read]
- `.planning/config.json` — `workflow.nyquist_validation` absent = enabled [VERIFIED: read]

### Secondary (MEDIUM confidence)

- `.planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/147-RESEARCH.md` — wave shape precedent [VERIFIED: read]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all patterns verified in codebase
- Architecture: HIGH — five files identified, all read and understood
- Pitfalls: HIGH — serde untagged behavior is the one non-obvious area; documented precisely
- Safety framing: HIGH — precedents verified in render.rs; five required sites enumerated

**Research date:** 2026-04-24
**Valid until:** 2026-05-24 (stable; no fast-moving external dependencies)

---

## RESEARCH COMPLETE

**Phase:** 148 — image-inline-svg-source
**Confidence:** HIGH

### Key Findings

1. **No new crates, no new dependencies.** All patterns (`#[serde(untagged)]`, `#[serde(flatten)]`, `html_escape`, verbatim SVG injection) already exist in the codebase. Phase 148 is a refactor + extension of existing machinery.

2. **Five in-tree struct literals need rewriting** in Wave 1 (4 in render.rs tests + 1 in component.rs fixture). The `all_known_types_round_trip` JSON string at component.rs:3719 needs no change — the URL wire format is backward-compatible.

3. **MCP catalog count stays at 41.** `test_all_components_present` asserts 41 and lists "Image" already. Phase 148 updates the description/props of the existing Image entry only — no count bump, no list change.

4. **One open question for the planner:** The serde untagged `deny_unknown_fields` policy for "both src and svg" input (A2). Recommendation: add `#[serde(deny_unknown_fields)]` to both variants to make rejection explicit and testable per CONTEXT.md D-10's stated intent.

5. **Safety framing is a first-class deliverable at 5 sites:** rustdoc on `ImageSource::InlineSvg` + rustdoc on `ImageProps::inline_svg` + inline comment in `render_image` + `COMPONENT_CATALOG ### Image` + MCP catalog description + docs `### Image` section. The planner must track all five sites explicitly in plan tasks.

### File Created

`.planning/phases/148-image-inline-svg-source/148-RESEARCH.md`

### Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Standard stack | HIGH | All verified in codebase reads |
| Architecture | HIGH | Five files fully read, line numbers confirmed |
| Serde untagged behavior | HIGH | Pattern well-understood; one assumption about deny_unknown_fields flagged |
| Pitfalls | HIGH | Derived from code reads, not training data alone |
| Safety / security | HIGH | Two precedent sites in render.rs verified |

### Open Questions

- `deny_unknown_fields` on `ImageSource` variants — planner must decide before Wave 0 tests
- `JsonSchema` derive compatibility on `ImageSource` with `#[serde(flatten)]` — verify at Wave 1 (fallback path documented)

### Ready for Planning

Research complete. Planner can create PLAN.md files following Phase 147's wave shape: Wave 0 RED → Wave 1 impl → Wave 2 surface updates.
