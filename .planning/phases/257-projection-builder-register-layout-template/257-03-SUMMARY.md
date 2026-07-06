---
phase: 257-projection-builder-register-layout-template
plan: "03"
subsystem: app
tags: [projection, register-layout, pos, controller-flip, integration-test]
dependency_graph:
  requires: [register_template, emit_register_root, JsonUiRenderer]
  provides: [cassa_service_def, cassa_products, cassa_render_test]
  affects:
    - app/src/controllers/cassa.rs
    - app/src/routes.rs
    - app/src/views/cassa.json (deleted)
    - app/src/tests/cassa_render.rs
    - app/src/tests/mod.rs
    - framework/src/lib.rs
tech_stack:
  added: []
  patterns: [projection-derived-controller, meaning-driven-field-mapping, fill-viewport-body-class]
key_files:
  created:
    - app/src/tests/cassa_render.rs
  modified:
    - app/src/controllers/cassa.rs
    - app/src/routes.rs
    - app/src/tests/mod.rs
    - framework/src/lib.rs
  deleted:
    - app/src/views/cassa.json
decisions:
  - "register_template + default_template added to ferro facade re-exports so app controllers can access without dev-dependency direct import"
  - "cassa_service_def() and cassa_products() extracted as pub helpers for test reuse without duplicating the derivation"
  - "lint filter uses f.rule directly (&&'static str) — .as_str() would hit unstable str::as_str on stable toolchain"
metrics:
  duration_minutes: 25
  completed_date: "2026-07-06"
  tasks_completed: 2
  files_modified: 6
requirements: [POS-10]
---

# Phase 257 Plan 03: App Controller Flip — Projection-Derived /cassa Summary

The `/cassa` sample endpoint now serves a projection-derived Register spec from a `ServiceDef` — the phase's killer feature proven end-to-end: one `ServiceDef` declaration derives the full tablet sale screen via `register_template()`, with no hand-authored JSON spec.

## What Was Built

**`cassa_service_def()` + `cassa_products()` — pub helpers in cassa.rs (D-15).** Two extracted `pub fn` helpers replace the monolithic 60-line handler. `cassa_service_def()` names the ServiceDef `"cassa"` so the derived confirm-action path (`/cassa/conferma`) hits the existing named route. `cassa_products()` synthesizes 24 product rows with the register data contract: `id`, `nome`, `prezzo` (display), `price_cents` (integer cents), `field` (hidden-input name).

**Projection-derived `index` handler (D-15).** `JsonUiRenderer.render(&service, &intents, &ctx)` with `VisualContext { templates: Some(register_template()), ..Default::default() }` replaces `JsonUi::render_file(...)`. The rendered page carries `fill_viewport=true` → `ferro-fill` body class chain (SC-3).

**`cassa.json` deleted + `rimuovi` handler/route removed (D-16).** `git rm app/src/views/cassa.json` eliminates the orphaned hand-authored spec; the `rimuovi` handler and its `POST /cassa/rimuovi/:id` route are deleted entirely (dead since Phase 256 client-side removal).

**`register_template` + `default_template` added to ferro facade (Rule 3 fix).** `ferro_json_ui` is a dev-dependency in the app crate — not a regular dependency. `JsonUiRenderer`/`VisualContext` were already re-exported from `ferro::`, but `register_template` was missing. Added both `register_template` and `default_template` to the `#[cfg(feature = "projections")]` block in `framework/src/lib.rs`.

**`cassa_render` integration test (SC-2/SC-3, D-17).** One test — `cassa_render_is_projection_derived_fill_viewport` — proves SC-2 and SC-3:
- `spec.fill_viewport == true` and `spec.layout == Some("dashboard")`
- Zero lint findings for all four register rules (`register-fill-viewport`, `register-grid-fill`, `register-selection-present`, `fill-viewport-layout-unknown`)
- `JsonUi::render` returns 200
- HTML contains `ferro-fill` (SC-3)
- HTML contains `data-selection-panel`, `data-filter-search`, `"Conferma ordine"` (register composition markers from Phase 256 render contracts)

## Tasks

| Task | Commit | Result |
|------|--------|--------|
| 1: Flip cassa.rs + delete cassa.json + rimuovi | bc0c3563 | Build clean; acceptance criteria all pass |
| 2: cassa_render integration test | 057c4b34 | 1/1 test green; full CI-exact gate green |

## Deviations from Plan

**1. [Rule 3 - Blocking] `register_template` not accessible from regular app code**

- **Found during:** Task 1 build
- **Issue:** `ferro-json-ui` is a `[dev-dependencies]` entry in `app/Cargo.toml`, so `use ferro_json_ui::register_template` compiles only in test code. The controller (regular source) cannot reference it directly.
- **Fix:** Added `register_template` and `default_template` to the `#[cfg(feature = "projections")]` re-export block in `framework/src/lib.rs` alongside the existing `JsonUiRenderer`/`VisualContext` exports. App controller now uses `ferro::register_template`.
- **Files modified:** `framework/src/lib.rs`
- **Commit:** bc0c3563

**2. [Rule 1 - Bug] `format!("…{e}")` needed instead of literal string in `error_response!`**

- **Found during:** Task 1 build (compiler warning: unused variable `e`)
- **Issue:** `ferro::error_response!(500, "cassa projection failed: {e}")` does not interpolate — the macro takes `$msg:expr` and calls `.to_string()` on it; string literal `{e}` is not interpolated by the macro.
- **Fix:** Changed to `ferro::error_response!(500, format!("cassa projection failed: {e}"))`.
- **Files modified:** `app/src/controllers/cassa.rs`
- **Commit:** bc0c3563

**3. [Rule 1 - Bug] `f.rule.as_str()` hit unstable `str::as_str` on stable toolchain**

- **Found during:** Task 2 test compilation
- **Issue:** `Finding.rule` is `&'static str`; calling `.as_str()` on a `&str` invokes the unstable `str::as_str()` feature (issue #130366).
- **Fix:** Removed `.as_str()` — `register_rules.contains(&f.rule)` compares `&&str` elements directly.
- **Files modified:** `app/src/tests/cassa_render.rs`
- **Commit:** 057c4b34

## CI Gate

Full CI-exact gate green (last plan of phase):
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets --all-features -- -D warnings` — clean (46s)
- `cargo test --all-features` — all test suites pass, exit 0
- `cargo doc --no-deps` — clean
- Schema export: no churn (docs/protocol/schemas/ unchanged)

## Known Stubs

None. The controller derives its spec from a real `ServiceDef` through `JsonUiRenderer` — no hardcoded JSON, no RawHtml, no render_file.

## Threat Surface Scan

No new network endpoints or auth paths. The `/cassa/rimuovi/:id` route is REMOVED (attack surface reduced). The `/cassa` GET handler now derives its HTML through the projection render pipeline (escaping intact); no raw string interpolation of product row values into markup (T-257-07 mitigated). `grep -rn RawHtml app/src/controllers/cassa.rs` → zero hits.

## Self-Check: PASSED
