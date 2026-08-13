---
phase: 244-offload-macro-job-payload-derivation
reviewed: 2026-08-13T00:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - ferro-macros/src/offload.rs
  - ferro-macros/src/service.rs
  - ferro-macros/src/lib.rs
  - ferro-queue/src/db.rs
  - ferro-queue/src/worker.rs
  - ferro-queue/src/lib.rs
  - ferro-queue/tests/offload_round_trip.rs
  - ferro-macros/tests/offload_macro.rs
  - ferro-macros/tests/ui/offload/pass/basic.rs
  - ferro-macros/tests/ui/offload/pass/result_method.rs
  - ferro-macros/tests/ui/offload/pass/ref_str_param.rs
  - ferro-macros/tests/ui/offload/fail/mut_ref_param.rs
  - framework/src/lib.rs
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 244: Code Review Report

**Reviewed:** 2026-08-13T00:00:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Phase 244 ships the `#[offload]` macro infrastructure: a proc-macro helper in `ferro-macros/src/offload.rs`, integration into `service_impl`, the inventory-based `JobRegistrarEntry` type in `ferro-queue/src/db.rs`, and the dual-path `WorkerLoop::from_registry`. The overall design is sound. Paths are fully qualified (`::ferro::*` throughout), `&mut` is rejected with a spanned error, the `&str`→`String` field type mapping is correct, and the `from_registry` dual-drain avoids double-registration (the runtime Vec and the inventory collection are separate namespaces). The `JobRegistrarEntry` re-export in `framework/src/lib.rs` is correct.

Three warnings and three info-level items are noted below.

## Warnings

### WR-01: `owned_type` maps `&T` to `T` by value, losing the inner reference's lifetime

**File:** `ferro-macros/src/offload.rs:84-91`

The `other` arm of the `Type::Reference` match — covering `&T` where T is neither `str` nor a slice — emits the inner type `T` verbatim and classifies the forward strategy as `Clone`. If `T` is itself a reference (e.g., `&&str`, `&SomeRef<'a>`), the generated struct field will contain a type that still has a lifetime, which cannot be serialized and will fail to compile. The error will surface at the use site with a confusing message rather than at the `#[offload]` annotation.

The issue is minor for idiomatic service APIs (which rarely pass `&&str` or `&Ref<'_>`), but it is a latent correctness gap rather than an accepted limitation. A spanned error at macro evaluation time — rejecting any `&T` where T is not `str` or a slice — would make the boundary explicit and produce a clear diagnostic.

**Fix:**
```rust
// In owned_type, after the &str and &[T] arms:
other => {
    // Check if the inner type is itself a reference — those cannot be
    // owned or serialized, so reject them with a clear error.
    if matches!(other, Type::Reference(_)) {
        return Err(syn::Error::new_spanned(
            ty,
            "#[offload] does not support double references (&& or &Ref<'_>) — \
             the job payload must be fully owned",
        ));
    }
    Ok(quote! { #other })
}
```

### WR-02: `JOB_REGISTRARS` Vec path does NOT drain — jobs registered via `Queue::register` can be registered twice if `from_registry` is called more than once

**File:** `ferro-queue/src/db.rs:83-87`, `ferro-queue/src/worker.rs:210-219`

`Queue::apply_registrars` iterates the `JOB_REGISTRARS` Vec without clearing it, and `WorkerLoop::register` uses `HashMap::insert` (which silently overwrites). A second call to `from_registry` — e.g., if the server boot path is re-entrant, or in a test that creates multiple worker loops — will call every runtime registrar function twice. The `HashMap::insert` overwrite means the final handler map is correct (no duplicate handlers run), but the silent overwrite conceals the double-call.

For the inventory path this is not an issue: `inventory::iter` is read-only by design, and `HashMap::insert` on the same key is idempotent. The risk is limited to the `JOB_REGISTRARS` Vec path, which is the manual `Queue::register` route.

**Fix:** The simplest fix is to clear `JOB_REGISTRARS` after draining in `apply_registrars`, or to document explicitly that `from_registry` must be called at most once per process lifetime. The doc-comment on `from_registry` currently says "build a WorkerLoop and apply all job types" with no such caveat.

```rust
pub(crate) fn apply_registrars(w: &mut crate::WorkerLoop) {
    let mut registrars = JOB_REGISTRARS.lock().unwrap();
    for r in registrars.iter() {
        r(w);
    }
    // Drain after applying so a second from_registry call starts clean.
    registrars.clear();
}
```

Alternatively, add a `#[doc]` note on `from_registry` stating that it should be called exactly once and that repeated calls produce duplicate registrations on the runtime path.

### WR-03: `handle()` resolves the service via `App::make` and calls `.expect()` — a missing binding panics the worker task, not the job

**File:** `ferro-macros/src/offload.rs:283-285`

```rust
let svc = ::ferro::App::make::<dyn #trait_ident>()
    .expect(#expect_msg);
```

`App::make` returns `Option<Arc<dyn Trait>>`. Calling `.expect()` on `None` panics the async task. The worker's panic isolation (`AssertUnwindSafe + catch_unwind`) does catch this, so the job will be counted as a failed attempt and retried. However, every retry will also panic (the binding is still missing), exhausting `max_retries` and permanently parking the job as failed with the error message "job handler panicked" rather than the clear diagnostic string in `expect_msg`. The actual message from `.expect()` is lost because `std::panic::catch_unwind` captures the panic payload as `Box<dyn Any>`, which is not preserved in the worker's error string.

**Fix:** Convert the `Option` to a `Result` and propagate it as a `::ferro::queue::Error` so the actual message is recorded in the `failed` row and visible in introspection:

```rust
let svc = ::ferro::App::make::<dyn #trait_ident>()
    .ok_or_else(|| ::ferro::queue::Error::job_failed(
        #job_ident_str,
        #expect_msg.to_string(),
    ))?;
```

This requires the generated `handle()` body to use `?` rather than `.expect()`. The return type `Result<(), ::ferro::queue::Error>` already supports `?`, so the change is straightforward.

## Info

### IN-01: `to_pascal_case` does not preserve interior upper-case letters

**File:** `ferro-macros/src/offload.rs:112-123`

`to_pascal_case` capitalizes the first character of each `_`-delimited segment, but leaves the remaining characters unchanged. For a snake_case method name this is correct (e.g., `build_monthly` → `BuildMonthly`). However, if a method name contains an acronym in the middle of a segment (e.g., `send_http_request` → `SendHttpRequest` rather than `SendHTTPRequest`), the output diverges from any separately computed PascalCase on the same string. This is acceptable for most cases, but worth noting since the generated struct ident and the dispatch key (`type_name`) both depend on it, and changing the conversion rule later would be a breaking dispatch-key change.

No fix required; document the behavior in the function's doc-comment.

### IN-02: Test file uses deprecated `std::env::set_var` without `unsafe` block (Rust 2024 edition warning)

**File:** `ferro-queue/tests/offload_round_trip.rs:44`

`std::env::set_var` is not `unsafe` in Rust 2021, but calling it from a multi-threaded context (a tokio async test) is documented as unsound. Rust 2024 marks it `unsafe`. If the workspace edition is upgraded to 2024, the test will require an `unsafe` block and a justification comment. The `serial_test::serial` attribute serializes test execution but does not eliminate the soundness concern for async tests sharing a process-wide environment.

This is a future-proofing note; no action required for the current edition.

### IN-03: `field_forward` for `&T` (non-str, non-slice) emits `.clone()` — the generated clone is on the *owned* field type, not on the original reference

**File:** `ferro-macros/src/offload.rs:98-107`

For a parameter `foo: SomeType` (non-reference), `field_forward` returns `Clone`, and `emit_job_items` emits `self.foo.clone()`. This is correct. For `foo: &SomeType` (immutable reference, not str/slice), `owned_type` maps the field to `SomeType`, and `field_forward` still returns `Clone`, emitting `self.foo.clone()`. This too is correct, as the field is `SomeType` and `clone()` is called on it.

The asymmetry is that `field_forward` is called on the *original parameter type* while the forwarding expression operates on the *owned field type*. The logic happens to produce the right output for all current cases, but the two concerns are coupled implicitly. A comment in `emit_job_items` near the `field_args` construction noting this dependency would prevent a future maintainer from breaking it by changing `owned_type` without updating `field_forward`.

No code change required; a comment is sufficient.

---

_Reviewed: 2026-08-13T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
