# Phase 257: Projection Builder — Register Layout Template - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-06
**Phase:** 257-projection-builder-register-layout-template
**Mode:** `--auto` (recommended option selected per area; all gray areas auto-selected)
**Areas discussed:** Register selection mechanism, emit_register_root composition, ServiceDef→Tile mapping + data contract, Builder API additions, /cassa flip shape

---

## Register selection mechanism (Collect→Register)

| Option | Description | Selected |
|--------|-------------|----------|
| Existing theme-template channel | `IntentSlotTemplate { layout: "Register" }` via `VisualContext.templates`; built-in Collect default stays Form; ship a `register_template()` helper. Zero new control surface; ferro-theme unchanged (`layout` is an open String). | ✓ |
| Change built-in Collect default on signals | Heuristically emit Register when products+cart signals detected. Breaks existing Collect→Form projections/tests; hidden magic. | |
| New ServiceDef layout hint | e.g. `LayoutHint::Register`. Duplicates the ThemeTemplates control surface (violates feedback_no_duplicate_control_surface). | |

**Notes:** The theme-template channel was designed exactly for "a theme can
override how any intent renders" — this is its first non-default-layout
production use.

---

## emit_register_root composition

| Option | Description | Selected |
|--------|-------------|----------|
| Mirror cassa.json + 256 D-11, lint rules as acceptance harness | Fill-viewport Grid (fill:true), ONE Form common ancestor, SelectionPanel + confirm Button (`disable_on_submit` + `form` pairing), TileGrid + Tile `$each`; emitted spec must pass all four register lint rules; layout "dashboard"; no Numpad in v1; search on by default. | ✓ |
| Minimal two-pane emission without lint conformance test | Faster, but the projector could emit specs failing its own published lint bar. | |
| Parameterized template (pane ratios/order knobs) | Speculative control surface with no consumer evidence — deferred. | |

---

## ServiceDef → Tile mapping + per-row data contract

| Option | Description | Selected |
|--------|-------------|----------|
| Meaning-driven mapping + documented per-row data contract | Identifier→item_id, EntityName→name, Money→price/price_cents at `/data/{service}`; rows carry the bound keys (incl. synthetic `field`, as the current handler already does). No new renderer surface. | ✓ |
| Renderer string-interpolation for per-row field names | New render-time surface (e.g. `"qty_{id}"` templating) — new mechanism for one consumer; rejected unless research finds it already exists (then Claude's discretion). | |
| Derive input name from item_id in the Tile renderer | Changes the 256 TileProps contract post-hoc. | |

**Notes:** SC-1's "browse-intent products + collect-intent cart fields" = one
ServiceDef; `IntentHint::Primary(Collect)` (existing surface) if derivation
scores otherwise. Seven intents + KNOWN_INTENTS untouched.

---

## Builder API additions

| Option | Description | Selected |
|--------|-------------|----------|
| `ElementBuilder.each()` + `SpecBuilder.fill_viewport()` only | Setter over the existing private `each` field; thread `fill_viewport` through `build()`. NestedElement stays directive-free (Phase-163 deferral note names ElementBuilder as the trigger that arrived). | ✓ |
| Also add NestedElement `.each()/.if_()` | No use case emerged for NestedElement directives; scope creep. | |

---

## /cassa flip shape

| Option | Description | Selected |
|--------|-------------|----------|
| ServiceDef in controller; delete cassa.json + rimuovi | Controller builds ServiceDef (Italian copy in app-land), `derive_intents`, `JsonUiRenderer` + register template, data merge; delete the spec file and the obsolete server-side remove endpoint (removal is client-side since 256). | ✓ |
| Keep cassa.json as fallback/reference | Orphan file contradicting "projection-derived"; old code is deleted completely (house rule). | |
| Keep rimuovi endpoint | Dead demo endpoint contradicting the 256 client-side removal model. | |

**Notes (world-state corrections):** `grep -rn RawHtml app/src/` already
returns zero hits — SC-2's grep gate passes pre-phase; the substantive work
is the derivation + file deletion. Current cassa.json cart pane is still the
pre-256 DataTable composition — the flip upgrades it to SelectionPanel.

---

## Claude's Discretion

Grid numbers (columns/spans/gap), template helper name/location + slots
semantics, confirm-action selection rule + no-actions behavior, per-row
data-contract key names, IntentHint usage in the sample, test organization,
SelectionPanel display-prop passthrough.

## Deferred Ideas

Numpad in the register template; category-strip derivation hint; register
template knobs (pane ratios/order/search toggle); sibling FilterTabs↔TileGrid
pairing; per-line extra columns; barcode wedge / payment / receipts / shift
close (standing).
