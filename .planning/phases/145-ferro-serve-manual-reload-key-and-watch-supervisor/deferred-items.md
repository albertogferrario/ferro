# Phase 145 — Deferred Items

Issues discovered during execution that are OUT OF SCOPE for Phase 145 and left for future work.

## ferro-json-ui: missing `compact` field in SwitchProps initializers

**Discovered during:** Plan 145-01 Task 1 (running `cargo clippy --all --all-targets -- -D warnings` as a workspace-wide gate).

**Error:**

```
error[E0063]: missing field `compact` in initializer of `component::SwitchProps`
  --> ferro-json-ui/src/render.rs:8023:21
  --> ferro-json-ui/src/resolve.rs:982:42
```

**Root cause:** Commit `fdd9ae70 feat(switch): add compact prop — toggle-first layout with gap-3 for inline grid use` added a new `compact: bool` field to `SwitchProps` in `ferro-json-ui/src/component.rs:383` but did not update the two struct literals in `ferro-json-ui/src/render.rs` and `ferro-json-ui/src/resolve.rs`. The `cargo clippy --all --all-targets -- -D warnings` workspace gate therefore fails on master before Phase 145 begins.

**Scope decision:** This is pre-existing, unrelated to `ferro serve` / watch supervisor work. Phase 145 scope is `ferro-cli/src/commands/serve.rs` and its deps only. `cargo clippy -p ferro-cli --all-targets -- -D warnings` passes clean on every plan commit.

**Fix (for a separate, small phase or direct commit):** add `compact: false` (or equivalent default) to both `SwitchProps { ... }` initializers, or make `compact` default-derivable. 11 total compile errors downstream of those two sites.

## ferro-json-ui: pre-existing rustfmt drift in render.rs

**Discovered during:** Plan 145-02a Task 1 (running `cargo fmt --all -- --check` after adding the `--watch` flag).

**Error:**

```
Diff in ferro-json-ui/src/render.rs:2286:
-    let mut html = String::from("<details class=\"group rounded-lg border border-border overflow-hidden\"");
+    let mut html =
+        String::from("<details class=\"group rounded-lg border border-border overflow-hidden\"");
```

**Root cause:** Single long-line string literal exceeds `rustfmt`'s default line-width; the file on master has not been re-formatted since. Independent of Phase 145.

**Scope decision:** Pre-existing on master; affects only `ferro-json-ui/src/render.rs`, nothing in `ferro-cli`. Plan 145-02a's gate is scoped to `cargo fmt --package ferro-cli -- --check` (exits 0) mirroring Plan 01's `-p ferro-cli` clippy gate.

**Fix (for a separate, small phase or direct commit):** run `cargo fmt --package ferro-json-ui` and commit the result, once the `SwitchProps.compact` compile errors above are fixed (otherwise rustfmt's output is subject to other churn).
