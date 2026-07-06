# Equivalence Record — Staff list (Browse)

**Status:** PARTIAL — data renders (Browse is data-bound); actions + image field + chrome do not. Branch `feat/208` UNMERGED.
**gestiscilo migration branch:** `feat/208-staff-projection-migration` @ `d4430ac5` (off master, unmerged)
**ferro intent test:** `cargo test -p ferro-projections --test catalog staff_browse_intent` — PASS
**Verified against:** ferro-rs 0.2.54, gestiscilo dev server, `/dashboard/staff`, tenant jetskiadriatic (id 3), 2026-06-12. Two probe staff rows inserted to test binding, then deleted (DB restored to 0 staff).

## Source (gestiscilo repo)

- Controller (before): `src/controllers/staff/list.rs`
- View JSON (before, deleted on branch): `src/views/staff/list.json`
- Backing model: `staff` table (tenant_id, slug, name, bio, avatar_url, sort_order, active, created_at)

## Functional Checklist (D-02: functional parity, not pixel-identity)

1. **Data fields shown:** **PARTIAL PASS** — the projection `DataTable` renders the entity's data, bound to `/data/staff`, with columns derived from the ServiceDef `fields`: NAME, BIO, SORT ORDER, ACTIVE (boolean formatted "Attivo"). The two probe rows (Mario Rossi / Lucia Bianchi) appeared correctly. **Missing:** the **avatar image** — `avatar_url` (FieldMeaning::ImageUrl) is not rendered as a DataTable column (system fields id/created_at are also excluded, which is correct). The bespoke view showed avatar thumbnails; the projection shows text columns only.
2. **Actions available:** **FAIL** — no per-row actions (View / Edit / Toggle active / Delete) render. ferro's `emit_actions_placeholder` is an intentional empty stub ("Deferred to Phase 118+"). The operator cannot act on rows from the migrated view.
3. **Primary-use-case flow:** **PARTIAL** — viewing the staff list (the read path) works; managing staff (the create/edit/delete affordances + the "Nuovo" CTA) is absent (no actions, no PageHeader action button).
4. **Intent confirmation:** **PASS** — `derive_intents(&service)[0].intent == Browse` (with `IntentHint::Primary(Browse)`; ferro test `staff_browse_intent` green).
5. **Intentional vs regression deltas:** Intentional (acceptable, projection composes differently): table layout instead of avatar cards. Regressions (block a real merge): no row actions, no avatar image, no create CTA, no dashboard chrome (sidebar/nav) — the projection emits a standalone spec.

## Evidence

- After (empty): `screenshots/after-staff-208-empty.png` — DataTable with a proper "Nessun elemento trovato" empty state (a real data-bound table, unlike the Orders placeholder).
- After (with rows): `screenshots/after-staff-208-rows.png` — NAME/BIO/SORT ORDER/ACTIVE columns + two bound rows.

## Abstraction gaps surfaced

### Positive result — Browse is the one data-bound intent

In contrast to Orders/Process, the Staff/Browse projection **genuinely renders the entity's data**: `emit_datatable_root` sets `data_path: /data/staff` and derives columns from the ServiceDef `fields` slot, so merging rows at `/data/staff` populates a real table. Intent classification → layout selection → column derivation → data binding all work end-to-end for Browse. This is the projection abstraction working as intended for a list view.

### Gap B (MEDIUM) — `actions` slot is an empty stub for every intent

`emit_actions_placeholder` renders nothing ("Intentionally empty. Deferred to Phase 118+"). Every migrated list/detail loses its per-row and page-level actions. For Staff this drops View/Edit/Toggle/Delete and the "Nuovo" CTA — so even the working Browse render is read-only and not a drop-in replacement for an operator-facing management page. Deferred ferro follow-up.

### Gap C (LOW-MEDIUM) — `ImageUrl` fields are not rendered as DataTable columns

`avatar_url` (FieldMeaning::ImageUrl) is declared on the ServiceDef but does not appear as a column; the DataTable column emit excludes it. Storage-backed image fields (the signed-avatar pattern, also RESEARCH §5 Gap 2) have no projection rendering in a Browse table. Deferred ferro follow-up.

## Assessment

Browse is the success case of the slice — and a real one: the data-bound `DataTable` renders the entity's rows and columns correctly from `ServiceDef` + `/data/{name}` merge. But it is **read-only and field-incomplete**: ferro's deferred `actions` slot strips all management affordances, and `ImageUrl` fields don't render. So even the best-case intent falls short of functional parity for an operator page. The pattern across all three entities is consistent: ferro's projection render is **layout-complete and (for Browse) data-bound, but content-incomplete** — actions everywhere, Process columns/cards, and Summarize values are deferred. Slice A's verdict is uniform: the abstraction is real and promising, and blocked on ferro maturing the builder's content binding, not on the migration technique.
