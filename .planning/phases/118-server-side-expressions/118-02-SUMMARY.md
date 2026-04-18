---
phase: 118-server-side-expressions
plan: 02
status: complete
---

## Plan 118-02: Wire expression resolver into the JsonUi pipeline

Delivers EXPR-03. Expression resolution is now live on every public `JsonUi::render*` path; the Phase 118 primitive is no longer dead code from the framework's perspective. Pitfall 3 (forgetting `resolve_with_errors`) is closed — both internal methods call `resolve_expressions`.

## Files modified

| File | Insertion points | LOC delta |
|------|------------------|-----------|
| `framework/src/json_ui/mod.rs` | import list (line 28); `JsonUi::resolve` (line 43); `JsonUi::resolve_with_errors` (line 156); new `Expression resolution tests` section (lines 709–861) | +158 |

## Commits

| SHA | Scope |
|-----|-------|
| `8d7e02b8` | `feat(118-02): wire resolve_expressions into JsonUi pipeline` — import + 2 call-site insertions in `JsonUi::resolve` and `JsonUi::resolve_with_errors`, updated doc comments |
| `ebb327b8` | `test(118-02): add end-to-end expression resolution integration tests` — 5 new integration tests, new section banner, helper `expression_spec_with_data_marker()` |

## Pipeline ordering (D-08)

```
JsonUi::resolve:              resolve_actions → resolve_expressions
JsonUi::resolve_with_errors:  resolve_actions → resolve_expressions → resolve_errors
```

All six public render paths inherit this order via one of the two internal helpers:

| Render method | Internal helper |
|---------------|-----------------|
| `render` | `resolve` (delegates through `render_with_config`) |
| `render_with_config` | `resolve` |
| `render_json` | `resolve` |
| `render_with_errors` | `resolve_with_errors` (delegates through `render_with_errors_config`) |
| `render_with_errors_config` | `resolve_with_errors` |
| `render_json_with_errors` | `resolve_with_errors` |

## Integration tests (5 new)

| # | Test fn | Public render path exercised | ROADMAP criterion verified | Status |
|---|---------|------------------------------|----------------------------|--------|
| 1 | `render_resolves_data_expression_before_html_emission` | `render` (via `render_with_config`) | `$data` resolves before HTML emission | pass |
| 2 | `render_json_returns_spec_with_no_expression_markers` | `render_json` | resolved spec is what JSON consumers see | pass |
| 3 | `render_with_config_honors_expression_resolution` | `render_with_config` | resolution applies regardless of config; numeric type preserved | pass |
| 4 | `render_with_errors_resolves_expressions_then_applies_errors` | `render_with_errors` (via `render_with_errors_config`) | pipeline order — template expands BEFORE error attachment reads `field` | pass |
| 5 | `render_json_with_errors_returns_resolved_spec_with_errors` | `render_json_with_errors` | JSON error path inherits expression resolution (W-3 revision, must_haves #5) | pass |

Each test asserts BOTH the resolved value is present AND the `$data`/`$template` marker is absent — the pairing is what proves single-pass resolution actually ran.

`response_body()` in the test module returns Debug-formatted bytes (`format!("{body_bytes:?}")`), so substring checks use unquoted literals (`Hello`, `$data`, `$template`) rather than `"Hello"`. This matches the file's pre-existing assertion style for the `render_with_errors tests` section.

## Gates

| Gate | Result |
|------|--------|
| `cargo test -p ferro-rs --all-features --lib -- json_ui::tests::render_resolves_data_expression_before_html_emission json_ui::tests::render_json_returns_spec_with_no_expression_markers json_ui::tests::render_with_config_honors_expression_resolution json_ui::tests::render_with_errors_resolves_expressions_then_applies_errors json_ui::tests::render_json_with_errors_returns_resolved_spec_with_errors` | 5/5 pass |
| `cargo test -p ferro-rs --all-features --lib -- json_ui::` | 36/36 pass (no regressions in the broader json_ui suite) |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p ferro-rs --all-features --all-targets -- -D warnings` | clean |
| No forbidden sigils in touched files (`grep -nE '"\$(if\|for\|state\|bind\|ref\|concat\|let)"' framework/src/json_ui/mod.rs`) | empty |
| No new dependencies (`git diff ferro-json-ui/Cargo.toml framework/Cargo.toml`) | empty |

### Deferred per thermal constraint

The following workspace-wide gates are deferred — the user requested reduced CPU load, and package-scoped equivalents already ran clean:

- `cargo clippy --all --all-targets -- -D warnings` — superset of the package-scoped clippy; the transitive dependency check is redundant when `ferro-rs` (which depends on everything touched) already clippies clean with `-D warnings`.
- `cargo test --all-features` — the workspace verifier (next phase step) or a final thermal-permissive run can take this.

## Deviations

- Test assertions use unquoted substring checks (`body.contains("Hello")`) rather than JSON-quoted literals (`body.contains("\"Hello\"")`). This is forced by the pre-existing `response_body()` helper returning `format!("{body_bytes:?}")` — Debug format escapes every `"` as `\"`, so any quote-heavy literal in a Rust source `contains()` call can never match. Same pattern used by pre-existing tests (`render_json_with_errors_includes_errors_in_response` at line 626 and surrounding tests).

## Unblocks

- **Phase 119** — Page Loader will codify the parse → `resolve_actions` → `resolve_expressions` → `Catalog::validate` → render order at the file-load entry point. The wiring and pipeline position are now proven end-to-end by Plan 118-02's integration tests.
- **Phase 118 is closed code-wise.** Outstanding: optional workspace-wide gate run when thermally permissible; phase-level verifier will make the final call.
