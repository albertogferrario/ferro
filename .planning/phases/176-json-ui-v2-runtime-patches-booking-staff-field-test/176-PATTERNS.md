# Phase 176: Pattern Map

**Mapped:** 2026-05-20
**Sources:** 176-CONTEXT.md + 176-RESEARCH.md + Phase 175 plans 175-04 (CheckboxGroup) and 175-05 (Switch)

---

## Plan Shape (recap from RESEARCH §"Plan Shape Recommendation")

- **Plan 176-01** — F7 (`Card.badge`) + F8 (`Card.subtitle`) combined. Wave 1, no deps. Touches `component.rs` + `containers.rs` + `catalog.rs` + `docs/src/json-ui/components.md`.
- **Plan 176-02** — F9 (`Grid.visible`) reproduction + audit + docs. Wave 1, no deps. Touches `containers.rs` (tests-only by default) + `docs/src/json-ui/components.md`.

Both plans Wave 1, no cross-plan coupling — `containers.rs` is touched in non-overlapping locations (`render_card` lines 31-108 + Card tests 1059+ for 176-01; `render_grid` tests 854+ for 176-02). Each plan ends with `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green.

---

## File-to-Analog Map

### Plan 176-01 — Card.badge (F7) + Card.subtitle (F8)

#### `ferro-json-ui/src/component.rs` (modify) — CardProps struct + serde tests

**Role:** Add `badge: Option<String>` and `subtitle: Option<String>` fields to `CardProps`; add round-trip + omit-empty serde tests mirroring the `footer` analog.

**Closest analog (in-struct):** `description: Option<String>` and `max_width: Option<FormMaxWidth>` (already exist in `CardProps` with `#[serde(default, skip_serializing_if = "Option::is_none")]`).

**Existing pattern (component.rs:166-179, VERBATIM):**

```rust
/// Props for Card component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
    /// IDs of footer elements (resolved against `Spec.elements`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<String>,
    #[serde(default)]
    pub variant: CardVariant,
}
```

**Change required:** Insert `badge` and `subtitle` after `description` (before `max_width`). Final shape (per RESEARCH §"Required changes for F7" + §"Required changes for F8"):

```rust
/// Props for Card component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional muted secondary line rendered immediately below the title and
    /// above the description. Pattern: name → role, customer → staff,
    /// title → category. Visually `text-sm text-text-muted`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Optional small badge text rendered alongside the title. Visually a
    /// Badge-styled pill inside the Card chrome — for status indicators,
    /// counters, countdown labels, etc. Independent of the title hierarchy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
    /// IDs of footer elements (resolved against `Spec.elements`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<String>,
    #[serde(default)]
    pub variant: CardVariant,
}
```

**Closest analog (round-trip serde test pattern):** `card_props_round_trips_footer` at component.rs:1331-1343 and `card_props_omits_empty_footer_in_json` at component.rs:1357-1371.

**Existing test pattern (component.rs:1331-1343, VERBATIM):**

```rust
#[test]
fn card_props_round_trips_footer() {
    let original = CardProps {
        title: "Hero".to_string(),
        description: None,
        max_width: None,
        footer: vec!["btn1".to_string(), "btn2".to_string()],
        variant: CardVariant::Bordered,
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: CardProps = serde_json::from_str(&json).unwrap();
    assert_eq!(original.footer, parsed.footer);
}
```

**Existing test pattern (component.rs:1357-1371, VERBATIM):**

```rust
#[test]
fn card_props_omits_empty_footer_in_json() {
    let card = CardProps {
        title: "Card".to_string(),
        description: None,
        max_width: None,
        footer: Vec::new(),
        variant: CardVariant::Bordered,
    };
    let json = serde_json::to_string(&card).unwrap();
    assert!(
        !json.contains("\"footer\""),
        "empty footer must be skipped, got: {json}"
    );
}
```

> Note: every existing `card_props_*` test constructs the full struct positionally. Adding `subtitle` and `badge` fields breaks these existing tests — they must be augmented with `subtitle: None, badge: None,` to compile. This is a mechanical fixup the planner must call out.

**New tests to add (per RESEARCH §"Test surface per finding"):**

```rust
#[test]
fn card_props_round_trips_badge() {
    let original = CardProps {
        title: "Hero".to_string(),
        description: None,
        subtitle: None,
        badge: Some("Scade tra 9m".to_string()),
        max_width: None,
        footer: Vec::new(),
        variant: CardVariant::Bordered,
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: CardProps = serde_json::from_str(&json).unwrap();
    assert_eq!(original.badge, parsed.badge);
}

#[test]
fn card_props_omits_empty_badge_in_json() {
    let card = CardProps {
        title: "Card".to_string(),
        description: None,
        subtitle: None,
        badge: None,
        max_width: None,
        footer: Vec::new(),
        variant: CardVariant::Bordered,
    };
    let json = serde_json::to_string(&card).unwrap();
    assert!(
        !json.contains("\"badge\""),
        "empty badge must be skipped, got: {json}"
    );
}

#[test]
fn card_props_round_trips_subtitle() {
    let original = CardProps {
        title: "Hero".to_string(),
        description: None,
        subtitle: Some("Marco Rossi".to_string()),
        badge: None,
        max_width: None,
        footer: Vec::new(),
        variant: CardVariant::Bordered,
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: CardProps = serde_json::from_str(&json).unwrap();
    assert_eq!(original.subtitle, parsed.subtitle);
}

#[test]
fn card_props_omits_empty_subtitle_in_json() {
    let card = CardProps {
        title: "Card".to_string(),
        description: None,
        subtitle: None,
        badge: None,
        max_width: None,
        footer: Vec::new(),
        variant: CardVariant::Bordered,
    };
    let json = serde_json::to_string(&card).unwrap();
    assert!(
        !json.contains("\"subtitle\""),
        "empty subtitle must be skipped, got: {json}"
    );
}
```

**Schemars regression:** existing `schema_for_card_props_generates` test (component.rs:1110-1113) is the canary — passes automatically once `CardProps` recompiles with the new fields.

**Existing schemars canary (component.rs:1110-1113, VERBATIM):**

```rust
#[test]
fn schema_for_card_props_generates() {
    assert_schema_nonempty_object::<CardProps>("CardProps");
}
```

---

#### `ferro-json-ui/src/render/containers.rs` (modify) — render_card + Card tests

**Role:** Add two new optional slots to the `render_card` HTML emission — `subtitle` between the title and the description, `badge` right-aligned with the title via a flex wrapper. Add render-side unit tests in the existing `tests` module.

**Closest analog (slot emission):** the existing `if let Some(ref desc) = props.description {` block at containers.rs:70-75 (optional muted-text slot beneath title).

**Visual chrome analog (Badge styling):** `render_badge` in atoms.rs:249-271 — `inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium` base + `bg-secondary/10 text-secondary-foreground` for `BadgeVariant::Secondary`.

**Existing render_card pattern (containers.rs:31-108, VERBATIM — full function for context):**

```rust
pub(crate) fn render_card(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: CardProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Card props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    // Body: rendered from `Element.children`.
    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    // Footer: rendered from `CardProps.footer`. Missing IDs surface via the
    // walker's missing-id diagnostic comment.
    let footer: String = props
        .footer
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    let (outer_class, inner_pad) = match props.variant {
        CardVariant::Bordered => (
            "rounded-lg border border-border bg-card shadow-sm overflow-visible",
            "p-4",
        ),
        CardVariant::Elevated => ("rounded-lg bg-card shadow-md overflow-visible", "p-8"),
    };

    let mut html = format!("<div class=\"{outer_class}\"><div class=\"{inner_pad}\">");
    html.push_str(&format!(
        "<h3 class=\"text-base font-semibold leading-snug text-text\">{}</h3>",
        html_escape(&props.title)
    ));
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"mt-1 text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }
    // ... body wrapper + footer wrapper ...
}
```

**Visual chrome analog (atoms.rs:249-271, VERBATIM — `render_badge`):**

```rust
pub(crate) fn render_badge(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: BadgeProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Badge", e),
    };
    let base = "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium";
    let variant_classes = match props.variant {
        BadgeVariant::Default => "bg-primary/10 text-primary",
        BadgeVariant::Secondary => "bg-secondary/10 text-secondary-foreground",
        BadgeVariant::Destructive => "bg-destructive/10 text-destructive",
        BadgeVariant::Outline => "border border-border text-text",
    };
    format!(
        "<span class=\"{} {}\" style=\"justify-self: start;\">{}</span>",
        base,
        variant_classes,
        html_escape(&props.label)
    )
}
```

**Change required (F7 — badge):** Replace the bare `<h3>` emission at containers.rs:66-69 with a conditional title-row wrapper. When `props.badge` is `Some`, wrap title + inline Badge span in `flex items-start justify-between gap-2`; when `None`, emit the title alone (preserving the existing `render_card_bordered_default` test invariant). Per RESEARCH §"Required changes for F7":

```rust
if let Some(ref badge) = props.badge {
    html.push_str("<div class=\"flex items-start justify-between gap-2\">");
    html.push_str(&format!(
        "<h3 class=\"text-base font-semibold leading-snug text-text\">{}</h3>",
        html_escape(&props.title)
    ));
    html.push_str(&format!(
        "<span class=\"inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium bg-secondary/10 text-secondary-foreground shrink-0\">{}</span>",
        html_escape(badge)
    ));
    html.push_str("</div>");
} else {
    html.push_str(&format!(
        "<h3 class=\"text-base font-semibold leading-snug text-text\">{}</h3>",
        html_escape(&props.title)
    ));
}
```

**Change required (F8 — subtitle):** Insert AFTER the title-row block (or after the standalone `<h3>` in the else branch) and BEFORE the `description` block. Per RESEARCH §"Required changes for F8":

```rust
if let Some(ref subtitle) = props.subtitle {
    html.push_str(&format!(
        "<p class=\"mt-0.5 text-sm text-text-muted\">{}</p>",
        html_escape(subtitle)
    ));
}
```

Spacing decision: RESEARCH suggests `mt-0.5` (4px) so the subtitle pairs visually with the title; the existing description block at line 71-74 uses `mt-1` (8px). Planner may unify to `mt-1` for simplicity if visual review prefers.

**Closest analog (render test pattern):** `card_emits_wrapper_and_title_escaped` (containers.rs:970-990) and `render_card_bordered_default` (containers.rs:1059-1073). Both use the local `build_spec(...)` helper (containers.rs:846-852) and `Element::new("Card").prop(...)` builder.

**Existing test pattern (containers.rs:846-852, VERBATIM — `build_spec` helper):**

```rust
fn build_spec(elements: Vec<(&str, ElementBuilder)>) -> Spec {
    let mut b = Spec::builder();
    for (id, el) in elements {
        b = b.element(id, el);
    }
    b.build().expect("ok")
}
```

**Existing test pattern (containers.rs:970-990, VERBATIM — `card_emits_wrapper_and_title_escaped`):**

```rust
#[test]
fn card_emits_wrapper_and_title_escaped() {
    let spec = build_spec(vec![(
        "root",
        Element::new("Card").prop("title", "<b>T</b>"),
    )]);
    let el = spec.elements.get("root").unwrap();
    let html = render_card(el, &spec, &json!({}), 1);
    assert!(
        html.contains("rounded-lg border border-border bg-card"),
        "got: {html}"
    );
    assert!(
        html.contains("&lt;b&gt;T&lt;/b&gt;"),
        "title must be escaped; got: {html}"
    );
    assert!(
        !html.contains("<b>T</b>"),
        "raw HTML must not appear; got: {html}"
    );
}
```

**Existing test pattern (containers.rs:1059-1073, VERBATIM — `render_card_bordered_default`):**

```rust
#[test]
fn render_card_bordered_default() {
    let spec = build_spec(vec![("root", Element::new("Card").prop("title", "X"))]);
    let el = spec.elements.get("root").unwrap();
    let html = render_card(el, &spec, &json!({}), 0);
    assert!(
        html.contains("border border-border"),
        "expected Bordered class, got: {html}"
    );
    assert!(
        html.contains("shadow-sm"),
        "expected shadow-sm, got: {html}"
    );
    assert!(html.contains("p-4"), "expected p-4, got: {html}");
}
```

**New tests to add (per RESEARCH §"Test surface per finding"):**

```rust
#[test]
fn render_card_emits_badge_when_present() {
    let spec = build_spec(vec![(
        "root",
        Element::new("Card")
            .prop("title", "Booking")
            .prop("badge", "Scade tra 9m"),
    )]);
    let el = spec.elements.get("root").unwrap();
    let html = render_card(el, &spec, &json!({}), 0);
    assert!(
        html.contains("Scade tra 9m"),
        "badge label must appear in DOM; got: {html}"
    );
    assert!(
        html.contains("bg-secondary/10"),
        "badge must use Secondary chrome; got: {html}"
    );
    assert!(
        html.contains("flex items-start justify-between"),
        "title-row wrapper must be emitted when badge present; got: {html}"
    );
}

#[test]
fn render_card_omits_badge_when_absent() {
    let spec = build_spec(vec![("root", Element::new("Card").prop("title", "X"))]);
    let el = spec.elements.get("root").unwrap();
    let html = render_card(el, &spec, &json!({}), 0);
    assert!(
        !html.contains("bg-secondary/10"),
        "no badge chrome when badge absent; got: {html}"
    );
}

#[test]
fn render_card_emits_subtitle_when_present() {
    let spec = build_spec(vec![(
        "root",
        Element::new("Card")
            .prop("title", "Booking")
            .prop("subtitle", "Marco Rossi"),
    )]);
    let el = spec.elements.get("root").unwrap();
    let html = render_card(el, &spec, &json!({}), 0);
    assert!(
        html.contains("Marco Rossi"),
        "subtitle text must appear in DOM; got: {html}"
    );
    assert!(
        html.contains("text-sm text-text-muted"),
        "subtitle must use muted styling; got: {html}"
    );
}

#[test]
fn render_card_omits_subtitle_when_absent() {
    let spec = build_spec(vec![("root", Element::new("Card").prop("title", "X"))]);
    let el = spec.elements.get("root").unwrap();
    let html = render_card(el, &spec, &json!({}), 0);
    // The description block also uses text-sm text-text-muted, so we assert by
    // counting muted paragraphs. With no subtitle and no description, none emit.
    assert!(
        !html.contains("text-sm text-text-muted"),
        "no muted text when both subtitle and description absent; got: {html}"
    );
}

#[test]
fn render_card_emits_title_subtitle_description_badge_together() {
    let spec = build_spec(vec![(
        "root",
        Element::new("Card")
            .prop("title", "Booking #1")
            .prop("subtitle", "Marco Rossi")
            .prop("description", "Customer detail")
            .prop("badge", "Scade tra 9m"),
    )]);
    let el = spec.elements.get("root").unwrap();
    let html = render_card(el, &spec, &json!({}), 0);
    assert!(html.contains("Booking #1"), "title; got: {html}");
    assert!(html.contains("Marco Rossi"), "subtitle; got: {html}");
    assert!(html.contains("Customer detail"), "description; got: {html}");
    assert!(html.contains("Scade tra 9m"), "badge; got: {html}");
}
```

---

#### `ferro-json-ui/src/catalog.rs` (modify) — Card catalog description string

**Role:** Update the Card catalog entry's description string. The JSON schema regenerates automatically from `schema_for!(CardProps)` once the struct gains new fields — no manual schema edit.

**Closest analog:** the entry IS the analog — only the description string changes.

**Existing pattern (catalog.rs:269-275, VERBATIM):**

```rust
    // === Containers (containers.rs) ===
    (
        "Card",
        "Content container with title, description, body children, and optional footer slot.",
        || to_value(schema_for!(CardProps)).unwrap(),
        &["footer"],
    ),
```

**Change required:** Update the description string only (per RESEARCH §"Spec / Catalog / Docs"):

```rust
    (
        "Card",
        "Content container with title, description, optional badge and subtitle, body children, and optional footer slot.",
        || to_value(schema_for!(CardProps)).unwrap(),
        &["footer"],
    ),
```

No new imports. No schema literal — schemars derive picks up new fields automatically. The `&["footer"]` slot-id field is unchanged because `badge` and `subtitle` are string slots, not element-id slots.

---

#### `docs/src/json-ui/components.md` (modify) — Card section: prop table + example

**Role:** Document the two new optional Card props in the existing Card section's prop table and add a short worked example.

**Closest analog:** existing Card section prop table at docs/src/json-ui/components.md:80-83.

**Existing pattern (components.md:76-96, VERBATIM):**

```markdown
### Card

Container with title, optional description, nested children, and footer.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Card heading |
| `description` | `string \| null` | Secondary text below the title |

Children are element IDs listed in the `"children"` array on the element, not in props.

```json
"user_card": {
  "type": "Card",
  "props": {
    "title": "User Details",
    "description": "Account information"
  },
  "children": ["name_text", "email_text"]
}
```
```

**Change required:** Add `subtitle` and `badge` rows to the prop table immediately after `description`. Add a follow-up worked example showing both new slots in use (analogous to the existing "Variant" subsection style at components.md:134-156). Keep tone neutral / scientific per CLAUDE.md.

Suggested addition (planner's discretion on exact wording):

```markdown
| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Card heading |
| `description` | `string \| null` | Secondary text below the title |
| `subtitle` | `string \| null` | Muted secondary identifier rendered between title and description |
| `badge` | `string \| null` | Small Badge-styled pill rendered right of the title (Secondary variant chrome) |
```

Plus a worked example showing all four text slots populated (e.g. countdown-badge kanban card from the consumer field test).

---

### Plan 176-02 — Grid.visible (F9) reproduction + audit

> Per RESEARCH §"Required changes (contingent on plan-time reproduction)": the load-bearing first task is **reproducing the consumer's failure against current ferro master**. The plan-time investigation decides whether the plan ships as (a) regression-test + docs only (no production code change — predicted outcome per RESEARCH §"Critical Pre-Planning Finding") or (b) test + actual code fix if a real root cause surfaces. Either way, the test is the load-bearing artifact.

#### `ferro-json-ui/src/render/containers.rs` (modify) — Grid tests; possibly NO production code

**Role:** Add visibility regression tests for `Grid` in the existing `tests` module. Confirms that `Grid` honors `visible` consistently with every other element type (via `render_element`'s shared visibility check at render/mod.rs:155-160 — see RESEARCH §"Visibility evaluator — verified architecture").

**Closest analog (overall plan shape):** Phase 175 plan 175-05 — F4 Switch regression test landed as docs + a single regression test, no production code change once F1 unblocked the consumer's path. Same shape: pin the closure with a public-API test.

**Closest analog (render test pattern):** `grid_recurses_children` at containers.rs:854-871 and `grid_scrollable_emits_flow_col` at containers.rs:873-883.

**Existing test pattern (containers.rs:854-871, VERBATIM — `grid_recurses_children`):**

```rust
#[test]
fn grid_recurses_children() {
    let spec = build_spec(vec![
        (
            "root",
            Element::new("Grid")
                .prop("columns", 2)
                .child("a")
                .child("b"),
        ),
        ("a", Element::new("Text").prop("content", "AAA")),
        ("b", Element::new("Text").prop("content", "BBB")),
    ]);
    let el = spec.elements.get("root").unwrap();
    let html = render_grid(el, &spec, &json!({}), 1);
    assert!(html.contains("grid-cols-2"), "got: {html}");
    assert!(html.starts_with("<div class=\"grid"), "got: {html}");
}
```

**Closest analog (visibility test pattern at the walker level):** `walker_root_hidden_emits_root_hidden_comment` at render/mod.rs:387-404 — uses `Visibility::Condition(VisibilityCondition { ... })` directly with `Eq` operator against a `data: {"show": false}` payload.

**Existing pattern (render/mod.rs:387-404, VERBATIM — walker visibility test):**

```rust
#[test]
fn walker_root_hidden_emits_root_hidden_comment() {
    let mut spec = Spec::builder()
        .element("root", Element::new("Text"))
        .build()
        .expect("ok");
    let el = spec.elements.get_mut("root").unwrap();
    el.visible = Some(Visibility::Condition(VisibilityCondition {
        path: "/show".into(),
        operator: VisibilityOperator::Eq,
        value: Some(json!(true)),
    }));
    let html = render_spec_to_html(&spec, &json!({"show": false}));
    assert!(
        html.contains("<!-- ferro-json-ui: root hidden -->"),
        "got: {html}"
    );
}
```

> Note: `ElementBuilder` exposes `.visible(Visibility)` (spec.rs:470-473), so tests can attach visibility via the builder without mutating `Element` post-build. Picking one approach consistently keeps each test in the existing module style. The `walker_root_hidden_*` test mutates post-build because the test target is the walker; for Grid tests, prefer `Element::new("Grid").visible(...)` if the planner wants the cleaner builder path.

> **Important:** Visibility-true at the root yields `<!-- ferro-json-ui: root hidden -->` ONLY when the visibility check trips on the root element. For Grid as a non-root element wrapped in a parent (typical real-world use), the hidden case yields an empty string in the parent's body — assert by ABSENCE (no `grid-cols-`, no `<div class="grid`) rather than by a diagnostic comment.

**New tests to add (per RESEARCH §"Test surface per finding"):**

```rust
#[test]
fn grid_renders_when_visible_true() {
    use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};
    let spec = build_spec(vec![(
        "root",
        Element::new("Grid")
            .prop("columns", 1)
            .visible(Visibility::Condition(VisibilityCondition {
                path: "/flag".into(),
                operator: VisibilityOperator::Eq,
                value: Some(json!(true)),
            })),
    )]);
    let html = crate::render::render_spec_to_html(&spec, &json!({"flag": true}));
    assert!(
        html.contains("<div class=\"grid"),
        "Grid must render when visible-true; got: {html}"
    );
}

#[test]
fn grid_hidden_when_visible_false() {
    use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};
    let spec = build_spec(vec![(
        "root",
        Element::new("Grid")
            .prop("columns", 1)
            .visible(Visibility::Condition(VisibilityCondition {
                path: "/flag".into(),
                operator: VisibilityOperator::Eq,
                value: Some(json!(true)),
            })),
    )]);
    let html = crate::render::render_spec_to_html(&spec, &json!({"flag": false}));
    assert!(
        !html.contains("<div class=\"grid"),
        "Grid must be absent when visible-false; got: {html}"
    );
}

#[test]
fn grid_visible_consumer_reproduction() {
    // Mirrors the consumer's chip-strip spec from
    // gestiscilo-it/app/src/views/calendario/calendar_day.json:73-85.
    // Grid is non-root here so the absent case yields no diagnostic — assert
    // by presence/absence of the grid wrapper substring.
    use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};
    let chip_strip_visible = Visibility::Condition(VisibilityCondition {
        path: "/has_staff".into(),
        operator: VisibilityOperator::Eq,
        value: Some(json!(true)),
    });
    let spec = build_spec(vec![
        (
            "root",
            Element::new("Grid").prop("columns", 1).child("staff_chips_row"),
        ),
        (
            "staff_chips_row",
            Element::new("Grid")
                .prop("columns", 1)
                .prop("gap", "sm")
                .visible(chip_strip_visible.clone()),
        ),
    ]);

    let html_visible =
        crate::render::render_spec_to_html(&spec, &json!({"has_staff": true}));
    assert!(
        html_visible.matches("<div class=\"grid").count() >= 2,
        "inner Grid must render when has_staff=true; got: {html_visible}"
    );

    let html_hidden =
        crate::render::render_spec_to_html(&spec, &json!({"has_staff": false}));
    assert_eq!(
        html_hidden.matches("<div class=\"grid").count(),
        1,
        "only outer Grid renders when has_staff=false; got: {html_hidden}"
    );
}
```

> The third test is the load-bearing one for F9 — it replicates the failing UAT spec shape so any future regression that breaks Grid visibility re-fires. Per RESEARCH §"Possible real causes": if these tests pass green against current master, F9 closes as "could not reproduce; consumer to re-test with patched runtime" (no production code change). If they fail, the planner has the reproduction in hand and can investigate the actual cause.

---

#### `docs/src/json-ui/components.md` (modify) — Grid section: visibility clarification

**Role:** Add a short paragraph to the existing Grid section clarifying that `visible` works the same on Grid as on every other v2 element (element-level, evaluated by the walker before component dispatch). This is the "audit which v2 components currently support `visible`" success criterion from CONTEXT §F9.

**Closest analog (overall doc shape):** Phase 175 plan 175-05 Task 2 — added a Switch section + a "Substitution: Checkbox styled as switch" subsection. Same shape: a clarifying paragraph + cross-reference, no new prop documentation (because `visible` is element-level, not a `GridProps` field).

**Existing pattern (components.md:158-176, VERBATIM — current Grid section):**

```markdown
### Grid

Responsive grid layout for arranging child elements in columns.

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `number \| null` | Number of columns (default: 2) |
| `gap` | `gap_size \| null` | Gap between items: `"none"`, `"xs"`, `"sm"`, `"md"`, `"lg"`, `"xl"` |

```json
"stats_grid": {
  "type": "Grid",
  "props": {
    "columns": 3,
    "gap": "md"
  },
  "children": ["revenue_stat", "orders_stat", "users_stat"]
}
```
```

**Change required:** Append a clarifying subsection (planner picks tone — keep neutral / scientific). Suggested addition:

```markdown
#### Visibility

`visible` is an element-level field that lives on every JSON-UI element, including
Grid. It is not a `GridProps` prop. When the visibility condition evaluates `false`
against the spec's `data` payload, the Grid and all of its children are absent from
the rendered DOM (no `hidden` attribute, no empty wrapper — the subtree is omitted).

```json
"staff_chips_row": {
  "type": "Grid",
  "props": { "columns": 1, "gap": "sm" },
  "children": ["staff_chip"],
  "visible": { "path": "/has_staff", "operator": "eq", "value": true }
}
```

Identical semantics apply to every other container and atom — Card, Form, Button,
Badge, and all plugin components. See `visibility.md` for the full operator set.
```

> If `docs/src/json-ui/visibility.md` does not exist, drop the cross-reference and let the Grid clarification stand alone.

---

## Test Naming Conventions (extracted)

From the existing test modules:

- **Card render tests** (containers.rs:1059+): `render_card_<variant>` (e.g. `render_card_bordered_default`, `render_card_elevated_no_border`, `render_card_omitted_variant_is_bordered`, `card_max_width_narrow_wraps_in_mx_auto`). The `render_card_*` prefix is used when the test exercises render output specifically; the `card_*` prefix is used when the test exercises a Card behavior more broadly (e.g. parse-time rejection).
- **CardProps serde tests** (component.rs:1331+): `card_props_round_trips_<field>` and `card_props_omits_empty_<field>_in_json` — mirror this pair for every new optional field.
- **Schemars canaries** (component.rs:1110+): `schema_for_<type>_props_generates` — already in place for CardProps; passes automatically.
- **Grid render tests** (containers.rs:854+): `grid_<behavior>` (e.g. `grid_recurses_children`, `grid_scrollable_emits_flow_col`).
- **Walker / visibility tests** (render/mod.rs:370+): `walker_<state>_<observable>` (e.g. `walker_missing_child_emits_diagnostic`, `walker_root_hidden_emits_root_hidden_comment`).

For Phase 176, the proposed new test names follow these conventions:

| Plan | Test name | Module |
|------|-----------|--------|
| 176-01 | `render_card_emits_badge_when_present` | containers.rs |
| 176-01 | `render_card_omits_badge_when_absent` | containers.rs |
| 176-01 | `render_card_emits_subtitle_when_present` | containers.rs |
| 176-01 | `render_card_omits_subtitle_when_absent` | containers.rs |
| 176-01 | `render_card_emits_title_subtitle_description_badge_together` | containers.rs |
| 176-01 | `card_props_round_trips_badge` | component.rs |
| 176-01 | `card_props_round_trips_subtitle` | component.rs |
| 176-01 | `card_props_omits_empty_badge_in_json` | component.rs |
| 176-01 | `card_props_omits_empty_subtitle_in_json` | component.rs |
| 176-02 | `grid_renders_when_visible_true` | containers.rs |
| 176-02 | `grid_hidden_when_visible_false` | containers.rs |
| 176-02 | `grid_visible_consumer_reproduction` | containers.rs |

---

## Wave / Dependency Notes

- **Both plans Wave 1, no `depends_on`.** Per RESEARCH §"Plan Shape Recommendation" — they touch disjoint sections of `containers.rs` (176-01 at `render_card` lines 31-108 + Card tests 1059+; 176-02 at `render_grid` tests 854+).
- **Plan 176-01 internal wave:** mirror Phase 175 plan 175-04's three-task shape — Wave 0 add failing tests (red), Wave 1 land the implementation (green), Wave 2 docs. The Phase 175 F2 plan is the canonical shape for component-extension work.
- **Plan 176-02 internal shape:** mirror Phase 175 plan 175-05's two-task shape — Task 1 regression test (green-on-first-run if F9 doesn't reproduce, red-then-investigate if it does), Task 2 docs clarification.
- **Pre-existing Card serde tests need fixup.** Adding `subtitle` + `badge` to `CardProps` breaks every existing `card_props_*` test that positionally constructs the struct. The planner MUST list this as a Wave 0 mechanical fixup task in 176-01, before introducing the new render slots, so the test suite stays green between waves.
- **CardProps field-order convention:** existing fields follow `title → description → max_width → footer → variant`. RESEARCH suggests inserting `subtitle` + `badge` between `description` and `max_width`. Either order (`description → subtitle → badge` or `description → badge → subtitle`) is acceptable. The chosen order must be reflected consistently across the struct, all tests, and the docs prop table.
- **No new `BUILTIN_TYPES` count change.** Per RESEARCH Assumption A6: Card and Grid are already registered. No `builtin_types_count_matches_dispatch` increment is needed (unlike Phase 175 plan 175-04 which bumped 42→43 for CheckboxGroup).
- **Final gate (per plan):** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green.

---

## PATTERN MAPPING COMPLETE
