# Phase 159: v12.0 End-to-End Browser Verification and Docs Build Check - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-15
**Phase:** 159-v12-0-end-to-end-browser-verification-and-docs-build-check
**Mode:** `--auto` (all decisions auto-selected)
**Areas discussed:** Server Startup, Verification Scope, Docs Build Tolerance, Test Artifacts

---

## Server Startup

| Option | Description | Selected |
|--------|-------------|----------|
| Plan provides command, user starts server | CLAUDE.md mandates user starts server; plan documents `cd app && cargo run` and waits | ✓ |
| Plan uses Bash to start server | Would violate CLAUDE.md constraint | |

**Auto-selected:** Plan provides the startup command; execution requires user to start the server.
**Notes:** CLAUDE.md explicitly states "Server is always ran by the user, dont start server instances." Port assumed 3000 unless discovered otherwise during planning.

---

## Verification Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Screenshot only | Quick but no DOM assertion | |
| Screenshot + DOM check | HTTP 200, StatCard text, DataTable headers, no error text | ✓ |
| Full HTML diff | Overkill for a verification phase | |

**Auto-selected:** Screenshot + DOM check (HTTP 200, StatCard visible, DataTable column headers visible, no error text).
**Notes:** Specific DOM selectors: text "Totale", "€ 1.245,00", and column headers "Data", "Descrizione", "Importo", "Stato".

---

## Docs Build Tolerance

| Option | Description | Selected |
|--------|-------------|----------|
| Zero tolerance (all warnings fail) | Too strict — external link latency varies | |
| Internal links only (external OK) | Standard CI practice; external links unreliable in dev | ✓ |
| Warnings acceptable | Too loose — misses real broken pages | |

**Auto-selected:** Internal broken links = blocking fail; external URL warnings = acceptable.
**Notes:** `create-missing = false` in book.toml means missing files referenced by SUMMARY.md will cause exit code ≠ 0 — most likely failure mode.

---

## Test Artifacts

| Option | Description | Selected |
|--------|-------------|----------|
| No artifacts | Nothing to review post-phase | |
| Screenshot to phase dir | Provides visual evidence; zero cost | ✓ |
| Full HTML capture | Excessive storage | |

**Auto-selected:** Screenshot saved to `.planning/phases/159-.../pagamenti-screenshot.png`.

---

## Claude's Discretion

- Port discovery from `app/src/main.rs` or `app/bootstrap.rs`
- Chrome viewport and resolution
- Order of checks (docs build first, browser test second)

## Deferred Ideas

- CI automation of Chrome MCP test
- Testing additional routes
- Performance profiling
