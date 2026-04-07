# Phase 124 — Deferred Items

## Out-of-scope drift discovered during 124-03

- `ferro-json-ui` calendar test has rustfmt drift (unrelated to plan 124-03 scope).
  `cargo fmt --all -- --check` fails on `ferro-json-ui/src/.../calendar*.rs`.
  Logged here per execution rule (scope boundary): not fixed in this plan.
