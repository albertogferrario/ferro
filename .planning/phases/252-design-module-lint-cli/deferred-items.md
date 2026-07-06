# Phase 252 Deferred Items

## Out-of-Scope Test Failures (Plan 06 CI gate)

### `commands::serve::tests::spawn_child_with_prefix_uses_new_process_group`

**File:** `ferro-cli/src/commands/serve.rs`
**Observed:** Fails under `cargo test --all-features` (full parallel suite) but passes in isolation (`cargo test -p ferro-cli --all-features -- commands::serve::tests::...`).
**Root cause:** Race condition — `getpgid(child_pid)` returns -1 (error) because the child process exits before the assertion can query its process group. Not a logic error in the serve command; a timing issue in the test harness under heavy parallelism.
**Relation to Plan 06 changes:** None. Plan 06 only modified `app/Cargo.toml`, `app/src/tests/design_lint.rs`, `app/src/tests/mod.rs`, and `app/src/views/*.json`. The serve.rs file was not touched.
**Disposition:** Pre-existing flaky test. Out of scope for Plan 06 to fix. Recommend adding a retry or using `waitpid` synchronization in the test to make the PGID assertion race-free.
