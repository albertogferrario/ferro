# Phase 148: image-inline-svg-source — Pattern Map

**Mapped:** 2026-04-24
**Files analyzed:** 5 modified
**Analogs found:** 5 / 5 (100% — every surface has a direct in-crate precedent)

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/component.rs` | new enum + struct refactor + constructors + tests | request-response (shape) | `Visibility` untagged enum (`visibility.rs:43-50`); `ComponentNode` flatten (`component.rs:1323-1332`); `AvatarProps` optional-src shape (`component.rs:617-626`) | exact |
| `ferro-json-ui/src/render.rs` | render function branch + new tests | request-response (HTML emit) | `render_image` URL path (`render.rs:2420-2447`); `render_stat_card` verbatim SVG injection (`render.rs:2748-2780`) | exact |
| `ferro-json-ui/src/lib.rs` | `COMPONENT_CATALOG` string entry added | public API catalog | `### Avatar` entry in `COMPONENT_CATALOG` (`lib.rs:167-168`); `### Separator` entry (`lib.rs:149`) | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | update existing `CatalogComponent` for "Image" | catalog entry | existing `CatalogComponent { name: "Image", … }` (`json_ui_catalog.rs:1113-1134`); `CatalogComponent { name: "Avatar", … }` pattern nearby | exact |
| `docs/src/json-ui/components.md` | new `### Image` doc section | documentation | `### Avatar` section (`components.md:623-654`); `### StatCard` icon-prop safety prose (`components.md:795-843`) | exact |

---

## Pattern Assignments

### 1. `ferro-json-ui/src/component.rs`

This file receives five discrete edits. Each has a named analog.

---

#### 1a. `ImageSource` untagged enum — new type

**Analog:** `Visibility` in `ferro-json-ui/src/visibility.rs:43-50`

This is the closest in-crate example of `#[serde(untagged)]` on an enum with struct variants where discrimination is by field presence.

**Pattern to copy — `Visibility` untagged enum shape** (`visibility.rs:37-50`):

```rust
/// Visibility rule with logical composition support.
///
/// Uses `#[serde(untagged)]` to support clean JSON:
/// - Simple: `{"path": "/data/users", "operator": "not_empty"}`
/// - Compound: `{"and": [...]}`
/// - Nested: `{"not": {"path": ..., "operator": ...}}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Visibility {
    And { and: Vec<Visibility> },
    Or { or: Vec<Visibility> },
    Not { not: Box<Visibility> },
    Condition(VisibilityCondition),
}
```

**Concrete `ImageSource` emission (D-01, D-12):**

```rust
/// Source for an [`ImageProps`] component — exactly one of `src` or `svg` must be set.
///
/// Discrimination is by field presence in the JSON wire format:
/// - `{"src": "..."}` → `Url` variant
/// - `{"svg": "..."}` → `InlineSvg` variant
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ImageSource {
    /// External image URL. The `src` attribute is HTML-escaped before emission.
    Url { src: String },
    /// Server-constructed inline SVG string.
    ///
    /// # Safety
    ///
    /// The `svg` value is emitted **verbatim without HTML escaping**. This variant is
    /// intended for server-constructed SVG (charts, sparklines, icons) — not for
    /// user-supplied strings. Callers that incorporate user data into the SVG output
    /// are responsible for sanitization before constructing this variant.
    ///
    /// Contrast with `Url`: the `src` attribute is always HTML-escaped as an attribute
    /// value, so `Url` is safe with caller-controlled URL strings.
    InlineSvg { svg: String },
}
```

**Note on `deny_unknown_fields`:** Per RESEARCH.md Pitfall 1 and Open Question 1, add `#[serde(deny_unknown_fields)]` to both struct variants. This makes input with both `src` and `svg` fields present return a deserialization error, matching CONTEXT.md D-10's stated intent ("ambiguous input is rejected"):

```rust
#[serde(untagged)]
pub enum ImageSource {
    Url {
        #[serde(deny_unknown_fields)]  // rejects if svg is also present
        src: String,
    },
    InlineSvg {
        #[serde(deny_unknown_fields)]  // rejects if src is also present
        svg: String,
    },
}
```

Note: `deny_unknown_fields` on `#[serde(untagged)]` struct variants applies to each variant independently. The planner should verify this behavior in Rust during Wave 0.

---

#### 1b. `ImageProps` refactor — `#[serde(flatten)]` on `source` field

**Analog:** `ComponentNode` in `ferro-json-ui/src/component.rs:1323-1332`

`ComponentNode` uses `#[serde(flatten)]` on the `component: Component` field — the identical pattern of flattening an enum into the parent JSON object without a wrapper key.

**Pattern to copy — `ComponentNode` flatten** (`component.rs:1323-1332`):

```rust
// JsonSchema skipped: contains Component via flatten — Component has custom Serialize/Deserialize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentNode {
    pub key: String,
    #[serde(flatten)]
    pub component: Component,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
}
```

**Concrete `ImageProps` refactor** (`component.rs:601-614`, post-Wave 1):

```rust
/// Props for Image component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImageProps {
    /// Image source — exactly one of `src` (URL) or `svg` (inline SVG) must be present.
    #[serde(flatten)]
    pub source: ImageSource,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// Optional label shown in a skeleton placeholder that sits behind the
    /// image. When the image fails to load (or is still being generated),
    /// the `<img>` is hidden via `onerror` and the placeholder remains
    /// visible, keeping the container at its aspect-ratio size.
    /// Not rendered for the `InlineSvg` source variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder_label: Option<String>,
}
```

**Note on `JsonSchema`:** `ComponentNode` skips `JsonSchema` entirely due to the flatten. `ImageProps` currently derives `JsonSchema` and should attempt to keep it. If the `schemars` derive fails on `#[serde(flatten)] source: ImageSource`, the fallback is to remove `JsonSchema` from `ImageSource` and add `#[schemars(with = "serde_json::Value")]` on the `source` field, or remove `JsonSchema` from `ImageProps` (it is not used for runtime validation). See RESEARCH.md Pitfall 2.

---

#### 1c. `ImageProps` convenience constructors — new `impl` block

**Analog:** `ComponentNode` factory methods in `component.rs:1334-1344` (`ComponentNode::card`, etc.), and the `ComponentNode::image` factory at `component.rs:1719-1726`.

**Closest match — `ComponentNode::image` factory** (`component.rs:1718-1726`):

```rust
/// Create an Image component node.
pub fn image(key: impl Into<String>, props: ImageProps) -> Self {
    Self {
        key: key.into(),
        component: Component::Image(props),
        action: None,
        visibility: None,
    }
}
```

**Concrete `ImageProps::url` and `ImageProps::inline_svg` constructors (D-03):**

```rust
impl ImageProps {
    /// Construct an Image backed by an external URL.
    pub fn url(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            source: ImageSource::Url { src: src.into() },
            alt: alt.into(),
            aspect_ratio: None,
            placeholder_label: None,
        }
    }

    /// Construct an Image backed by a server-constructed inline SVG string.
    ///
    /// # Safety
    ///
    /// `svg` is emitted verbatim without HTML escaping. Intended for
    /// server-constructed SVG (charts, sparklines, icons). Not suitable for
    /// user-supplied strings. Callers that incorporate user data into `svg`
    /// are responsible for sanitization before calling this constructor.
    pub fn inline_svg(svg: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            source: ImageSource::InlineSvg { svg: svg.into() },
            alt: alt.into(),
            aspect_ratio: None,
            placeholder_label: None,
        }
    }
}
```

---

#### 1d. `ComponentNode::image_svg` convenience factory (at Claude's discretion — recommended yes)

**Analog:** `ComponentNode::image` factory at `component.rs:1718-1726`. Trivial sibling:

```rust
/// Create an Image component node backed by a server-constructed inline SVG.
///
/// `svg` is emitted verbatim — intended for server-constructed SVG only.
pub fn image_svg(key: impl Into<String>, svg: impl Into<String>, alt: impl Into<String>) -> Self {
    Self {
        key: key.into(),
        component: Component::Image(ImageProps::inline_svg(svg, alt)),
        action: None,
        visibility: None,
    }
}
```

---

#### 1e. Call-site rewrites and test updates

**In-tree struct literals to rewrite (see RESEARCH.md Pitfall 3 inventory):**

| File | Line | Old form | New form |
|------|------|----------|----------|
| `component.rs` | 2173 | `ImageProps { src: "/img/screenshot.png"..., alt: "Page screenshot"..., ... }` | `ImageProps::url("/img/screenshot.png", "Page screenshot")` |
| `render.rs` | 3758 | `ImageProps { src: "/img/page.png"..., aspect_ratio: Some("16/9"...)... }` | `ImageProps { source: ImageSource::Url { src: "...".into() }, alt: "...".into(), aspect_ratio: Some("16/9".into()), placeholder_label: None }` or `ImageProps::url(...)` with chained field |
| `render.rs` | 3780 | `ImageProps { src: "/img/page.png"..., aspect_ratio: None... }` | `ImageProps::url("/img/page.png", "Page")` |
| `render.rs` | 3798 | `ImageProps { src: "x\" onerror=\"alert(1)"..., alt: "Test"... }` | `ImageProps::url("x\" onerror=\"alert(1)", "Test")` |

The `all_known_types_round_trip` JSON string at `component.rs:3719` needs no change — the URL wire format is preserved.

**`image_round_trips` test extension** (Wave 0 RED — extends `component.rs:3695-3707`):

```rust
#[test]
fn image_round_trips() {
    // Existing URL variant — stays green
    let json = r#"{"type": "Image", "src": "/img/s.png", "alt": "Screenshot"}"#;
    let component: Component = serde_json::from_str(json).unwrap();
    match component {
        Component::Image(props) => {
            assert!(matches!(props.source, ImageSource::Url { .. }));
            assert_eq!(props.alt, "Screenshot");
            assert!(props.aspect_ratio.is_none());
        }
        _ => panic!("expected Image"),
    }

    // InlineSvg variant — new assertion
    let json_svg = r#"{"type": "Image", "svg": "<svg></svg>", "alt": "Chart"}"#;
    let component_svg: Component = serde_json::from_str(json_svg).unwrap();
    match component_svg {
        Component::Image(props) => {
            assert!(matches!(props.source, ImageSource::InlineSvg { .. }));
            assert_eq!(props.alt, "Chart");
        }
        _ => panic!("expected Image"),
    }

    // Neither src nor svg — must fail
    let json_neither = r#"{"type": "Image", "alt": "Bad"}"#;
    serde_json::from_str::<Component>(json_neither)
        .expect_err("should reject input with neither src nor svg");
}
```

---

### 2. `ferro-json-ui/src/render.rs`

#### 2a. `render_image` branch on `props.source`

**Analog (URL path):** Existing `render_image` at `render.rs:2420-2447`. Unchanged for the `Url` branch.

**Analog (verbatim SVG injection):** `render_stat_card` at `render.rs:2748-2780`, specifically the icon injection block:

```rust
// render.rs:2751-2756
if let Some(ref icon) = props.icon {
    html.push_str(&format!(
        "<span class=\"inline-block mb-2 w-6 h-6\">{icon}</span>"
    ));
    // raw
}
```

**Pattern to copy — existing `render_image` URL path** (`render.rs:2420-2447`):

```rust
fn render_image(props: &ImageProps) -> String {
    let container_style = match &props.aspect_ratio {
        Some(ratio) => format!(" style=\"aspect-ratio: {}\"", html_escape(ratio)),
        None => String::new(),
    };

    // Placeholder sits behind the image in the same box. When the `<img>`
    // fails to load, onerror hides it so the placeholder remains visible.
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

**Concrete `render_image` post-Wave 1 shape (D-05, D-06):**

```rust
fn render_image(props: &ImageProps) -> String {
    let container_style = match &props.aspect_ratio {
        Some(ratio) => format!(" style=\"aspect-ratio: {}\"", html_escape(ratio)),
        None => String::new(),
    };

    match &props.source {
        ImageSource::Url { src } => {
            // Placeholder sits behind the image in the same box. When the `<img>`
            // fails to load, onerror hides it so the placeholder remains visible.
            let placeholder = match &props.placeholder_label {
                Some(label) => format!(
                    "<div class=\"absolute inset-0 flex items-center justify-center \
                     rounded-md bg-surface text-xs text-text-muted\">{}</div>",
                    html_escape(label)
                ),
                None => {
                    String::from("<div class=\"absolute inset-0 rounded-md bg-surface\"></div>")
                }
            };
            format!(
                "<div class=\"relative w-full\"{container_style}>\
                    {placeholder}\
                    <img src=\"{src}\" alt=\"{alt}\" \
                         class=\"relative w-full h-full rounded-md object-cover object-top\" \
                         loading=\"lazy\" onerror=\"this.style.display='none'\">\
                 </div>",
                src = html_escape(src),
                alt = html_escape(&props.alt),
            )
        }
        ImageSource::InlineSvg { svg } => {
            // SAFETY: svg is emitted verbatim without html_escape. This is the one
            // deliberate html_escape bypass in render_image. Intended for
            // server-constructed SVG (charts, sparklines, icons) — not user-supplied
            // strings. alt IS html_escape'd below.
            format!(
                "<div class=\"relative w-full\"{container_style}>\
                    <div role=\"img\" aria-label=\"{alt}\">{svg}</div>\
                 </div>",
                alt = html_escape(&props.alt),
            )
        }
    }
}
```

Key differences from URL path: no placeholder div, `svg` is unescaped, wrapper is `<div role="img" aria-label="...">` not `<img>`.

---

#### 2b. New render tests for `InlineSvg` branch

**Analog:** Existing Image render tests at `render.rs:3752-3809`. Wave 0 adds three new functions alongside the three existing ones.

**Pattern to copy — `image_xss_src_escaped` test** (`render.rs:3794-3809`):

```rust
#[test]
fn image_xss_src_escaped() {
    let view = JsonUiView::new().component(ComponentNode {
        key: "img".to_string(),
        component: Component::Image(ImageProps {
            src: "x\" onerror=\"alert(1)".to_string(),
            alt: "Test".to_string(),
            aspect_ratio: None,
            placeholder_label: None,
        }),
        action: None,
        visibility: None,
    });
    let html = render_to_html(&view, &json!({}));
    assert!(html.contains("src=\"x&quot; onerror=&quot;alert(1)\""));
}
```

**New InlineSvg render tests (Wave 0 RED pattern):**

```rust
#[test]
fn inline_svg_renders_div_role_img() {
    let view = JsonUiView::new().component(ComponentNode::image(
        "chart",
        ImageProps::inline_svg("<svg><rect/></svg>", "Revenue chart"),
    ));
    let html = render_to_html(&view, &json!({}));
    assert!(html.contains("<div role=\"img\""));
    assert!(html.contains("aria-label=\"Revenue chart\""));
    assert!(html.contains("<svg><rect/></svg>"));
    assert!(!html.contains("<img"));
}

#[test]
fn inline_svg_with_script_passes_through() {
    // Load-bearing test: asserts the deliberate html_escape bypass is working.
    // A <script> tag inside the SVG passes through unescaped. This is intentional
    // (server-constructed SVG may legitimately use script for animations).
    let svg = "<svg><script>alert(1)</script></svg>";
    let view = JsonUiView::new().component(ComponentNode::image(
        "chart",
        ImageProps::inline_svg(svg, "Chart"),
    ));
    let html = render_to_html(&view, &json!({}));
    assert!(html.contains("<script>alert(1)</script>"));
    assert!(!html.contains("&lt;script&gt;"));
}

#[test]
fn inline_svg_alt_xss_escaped() {
    // alt IS escaped even for the InlineSvg variant.
    let view = JsonUiView::new().component(ComponentNode::image(
        "chart",
        ImageProps::inline_svg("<svg/>", "\" onload=\"alert(1)"),
    ));
    let html = render_to_html(&view, &json!({}));
    assert!(html.contains("aria-label=\"&quot; onload=&quot;alert(1)\""));
}
```

---

### 3. `ferro-json-ui/src/lib.rs`

#### `COMPONENT_CATALOG` — add `### Image` section

**Insertion point:** Between `### Avatar` (line 167) and `### Skeleton` (line 170), or near the Separator/Avatar cluster. Keep density matching existing one-liner entries.

**Analog — `### Avatar` entry** (`lib.rs:167-168`):

```
### Avatar
Props: src (Option<String>), alt (String), fallback (Option<String>), size (Option<xs|sm|default|lg>)
```

**Analog — `### Separator` entry** (`lib.rs:149`):

```
### Separator
Props: orientation (Option<horizontal|vertical>)
```

**Concrete `### Image` section to add (D-13):**

```
### Image
Props: src OR svg (exactly one required — URL or inline SVG), alt (String), aspect_ratio (Option<String>), placeholder_label (Option<String>)
URL variant: src (String) — image source URL; attribute is HTML-escaped
SVG variant: svg (String) — server-constructed inline SVG emitted verbatim; not for user input
```

This matches the two-line density of `### StatCard` / `### DescriptionList` entries when both source variants need documentation.

---

### 4. `ferro-mcp/src/tools/json_ui_catalog.rs`

#### Update existing `CatalogComponent` for "Image"

**Target (current shape, `json_ui_catalog.rs:1113-1134`):**

```rust
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

**Analog for dual-source description style:** The `CatalogComponent` for `KanbanBoard` (nearby in the file) uses a longer description to explain multiple behaviors. The `prop()` helper signature (`name, type_name, required, description`) is at `json_ui_catalog.rs:1194-1201`.

**Concrete updated entry (D-14) — count stays 41, no new component:**

```rust
CatalogComponent {
    name: "Image".to_string(),
    description: "Bounded visual asset rendered into a box. Accepts either an external URL \
                  (src) or a server-constructed inline SVG string (svg) — exactly one must \
                  be set. The URL variant HTML-escapes the src attribute; the SVG variant \
                  emits the svg string verbatim (intended for server-constructed SVG — \
                  charts, sparklines, icons — not user input). alt is required on both \
                  variants (compile-enforced accessibility). placeholder_label applies \
                  to the URL variant only."
        .to_string(),
    props: vec![
        prop("src", "String", false, "Image source URL (URL variant — use when svg is absent)"),
        prop("svg", "String", false, "Inline SVG string emitted verbatim (SVG variant — use when src is absent). Server-constructed only."),
        prop("alt", "String", true, "Alt text for accessibility — required on both source variants"),
        prop("aspect_ratio", "Option<String>", false, "CSS aspect ratio (e.g., \"16/9\")"),
        prop("placeholder_label", "Option<String>", false, "Label shown in the skeleton placeholder — URL variant only"),
    ],
    variants: None,
},
```

Note: `src` and `svg` are both marked `required: false` in the `prop()` call because the real constraint is "exactly one of" — neither is individually required in the type system. The description carries the constraint.

---

### 5. `docs/src/json-ui/components.md`

#### New `### Image` section

**Analog — `### Avatar` section** (`components.md:623-654`): density to mirror — props table, Rust example, JSON output.

**Analog — `### StatCard` icon prose** (`components.md:795-843`): shows how to document an SVG-accepting prop with a usage example.

**Concrete `### Image` section structure (D-15):**

The section must include:
1. Opening paragraph describing the dual-source concept.
2. Props table covering the flattened wire shape (not the Rust enum structure): `src` (optional, URL variant), `svg` (optional, SVG variant), `alt` (required), `aspect_ratio` (optional), `placeholder_label` (optional).
3. Safety callout as a blockquote on the `svg` field.
4. Rust examples using `ImageProps::url(...)` and `ImageProps::inline_svg(...)` constructors.
5. JSON output examples for both variants.
6. Use-case list for the SVG variant.
7. Explicit "For rendering HTML (not SVG), no generic escape hatch exists; author a narrower component."

**Pattern for safety callout (blockquote style, matching docs conventions):**

```markdown
> **Safety note — `svg` variant:** The `svg` value is emitted verbatim without HTML escaping.
> Intended for server-constructed SVG (charts, sparklines, icons). Not suitable for
> user-supplied strings. Callers that incorporate user data into the SVG output are
> responsible for sanitization.
```

**Rust example pattern (follows `### Avatar` style):**

```markdown
```rust
use ferro::{ComponentNode, ImageProps};

// URL variant
let url_node = ComponentNode::image("hero", ImageProps::url("/img/hero.png", "Hero image"));

// SVG variant — server-constructed chart
let svg = bar_chart_svg(&weekly_data, 800, 300);
let chart_node = ComponentNode::image_svg("revenue-chart", svg, "Incassi settimanali");
```
```

---

## Shared Patterns

### `html_escape` usage discipline

**Source:** `ferro-json-ui/src/render.rs` — every dynamic string in rendered HTML (attributes and content) passes through `html_escape(...)`. The one documented exception is the `svg` body in `render_image`'s `InlineSvg` branch (D-06).

**Apply to:** All render test assertions for attribute values. Both `src` and `alt` in the URL branch of `render_image`. `alt` in the InlineSvg branch. `svg` body is explicitly NOT escaped — tests must assert unescaped output.

### `#[serde(default, skip_serializing_if = "Option::is_none")]` on optional props

**Source:** Established throughout `component.rs` — every `Option<T>` field uses this pair. `aspect_ratio` and `placeholder_label` on `ImageProps` already use it. No change.

**Apply to:** `aspect_ratio` and `placeholder_label` stay unchanged. The new `source: ImageSource` field uses `#[serde(flatten)]` instead.

### Rustdoc safety note style

**Source:** No existing precedent in the codebase for a `# Safety` section in component rustdoc (this is the first deliberate bypass in `ferro-json-ui`). Prose style follows the existing `ImageProps` placeholder_label doc and the `ComponentNode` flatten comment.

**Apply to:** `ImageSource::InlineSvg` variant doc and `ImageProps::inline_svg` constructor doc (both required per D-12).

### Test assertion pattern for round-trips

**Source:** `all_known_types_round_trip` at `component.rs:3709-3750` — serialize to value, check `["type"]` field, reparse, check again.

**Apply to:** The InlineSvg entry added to `all_known_types_round_trip` must use the same two-step assert pattern: `serde_json::to_value(&component).unwrap()["type"] == "Image"` and `from_value(serialized)` round-trip.

---

## Files NOT Modified (deliberate)

| File | Reason |
|------|--------|
| `ferro-json-ui/src/resolve.rs` | D-09: All three resolver passes already handle `Component::Image(_)` in their leaf OR-chains. No new variant, no change. |
| `ferro-json-ui/src/render.rs:354` | D-07: `Component::Image(props) => render_image(props)` dispatch arm unchanged. |
| `ferro-json-ui/src/render.rs:193` | D-08: `collect_plugin_types_node` leaf list unchanged (Image already present). Count stays 41. |
| `Cargo.toml` files | No new dependencies. `#[serde(untagged)]`, `#[serde(flatten)]`, and `html_escape` all exist in the workspace. |
| `framework/src/lib.rs:85` | Re-exports `ImageProps` by name only — no struct literal, no change needed. |

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/`, `ferro-mcp/src/tools/`, `docs/src/json-ui/`
**Files scanned:** `component.rs`, `render.rs`, `visibility.rs`, `lib.rs`, `json_ui_catalog.rs`, `components.md`
**Pattern extraction date:** 2026-04-24
