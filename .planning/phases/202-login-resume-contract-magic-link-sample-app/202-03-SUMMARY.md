---
phase: 202-login-resume-contract-magic-link-sample-app
plan: "03"
subsystem: ui
tags: [magic-link, passwordless, json-ui, auth, ferro-json-ui, views]
dependency_graph:
  requires:
    - 202-02 (magic-link handlers; login.json email-only form; login_confirm.json stub)
  provides:
    - app/src/views/login_confirm.json reconciled: element-level visible + action replacing invalid Button props
    - login_view_is_valid_and_posts_to_login: submit label assertion added
  affects:
    - SC-4 (both auth views confirmed valid ferro-json-ui/v2 with layout:auth)
tech-stack:
  added: []
  patterns:
    - "ferro-json-ui visibility gate: element-level visible with {path, operator:is_true} — NOT a Button prop"
    - "ferro-json-ui href navigation: element-level action with {handler:{$data:...}, method:GET} — NOT a Button prop"
    - "ActionHandler::Binding resolves $data path against spec.data at render time (resolve_actions pipeline step)"

key-files:
  created: []
  modified:
    - app/src/views/login_confirm.json
    - app/src/controllers/auth_controller.rs

key-decisions:
  - "login_confirm.json dev_link fallback: Button.props has no href/visible fields — moved to element-level action + visible with proper Visibility{is_true} condition and ActionHandler::Binding($data)"
  - "login.json was already fully compliant with UI-SPEC (Plan 02 authored it correctly); no change needed"
  - "dev link security: element-level visible:{path:/dev_mode,operator:is_true} renders no HTML for the button when dev_mode=false — URL never reaches production HTML"

patterns-established:
  - "Element-level vs prop-level distinction: visible and action are spec.Element fields, never ButtonProps fields"
  - "IsTrue operator for boolean dev-mode gates over Eq{value:true} — handles missing path cleanly"

requirements-completed: [SC-4]

duration: ~8min
completed: "2026-06-11"
---

# Phase 202 Plan 03: Auth view reconciliation against UI-SPEC contract Summary

**login_confirm.json corrected: dev_link Button visibility and navigation moved from unsupported Button props to element-level visible (is_true condition) + action (ActionHandler::Binding) so the verify URL is never in production HTML.**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-06-11T05:18:00Z
- **Completed:** 2026-06-11T05:26:30Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Discovered that `ButtonProps` has no `href` or `visible` fields — the UI-SPEC's primary form (Button props) was architecturally incorrect for the framework.
- Applied UI-SPEC documented fallback: moved `visible` to element-level with proper `Visibility{path, operator:is_true}` condition; moved `href` to element-level `action` with `ActionHandler::Binding{$data:/dev_link}` and `method:GET`.
- Confirmed `login.json` was already fully compliant with UI-SPEC (Plan 02 authored it correctly).
- Added the missing submit label assertion (`"Send login link"`) to `login_view_is_valid_and_posts_to_login` test.

## Task Commits

1. **Tasks 1+2: Reconcile login_confirm.json + add submit label assertion** - `8527cb6a` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `app/src/views/login_confirm.json` — dev_link element corrected: removed `href`/`visible` from Button props; added element-level `action` (ActionHandler::Binding) and `visible` (is_true condition)
- `app/src/controllers/auth_controller.rs` — added `submit.label == "Send login link"` assertion to `login_view_is_valid_and_posts_to_login` test

## Decisions Made

**UI-SPEC primary mechanism vs. framework reality:**

The UI-SPEC's View 2 specified `"visible": {"$data": "/dev_mode"}` and `"href": {"$data": "/dev_link"}` as Button props. Inspection of `ButtonProps` in `ferro-json-ui/src/component.rs` shows neither field exists on the struct — they would be silently dropped by serde (no `deny_unknown_fields`), resulting in the dev link always being visible and not navigating anywhere.

The UI-SPEC documented a fallback: "If Button does not support `visible`, use two separate files or an `$if` wrapper element." Applied a better option that the framework actually supports:

- `visible` → element-level field with `{"path": "/dev_mode", "operator": "is_true"}` — evaluated by `Visibility::evaluate()` against merged spec data; renders no HTML when false (T-202-DEVLEAK mitigation confirmed)
- `href` → element-level `action` with `{"handler": {"$data": "/dev_link"}, "method": "GET"}` — resolved by `resolve_actions` via `ActionHandler::Binding`, renders the button wrapped in `<a href="...">` for GET navigation

The expression resolver comment (`el.visible` is untouched) was the key signal: `$data` bindings only work in `el.props`, not in the element-level `visible` field.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] login_confirm.json: href and visible placed in Button props where they have no effect**
- **Found during:** Task 1 — verifying Button component support per plan instructions
- **Issue:** `ButtonProps` struct has no `href` or `visible` fields. The UI-SPEC primary form would render the dev link always-visible (no `visible` gate) and non-navigating (href silently dropped). The raw verify URL would appear in production HTML, violating T-202-DEVLEAK.
- **Fix:** Moved `visible` to element-level with `{"path": "/dev_mode", "operator": "is_true"}`. Moved `href` to element-level `action` with `ActionHandler::Binding`. Removed both from Button `props`. Kept `label: {"$data": "/dev_link_label"}` in props (expression resolver handles it before decode_props runs).
- **Files modified:** `app/src/views/login_confirm.json`
- **Verification:** `cargo test -p app` green; `! grep visible.*\$data login_confirm.json` passes; element-level visible uses proper Visibility condition
- **Committed in:** 8527cb6a

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug in view file)
**Impact on plan:** Essential correctness fix. Without it, T-202-DEVLEAK mitigation would not function and the dev link would always render in production HTML.

## Dev-link Render Mechanism (UI-SPEC primary vs. fallback)

The UI-SPEC primary form (`"visible": {"$data": "/dev_mode"}` in Button props) is **not supported** by ferro-json-ui — `ButtonProps` has no such field.

**Mechanism implemented:** Element-level `visible` + element-level `action`:

```json
"dev_link": {
  "type": "Button",
  "props": {
    "label": { "$data": "/dev_link_label" },
    "variant": "outline"
  },
  "action": {
    "handler": { "$data": "/dev_link" },
    "method": "GET"
  },
  "visible": {
    "path": "/dev_mode",
    "operator": "is_true"
  }
}
```

Security guarantee: when `dev_mode: false`, `Visibility::evaluate()` returns false and the renderer emits an empty string for the element — no HTML, no anchor, no URL. The verify URL does not appear in production HTML (T-202-DEVLEAK mitigated).

## Known Stubs

None. Both view files are fully wired to the handler data contract from Plan 02.

## Threat Surface Scan

No new network endpoints or trust boundaries introduced in this plan. The fix to `login_confirm.json` strengthens the T-202-DEVLEAK mitigation — moving from a silently-ignored prop (no protection) to a functioning element-level visibility gate (URL absent from HTML when dev_mode=false).

## Self-Check: PASSED

- `app/src/views/login.json`: EXISTS
- `app/src/views/login_confirm.json`: EXISTS
- Commit 8527cb6a: FOUND in git log
- `cargo test -p app login_view`: 1 passed, 0 failed
- `cargo test -p app`: 14 passed, 0 failed
- `cargo clippy -p app --all-targets -- -D warnings`: clean
- `cargo fmt --all -- --check`: clean
- No password field in login.json: CONFIRMED
- Element-level visible with is_true in login_confirm.json: CONFIRMED
- Submit label assertion in test: CONFIRMED
