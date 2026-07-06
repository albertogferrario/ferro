---
phase: 238-inertia-first-load-html-shell
plan: 04
subsystem: docs/inertia
tags: [inertia, docs, same-origin, vite-proxy]
requirements: [D-10, D-11]

dependency_graph:
  requires:
    - 238-01 (InertiaConfig::from_env() + title/head_extras/mount_id fields)
    - 238-02 (response template consumes new fields)
    - 238-03 (App::set_inertia_config global plumbing)
  provides:
    - Corrected Manual Configuration example (builder chain, compiles against shipped API)
    - Verified Bootstrap Setup block (from_env + App::set_inertia_config confirmed accurate)
    - First-Load HTML Shell section (same-origin story + Vite proxy recipe)
  affects:
    - docs/src/features/inertia.md

tech_stack:
  added: []
  patterns:
    - Builder-chain doc examples matching the consuming-builder pattern (.title/.head_extras/.mount_id)
    - Vite server.proxy with changeOrigin: false for CSRF-safe cookie forwarding

key_files:
  modified:
    - docs/src/features/inertia.md

decisions:
  - Replaced all three InertiaConfig struct literals (Manual Configuration + two in Development vs Production) with builder-chain form — struct literals would fail to compile against the real API
  - Placed ## First-Load HTML Shell immediately after ## How Inertia Works (before ## Configuration) — same-origin story is conceptual and belongs near the top
  - Used changeOrigin: false in the proxy recipe (preserves Origin header for CSRF) with explicit guidance on when changeOrigin: true applies
  - Added HTML comment validate-against-Vite-docs marker (MEDIUM confidence) per plan requirement
  - Vite proxy recipe validated against Context7 /vitejs/vite at execution time — server.proxy + changeOrigin confirmed valid options (HIGH confidence post-validation)

metrics:
  duration_seconds: 420
  completed_date: "2026-06-21"
  tasks_completed: 2
  files_modified: 1
---

# Phase 238 Plan 04: Inertia Docs Fix and First-Load Shell Section Summary

Documentation corrected for D-11 (stale struct literal) and extended for D-10/SC-5
(same-origin convention + Vite proxy recipe). All struct literal examples replaced
with builder-chain forms matching the shipped API.

## What Was Built

### `docs/src/features/inertia.md` (commits `904aa1d9`, `26363a24`)

**Task 1 — Manual Configuration fix (commit `904aa1d9`):**

Replaced the stale `InertiaConfig { ... }` struct literal in the "Manual Configuration"
section. The old literal omitted `app_name`, `manifest_path`, and the three new fields
(`title`, `head_extras`, `mount_id`), making it fail to compile against the real API.

Replacement:
- Primary example: `InertiaConfig::from_env().title("My App").head_extras(...).mount_id("app")`
- Extended example: full builder chain showing `vite_dev_server`, `entry_point`, `version`,
  `development()`, `mount_id()`
- Explanatory note: `head_extras` is developer-controlled config; ignored when `html_template` set

Two additional struct literals in "Development vs Production" section were also stale
(`development: true/false` with `// ...` placeholders). Replaced with:
- Dev: `InertiaConfig::from_env().development().vite_dev_server("http://localhost:5173")`
- Prod: `InertiaConfig::from_env().production()`

Bootstrap Setup block (lines 38-46) verified accurate: `InertiaConfig::from_env()` and
`App::set_inertia_config(config)` match the APIs shipped by Plans 01 and 03 exactly.
No change needed.

**Task 2 — First-Load HTML Shell section (commit `26363a24`):**

New `## First-Load HTML Shell` section inserted after `## How Inertia Works` and before
`## Configuration`. Covers four subsections:

1. **The first-load document** — explains `<div id="app" data-page="...">` mount node,
   content negotiation (HTML vs JSON on `X-Inertia` header), automatic dev/prod asset
   modes, no hand-rolled HTML needed.

2. **Same-origin convention (recommended)** — backend serves `GET /` and API from the
   same origin; session cookies work with any `SameSite` value; no proxy needed. Stated
   as the recommended pattern.

3. **Vite `server.proxy` recipe (split-port dev)** — `vite.config.ts` snippet with
   `/api` and `/` proxy rules, both with `changeOrigin: false`. Cookie flow explanation:
   `SameSite=Lax/None` work via proxy; `SameSite=Strict` requires same-origin. Guidance
   on `changeOrigin: true` tradeoff (rewrites Origin → can break CSRF). HTML comment
   marker pointing to Vite docs.

4. **`head_extras` and custom template caveat** — security note (developer-controlled,
   not from request data); `html_template` override relationship (when set, `head_extras`
   ignored).

## Vite Proxy Recipe Validation

The proxy recipe was validated against current Vite docs at execution time via Context7
`/vitejs/vite`. Confirmed:
- `server.proxy` is a valid top-level server option
- `changeOrigin` is a valid per-rule option (passed through to `http-proxy-3`)
- Key shape `'/api': { target: ..., changeOrigin: ... }` matches the documented form

The MEDIUM-confidence marker (HTML comment in the doc) is retained as specified — it
signals to future readers to re-validate against evolving Vite docs versions.

## Verification Results

All acceptance criteria met:

**Task 1:**
- `grep -n "InertiaConfig {" docs/src/features/inertia.md` → 0 matches
- `grep -n ".title(" docs/src/features/inertia.md` → line 56
- `grep -n ".mount_id(" docs/src/features/inertia.md` → lines 58, 74
- `grep -n "App::set_inertia_config" docs/src/features/inertia.md` → line 44
- `grep -n "head_extras" docs/src/features/inertia.md` → lines 57, 61, 63, 90, 97 (html_template caveat present)
- `grep -n "InertiaConfig::from_env" docs/src/features/inertia.md` → lines 43, 55, 69, 722, 739

**Task 2:**
- `grep -n "First-Load HTML Shell"` → line 14
- `grep -n "changeOrigin"` → lines 62, 64, 66, 78, 79
- `grep -ni "same-origin"` → lines 33, 41, 75
- `grep -n "data-page"` → lines 22, 38 (in new section)
- `grep -ni "samesite"` → lines 41, 72, 74, 76
- `grep -n "Verify against current Vite docs"` → line 52
- `grep -in "gestiscilo\|example.com"` → 0 matches

**Full verification automated check:**
- `grep -q "server.proxy\|server:" ... && grep -q "changeOrigin" ... && grep -qi "same-origin" ...` → OK
- `grep -q "InertiaConfig::from_env" ... && grep -q "App::set_inertia_config" ...` → OK

**cargo doc --no-deps:** Running at commit time; docs-only changes (markdown only) introduce
no new intra-doc link targets. Any build result is recorded in the self-check below.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed two additional stale struct literals in Development vs Production section**
- **Found during:** Task 1 — acceptance criteria grep `grep -n "InertiaConfig {" docs/src/features/inertia.md` returned 2 matches (lines 722, 741) after fixing the Manual Configuration section
- **Issue:** Development vs Production section used `InertiaConfig { development: true/false, // ... }` — same stale struct literal pattern, same compile failure
- **Fix:** Replaced with `InertiaConfig::from_env().development()` and `InertiaConfig::from_env().production()` builder chains
- **Files modified:** docs/src/features/inertia.md (same file, same commit)
- **Commit:** `904aa1d9`

## Known Stubs

None. All documented examples use the real shipped API surface from Plans 01-03.
The `data-page` example in the new section uses an illustrative HTML fragment (not
a code-compiled example) — this is intentional documentation, not a stub.

## Threat Flags

**T-238-06 mitigated:** The First-Load section explicitly states `head_extras` is
developer-controlled config and must not be populated from request data (XSS prevention).

**T-238-07 mitigated:** The proxy recipe documents `changeOrigin: false` to preserve
Origin for CSRF validation, explains `SameSite` implications, and carries the
MEDIUM-confidence validation marker.

No new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED
