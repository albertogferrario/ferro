# Phase 159: v12.0 End-to-End Browser Verification and Docs Build Check - Context

**Gathered:** 2026-05-15
**Status:** Ready for planning
**Mode:** `--auto` — decisions auto-selected from codebase analysis and upstream phase context. Downstream agents may override any auto-choice by editing this file before `/gsd-plan-phase`.

<domain>
## Phase Boundary

Verification-only phase. All implementation is complete from Phase 121 (docs rewrite + pagamenti field test). This phase confirms two things work end-to-end before v1 API removal begins:

1. **Browser test** — Start the ferro sample app, navigate to `/pagamenti` via Chrome MCP, verify `JsonUi::render_file` produces a correctly rendered HTML page (StatCard + DataTable visible, no error page).
2. **Docs build check** — Run `mdbook build docs/` and confirm the rewritten JSON-UI docs build clean with no broken internal links.

Both checks must pass before Phase 160 (v1 API removal) begins.

**What this phase does NOT do:**
- Write any new Rust code
- Change the pagamenti spec, controller, or routes
- Fix fundamental doc content issues (those belong in a separate phase)
- Start Phase 160 (v1 removal) if either check fails

</domain>

<decisions>
## Implementation Decisions

### Server Startup

- **D-01:** Server startup is the user's responsibility per CLAUDE.md ("Server is always ran by the user"). The plan documents the exact command (`cargo run` from `app/`) and waits for the user to start it before Chrome MCP proceeds. Do NOT use `Bash` to start the server binary in the plan.
- **D-02:** The plan should provide the startup command explicitly: `cd app && cargo run` (or equivalent). The app listens on the default port — check `app/src/main.rs` or `app/bootstrap.rs` for the configured port. Assume 3000 unless discovered otherwise.

### Chrome MCP Verification Scope

- **D-03:** A passing browser test requires ALL of the following:
  1. HTTP 200 response (no 404, 500, or compile error page)
  2. Page contains rendered HTML — not an empty body
  3. StatCard element visible (check for text "Totale" or "€ 1.245,00")
  4. DataTable element visible (check for column headers: "Data", "Descrizione", "Importo", "Stato")
  5. No panic/error text visible ("thread 'main' panicked", "500 Internal Server Error")
- **D-04:** Use `mcp__chrome-devtools__navigate_page` then `mcp__chrome-devtools__take_screenshot` + `mcp__chrome-devtools__evaluate_script` (to check DOM). Screenshot saved to the phase directory as evidence.

### Docs Build Tolerance

- **D-05:** `mdbook build docs/` must exit with code 0. Any exit code ≠ 0 is a blocking failure.
- **D-06:** Internal broken links (links to pages within `docs/src/`) = **blocking failure**. Must fix before proceeding.
- **D-07:** External URL broken links (https://...) = **acceptable** if mdbook reports them as warnings only. Use `--no-follow-web-links` flag if available to suppress external link noise. If the flag is not available in the installed mdbook version, document external link errors as non-blocking.
- **D-08:** Missing section files (referenced in SUMMARY.md but file doesn't exist) = **blocking failure**.

### Test Artifact Capture

- **D-09:** Save a screenshot from the Chrome MCP test to `.planning/phases/159-v12-0-end-to-end-browser-verification-and-docs-build-check/pagamenti-screenshot.png`. This is the evidence record for the phase gate check.
- **D-10:** Capture the `mdbook build` output (stdout + stderr) to a text artifact or include it verbatim in the verification report.

### Pass/Fail Gate

- **D-11:** Phase 159 passes only if BOTH checks pass. If either fails, document the specific failure, attempt a minimal fix if the fix is trivial (e.g., a single bad link in SUMMARY.md), and re-run the check. If the fix is non-trivial, stop, document the failure, and do NOT advance to Phase 160.

### Claude's Discretion

- Port discovery: Check `app/src/main.rs` or `app/bootstrap.rs` for the configured listen port. Default assumption is 3000 if not found.
- Screenshot resolution and Chrome viewport: standard desktop resolution is fine.
- Order of checks: run the docs build first (no server required), then the browser test (requires user to start the server).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Implementation
- `app/src/controllers/pagamenti.rs` — the handler under test; data-only, calls `JsonUi::render_file`
- `app/src/views/pagamenti.json` — the v2 JSON spec being tested end-to-end
- `app/src/routes.rs` — confirms `/pagamenti` route is registered

### Docs
- `docs/book.toml` — mdbook configuration; build output dir is `docs/book/`
- `docs/src/SUMMARY.md` — source of truth for all linked pages; broken links originate here
- `docs/src/json-ui/` — the 8 pages rewritten in Phase 121 (getting-started, components, actions, data-binding, layouts, plugins, expressions, json-schema)

### Framework Rendering Path
- `ferro-json-ui/src/lib.rs` — public API (`JsonUi::render_file` entry point)

### Prior Phase Context
- `.planning/phases/121-documentation-and-field-test/121-CONTEXT.md` — decisions from Phase 121 (field test spec, doc page list)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `pagamenti.rs` controller and `pagamenti.json` spec: already complete from Phase 121 — this phase only verifies them
- `/pagamenti` route: already registered in `app/src/routes.rs`
- Chrome DevTools MCP: available as `mcp__chrome-devtools__*` tools

### Established Patterns
- Chrome MCP test pattern: `navigate_page` → `take_screenshot` → `evaluate_script` for DOM checks
- `mdbook build docs/`: standard build command; exit code 0 = success
- Phase gate pattern: both checks must pass before next phase proceeds

### Integration Points
- Chrome MCP instance 1 (`mcp__chrome-devtools__*`): use for the browser test
- Local app server (user-started): target URL `http://localhost:{PORT}/pagamenti`
- `docs/` directory: mdbook source root

</code_context>

<specifics>
## Specific Ideas

- The pagamenti demo uses Italian locale strings ("€ 1.245,00", "Totale") — DOM checks should use these exact strings or the column header keys ("Data", "Descrizione", etc.) which are locale-independent
- `create-missing = false` in `book.toml` means any page referenced in SUMMARY.md that doesn't exist on disk will cause `mdbook build` to fail — this is the most likely failure mode for the docs check

</specifics>

<deferred>
## Deferred Ideas

- Automated CI integration of the Chrome MCP test — belongs in a later CI/CD phase
- Testing additional routes beyond `/pagamenti` — scope is fixed to this one route for Phase 159
- Performance profiling of the render path — out of scope for verification

</deferred>

---

*Phase: 159-v12-0-end-to-end-browser-verification-and-docs-build-check*
*Context gathered: 2026-05-15*
