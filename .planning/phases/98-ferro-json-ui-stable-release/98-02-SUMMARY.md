---
phase: 98-ferro-json-ui-stable-release
plan: 02
subsystem: ui
tags: [ferro-json-ui, sse, javascript, tailwind, dashboard, layout]

# Dependency graph
requires:
  - phase: 98-01
    provides: SidebarProps, HeaderProps, SidebarNavItem, SidebarGroup component types and render functions

provides:
  - DashboardLayout struct implementing Layout trait with persistent sidebar/header shell
  - DashboardLayoutConfig struct for per-app sidebar/header/SSE configuration
  - FERRO_RUNTIME_JS const (~5KB vanilla JS IIFE) handling SSE/toast/live-value/checklist/dropdown/sidebar-toggle
  - runtime.rs module declared as pub(crate) in ferro-json-ui

affects:
  - 98-03 (component rendering improvements may use DashboardLayout and runtime)
  - 98-04 (docs should document DashboardLayout usage and JS runtime data attributes)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "DashboardLayout registers itself via user call to register_layout() at startup — not auto-registered like DefaultLayout/AppLayout/AuthLayout"
    - "JS runtime uses IIFE with var declarations for maximum browser compat"
    - "Data attribute convention: all JS behaviors driven by data-* attributes, no configuration objects"
    - "base_document_ext extends base_document with optional body data attributes for SSE URL injection"

key-files:
  created:
    - ferro-json-ui/src/runtime.rs
  modified:
    - ferro-json-ui/src/layout.rs
    - ferro-json-ui/src/lib.rs

key-decisions:
  - "DashboardLayout not auto-registered in LayoutRegistry::new() — requires dynamic config (sidebar/header/sse_url) that varies per app, unlike stateless DefaultLayout/AppLayout/AuthLayout"
  - "#[allow(dead_code)] applied to FERRO_RUNTIME_JS in Task 1 commit then removed in Task 2 when the constant gained its first consumer in layout.rs"
  - "base_document_ext added alongside base_document to inject optional body data attributes (data-sse-url) without modifying the base helper signature"
  - "Test for 'no SSE URL on body' checks the body opening tag only, since data-sse-url appears as a string literal inside the embedded JS runtime"

patterns-established:
  - "Layout-level JS injection: runtime script appended after ctx.scripts to ensure it loads after plugin scripts"
  - "Mobile-first sidebar: hidden class by default, md:flex for desktop — toggled by data-sidebar-toggle button in header"

requirements-completed: [DASH-01, DASH-02, DASH-03, JS-01, JS-02, JS-03]

# Metrics
duration: 5min
completed: 2026-03-11
---

# Phase 98 Plan 02: DashboardLayout and JS Runtime Summary

**DashboardLayout with persistent sidebar/header shell and ~5KB vanilla JS runtime for SSE/toast/live-value/checklist/dropdown/mobile-toggle behaviors**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-11T16:24:00Z
- **Completed:** 2026-03-11T16:29:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Built-in JS runtime (`FERRO_RUNTIME_JS`) as a vanilla IIFE (~5KB) with SSE connection management, live-value replacement, toast stacking/auto-dismiss, checklist dismiss, notification dropdown toggle, and sidebar mobile toggle
- `DashboardLayout` implementing `Layout` trait with fixed sidebar (`w-64`, `hidden md:flex`), sticky header with hamburger button, main content area (`md:pl-64`), toast container, and FERRO_RUNTIME_JS injected once per page
- `DashboardLayoutConfig` struct for per-app sidebar/header data and optional SSE URL, user-registered at app startup
- 13 new DashboardLayout unit tests covering structure, SSE URL injection, XSS escaping, mobile toggle, toast container, and notification toggle
- Total ferro-json-ui tests: 317 unit + 7 doc-tests

## Task Commits

1. **Task 1: Create built-in JS runtime** - `2407941` (feat)
2. **Task 2: Add DashboardLayout with persistent sidebar/header shell** - `b6a4906` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `ferro-json-ui/src/runtime.rs` - FERRO_RUNTIME_JS const: IIFE handling SSE, live-values, toasts, checklists, dropdowns, sidebar mobile toggle
- `ferro-json-ui/src/layout.rs` - DashboardLayoutConfig, DashboardLayout, base_document_ext helper, layout_sidebar_html and layout_header_html private helpers, 13 new tests
- `ferro-json-ui/src/lib.rs` - Added `pub(crate) mod runtime`, exported DashboardLayout and DashboardLayoutConfig from public API

## Decisions Made

- DashboardLayout not auto-registered in `LayoutRegistry::new()` — unlike the stateless `DefaultLayout`/`AppLayout`/`AuthLayout`, it requires runtime config (sidebar items, header data, SSE URL) that varies per application
- `base_document_ext` added alongside the existing `base_document` to support optional body data attributes without changing the shared helper signature used by three existing layouts
- JS runtime IIFE uses `var` (not `let`/`const`) for maximum browser compat; `DOMContentLoaded` for auto-init; reads all config from data attributes
- Test for "no SSE URL attribute" checks only the body tag opening substring, because the `data-sse-url` string literal also appears inside the embedded JS runtime source code

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

- Test `dashboard_layout_no_sse_url_when_not_configured` initially failed because `data-sse-url` appears as a string literal inside `FERRO_RUNTIME_JS` (embedded in the HTML). Fixed by checking only the `<body ...>` tag substring, not the full HTML document.

## User Setup Required

None - no external service configuration required. Users call `register_layout("dashboard", DashboardLayout::new(config))` at app startup with their `DashboardLayoutConfig`.

## Next Phase Readiness

- DashboardLayout and FERRO_RUNTIME_JS ready for use in plan 98-03 (additional component rendering improvements)
- Users can register DashboardLayout and set `sse_url` to enable real-time updates via StatCard `data-sse-target` attributes
- Plan 98-04 should document DashboardLayout usage, DashboardLayoutConfig fields, and all JS data attributes

## Self-Check: PASSED

All created files exist on disk. All task commits exist in git log.

---
*Phase: 98-ferro-json-ui-stable-release*
*Completed: 2026-03-11*
