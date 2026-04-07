# Deferred Items — Phase 125

## Pre-existing uncommitted work stashed during 125-02 execution

At the start of plan 125-02 the working tree contained unrelated in-progress
changes that did NOT compile:

- `ferro-cli/src/templates/ignore_patterns.rs`
- `ferro-json-ui/src/action.rs` (added `target: Option<String>` field to `Action`)
- `ferro-json-ui/src/component.rs`
- `ferro-json-ui/src/layout.rs` (other edits — separate from the `.as_str()` update made here)
- `ferro-json-ui/src/render.rs` (≈26 `Action { ... }` literals missing the new `target` field)
- `ferro-json-ui/src/resolve.rs` (missing `target` field)
- `ferro-json-ui/src/view.rs` (missing `target` field)

These 26 `E0063 missing field target` errors blocked verification of the
runtime split. They are out of scope for plan 125-02 (scope boundary: only
fix issues directly caused by the current task's changes).

Action taken: the pre-existing modifications were stashed via

    git stash push -m "pre-existing-unrelated-125-02" -- <files>

so the runtime split could be validated in isolation. Attempted
`git stash pop` after the split conflicts on `layout.rs` and `render.rs`
because plan 125-02 also touched `layout.rs` (one-line `.as_str()` update
to accommodate the new `LazyLock<String>` bundle).

The stash remains in `git stash list` as `stash@{0}: pre-existing-unrelated-125-02`.
The user must:

1. Pop it manually (`git stash pop`) and resolve the `layout.rs` conflict
   (keep the `.as_str()` call from 125-02, keep any other stashed edits).
2. Complete the `target` field addition: add `target: None` (or appropriate
   value) to every `Action { ... }` literal flagged by `cargo build -p ferro-json-ui`.

This is not a bug introduced by 125-02 — it is pre-existing unfinished work
that was already broken on disk before the runtime split began.
