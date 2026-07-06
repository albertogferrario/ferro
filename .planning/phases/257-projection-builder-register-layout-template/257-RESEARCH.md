# Phase 257: Projection Builder — Register Layout Template - Research

**Researched:** 2026-07-06
**Domain:** ferro-json-ui projection builder / builder API / sample app flip
**Confidence:** HIGH

## Summary

Phase 257 wires the Register layout template into the projection pipeline. The projection pipeline already supports layout-keyed dispatch in `build_display_spec`; the Register arm follows the same pattern as DataTable, Card, and KanbanBoard. The supporting builder additions (`SpecBuilder.fill_viewport`, `ElementBuilder.each`) are small additions over already-existing private fields. The `/cassa` sample app flip is the proof case — deleting the 89-line hand-authored spec in favour of a projection-derived one.

The element tree for `emit_register_root` must satisfy all four published register lint rules simultaneously: `fill_viewport=true`, root Grid with `fill=true`, SelectionPanel present alongside TileGrid, and layout="dashboard" (an app-shell layout). A two-level Grid structure (outer fill-viewport Grid → Form → inner responsive Grid → panes) achieves this while honouring the 256 D-11 Form-as-DOM-ancestor requirement for tile hidden inputs.

One pre-existing framework limitation requires a targeted fix before the phase's acceptance criteria can be met: `Catalog::validate`'s `strip_expr_objects` replaces `{"$data": ...}` with `""` before per-element props validation. This works for `String` fields but breaks for `Option<u64>` fields like `TileProps.price_cents`, which validate against `anyOf[integer, null]` — neither of which accepts `""`. The fix is to skip per-element validation for elements whose `each.is_some()` (template elements have data-bound props that cannot be type-checked before expansion).

**Primary recommendation:** Implement in order — (1) builder API additions (low-risk, self-contained), (2) catalog_validate fix (enables price_cents emission), (3) `emit_register_root` + "Register" arm in `build_display_spec`, (4) `register_template()` helper, (5) `/cassa` flip + route cleanup.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Register is selected via the existing intent-template override channel: an `IntentSlotTemplate` with `layout: Some("Register")` for Collect, supplied through `VisualContext.templates`. Built-in `default_template(Intent::Collect)` remains Form. No new ServiceDef hint, no new config knob.

**D-02:** ferro-json-ui ships a ready-made helper (`register_template() -> IntentModeTemplates` or similar, exact name planner's call) in `projection/intent_layout.rs`.

**D-04:** The emitted element tree mirrors the cassa.json shape updated to the 256 contract: root Grid with `fill: true`, responsive columns (mobile 1 / md split, spans weighting tiles pane wider), ONE Form with HTML `id` + confirm action as common ancestor, SelectionPanel pane with confirm Button (`disable_on_submit: true` + `form` pairing), TileGrid pane with Tile `$each` template element.

**D-05:** The four register lint rules are the acceptance harness: emitted spec MUST yield ZERO findings from `design::lint` for `register-fill-viewport`, `register-grid-fill`, `register-selection-present`, `fill-viewport-layout-unknown`. Integration test asserts this.

**D-06:** Register arm emits `fill_viewport: true` AND `layout: "dashboard"`. `SpecBuilder.fill_viewport(bool)` must exist.

**D-07:** Numpad NOT part of v1 register template. TileGrid `search: true` enabled by default. Categories/FilterTabs emitted only if cleanly derivable — omitting is acceptable.

**D-08:** Confirm action derives from `ServiceDef.actions`. Exact selection rule is planner's call; no-actions case must be handled (silent broken output not acceptable).

**D-09:** Items collection binds at `/data/{service.name}`. Tile props meaning-driven: `FieldMeaning::Identifier` → `item_id`, `EntityName` → `name`, `Money` → `price` (display) + `price_cents` (machine). Integer cents only. Never hardcoded field names.

**D-10:** Per-row data contract is documented; NO new renderer interpolation surface for hidden-input `field` name.

**D-11:** ONE ServiceDef carrying both browsable items and Collect signals. `IntentHint::Primary(Collect)` available if derive_intents does not score Collect primary. `KNOWN_INTENTS` and seven-intent vocabulary untouched.

**D-12:** `ElementBuilder.each(path, as_)` — public consuming setter over already-private `each: Option<EachDirective>` field. NestedElement stays directive-free.

**D-13:** `SpecBuilder.fill_viewport(bool)` — consuming setter; default stays `false`; existing callers unaffected.

**D-14:** SC-4 test set: `$each` directive serde round-trip through builder; `catalog_validate` accepting directive on products-pane element; integration test covering `$each`-scoped `$data.*` path handling.

**D-15:** `app/src/controllers/cassa.rs` builds ServiceDef in Rust (Italian copy is app-land). `app/src/views/cassa.json` DELETED.

**D-16:** `rimuovi` handler + route DELETED. `conferma` stays.

**D-17:** SC-2: `GET /cassa` returns valid rendered HTML — integration-test through existing app harness. RawHtml grep already passes pre-phase.

**D-18:** CI-exact gate before every commit: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, plus `cargo doc` clean.

**D-19:** `docs/src` documentation is Phase 258 scope. This phase: rustdoc on every new public surface.

### Claude's Discretion

- Exact grid columns/spans/gap numbers for register root; pane order.
- Exact register-template helper name/location and its `slots` list (D-02/D-03).
- Confirm-action selection rule from `ServiceDef.actions` + no-actions behavior (D-08).
- Per-row data-contract key names; use of existing interpolation mechanism if found (D-10).
- Whether sample ServiceDef needs `IntentHint::Primary(Collect)` (D-11).
- Test organization (builder unit tests vs. projection integration tests vs. app-crate e2e).
- SelectionPanel display props passthrough (currency symbol etc.) — neutral defaults.

### Deferred Ideas (OUT OF SCOPE)

- Numpad in the register template
- Category strip derivation hint
- Register template knobs (pane ratios, pane order, search toggle as parameters)
- Sibling FilterTabs↔TileGrid pairing (`data-filter-for`)
- Per-line extra columns generic mechanism
- Barcode wedge, payment flow, receipts, shift close
- MCP `generation_context` register guidance (Phase 258)
- `json_ui_catalog` updates (Phase 258)
- `docs/src` chapters (Phase 258)
- Publish (Phase 258)

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| POS-10 | ServiceDef renders a working sale screen via the Register layout template under the Collect intent; the seven-intent vocabulary is unchanged | Verified: layout-keyed dispatch exists in `build_display_spec`; Collect→Register override flows through existing `VisualContext.templates` channel; four lint rules constitute the acceptance harness; `emit_register_root` element tree satisfies all four rules with the two-grid structure |

</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Register layout arm (`build_display_spec`) | ferro-json-ui projection | — | Projection builder owns layout dispatch; all other layout arms live here |
| `emit_register_root` element assembly | ferro-json-ui projection | — | Structural composition is projection logic; mirrors emit_datatable_root/emit_kanban_root |
| `SpecBuilder.fill_viewport(bool)` | ferro-json-ui spec | — | Builder owns spec field population; `Spec.fill_viewport` already defined in schema |
| `ElementBuilder.each(path, as_)` | ferro-json-ui spec | — | Builder API over existing private field; directive storage already in Element |
| `register_template()` helper | ferro-json-ui projection/intent_layout | ferro-theme (IntentSlotTemplate type) | Helper lives in the crate that owns intent dispatch; uses ferro-theme's open `layout: Option<String>` |
| catalog_validate fix (skip $each elements) | ferro-json-ui catalog | — | Validation pipeline owns strip logic; fix is localised to Stage 2 loop |
| `/cassa` ServiceDef + JsonUiRenderer call | app controllers | ferro-json-ui (renderer) | App-land owns Italian copy and product data synthesis; renderer is library |
| `rimuovi` route deletion | app routes + controllers | — | Server-side handler no longer needed after 256 client-side removal |
| Register lint rules (acceptance harness) | ferro-json-ui design/rules | — | Already shipped in 256; not changed this phase, only asserted against |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ferro-json-ui (in-workspace) | 0.2.86 | Spec, SpecBuilder, ElementBuilder, catalog, projection, design/lint | The target crate for all changes |
| ferro-projections (in-workspace) | 0.2.86 | ServiceDef, IntentHint, derive_intents, Renderer trait | Provides the ServiceDef surface the builder operates on |
| ferro-theme (in-workspace) | 0.2.86 | IntentSlotTemplate, ThemeTemplates, IntentModeTemplates | Types for the Collect→Register template override |
| serde / serde_json | workspace-pinned | Spec/Element serialization; JSON props in catalog validate | Used throughout existing code |

No new dependencies. The phase uses exclusively existing workspace crates. [VERIFIED: grep workspace Cargo.toml]

### Installation

No new packages. All crates already in `Cargo.toml` workspace.

---

## Architecture Patterns

### System Architecture Diagram

```
app/cassa.rs controller
  └─ builds ServiceDef { fields, actions, intent_hints }
  └─ calls derive_intents() → [Collect (Primary), Browse, ...]
  └─ constructs VisualContext { templates: Some(register_template()) }
  └─ calls JsonUiRenderer::render(service_def, intent, ctx)
       └─ Spec::from_service_def(service_def, intent, ctx)
            └─ pick_intent_template(intent, ctx.templates)
                 → IntentSlotTemplate { layout: Some("Register"), slots: [...] }
            └─ build_display_spec(service, &intent, &template, &ctx)
                 └─ match layout { "Register" => emit_register_root(service) }
                      └─ returns (ElementBuilder root, Vec<(String, ElementBuilder)> aux_elements)
                 └─ Spec::builder()
                      .fill_viewport(true)          ← D-13 new setter
                      .layout("dashboard")
                      .root(root_built)
                      .add_elements(aux_elements_built)
                      .build()                      ← catalog_validate runs here
       └─ JsonUiRenderer produces Spec → serialize → HTTP response
```

### Recommended File Locations

```
ferro-json-ui/src/
├── projection/
│   ├── builder.rs          # add "Register" arm + emit_register_root()
│   └── intent_layout.rs    # add register_template() helper
├── spec.rs                 # add fill_viewport setter (SpecBuilder) + each setter (ElementBuilder)
└── catalog.rs              # fix strip_expr_objects for $each template elements

app/src/
├── controllers/cassa.rs    # flip to projection-derived spec; delete rimuovi
├── routes.rs               # delete /cassa/rimuovi/:id route
└── views/cassa.json        # DELETE
```

### Pattern 1: Layout Arm in `build_display_spec`

**What:** Add a "Register" arm to the layout match that returns an ElementBuilder + populates aux_elements, then the surrounding code assembles the Spec.

**When to use:** Every new layout template follows this pattern.

```rust
// Source: ferro-json-ui/src/projection/builder.rs :241
fn build_display_spec(
    service: &ServiceDef,
    intent: &Intent,
    template: &IntentSlotTemplate,
    ctx: &VisualContext,
) -> Result<Spec, ProjectionError> {
    let layout = template.layout.as_deref().unwrap_or("Card");
    let mut aux_elements: Vec<(String, ElementBuilder)> = Vec::new();
    let root = match layout {
        "DataTable" => emit_datatable_root(service),
        "Card"      => emit_card_root(service, &template.slots, &mut aux_elements),
        "Form"      => { return build_input_spec(service); }
        "KanbanBoard" => emit_kanban_root(service, ctx),
        "StatCard"  => emit_statcard_root(service, &template.slots, &mut aux_elements),
        // NEW:
        "Register"  => emit_register_root(service, &mut aux_elements)?,
        other => { return Err(ProjectionError::UnknownComponent { type_name: other.to_string() }); }
    };
    // shared Spec assembly follows...
}
```

Note: "Register" must NOT short-circuit like "Form" does — it participates in the normal Spec assembly path that calls `Spec::builder()`.

### Pattern 2: `emit_register_root` Element Tree

**What:** Returns the root ElementBuilder (outer fill-viewport Grid) and populates aux_elements with all child elements. Mirrors `emit_card_root` (which also uses aux_elements extensively).

**Two-grid structure (resolves Form-ancestor + Grid-root tension):**

```
spec.root → "register_root": Grid(fill=true, columns=1)
              children: ["sale_form"]

"sale_form": Form(id="sale_form", action=<from service.actions>, method=POST)
              children: ["panes_grid"]

"panes_grid": Grid(fill=true, columns=1, md_columns=3, spans=[1,2])
               children: ["selection_pane", "tiles_pane"]

"selection_pane": SelectionPanel(form_id="sale_form")
                   children: ["confirm_btn"]

"confirm_btn": Button(label=<action label>, form="sale_form",
                      button_type=Submit, disable_on_submit=true)

"tiles_pane": TileGrid(form_id="sale_form", search=true,
                       data_path="/data/{service.name}")
               children: ["tile_tmpl"]

"tile_tmpl": Tile($each{path="/data/{service.name}", as_="p"},
                  item_id={"$data":"/p/{id_field}"},
                  name={"$data":"/p/{name_field}"},
                  price={"$data":"/p/{price_field}"},
                  price_cents={"$data":"/p/{price_cents_field}"},
                  field={"$data":"/p/{field_key}"})
```

**Why two grids:** Tile hidden inputs have no `form=` attribute (atoms.rs renders `<input>` as sibling to `<button>` — NOT inside it, and NOT with `form=`). Form must therefore be a DOM ancestor. But lint rule `register-grid-fill` checks that the SPEC ROOT is a Grid with `fill=true`. Solution: outer Grid is root (satisfies lint), Form is its only child (is DOM ancestor), inner Grid handles responsive layout.

**Lint rule compliance verification:**
- `register-fill-viewport`: TileGrid present + `fill_viewport=true` → no finding [VERIFIED: rules.rs:check_pos_fill_viewport]
- `register-grid-fill`: spec.root element IS Grid with `fill=true` → no finding [VERIFIED: rules.rs:check_pos_grid_fill — checks `spec.elements.get(&spec.root)` not children]
- `register-selection-present`: both TileGrid AND SelectionPanel in spec.elements → no finding [VERIFIED: rules.rs:check_pos_cart_present]
- `fill-viewport-layout-unknown`: `spec.layout = "dashboard"` → `is_app_shell_layout` returns true → no finding [VERIFIED: rules.rs:is_app_shell_layout matches "dashboard"|"app"]

### Pattern 3: Builder Setters (D-12, D-13)

**What:** Consuming setter methods (`mut self -> Self`) following the house convention.

```rust
// Source: ferro-json-ui/src/spec.rs — SpecBuilder additions

// D-13: fill_viewport setter
// Add field `fill_viewport_: bool` to SpecBuilder struct
// Default false (unchanged for existing callers)
pub fn fill_viewport(mut self, v: bool) -> Self {
    self.fill_viewport_ = v;
    self
}

// In SpecBuilder::build(), change:
//   fill_viewport: false,
// to:
//   fill_viewport: self.fill_viewport_,

// D-12: each setter on ElementBuilder
// Private field `each: Option<EachDirective>` already exists
pub fn each(mut self, path: impl Into<String>, as_: impl Into<String>) -> Self {
    self.each = Some(EachDirective { path: path.into(), as_: as_.into() });
    self
}
```

### Pattern 4: register_template() Helper (D-02)

**What:** A public function returning `IntentModeTemplates` (or `ThemeTemplates`) that provides the Collect→Register override. Apps supply it to `VisualContext.templates`.

```rust
// Source: ferro-json-ui/src/projection/intent_layout.rs (to add)
pub fn register_template() -> ThemeTemplates {
    ThemeTemplates {
        collect: Some(IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["items".into(), "actions".into()],
                layout: Some("Register".into()),
            },
            input: IntentSlotTemplate::default(),
        }),
        browse: None,
        focus: None,
        process: None,
        summarize: None,
        analyze: None,
        track: None,
    }
}
```

The slots list is Claude's discretion (D-03). `emit_register_root` may ignore slot granularity as `emit_datatable_root` does — state in rustdoc.

### Pattern 5: Meaning-Driven Tile Prop Mapping (D-09)

**What:** Map ServiceDef fields to Tile props via `lookup_meaning`, matching `emit_datatable_root`'s approach.

```rust
// Source: ferro-json-ui/src/projection/builder.rs :292 (emit_datatable_root)
// The same lookup_meaning / is_system_field pattern applies to Tile mapping:

let id_field   = lookup_meaning(&FieldMeaning::Identifier, &service.fields);
let name_field = lookup_meaning(&FieldMeaning::EntityName, &service.fields);
let money_field = lookup_meaning(&FieldMeaning::Money, &service.fields);

// item_id, name, price — derived from meanings, not hardcoded field names
// price_cents — from the same Money field; the cassa handler supplies both
//               "prezzo" (display string) and a cents-integer key in the row
```

The `field` key for TileGrid hidden inputs (required TileProps.field) must come from a row-level key. The existing `cassa.rs` synthesises `field: "qty_{id}"`. The per-row data contract documents this key name so handler and projector agree. [VERIFIED: component.rs:TileProps field is required String]

### Pattern 6: `/cassa` Controller Flip (D-15)

**What:** Replace `JsonUi::render_file("src/views/cassa.json", data)` with full projection call.

```rust
// Source: app/src/controllers/cassa.rs (new shape)
pub async fn index(req: Request) -> Response {
    let service = ServiceDef {
        name: "prodotti".into(),
        // fields with meanings: Identifier, EntityName, Money, + qty field
        // actions: [Action { name: "conferma", route: "cassa.conferma", method: Post }]
        intent_hints: vec![IntentHint::Primary(Intent::Collect)],
        ..Default::default()
    };
    let intents = derive_intents(&service);
    let primary = intents.first().map(|s| s.intent.clone()).unwrap_or(Intent::Collect);
    let ctx = VisualContext {
        base: BaseContext::from_request(&req),
        mode: RenderMode::Display,
        templates: Some(register_template()),
    };
    let spec = JsonUiRenderer::render(&service, &primary, &ctx)?;
    // merge product rows into spec.data at "/data/prodotti"
    let products = vec![ /* synthesised rows with id, nome, prezzo, price_cents, field */ ];
    let data = serde_json::json!({ "prodotti": products });
    JsonUi::render_spec_with_data(&req, spec, data)
}
```

### Anti-Patterns to Avoid

- **"Form" layout short-circuit:** The existing "Form" arm does `return build_input_spec(service)` before reaching the Spec builder. The "Register" arm must NOT do this — it participates in the normal assembly path.
- **Hardcoded field names in emit_register_root:** Use `lookup_meaning()` for all prop mappings. Never assume `"id"`, `"nome"`, `"prezzo"` — those are app-land names.
- **Skipping fill_viewport on spec.data-null specs:** The lint rule `register-fill-viewport` fires only when TileGrid is present. The projector always emits TileGrid, so `fill_viewport` must always be set.
- **Using `render_file` after the flip:** `cassa.json` is deleted; any `render_file` call produces a runtime error. Delete the file and the call together.
- **Placing Tile `$each` on a NestedElement:** NestedElement stays directive-free (Phase 163 deferral was for this exact case, now resolved via ElementBuilder).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Register lint checking | Custom spec validator | `design::lint` with the four register rules | Already shipped in Phase 256; rules.rs is the canonical acceptance harness |
| Responsive layout columns | Per-element CSS magic | `GridProps { columns, md_columns, spans }` | Grid is the existing responsive layout primitive |
| Form-scoped hidden inputs | `form=` attributes on Tile inputs | Form as DOM ancestor | atoms.rs renders Tile hidden inputs without `form=` — this is intentional (D-11); Form ancestor is the solution |
| Template expansion at build time | Pre-expand $each in projector output | `EachDirective` + `resolve.rs::expand_each` | expand_each runs at render time with real row data; projector emits the template, renderer expands it |
| Confirm button submission | Custom JS submit handler | `disable_on_submit: true` + `form: "sale_form"` | 255 shipped the double-submit protection; Button already has both props |
| Meaning → field mapping | hardcoded field names | `lookup_meaning(&FieldMeaning::*, &service.fields)` | Meaning-driven mapping is the whole point of ServiceDef; emit_datatable_root shows the pattern |

**Key insight:** Every mechanism this phase needs already exists in the framework. The work is assembling them correctly, not inventing new primitives.

---

## Critical Finding: catalog_validate and $each Template Elements

**This is a blocking pre-existing limitation that the plan must address.**

### What goes wrong

`Catalog::validate` Stage 2 calls `strip_expr_objects` before per-element JSON Schema validation. `strip_expr_objects` replaces any `{"$data": "/p/..."}` value with `""` (empty string). [VERIFIED: catalog.rs:1162]

For `String` fields, `""` passes JSON Schema (anyOf[string, null] accepts `""`). For `Option<u64>` fields like `TileProps.price_cents`, the schema is `anyOf[integer, null]` — neither branch accepts `""`. Catalog validation FAILS for any Tile element that has a `$data`-bound `price_cents`. [VERIFIED: component.rs:TileProps price_cents type]

Stage 3 (envelope validation) has the same strip call and the same problem.

### Root cause

`strip_expr_objects` was designed for required-String props. The `$each` use case introduces data-bound props of any type, and template elements cannot be type-checked before expansion.

### Recommended fix

In `Catalog::validate` Stage 2, inside the per-element loop, add an early-continue for elements whose `each` field is `Some`:

```rust
// catalog.rs — Stage 2, inside for (id, el) in &spec.elements loop
if el.each.is_some() {
    // Template elements have data-bound props; types cannot be validated
    // before $each expansion. Skip per-element props validation.
    continue;
}
```

The same guard belongs in Stage 3 for $each template elements.

This is a targeted fix (3-4 lines) that does not change validation for non-template elements. The `validate_directives` path (spec.rs:809) already handles $each structural rules separately and is unaffected.

**Impact if not fixed:** `price_cents` is required by D-09 (meaning-driven Money→price_cents mapping). Without the fix, `emit_register_root` cannot emit a catalog-valid spec, blocking SC-1.

---

## Common Pitfalls

### Pitfall 1: Form is not a DOM ancestor → hidden inputs lost

**What goes wrong:** SelectionPanel and TileGrid are both `<div>` elements. Tile hidden inputs have no `form=` attribute. If Form is not a DOM ancestor of the hidden inputs, submitting the form sends no qty values.

**Why it happens:** 256 D-11 specifies Form as the scoping mechanism, not `form=` attributes. atoms.rs emits hidden inputs as siblings of the button, inside a wrapping div — not inside the button (invalid HTML) and without `form=`.

**How to avoid:** Form must be a DOM ancestor of both TileGrid and SelectionPanel in the rendered HTML. In the spec, Form must be an ancestor element of both — achieved by the two-grid structure: outer Grid (root) → Form → inner Grid → [SelectionPanel, TileGrid].

**Warning signs:** `GET /cassa` page loads; qty clicks register client-side; but POST to `/cassa/conferma` sends no `qty_*` fields.

### Pitfall 2: "Register" arm accidentally short-circuits like "Form"

**What goes wrong:** Copying the "Form" arm pattern adds `return build_input_spec(service)` or similar, skipping the Spec builder path. The spec never has `fill_viewport: true` or `layout: "dashboard"`.

**Why it happens:** "Form" is the only existing arm that short-circuits. The others return an `ElementBuilder` and let the shared assembly path handle Spec construction.

**How to avoid:** The "Register" arm returns `emit_register_root(service, &mut aux_elements)?` — an `ElementBuilder`, not a `Result<Spec>`. The `?` is for the function's own error type, not a short-circuit.

### Pitfall 3: `is_app_shell_layout` rejects "Register" as layout

**What goes wrong:** `fill-viewport-layout-unknown` lint rule fires on the emitted spec because `is_app_shell_layout` only accepts `"app"` and `"dashboard"`. If the Register arm emits `layout: Some("Register")` as the spec layout, the lint fires.

**Why it happens:** Confusion between the TEMPLATE layout name ("Register" — used in `IntentSlotTemplate.layout` to select `emit_register_root`) and the SPEC layout name (`"dashboard"` — the HTML shell layout).

**How to avoid:** `spec.layout` must be `"dashboard"` (or `"app"`). `"Register"` is only the template dispatch key, never the spec's layout field. [VERIFIED: rules.rs:is_app_shell_layout]

### Pitfall 4: Tile `$each` on a NestedElement breaks serde

**What goes wrong:** Using `NestedElement` for the tile template instead of a top-level Element with `ElementBuilder.each()`. `NestedElement` has no `each` field; the directive would be silently dropped.

**Why it happens:** `NestedElement` is a different type from `Element`. The Phase 163 deferral note in spec.rs:545 says "If a use case emerges for Rust-side directive injection, add methods in a follow-up phase" — this is the follow-up, but for `ElementBuilder` (top-level elements), not `NestedElement`.

**How to avoid:** The tile template element must be a top-level `Element` (registered in `spec.elements`) with `each: Some(EachDirective { ... })` set via `ElementBuilder.each()`.

### Pitfall 5: OnceLock pollution in projection tests

**What goes wrong:** Tests that call `Spec::from_service_def` (which initialises the global catalog OnceLock) cannot reliably reset state between tests. Later tests see a stale catalog.

**Why it happens:** The global catalog registry is a `OnceLock<Catalog>`.

**How to avoid:** New projection tests MUST use `Spec::from_service_def_with_catalog` with an injected catalog instance. Existing projection tests in `builder.rs` already follow this pattern. [VERIFIED: projection/mod.rs — from_service_def_with_catalog exists]

### Pitfall 6: Schema export test dirties the working tree

**What goes wrong:** `cargo test --all-features` regenerates `docs/protocol/schemas/*.json` (Phase 94 export test). After any test run, the tree appears dirty with schema churn. `$each`/`fill_viewport` are already in the schema — no wire-schema changes this phase. The churn is cosmetic.

**How to avoid:** After the full gate run, `git checkout docs/protocol/schemas/` to discard schema regen churn. Do not fold it into phase commits. [VERIFIED: project memory — project_schema_export_test_dirties_tree.md]

### Pitfall 7: `cargo clippy --all-targets --all-features` catches test-code warnings

**What goes wrong:** Running `cargo clippy --all` without `--all-targets` misses warnings in test modules (dead fixtures, unused imports inside `#[cfg(test)]` blocks). CI uses `--all-targets --all-features`. Pre-push clippy with fewer flags gives false confidence.

**How to avoid:** Always use the CI-exact command: `cargo clippy --all --all-targets -- -D warnings`. [VERIFIED: project memory — feedback_ci_clippy_command_match.md, CLAUDE.md]

---

## Code Examples

### Existing: `emit_datatable_root` structure (reference for emit_register_root)

```rust
// Source: ferro-json-ui/src/projection/builder.rs :292
fn emit_datatable_root(service: &ServiceDef) -> ElementBuilder {
    let data_path = format!("/data/{}", service.name);
    let columns: Vec<ColumnSpec> = service.fields.iter()
        .filter(|f| !is_system_field(f))
        .filter_map(|f| {
            let component = lookup_meaning(&f.meaning, &service.fields);
            // ... build ColumnSpec from component choice
        })
        .collect();
    ElementBuilder::new("DataTable")
        .prop("data_path", json!(data_path))
        .prop("columns", json!(columns))
        // row_actions derived from service.actions
}
```

### Existing: `EachDirective` serde shape

```rust
// Source: ferro-json-ui/src/spec.rs :197
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EachDirective {
    pub path: String,
    #[serde(rename = "as")]
    pub as_: String,
}
```

JSON wire form (already valid in schema): `"$each": {"path": "/data/prodotti", "as": "p"}`

### Existing: `pick_intent_template` override channel

```rust
// Source: ferro-json-ui/src/projection/intent_layout.rs
pub fn pick_intent_template<'a>(
    intent: &Intent,
    templates: Option<&'a ThemeTemplates>,
) -> &'a IntentModeTemplates {
    // user-supplied override wins, then default_template
}
pub fn default_template(intent: Intent) -> IntentModeTemplates {
    match intent {
        Intent::Collect => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["title".into(), "fields".into(), "actions".into()],
                layout: Some("Form".into()),
            },
            input: IntentSlotTemplate::default(),
        },
        // ...
    }
}
```

### Existing: `/cassa` confirm action route

```rust
// Source: app/src/routes.rs
post!("/cassa/conferma", controllers::cassa::conferma).name("cassa.conferma")
// The conferma route name is the Action target in ServiceDef.actions
```

### Existing: `VisualContext` construction

```rust
// Source: ferro-json-ui/src/projection/mod.rs
let ctx = VisualContext {
    base: BaseContext { /* ... */ },
    mode: RenderMode::Display,
    templates: Some(register_template()),  // D-02 helper
};
```

---

## Runtime State Inventory

> Phase 257 is NOT a rename/refactor/migration — it is a greenfield feature addition plus a targeted file deletion. This section is included only to document the `cassa.json` deletion explicitly.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | None — no DB rows reference `cassa.json` | None |
| Live service config | None | None |
| OS-registered state | None | None |
| Secrets/env vars | None | None |
| Build artifacts | `app/src/views/cassa.json` — tracked in git, will be deleted | `git rm app/src/views/cassa.json` as part of the `/cassa` flip commit |

The `app/src/tests/design_lint.rs` test globs `src/views/*.json`. After `cassa.json` is deleted, the test still passes (fewer files, all remaining pass lint). [VERIFIED: design_lint.rs test structure]

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | Yes | stable (rustfmt 1.8.0-stable) | — |
| cargo test --all-features | D-18 gate | Yes | workspace 0.2.86 | — |
| cargo clippy --all --all-targets | D-18 gate | Yes | — | — |
| cargo doc | D-18 gate | Yes | — | — |

No external dependencies. Step 2.6: No missing dependencies that block execution.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in + cargo test |
| Config file | none (workspace-level) |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| POS-10 / D-05 | `emit_register_root` output yields zero findings from all four register lint rules | integration | `cargo test -p ferro-json-ui register_lint` | No — Wave 0 |
| D-12 | `ElementBuilder.each(path, as_)` setter round-trips through serde | unit | `cargo test -p ferro-json-ui each_builder` | No — Wave 0 |
| D-13 | `SpecBuilder.fill_viewport(true)` sets field in built Spec | unit | `cargo test -p ferro-json-ui fill_viewport_builder` | No — Wave 0 |
| D-14 | `catalog_validate` accepts $each element with $data-bound props after catalog_validate fix | integration | `cargo test -p ferro-json-ui catalog_each_template` | No — Wave 0 |
| D-14 | $each directive validates with both null-data and populated-data paths | integration | `cargo test -p ferro-json-ui each_directive_validation` | No — Wave 0 |
| SC-1 | `Spec::from_service_def` with register_template() produces catalog-valid spec | integration | `cargo test -p ferro-json-ui register_projection` | No — Wave 0 |
| SC-2 / D-17 | `GET /cassa` returns 200 with rendered HTML (app-level) | integration | `cargo test -p app cassa_projection_render` | No — Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full CI-exact gate: `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features && cargo doc`

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/projection/tests/register_projection.rs` — SC-1, D-05, D-14
- [ ] `ferro-json-ui/src/spec_tests.rs` (or existing spec test file) — D-12, D-13 builder unit tests
- [ ] `app/src/tests/cassa_render.rs` — SC-2, D-17 app-level render test

*(Use `from_service_def_with_catalog` pattern in all projection tests — not the global OnceLock variant)*

---

## Security Domain

This phase adds no authentication, session management, access control, or cryptographic operations. The `/cassa` endpoint is an existing demo route with no auth changes.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | No | — |
| V5 Input Validation | Peripheral | ServiceDef fields flow into HTML attributes via existing catalog_validate + render pipeline — no new user-input surfaces |
| V6 Cryptography | No | — |

No new threat vectors introduced. The projection pipeline is a server-side spec generator; its output goes through the existing catalog validation gate (SC-1) before rendering.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `register_template()` returning `ThemeTemplates` (not `IntentModeTemplates`) is the correct return type for `VisualContext.templates` | Pattern 4 | If VisualContext.templates is `Option<IntentModeTemplates>`, the helper signature changes — low risk, same code |
| A2 | `from_service_def_with_catalog` accepts an injected `Catalog` instance for test isolation | Pitfall 5 | If the API differs, test pattern needs adjustment — not a blocking concern |
| A3 | `conferma` action route name is `"cassa.conferma"` — used as the Form action in ServiceDef | Pattern 6 | Verified in routes.rs; correct [VERIFIED: app/src/routes.rs :24] |

---

## Open Questions

1. **Confirm-action selection rule (D-08)**
   - What we know: `service.actions` is a `Vec<ServiceAction>`; `emit_datatable_root` uses them for row actions
   - What's unclear: Which action becomes the Form submit? First? Named "conferma"? First with method=POST?
   - Recommendation: Use first action with method=Post; error (not silent omit) if no Post action found; document convention in rustdoc

2. **Per-row `field` key convention (D-10)**
   - What we know: Current `cassa.rs` synthesises `field: "qty_{id}"` client-side; `TileProps.field` is a required String
   - What's unclear: Should the projector emit a $data binding for `field` that expects the row to contain a `field` key, or synthesise the `qty_{id}` pattern itself?
   - Recommendation: Require the row to carry a `field` key (simpler, consistent with data-contract pattern); document the key name in rustdoc and in the cassa controller comment

3. **Whether `derive_intents` scores Collect primary for the sample ServiceDef (D-11)**
   - What we know: `IntentHint::Primary(Collect)` is available as a fallback
   - What's unclear: Whether the ServiceDef fields (items collection + qty fields + confirm action) naturally score Collect
   - Recommendation: Include `IntentHint::Primary(Intent::Collect)` in the sample ServiceDef unconditionally; remove only after verifying derivation scores Collect highest without it

---

## Sources

### Primary (HIGH confidence — verified in source)

- `ferro-json-ui/src/projection/builder.rs` — build_display_spec :241, emit_datatable_root :292, emit_kanban_root :403 [VERIFIED: Read tool]
- `ferro-json-ui/src/spec.rs` — SpecBuilder :359, ElementBuilder :471, EachDirective :197, validate_directives :809 [VERIFIED: Read tool]
- `ferro-json-ui/src/design/rules.rs` — four register rules, REGISTER_TRIGGER_TYPES, is_app_shell_layout [VERIFIED: Read tool]
- `ferro-json-ui/src/catalog.rs` — strip_expr_objects :1162, validate Stage 2 :766, validate Stage 3 :825 [VERIFIED: Read tool]
- `ferro-json-ui/src/component.rs` — TileProps :1359, TileGridProps :1405, SelectionPanelProps :1438, FormProps :263, ButtonProps :299, GridProps :888 [VERIFIED: Read tool]
- `ferro-json-ui/src/render/atoms.rs` — render_tile :1365-1456 (hidden input as sibling, no form= attr) [VERIFIED: Read tool]
- `ferro-json-ui/src/render/containers.rs` — render_tile_grid :908, render_selection_panel :1538 [VERIFIED: Read tool]
- `ferro-json-ui/src/projection/intent_layout.rs` — pick_intent_template, default_template [VERIFIED: Read tool]
- `ferro-json-ui/src/projection/mod.rs` — VisualContext, JsonUiRenderer, from_service_def_with_catalog [VERIFIED: Read tool]
- `app/src/controllers/cassa.rs` — current render_file call, rimuovi handler, conferma handler [VERIFIED: Read tool]
- `app/src/routes.rs` — cassa routes :23-25 [VERIFIED: Read tool]
- `app/src/views/cassa.json` — hand-authored spec (composition reference) [VERIFIED: Read tool]

### Secondary (MEDIUM confidence)

- `.planning/phases/257-projection-builder-register-layout-template/257-CONTEXT.md` — locked decisions, Claude's discretion, deferred items [CITED: planning file]
- `.planning/phases/256-component-renderers-builtin-lockstep/256-CONTEXT.md` — 256 D-11 (Form ancestor scoping), D-04 (price_cents), D-01 (tile markup) [CITED: planning file]

### Tertiary

None. All critical claims verified in source code.

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new dependencies; all existing workspace crates verified
- Architecture: HIGH — element tree verified against lint rules source; Form-ancestor requirement verified in render code; strip_expr_objects limitation verified in catalog.rs
- Pitfalls: HIGH — all pitfalls derived from direct source code inspection; not training-data assumptions
- catalog_validate fix: HIGH — root cause verified at catalog.rs:1162; fix is localised and targeted

**Research date:** 2026-07-06
**Valid until:** 2026-08-06 (stable internal crates; no external dependency drift)
