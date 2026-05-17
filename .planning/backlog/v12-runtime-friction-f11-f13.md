# v12.0 runtime friction — F11–F13 (from gestiscilo, 2026-05-17)

Surfaced during a full dashboard re-walk of gestiscilo against patched ferro `v12.0/json-ui-v2` HEAD `ced0e714` (Phase 164 COMPLETED). The F1–F10 frictions from `gestiscilo/.planning/V7-RUNTIME-FRICTION.md` are all confirmed resolved by Phases 162–164. Three new frictions emerged once the previous blockers were out of the way; all three are fixable on the consumer side and have been (gestiscilo commit `47ff336`). Two carry low-cost ferro recommendations.

Full consumer-side analysis: `gestiscilo/.planning/V7-RUNTIME-FRICTION-RESOLVED.md`.

## F11 — `PageHeader.children` silently dropped at render

**Severity.** High. Silent rendering pathology — pages return 200 with chrome but no body.

**Symptom.** 7 dashboard pages plus all detail/edit pages rendered only the title + actions row. Tables, Cards, Forms, EmptyStates declared as `PageHeader.children` never reached the DOM. No log line, no console message, no validation error. Pages looked "almost right" — the kind of bug that takes longer to root-cause than a hard failure.

**Affected gestiscilo surface (before fix).** 24 JSON specs + 4 Rust builder call sites in `controllers/cassa/{orders.rs,products.rs}`. Pattern across all of them: `root` element is a `PageHeader` with `children: [body_1, body_2, ...]`. In v2 `PageHeader` is chrome-only — it renders title/actions/breadcrumb but never walks its `children`.

**Gestiscilo fix (already applied).** Mechanical Grid-wrap: rename the PageHeader root to `page_header`, introduce a new root `Grid { columns: 1, gap: "md" }` whose children are `[page_header, ...original children]`.

**Recommended ferro action (either, not both).**

1. **Render `PageHeader.children` after the chrome block.** The author already named the children; honoring them matches the intent. Estimated change: a few lines in `render_page_header` plus the dispatcher walk.
2. **Hard-fail at spec validation** when `PageHeader.children` is non-empty, with a message pointing at the Grid-wrap pattern:
   ```
   PageHeader is chrome-only and does not render children. Wrap content in a Grid:
   { root: "page", elements: { page: { type: "Grid", children: ["page_header", ...] } } }
   ```

Option 1 is more author-friendly; option 2 is more architecturally honest about PageHeader's role. Either prevents the next consumer hitting this surface. Silent-drop is unacceptable for any framework primitive — both options remove the silence.

## F12 — `Visibility.operator: "equals"` rejected (use `"eq"`)

**Severity.** Low (pure consumer authoring bug).

**Symptom.** Hard 500 on `/dashboard/clienti` and `/dashboard/inventario/flotta`:
```
failed to parse JSON: unknown variant `equals`, expected one of
`exists`, `not_exists`, `eq`, `not_eq`, `gt`, `lt`, `gte`, `lte`,
`contains`, `not_empty`, `empty` at line 47 column 5
```

Full grep showed 11 specs using `"equals"`. All fixed consumer-side (sed replace).

**No ferro action required.** Phase 164 D-19/F5 hand-rolled deserializer is doing exactly its job — the error message named the offending variant and listed accepted forms, which is what made the issue trivially fixable. Calling it out here just to acknowledge the deserializer paid for itself on its first runtime surface.

## F13 — `Visibility.operator: "empty"` semantics with boolean paths

**Severity.** Low–Medium. Documented behavior but counterintuitive in the dominant consumer pattern.

**Symptom.** Consumer pattern is `controller passes has_items: bool` paired with two visibility gates:
```json
"table":       { "visible": { "path": "/has_items", "operator": "eq", "value": true } }
"empty_state": { "visible": { "path": "/has_items", "operator": "empty" } }
```
When `has_items: false`, both gates evaluate falsy: table hides correctly, but the empty-state element also stays hidden because `Empty` is `false` for `serde_json::Value::Bool(_)` (per `ferro-json-ui/src/visibility.rs:165–171`). Page renders blank where the empty state should appear.

**Cause.** `Empty`/`NotEmpty` are documented to treat numbers and booleans as non-empty (the boolean carve-out comment is on the resolver). Author intent for `has_X: bool` paths reads as "empty = falsy" by intuition, not "empty = empty-string/array/object/null".

**Gestiscilo fix.** 4 specs rewritten to `operator: "eq", "value": false` on boolean paths. The other 3 `empty` usages were correct (string `""` / `Option<String>` paths).

**Recommended ferro action (light, optional).**

- **Docs:** surface the boolean carve-out in `docs/src/json-ui/visibility.md` as a callout, not just a source comment. The two-line note: *"`empty`/`not_empty` evaluate booleans and numbers as non-empty. For `has_X: bool` paths use `eq, value: false` instead."*
- **Optional ergonomics:** add `is_true` / `is_false` operators that match Rust `bool` directly. Lowest priority — `eq, value: false` already works.

## F14 — `Action.handler` is `String` literal-only; `$data` / `$template` bindings rejected at parse time

**Severity.** Medium. Blocks two important v2 patterns surfaced when comparing gestiscilo prod (v1) vs local (v2): per-row navigation in `$each` templates, and data-driven `KanbanColumn.children` rendering via `JsonUi::render_file`.

**Symptom A — `$each` + per-cell action URL.** Gestiscilo migrated `calendar_month.json` from `DataTable` to a `Grid(7)` of `CalendarCell` elements via `$each` over `/calendar_days`. Each cell needs its own day URL:

```json
"day_cell": {
  "type": "CalendarCell",
  "$each": { "path": "/calendar_days", "as": "cell" },
  "props": { "day": { "$data": "/cell/day" }, ... },
  "action": {
    "handler": { "$data": "/cell/action_url" },
    "method": "GET"
  }
}
```

Result: `Failed to load spec: invalid type: map, expected a string at line 57 column 19`. The parser hits `handler: {map}` and rejects because `Action.handler` is declared `pub handler: String` (`ferro-json-ui/src/action.rs:78`).

**Symptom B — `KanbanBoard.data_path` with per-card navigation.** Wired `kanban_orders.props.data_path` to `/kanban_columns` (Phase 164 D-13a). Column headers render with correct counts ("Confermati (8)"), but column bodies are empty. `KanbanColumnProps.children: Vec<String>` expects pre-registered element IDs; with `JsonUi::render_file` (not `Spec::builder`), the only way to populate cards is `$each` per column — which loops back to Symptom A for the per-card click action.

**Cause.** `Action.handler: String` is parsed eagerly during `Spec::from_json`, before `$each` expansion and `resolve_actions`. The migration guide implies per-row `$data` on action URLs works ("controller pre-resolves URLs into spec.data and the templated element references them via `{ "$data": "/order/advance_url" }`"), but the wire-format parser doesn't accept maps where strings are required.

**Gestiscilo workaround (applied).** Reverted both pages to `DataTable` (per-row navigation via `row_href` interpolation is the one mechanism that works with `data_path` today). `dashboard/index.json` kanban placeholder restored. F14 documented in `gestiscilo/.planning/V7-RUNTIME-FRICTION-RESOLVED.md`.

**Recommended ferro action.** Make `Action.handler` accept either a literal `String` or a `{$data|$template: ...}` binding, resolved during `resolve_actions`. The renderer already runs `resolve_actions` for URL synthesis — extending it to walk a handler-side binding parallels the existing per-prop resolution. Alternative: document explicitly that per-row navigation in `$each` requires `Spec::builder` (not `JsonUi::render_file`) and remove the misleading migration-guide example.

This is a real blocker for two v2-native consumer patterns. Without it, any spec authored as a JSON file is restricted to literal action URLs, forcing consumers into `Spec::builder` for any list/table/grid with per-row navigation.

## Non-blocking

None of F11, F13, F14 blocks Phase 160 (v1 deletion) or Phase 161 (merge + publish). They are inputs for the next round of v12.x polish if the friction loop gets a follow-on phase.
