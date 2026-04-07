# Deferred Items - Phase 122

## Pre-existing fmt drift (out of scope)

Discovered while running `cargo fmt --all -- --check` for plan 122-01:

- `ferro-json-ui/src/component.rs` lines ~3517, ~3526 — long-line tuple formatting
- `ferro-json-ui/src/render.rs` line ~3761 — long-line `assert!` formatting

Not touched by 122-01. Should be fixed in a standalone fmt sweep.
