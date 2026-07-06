---
phase: 222
name: cache-events-bridge
status: passed
verified: 2026-06-14
method: retroactive (code shipped before GSD planning artifacts were generated)
---

# Phase 222 — Cache-Events Bridge — Retroactive Verification

## Context

This phase was implemented and committed **before** the GSD discuss→plan→execute
pipeline ran for it. A `/gsd-discuss-phase 222 --auto` invocation on 2026-06-14
surfaced the discrepancy: ROADMAP listed Phase 222 as `[ ]` / "Scoped" while the
repository already contained the shipped implementation. Rather than re-plan over
existing code, the implementation was audited against the scoped success criteria
and the planning state reconciled to reality.

## Shipped surface

`ferro-cache/src/invalidator.rs` (commits `4d81a596`, `2172c8e0`):

- `pub fn register_invalidator_on<E, F>(dispatcher: &EventDispatcher, cache: Arc<Cache>, key_fn: F)`
- `pub fn register_invalidator<E, F>(cache: Arc<Cache>, key_fn: F)` — wraps the above on `global_dispatcher()`
- Bound: `F: Fn(&E) -> Vec<String> + Send + Sync + 'static`
- Exported from `ferro-cache/src/lib.rs`.

## Locked decisions (resolved to lean defaults)

| # | Decision | Resolution | Evidence |
|---|----------|-----------|----------|
| D-01 | `keys()` return type | `Vec<String>` | closure signature returns `Vec<String>` |
| D-02 | execution model | synchronous in-dispatch | listener runs inline in `dispatcher.on`, flush awaited before `Ok(())` |
| D-03 | single vs multi-invalidator | multi (all run) | test `all_registered_invalidators_run` |
| D-04 | closure type | `Fn` | `Fn(&E) -> Vec<String> + Send + Sync + 'static` |
| D-05 | cache attach | closure-captured `Arc<Cache>` | `cache: Arc<Cache>` param, cloned per dispatch |

## Success criteria audit

| SC | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| 1 | `CacheInvalidator<E>` trait + closure helper | ⚠️ **DEVIATION** | No trait shipped. Closure-only surface (`register_invalidator` / `register_invalidator_on`) delivers the same capability with less indirection; the trait existed in scope only to be blanket-impl'd by the closure. Documented in ROADMAP §222. |
| 2 | dispatch flushes tags (insert→dispatch→`None`) | ✅ | test `flushes_matching_tag`; `does_not_flush_unrelated_tags` |
| 3 | multiple invalidators all run, order documented | ✅ | test `all_registered_invalidators_run`; doc comment |
| 4 | failure logged + swallowed, no propagation | ✅ | always returns `Ok(())`; `tracing::warn!` on per-tag flush error (invalidator.rs:92, :100) |
| 5 | synchronous-only documented/enforced | ✅ | crate doc invalidator.rs:46–47; sync by construction (no `ShouldQueue`) |
| 6 | doc example end-to-end | ✅ | invalidator.rs:9–29 + lib.rs examples (doctests present, `ignore`d) |
| 7 | CHANGELOG + version bump 0.2.58→0.2.59, stable path | ✅ | `Cargo.toml` version `0.2.59`; CHANGELOG entries; exported from `lib.rs` |

## Test evidence

```
cargo test -p ferro-cache --all-features
→ 23 passed; 0 failed (incl. 6 invalidator tests); finished in 0.24s
```

## Verdict

**PASSED with one documented deviation (SC#1).** The phase's killer capability —
declarative one-line-at-boot cache invalidation wired to the event bus — is fully
delivered. The `CacheInvalidator` trait named in the original scope was intentionally
not shipped; the closure-only API is the simpler equivalent.

## Outstanding (not part of this phase's code)

- **Publish:** the consumer companion (gestiscilo Phase 210) blocks on the `0.2.59`
  crates.io publish. Verify whether `ferro-rs 0.2.59` is published before unblocking
  the consumer. (Cross-references the v15.0 "tagged-locally-not-pushed" state.)
- **Boilerplate:** ROADMAP carries a global `**Plans:** 3/3 plans complete` line on
  every phase entry (template artifact) — not corrected here; out of scope.
