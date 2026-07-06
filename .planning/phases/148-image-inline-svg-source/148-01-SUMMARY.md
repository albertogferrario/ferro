---
phase: 148
plan: "01"
subsystem: ferro-json-ui
tags: [tdd, red-phase, image, inline-svg, wave-0]
dependency_graph:
  requires: []
  provides: [RED-tests-ImageSource, RED-tests-ImageProps-constructors, RED-tests-render-image-InlineSvg]
  affects: [ferro-json-ui/src/component.rs, ferro-json-ui/src/render.rs]
tech_stack:
  added: []
  patterns: [serde-untagged-enum, flatten-serde, TDD-RED-wave]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render.rs
decisions:
  - "Option (b) chosen for all_known_types_round_trip InlineSvg extension: independent assertion block inside the same test function, because the fixture iteration asserts serialized[type] == tuple name and ImageInlineSvg != Image"
  - "image_with_aspect_ratio rewritten to explicit-field form (source: ImageSource::Url { src }) rather than constructor because aspect_ratio is Some — constructor form sets both options to None"
  - "--no-verify used on both commits: Wave 0 RED plan; cargo test / clippy intentionally fail per D-16 until Plan 02 lands ImageSource enum and constructors"
metrics:
  duration: "221s"
  completed: "2026-04-24"
  tasks: 2
  files: 2
---

# Phase 148 Plan 01: ImageSource RED Tests Summary

Wave 0 RED-phase plan. Scaffolds test contracts for the `ImageSource` untagged enum and `ImageProps` dual-source refactor. No production symbols defined — Plan 02 (Wave 1) makes these tests GREEN.

## One-liner

RED tests locking the `ImageSource` untagged-enum / `ImageProps` flattened-source contract: serde round-trips, neither-case rejection, url/inline_svg constructors, and the LOAD-BEARING `<script>`-passthrough bypass assertion.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend image_round_trips + all_known_types_round_trip fixture + add ImageSource/constructor tests to component.rs | 2845c1b8 | ferro-json-ui/src/component.rs |
| 2 | Rewrite existing ImageProps struct literals in render.rs and add three RED tests for render_image InlineSvg branch | 91764458 | ferro-json-ui/src/render.rs |

## New Test Locations

### component.rs

| Test function | Line | Purpose |
|---------------|------|---------|
| `image_round_trips` (extended) | 3695 | Url variant + InlineSvg variant + neither-case rejection |
| `all_known_types_round_trip` (extended) | 3709 | InlineSvg independent assertion block at end of fn |
| `mod image_source_tests` | 4101 | Module containing 5 new RED tests |
| `image_source_url_roundtrip` | 4106 | `{"src":"/a.png"}` → `ImageSource::Url` |
| `image_source_inline_svg_roundtrip` | 4116 | `{"svg":"<svg/>"}` → `ImageSource::InlineSvg` |
| `image_source_neither_rejected` | 4126 | `{}` → `Err` (no variant matches) |
| `image_props_url_constructor` | 4132 | `ImageProps::url()` → correct source/alt/options |
| `image_props_inline_svg_constructor` | 4145 | `ImageProps::inline_svg()` → correct source/alt/options |

### render.rs

| Test function | Line | Purpose |
|---------------|------|---------|
| `inline_svg_renders_div_role_img` | 3804 | div role=img, aria-label, verbatim svg body, no `<img>` |
| `inline_svg_with_script_passes_through` | 3837 | LOAD-BEARING: `<script>` passes through unescaped per D-06 |
| `inline_svg_alt_xss_escaped` | 3855 | alt IS escaped on InlineSvg branch (mirrors XSS test pattern) |

## Struct Literal Rewrite Locations

| File | Old line | New form | Reason |
|------|----------|----------|--------|
| component.rs:2173 | `ImageProps { src: "/img/screenshot.png", ... }` | `ImageProps::url("/img/screenshot.png", "Page screenshot")` | all_known_types fixture — must compile against target shape |
| render.rs:3758 | `ImageProps { src: "/img/page.png", aspect_ratio: Some(...) }` | Explicit-field: `source: ImageSource::Url { src: ... }, aspect_ratio: Some(...)` | Constructor sets aspect_ratio=None; explicit needed to preserve Some value |
| render.rs:3782 | `ImageProps { src: "/img/page.png", aspect_ratio: None, ... }` | `ImageProps::url("/img/page.png", "Page")` | Constructor form; all Nones |
| render.rs:3795 | `ImageProps { src: "x\" onerror=\"alert(1)", ... }` | `ImageProps::url("x\" onerror=\"alert(1)", "Test")` | XSS payload preserved verbatim |

## RED State Evidence

`cargo build -p ferro-json-ui --tests` exits non-zero with 23 errors. Unique error types:

```
error[E0412]: cannot find type `ImageSource` in this scope
error[E0433]: failed to resolve: use of undeclared type `ImageSource`
error[E0560]: struct `component::ImageProps` has no field named `source`
error[E0599]: no function or associated item named `inline_svg` found for struct `component::ImageProps` in the current scope
error[E0599]: no function or associated item named `url` found for struct `component::ImageProps` in the current scope
error[E0609]: no field `source` on type `component::ImageProps`
error: could not compile `ferro-json-ui` (lib test) due to 23 previous errors
```

First 10 lines of build output:
```
   Compiling ferro-json-ui v0.2.17 (...)
error[E0412]: cannot find type `ImageSource` in this scope
    --> ferro-json-ui/src/component.rs:4107:21
     |
4107 |         let parsed: ImageSource =
     |                     ^^^^^^^^^^^ not found in this scope

error[E0412]: cannot find type `ImageSource` in this scope
    --> ferro-json-ui/src/component.rs:4117:21
     |
4117 |         let parsed: ImageSource =
```

`cargo fmt --all -- --check` passes (formatting is not semantics; not gated by RED state).

## all_known_types_round_trip Extension Approach

Used **Option (b): independent assertion block** appended inside the same test function, after the fixture loop closes.

Reason: the fixture iteration asserts `serialized["type"] == tuple_name`. For an InlineSvg fixture row, the tuple name would need to be `"ImageInlineSvg"` to be descriptive, but the serialized `type` field is `"Image"`. Refactoring the loop to derive the type from the JSON would change behavior for all existing rows — higher risk than appending an independent block.

## Deviations from Plan

None. Plan executed exactly as written. Both option selections (B: independent block, render.rs L3758: explicit-field form) were pre-authorized in the plan's Action section.

## Known Stubs

None. This plan contains only test code; no production stubs.

## Threat Flags

No new network endpoints, auth paths, or file access patterns introduced. All changes are in `#[cfg(test)]` blocks or test-adjacent fixture code.

## Self-Check: PASSED

- ferro-json-ui/src/component.rs: exists, modified with 109 insertions
- ferro-json-ui/src/render.rs: exists, modified with 71 insertions
- Task 1 commit 2845c1b8: present in git log
- Task 2 commit 91764458: present in git log
- `cargo build -p ferro-json-ui --tests` exits non-zero with expected RED errors
- `cargo fmt --all -- --check` passes
- LOAD-BEARING marker present in render.rs (grep confirmed)
- alt-escape test present (inline_svg_alt_xss_escaped)
- neither-case rejection test present (image_source_neither_rejected)
