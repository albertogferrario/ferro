# Phase 124 — Deferred Items

## Pre-existing fmt drift in `ferro-json-ui` (out of scope)

Discovered during 124-02. `cargo fmt --all -- --check` reports unrelated
formatting issues in:

- `ferro-json-ui/src/component.rs:1991`
- `ferro-json-ui/src/render.rs` (multiple sites: 256, 1950, 3261, 3282, 3299,
  5536, 5554, 5571, 5588, 5621, 7370, 7388, 7400, 7416)

These are pre-existing and unrelated to plan 124-02 (generate-routes JSON).
Not fixed here per scope boundary. Recommended action: dedicated `chore: fmt`
commit on `ferro-json-ui` outside this phase, or fold into a janitor pass.
