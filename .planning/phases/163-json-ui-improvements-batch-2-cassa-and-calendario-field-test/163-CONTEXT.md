# Phase 163: JSON-UI improvements batch 2 — iteration directives and spec construction ergonomics — Context

**Gathered:** 2026-05-16
**Status:** Ready for planning
**Source:** `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` — specifically the "Extended Iteration Gap" section (lines 272–338) and the iteration-related items in "Suggested ferro improvements (from blast radius analysis)" (lines 262–268).

## Phase Boundary

Phase 163 ships the **spec-level iteration directives** and **Rust-side spec construction ergonomics** identified by the gestiscilo migration. The input is the unified Phase 138 FRICTION.md — gestiscilo migrated phases 139–143 without producing separate friction files, so all migration friction is captured in one document. This phase reads the iteration-and-ergonomics slice; Phase 162 reads the components-and-API-surface slice; Phase 164 reads the closing-cleanup slice.

The original split (one ferro phase per gestiscilo phase) is dropped. The three phases now slice the same FRICTION.md by concern.

## Planning Note — Bidirectional Adaptation

See Phase 162 CONTEXT for the full statement. Summary: the friction loop is two-way — ferro evolves AND consumer UIs are allowed to be redesigned. The four heterogeneous-iteration sites in cassa surfaced from a v1 implementation pattern; before adding ferro complexity to satisfy them verbatim, the planner MUST evaluate whether the v1 UI was actually right. Status-dependent badges and conditional header actions are valid intent; the question is whether they require a `$if` directive or whether a per-status separate route would express the same user value more cleanly.

That said: the iteration gap is real even after redesign challenges. Three of the four cassa sites would benefit from `$each` regardless of UI reshaping (any data-driven list of custom-shape elements needs iteration). So Phase 163 ships `$each` for confident reasons; `$if` and `$template` get more scrutiny against the "could redesign solve this?" question.

## Slice from FRICTION.md

The relevant sections of FRICTION.md for Phase 163:

- "Extended Iteration Gap — Heterogeneous Iteration (added 2026-05-16 — phase 140 incidence)" — lines 272–338. Four heterogeneous-iteration sites in cassa/orders.rs and cassa/products.rs that forced `Spec::builder()` fallback. Three suggested directives: `$each`, `$if`, `$template`.
- "Codebase-Wide Blast Radius" `make_node` / `make_node_with_action` observation (lines 248–249) — ~12 controllers use these v1 builder helpers. The codemod proposal targets this.
- "Suggested ferro improvements (from blast radius analysis)" `SpecBuilder` ergonomic DSL bullet (line 267) — Rust-side construction layer for cases neither directives nor JSON specs cover.

## Implementation Decisions

### Iteration directives

- **D-01:** Ship `$each` as a spec-level directive on any element entry in `Spec.elements`. Syntax: `"element_id": { "type": "X", "$each": { "path": "/data_array_path", "as": "row" }, "props": { "key": { "$data": "/row/field" } } }`. At resolve time, ferro instantiates one element per row in the data array, with the loop variable bound to `$data` paths that start with `/row/`. Auto-suffixed IDs (`element_id-0`, `element_id-1`, …) prevent collisions. Children referenced by the templated element get the same auto-suffix applied.
- **D-02:** `$each` closes 3 of the 4 cassa heterogeneous-iteration sites identified in FRICTION.md (orders kanban list, new order ProductTile list, product detail magazzino_links_rows). The fourth (orders detail conditional actions) needs `$if` (D-03).
- **D-03:** Ship `$if` as a spec-level directive for **conditional emission** (different from the existing `visible` operator, which renders hidden DOM). Syntax: `"element_id": { "type": "X", "$if": { "path": "/flag", "operator": "equals", "value": true }, "props": { ... } }`. Elements whose `$if` evaluates falsy are NOT rendered at all (no hidden DOM, no JS). Missing IDs in `children` arrays that point to absent elements are silently skipped — this is required for `$if` to compose cleanly with parent `children`.
- **D-04:** Reuse the existing visibility expression evaluator (`visibility.rs`) for `$if` predicate evaluation. Do NOT add a parallel expression engine. The semantics of WHEN evaluation happens differ (resolve-time for `$if`, render-time for `visible`), but the expression syntax is shared.
- **D-05:** Do NOT ship a separate `$template` element. Reason: `$each` already templates by binding `row` and auto-suffixing IDs. A separate `$template` element type would be a parallel mechanism. Re-evaluate if a real use case appears that `$each` cannot express.

### Spec construction ergonomics

- **D-06:** Add `SpecBuilder` ergonomic methods that accept nested Rust types and emit the flat element map. Concretely: `SpecBuilder::element(id, type_name).child(...).child(...)` returns a builder whose `.build()` walks the nesting and emits the flat `elements` map with `children: Vec<String>` of IDs auto-generated by structural position. The existing low-level builder stays; this is a layer on top. This addresses cases where `$each` / `$if` cannot express the construction declaratively (e.g., truly runtime-shaped element graphs from complex domain state).
- **D-07:** `SpecBuilder` ergonomic layer is NOT a Component-style nested API. It is a flat-map-emitter with ergonomic syntax. The `Spec` runtime type stays canonical (flat map of `Element`); the nested DSL is sugar over construction only.
- **D-08:** Document the **decision rubric** in `docs/src/json-ui/spec-construction.md`: "Static spec → JSON file + `render_file`. Homogeneous iteration → JSON file + `$each`. Conditional emission → JSON file + `$if`. Heterogeneous runtime construction → Rust + `SpecBuilder`." Consumers pick by question, not by precedent.

### Migration codemod

- **D-09:** Ship `ferro json-ui:migrate-v1` as a `ferro-cli` subcommand. Input: a controller file using `make_node(id, Component::X(props))` patterns. Output: a stub JSON spec file under `src/views/{module}/{controller}.json` with the flat elements map pre-generated, plus a rewritten controller using `JsonUi::render_file(...)`. The codemod is best-effort: cases involving runtime branching or `Spec::builder()` fall-throughs get a `// TODO: codemod could not auto-translate` marker, not a silent skip.
- **D-10:** Codemod operates on a single file per invocation (`ferro json-ui:migrate-v1 src/controllers/X.rs`). No directory-recursive mode in this phase. Reason: each migration needs human review; batch mode invites silent regressions.
- **D-11:** Codemod gates: AST-based (use `syn`), not regex-based. Idempotent (running on an already-migrated file is a no-op + warning). Dry-run flag (`--dry-run`) prints the proposed JSON spec and controller rewrite without writing.

### Catalog and validation

- **D-12:** Spec validator (the one Phase 162 D-07/D-08 enhances) MUST emit a clear error when a `$each` `path` resolves to a non-array `$data` value, when an `$if` `path` resolves to a missing key (vs falsy), and when `$each.as` collides with a reserved name. These errors land at `Spec::validate` time, not at render time.
- **D-13:** Both `$each` and `$if` MUST be reflected in `ferro-mcp`'s `json_ui_catalog` tool output, so agents authoring specs see the directives in the catalog surface.

### Claude's discretion

- Exact JSON wire syntax (`$each` vs `each` vs `__each__` etc.) — `$`-prefix is the established convention from `$data` / `$expr`; planner confirms.
- Whether `$each` produces a `Vec<Element>` slot in the parent's `children` field, or whether it spawns the templated element AND each clone as separate entries in `Spec.elements` with the parent's `children` rewritten — implementation detail.
- Whether the codemod ships with a hand-curated list of migration patterns or generates them from `ferro-mcp`'s `code_templates` — implementation choice.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Friction source
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/138-json-ui-v2-migration-auth-account-onboarding-pages/FRICTION.md` — lines 272–338 and lines 248–249, 267 specifically. Plans MUST cite the exact line range when justifying a directive's shape.

### Spec resolution pipeline (where `$each` / `$if` plug in)
- `ferro-json-ui/src/spec.rs` — `Spec::from_json`, `Spec::validate`, `Element` definition. D-01/D-03/D-12 land here.
- `ferro-json-ui/src/resolve.rs` — resolver passes that walk `Spec.elements`. The new directive resolution happens here before render.
- `ferro-json-ui/src/visibility.rs` — existing visibility expression evaluator. D-04 reuses this for `$if`.
- `ferro-json-ui/src/render.rs` — element rendering. After resolve, `$each`-expanded elements are normal elements; `$if`-rejected elements have been deleted from the map; render is unchanged.

### Cli codemod target
- `ferro-cli/src/commands/` — new `json_ui_migrate_v1.rs` subcommand lives here. D-09/D-10/D-11 land here.

### Documentation
- `docs/src/json-ui/spec-construction.md` — new file. D-08 worked examples.
- `docs/src/json-ui/expressions.md` — extend with `$each` / `$if` sections.

## Predecessor and successor

- Phase 162 lands first — it owns the catalog surface decisions and the spec validator framework that Phase 163's `$each` / `$if` validation plugs into.
- Phase 163 does NOT depend on Phase 162 completing for design (decisions above are locked) but DOES depend on Phase 162 having merged the spec validator framework before Phase 163's plan 02 (validation extensions) executes.
- Phase 164 (closing cleanup) follows Phase 163.

## Release cadence

Same as Phase 162 D-23/D-24/D-25 — no mid-loop publish. Phase 163 CHANGELOG entries accumulate; the single v12.0 publish happens at Phase 161.
