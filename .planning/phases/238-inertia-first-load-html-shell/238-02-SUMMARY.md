---
phase: 238-inertia-first-load-html-shell
plan: 02
subsystem: ferro-inertia
tags: [inertia, html, content-negotiation, response, tdd]
requirements: [D-01, D-05, D-06, D-07, D-12]

dependency_graph:
  requires:
    - 238-01 (InertiaConfig::title / head_extras / mount_id fields and builders)
  provides:
    - to_html_response honoring title/head_extras/mount_id in dev and prod branches
    - content_negotiation_tests: 8 tests proving SC-1/SC-2/SC-3-dev/SC-4/T-238-03
  affects:
    - ferro-inertia/src/response.rs

tech_stack:
  added: []
  patterns:
    - TDD RED/GREEN for both template extension and test module
    - development:true for all HTML-structure tests (D-09 OnceLock isolation)
    - MockReq implementing InertiaRequest (inertia_header + path only — 2 required methods)

key_files:
  modified:
    - ferro-inertia/src/response.rs

decisions:
  - title_text / head_extras / mount_id computed once before the dev/prod branch (shared locals, not per-branch duplication)
  - dev branch uses positional format args; head_extras injected as {} before </head>; mount_id and page_json as {} {} on the div
  - prod branch uses named format args; app_name arg removed; title_text/head_extras/mount_id added
  - head_extras injects a leading whitespace line (indented) — cosmetic, does not affect parsing
  - custom-template early-return left unchanged (D-06); it processes {page}/{csrf} only
  - prod-leak security test (T-238-03) uses production() branch with absent-assertion so OnceLock bleed is harmless
  - prod manifest resolution (SC-3 prod) covered by existing manifest.rs::parse_manifest_and_resolve_entry; not duplicated here

metrics:
  duration_seconds: 250
  completed_date: "2026-06-21"
  tasks_completed: 2
  files_modified: 1
---

# Phase 238 Plan 02: HTML Template Extension + Content-Negotiation Tests Summary

Extended the existing `to_html_response` dev/prod templates to honor `title`/`head_extras`/`mount_id` from `InertiaConfig`; added an 8-test `content_negotiation_tests` module proving SC-1/SC-2/SC-3-dev/SC-4 and the T-238-03 security invariant.

## What Was Built

### `ferro-inertia/src/response.rs` — Task 1 (commit `e3027945`)

**Template extension in `to_html_response`:** After the custom-template early-return (unchanged), computed three shared locals:

```rust
let title_text = self.config.title.as_deref().unwrap_or(&self.config.app_name);
let head_extras = self.config.head_extras.as_deref().unwrap_or("");
let mount_id = self.config.mount_id.as_str();
```

**Dev branch changes:**
- `{}` for title now receives `title_text` instead of `self.config.app_name`
- New `{}` line before `</head>` receives `head_extras` (empty string when None)
- `<div id="app" data-page="{}">` → `<div id="{}" data-page="{}">` receiving `mount_id, page_json`
- Format arg count/order updated: `csrf, title_text, vite_dev_server×3, entry_point, head_extras, mount_id, page_json`

**Prod branch changes (named args):**
- `{app_name}` → `{title_text}` in `<title>` tag; `app_name = self.config.app_name` arg removed
- New `{head_extras}` line before `</head>`; named arg `head_extras = head_extras` added
- `id="app"` → `id="{mount_id}"`; named arg `mount_id = mount_id` added

**Preserved unchanged (D-06/D-07):**
- Custom-template early-return at `:394-399` — `{page}` / `{csrf}` replacement only
- Data-page escaping block at `:383-389` — `& < > " '` encoding intact (`&#x27;` still present)

### `ferro-inertia/src/response.rs` — Task 2 (commit `612853f0`)

**`#[cfg(test)] mod content_negotiation_tests`** added at the bottom of the file (484 lines). Contains a `MockReq` struct implementing `InertiaRequest` via the two required methods (`inertia_header` + `path`).

**Eight tests:**

| Test | Proves |
|------|--------|
| `non_inertia_request_returns_html_document` | content_type = text/html, body has DOCTYPE + data-page (SC-2) |
| `inertia_request_returns_json_contract` | content_type = application/json, body parses with component (SC-2) |
| `html_data_page_equals_json_contract` | unescaped data-page JSON == JSON-path body parsed equal (SC-1/D-12) |
| `dev_mode_emits_vite_client_script` | HTML contains `{vite_dev_server}/@vite/client` (SC-3 dev) |
| `title_override` | `<title>Explicit</title>` present; fallback `<title>Fallback</title>` absent (SC-4) |
| `head_extras_in_html` | raw meta tag string appears in body (SC-4) |
| `mount_id_applied` | `id="root" data-page=` present (SC-4) |
| `prod_mode_does_not_leak_dev_server` | prod body contains neither `/@vite/client` nor `@react-refresh` (T-238-03) |

All 8 tests use `development()` except `prod_mode_does_not_leak_dev_server` which uses `production()`. No test calls `resolve_assets()` directly for structural assertions (D-09 isolation).

## TDD Gate Compliance

**Task 1:**
- RED: `template_field_tests` (title_override / head_extras_in_html / mount_id_applied) — 3/3 FAILED before template change (confirmed panics at assertion lines)
- GREEN: `e3027945` — 3/3 pass after template extension
- REFACTOR: not needed

**Task 2:**
- Tests added in `content_negotiation_tests` supersede `template_field_tests` (which was the Task 1 RED scaffold). Tests were structurally new — the Task 1 template changes already provided GREEN for the three overlapping tests; the 5 new tests (non_inertia, inertia, html_data_page_equals, dev_mode, prod_mode_does_not_leak) were GREEN immediately because content-negotiation already existed and the template changes from Task 1 were in place.

## Verification Results

All acceptance criteria met:

- `grep -n "self.config.mount_id" ferro-inertia/src/response.rs` → line 405
- `grep -n "title_text" ferro-inertia/src/response.rs` → lines 403, 433, 459, 469 (both branches)
- `grep -n "head_extras" ferro-inertia/src/response.rs` → lines 404, 438, 462, 470 + test lines
- `grep -c "&#x27;" ferro-inertia/src/response.rs` → 1 (escaping block unchanged, D-07)
- `grep -n 'replace("{page}"' ferro-inertia/src/response.rs` → line 396 (custom-template path intact, D-06)
- `grep -n "mod content_negotiation_tests" ferro-inertia/src/response.rs` → line 484
- `cargo test -p ferro-inertia` → 19/19 unit + 3/3 doc-tests pass
- `cargo clippy --all --all-targets -- -D warnings` → clean
- `cargo fmt --all -- --check` → clean

**SC-1:** proven by `html_data_page_equals_json_contract`
**SC-2:** proven by `non_inertia_request_returns_html_document` + `inertia_request_returns_json_contract`
**SC-3 dev:** proven by `dev_mode_emits_vite_client_script`
**SC-3 prod:** covered by existing `manifest.rs::tests::parse_manifest_and_resolve_entry` (not duplicated)
**SC-4:** proven by `title_override` + `head_extras_in_html` + `mount_id_applied`
**T-238-03:** proven by `prod_mode_does_not_leak_dev_server`

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All three template fields (`title`, `head_extras`, `mount_id`) are fully wired: config fields set, defaults applied, templates consume them in both dev and prod branches. Tests prove round-trip correctness.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The `head_extras` injection path (T-238-02) is in place as designed — developer-controlled config only, not reachable from request data.

## Self-Check: PASSED

- `ferro-inertia/src/response.rs` exists and modified: FOUND
- Commit `e3027945` exists: FOUND (`git log --oneline | grep e3027945`)
- Commit `612853f0` exists: FOUND
- 19 unit tests pass: CONFIRMED
- clippy clean: CONFIRMED
- fmt clean: CONFIRMED
