---
phase: 238-inertia-first-load-html-shell
plan: 01
subsystem: ferro-inertia
tags: [inertia, config, env, tdd]
requirements: [D-03, D-05, D-07]

dependency_graph:
  requires: []
  provides:
    - InertiaConfig::from_env() constructor
    - InertiaConfig::title/head_extras/mount_id fields and builders
  affects:
    - ferro-inertia/src/config.rs
    - Any downstream code that constructs InertiaConfig via struct literal (none found — all use builder pattern)

tech_stack:
  added: []
  patterns:
    - from_env() + Default::default() delegation (mirrors framework/src/config/providers/app.rs)
    - Consuming builder pattern (mut self -> Self)

key_files:
  modified:
    - ferro-inertia/src/config.rs

decisions:
  - from_env() reads VITE_ENTRY_POINT and INERTIA_VERSION (both were previously hardcoded — OQ-1 and OQ-2 from RESEARCH.md resolved)
  - title: Option<String> kept separate from app_name so app_name retains its role; title provides an explicit override
  - head_extras: Option<String> (raw HTML string, not Vec) — consumers concatenate if needed
  - No APP_URL read into InertiaConfig::from_env() — VITE_DEV_SERVER is the correct field; APP_URL belongs in AppConfig (separate concern)
  - Default::default() delegates to from_env() — zero-change path for existing apps

metrics:
  duration_seconds: 127
  completed_date: "2026-06-21"
  tasks_completed: 2
  files_modified: 1
---

# Phase 238 Plan 01: InertiaConfig from_env() + New Fields Summary

`InertiaConfig::from_env()` constructor added, extracting and extending the env-reading logic previously embedded in `Default::default()`; three new configurable fields (`title`, `head_extras`, `mount_id`) with consuming builders added.

## What Was Built

### `ferro-inertia/src/config.rs` (commit `2721a6e1`)

**New constructor:** `InertiaConfig::from_env()` reads five env vars:
- `APP_NAME` → `app_name` (was already read in `Default`)
- `VITE_DEV_SERVER` → `vite_dev_server` (was already read in `Default`)
- `VITE_ENTRY_POINT` → `entry_point` (was hardcoded `"src/main.tsx"` — gap fixed)
- `INERTIA_VERSION` → `version` (was hardcoded `"1.0"` — gap fixed)
- `APP_ENV` → `development` flag (was already read in `Default`)

**Default delegation:** `impl Default for InertiaConfig { fn default() -> Self { Self::from_env() } }` — existing apps keep working with zero changes.

**Three new fields on the struct:**
- `pub title: Option<String>` — optional page title override; when `Some`, overrides `app_name` in `<title>`
- `pub head_extras: Option<String>` — raw HTML injected into `<head>` (SECURITY: developer-controlled config only); ignored when `html_template` is set
- `pub mount_id: String` — id of the mount node, defaults to `"app"`

**Three new consuming builders** (matching existing `pub fn NAME(mut self, ...) -> Self` shape):
- `.title(t)` → sets `title = Some(t.into())`
- `.head_extras(h)` → sets `head_extras = Some(h.into())`
- `.mount_id(id)` → sets `mount_id = id.into()`

**Four unit tests** (TDD — tests written first, then implementation):
- `from_env_reads_defaults` — verifies defaults when CI env vars are unset
- `new_fields_default_to_none` — verifies `title` and `head_extras` are `None` by default
- `builders_set_new_fields` — verifies all three builders set their respective fields
- `default_equals_from_env_shape` — verifies `Default::default()` delegates to `from_env()`

**No env mutation in tests** (`grep -c "set_var"` = 0).

## TDD Gate Compliance

- RED commit: tests added first, compilation failed with `E0599`/`E0609` errors confirming tests were genuinely failing
- GREEN commit: `2721a6e1` — all 11 unit tests + 3 doc-tests pass
- REFACTOR: not needed (code was clean on first pass, fmt applied in place)

## `cargo build -p ferro-inertia` Result

**Exit 0 — no response.rs struct-construction errors.** The existing `response.rs` does not construct `InertiaConfig` via struct literal; it calls `InertiaConfig::default()` or receives a config via function arguments. The new fields initialize correctly via `from_env()` (title=None, head_extras=None, mount_id="app"). The Wave 1→2 hand-off note in the plan was precautionary; in practice `response.rs` does not break.

Plan 02 (response template extension) will consume the new fields to inject `title_text`, `head_extras`, and `mount_id` into the HTML output.

## Verification Results

All acceptance criteria met:
- `grep -n "pub fn from_env" ferro-inertia/src/config.rs` → line 54
- `grep -n "pub title: Option<String>"` → line 40
- `grep -n "pub head_extras: Option<String>"` → line 44
- `grep -n "pub mount_id: String"` → line 46
- `grep -n "VITE_ENTRY_POINT"` → lines 52, 68, 189
- `grep -n "INERTIA_VERSION"` → lines 52, 70, 189
- `grep -n "fn default"` at line 178: `fn default() -> Self { Self::from_env() }`
- `grep -c "set_var"` → 0
- `cargo test -p ferro-inertia` → 11/11 unit + 3/3 doc-tests pass
- `cargo clippy -p ferro-inertia --all-targets -- -D warnings` → clean
- `cargo fmt --all -- --check` → clean (after fmt applied)
- `cargo build -p ferro-inertia` → exit 0

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The new fields are wired: `from_env()` initializes them to their documented defaults; builders set them. Template consumption of `title`/`head_extras`/`mount_id` is Plan 02's scope (response.rs extension), not a stub in this plan.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. `head_extras` carries raw HTML but is config-only (developer-controlled); the injection site (response.rs) is Plan 02's scope. T-238-01 mitigation (field doc comment marking developer-controlled) is in place.

## Self-Check: PASSED
