# Phase 159 — Browser Check

**Verdict:** FAIL
**URL:** http://localhost:8080/pagamenti
**Run at:** 2026-05-15T21:06:25Z
**Screenshot:** [pagamenti-screenshot.png](./pagamenti-screenshot.png)

## D-03 Assertion Results

| # | Assertion | Result |
|---|-----------|--------|
| 1 | HTTP 200 (no 404/500/compile-error page) | PASS |
| 2 | Rendered HTML body is non-empty | PASS |
| 3 | StatCard visible — "Totale" AND "€ 1.245,00" present | FAIL |
| 4 | DataTable headers visible — "Data","Descrizione","Importo","Stato" all present | FAIL |
| 5 | No panic / 500-error text in body | PASS |

## evaluate_script return value

```json
{"http_status_marker_no_error_page":true,"body_non_empty":true,"statcard_visible":false,"datatable_headers_visible":false,"datatable_rows_visible":false,"body_length":38222}
```

## Notes

Assertions 3 and 4 failed. The server responded with HTTP 200 and a non-empty body, but the rendered page is a JSON-UI 404 "Pagina non trovata" page (schema `ferro-json-ui/v1`, layout `auth`) — not the pagamenti content.

**Root cause:** `JsonUi::render_file("views/pagamenti.json", data)` in `app/src/controllers/pagamenti.rs:34` resolves the path relative to the process working directory. The spec file is located at `app/src/views/pagamenti.json`, so the path `"views/pagamenti.json"` only resolves if CWD is `app/src/`. When the server runs with CWD `app/` (the documented startup: `cd app && cargo run`), the path expands to `app/views/pagamenti.json` which does not exist. `ferro_json_ui::load_cached` fails, the handler propagates a 500 error, and the framework's catch-all error handler renders the JSON-UI 404 page.

**Evidence from DOM inspection:**
```
data-view contains: "$schema":"ferro-json-ui/v1","layout":"auth","title":"Pagina non trovata — gestiscilo.it"
```

**Fix required (out of scope for Phase 159):** Update the controller path to `"src/views/pagamenti.json"`, or move the spec file to `app/views/pagamenti.json`. Either change allows the server to load the spec correctly when started with `cd app && cargo run`. Per D-11 this is a non-trivial code change and is deferred to a follow-up phase.

**Phase 159 gate result:** FAIL — Phase 160 (v1 API removal) remains blocked until this is resolved.
