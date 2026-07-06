# Phase 209 — What the Migration Revealed (SC#5)

> First real-world validation of the projection/intent abstraction against a production
> codebase (gestiscilo). Target: three entities — Staff (Browse), Orders (Process),
> Statistics (Summarize). Orders and Staff were migrated on branches and observed against a
> running server; Statistics was assessed from confirmed ferro source. This note records the
> abstraction gaps surfaced. An empty note fails the phase (SC#5); finding nothing would be a
> red flag, not a success. This slice found a great deal.

## Headline finding

ferro's projection render (0.2.54) is **layout-complete but content-incomplete**. The pipeline that matters — `derive_intents` → intent-appropriate layout selection → `JsonUiRenderer` — works: each entity derives the right intent and selects the right outer layout (Browse→DataTable, Process→KanbanBoard, Summarize→StatCard). But the **content binding** behind those layouts is partial, and several slots are explicitly deferred placeholders in ferro's own code. Only the Browse/DataTable path is genuinely data-bound. The abstraction is real and structurally sound; it is **not yet content-complete for production migration**, and the gaps are concrete, named, and already on ferro's backlog ("Phase 118+", "deferred idea").

The migrations used the projection API correctly. None of the gaps below are migration mistakes — each was traced to a specific ferro function.

## Abstraction gaps surfaced

### Gap A: Process render is a placeholder kanban — not state-machine-aware, not data-bound (MAJOR, blocking)

- **Entity / intent:** Orders kanban / Process. Branch `feat/207` @ `eddeaaf1` (unmerged). Evidence: `screenshots/after-orders-207.png`.
- **What happened:** `/dashboard/cassa/ordini` rendered a single bordered card "Order / 0" — no columns, no order cards.
- **Why it is structural:** `ferro-json-ui/src/projection/builder.rs::emit_kanban_root` hard-codes one `KanbanColumnProps { title: display_name /*"Order"*/, count: 0 }` and `KanbanBoardProps { data_path: None }`. Its doc comment: *"Full state-machine awareness is a deferred idea ... for now we emit a single placeholder column carrying the service's display name."* The Process render is not state-machine-aware (no columns derived from the `ServiceDef` state machine) and not data-bound.
- **Workaround applied (D-05):** none viable — the missing kanban is exactly what the projection was meant to derive.
- **Deferred ferro follow-up:** state-machine→column derivation + card data binding in `emit_kanban_root`.

### Gap B: the `actions` slot is an empty stub for every intent (MEDIUM, pervasive)

- **Entity / intent:** all (observed on Staff/Browse). Evidence: `screenshots/after-staff-208-rows.png` (no row actions).
- **What happened:** no per-row or page-level actions (View/Edit/Delete, status transitions, "Nuovo" CTA) render in any migrated view.
- **Why it is structural:** `builder.rs::emit_actions_placeholder` is *"Intentionally empty. Deferred to Phase 118+."* Every migrated page is therefore read-only.
- **Deferred ferro follow-up:** action wiring (Button/DropdownMenu elements from `ServiceDef` actions).

### Gap C: Summarize render has empty stat values (MEDIUM)

- **Entity / intent:** Statistics dashboard / Summarize (assessed from source — not migrated live).
- **What happened (predicted, source-confirmed):** `builder.rs::emit_statcard_root` constructs `StatCardProps { value: String::new() }` — the stat value is empty and not data-bound. A Summarize migration would render stat-card labels with no numbers (and the SVG chart has no FieldMeaning — the forecast Gap 1).
- **Deferred ferro follow-up:** StatCard value binding to runtime data.

### Gap D: `ImageUrl` fields don't render in a Browse DataTable (LOW-MEDIUM)

- **Entity / intent:** Staff/Browse. `avatar_url` (FieldMeaning::ImageUrl) is declared but not emitted as a DataTable column, so avatars are lost. Storage-backed/computed image fields (also forecast Gap 2 — signed URLs) have no projection rendering. Deferred ferro follow-up.

### Gap E: the projection emits a standalone spec — dashboard layout chrome is not reproduced (LOW-MEDIUM)

- The projection render replaces the whole page; the bespoke handlers wrapped views in a dashboard shell (sidebar/nav). There is no layout-context mechanism for the projection to render inside an app shell. Deferred ferro follow-up (a `VisualContext`/layout-slot concept).

## What worked (the positive signal)

**Browse is data-bound end-to-end.** `emit_datatable_root` sets `data_path: /data/{service.name}` and derives columns from the `fields` slot; merging rows at `/data/staff` produced a correct table (NAME/BIO/SORT ORDER/ACTIVE, two rows, boolean formatted "Attivo"). Intent classification → layout → column derivation → data binding all work for a list view. This is the abstraction delivering on its promise for one of the seven intents — concrete proof the design is sound, not just plausible.

## Weak-signal findings (non-blocking observations)

- **Staff/Browse required an explicit `IntentHint::Primary(Browse)`** — `bio` (FreeText) + `avatar_url` (ImageUrl) pull the structural signal toward Focus, so Browse is not derived without the hint. The structural signal alone misclassifies this common entity shape (RESEARCH Risk 1, confirmed).
- **Data-path contract is implicit:** rows must be merged at exactly `/data/{service.name}` to bind. A migrator without source access to `emit_datatable_root` would not know this; it is undocumented in the public API surface.

## Assessment (compressive-dimension validation signal)

The projection/intent abstraction holds up against a real codebase at the level it claims to operate — intent derivation and layout selection are correct across Browse, Process, and Summarize. The gap is one level down: **content binding**. Browse binds data; Process, Summarize, and actions (everywhere) are ferro-documented deferred placeholders. So the honest v1.0-criterion-#2 reading is: the abstraction is real and the Browse path is production-shaped, but **real-world migration of Process and Summarize views — and any view needing actions or image fields or app-shell layout — is blocked until ferro matures the projection builder's content binding.** That maturation is the concrete, prioritized follow-up this validation produced; it is the difference between "the abstraction classifies correctly" and "the abstraction renders a usable app." Slice A did exactly what COMP-01 was for: it converted a plausible abstraction into a measured one with a specific, ordered gap list.

## Deferred ferro follow-up (NOT done in this slice — D-04/D-05)

A ferro projection-builder maturation phase: (1) state-machine→kanban column derivation + card data binding; (2) StatCard value binding; (3) action-slot wiring; (4) ImageUrl column rendering; (5) a layout/app-shell context so projections render inside chrome. See the drafted ferro phase in ROADMAP.md.
