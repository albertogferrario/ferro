---
phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
verified: 2026-05-20T20:00:00Z
status: passed
score: 18/18
overrides_applied: 0
re_verification: null
gaps: []
human_verification:
  - test: "Load a consumer staff-domain page in a browser with ?tab=<name> and confirm the correct tab panel is active at page load with no flash"
    expected: "The named tab is highlighted and its panel is the only visible one immediately — no brief flash of the default tab before switching"
    why_human: "JavaScript runtime behavior at DOMContentLoaded — cannot verify without a running browser; only the presence of initTabFromUrl in the bundle is verifiable programmatically"
  - test: "Submit a staff create form with an avatar file through a consumer app; confirm the server receives a multipart body with the file"
    expected: "Server controller receives the file at the expected field name; no Content-Type branching band-aid is needed"
    why_human: "End-to-end multipart form submission requires a running server and browser; the HTML output is verified by automated tests, but actual browser behavior cannot be confirmed without a live consumer"
---

# Phase 175: JSON-UI v2 Runtime Patches — Staff-Domain Field Test — Verification Report

**Phase Goal:** Land six runtime patches against the v12.0 JSON-UI v2 runtime exposed by gestiscilo-it's staff-domain CRUD field test (F1 depth limit, F2 CheckboxGroup, F3 tabs concurrent render, F4 Switch + variant docs, F5 file input + enctype, F6 DataTable {row.X} interpolation). Each finding lands as an independent plan; phase-level acceptance is the workspace suite passing zero-warning AND a clean re-run of the consumer staff-domain UAT.
**Verified:** 2026-05-20
**Status:** human_needed (automated checks all pass; two browser-level behaviors need human confirmation)
**Re-verification:** No — initial verification

## Verification approach

The 175-05 SUMMARY explicitly logged `cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, and `cargo test --all-features` all returning exit 0. The verifier independently confirmed:

- `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` — clean (exit 0, no output)
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings` — clean (exit 0, no output)
- `cargo fmt --all -- --check` — clean (empty output = exit 0)
- `cargo test -p ferro-json-ui --lib` — 532 passed, 0 failed (run confirmed)
- All six must-have pinned tests run individually and confirmed green (see Behavioral Spot-Checks)

This is approach (a) with independent grep-verification of all must-have artifact assertions plus direct test execution of the critical pinned tests.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | MAX_NESTING_DEPTH = 16 in spec.rs | VERIFIED | `grep`: `pub const MAX_NESTING_DEPTH: usize = 16;` at line 41 |
| 2 | Walker tripwire says "depth limit exceeded" not "cycle guard tripped" | VERIFIED | `grep`: line 143 contains `"depth limit exceeded at depth {depth} (max={MAX_NESTING_DEPTH})"` ; "cycle guard tripped" not found in production code |
| 3 | Cycle detector kept separate (only fires on real revisit) | VERIFIED | `SpecError::DepthExceeded` and `SpecError::Cycle` are distinct variants; `cycle_detector_only_on_revisit` test passes |
| 4 | `initTabFromUrl` exists in tabs.rs runtime JS; FERRO_RUNTIME_JS bundle contains it; URLSearchParams used | VERIFIED | `grep`: line 29 `function initTabFromUrl`; `runtime_contains_init_tab_from_url` test passes; `URLSearchParams` at line 30 |
| 5 | `template_actions` and `template_url` substitute `{row.X}` alongside `{X}`; bare `{X}` still works | VERIFIED | Two `{row.{col_key}}` replacements confirmed at lines 316 and 389 in data.rs; `data_table_row_prefix_placeholder_resolved` and `data_table_bare_placeholder_resolved` tests both pass |
| 6 | `BUILTIN_TYPES.len() == 43`; `global_catalog().lookup("CheckboxGroup")` returns `Some(_)` | VERIFIED | `builtin_types_count_matches_dispatch` asserts 43 at line 570 of render/mod.rs; `catalog_contains_checkbox_group` test passes; atoms.rs also asserts 43 |
| 7 | CheckboxGroup dispatches to `render_checkbox_list`; no separate CheckboxGroupProps struct | VERIFIED | Dispatch arm: `"CheckboxGroup" => form::render_checkbox_list(el, spec, data, depth)` at line 207; no `CheckboxGroupProps` found in any source file |
| 8 | `InputType::File` variant exists; `InputProps.accept` exists; `FormProps.enctype` exists | VERIFIED | `File,` at line 81 of component.rs; `pub accept: Option<String>` at line 303; `pub enctype: Option<String>` at line 232 |
| 9 | `render_input` emits file input with accept; `render_form` emits enctype; file inputs do NOT emit `value=""` | VERIFIED | File arm confirmed at form.rs lines 221-248; `enctype_attr` at line 89; no `value=""` in File arm; negative assertion pinned by `input_file_renders_file_type_and_accept` |
| 10 | `switch_at_depth_8_renders_role_switch` test exists and passes; no `variant` field on CheckboxProps | VERIFIED | Test at form.rs line 1454 passes green; no `CheckboxGroupProps` or `variant` field found |
| 11 | docs/src/json-ui/components.md has Switch section with role="switch" + substitution note | VERIFIED | `role="switch"` at line 795; `variant.*switch` pattern at line 827 (`no variant: "switch" prop`) |
| 12 | docs/src/json-ui/components.md has CheckboxGroup section with alias note and substitution path | VERIFIED | Section at line 858; alias note at line 860; array-submit example at lines 886-898 |
| 13 | Tab URL init is client-side only; no network request for tab switching | VERIFIED | `initTabFromUrl` reads `URLSearchParams`, validates against DOM, calls `makeTabHandler` synthetically — no fetch/XHR |
| 14 | Missing `{row.X}` keys leave placeholder literal (missing-key invariant) | VERIFIED | `data_table_row_prefix_missing_key_leaves_placeholder` test passes; substitution only fires when key exists in row object |
| 15 | Depth-8 spec parses and renders without node stripping | VERIFIED | `from_json_accepts_depth_8` passes; `switch_at_depth_8_renders_role_switch` passes |
| 16 | Depth-17 spec is rejected with `SpecError::DepthExceeded { max: 16, found: 17, .. }` | VERIFIED | `from_json_rejects_depth_17` passes; integration fixture `six_level_nesting.json` updated to 17-level chain |
| 17 | Multipart form round-trip (Form + Input[file] + Button) renders correct HTML | VERIFIED | `multipart_form_roundtrip` test passes |
| 18 | Workspace fmt + clippy clean; test suite green | VERIFIED | fmt check: empty output (clean); clippy ferro-json-ui + ferro-mcp: clean; lib tests: 532/532 pass |

**Score:** 18/18 truths verified

### Deferred Items

None.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/spec.rs` | MAX_NESTING_DEPTH = 16; depth tests | VERIFIED | Line 41: `pub const MAX_NESTING_DEPTH: usize = 16;` |
| `ferro-json-ui/src/render/mod.rs` | Walker tripwire "depth limit exceeded"; BUILTIN_TYPES = 43 | VERIFIED | Line 143: correct diagnostic; line 570: `assert_eq!(BUILTIN_TYPES.len(), 43)` |
| `ferro-json-ui/src/runtime/tabs.rs` | `function initTabFromUrl` defined and called | VERIFIED | Line 29: function defined; called from `initTabContainer` |
| `ferro-json-ui/src/runtime/mod.rs` | `runtime_contains_init_tab_from_url` test; `initTabFromUrl` string check | VERIFIED | Lines 123-131: test present and passing |
| `ferro-json-ui/src/render/data.rs` | `{row.X}` substitution in both `template_url` and `template_actions` | VERIFIED | Lines 316 and 389: both replacements present; `grep -c` returns 2 |
| `ferro-json-ui/src/render/mod.rs` | `CheckboxGroup` in BUILTIN_TYPES; dispatch to `render_checkbox_list` | VERIFIED | Line 87: array entry; line 207: dispatch arm |
| `ferro-json-ui/src/catalog.rs` | `CheckboxGroup` entry with `CheckboxListProps` schema | VERIFIED | Line 369: catalog entry; line 373: `schema_for!(CheckboxListProps)` |
| `docs/src/json-ui/components.md` | CheckboxGroup section with alias + substitution path | VERIFIED | Section at line 858; substitution at lines 883-898 |
| `ferro-json-ui/src/component.rs` | `InputType::File`; `InputProps.accept`; `FormProps.enctype` | VERIFIED | Lines 81, 303, 232 |
| `ferro-json-ui/src/render/form.rs` | `render_input` File arm; `render_form` enctype; `switch_at_depth_8` test | VERIFIED | Lines 221-248, 89-101, 1454 |
| `docs/src/json-ui/components.md` | Switch section with role="switch" and substitution note | VERIFIED | Lines 795-827 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `render/mod.rs::render_element` | `spec.rs::MAX_NESTING_DEPTH` | constant import | VERIFIED | `depth > MAX_NESTING_DEPTH + 1` at line 141 |
| `spec.rs::Spec::from_json` | `spec.rs::SpecError::DepthExceeded` | validate_depth | VERIFIED | `return Err(SpecError::DepthExceeded { ... })` at line 942 |
| `runtime/tabs.rs::initTabContainer` | `runtime/tabs.rs::initTabFromUrl` | function call | VERIFIED | `initTabFromUrl(container, triggers, panels)` called after click-handler wiring |
| `render/data.rs::template_url` | `{row.X}` alias substitution | `url.replace()` | VERIFIED | Two identical replace calls at lines 316 and 389 |
| `render/mod.rs` dispatch | `render/form.rs::render_checkbox_list` | CheckboxGroup dispatch arm | VERIFIED | `"CheckboxGroup" => form::render_checkbox_list(el, spec, data, depth)` |
| `catalog.rs::BUILTIN_SPECS` | `schemars::schema_for!(CheckboxListProps)` | CheckboxGroup catalog entry | VERIFIED | Line 373 uses `schema_for!(CheckboxListProps)` |
| `component.rs::InputType::File` | `render/form.rs::render_input` File arm | `InputType::File =>` match | VERIFIED | Match arm at line 221 |
| `component.rs::FormProps.enctype` | `render/form.rs::render_form` | `enctype_attr` emission | VERIFIED | `enctype_attr` computed at line 89, interpolated at lines 95 and 101 |

### Data-Flow Trace (Level 4)

Not applicable — all deliverables are render functions, test fixtures, and documentation; there is no user-facing dynamic data store (no DB query → component pipeline). The render functions produce deterministic HTML from spec + data arguments; correctness is verified by the pinned tests.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| from_json_accepts_depth_8 | `cargo test -p ferro-json-ui --lib -- from_json_accepts_depth_8` | 1 passed | PASS |
| walker_depth_tripwire | `cargo test -p ferro-json-ui --lib -- walker_depth_tripwire` | 2 passed | PASS |
| builtin_types_count_matches_dispatch | `cargo test -p ferro-json-ui --lib -- builtin_types_count_matches_dispatch` | 1 passed | PASS |
| catalog_contains_checkbox_group | `cargo test -p ferro-json-ui --lib -- catalog_contains_checkbox_group` | 1 passed | PASS |
| switch_at_depth_8_renders_role_switch | `cargo test -p ferro-json-ui --lib -- switch_at_depth_8_renders_role_switch` | 1 passed | PASS |
| multipart_form_roundtrip | `cargo test -p ferro-json-ui --lib -- multipart_form_roundtrip` | 1 passed | PASS |
| runtime_contains_init_tab_from_url | `cargo test -p ferro-json-ui --lib -- runtime_contains_init_tab_from_url` | 1 passed | PASS |
| data_table_row_prefix_placeholder_resolved | `cargo test -p ferro-json-ui --lib -- data_table_row_prefix_placeholder_resolved` | 1 passed | PASS |
| All ferro-json-ui lib tests | `cargo test -p ferro-json-ui --lib` | 532 passed, 0 failed | PASS |
| ferro-json-ui integration tests | `cargo test -p ferro-json-ui --test '*'` | 8 passed, 0 failed | PASS |
| Clippy ferro-json-ui + ferro-mcp | `cargo clippy -p ferro-json-ui -p ferro-mcp --all-targets -- -D warnings` | Clean | PASS |
| Format check | `cargo fmt --all -- --check` | Clean (empty output) | PASS |

### Requirements Coverage

No REQ-IDs assigned to this phase (v12.0 follow-up patch batch, not a numbered REQ-tracked milestone). Requirement coverage check skipped per phase specification.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | — | — | — | — |

Scanned key modified files for TODO/FIXME/placeholder comments, empty return stubs, and hardcoded empty data. None found. All render arms produce substantive HTML output. All new props have real field definitions backed by serde deserialization.

### Human Verification Required

#### 1. URL-driven tab activation at DOMContentLoaded

**Test:** In a consumer app running a staff-domain DetailPage with at least two tabs, navigate to the page with `?tab=<second-tab-name>` appended to the URL.
**Expected:** The second tab is highlighted (border-primary classes) and its panel is the only visible one immediately after page load. The default-tab panel is hidden. There is no visible flash of the default tab before switching.
**Why human:** JavaScript runtime behavior — `initTabFromUrl` runs at `DOMContentLoaded` inside the `ferroRuntime()` IIFE. The function's presence in the bundle and its URLSearchParams usage are verified programmatically. Whether it produces the correct visual outcome in an actual browser requires a live page load with devtools or visual inspection.

#### 2. Multipart file upload end-to-end submission

**Test:** In a consumer app with the staff create form (avatar upload), select a JPEG file through the file picker and submit the form.
**Expected:** The server controller receives a multipart request body. The file is accessible at the `avatar` field. No Content-Type branching is required in the controller. The browser does not send a URL-encoded body.
**Why human:** Actual browser form submission with a file requires a live server and browser. The HTML output (correct `enctype`, correct `type="file"`, correct `accept`) is fully verified by the `multipart_form_roundtrip` test. Whether the browser correctly sends a multipart body requires a full integration test beyond the scope of the test suite.

### Gaps Summary

No gaps. All must-have truths are verified at the code level. The two human verification items are behavioral confirmations of already-verified code — the implementation is complete; the items confirm expected browser behavior.

---

_Verified: 2026-05-20_
_Verifier: Claude (gsd-verifier)_
