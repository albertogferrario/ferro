---
phase: 159-v12-0-end-to-end-browser-verification-and-docs-build-check
verified: 2026-05-15T21:11:20Z
status: gaps_found
score: 7/12 must-haves verified
overrides_applied: 0
gaps:
  - truth: "GET http://localhost:8080/pagamenti returns HTTP 200 (not 404, not 500, not a compile error page)"
    status: partial
    reason: "HTTP 200 is returned, but the route serves a JSON-UI 404 catch-all page (schema ferro-json-ui/v1, layout auth, title 'Pagina non trovata') rather than the pagamenti content. The route responds 200 with the wrong page — technically not a network-level error, but the pagamenti content is not rendered."
    artifacts:
      - path: "app/src/controllers/pagamenti.rs"
        issue: "Line 34: JsonUi::render_file(\"views/pagamenti.json\", data) resolves relative to CWD. The spec is at app/src/views/pagamenti.json; when the server starts with CWD=app/ (documented startup: cd app && cargo run), the path expands to app/views/pagamenti.json which does not exist. The framework's error handler returns the catch-all 404 page."
    missing:
      - "Fix the path argument: change \"views/pagamenti.json\" to \"src/views/pagamenti.json\" (one-line change in app/src/controllers/pagamenti.rs line 34), OR move app/src/views/pagamenti.json to app/views/pagamenti.json to match the existing path string"
      - "Re-run the Chrome MCP browser test after the fix to confirm all five D-03 assertions pass"
  - truth: "The StatCard is visible — the literal string 'Totale' AND the literal string '€ 1.245,00' both appear in the rendered HTML body"
    status: failed
    reason: "DOM assertion returned statcard_visible=false. The page renders the JSON-UI 404 catch-all, not the pagamenti StatCard. Root cause is the render_file path bug above."
    artifacts:
      - path: "app/src/controllers/pagamenti.rs"
        issue: "render_file path does not resolve; StatCard is never rendered"
      - path: ".planning/phases/159-v12-0-end-to-end-browser-verification-and-docs-build-check/BROWSER-CHECK.md"
        issue: "evaluate_script JSON confirms statcard_visible=false, datatable_rows_visible=false"
    missing:
      - "Fix the render_file path (same fix as gap above) — the StatCard will appear once pagamenti.json loads correctly"
  - truth: "The DataTable is visible — all four literal column headers 'Data', 'Descrizione', 'Importo', 'Stato' appear in the rendered HTML body"
    status: failed
    reason: "DOM assertion returned datatable_headers_visible=false. The same root cause prevents the DataTable from rendering."
    artifacts:
      - path: "app/src/controllers/pagamenti.rs"
        issue: "render_file path does not resolve; DataTable is never rendered"
    missing:
      - "Fix the render_file path (same fix as gaps above) — the DataTable will appear once pagamenti.json loads correctly"
---

# Phase 159: v12.0 End-to-End Browser Verification and Docs Build Check — Verification Report

**Phase Goal:** Produce pass/fail evidence for the Phase 159 gate: (1) mdbook docs build clean, (2) Chrome MCP browser test of /pagamenti passes with all 5 D-03 assertions. Both checks must PASS for Phase 160 to be unblocked.
**Verified:** 2026-05-15T21:11:20Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

Plan 01 (docs build) must-haves:

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | mdbook build docs/ exits with code 0 on the current branch | VERIFIED | mdbook-build.log final line: EXIT_CODE=0 |
| 2 | No internal broken links reported by mdbook | VERIFIED | DOCS-CHECK.md "Blocking issues: None"; log contains no ERROR/WARN lines |
| 3 | No SUMMARY.md entry references a missing file (create-missing = false) | VERIFIED | create-missing = false confirmed in docs/book.toml; build exits 0 with no missing-file errors |
| 4 | External URL warnings, if any, documented as non-blocking | VERIFIED | DOCS-CHECK.md "Non-blocking external link warnings: None" — no warnings of any kind |
| 5 | Full mdbook build stdout+stderr captured to log file in phase directory | VERIFIED | mdbook-build.log exists, non-empty, contains complete build output |

Plan 02 (browser test) must-haves:

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 6 | GET http://localhost:8080/pagamenti returns HTTP 200 | PARTIAL | HTTP 200 returned, but serves JSON-UI 404 catch-all — wrong content |
| 7 | The rendered page body is non-empty | VERIFIED | evaluate_script: body_non_empty=true, body_length=38222 |
| 8 | StatCard visible — "Totale" AND "€ 1.245,00" both appear in rendered HTML body | FAILED | evaluate_script: statcard_visible=false — render_file path does not resolve |
| 9 | DataTable visible — all four headers "Data","Descrizione","Importo","Stato" appear | FAILED | evaluate_script: datatable_headers_visible=false — same root cause |
| 10 | No panic or 500-error text in body | VERIFIED | evaluate_script: http_status_marker_no_error_page=true |
| 11 | Screenshot saved as evidence artifact | VERIFIED | pagamenti-screenshot.png exists, 111166 bytes |

Roadmap-level success criteria:

| # | Success Criterion | Status | Evidence |
|---|-------------------|--------|----------|
| 12 | Chrome MCP browser test of /pagamenti passes | FAILED | BROWSER-CHECK.md Verdict: FAIL — assertions 3 and 4 failed |
| 13 | mdbook build docs/ exits cleanly with no broken links | VERIFIED | DOCS-CHECK.md Verdict: PASS, Exit code: 0 |

**Score:** 7/12 must-haves verified (Plan 01 fully closed; Plan 02 partial — 3 of 6 truths pass)

**Phase gate (D-11): FAIL** — both checks must pass for the gate to close. The docs check passes; the browser test fails on assertions 3 and 4.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `mdbook-build.log` | Verbatim mdbook stdout+stderr (min 1 line) | VERIFIED | 4 lines, EXIT_CODE=0 on final line, complete output |
| `DOCS-CHECK.md` | Pass/fail report with exit code, "Exit code: 0" | VERIFIED | Verdict: PASS, Exit code: 0, cites mdbook-build.log, three required sections present |
| `pagamenti-screenshot.png` | PNG screenshot of /pagamenti (non-empty) | VERIFIED | Exists, 111166 bytes — shows "Pagina non trovata" 404 page |
| `BROWSER-CHECK.md` | Pass/fail report with URL and 5-row D-03 table | VERIFIED | Verdict: FAIL, URL present, 5-row table present, evaluate_script JSON embedded |

All four evidence artifacts exist and are substantive. The screenshot and BROWSER-CHECK.md accurately document the failure.

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `docs/book.toml` | `docs/src/SUMMARY.md` | mdbook resolves every SUMMARY entry against filesystem | WIRED | create-missing = false confirmed; build passes with no missing-file errors |
| `DOCS-CHECK.md` | `mdbook-build.log` | report cites log as evidence | WIRED | "Log: [mdbook-build.log](./mdbook-build.log)" present in DOCS-CHECK.md |
| Chrome MCP | http://localhost:8080/pagamenti | mcp__chrome-devtools__navigate_page | WIRED | Navigation executed; page loaded; evaluate_script ran and returned JSON |
| `BROWSER-CHECK.md` | `pagamenti-screenshot.png` | report cites screenshot | WIRED | "Screenshot: [pagamenti-screenshot.png](./pagamenti-screenshot.png)" present |
| `app/src/controllers/pagamenti.rs` | `app/src/views/pagamenti.json` | JsonUi::render_file | BROKEN | render_file("views/pagamenti.json") resolves to app/views/pagamenti.json (does not exist when CWD=app/); spec at app/src/views/pagamenti.json is never loaded |

The broken key link between `pagamenti.rs` and `pagamenti.json` is the root cause of the browser test failure.

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `app/src/controllers/pagamenti.rs` | JSON `data` serde_json::Value | Hardcoded inline in handler | Yes (hardcoded sample data) | STATIC — but intentional for the field-test demo |
| `JsonUi::render_file` return | HTML response | app/src/views/pagamenti.json | NO — file not found at resolved path | DISCONNECTED — render_file path does not resolve |

The data variable (sample JSON) is populated and passed to render_file. The disconnect is at the file I/O layer: the spec JSON file is at the wrong relative path for the configured CWD.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| mdbook build exits 0 | tail -1 mdbook-build.log \| grep EXIT_CODE=0 | EXIT_CODE=0 | PASS |
| DOCS-CHECK.md contains PASS verdict | grep "Verdict: PASS" DOCS-CHECK.md | Match found | PASS |
| render_file path resolves | ls app/views/pagamenti.json | No such file | FAIL |
| BROWSER-CHECK.md contains FAIL verdict | grep "Verdict: FAIL" BROWSER-CHECK.md | Match found | PASS (correctly documents failure) |
| Screenshot non-empty | ls -la pagamenti-screenshot.png | 111166 bytes | PASS |

### Requirements Coverage

No REQUIREMENTS.md entries are assigned to Phase 159 (verification-only phase). Phase gate D-11 is defined in 159-CONTEXT.md and explicitly requires both checks to pass. D-11 status: FAIL.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `app/src/controllers/pagamenti.rs` | 34 | `JsonUi::render_file("views/pagamenti.json", data)` — relative path assumes CWD=app/src/, but documented startup is `cd app && cargo run` (CWD=app/) | Blocker | Prevents pagamenti JSON-UI spec from loading; render_file silently fails to find the file, triggering the catch-all 404 page |

### Human Verification Required

None. The failure is fully diagnosed programmatically:
- The evaluate_script JSON (`statcard_visible=false`, `datatable_headers_visible=false`) confirms which assertions failed.
- The path resolution bug is confirmed by `ls app/src/views/pagamenti.json` (exists) vs `ls app/views/pagamenti.json` (does not exist).
- The screenshot shows the resulting 404 page.

No UI polish, real-time behavior, or external service judgment is needed. The fix and re-test can be fully automated.

### Gaps Summary

The phase has one root cause blocking three must-haves (6-partial, 8-failed, 9-failed):

**Root cause:** `JsonUi::render_file("views/pagamenti.json", data)` in `app/src/controllers/pagamenti.rs` line 34 uses a CWD-relative path. The documented server startup is `cd app && cargo run`, making CWD = `app/`. The spec file lives at `app/src/views/pagamenti.json`, so the resolved path `app/views/pagamenti.json` does not exist. The framework's error handler serves the catch-all "Pagina non trovata" JSON-UI 404 page.

**Fix:** One-line change to `app/src/controllers/pagamenti.rs` line 34 — change `"views/pagamenti.json"` to `"src/views/pagamenti.json"`. After the fix, re-run the Chrome MCP browser test to confirm all five D-03 assertions pass and close the Phase 159 gate.

Phase 160 (v1 API removal) remains blocked until the gate closes.

No later phase in the roadmap (160, 161, 162, 163, 164) specifically addresses this path bug in the sample app's pagamenti controller. Phase 162 mentions "render_file ergonomics" in the context of FRICTION.md triage from gestiscilo, but the sample app path correction is a separate, pre-existing issue that must be resolved before Phase 160 begins.

---

_Verified: 2026-05-15T21:11:20Z_
_Verifier: Claude (gsd-verifier)_
