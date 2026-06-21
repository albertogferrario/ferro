---
phase: 238
slug: inertia-first-load-html-shell
status: resolved
source: 238-REVIEW.md
fixed: 4
accepted: 2
fix_commit: e09d71b1
date: 2026-06-21
---

# Phase 238 — Code Review Fix Summary

Findings from `238-REVIEW.md` (0 critical, 4 warnings, 2 info). All warnings fixed
in commit `e09d71b1`; both info items reviewed and accepted as-is.

## Fixed (commit e09d71b1)

| ID | Severity | Finding | Fix |
|----|----------|---------|-----|
| WR-01 | warning | `title`/`mount_id` interpolated raw into `<title>` and `id=""` — a `"`/`<`/`>` in dev config would break HTML structure (not XSS; config-controlled) | Added `escape_html()` helper; applied to `title_text` and `mount_id` in `ferro-inertia/src/response.rs` |
| WR-02 | warning | CSRF token interpolated unescaped into `content="..."` | `escape_html()` applied to `csrf` for both the custom-template and default-template paths |
| WR-03 | warning | `ferro-inertia::Inertia::render` (and siblings) use `InertiaConfig::default()`, silently ignoring the framework's process-global config — a trap for direct embedders | Added rustdoc `# Configuration` note pointing direct embedders at `render_with_config` and clarifying the global lives in the framework wrapper |
| WR-04 | warning | 6 doc examples call nonexistent `SavedInertiaContext::from_request(&req)` — would not compile | Replaced all 6 with the real API `SavedInertiaContext::from(&req)` in `docs/src/features/inertia.md` |

**Regression safety:** `escape_html()` performs the exact same 5-character escaping
(`& < > " '`) previously inlined for `page_json`, so the SC-1
`html_data_page_equals_json_contract` equality test stays green. `head_extras` is
deliberately left raw — it is a documented developer-controlled trust boundary
(`InertiaConfig::head_extras`), never populated from request data.

**Verification:** `cargo fmt --all -- --check` clean · `cargo clippy -p ferro-inertia --all-targets -- -D warnings` clean · `cargo test -p ferro-inertia` 19/19 pass.

## Accepted as-is (info)

| ID | Finding | Rationale |
|----|---------|-----------|
| IN-01 | `from_env()` falls back to `"Ferro"` when `APP_NAME` unset → `<title>Ferro</title>` | Per CLAUDE.md "project-agnostic crates", the crate reads `APP_NAME` and falls back to a generic framework name (not a tenant identity). A neutral default title is acceptable; apps set `APP_NAME` or `.title()`. |
| IN-02 | second-`set_inertia_config` warning uses bare `eprintln!` | Config is set once at bootstrap before logging is necessarily initialized; `eprintln!` to stderr is adequate for a misconfiguration signal. Routing through the logging layer is a possible future refinement, not a defect. |
