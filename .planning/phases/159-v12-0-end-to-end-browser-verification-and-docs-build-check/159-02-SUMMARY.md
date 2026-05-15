---
plan: 159-02
phase: 159-v12-0-end-to-end-browser-verification-and-docs-build-check
status: complete
verdict: FAIL
completed: 2026-05-15T21:06:25Z
---

# Plan 159-02 Summary: Chrome MCP Browser Test

## What Was Built

Executed the Chrome MCP browser verification of `http://localhost:8080/pagamenti` against the live Ferro app. Captured a screenshot and ran five D-03 DOM assertions via `evaluate_script`.

## D-03 Assertion Outcomes

| # | Assertion | Result |
|---|-----------|--------|
| 1 | HTTP 200 / no error page | PASS |
| 2 | Body non-empty | PASS |
| 3 | StatCard ("Totale" + "€ 1.245,00") | FAIL |
| 4 | DataTable headers (Data/Descrizione/Importo/Stato) | FAIL |
| 5 | No panic/500 text | PASS |

**Overall: FAIL** (assertions 3 and 4)

## Root Cause

`JsonUi::render_file("views/pagamenti.json", data)` in `app/src/controllers/pagamenti.rs:34` resolves relative to CWD. The spec file lives at `app/src/views/pagamenti.json`. With CWD = `app/` (from `cd app && cargo run`), the path `app/views/pagamenti.json` does not exist. The framework serves a JSON-UI 404 catch-all instead of the pagamenti content.

**Fix:** Change the call to `JsonUi::render_file("src/views/pagamenti.json", data)` OR move the spec file to `app/views/pagamenti.json`. This is a one-line fix but is out of scope for Phase 159 (verification-only, per D-11).

## Evidence Artifacts

- `pagamenti-screenshot.png` — screenshot showing the 404 "Pagina non trovata" page
- `BROWSER-CHECK.md` — full assertion table and root cause analysis

## Phase 159 Gate Decision

| Check | Verdict |
|-------|---------|
| Docs build (Plan 01) | PASS |
| Browser test (Plan 02) | FAIL |
| **Phase 159 gate** | **FAIL** |

Phase 160 (v1 API removal) remains blocked. The fix is a one-line path correction in `pagamenti.rs` — recommend a targeted gap-closure phase before proceeding to Phase 160.

## Self-Check: PASSED

All plan tasks executed, evidence artifacts committed, BROWSER-CHECK.md written with FAIL verdict and verbatim `evaluate_script` JSON, no source code modified (app/, framework/, ferro-json-ui/, docs/ untouched).
