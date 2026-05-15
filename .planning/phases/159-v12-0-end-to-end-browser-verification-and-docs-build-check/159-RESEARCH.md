# Phase 159: v12.0 End-to-End Browser Verification and Docs Build Check - Research

**Researched:** 2026-05-15
**Domain:** Verification — Chrome MCP browser test + mdbook docs build
**Confidence:** HIGH (all findings verified directly against the codebase and environment)

## Summary

This is a verification-only phase. No new code is written. It gates Phase 160 (v1 API removal) on two checks: a Chrome MCP browser test of `/pagamenti` and a clean `mdbook build docs/` run. Both checks were partially pre-validated during research — the app compiles cleanly (`cargo check -p app` exits 0) and `mdbook build docs/` already exits 0 with no errors. The browser test itself cannot be pre-run here (server not started), but the full rendering path from handler through `JsonUi::render_file` to HTML output is implemented and compilation-verified.

**Primary recommendation:** Run docs build first (no server needed, already known-good), then prompt the user to start the app server and proceed with the Chrome MCP browser test. Both checks are expected to pass on the current branch state.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Server startup is the user's responsibility per CLAUDE.md. The plan documents the exact command and waits for the user. Do NOT use Bash to start the server binary.
- **D-02:** Startup command: `cd app && cargo run`. App listens on port 8080 (confirmed from app/.env: `SERVER_PORT=8080`). Assumption of port 3000 is incorrect — verified port is 8080.
- **D-03:** Passing browser test requires ALL of: HTTP 200, non-empty body, StatCard with "Totale" or "€ 1.245,00" visible, DataTable with headers "Data"/"Descrizione"/"Importo"/"Stato", no panic/error text.
- **D-04:** Use `mcp__chrome-devtools__navigate_page` → `mcp__chrome-devtools__take_screenshot` + `mcp__chrome-devtools__evaluate_script` for DOM checks. Screenshot saved to phase directory.
- **D-05:** `mdbook build docs/` must exit code 0.
- **D-06:** Internal broken links = blocking failure.
- **D-07:** External URL broken links = acceptable if warnings only. `--no-follow-web-links` flag does NOT exist in mdbook v0.5.2 (verified). Document external link errors as non-blocking.
- **D-08:** Missing SUMMARY.md section files = blocking failure.
- **D-09:** Screenshot saved to `.planning/phases/159-v12-0-end-to-end-browser-verification-and-docs-build-check/pagamenti-screenshot.png`.
- **D-10:** Capture `mdbook build` stdout+stderr verbatim in the verification report.
- **D-11:** Phase passes only if BOTH checks pass. Minimal fixes allowed for trivial issues; stop and document for non-trivial failures.

### Claude's Discretion

- Port discovery: resolved — port is 8080 (from app/.env).
- Screenshot resolution: standard desktop resolution.
- Order of checks: docs build first (no server required), then browser test.

### Deferred Ideas (OUT OF SCOPE)

- Automated CI integration of the Chrome MCP test
- Testing additional routes beyond `/pagamenti`
- Performance profiling of the render path
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Browser verification | Client (Chrome MCP) | — | Navigate, screenshot, DOM eval all happen in Chrome |
| Page rendering | API/Backend (Ferro app) | — | `JsonUi::render_file` runs in the Rust server; HTML is server-rendered |
| Docs build | Build tooling (mdbook) | — | Static site generation; no server involved |
| Data assembly | API/Backend | — | `pagamenti.rs` controller assembles JSON data inline |

## Standard Stack

### Core

| Tool | Version | Purpose | Notes |
|------|---------|---------|-------|
| mdbook | v0.5.2 | Docs build | Installed at PATH; `mdbook build docs/` is the exact command |
| Chrome MCP | — | Browser automation | `mcp__chrome-devtools__*` tools |
| Ferro app | 0.2.11 | Web server under test | Start with `cargo run` from `app/` |

### Key Discovery: No `--no-follow-web-links` Flag

mdbook v0.5.2 `build` subcommand only supports `--dest-dir` and `--open`. There is no `--no-follow-web-links` flag. Decision D-07 must fall back to: document any external URL warnings as non-blocking without a special flag.

**Installation:** Nothing to install — all tools are already present.
[VERIFIED: `mdbook --version` → v0.5.2, `cargo check -p app` → Finished cleanly]

## Architecture Patterns

### System Architecture Diagram

```
User starts server
       │
       ▼
app/ (cargo run) ── DB::init() ── SQLite ./database.db
       │
       ▼ (auto-migrate on startup)
HTTP server @ 127.0.0.1:8080
       │
       ▼ GET /pagamenti
controllers/pagamenti::index()
       │
       ▼ JsonUi::render_file("views/pagamenti.json", data)
framework/json_ui → ferro-json-ui → load_cached() → parse spec
       │                          → merge_data(handler_data)
       │                          → build_response() → render_layout("dashboard")
       ▼
HTML response (200, text/html; charset=utf-8)
       │
       ▼
Chrome MCP ── navigate_page(http://localhost:8080/pagamenti)
            ── take_screenshot → pagamenti-screenshot.png
            ── evaluate_script → DOM assertions


docs/src/ ──── mdbook build docs/ ──── docs/book/ (static HTML)
                     │
                     ▼ exit code 0 = pass
```

### Key Verified Facts

**App port:** 8080 (not 3000).
Source: `app/.env` — `SERVER_HOST=127.0.0.1`, `SERVER_PORT=8080`
[VERIFIED: read app/.env directly]

**App startup requires a database.** `bootstrap.rs` calls `DB::init()` which requires `DATABASE_URL`. The `.env` file sets `DATABASE_URL=sqlite://./database.db`. The SQLite file is created automatically by `main.rs` if absent. No external DB server required.
[VERIFIED: read app/src/bootstrap.rs and app/src/main.rs]

**`/pagamenti` route is registered.**
`app/src/routes.rs` line 13: `get!("/pagamenti", controllers::pagamenti::index).name("pagamenti.index")`
[VERIFIED: read app/src/routes.rs]

**`pagamenti.rs` handler is data-only, no DB queries.** Data is hardcoded inline — three payment records. No database queries are made by this handler. The DB must be reachable only because `DB::init()` runs on startup globally.
[VERIFIED: read app/src/controllers/pagamenti.rs]

**`pagamenti.json` spec content (v2):** Defines a `Card` containing a `StatCard` (label: "Totale", value: `$data:/meta/totale_formattato`) and a `DataTable` (columns: Data, Descrizione, Importo, Stato; data_path: `/pagamenti`).
[VERIFIED: read app/src/views/pagamenti.json]

**Expected DOM strings for assertions:**
- StatCard: "Totale" and "€ 1.245,00"
- DataTable headers: "Data", "Descrizione", "Importo", "Stato"
- DataTable rows: "Abbonamento mensile", "€ 99,00", "Completato" etc.
[VERIFIED: read pagamenti.json + pagamenti.rs]

**`mdbook build docs/` exits 0 on current branch.** Run during research — output was clean, no warnings, no errors.
[VERIFIED: `mdbook build docs/` executed, exit code: 0]

**All SUMMARY.md entries exist on disk.** Every `.md` file referenced in `docs/src/SUMMARY.md` is present on disk. No missing files.
[VERIFIED: shell loop over SUMMARY.md references]

**`create-missing = false` in book.toml.** This means any missing file would cause mdbook to fail rather than auto-create it. Since all files exist, this is not a concern for the current build.
[VERIFIED: read docs/book.toml]

**`JsonUi::render_file` is in framework crate, not ferro-json-ui.** The public API lives in `framework/src/json_ui/mod.rs`. It delegates to `ferro_json_ui::load_cached()` for the spec cache.
[VERIFIED: grep of render_file across workspace]

**App compiles cleanly on current branch.**
`cargo check -p app` finished with no errors (30s on dev profile).
[VERIFIED: cargo check -p app exit 0]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| DOM assertions | Custom response parser | `mcp__chrome-devtools__evaluate_script` with `document.body.innerHTML.includes(...)` |
| Screenshot capture | Any custom tool | `mcp__chrome-devtools__take_screenshot` |
| Link checking | Custom link validator | `mdbook build` itself fails on internal broken links |

## Common Pitfalls

### Pitfall 1: Wrong Port
**What goes wrong:** Navigating to `http://localhost:3000/pagamenti` — default assumption in CONTEXT.md was 3000, actual is 8080.
**Why it happens:** The `.env` specifies `SERVER_PORT=8080` which overrides any framework default.
**How to avoid:** Use `http://localhost:8080/pagamenti`.
**Warning signs:** Chrome returns "Connection refused" or ERR_CONNECTION_REFUSED.

### Pitfall 2: Server Startup Fails Without Database
**What goes wrong:** `cargo run` panics immediately — "DATABASE_URL not set" or SQLite cannot be created.
**Why it happens:** `bootstrap.rs` runs `DB::init()` before the server starts listening.
**How to avoid:** Run `cargo run` from within the `app/` directory (not the workspace root) so `.env` is loaded from `app/.env`. The SQLite file `database.db` is created automatically if absent.
**Warning signs:** Server exits immediately with "Error: Failed to connect to database".

### Pitfall 3: `render_file` Path Relative to CWD
**What goes wrong:** `JsonUi::render_file("views/pagamenti.json", ...)` fails to find the file.
**Why it happens:** The path is resolved relative to the process working directory. If the server is started from the workspace root instead of `app/`, the path `views/pagamenti.json` won't resolve.
**How to avoid:** Start with `cd app && cargo run`, not `cargo run -p app` from the workspace root.
**Warning signs:** 500 response with "Failed to load spec: ..." in the page body.

### Pitfall 4: mdbook External Link Noise
**What goes wrong:** mdbook may print warnings about external URLs (https://...) that cannot be validated without network access.
**Why it happens:** mdbook v0.5.2 does not have a `--no-follow-web-links` flag.
**How to avoid:** Treat any warnings about external URLs as non-blocking. Only exit code ≠ 0 or internal link errors are blocking.
**Warning signs:** Lines like `WARN [mdbook::preprocess::links] ...` for https:// URLs.

### Pitfall 5: Screenshot Path
**What goes wrong:** Screenshot saved to wrong location.
**How to avoid:** Save to exactly: `.planning/phases/159-v12-0-end-to-end-browser-verification-and-docs-build-check/pagamenti-screenshot.png`

## Code Examples

### Chrome MCP DOM Assertion Pattern
```javascript
// Source: Chrome MCP evaluate_script — check for StatCard
document.body.innerHTML.includes('Totale') && document.body.innerHTML.includes('€ 1.245,00')

// Check for DataTable headers
['Data', 'Descrizione', 'Importo', 'Stato'].every(h => document.body.innerHTML.includes(h))

// Check for no error text
!document.body.innerHTML.includes('panicked') && !document.body.innerHTML.includes('500 Internal Server Error')
```
[ASSUMED: evaluate_script accepts a JS expression returning boolean]

### mdbook Build Command
```bash
# From workspace root
mdbook build docs/
echo "Exit code: $?"
```
[VERIFIED: executed during research, exit code 0]

### Server Startup (user runs this)
```bash
cd /path/to/ferro/app
cargo run
# Server starts at http://127.0.0.1:8080
```

## Runtime State Inventory

Not applicable — this is a verification-only phase with no renames, refactors, or migrations.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| mdbook | Docs build check | Yes | v0.5.2 | — |
| Chrome MCP | Browser test | Yes (configured in ~/.claude.json) | — | — |
| Ferro app (compiled) | Browser test | Yes (cargo check passes) | 0.2.11 | — |
| SQLite (via app/.env) | App startup (DB::init) | Yes (sqlite: file-based) | — | — |

**Missing dependencies with no fallback:** None.

**Note:** The app server itself is started by the user — it is not a tool dependency of the plan, it is a prerequisite the user satisfies.

## Validation Architecture

nyquist_validation key is absent from config.json — treated as enabled.

This phase is itself a verification phase. There are no unit/integration tests to write. The "tests" are the two manual checks performed during execution:

| Check | Type | Pass Criterion |
|-------|------|----------------|
| mdbook build | Build check | Exit code 0, no internal link errors |
| Chrome MCP browser test | Manual/MCP | All 5 criteria in D-03 satisfied |

No Wave 0 gaps — no test infrastructure files need to be created.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `evaluate_script` accepts a JS expression returning boolean and the return value is inspectable | Code Examples | Would need a different DOM assertion pattern |

## Open Questions

1. **Will the server be running when the Chrome MCP test executes?**
   - What we know: CLAUDE.md forbids the plan from starting the server.
   - What's unclear: Whether the user will have the server started before the agent proceeds.
   - Recommendation: The plan must include an explicit "pause here" step asking the user to start the server and confirm before proceeding with Chrome MCP.

## Sources

### Primary (HIGH confidence)
- `app/.env` — port (8080), DATABASE_URL (sqlite://./database.db)
- `app/src/routes.rs` — `/pagamenti` route confirmed registered
- `app/src/controllers/pagamenti.rs` — handler is data-only, no DB queries
- `app/src/views/pagamenti.json` — spec structure, expected DOM strings
- `app/src/bootstrap.rs` — DB::init() required on startup
- `docs/book.toml` — mdbook config, `create-missing = false`, build-dir: `book`
- `docs/src/SUMMARY.md` — all referenced files verified to exist on disk
- `mdbook build docs/` executed — exit code 0, clean output
- `cargo check -p app` — clean compile on current branch
- `mdbook --help` — confirmed no `--no-follow-web-links` flag in v0.5.2

## Metadata

**Confidence breakdown:**
- Port and startup: HIGH — read directly from app/.env
- Route registration: HIGH — read from routes.rs
- DOM strings: HIGH — read from pagamenti.json + pagamenti.rs
- mdbook build status: HIGH — executed during research
- App compilation: HIGH — cargo check executed during research
- mdbook flag availability: HIGH — read from mdbook --help

**Research date:** 2026-05-15
**Valid until:** 2026-06-15 (stable codebase; only changes if Phase 121 artifacts are modified)
