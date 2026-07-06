# Phase 209 — Publish Decision (D-06 / SC#4) + Branch Discipline (SC#1, SC#3)

## Publish path

**Path taken:** A (no publish).

**Rationale:** The slice made **zero ferro source changes**. The only ferro-repo edits are (a) the intent-assertion test fixtures in `ferro-projections/tests/catalog.rs` (test code, not API surface) and (b) the `.planning/` validation artifacts. No `ferro-*` library crate changed; no version bumped. Per D-06 the default expectation is zero ferro changes, and that held — the validation consumed the existing published surface and the gaps it found are routed to a *future* ferro phase, not a mid-slice fix (D-04/D-05).

**ferro version migrated against:** 0.2.54 (crates.io), for both attempted entities.

**Published version (Path B):** none — Path B (a forced minimal ferro fix + single end-of-slice publish) was not exercised. The discovered gaps (Process placeholder render, Summarize empty values, deferred actions) are substantial builder-maturation work, explicitly out of scope for this slice and deferred to a dedicated ferro phase.

## Branch discipline (SC#1, SC#3)

The strict one-per-merge discipline was **not exercised**, because **no entity reached merge-worthy functional parity** — the migrations surfaced blocking ferro gaps before any merge. Both branches are left UNMERGED by design:

- `feat/207-orders-projection-migration` @ `eddeaaf1` — Orders/Process. BLOCKED (Gap A: placeholder kanban). Not merged.
- `feat/208-staff-projection-migration` @ `d4430ac5` — Staff/Browse. PARTIAL (data renders; actions/image/chrome do not, Gaps B/C-area/D/E). Not merged.
- Statistics/Summarize — not migrated (Gap C predicted from source; not worth a branch until ferro binds StatCard values).

**SC#1 (delete render_file, merge per slice):** the render_file deletion was performed on each branch, but neither branch is merged to master — so production `master` is unchanged and still uses `render_file`. This is the correct outcome given the gaps.

**SC#3 (no branch > 2 weeks; no ferro API change while a branch is open):**
- No branch was alive more than a few hours (opened and assessed 2026-06-12).
- No ferro master API change was made while either branch was open (D-04 honored — ferro changes are deferred to a future phase).

## Net

Path A, no publish, no ferro source change, migrated against 0.2.54. The slice's value is the validation finding (WEAKNESS-NOTE.md), not a shipped migration. gestiscilo `master` is untouched; the two probe branches preserve the migration code for re-verification once ferro's projection builder is matured.
