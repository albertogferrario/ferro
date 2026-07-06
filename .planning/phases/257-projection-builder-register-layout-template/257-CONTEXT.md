# Phase 257: Projection Builder — Register Layout Template - Context

**Gathered:** 2026-07-06 (auto mode — recommended defaults selected, logged in 257-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

The projection pipeline learns the Register layout, and the `/cassa` sample
proves it end-to-end:

- **`layout: "Register"` arm** in
  `ferro-json-ui/src/projection/builder.rs::build_display_spec()` +
  **`emit_register_root()`** emitting the register composition: fill-viewport
  Grid, one Form common ancestor, SelectionPanel pane (with confirm Button),
  TileGrid pane with a Tile `$each` template (POS-10).
- **Builder API additions**: `Spec::builder().fill_viewport(bool)` (currently
  hardcoded `false` in `SpecBuilder::build()`) and
  `ElementBuilder.each(path, as_)` (public setter over the existing private
  `each` field; serde + `catalog_validate` round-trip per SC-4).
- **Collect→Register template wiring**: the register template is an
  `IntentSlotTemplate { layout: Some("Register"), .. }` supplied through the
  EXISTING `VisualContext.templates` (ThemeTemplates) channel; the built-in
  `default_template(Collect)` stays Form.
- **`/cassa` flip**: the sample app serves a projection-derived spec — the
  hand-authored `app/src/views/cassa.json` is deleted and the controller
  builds a `ServiceDef` + renders through `JsonUiRenderer`.

NOT this phase: MCP `generation_context` register guidance, `json_ui_catalog`
updates, `docs/src` chapters, publish (all Phase 258); new intents or changes
to the seven-intent vocabulary / `KNOWN_INTENTS` (frozen); new builtin
components or runtime JS (256 shipped them); payment flow / receipts / shift
close (out of milestone scope).

**Milestone constraints carried into every decision:** seven-intent
vocabulary frozen — Register is a LAYOUT template name, never an intent; no
new parallel archetype surface (PROJECT.md v16.6); structural vocabulary only
in `ferro-*` crates; every emitted class a full string literal; no new
crates; single publish at Phase 258.

**World-state corrections found during scouting (feedback_validate_scope_premises):**
1. `grep -rn RawHtml app/src/` returns zero hits TODAY — the RawHtml
   elimination already happened (the sample is a hand-authored JSON spec).
   SC-2's grep gate passes pre-phase; the substantive SC-2 work is the
   projection derivation + `cassa.json` deletion. Verification must not
   claim the grep as evidence of this phase's work.
2. The current `cassa.json` cart pane still uses the pre-256 DataTable +
   StatCard composition — the flip simultaneously upgrades `/cassa` to the
   256 SelectionPanel live view.
3. `Element.$each` + full `validate_directives` rules already exist
   (spec.rs:809 — reserved `as` names, path-resolves-to-array when data
   non-null, correlated-sibling allowance, nested-`$each` rejection). What's
   missing is only the ElementBuilder setter and the projector emission.

</domain>

<decisions>
## Implementation Decisions

### Register template selection (Collect→Register)
- **D-01:** Register is selected via the existing intent-template override
  channel: an `IntentSlotTemplate` with `layout: Some("Register")` for
  Collect, supplied through `VisualContext.templates`
  (`pick_intent_template` already dispatches it). The built-in
  `default_template(Intent::Collect)` remains Form — existing Collect
  projections and tests are untouched. No new ServiceDef hint, no new config
  knob (feedback_no_duplicate_control_surface). ferro-theme needs NO code
  change (`IntentSlotTemplate.layout` is an open `Option<String>`).
- **D-02:** ferro-json-ui ships a ready-made helper (suggested:
  `register_template() -> IntentModeTemplates` in
  `projection/intent_layout.rs`; exact name/location planner's call) so apps
  and agents get the Collect→Register override without hand-building
  template JSON. Phase 258's `generation_context` will point at it.
- **D-03:** The register template's `slots` list semantics are planner's
  call — `emit_register_root` may ignore slot granularity the way
  `emit_datatable_root` does; whatever is chosen must be stated in rustdoc.

### emit_register_root composition
- **D-04:** The emitted element tree mirrors the battle-tested hand-authored
  `cassa.json` shape updated to the 256 contract (256 D-11): root Grid with
  `fill: true`, responsive columns (mobile 1 / md split, spans weighting the
  tiles pane wider — exact numbers planner's call, cassa.json's
  `columns: 1, md_columns: 3, spans: [1, 2]` is the reference); ONE `Form`
  (with HTML `id` + confirm action) as common ancestor of the hidden-input
  scope; a SelectionPanel pane whose confirm slot is a Button child with
  `disable_on_submit: true` + `form` pairing; a TileGrid pane containing the
  Tile `$each` template element.
- **D-05:** The four register lint rules are the acceptance harness: the
  emitted spec must yield ZERO findings from `design::lint` for
  `register-fill-viewport`, `register-grid-fill`,
  `register-selection-present`, and `fill-viewport-layout-unknown`. An
  integration test asserts this — the projector must satisfy its own
  published lint bar.
- **D-06:** The Register arm emits `fill_viewport: true` AND a
  lint-supported shell layout (`"dashboard"`, matching the sample and the
  ferro-fill CSS chain which supports app/dashboard only). This is why
  `SpecBuilder.fill_viewport(bool)` must exist. Planner verifies the CSS
  class chain end-to-end (SC-3: correct `fill_viewport` class chain on the
  rendered HTML page).
- **D-07:** Numpad is NOT part of the v1 register template (ROADMAP names
  selection_pane + tiles_pane only; Numpad remains an author-composable
  builtin — 258 documents when to add it). TileGrid `search: true` is
  enabled by default in the register template (near-zero cost, big demo
  payoff). A categories/FilterTabs strip is emitted only if a category-ish
  source is cleanly derivable from the ServiceDef — omitting categories
  entirely in v1 is acceptable (planner's call; do not invent a hint).
- **D-08:** The confirm action derives from `ServiceDef.actions` (existing
  surface — same source `emit_datatable_root` uses for row actions). Exact
  selection rule (first action vs. named convention) is planner's call;
  it must be documented in rustdoc and covered by a test, including the
  no-actions case (error vs. omitted-confirm is planner's call, but silent
  broken output is not acceptable).

### ServiceDef → Tile mapping + per-row data contract
- **D-09:** The items collection binds at `/data/{service.name}` (the
  existing Browse convention from `emit_datatable_root`). Tile props inside
  the `$each` template are meaning-driven — `FieldMeaning::Identifier` →
  `item_id`, `EntityName` → `name`, `Money` → `price` (display) +
  `price_cents` (machine) — never hardcoded field names. Integer cents only,
  never float; `price`/`price_cents` agree from one authoritative source
  (256 D-04 rustdoc contract).
- **D-10:** The per-row hidden-input `field` name (`TileProps.field` is
  required) and the price display/cents split are satisfied via a
  DOCUMENTED per-row data contract: handler-supplied rows carry the keys
  the template binds (exactly how DataTable's `data_path` rows work today;
  the current cassa handler already synthesizes `field: "qty_{id}"`).
  Exact row-key convention is planner's call after research; NO new
  renderer interpolation surface may be introduced for this — if research
  finds an existing render-time mechanism that avoids the synthetic keys,
  using it is Claude's discretion.
- **D-11:** SC-1's "browse-intent products and collect-intent cart fields"
  means ONE ServiceDef carrying both the browsable items collection and the
  Collect signals (quantity/cart fields + confirm action). If
  `derive_intents` does not score Collect primary for the sample's
  ServiceDef, the sample uses the existing `IntentHint::Primary(Collect)` —
  no per-field intent system, no new hint variant, no derivation-signal
  changes. The `KNOWN_INTENTS` drift guard and seven-intent vocabulary are
  untouched (SC-1).

### Builder API additions
- **D-12:** `ElementBuilder.each(path, as_)` — public consuming setter
  (`mut self -> Self`, house builder convention) over the already-existing
  private `each: Option<EachDirective>` field. `NestedElement` stays
  directive-free (the spec.rs Phase-163 note deferred `.each()` "until a use
  case emerges" — the use case that emerged is ElementBuilder, not
  NestedElement).
- **D-13:** `SpecBuilder.fill_viewport(bool)` — consuming setter threaded
  through `build()` (which currently hardcodes `fill_viewport: false`).
  Default stays `false`; existing builder callers are unaffected.
- **D-14:** SC-4 test set: `$each` directive serde round-trip through the
  builder; `catalog_validate` accepting the directive on a products-pane
  element; an integration test covering `$each`-scoped `$data.*` path
  handling against `catalog_validate` (respecting `validate_directives`'s
  best-effort rule — the path-is-array check only fires when `spec.data` is
  non-null; the projector emits specs whose data is merged later by the
  handler, so tests must cover both the null-data and populated-data
  validation paths).

### /cassa flip
- **D-15:** `app/src/controllers/cassa.rs` builds the `ServiceDef` in Rust
  (Italian display names/copy live in app-land — allowed; `ferro-*` crates
  stay neutral), calls `derive_intents`, renders via `JsonUiRenderer` with a
  `VisualContext` carrying the register template (D-02 helper), and merges
  the products rows via the existing data-merge path.
  `app/src/views/cassa.json` is DELETED — no orphan spec file, no
  `JsonUi::render_file` call remains.
- **D-16:** The obsolete server-side `rimuovi` handler + route are deleted —
  line removal is client-side since 256 (`data-selection-remove` sets qty 0);
  a dead demo endpoint contradicts the composition the register now
  demonstrates. `conferma` stays as the confirm POST target (and remains a
  plain-redirect demo; the 255 D-18 idempotency-field demonstration
  discretion carries forward).
- **D-17:** SC-2 verification: `GET /cassa` returns a valid rendered HTML
  page — integration-test through the existing app test harness patterns;
  plus the D-05 lint-clean assertion on the derived spec. The RawHtml grep
  is recorded as already-passing pre-phase (world-state correction above).

### Gate
- **D-18:** CI-exact gate before every commit: `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, plus `cargo doc` clean. Re-run fmt after any
  hand-edit. Serialize CPU-heavy runs (one at a time). Schema-export churn:
  no wire-schema changes are expected (`$each`/`fill_viewport` are already
  in the schema) — discard regen churn unless a real diff appears.
- **D-19:** `docs/src` register/composition documentation is Phase 258 scope
  (per roadmap split); this phase's documentation obligation is rustdoc on
  every new public surface (`each`, `fill_viewport`, the template helper,
  `emit_register_root` contract incl. the D-10 data contract).

### Claude's Discretion
- Exact grid columns/spans/gap numbers for the register root; pane order.
- Exact register-template helper name/location and its `slots` list (D-02/D-03).
- Confirm-action selection rule from `ServiceDef.actions` + no-actions
  behavior (D-08, within its constraints).
- Per-row data-contract key names; use of an existing interpolation
  mechanism if research finds one (D-10).
- Whether the sample ServiceDef needs `IntentHint::Primary(Collect)` (D-11).
- Test organization (builder unit tests vs. projection integration tests vs.
  app-crate e2e).
- SelectionPanel display props passthrough (currency symbol etc.) — neutral
  defaults; sample-specific copy only where the ServiceDef surface allows.

### Folded Todos
None — `todo match-phase 257` returned 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition + requirements
- `.planning/ROADMAP.md` — v16.6 Phase 257 section (goal + SC 1–4) and the
  overview bullet naming the exact deliverables (`layout: "Register"` arm,
  `emit_register_root()`, `Spec::builder().fill_viewport(bool)`,
  `ElementBuilder.each(path, as_)`, `IntentSlotTemplate` Collect→Register,
  `/cassa` flip).
- `.planning/REQUIREMENTS.md` — POS-10 text; the Vocabulary decision
  (2026-07-05: "Register" retained as structural layout term).
- `.planning/PROJECT.md` — v16.6 milestone section ("no new parallel
  archetype surface" constraint).

### Prior phase contracts this phase builds on
- `.planning/phases/256-component-renderers-builtin-lockstep/256-CONTEXT.md`
  — D-01/D-02 (tile tap-to-add markup), D-04 (`price_cents`/
  `data-unit-price`), D-07..D-15 (SelectionPanel live view + Form-ancestor
  scoping D-11 — the composition emit_register_root must emit), D-16..D-20
  (TileGrid/FilterTabs), "Integration Points → Phase 257 consumes".
- `.planning/phases/255-pos-runtime-modules-double-submit-protection/255-CONTEXT.md`
  — final attribute vocabulary (V-01..V-05), D-16 (`disable_on_submit`),
  D-18 (idempotency discretion).
- `.planning/phases/254-props-contracts-touch-foundation-design-rules/254-CONTEXT.md`
  — props contracts; D-19 (`row_weights` semantics, available to the
  register grid if needed).
- `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md` — the
  gestiscilo picker audit (battle-tested spec for composition tiebreaks).

### Milestone research (2026-07-04 — read through the 255 rename map)
- `.planning/research/FEATURES.md` — register composition evidence.
- `.planning/research/PITFALLS.md` — integer-cents rule; fill-viewport
  pitfalls.
- `.planning/research/ARCHITECTURE.md` — integration-point anchors
  (pre-rename naming).

### Source anchors (current on this branch)
- `ferro-json-ui/src/projection/builder.rs` :241 `build_display_spec` (the
  layout match the Register arm extends; note Collect/Form currently
  short-circuits to `build_input_spec`), :292 `emit_datatable_root` (the
  meaning-driven mapping + `/data/{service}` convention to mirror), aux
  -elements pattern in `emit_card_root`/`emit_kanban_root`.
- `ferro-json-ui/src/projection/intent_layout.rs` — `default_template`
  (Collect→Form stays), `pick_intent_template` (the override channel D-01
  rides).
- `ferro-json-ui/src/projection/mod.rs` — `VisualContext`
  (`templates: Option<ThemeTemplates>`, `RenderMode`), `JsonUiRenderer`.
- `ferro-theme/src/template.rs` :10 — `IntentSlotTemplate` (open `layout:
  Option<String>` — no ferro-theme change needed).
- `ferro-json-ui/src/spec.rs` :359 `SpecBuilder` (build() hardcodes
  `fill_viewport: false` — D-13 target), :471 `ElementBuilder` (private
  `each` field — D-12 target), :545 NestedElement Phase-163 deferral note,
  :809 `validate_directives` ($each rules D-14 tests against).
- `ferro-json-ui/src/design/rules.rs` — the four register rules (D-05
  acceptance harness; `check_fill_viewport_layout_unknown` defines the
  app/dashboard layout constraint D-06 satisfies).
- `ferro-projections/src/service.rs` :63 `ServiceDef` (fields, actions,
  `intent_hints`); `ferro-projections/src/intent.rs` :78 `IntentHint`
  (Primary/Exclude — D-11).
- `app/src/controllers/cassa.rs` + `app/src/views/cassa.json` — the current
  hand-authored sample (composition reference AND the flip target; note the
  stale DataTable cart pane, world-state correction 2).
- `ferro-json-ui/src/catalog.rs` — `Catalog::validate` (the
  `catalog_validate` gate SC-1/SC-4 reference); BUILTIN count 52 (unchanged
  this phase).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `build_display_spec`'s layout match + `aux_elements` pattern — the Register
  arm is one more arm returning an `ElementBuilder` root plus aux elements;
  `emit_kanban_root`/`emit_card_root` show the multi-element emission idiom.
- `emit_datatable_root` — the meaning-driven field mapping
  (`lookup_meaning`, `is_system_field`, readable filtering) and
  `/data/{service.name}` data-path convention, plus action-derived row
  actions — all directly reusable for Tile prop mapping + confirm action.
- `pick_intent_template` + `VisualContext.templates` — the complete
  override plumbing already exists; D-01 needs zero new dispatch code
  upstream of the layout match.
- `Element.$each` + `validate_directives` — directive storage, serde
  (`rename = "$each"`), and all validation rules shipped in Phase 163; the
  ElementBuilder setter is a 5-line addition.
- The hand-authored `cassa.json` — a working, lint-conceived register spec
  to use as the target-output reference for `emit_register_root` (modulo
  the pre-256 cart pane).
- `design::lint` + the four register rules — a ready-made acceptance
  harness for projector output (D-05).
- 256's registered builtins (`TileGrid`, `SelectionPanel`, `Numpad`,
  `FilterTabs`, `QuantityStepper`, count 52) — `catalog_validate` accepts
  register compositions already.

### Established Patterns
- Consuming builder setters (`mut self -> Self`).
- Projection tests use `Spec::from_service_def_with_catalog`
  (injected-catalog pattern) to stay immune to OnceLock pollution — follow
  it for new projection tests.
- Meaning-driven mapping, never hardcoded field names, in every emit helper.
- CI-exact gate incl. `--all-features`; fmt after any hand-edit; serialize
  CPU-heavy runs.

### Integration Points
- `projection/builder.rs` — new match arm + `emit_register_root` +
  Spec::builder call site (needs `.fill_viewport(true)` + layout emission
  for the Register arm).
- `projection/intent_layout.rs` — D-02 helper (+ rustdoc table update).
- `spec.rs` — the two builder setters.
- `app/src/controllers/cassa.rs` (+ route table for `rimuovi` deletion) and
  deletion of `app/src/views/cassa.json`.
- Phase 258 consumes: the register template helper name, the composition
  shape, and the D-10 data contract for `generation_context` guidance.

</code_context>

<specifics>
## Specific Ideas

- **The killer feature is compressive:** one `ServiceDef` → a working
  tablet sale screen. `/cassa` flipping from an 89-line hand-authored spec
  to a projection call is the demo that proves the core abstraction covers
  a POS register. Spend the polish budget on `emit_register_root` output
  quality (it must pass the register lint rules the framework itself
  publishes) — the builder setters are commodity.
- Register is a LAYOUT template within Collect — the phase's conceptual
  claim is that the seven-intent vocabulary did NOT need to grow to cover a
  register. Nothing in the diff may weaken that claim.
- The register template override is also the first real exercise of the
  theme-template channel for a non-default layout — the "themes can
  override how any intent renders" architecture earns its keep here.

</specifics>

<deferred>
## Deferred Ideas

- **Numpad in the register template** — v1 template omits it; 258 documents
  manual composition; revisit on gestiscilo friction.
- **Category strip derivation hint** — if no clean category source is
  derivable (D-07), do NOT add a ServiceDef hint for it this phase; revisit
  with evidence.
- **Register template knobs** (pane ratios, pane order, search toggle as
  template parameters) — v1 is opinionated defaults; parameterize only on
  consumer friction.
- **Sibling FilterTabs↔TileGrid pairing** (`data-filter-for`) — still
  deferred from 256 D-18.
- **Per-line extra columns generic mechanism** — still deferred (256).
- **Barcode wedge, payment flow, receipts, shift close** — standing
  milestone deferrals.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.

</deferred>

---

*Phase: 257-projection-builder-register-layout-template*
*Context gathered: 2026-07-06*
