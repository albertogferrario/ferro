# Deferred items — Phase 183

## Pre-existing fmt drift in ferro-json-ui (discovered during Plan 01 Task 3)

`cargo fmt --all -- --check` flags `ferro-json-ui/src/lib.rs:46-62`: the `pub use runtime::FERRO_RUNTIME_JS;` line is out of alphabetical/visual order relative to the surrounding `pub use` block. The fix is to move that line below `pub use config::JsonUiConfig;` (where `cargo fmt` wants it).

- **Out of scope for Phase 183** — issue is in `ferro-json-ui`, not `ferro-bundle`. Per CLAUDE.md scope-boundary rule and Plan 01 Task 3 action block ("If clippy fails with warnings against OTHER crates in the workspace: Out of scope for Phase 183. Do not 'fix' pre-existing warnings here. Surface the failure in the SUMMARY as a pre-existing condition.").
- **Surfaced:** 2026-06-06 during Plan 01 Task 3 build/lint gate.
- **Recommended phase scope:** small follow-up "chore: ferro-json-ui fmt drift" or fold into the next ferro-json-ui phase.
