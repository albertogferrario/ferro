---
phase: 112-agent-first-philosophy
plan: 02
subsystem: documentation
tags: [mcp-tools, documentation, discoverability, agent-first]
dependency_graph:
  requires: []
  provides: [mcp-tool-discoverability-via-feature-docs]
  affects: [docs/src/features]
tech_stack:
  added: []
  patterns: [mcp-tools-section-format]
key_files:
  created: []
  modified:
    - docs/src/features/events.md
    - docs/src/features/queues.md
    - docs/src/features/notifications.md
    - docs/src/features/broadcasting.md
    - docs/src/features/storage.md
    - docs/src/features/caching.md
    - docs/src/features/authentication.md
    - docs/src/features/api-resources.md
    - docs/src/features/rate-limiting.md
    - docs/src/features/database.md
    - docs/src/features/derive-macros.md
    - docs/src/features/validation.md
    - docs/src/features/localization.md
    - docs/src/features/inertia.md
    - docs/src/features/json-ui.md
    - docs/src/features/projections.md
    - docs/src/features/stripe.md
    - docs/src/features/api.md
    - docs/src/features/whatsapp.md
decisions:
  - Stripe section documents all 3 tools found in ferro-mcp/src/tools/stripe.rs (stripe_config_status, stripe_webhook_events, stripe_subscription_info)
  - generate_types is a real MCP tool (not CLI-only) — confirmed from ferro-mcp/src/tools/generate_types.rs
  - Detail level varies by tool count: rich (5+ tools with bullets), medium (2-3 tools with subsections), one-liner (1 obvious tool)
metrics:
  duration: 248s
  completed: "2026-03-26"
  tasks_completed: 2
  files_modified: 19
---

# Phase 112 Plan 02: MCP Tools Documentation Coverage Summary

Added `## MCP Tools` sections to all feature documentation pages that have relevant ferro-mcp tools, and standardized existing MCP sections to use the new heading pattern.

## What Was Built

Every feature page with relevant MCP tools now ends with a `## MCP Tools` section that names each tool, describes what it returns, and explains when to use it. Agents reading feature docs can now discover the MCP tools available for each feature without consulting api-mcp.md separately.

**17 pages received new `## MCP Tools` sections:**
- events.md — `list_events` (one-liner)
- queues.md — `list_jobs`, `job_history`, `queue_status` (medium)
- notifications.md — `code_templates` (one-liner)
- broadcasting.md — `list_broadcast_channels` (one-liner)
- storage.md — `code_templates` (one-liner)
- caching.md — `cache_inspect` (one-liner)
- authentication.md — `list_policies`, `session_inspect` (medium)
- api-resources.md — `list_resources` (one-liner)
- rate-limiting.md — `list_rate_limiters` (one-liner)
- database.md — 7 tools (rich section with bullets)
- derive-macros.md — `explain_model` (one-liner, cross-references database.md)
- validation.md — `code_templates` (one-liner)
- localization.md — `list_lang_files` (one-liner)
- inertia.md — `inspect_props`, `list_props`, `generate_types` (medium/rich)
- json-ui.md — `json_ui_catalog`, `json_ui_inspect`, `json_ui_generate` (rich)
- projections.md — 5 tools (rich section with bullets)
- stripe.md — `stripe_config_status`, `stripe_webhook_events`, `stripe_subscription_info` (medium)

**3 pages standardized:**
- api.md — `## MCP Integration` → `## MCP Tools`, subsections reformatted with backtick tool names
- whatsapp.md — `## MCP Introspection` → `## MCP Tools`, content rewritten to match standard format
- ai.md — already correct (`## MCP Tools`), no changes needed

**4 pages correctly skipped** (no relevant MCP tools): multi-tenancy.md, testing.md, static-files.md, themes.md

## Verification Results

- `grep -rl "## MCP Tools" docs/src/features/ | wc -l` → **20** (requirement: ≥ 19)
- `grep -l "## MCP Integration\|## MCP Introspection" docs/src/features/` → **None** (all standardized)
- All 4 skipped pages confirmed MCP-free
- `mdbook build docs/` → success, no errors

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

- All 19 modified doc files exist on disk: PASSED
- Task 1 commit (b4be7e15) exists: PASSED
- Task 2 commit (bdd19c62) exists: PASSED
