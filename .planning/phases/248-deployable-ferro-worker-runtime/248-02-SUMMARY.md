---
phase: 248-deployable-ferro-worker-runtime
plan: "02"
subsystem: ferro-macros
tags: [proc-macro, queue-routing, offload, trybuild, wave-1]
dependency_graph:
  requires:
    - 248-01 (JobRegistrarEntry.queue: Option<&'static str> field — emitted code targets this field)
  provides:
    - ferro-macros/src/service.rs (parse_nested_meta queue arg before strip)
    - ferro-macros/src/offload.rs (declared_queue field + collect_info param + emit_job_items queue/on_queue)
    - ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr (regenerated, placeholder replaced)
  affects:
    - Any crate whose #[offload]-derived JobRegistrarEntry now carries queue: Some("name") instead of None
tech_stack:
  added: []
  patterns:
    - "syn-2 parse_nested_meta for optional key=value attr args"
    - "#[async_trait] on impl Offloadable override to match trait lifetime signature"
    - "queue_name_tokens / on_queue_tokens conditional token fragments (Some vs empty)"
    - "TRYBUILD=overwrite + re-run without env var for snapshot stability gate"
key_files:
  created: []
  modified:
    - ferro-macros/src/service.rs
    - ferro-macros/src/offload.rs
    - ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr
decisions:
  - "Emit #[::ferro::async_trait] on impl Offloadable override: the Offloadable trait is async_trait-annotated; overriding offload() without async_trait produces E0195 lifetime mismatch. Adding async_trait to the impl block resolves it regardless of whether an override is present."
  - "offload() override emitted only when declared_queue is Some; bare #[offload] uses the trait-provided default (no code size penalty, no behavioral change)"
  - "on_queue_tokens inserted after .with_handle_key(...) and before .dispatch() in the PendingDispatch chain, matching the ferro-queue builder API order"
metrics:
  duration_seconds: ~480
  completed_date: "2026-08-14"
  tasks_completed: 2
  files_created: 0
  files_modified: 3
---

# Phase 248 Plan 02: #[offload(queue = "name")] Macro Arg Parsing Summary

## One-liner

Taught `#[offload]` to accept `queue = "name"` via syn-2 `parse_nested_meta`,
threading the declared queue into both the `JobRegistrarEntry` inventory entry
and an emitted `Offloadable::offload()` override that routes dispatch via `.on_queue()`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Parse #[offload(queue = "name")] and thread declared_queue through collect_info | b887a314 | ferro-macros/src/service.rs, ferro-macros/src/offload.rs |
| 2 | Emit queue into JobRegistrarEntry + .on_queue() in Offloadable::offload; run trybuild UI gate | a972fe49 | ferro-macros/src/offload.rs, ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr |

## Exact Emitted Fragments

### JobRegistrarEntry (inventory::submit! block)

For `#[offload(queue = "reports")]`:
```rust
::ferro::inventory::submit! {
    ::ferro::queue::JobRegistrarEntry {
        register: |w: &mut ::ferro::queue::WorkerLoop| { w.register::<ReportsBuildMonthlyJob>(); },
        name: "ReportsBuildMonthlyJob",
        queue: Some("reports"),
    }
}
```

For bare `#[offload]` (no queue arg):
```rust
::ferro::inventory::submit! {
    ::ferro::queue::JobRegistrarEntry {
        register: |w: &mut ::ferro::queue::WorkerLoop| { w.register::<SomeJob>(); },
        name: "SomeJob",
        queue: None,
    }
}
```

### Offloadable::offload() override (only when queue is declared)

```rust
#[::ferro::async_trait]
impl ::ferro::queue::Offloadable for ReportsBuildMonthlyJob
where
    (): ::ferro::queue::OffloadSerializable,
{
    type Output = ();

    async fn offload(
        self,
    ) -> ::std::result::Result<
        ::ferro::queue::OffloadHandle<Self::Output>,
        ::ferro::queue::Error,
    > {
        let key = ::ferro::queue::HandleKey::new();
        ::ferro::queue::PendingDispatch::new(self)
            .with_handle_key(key.as_str().to_string())
            .on_queue("reports")
            .dispatch()
            .await?;
        Ok(::ferro::queue::OffloadHandle::new(key))
    }
}
```

For bare `#[offload]`, no `offload()` override is emitted — the trait-provided default
(in `ferro-queue/src/offload.rs`) is used as-is (no `.on_queue()` call, routes to "default").

## Cross-Plan Dependency on Plan 01

The `queue: Some("reports")` token in the emitted `JobRegistrarEntry` targets the
`pub queue: Option<&'static str>` field added by Plan 01 Task 1 in `ferro-queue/src/db.rs`.
The trybuild fixtures compiled correctly only after both plans were integrated in this wave —
the Plan 00 pass fixture (`queue_arg.rs`) was RED until Plan 01's field existed AND Plan 02's
macro parsing was wired.

## queue_unknown_arg.stderr Regeneration

The Plan 00 placeholder was replaced via `TRYBUILD=overwrite`. Final content:

```
error: unknown #[offload] argument; expected `queue = "name"`
  --> tests/ui/offload/fail/queue_unknown_arg.rs:26:15
   |
26 |     #[offload(retries = 3)]
   |               ^^^^^^^

error[E0405]: cannot find trait `Reports` in this scope
  --> tests/ui/offload/fail/queue_unknown_arg.rs:31:6
   |
31 | impl Reports for ReportBuilder {
   |      ^^^^^^^ not found in this scope
```

The primary error is the expected diagnostic. `E0405` is a cascade error (the trait was not
emitted because the macro returned a compile error early) — expected and stable across runs.

Snapshot stability confirmed: second run (without `TRYBUILD=overwrite`) passed in 7.6s
using cached artifacts, with no diff.

## Trybuild UI Gate Results

```
cargo test -p ferro-macros --test offload_macro
  tests/ui/offload/pass/basic.rs              [should pass]          ok
  tests/ui/offload/pass/offload_handle.rs     [should pass]          ok
  tests/ui/offload/pass/queue_arg.rs          [should pass]          ok  ← was RED (Plan 00)
  tests/ui/offload/pass/ref_str_param.rs      [should pass]          ok
  tests/ui/offload/pass/result_method.rs      [should pass]          ok
  tests/ui/offload/fail/mut_ref_param.rs      [should fail]          ok
  tests/ui/offload/fail/non_serializable_param.rs  [should fail]     ok
  tests/ui/offload/fail/non_serializable_return.rs [should fail]     ok
  tests/ui/offload/fail/queue_unknown_arg.rs  [should fail]          ok  ← stderr regenerated
9/9 passed
```

## Full Changed-Crates Test Results

```
cargo test -p ferro-macros        → ok. 0 passed; 0 failed; 46 ignored (doctest-only)
cargo test -p ferro-queue         → ok. 1 passed; 0 failed; 5 ignored
cargo clippy --all --all-targets -- -D warnings → clean (0 warnings)
cargo fmt --all -- --check        → clean
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added #[async_trait] to emitted impl Offloadable block**

- **Found during:** Task 2 — `TRYBUILD=overwrite` run showed `E0195: lifetime parameters or
  bounds on method \`offload\` do not match the trait declaration` for `queue_arg.rs`.
- **Issue:** The `Offloadable` trait is `#[async_trait]`-annotated. Overriding its `async fn
  offload()` in a plain `impl` block without `#[async_trait]` on the impl produces a lifetime
  mismatch because the trait's async_trait desugaring introduces lifetime parameters that the
  raw `impl` does not match.
- **Fix:** Emit `#[::ferro::async_trait]` on the `impl ::ferro::queue::Offloadable for #job_ident`
  block unconditionally. For jobs without a declared queue, the impl has no async method body
  (only `type Output`), so async_trait is a no-op. For jobs with a declared queue, async_trait
  correctly desugars the override to match the trait's lifetime contract.
- **Files modified:** ferro-macros/src/offload.rs
- **Commit:** a972fe49

## Known Stubs

None — both pass fixtures compile, both fail fixtures reject correctly, no placeholder remains.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. The queue
name is a compile-time `&'static str` literal (developer-controlled, never user/request input).
It flows to `PendingDispatch::on_queue()` → parameterized DB claim filter — no SQL injection
surface (matches T-248-02-02 `accept` disposition in the plan's threat register).

## Self-Check: PASSED

```
[ -f "ferro-macros/src/service.rs" ]                                    → FOUND
[ -f "ferro-macros/src/offload.rs" ]                                    → FOUND
[ -f "ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr" ]    → FOUND

git log → b887a314, a972fe49 both present
grep "parse_nested_meta" ferro-macros/src/service.rs                    → FOUND
grep 'unknown #[offload] argument' ferro-macros/src/service.rs          → FOUND
grep "declared_queue: Option<String>" ferro-macros/src/offload.rs       → FOUND
grep "queue_name_tokens" ferro-macros/src/offload.rs                    → FOUND
grep "on_queue" ferro-macros/src/offload.rs                             → FOUND
! grep "::ferro_queue::" ferro-macros/src/offload.rs                    → PASS (none)
! grep -i "regenerate" queue_unknown_arg.stderr                         → PASS (none)

cargo test -p ferro-macros --test offload_macro  → 9/9 ok
cargo test -p ferro-macros                       → ok (0 failed)
cargo test -p ferro-queue                        → ok (1 passed, 0 failed)
cargo fmt --all -- --check                       → clean
cargo clippy --all --all-targets -- -D warnings  → clean
```
