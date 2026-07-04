# gestiscilo Phase 232 — Handoff Brief

Ferro v16.5 JSON-UI Design System has been published. This brief covers
everything gestiscilo Phase 232 needs to adopt it.

## Pin

```toml
ferro = { package = "ferro-rs", version = "0.2.85" }
```

`ferro-json-ui`, `ferro-theme`, and `ferro-mcp` ship at the same workspace
version (0.2.85). No separate pins needed.

`ferro-payments` is independently versioned at `0.1.5` (unchanged from the
prior pin; no updates in this release cycle affecting gestiscilo).

## Breaking changes — apply before pinning

**Canonical variant/tone/size enums shipped in 0.2.85.** Retired enum values
are rejected at spec-parse time (serde unknown-variant error). The gestiscilo
spec tree carries at least one known violation:

- `views/cassa/orders_nuovo.json` — `btn_submit` uses `"variant": "default"`.
  Serde will reject this component at parse; the page will 500.

Apply the migration table in `docs/src/json-ui/components.md` (section
"Canonical Enums — Migration Table") across all 68 specs before pinning. The
table lists every retired value and its canonical replacement.

Quick grep to find violations before the pin:

```bash
grep -r '"variant": "default"\|"variant": "link"\|"variant": "info"\|"variant": "error"' app/src/views/
grep -r '"size": "xs"' app/src/views/
grep -r '"tone": "info"\|"tone": "error"' app/src/views/
```

## Default theme change

The default theme now uses white surfaces and cards with a monochrome primary.
Tenant theme overrides (gestiscilo's `custom.css`) are unaffected. If the app
uses no explicit theme override and relies on the prior teal/colored defaults,
expect a visual diff on components that were not customized.

## New capabilities relevant to gestiscilo FRICTION items

### Viewport-pinned workspaces (`Spec.fill_viewport` + `Grid fill`)

Direct fix for the cassa 200px-cart / page-scroll problem. The reference recipe
is at `app/src/views/cassa.json` in the ferro workspace.

Pattern:

```json
{
  "spec": {
    "fill_viewport": true,
    "layout": "dashboard",
    "components": [{
      "type": "Grid",
      "fill": true,
      "columns": 2,
      "children": [
        { "...": "left pane — scrolls internally" },
        { "...": "right pane — scrolls internally" }
      ]
    }]
  }
}
```

`fill_viewport: true` makes the Spec container stretch to the viewport height
and disables body scroll. `Grid fill: true` makes the grid fill its parent and
makes each column an `overflow-y: auto` scroll container.

### Grid `spans`

Asymmetric column widths without RawHtml. Set `spans` on Grid children to
assign each child a fractional width (values are `fr` units):

```json
{
  "type": "Grid",
  "columns": 3,
  "children": [
    { "span": 2, "...": "two-thirds column" },
    { "span": 1, "...": "one-third column" }
  ]
}
```

### Bounded content column (`max_width` on layouts)

Dashboard-family layouts now constrain the content column to `max-w-7xl`
(~80rem) and left-anchor the column when the viewport is narrower. This matches
the page-header edge alignment. No per-spec change needed; the layout applies it
automatically.

### `register_layout` via the `ferro::` facade

The `register_layout` function is now re-exported from the `ferro::` facade.
Prior consumers calling `ferro_json_ui::register_layout(...)` or
`ferro_inertia::register_layout(...)` can migrate to `ferro::register_layout(...)`.

## Design lint

### `design_lint` MCP tool

Available on the running app's `/mcp` endpoint after the pin. Input:

- `spec_json: String` — inline JSON to lint, **or**
- `path: String` — path to a `.json` spec file

Output: `FileFinding[]` — each entry carries `file` plus the flattened finding
fields `rule`, `element_id`, `severity` (`warning | info` — there is no error
severity; lint is diagnostics-only), `message`, and `suggestion`. Identical to
the CLI `--json` shape.

### CLI gate

```bash
# Report only (always exits 0)
ferro design:lint app/src/views

# CI mode: non-zero exit when any warning-level finding exists (info never fails)
ferro design:lint app/src/views --deny

# JSON output for CI integration
ferro design:lint app/src/views --json
```

Add `ferro design:lint app/src/views --deny` to the CI pipeline after the
Phase 232 sweep is lint-clean. This prevents regressions.

### `prefer-components` Info rule

Every `RawHtml` component in a spec surfaces as an Info finding. Info severity
never fails `--deny`. The gestiscilo sweep will report approximately 8 RawHtml
sections (the shared product-picker across `cassa` and `calendario` pages).

Expected workflow: run `ferro design:lint --json`, review the `prefer-components`
findings, and either:

1. Replace with catalog components (preferred where the catalog covers the case), or
2. Add `"allow": ["prefer-components"]` to the Spec's `design` block to
   acknowledge the escape hatch.

The `prefer-components` rule is the signal source for which picker surfaces
need promotion into the catalog (POS component suite — next design-system
iteration).

### `generation_context` design-system summary

The `generation_context` MCP tool now includes a `design_system` section
with:

- `tokens` — the 30-slot token vocabulary with a one-line purpose each.
- `intent_patterns` — design rules grouped per projection intent (plus an
  `all` bucket), each with id, title, and rationale.
- `canonical_variants` — the canonical `variant`/`tone`/`size` value lists.
- `docs` — pointer to the `docs/src/design-system/` chapter.

An authoring agent should read `generation_context` before writing specs to
get the canonical vocabulary and avoid retired values.

## FRICTION.md request

Phase 232 should produce a FRICTION.md at the conclusion of the sweep, covering:

- Which retired values were found and fixed.
- Which `prefer-components` findings were accepted vs. suppressed.
- Any design-system gaps discovered during the sweep (new rows for the
  shared friction backlog).
- Visual diff observations after the theme change.

The FRICTION.md format follows the Phase 253 friction report
(`.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md`) as the
reference template.

## Reference files (ferro workspace)

| Purpose | Path |
|---|---|
| Migration table (enum retired values) | `docs/src/json-ui/components.md` |
| Canonical enums section | `docs/src/json-ui/components.md` |
| Spec.fill_viewport + Grid fill recipe | `app/src/views/cassa.json` |
| Grid spans recipe | `app/src/views/prodotto_nuovo.json` (2/3–1/3 split) |
| Design-system docs chapter | `docs/src/design-system/` |
| Action patterns (forms, confirm) | `docs/src/json-ui/actions.md` |
