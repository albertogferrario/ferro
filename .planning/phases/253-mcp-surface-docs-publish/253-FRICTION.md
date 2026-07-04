# Phase 253 — Dogfood friction report (pre-publish review pass)

Authored three new app views (`ordini`, `prodotti`, `prodotto_nuovo`) through the
design system as an authoring agent would: component schemas + intents in,
`design:lint` as the validation loop, browser verification after. This is the
same exercise gestiscilo Phase 232 will run at 68-spec scale. Findings below,
split by whether the design system caught the problem.

## Caught by the system (loop works)

| Finding | Layer | Outcome |
|---|---|---|
| `destructive-confirmation` flagged both new specs despite correct `action.confirm` | lint rule bug (checked entry-level `confirm`, a shape the renderer ignores; 252 fixture encoded the bug) | Fixed in this phase + regression test. The false-negative direction (entry-level confirm passing) would have shipped broken confirm dialogs. |

## NOT caught by the system (rule/API gap candidates)

| # | Gap | What happened | Candidate fix |
|---|---|---|---|
| 1 | Silent layout fallback | `layout: "dashboard"` was never registered in the app; registry silently fell back to the bare default layout — every page rendered with no shell and no padding. No diagnostic at lint, render, or runtime. | Runtime warn on unknown layout name; and/or surface registered layouts in `generation_context` so an authoring agent knows what exists. |
| 2 | `register_layout` missing from the `ferro::` facade | `DashboardLayout`'s own rustdoc example (`register_layout("dashboard", …)`) could not compile against the facade — apps had no public way to register layouts. | Fixed in this phase (facade re-export). |
| 3 | Form/Card width composition | `Form.max_width: narrow` inside a full-width Card centers the fields while the Card title/description stay left — misaligned. Lint-clean. | Authoring guidance: `max_width` belongs on the Card when the form is card-wrapped (fixed in the spec; documented pattern candidate for `docs/src/design-system/patterns.md` or a lint rule). |
| 4 | Card-wraps-PageHeader double title | `pagamenti.json` (Phase 243-era) used a root Card titled identically to its PageHeader child — duplicate heading. Lint-clean. | Cheap rule candidate: PageHeader nested inside a titled Card → warning. Fixed in the spec (Grid root). |
| 5 | Kanban card kebab is visually faint | `⋮` trigger renders `text-text-muted` at small size — present and consistent with DataTable, but easy to miss on white cards. | Polish candidate for the component quality bar; not changed (cross-screen consistency wins). |

Items 1, 3, 4, 5 are deliberately NOT folded into Phase 253 (publish phase; rule-set
changes belong to the friction loop). They should be reconciled with gestiscilo
Phase 232's FRICTION.md before the next design-system iteration.

## Cassa review (gestiscilo POS — read-only consumer audit at the gate)

The gestiscilo cassa (`views/cassa/orders_nuovo.json` + ~1500 lines of
hand-built HTML/CSS/JS in `controllers/helpers.rs`) is the strongest friction
datum available: the entire POS interaction — cart panel, product tap-grid,
category tab strip, search, cart runtime — is `RawHtml` escape hatches. The
design system could not express the page, so the reference consumer bypassed it.

Ferro-side responses shipped in this phase (pre-publish):
- **`Spec.fill_viewport` + Grid `fill` mode** — viewport-pinned workspace pages
  with internally-scrolling panes (the "cart and product buttons fit screen
  height" requirement). Demoed at the sample app's `/cassa`.
- **Grid `spans`** — asymmetric pane rows without RawHtml.
- DataTable's existing mobile card fallback covers the cassa cart's bespoke
  mobile-CSS reimplementation.

Remaining gaps for the next design-system iteration (POS component suite):
- Product tap-grid with category filter + search (ProductTile exists but lacks
  category tokens, staff pickers, cart synchronization).
- Client-side cart runtime (line totals, qty sync) as a runtime primitive.
- Fill-grid mobile row weighting (currently equal-height rows; a POS wants the
  product pane taller than the cart on phones).
- DataTable mobile card density option (current cards are touch-friendly but
  tall for compact carts).
- **Adoption risk:** gestiscilo specs carry retired enum values (e.g.
  `"variant": "default"` in `orders_nuovo.json`) that 0.2.85 REJECTS at spec
  parse — the Phase 232 sweep must apply the migration table before pinning.

## Theme change (operator feedback at the publish gate)

Default theme reworked per review: white `--color-surface`/`--color-card` and
near-black `--color-primary` in light mode; dark mode inverts the primary
(near-white button, dark text). `docs/src/features/themes.md` defaults table
synced. Ring stays accent-family blue.
