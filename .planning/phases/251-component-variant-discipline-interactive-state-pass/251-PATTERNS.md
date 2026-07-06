# Phase 251: Component variant discipline + interactive-state pass - Pattern Map

**Mapped:** 2026-07-03
**Files analyzed:** 24 (2 new artifacts, 20 modified files, 2 generated/regenerated)
**Analogs found:** 23 / 24 (1 partial — schema-walking `$ref` resolver has no exact in-tree precedent)

This is a refactor phase: most files are modified in place, so their "analog" is the
existing convention inside the same file or a sibling in the same crate. The three
genuinely new artifacts (shared canonical enums, shared interactive-base constants,
D-19 schema-walking drift guard) each have a strong in-tree model, excerpted below.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/component.rs` (new `Variant`/`Tone`/`Size`, `CardAppearance`) | model (wire contract) | transform (JSON↔Rust) | `ButtonVariant` at component.rs:50-63 (same file) | exact |
| `ferro-json-ui/src/render/classes.rs` (NEW — or consts in `render/mod.rs`, planner's call) | utility (shared class constants) | transform | atoms.rs:137 base string + toasts.rs:4-9 `VARIANT_CLASSES` const style + `badge_inline_html` lockstep-doc pattern (atoms.rs:258-261) | role-match |
| `ferro-json-ui/src/catalog.rs` (D-19 guard, prose, optional `$ref` resolution) | test + config | batch (schema walk) | `builtin_types_count_drift_guard` catalog.rs:1092-1102 + `render_field_type` catalog.rs:922-963 | exact (guard) / partial ($ref walk) |
| `ferro-json-ui/src/render/atoms.rs` | renderer | transform (spec→HTML) | itself — `badge_inline_html` :261-280, `render_alert` :289-300 | exact |
| `ferro-json-ui/src/render/containers.rs` | renderer | transform | itself — `button_variant_classes` :969-1002 (deduped into shared constants) | exact |
| `ferro-json-ui/src/render/data.rs` | renderer | transform | itself — `BadgeCell` :358-373, MCG badge plumbing :683-688, :746-748 | exact |
| `ferro-json-ui/src/render/form.rs` | renderer | transform | itself — dual-ring pattern :175-184 | exact |
| `ferro-json-ui/src/layout.rs` | renderer | transform | itself — `layout_sidebar_nav_item` :144-161 + INT-07 test :1280-1297 | exact |
| `ferro-json-ui/src/runtime/toasts.rs` | JS runtime constant | event-driven (DOM) | itself — `VARIANT_CLASSES` :4-9, `data-toast-variant` selector :84 | exact |
| `ferro-json-ui/src/runtime/tabs.rs` | JS runtime constant | event-driven (DOM) | itself — classList literals :62-73 (lockstep with containers.rs tab classes) | exact |
| `ferro-json-ui/src/runtime/mod.rs` | test | — | `variant_classes_use_semantic_tokens` :70-79 | exact |
| `ferro-json-ui/src/action.rs` (OQ-1: `DialogVariant`/`NotifyVariant` → `Tone`) | model | transform | `DialogVariant`/`NotifyVariant` :22-67 (same derive stack as component enums) | exact |
| `ferro-json-ui/src/projection/component_map.rs` | service (props emitter) | transform | itself — `badge_variant_for` :164-171, `build_relationship_button_props` :342-354 | exact |
| `ferro-json-ui/src/projection/builder.rs` | service (spec emitter) | transform | itself — :370 `CardVariant::Bordered`, :688 `ActionItem { variant: None }` | exact |
| `ferro-json-ui/src/loader.rs` | test fixture | file-I/O | itself — gated-Alert spec :314-324 | exact |
| `ferro-json-ui/src/lib.rs` | re-export surface | — | itself — export lists :47-63 | exact |
| `framework/src/lib.rs` | re-export facade | — | itself — `#[cfg(feature = "json-ui")]` block :84-99 | exact |
| `ferro-mcp/src/tools/code_templates.rs` | agent-facing template text | — | itself — :1093-1096 (`"variant": "default"`) | exact |
| `ferro-mcp/src/tools/json_ui_validate_spec.rs` | test | — | itself — :107-144 | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | agent-facing prose | — | itself — ACTION_API prose :277-279 (OQ-1-dependent) | exact |
| `app/src/views/login.json`, `login_confirm.json` | sample spec (fixture) | config | themselves — login.json :12, :41 | exact |
| `docs/src/json-ui/components.md` (+ migration table, D-17) | docs | — | "Shared Enum Values" section components.md:40-58 | exact |
| `ferro-json-ui/assets/input.css` (verify safelist; likely no edit) | config (CSS) | — | itself — `@source inline(...)` :66-73, `@utility duration-*` :80-88 | exact |
| `ferro-json-ui/assets/ferro-base.css` (regenerated) | generated asset | batch | `scripts/gen-ferro-base-css.sh` (Phase 250 workflow) | exact |

## Pattern Assignments

### 1. `ferro-json-ui/src/component.rs` — new canonical `Variant` / `Tone` / `Size` enums (model)

**Analog:** `ButtonVariant`, same file, lines 49-63 — the exact derive stack + serde/strum convention for the three new enums:

```rust
// ferro-json-ui/src/component.rs:49-63
/// Button visual variants (aligned to shadcn/ui).
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, strum::AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ButtonVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}
```

Copy this shape verbatim for `Variant { Primary(#[default]), Secondary, Outline, Ghost, Destructive }`,
`Tone { Neutral(#[default]?…), Success, Warning, Destructive }`, `Size { Sm, Md(#[default]), Lg }`.
Also add `Copy` (precedent: `CardVariant` at :186 and `ColumnAlign` at :151 derive `Copy` — small unit enums should).

**Doc-comment discipline (schema-shape critical, RESEARCH Pitfall 4):** container-level doc
comment ONLY — model on `CardVariant` :180-192, NOT on `BadgeVariant`, whose per-variant doc
on `Warning` (:109-111) degrades the schemars output to `anyOf`-of-`const`:

```rust
// ferro-json-ui/src/component.rs:180-192 — container-level docs only (GOOD shape)
/// Visual variant for Card chrome.
///
/// - `Bordered` (default): `border + bg-card + shadow-sm` with `p-4`. ...
/// - `Elevated`: `bg-card + shadow-md` (no border) with `p-8`. ...
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CardVariant {
    #[default]
    Bordered,
    Elevated,
}
```

**Props-field pattern** (for `tone` adoption and `CardProps.appearance`) — `#[serde(default)]`
non-optional enum field, model on `CardProps.variant`:

```rust
// ferro-json-ui/src/component.rs:215-216
    #[serde(default)]
    pub variant: CardVariant,     // becomes: pub appearance: CardAppearance,
```

**Enum unit-test pattern** — copy the `card_variant_tests` module shape (component.rs:1906-1968):
default assertion, snake_case serialize/deserialize, props-default, props-with-value, roundtrip:

```rust
// ferro-json-ui/src/component.rs:1910-1951 (excerpt)
#[test]
fn card_variant_default_is_bordered() {
    assert_eq!(CardVariant::default(), CardVariant::Bordered);
}
#[test]
fn card_props_without_variant_defaults_to_bordered() {
    let v = serde_json::json!({"title": "x"});
    let p: CardProps = serde_json::from_value(v).unwrap();
    assert_eq!(p.variant, CardVariant::Bordered);
}
```

**strum↔serde agreement test** — extend `variant_enums_strum_matches_serde_wire_format`
(component.rs:1845-1895) to the new enums; the generic checker is reusable as-is:

```rust
// ferro-json-ui/src/component.rs:1845-1856
fn variant_enums_strum_matches_serde_wire_format() {
    fn check<T: AsRef<str> + serde::Serialize>(variants: &[T], label: &str) {
        for v in variants {
            let json = serde_json::to_string(v).expect("serialize");
            let json_stripped = json.trim_matches('"');
            assert_eq!(v.as_ref(), json_stripped,
                "strum AsRefStr drifted from serde for {label} variant");
        }
    }
    check(&[AlertVariant::Info, /* ... */], "AlertVariant");
    // ...
}
```

**Pre-existing gap found during mapping:** the existing test's `BadgeVariant` list (:1866-1874)
omits `BadgeVariant::Warning` — enumerate ALL variants of the new enums when rewriting (or
derive the list via `strum::VariantArray` to make omission impossible).

---

### 2. `ferro-json-ui/src/render/classes.rs` (NEW) — shared interactive-base constants (utility)

**Consolidation seeds (the three duplicates to collapse):**

```rust
// ferro-json-ui/src/render/atoms.rs:137 — seed 1 (Button)
let base = "inline-flex items-center justify-center rounded-md font-medium transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2";
```

```rust
// ferro-json-ui/src/render/containers.rs:969-975 — seed 2 (verbatim duplicate table, per-variant)
fn button_variant_classes(variant: &ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Default => {
            "inline-flex items-center justify-center rounded-md font-medium text-sm px-4 py-2 \
             transition-colors duration-150 bg-primary text-primary-foreground hover:bg-primary/90 \
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
        }
        // ... 5 more variants, all repeating the same base + ring fragment
```

```rust
// ferro-json-ui/src/layout.rs:154 — seed 3 (sidebar nav item, same fragment again)
"flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium bg-card text-primary transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2",
```

**Const-definition style to copy** — `pub(crate) const` string constants; the crate precedent
for "class table as a named constant near its consumers" is `runtime/toasts.rs:4-9`
(`VARIANT_CLASSES`). Target shape (composing via `concat!` keeps literals scannable by Tailwind):

```rust
// New module — target pattern, composed of the migrated fragments:
pub(crate) const FOCUS_RING: &str =
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2";
pub(crate) const MOTION_FAST: &str = "transition-colors duration-fast ease-base";
pub(crate) const MOTION_BASE: &str = "transition-opacity duration-base ease-base";
pub(crate) const DISABLED_BASE: &str = "disabled:opacity-50 disabled:pointer-events-none";
pub(crate) const INTERACTIVE_BASE: &str =
    concat!("transition-colors duration-fast ease-base ",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2");
```

Note: per D-15, `motion-reduce:transition-none` is dropped from every site where a token
duration takes over (the 0.01ms collapse in input.css:96-102 handles reduced motion).

**Shared-helper doc pattern** — copy the lockstep-comment style from `badge_inline_html`
(atoms.rs:258-260), which is the crate's precedent for "one string, two consumers":

```rust
// ferro-json-ui/src/render/atoms.rs:258-260
/// Render a Badge `<span>` from `(variant, label)`. Shared by `render_badge` and
/// the DataTable `ColumnFormat::Badge` cell renderer so both surfaces stay in
/// lockstep — the base+variant CSS string is the single source of truth.
```

If a new module is created, register it in `render/mod.rs` (`mod classes;` +
`pub(crate) use`) — and note the crate scanner (`@source "../../ferro-json-ui/src"`,
input.css:4) picks up literals anywhere under `src/`, so location does not affect CSS output.

---

### 3. `ferro-json-ui/src/catalog.rs` — D-19 schema-walking drift guard (test)

**Analog for the guard's placement, tone, and comment style** — `builtin_types_count_drift_guard`:

```rust
// ferro-json-ui/src/catalog.rs:1092-1102
#[test]
fn builtin_types_count_drift_guard() {
    // SINGLE source of truth for the absolute builtin-component count. When
    // BUILTIN_TYPES changes, update the number HERE and nowhere else — every
    // other test asserts its invariant relationally (against
    // BUILTIN_TYPES.len()), so a component addition breaks only this test.
    // History: 39 → 40 (CheckboxList) → 42 (DetailPage) → 43 (CheckboxGroup)
    // → 44 (MediaCardGrid) → 45 (StreamText) → 47 (SegmentedControl, SidebarLayout)
    // → 47 (DropdownMenu replaced by ActionGroup).
    assert_eq!(crate::render::BUILTIN_TYPES.len(), 47);
}
```

**Analog for building the catalog inside a test without plugin pollution** —
`build_populates_all_builtins`:

```rust
// ferro-json-ui/src/catalog.rs:1119-1123
// Use build_builtins_only() to avoid pollution from BadPlugin_117.
let cat = Catalog::build_builtins_only().expect("build succeeds");
```

**The schema structure the walker must traverse** — `assemble_full_schema` (catalog.rs:480-557).
Key facts for the walker, verified: each `$defs/Element/oneOf` entry is
`allOf: [ {properties.type.const}, {properties: {props: <schema>, children, action: $ref #/$defs/Action, visible: $ref #/$defs/Visibility}} ]`;
component-local `$defs` are HOISTED to the root (`hoist_defs`, :500-501), so all `$ref`s
resolve against the root `$defs` map:

```rust
// ferro-json-ui/src/catalog.rs:502-521 (the oneOf variant shape)
serde_json::json!({
    "allOf": [
        { "type": "object", "required": ["type"],
          "properties": { "type": { "const": name } } },
        { "type": "object",
          "properties": {
              "props": props_schema,
              "children": { "type": "array", "items": { "type": "string" } },
              "action":   { "$ref": "#/$defs/Action" },
              "visible":  { "$ref": "#/$defs/Visibility" }
          } }
    ]
})
```

**Partial analog for enum-value extraction from schema JSON** — `render_field_type`
(catalog.rs:922-963) already handles the inline-`enum` and `anyOf`-with-null shapes but does
NOT resolve `$ref` (fallback #5 → `<see schema>`). The new walker extends exactly these cases:

```rust
// ferro-json-ui/src/catalog.rs:923-928 — inline enum detection to reuse
// 1) Detect enum inline: {type: "string", enum: [...]} or {enum: [...]}
if let Some(variants) = schema.get("enum").and_then(|v| v.as_array()) {
    let names: Vec<&str> = variants.iter().filter_map(|v| v.as_str()).collect();
    // ...
}
// :929-943 — anyOf/oneOf with null → Option<T> unwrap (Avatar's Option<Size> hits this)
```

Guard requirements from RESEARCH (restated for the planner): assert the three `$defs`
directly (`$defs/Variant`, `$defs/Tone`, `$defs/Size` equal the canonical value arrays);
then walk every `oneOf` props subtree transitively, resolving `$ref` with a visited-set,
asserting any property named `variant`/`tone`/`size` carries exactly the canonical set;
handle both `enum`-array and `anyOf[].const` shapes defensively.

**Catalog prose sites to update in the same file** — BUILTIN_SPECS descriptions, e.g.:

```rust
// ferro-json-ui/src/catalog.rs:138-149
(
    "Badge",
    "Small variant-styled label.",                                 // → tone wording
    || to_value(schema_for!(BadgeProps)).unwrap(),
    &[],
),
(
    "Alert",
    "Inline notice with info / success / warning / error variants.", // → canonical tone values
    || to_value(schema_for!(AlertProps)).unwrap(),
    &[],
),
```

After prose edits, re-check `prompt_under_size_budget` (catalog.rs:~1716) still passes.

---

### 4. `ferro-json-ui/src/render/atoms.rs` — tone/variant match arms + interactive pass (renderer)

**Enum→class mapping pattern (KEEP this shape — full-literal match arms, never `format!` of
class fragments):** `badge_inline_html` is the model every tone-adopting component follows:

```rust
// ferro-json-ui/src/render/atoms.rs:261-280
pub(crate) fn badge_inline_html(variant: BadgeVariant, label: &str) -> String {
    let base = "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium";
    let variant_classes = match variant {
        BadgeVariant::Default => "bg-primary/10 text-primary",
        BadgeVariant::Secondary => "bg-secondary/10 text-secondary-foreground",
        BadgeVariant::Destructive => "bg-destructive/10 text-destructive",
        BadgeVariant::Warning => "bg-warning/10 text-warning",
        BadgeVariant::Outline => "border border-border text-text",
    };
    format!(
        "<span class=\"{} {}\" style=\"justify-self: start;\">{}</span>",
        base, variant_classes, html_escape(label)
    )
}
```

New signature: `badge_inline_html(tone: Tone, label: &str)`; arms collapse per D-09
(`Neutral` = the chosen filled-vs-outline treatment, Claude's discretion).

**Alert tone mapping + icon pairing** — `render_alert` (atoms.rs:289-300) shows the two-match
pattern (classes + icon) that renames `AlertVariant::Info→Tone::Neutral`,
`Error→Destructive`:

```rust
// ferro-json-ui/src/render/atoms.rs:289-294
let variant_classes = match props.variant {
    AlertVariant::Info => "bg-primary/10 border-primary text-primary",
    AlertVariant::Success => "bg-success/10 border-success text-success",
    AlertVariant::Warning => "bg-warning/10 border-warning text-warning",
    AlertVariant::Error => "bg-destructive/10 border-destructive text-destructive",
};
```

**Toast SSR side of the JS lockstep contract** (pairs with runtime/toasts.rs, §9):

```rust
// ferro-json-ui/src/render/atoms.rs:818-835
let variant_classes = match props.variant {
    ToastVariant::Info => "bg-primary/70 text-primary-foreground",
    // ...
};
let variant_str = match props.variant {
    ToastVariant::Info => "info",
    // ...
};
format!(
    "<div class=\"... transition-opacity duration-300 backdrop-blur-md {variant_classes}\" \
     data-toast-variant=\"{variant_str}\" data-toast-timeout=\"{timeout}\">...",
```

With `Tone: strum::AsRefStr`, `variant_str` can become `props.tone.as_ref()` (the strum test
guarantees wire agreement). `duration-300` → `duration-base` per D-15.

**Button disabled contract** (today's hole D-16/Pitfall 3 fixes) — current conditional at
atoms.rs:157-166 emits ` opacity-50 cursor-not-allowed` + `disabled` attr; the anchor-wrap
path (atoms.rs:212-240) bypasses `disabled:`. The in-tree precedent for the aria-disabled
non-native-control treatment is `layout_sidebar_nav_item`'s disabled arm:

```rust
// ferro-json-ui/src/layout.rs:146-149, 162-163 — aria-disabled precedent
let (tag, classes) = if disabled {
    ("span",
     "flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-text-muted opacity-50 cursor-not-allowed select-none")
} else ...
format!("<{tag} aria-disabled=\"true\" class=\"{classes}\">")
```

(Adapt: D-16 standardizes on `opacity-50 pointer-events-none`; skip the anchor wrap when disabled.)

---

### 5. `ferro-json-ui/src/render/containers.rs` — dedupe `button_variant_classes` (renderer)

The whole per-variant table at :969-1002 (excerpted in §2) collapses: base + ring + motion
come from the shared constants; only the per-variant color fragment stays as match arms —
matching how atoms.rs `render_button_inner` already splits `base` from `variant_classes`
(atoms.rs:137-148). The `ButtonVariant::Link` arm (:996-1000) is deleted (D-07). Doc comment
at :967-968 ("Matches the `render_button_inner` variant table in `atoms.rs`") becomes true
by construction — update it to point at the shared constants.

Tab trigger classes (:254-275) are the SSR half of the tabs.rs lockstep (§9).

---

### 6. `ferro-json-ui/src/render/data.rs` — data-driven tone plumbing (renderer)

**BadgeCell rename** — the inline-deserialization pattern stays; only the field name/type change:

```rust
// ferro-json-ui/src/render/data.rs:358-373
#[derive(serde::Deserialize)]
struct BadgeCell {
    variant: BadgeVariant,      // → tone: Tone
    label: String,
}
match serde_json::from_value::<BadgeCell>(v.clone()) {
    Ok(cell) => return badge_inline_html(cell.variant, &cell.label),
    Err(e) => {
        return format!(
            "<!-- ferro-json-ui: invalid Badge cell value: {} -->",
            html_escape(&e.to_string())
        );
    }
}
```

Also update the sibling diagnostic string at :376 (`expected object {variant, label}` →
`{tone, label}`) and the `ColumnFormat::Badge` doc comment (component.rs:130-132).

**MediaCardGrid plumbing** — string-keyed row lookup + string-match styling:

```rust
// ferro-json-ui/src/render/data.rs:683-688
let badge_variant = props
    .badge_variant_key            // → badge_tone_key
    .as_deref()
    .and_then(|k| row.get(k))
    .and_then(|v| v.as_str())
    .unwrap_or("outline");        // → "neutral"

// ferro-json-ui/src/render/data.rs:746-748
let badge_classes = match badge_variant {
    "destructive" => "bg-destructive/10 text-destructive",
    _ => "border border-border text-text",
};
```

Recommended upgrade while here: parse the string through the shared `Tone`
(`serde_json::from_value::<Tone>`) or match all four canonical values so MCG and
`badge_inline_html` cannot drift — same lockstep rationale as atoms.rs:258-260.

---

### 7. `ferro-json-ui/src/render/form.rs` — ring migration, error ring preserved (renderer)

**The dual-ring pattern — only the non-error branch flips to `ring-ring`:**

```rust
// ferro-json-ui/src/render/form.rs:180-184
let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
    // → focus-visible:ring-ring
};
```

Tests pinning this: form.rs:941-942, :1075-1082 (Switch `peer-focus:ring-destructive/30`
STAYS), :1121-1122. Input/Textarea/Select/Checkbox share `duration-150 +
motion-reduce:transition-none` and `disabled:opacity-50 disabled:cursor-not-allowed`
(:198, :264, :376, :480) — cursor → `pointer-events-none` per D-16, durations → `duration-fast`.

---

### 8. `ferro-json-ui/src/layout.rs` — third base-string copy + test flip (renderer + test)

Class sites :154, :159 (excerpted in §2) compose from the shared constants. The pinning test
flips its two expected strings:

```rust
// ferro-json-ui/src/layout.rs:1288-1296 (INT-07)
let html = layout_sidebar_nav_item(&item);
assert!(html.contains("focus-visible:ring-primary"), ...);  // → "focus-visible:ring-ring"
assert!(html.contains("duration-150"), ...);                // → "duration-fast"
```

Hover-only sites with no ring (:242, :259, :268, :304, :459, :492) gain the shared
`FOCUS_RING` constant per the DS-04 inventory.

---

### 9. `ferro-json-ui/src/runtime/toasts.rs` + `runtime/tabs.rs` — JS/SSR lockstep (runtime)

**Toasts — everything keyed by the old vocabulary changes with the SSR side (§4) in the
same task:**

```javascript
// ferro-json-ui/src/runtime/toasts.rs:4-9, :16, :18
var VARIANT_CLASSES = {                      // keys → neutral/success/warning/destructive
    info: 'bg-primary text-primary-foreground',
    success: 'bg-success text-primary-foreground',
    warning: 'bg-warning text-primary-foreground',
    error: 'bg-destructive text-primary-foreground'
};
var variant = toast.variant || 'info';       // → toast.tone || 'neutral'
var colorClass = VARIANT_CLASSES[variant] || VARIANT_CLASSES.info;
```

```javascript
// :22 duration-300 → duration-base ; :53-57 dismiss timer (OQ-5: transitionend or ≥500ms fallback)
el.className = '... opacity-0 transition-opacity duration-300';
setTimeout(function() { ... }, 300);
// :72 hardcoded default
showToast({ message: msg, variant: 'success' });   // → tone
// :84 attribute selector — pairs with atoms.rs data-toast-variant emission
var toasts = document.querySelectorAll('[data-toast-variant]:not([data-toast-handled])');
```

**Tabs — classList literals must equal the SSR strings** (containers.rs:254-275):

```javascript
// ferro-json-ui/src/runtime/tabs.rs:64-71
if (t.getAttribute('data-tab') === value) {
    t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
    t.classList.add('border-primary', 'text-primary', 'font-semibold');
    ...
```

If the tab SSR class pass adds/changes any of these literals, mirror here; the guard test
style to extend is `runtime/mod.rs`:

```rust
// ferro-json-ui/src/runtime/mod.rs:70-79 — string-containment guard over the JS source
#[test]
fn variant_classes_use_semantic_tokens() {
    assert!(FERRO_RUNTIME_JS.contains("bg-primary"));
    ...
    assert!(!FERRO_RUNTIME_JS.contains("bg-blue-500"));
}
```

Add negative assertions for the retired vocabulary (e.g. `!contains("data-toast-variant")`
after the rename) — cheap drift protection in the same test style.

---

### 10. `ferro-json-ui/src/projection/` — emitter updates (service)

```rust
// ferro-json-ui/src/projection/component_map.rs:164-171 — all three arms → Tone::Neutral (D-09)
fn badge_variant_for(meaning: &FieldMeaning) -> BadgeVariant {
    match meaning {
        FieldMeaning::Status => BadgeVariant::Default,
        FieldMeaning::Category => BadgeVariant::Secondary,
        FieldMeaning::Boolean => BadgeVariant::Outline,
        _ => BadgeVariant::Default,
    }
}
```

```rust
// ferro-json-ui/src/projection/component_map.rs:342-354 — ButtonVariant::Link removed (D-07)
pub fn build_relationship_button_props(rel: &RelationshipDef) -> serde_json::Value {
    serde_json::to_value(ButtonProps {
        label: format!("{} \u{2192}", field_display_name(&rel.target)),
        variant: ButtonVariant::Link,          // → Variant::Ghost (visible behavior change)
        size: crate::component::Size::default(),
        ...
```

`builder.rs:370` `CardVariant::Bordered` → `CardAppearance::Bordered`; :688
`ActionItem { variant: None }` type follows the shared `Variant`.

---

### 11. `ferro-json-ui/src/action.rs` — OQ-1 normalization (model)

`DialogVariant` (:22-31) and `NotifyVariant` (:56-67) carry the identical derive stack as the
component enums (excerpted in §1 — same pattern). If OQ-1 is decided "normalize" (RESEARCH
recommendation), both fields rename `variant` → `tone` reusing the shared `Tone` from
`component.rs` (import direction: action.rs already sits beside component.rs; component.rs
imports `crate::action::Action` (:9), so `Tone` must live in a location both can use —
component.rs works since action.rs currently has no import from component.rs; verify no cycle,
else hoist the three enums to their own small module and re-export from component.rs).
Builder ergonomics to preserve: `Action::confirm()` / `confirm_danger()` (:216, :226).

---

### 12. Re-export surfaces — `ferro-json-ui/src/lib.rs` + `framework/src/lib.rs`

Compiler-driven; the two lists to edit:

```rust
// ferro-json-ui/src/lib.rs:47-63 (excerpt)
pub use action::{Action, ActionOutcome, ConfirmDialog, DialogVariant, HttpMethod, NotifyVariant};
pub use component::{
    ActionCardProps, ActionCardVariant, ..., AlertVariant, ..., BadgeVariant, ...,
    ButtonVariant, CardProps, CardVariant, ..., Size, ..., ToastVariant,
};
```

```rust
// framework/src/lib.rs:84-99 — same names behind #[cfg(feature = "json-ui")]
pub use ferro_json_ui::{ ..., AlertVariant, ..., BadgeVariant, ..., ButtonVariant, ...,
    DialogVariant, ..., NotifyVariant, ..., Size, ..., ToastVariant, ... };
```

Old names out, `Variant`/`Tone`/`Size`/`CardAppearance` in, both places (delete completely —
no deprecated aliases, per D-02 and CLAUDE.md feature-branch rules).

---

### 13. ferro-mcp agent-facing surfaces

```rust
// ferro-mcp/src/tools/code_templates.rs:1093-1096
"create-btn": {
  "type": "Button",
  "props": { "label": "Create {{Entity}}", "variant": "default" },   // → "primary"
```

```rust
// ferro-mcp/src/tools/json_ui_validate_spec.rs:107-126 — negative test (empty string still invalid)
fn reports_catalog_error_on_bad_variant() {
    // Alert.variant="" is catalog-invalid ...
    ... {"type": "Alert", "props": {"variant": "", "message": "x"}} ...
// :133-138 — positive test
    ... {"type": "Alert", "props": {"variant": "info", "message": "hello"}} ...
    // → {"tone": "neutral", ...}; the bad-variant test keeps its shape with the new prop name
```

`json_ui_catalog.rs:277-279` ACTION_API prose (`variant: NotifyVariant`,
`DialogVariant (default|danger)`) — update iff OQ-1 normalizes. The 47-count mirror
(:294-295) is untouched. Component schemas auto-derive via `global_catalog()` — no edits.

---

### 14. `app/` sample specs (fixture)

```json
// app/src/views/login.json:8-12, :36-42
"card": { "type": "Card", "props": { ..., "variant": "elevated" } },      // → "appearance": "elevated"
"submit": { "type": "Button", "props": { ..., "variant": "default" } }    // → "variant": "primary"
```

`login_confirm.json`: same Card change at :12; its Button `"variant": "outline"` (:26) is
already canonical. These are also the Chrome MCP visual-verification pages (auth layout).

---

### 15. `docs/src/json-ui/components.md` — canonical enums + migration table (docs)

**Analog for the enum documentation format AND the section to rewrite** — "Shared Enum Values":

```markdown
<!-- docs/src/json-ui/components.md:40-58 -->
## Shared Enum Values

Several props accept fixed-string enum values. The valid strings are listed here; each component section references these by name.

**size** — `"xs"` | `"sm"` | `"default"` | `"lg"`

**button_variant** — `"default"` | `"secondary"` | `"destructive"` | `"outline"` | `"ghost"` | `"link"`

**alert_variant** — `"info"` | `"success"` | `"warning"` | `"error"`

**badge_variant** — `"default"` | `"secondary"` | `"destructive"` | `"outline"`
...
```

This section collapses to three entries (`variant`, `tone`, `size`) — the "one word, one
meaning" statement in docs form. The D-17 migration table goes in this file as a new
`## Component vocabulary migration` section (no in-tree migration-table precedent exists —
use the RESEARCH.md Migration Table Skeleton, lines 129-148, as the content source; the
table format itself matches the file's existing pipe-table style at :26-36).

**Pre-existing doc drift to fix in the same sweep:** `badge_variant` list at :50 omits
`warning`; `column_format` at :52 omits `badge`/`image`/`icon`; components.md:182 documents a
GapSize `"xs"` that doesn't exist (RESEARCH-noted). Also sweep `actions.md` (OQ-1-dependent)
and `forms.md:138` (`.prop("variant", "error")` builder example).

---

### 16. `assets/input.css` + `ferro-base.css` regen (config + generated)

Safelist already covers this phase (verify, likely zero edits):

```css
/* ferro-json-ui/assets/input.css:66-73 */
@source inline("font-sans font-mono");
@source inline("grid-cols-1 ... lg:grid-cols-12");
@source inline("duration-fast duration-base duration-slow ease-base font-display ring-ring");
```

`focus-visible:ring-ring` is NOT safelisted but is generated from the render-source literals
(scanner: `@source "../../ferro-json-ui/src"` at :4) — holds only while every emitted class
is a complete literal (§Shared Patterns, match-arm rule). Regen workflow (run ONCE, after
all class changes, D-04):

```bash
# scripts/gen-ferro-base-css.sh — auto-installs pinned Tailwind CLI into .tooling/bin/
bash scripts/gen-ferro-base-css.sh
# smoke: grep -c "ring-ring\|duration-fast" ferro-json-ui/assets/ferro-base.css
```

---

## Shared Patterns

### Enum definition convention
**Source:** `ferro-json-ui/src/component.rs:49-63` (ButtonVariant, excerpt in §1)
**Apply to:** `Variant`, `Tone`, `Size`, `CardAppearance`, action.rs tone (OQ-1)
Derives `Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, strum::AsRefStr`;
`#[serde(rename_all = "snake_case")]` + `#[strum(serialize_all = "snake_case")]`; `#[default]`
on the default variant; container-level doc comments ONLY (schema-shape, Pitfall 4).

### Match-arm full-literal class strings (Tailwind scanner contract)
**Source:** `badge_inline_html` atoms.rs:261-280 (§4)
**Apply to:** every tone/variant→class mapping and every shared constant
Exhaustive `match` returning `&'static str` full class literals — never `format!("bg-{tone}")`.
Dynamic concatenation is purged from ferro-base.css unless added to `@source inline(...)`.
Make this an explicit review criterion (RESEARCH regen section).

### Renderer error handling (infallible render, HTML-comment diagnostics)
**Source:** `render_badge` atoms.rs:250-256; BadgeCell error arm data.rs:366-372
```rust
let props: BadgeProps = match decode_props(&el.props) {
    Ok(p) => p,
    Err(e) => return decode_diagnostic("Badge", e),
};
```
**Apply to:** unchanged in every touched renderer — preserve while editing; diagnostic prose
mentioning old prop names (`expected object {variant, label}`) updates with the rename.

### Escape discipline
**Source:** doc comment data.rs:349-352 — wrapper markup is server-controlled (unescaped);
every user-supplied string goes through `html_escape` (label, hrefs, attrs).
**Apply to:** all touched render sites — a security property to preserve, per RESEARCH §Security.

### JS/SSR lockstep contract
**Source:** atoms.rs:824-835 ↔ runtime/toasts.rs:4-9,:84 (toasts); containers.rs:254-275 ↔
runtime/tabs.rs:64-71 (tabs)
**Apply to:** any task touching toast/tab classes or data attributes — change both sides in
the same task; close with `grep -rn "data-toast-variant\|VARIANT_CLASSES"` (Pitfall 1).

### Drift-guard test convention
**Source:** catalog.rs:1092-1117 — ONE absolute assertion (the count), everything else
relational; loud comment explaining where the single source of truth lives.
**Apply to:** the D-19 guard (assert canonical `$defs` once; walk relationally) and the
extended strum test.

### Workspace gate (CI-exact) + known environment traps
**Apply to:** every commit.
```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets --all-features -- -D warnings
cargo test --all-features
```
Check `df` / clean `target/` before the full gate; `git checkout docs/protocol/schemas/`
after full test runs (export-test churn); crate-scoped fast loop: `cargo test -p ferro-json-ui`.

## No Analog Found

| File/Artifact | Role | Data Flow | Reason |
|------|------|-----------|--------|
| Transitive `$ref`-resolving schema walker (inside the D-19 guard) | test helper | batch | `render_field_type` (catalog.rs:922-963) handles inline `enum` + `anyOf`-null but never resolves `$ref` (fallback #5 → `<see schema>`). The walker's recursion + visited-set is new code; model its shape extraction on render_field_type and its traversal on the `assemble_full_schema` structure (§3). Self-correcting: it runs against the actually-built catalog in a test. |
| "Component vocabulary migration" docs table | docs | — | No migration-table precedent in `docs/src/` (verified by grep). Content source: RESEARCH.md Migration Table Skeleton (:129-148); format: the existing pipe tables in components.md. |

## Metadata

**Analog search scope:** `ferro-json-ui/src/` (component, action, catalog, layout, loader,
render/*, runtime/*, projection/*), `framework/src/lib.rs`, `ferro-mcp/src/tools/`,
`app/src/views/`, `docs/src/json-ui/`, `ferro-json-ui/assets/`, `scripts/`
**Files scanned:** 22 read directly this session (all line numbers verified against current master)
**Pattern extraction date:** 2026-07-03

**Discrepancies surfaced during mapping** (audit-report-fix discipline; feed to planner):
1. `variant_enums_strum_matches_serde_wire_format` (component.rs:1866-1874) omits
   `BadgeVariant::Warning` — pre-existing test gap; the rewritten test must enumerate all
   variants (or use `strum::VariantArray` to make omission structurally impossible).
2. `docs/src/json-ui/components.md:50` `badge_variant` docs omit `warning`; `:52`
   `column_format` omits `badge|image|icon`; `:182` documents a nonexistent GapSize `"xs"` —
   fold into the D-17/D-18 docs sweep (RESEARCH noted the GapSize case; the first two are
   additional confirmations of the same drift class).
3. `containers.rs:946` doc says ButtonGroup gap "does not influence the emitted CSS" — while
   editing `render_button_group` for the class pass, do not mistake `GapSize` for a size-axis
   rename target (prop is named `gap`, out of D-06 scope).
