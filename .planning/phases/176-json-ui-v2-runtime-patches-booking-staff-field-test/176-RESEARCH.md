# Phase 176: v12.0.2 JSON-UI v2 Runtime Patches — Research

**Phase:** 176
**Researched:** 2026-05-20
**Domain:** ferro-json-ui v2 — Card slot template, Grid container, element-level visibility
**Confidence:** HIGH for F7/F8 (root cause verified by source read). MIXED for F9 — see Critical Pre-Planning Finding below.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **F7 visual semantics** — `Card.badge` rendered as a Badge-styled span inside the Card chrome. Variant defaults to `secondary` to differentiate from the title. Layout convention left to plan-time (right-aligned with title, OR below description, OR top-right corner).
- **F8 visual semantics** — `Card.subtitle` rendered as `<p class="text-sm text-text-muted">` immediately below the title and above the description. Vertical stacking: title → subtitle → description → body → footer.
- **F9 acceptance** — "audit which v2 components currently support `visible` and document the union — Grid joining if absent, or fixing the evaluator scope if Grid is supposed to support it." Either fix path acceptable as long as criterion 3 passes.
- **F7+F8 coupling default** — one combined plan touching the same `render_card` template + `CardProps` struct + catalog entry + docs section. Planner may split if doctest discipline becomes unwieldy.
- **F9 independence** — ships as a separate plan; touches a different file set than F7/F8.
- **No version bump in the patch itself** — workspace version stays 0.2.35 in the merge commit; publish/version bump happens later. Phase 175 same posture.
- **No co-author lines** in commits (project rule).

### Claude's Discretion

- Where exactly to slot the `badge` element in Card chrome (right-of-title, below-description, top-right corner)
- Whether `subtitle` lives in the same paragraph spacing wrapper as `description` or as its own `<p>`
- Whether F9 ends up needing any production code change at all (see Critical Pre-Planning Finding)
- Whether the visibility-evaluator audit ships as a separate doc page or extends the existing component docs

### Deferred Ideas (OUT OF SCOPE)

- Clickable / interactive Card.badge or Card.subtitle (file as future finding if needed)
- Systematic `visible` audit on every container component (Card, Form, Tabs, Wave, etc.) — F9 only touches Grid
- v12.0.3 migration story — both new Card props are `Option<String>` and existing specs are unaffected
</user_constraints>

---

## Summary

Phase 176 is a three-finding runtime-patch batch against ferro-json-ui v2, follow-up to Phase 175 (v12.0.1). All three findings come from the gestiscilo-it booking↔staff binding β UAT (chrome-mcp field test 2026-05-20) — a kanban dashboard with countdown badges (F7) + staff-name secondary identifiers (F8) + a per-staff filter chip strip gated on `has_staff` (F9).

**F7 + F8** are unambiguous: `CardProps` does not declare `badge` or `subtitle` fields, so serde's default deserializer silently discards them on the wire. The `render_card` template emits only `title` + `description` + body + footer slots; there is nowhere for `badge` / `subtitle` to land. Fix is purely additive — add two `Option<String>` fields to `CardProps`, emit two new slots in the Card chrome, update the catalog schema (free via `schema_for!(CardProps)` regeneration), update docs.

**F9** is a misdiagnosis in the consumer field report (verified by code read). The CONTEXT's two proposed root causes — (a) Grid does not parse `visible` at all, (b) Grid evaluates against the wrong scope — are BOTH incorrect against the code as it stands. Visibility on Grid (and every other element) is handled at the **walker** (`render_element`) BEFORE component dispatch — Grid never touches `visible`. The `Visibility::evaluate` function and `resolve_path` are both correct. The consumer's symptom is real (the Grid does not render in their UAT) but the root cause is something OTHER than what the field report claims. See Critical Pre-Planning Finding §F9 below for the full trace and the recommended plan-time investigation.

**Critical pre-planning finding (F9):** The planner MUST reproduce the consumer's failing spec end-to-end (load `gestiscilo-it/app/src/views/calendario/calendar_day.json` with the consumer's `data` payload through `JsonUi::render_file`) and capture the actual rendered DOM before writing a fix. The CONTEXT's hypothesized root causes are falsified by the code; the real cause is plan-time discovery work.

---

## Card Template (F7 + F8)

### Current state — `CardProps` struct

[VERIFIED: source read at `ferro-json-ui/src/component.rs:166-179`]

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

No `badge`. No `subtitle`. Serde has no `deny_unknown_fields` attribute (`grep -n "deny_unknown_fields" ferro-json-ui/src/component.rs` → no matches), so consumer JSON carrying `badge: "..."` or `subtitle: "..."` deserializes successfully but the extra keys are silently dropped.

### Current state — `render_card` template

[VERIFIED: source read at `ferro-json-ui/src/render/containers.rs:31-108`]

The current emission order, distilled:

```
<div class="{outer_class}">         (variant: Bordered → border+shadow-sm+p-4 OR Elevated → shadow-md+p-8, no border)
  <div class="{inner_pad}">
    <h3 class="text-base font-semibold leading-snug text-text">{title}</h3>
    {if description}
      <p class="mt-1 text-sm text-text-muted">{description}</p>
    {/if}
    {if !el.children.is_empty()}
      <div class="mt-3 flex flex-wrap gap-3 ...">
        {body — concat of render_element(child) for child in el.children}
      </div>
    {/if}
  </div>
  {if !props.footer.is_empty()}
    <div class="border-t border-border px-6 py-4 flex items-center justify-between gap-2">
      {footer — concat of render_element(footer_id)}
    </div>
  {/if}
</div>
{max_width wrapper if Narrow or Wide}
```

Existing chrome reference: see `render_card_bordered_default` test at line 1060 and `render_card_elevated_no_border` at line 1076.

### Required changes for F7 (`badge`)

**Struct change** (component.rs:168):
Add after `description`, before `max_width`:

```rust
/// Optional small badge text rendered alongside the title. Visually a
/// Badge-styled pill inside the Card chrome — for status indicators,
/// counters, countdown labels, etc. Independent of the title hierarchy.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub badge: Option<String>,
```

**Render change** (containers.rs:67):
Replace the bare `<h3>` emission with a title-row wrapper that holds the title on the left and the badge on the right (or whatever layout convention the planner picks). Recommended pattern, mirroring `render_badge` chrome at atoms.rs:254-270 inlined:

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

(Variant default = Secondary per CONTEXT "Specific Ideas"; class mirrors `render_badge` Secondary chrome at atoms.rs:257.)

### Required changes for F8 (`subtitle`)

**Struct change** (component.rs:168 — same block as F7):
Add after `description`:

```rust
/// Optional muted secondary line rendered immediately below the title and
/// above the description. Pattern: name → role, customer → staff,
/// title → category. Visually `text-sm text-text-muted`.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub subtitle: Option<String>,
```

**Render change** (containers.rs, after the title-row block, before the description block):

```rust
if let Some(ref subtitle) = props.subtitle {
    html.push_str(&format!(
        "<p class=\"mt-0.5 text-sm text-text-muted\">{}</p>",
        html_escape(subtitle)
    ));
}
```

Spacing: `mt-0.5` (4px) keeps the subtitle visually paired with the title, whereas the existing description uses `mt-1` (8px) to separate from the title block. Plan-time decision: planner can align both at `mt-1` for simplicity if visual review prefers it.

### Coupling assessment

**F7 + F8 share:**
- `CardProps` struct (component.rs:168) — both add an `Option<String>` field
- `render_card` template (containers.rs:31) — both add a slot inside the inner padding wrapper
- Catalog schema (catalog.rs:271-275) — auto-regenerates from `schema_for!(CardProps)`; no manual catalog edit needed
- Doc page section "### Card" at `docs/src/json-ui/components.md:76`
- Test module at `containers.rs:1059+` and component round-trip tests at `component.rs:1332+`

**Recommendation: ONE combined plan (175 precedent: F5a + F5b shipped as one plan because shipping one alone unblocks nothing).** Here F7 and F8 are independent — F7 alone closes the countdown-badge use case, F8 alone closes the staff-name use case. But the file-set overlap is total. Splitting would mean two plans modifying the same struct + same render function + same docs section + same test module in adjacent commits. Combined plan is cleaner; the planner gets one wave with both fixes, and the doctests demonstrate both slots interacting (title + subtitle + description + badge all present).

If the planner prefers split for reviewability, the natural split is by render-slot location: 176-01 = F7 (title-row badge) and 176-02 = F8 (subtitle paragraph). They do not collide in line-edits beyond the shared struct.

---

## Grid Template (F9)

### Current state — `GridProps` struct

[VERIFIED: source read at `ferro-json-ui/src/component.rs:809-828`]

```rust
/// Props for Grid component — multi-column layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GridProps {
    /// Number of columns (1-12) at base (mobile) viewport.
    #[serde(default = "default_grid_columns")]
    pub columns: u8,
    /// Number of columns at md breakpoint (768px+). When set, creates a responsive grid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md_columns: Option<u8>,
    /// Number of columns at lg breakpoint (1024px+). Optional; falls back to md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lg_columns: Option<u8>,
    /// Gap between grid items.
    #[serde(default)]
    pub gap: GapSize,
    /// Enables horizontal scroll mode. Children get `min-w-[280px]` and the grid
    /// uses `grid-flow-col` auto-cols layout for Trello-like horizontal scrolling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scrollable: Option<bool>,
}
```

**There is no `visible` field in `GridProps` — and there should not be.** `visible` is element-level, not prop-level (see Visibility Evaluator Architecture below).

### Current state — `render_grid` template

[VERIFIED: source read at `ferro-json-ui/src/render/containers.rs:667-708`]

```rust
pub(crate) fn render_grid(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: GridProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => { return format!("<!-- ferro-json-ui: failed to decode Grid props: {} -->", html_escape(&e.to_string())); }
    };
    let gap = match props.gap { GapSize::None => "gap-0", GapSize::Sm => "gap-2", ... };
    let body: String = el.children.iter().map(|cid| render_element(cid, spec, data, depth + 1)).collect();
    if props.scrollable == Some(true) { return format!("<div class=\"overflow-x-auto\">...{body}...</div>"); }
    let cols = props.columns.clamp(1, 12);
    let mut col_classes = format!("grid-cols-{cols}");
    if let Some(md) = props.md_columns { col_classes.push_str(&format!(" md:grid-cols-{}", md.clamp(1, 12))); }
    if let Some(lg) = props.lg_columns { col_classes.push_str(&format!(" lg:grid-cols-{}", lg.clamp(1, 12))); }
    format!("<div class=\"grid w-full {col_classes} {gap}\">{body}</div>")
}
```

`render_grid` does NOT consult `el.visible` — and it should not, because that's the walker's job at `render_element`.

### Root cause classification

**The CONTEXT's two hypotheses are BOTH wrong against the current code:**

- (a) "Grid does not parse `visible` at all" — false. `visible` is parsed at `Element` level (spec.rs:117 `pub visible: Option<Visibility>`), not at `GridProps` level. The Element deserializer always parses `visible`. Grid is no different from Card / Button / Badge / any other component in this respect.
- (b) "Grid evaluates `visible` against the wrong scope" — false. Grid does not evaluate `visible` at all; `render_element` does, at containers.rs:155-160:

```rust
// (3) Visibility check. Invisible → no output, no children walked.
if let Some(vis) = &el.visible {
    if !vis.evaluate(data) {
        return String::new();
    }
}
```

This runs for every element type identically. The scope is always the same `data: &Value` passed into `render_spec_to_html(spec, data)` from `framework/src/json_ui/mod.rs:154`. There is no per-component scope shifting.

**The actual code path the consumer's failing spec exercises:**

Spec at `gestiscilo-it/app/src/views/calendario/calendar_day.json:73-85`:
```json
"staff_chips_row": {
  "type": "Grid",
  "props": { "columns": 1, "gap": "sm" },
  "children": ["staff_chip"],
  "visible": { "path": "/has_staff", "operator": "eq", "value": true }
}
```

Data populated at `gestiscilo-it/app/src/controllers/calendario/calendar.rs:515`:
```rust
"has_staff": !staff_list_sorted.is_empty(),  // = true in the failing UAT (4 staff present)
```

Pipeline:
1. `Spec::from_json` parses the file → `Element { visible: Some(Visibility::Condition { path: "/has_staff", operator: Eq, value: Some(true) }) }`. ✅
2. `merge_data` lands `has_staff: true` into `spec.data`. ✅
3. `expand_directives` removes `$if`-falsy (no `$if` here), expands `$each` on `staff_chip` template (4 chips), rewrites `staff_chips_row.children` from `["staff_chip"]` to `["staff_chip-0", "staff_chip-1", "staff_chip-2", "staff_chip-3"]`. ✅
4. `resolve_actions` + `resolve_expressions` walk props. Neither touches `el.visible` (expression.rs:19 documents this explicitly). ✅
5. `render_spec_to_html(resolved_spec, &spec.data)` walks from root. Hits `staff_chips_row`. Visibility check: `resolve_path(&data, "/has_staff") = Some(&Value::Bool(true))`; operator Eq with target `Some(Value::Bool(true))`; result = `true`. Element renders. ✅

By this trace, the Grid SHOULD render. The consumer's symptom (Grid absent from DOM) cannot be reproduced from the code path above.

**Possible real causes (planner verifies at plan time, in priority order):**

1. **The consumer's chrome-mcp snapshot was taken against a different code state than the one in this repo.** Maybe the consumer's local-path `ferro` checkout was at a stale commit. The planner reproduces the failure end-to-end on a fresh checkout. If the Grid renders correctly, the finding is invalid and F9 is a no-op (close as "could not reproduce; consumer re-test required").
2. **Some other element in the same spec fails to render, and the consumer misattributed the missing DOM region to Grid.** The chip-strip Grid is rendered inside the root Grid (depth 2). If the root Grid emits but `staff_chips_row` is in the children list AND `render_element("staff_chips_row", ...)` returns "" for some other reason (e.g. all 4 chip children fail to decode and the body is empty BUT THE GRID DIV SHOULD STILL BE EMITTED — so this theory also fails on inspection). One plausible variant: BadgeProps deserialization fails for the `staff_chip` clones because their JSON contains `href` (which BadgeProps does not declare — see component.rs:365-370). But `serde_json::from_value` on a struct without `deny_unknown_fields` ignores extra keys, so this would not fail either.
3. **The catalog's full-spec envelope validator (catalog.rs:722-740) emits an error that aborts the resolve pipeline.** But validate's contract is to log errors and continue (catalog.rs:55-61); it does not abort. Falsified.
4. **`expand_directives` rewriting of `staff_chips_row.children` mistakenly drops it from the root's children list.** `rewrite_parent_children` at resolve.rs:504-523 iterates each element's children and rewrites/prunes. It only prunes IDs in the `if_removed` set; `staff_chips_row` itself has no `$if` (only `visible`), so it is never in `if_removed`. Falsified.
5. **The consumer's `has_staff_widget` (a different field at booking_dettaglio.json:172) is being confused with `has_staff` in the calendar_day view.** Plausible authoring error on the consumer side — worth checking the chrome-mcp output for the actual data payload.

The strongest plan-time investigation is: **reproduce against current ferro master**. If it does not repro, F9 closes as invalid finding. If it does, the actual cause becomes plan-time discovery.

### Visibility evaluator — verified architecture

**File path:** `ferro-json-ui/src/visibility.rs` (single file, 200 lines).

**Key types:**
- `Visibility` (enum) — `And { and: Vec<Visibility> }`, `Or { or: Vec<Visibility> }`, `Not { not: Box<Visibility> }`, `Condition(VisibilityCondition)`. Hand-rolled `Deserialize` (lines 69-114) for legible error messages on malformed input.
- `VisibilityCondition { path: String, operator: VisibilityOperator, value: Option<Value> }`
- `VisibilityOperator` — `Exists, NotExists, Eq, NotEq, Gt, Lt, Gte, Lte, Contains, NotEmpty, Empty, IsTrue, IsFalse`. (Note Phase 165 F13 added `IsTrue` / `IsFalse`.)

**Evaluation function:** `Visibility::evaluate(&self, data: &Value) -> bool` at visibility.rs:134. Compositional (And/Or/Not), terminating at `evaluate_condition` (line 144). Infallible — malformed conditions, missing paths, and type mismatches all resolve to `false`.

**Path resolution:** `evaluate_condition` calls `crate::data::resolve_path(data, &c.path)` at visibility.rs:146. `resolve_path` (data.rs:19-45) takes a slash-separated path like `/has_staff` against the data root. There is NO per-component scope shifting.

**Where the walker checks visibility:** `ferro-json-ui/src/render/mod.rs:155-160` (inside `render_element`):
```rust
// (3) Visibility check. Invisible → no output, no children walked.
if let Some(vis) = &el.visible {
    if !vis.evaluate(data) {
        return String::new();
    }
}
```

**Which components support visible:** ALL of them. Visibility is element-level, checked once per element in the walker, BEFORE component dispatch. There is no concept of "component X supports visible but component Y doesn't". The walker code is shared across all 42 builtin types and every plugin component.

**Consumer evidence that visibility works for SOME components:** consumer's `calendar_day.json:67-71` declares `Badge.visible` on the `summary_badge`, and the field report says it "renders correctly when its path evaluates to true". Same evaluator path that Grid would hit. This is further evidence that the F9 root cause is something other than what CONTEXT proposes.

### Required changes (contingent on plan-time reproduction)

**If F9 reproduces:**
- Plan-time discovery work surfaces the actual root cause. Fix it.
- Add an integration test asserting the chip-strip Grid renders when `has_staff: true` and is absent when `has_staff: false`, using the exact spec shape from the consumer's `calendar_day.json:73-85`.

**If F9 does NOT reproduce:**
- F9 ships as a regression test only (no production code change). The test fixture exercises a Grid with `visible: {path: "/flag", operator: "eq", value: true}` against `data: {flag: true}` and `data: {flag: false}`; asserts the Grid is present in the first case and absent in the second.
- Update `docs/src/json-ui/components.md` Grid section to add an explicit note about `visible` behaving the same on Grid as on every other element. This addresses the "audit which v2 components currently support visible and document the union" success criterion.
- Mark the finding as "could not reproduce against current ferro master; consumer to re-test with patched runtime".

**Either way, the test is the load-bearing artifact for success criterion 3.**

---

## Spec / Catalog / Docs

### JSON schema authority

[VERIFIED: source read]

**CardProps schema:** auto-generated via `schemars::JsonSchema` derive at `ferro-json-ui/src/component.rs:167`. Catalog entry at `catalog.rs:270-275`:

```rust
(
    "Card",
    "Content container with title, description, body children, and optional footer slot.",
    || to_value(schema_for!(CardProps)).unwrap(),
    &["footer"],
),
```

Adding `badge: Option<String>` and `subtitle: Option<String>` to `CardProps` regenerates the catalog schema automatically — `schema_for!(CardProps)` reflects the new fields without any catalog.rs edit. The catalog description string in the entry should be updated to mention the new slots:

```rust
"Content container with title, description, optional badge and subtitle, body children, and optional footer slot."
```

**GridProps schema:** same pattern at `catalog.rs:306-311`. No changes needed for F9 — `visible` is on `Element`, not on `GridProps`.

### MCP catalog touchpoints

[VERIFIED: source read]

`ferro-mcp` consumes `global_catalog()` for the `json_ui_catalog` MCP tool. The tool emits per-component prop schemas from the catalog `Component` entries. Since `schema_for!(CardProps)` regenerates automatically, the MCP tool's Card entry will automatically expose `badge` and `subtitle` as optional string fields once the struct is updated.

No manual ferro-mcp edits required. The contract is one-way: catalog → MCP. F7+F8 ship; the MCP tool's next response reflects them.

### Docs touchpoints

[VERIFIED: source read]

**File:** `docs/src/json-ui/components.md`
- **F7+F8 docs:** Card section at line 76 — add `badge` and `subtitle` rows to the prop table at lines 80-83, plus an example block showing the new slots in use. Mirror the existing "Variant" subsection style at lines 134-156.
- **F9 docs:** Grid section at line 158 — add a subsection clarifying that `visible` on Grid works identically to `visible` on every other element (since this came out of consumer confusion, an explicit doc paragraph closes the ambiguity). Phase 175's F4 docs (Switch ⟷ Checkbox-styled-as-switch) at `175-05-PLAN.md` is the analog precedent for adding a clarifying doc section.

No changes to `actions.md` / `data-binding.md` / `expressions.md` / `getting-started.md` — those don't reference Card or Grid prop shape.

---

## Test Strategy

### Existing test patterns

[VERIFIED: source read]

**Test infrastructure:**
| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (`cargo test`) |
| Config file | none (workspace `Cargo.toml`) |
| Quick run | `cargo test -p ferro-json-ui` |
| Full suite | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

**Existing Card / Grid test locations:**
- `ferro-json-ui/src/render/containers.rs:1059` onward — `render_card_bordered_default`, `render_card_elevated_no_border`, `render_card_omitted_variant_is_bordered`, `card_max_width_narrow_wraps_in_mx_auto`
- `ferro-json-ui/src/render/containers.rs:855` onward — `grid_recurses_children`, `grid_scrollable_emits_flow_col`
- `ferro-json-ui/src/component.rs:1332` — `card_props_round_trips_footer` (CardProps serde discipline)
- `ferro-json-ui/src/component.rs:1112` — `schema_for_card_props_generates` (schemars discipline)
- `ferro-json-ui/src/visibility.rs:213-654` — comprehensive Visibility unit tests including condition shape round-trip, And/Or/Not composition, every operator, missing-path edge cases

**Test fixture builder:** `build_spec(elements: Vec<(&str, ElementBuilder)>)` at `containers.rs:846` — wraps `Spec::builder()` for fluent test setup. Used by every container test.

### Test surface per finding

**F7 (`Card.badge`):**
- Unit test in `containers.rs` tests module: `render_card_emits_badge_when_present` — assert HTML contains the badge label + the Badge-style class (e.g. `bg-secondary/10 text-secondary-foreground`).
- Unit test: `render_card_omits_badge_when_absent` — assert no `bg-secondary` and no rounded-pill span when `badge: None`.
- Unit test in `component.rs` tests module: `card_props_round_trips_badge` — mirror `card_props_round_trips_footer` (line 1332) and `card_props_omits_empty_badge_in_json` mirroring `card_props_omits_empty_footer_in_json` (line 1358).
- Doctest on the new `pub badge: Option<String>` field comment block — a small end-to-end snippet showing the badge in a spec literal.
- Schemars regression: ensure `assert_schema_nonempty_object::<CardProps>("CardProps")` at component.rs:1112 still passes (auto — schemars handles it).

**F8 (`Card.subtitle`):**
- Unit test in `containers.rs` tests module: `render_card_emits_subtitle_when_present` — assert HTML contains the subtitle text + muted class.
- Unit test: `render_card_omits_subtitle_when_absent`.
- Unit test in `component.rs`: `card_props_round_trips_subtitle` + `card_props_omits_empty_subtitle_in_json` (mirror pattern).
- Doctest similar to F7.

**Combined F7+F8 (if shipped together):**
- Unit test: `render_card_emits_title_subtitle_description_badge_together` — full happy-path with all four text slots populated.

**F9 (`Grid.visible`):**
- Unit test in `containers.rs` tests module: `grid_renders_when_visible_true` — build a spec with `Grid` element having `visible: {path: "/flag", operator: "eq", value: true}` and `data: {flag: true}`; assert the rendered HTML contains `<div class="grid`.
- Unit test: `grid_hidden_when_visible_false` — same spec, `data: {flag: false}`; assert HTML does NOT contain `<div class="grid` (the entire Grid div should be absent).
- Integration-style test (in same module, since the renderer is pure): `grid_visible_consumer_reproduction` — replicate the chip-strip spec from `calendar_day.json:73-85` against the consumer's exact data shape; assert the four chip clones render inside the Grid.
- If F9 root cause IS found (not just "regression test only"): add a test asserting the specific failure mode is closed.

### Wave 0 — tests to add

Mirror Phase 175 VALIDATION.md structure:

- [ ] `containers.rs` (tests module) — `render_card_emits_badge_when_present`, `render_card_omits_badge_when_absent`, `render_card_emits_subtitle_when_present`, `render_card_omits_subtitle_when_absent`, `render_card_emits_title_subtitle_description_badge_together`
- [ ] `containers.rs` (tests module) — `grid_renders_when_visible_true`, `grid_hidden_when_visible_false`, `grid_visible_consumer_reproduction`
- [ ] `component.rs` (tests module) — `card_props_round_trips_badge`, `card_props_round_trips_subtitle`, `card_props_omits_empty_badge_in_json`, `card_props_omits_empty_subtitle_in_json`

---

## Plan Shape Recommendation

**Recommended: 2 plans.**

### Plan 176-01 — F7+F8 combined: Card badge + subtitle slots
- **Wave:** 1
- **Depends on:** none
- **Files modified:**
  - `ferro-json-ui/src/component.rs` — CardProps + tests
  - `ferro-json-ui/src/render/containers.rs` — render_card + tests
  - `ferro-json-ui/src/catalog.rs` — description string only (schema auto-regenerates)
  - `docs/src/json-ui/components.md` — Card section: prop table + example
- **Approx LOC:** +60 (props +6, render +20, tests +25, docs +10)

### Plan 176-02 — F9: Grid visibility reproduction + audit
- **Wave:** 1 (independent of 176-01 — no shared files)
- **Depends on:** none
- **Files modified:**
  - `ferro-json-ui/src/render/containers.rs` — tests module ONLY (or production code IF the planner identifies a real root cause)
  - `docs/src/json-ui/components.md` — Grid section: visibility clarification paragraph
- **Approx LOC:** +30 if regression-test-only; potentially more if a real root cause is found.

Both plans can run in parallel (Wave 1). Each ends with `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green per project convention.

**Alternative: 3 plans** if the planner wants F7 and F8 split for review granularity (176-01 = F7, 176-02 = F8, 176-03 = F9). Acceptable but the file-set overlap between F7/F8 is total; the combined plan is cleaner.

---

## Risks & Coupling

| Risk | Plan | Mitigation |
|------|------|-----------|
| F7 badge chrome conflicts with existing Card title styling | 176-01 | Visual review against `render_card_bordered_default` test output before commit; if the title-row wrapper changes the default appearance of badge-less Cards, the wrapping is conditional |
| F8 subtitle spacing collides visually with description (`mt-1`) | 176-01 | Use `mt-0.5` (4px) for subtitle vs `mt-1` (8px) for description; visual review |
| F9 cannot be reproduced against current ferro | 176-02 | Plan documents the reproduction attempt explicitly; if no repro, ships as regression test + docs clarification (no production code change). This is a legitimate outcome — Phase 175 F4 set the precedent (Switch did not need code changes after F1 closed the depth issue) |
| Catalog description string in catalog.rs:272 drifts from reality | 176-01 | Update at the same time as CardProps; add to plan's `must_haves.truths` |
| `schemars` schema regeneration produces unexpected JSON | 176-01 | Existing `schema_for_card_props_generates` test (component.rs:1112) is the canary — passes if schema generates non-empty object; planner should also eyeball the generated JSON shape with a `cargo run --bin ferro -- generate-schema Card` if such a binary exists, OR a one-off test that asserts `badge` appears in the schema's properties |
| F9 consumer re-test step (chrome-mcp UAT) cannot run from this repo | 176-02 | Per Phase 175 precedent (VALIDATION.md Manual-Only Verifications), this is acceptable — flagged as manual verification, executed in the gestiscilo-it consumer repo by the operator |

**No cross-plan coupling.** Both plans touch `containers.rs` but in non-overlapping locations: 176-01 in `render_card` (line 31-108) and its tests (line 1059+); 176-02 in `render_grid` tests only (line 855+).

---

## Consumer Re-Test Loop

**Local-path consumer status (verified by reading memory note `project_v12_merge_task.md` ref and feedback note `feedback_friction_loop_release_cadence.md`):**

ferro is in a friction loop with gestiscilo-it. Per memory: "single publish at Phase 161 (v12.0 merge to master)". Phase 161 is the v12.0 merge gate. **Phase 176 is post-merge** (v12.0 is on master per STATE.md "Push master + publish v12.0 release") but pre-publish of the next patch. The consumer (gestiscilo-it) consumes ferro via local-path dependency in `Cargo.toml`.

**Operational consequence:**
- No crates.io publish needed for Phase 176 to be testable by the consumer.
- After 176-01 + 176-02 land, the consumer cargos against the patched ferro path and re-runs their β UAT.
- The chrome-mcp re-test result lands in the gestiscilo-it repo (the consumer's `.planning/phases/152-booking-staff-binding/` UAT artifacts), not in the ferro repo.
- Phase 176 success criterion 6 ("consumer re-runs its β UAT against the patched runtime and confirms F7/F8/F9 closed") is a Manual-Only Verification in the VALIDATION.md sense.

**Publish discipline reminder** (from memory `feedback_friction_loop_release_cadence.md`): when ferro is in a friction loop with a consumer, publish ONCE at the end. Phase 176 should NOT bump the workspace version or push to crates.io. The next publish event is whatever batches v12.0.1 + v12.0.2 + any further patches together.

---

## Validation Architecture

For VALIDATION.md generation. Mirror Phase 175's VALIDATION.md structure (16 task-level rows + 1 phase gate).

**Dimensions:**

1. **Per-finding render correctness (DOM assertion)** — F7 badge slot in DOM, F8 subtitle slot in DOM, F9 Grid + children in DOM when visible-true / absent when visible-false. Evidence: unit tests in `containers.rs` tests module; specific assertions on HTML substrings.

2. **Schema correctness (catalog JSON)** — CardProps schema regenerates with new `badge` and `subtitle` properties. Evidence: existing `schema_for_card_props_generates` test (component.rs:1112) passes; an additional assertion that the generated schema's `properties` map contains `badge` and `subtitle` keys closes the gap. F9 has no schema change.

3. **Serde round-trip / no-additional-properties regression** — `CardProps` with `badge: Some("X")` and `subtitle: Some("Y")` round-trips through JSON serialization and parses back identically. Evidence: `card_props_round_trips_badge`, `card_props_round_trips_subtitle` plus the `_omits_empty_*` skip-serializing variants.

4. **Regression — existing Card behavior unchanged** — `render_card_bordered_default`, `render_card_elevated_no_border`, `render_card_omitted_variant_is_bordered`, `card_max_width_narrow_wraps_in_mx_auto`, `card_props_round_trips_footer` all still pass. Evidence: existing tests in containers.rs + component.rs are not modified, just augmented.

5. **Regression — existing Grid behavior unchanged** — `grid_recurses_children`, `grid_scrollable_emits_flow_col` still pass. Visibility tests for OTHER components (Badge.visible, Card.visible) still pass.

6. **Visibility evaluator regression** — all existing `visibility.rs` tests (213-654) still pass. Phase 165 F13 (`IsTrue` / `IsFalse`) tests unchanged.

7. **Docs correctness** — `docs/src/json-ui/components.md` mentions `badge`, `subtitle` in the Card section AND clarifies Grid visibility semantics. Evidence: `grep -q "badge" docs/src/json-ui/components.md` and `grep -q "subtitle"` and `grep -q "visible" docs/src/json-ui/components.md` (the Grid clarification text).

8. **Consumer re-test (Manual-Only)** — gestiscilo-it β UAT re-runs against the patched ferro local-path dependency; chrome-mcp snapshot shows badge text "Scade tra Nm", subtitle text "Marco Rossi", and chip strip Grid visible when `has_staff: true`. Cannot be reduced to a Rust unit test.

9. **Phase gate** — `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green at HEAD.

**Phase-level acceptance:** all 8 evidence dimensions green + manual-verification dimension confirmed by the consumer.

---

## Canonical References (read these before planning)

**Phase context and decisions:**
- `.planning/phases/176-json-ui-v2-runtime-patches-booking-staff-field-test/176-CONTEXT.md` — phase scope + locked decisions (F7/F8 visual semantics, F9 acceptance criteria)
- `.planning/ROADMAP.md` lines 1918-1942 — Phase 176 description, success criteria, requirements

**Phase 175 precedent (read before slicing 176):**
- `.planning/phases/175-json-ui-v2-runtime-patches-staff-domain-field-test/175-CONTEXT.md` — same loop pattern
- `.planning/phases/175-json-ui-v2-runtime-patches-staff-domain-field-test/175-RESEARCH.md` — research artifact this file mirrors
- `.planning/phases/175-json-ui-v2-runtime-patches-staff-domain-field-test/175-VALIDATION.md` — VALIDATION.md template
- `.planning/phases/175-json-ui-v2-runtime-patches-staff-domain-field-test/175-04-PLAN.md` — F2 plan (CheckboxGroup) — closest shape to F7/F8 (component-extension)
- `.planning/phases/175-json-ui-v2-runtime-patches-staff-domain-field-test/175-05-PLAN.md` — F4 plan (Switch) — closest shape to F9 (docs + regression test, possibly no production code)

**Consumer field test (verbatim source):**
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md` — Bugs R2/R3/R4 corresponding to F7/F8/F9
- `/Users/alberto/repositories/gestiscilo-it/app/src/views/calendario/calendar_day.json` — the spec exercising F9 (chip strip Grid lines 73-85)
- `/Users/alberto/repositories/gestiscilo-it/app/src/controllers/calendario/calendar.rs` — data emission for the failing UAT (line 504-526)

**Source files to read at plan time:**
- `ferro-json-ui/src/component.rs:166-179` — CardProps struct
- `ferro-json-ui/src/component.rs:809-828` — GridProps struct
- `ferro-json-ui/src/component.rs:1112` — `schema_for_card_props_generates` test
- `ferro-json-ui/src/component.rs:1332-1369` — CardProps serde round-trip + omit-empty tests
- `ferro-json-ui/src/render/containers.rs:31-108` — render_card
- `ferro-json-ui/src/render/containers.rs:667-708` — render_grid
- `ferro-json-ui/src/render/containers.rs:846-1105` — Card/Grid tests + `build_spec` helper
- `ferro-json-ui/src/render/mod.rs:131-160` — render_element (visibility check at line 155-160)
- `ferro-json-ui/src/render/atoms.rs:249-271` — render_badge (chrome reference for F7 inline-badge style)
- `ferro-json-ui/src/visibility.rs` — entire file (Visibility enum + evaluate + all tests)
- `ferro-json-ui/src/spec.rs:95-138` — Element struct (visible field at line 117)
- `ferro-json-ui/src/data.rs:19-45` — resolve_path
- `ferro-json-ui/src/resolve.rs:233-242` — expand_directives pipeline (for F9 root cause investigation)
- `ferro-json-ui/src/catalog.rs:269-311` — Card + Grid catalog registrations
- `framework/src/json_ui/mod.rs:74-206` — render entry / pipeline (for F9 reproduction)
- `docs/src/json-ui/components.md:76-208` — Card + Grid + (others as analog) doc sections

**Project convention:**
- `CLAUDE.md` — testing discipline, no co-author lines, project-agnostic crate rule
- `~/.claude/projects/-Users-alberto-repositories-albertogferrario-ferro/memory/feedback_friction_loop_release_cadence.md` — single publish at end of friction loop; do NOT bump version mid-loop

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | F7 + F8 ship as one combined plan with shared edits to CardProps / render_card / catalog / docs | Plan Shape | Low — even if split, the file set is identical; reviewer experiences a doubled commit chain but no merge conflict |
| A2 | F9 cannot be reproduced against current ferro master code (visibility-evaluator architecture verified correct) | F9 Root Cause | MEDIUM — if F9 DOES reproduce, the plan-time investigation must identify the actual cause. The regression-test-only outcome is conditional on no-repro |
| A3 | `schemars` auto-regenerates the catalog JSON for CardProps after adding optional fields, no manual edit | Spec / Catalog | Low — schemars derive is already in place; the schema_for_card_props_generates test guards the contract |
| A4 | The Badge-style inline span chrome for F7 visually matches consumer expectations from CONTEXT "Specific Ideas" | F7 Visual | Low — CONTEXT explicitly says "Badge component-styled, top-right of title"; the proposed flex-justify-between wrapper matches |
| A5 | The consumer's local-path ferro dependency picks up these changes without a publish | Consumer Re-Test | Verified — memory note confirms single publish at friction-loop end |
| A6 | No new component added to BUILTIN_TYPES; no count assertion changes | Plan Shape | Verified — BUILTIN_TYPES contains "Card" and "Grid" already at render/mod.rs:64,79; only props change |
| A7 | No new visibility operator added; existing operators sufficient for F9 | F9 | Verified — consumer uses `eq` against a boolean, which is fully supported; alternatively `is_true` (Phase 165 F13) would work too |

---

## Metadata

**Confidence breakdown:**
- F7 / F8 root cause: HIGH (CardProps lacks badge/subtitle fields; serde silently drops unknown keys; render_card has no slot for them — all verified by source read)
- F9 root cause: LOW for the CONTEXT-stated hypotheses; MEDIUM for the "cannot reproduce against current code" hypothesis — plan-time reproduction is the load-bearing step
- Plan shape (2 plans): HIGH
- Test strategy: HIGH (existing test patterns extensively documented in containers.rs)

**Research date:** 2026-05-20

**Valid until:** Stable against the current ferro-json-ui codebase. Re-research if `render_element` visibility pipeline (mod.rs:131-160) changes or if CardProps grows additional fields.

---

## RESEARCH COMPLETE
