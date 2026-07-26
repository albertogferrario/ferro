---
phase: 261-asset-ergonomics
plan: "04"
subsystem: ferro-macros
tags: [asset, macro, proc-macro, trybuild]
dependency_graph:
  requires: [ferro-bundle-leaf, ferro-bundle-reexport]
  provides: [asset-macro, ferro-asset-reexport]
  affects: [ferro-macros, framework]
tech_stack:
  added: []
  patterns: [OnceLock lazy register-once, include_bytes! call-site-relative, mime_from_ext MIME inference]
key_files:
  created:
    - ferro-macros/src/asset.rs
    - ferro-macros/tests/asset_macro.rs
    - ferro-macros/tests/ui/asset/pass/minimal.rs
    - ferro-macros/tests/ui/asset/pass/fixture.js
  modified:
    - ferro-macros/src/lib.rs
    - framework/src/lib.rs
decisions:
  - "include_bytes!(#path_lit) re-emits the original literal so path resolution is call-site-source-relative (D-01)"
  - "static OnceLock<String> per call site: get_or_init registers Bundle exactly once, .as_str() yields &'static str (D-02/D-03)"
  - "Bundle name from sanitized path: keep [a-zA-Z0-9-], map rest to _ (D-04)"
  - "trybuild resolves include_bytes! relative to fixture source file dir (not crate root) — fixture uses 'fixture.js' sibling path"
  - "pub use ferro_macros::asset placed alphabetically after action in framework/src/lib.rs (rustfmt requirement)"
metrics:
  duration_seconds: 2523
  completed_date: "2026-07-26"
  tasks_completed: 2
  files_modified: 6
requirements: [LIVE-03]
---

# Phase 261 Plan 04: asset!() Proc-Macro + Trybuild Pass Summary

Implemented the `asset!("path")` function-like proc-macro in `ferro-macros`, registered it, re-exported it as `ferro::asset!`, and proved it with a trybuild pass fixture. The macro collapses `Bundle::new(name, bytes).content_type(ct).hashed_url()` into a single call-site expression: file embedded via `include_bytes!` (source-relative), bundle registered exactly once per call site via a `static OnceLock<String>`, content-hashed URL returned as `&'static str`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | asset.rs macro + lib.rs registration + framework re-export | 485879f0 | ferro-macros/src/asset.rs, ferro-macros/src/lib.rs, framework/src/lib.rs |
| 2 | trybuild pass fixture + harness | 30e8e3f9 | ferro-macros/tests/asset_macro.rs, tests/ui/asset/pass/minimal.rs, tests/ui/asset/pass/fixture.js |
| — | rustfmt fixes (closure expansion + alphabetical re-export) | a8572618 | ferro-macros/src/asset.rs, framework/src/lib.rs |

## Key API

```rust
// Usage at the call site:
let url: &'static str = ferro::asset!("assets/app.js");

// Expansion (abridged):
{
    static __FERRO_ASSET_URL: ::std::sync::OnceLock<::std::string::String>
        = ::std::sync::OnceLock::new();
    __FERRO_ASSET_URL
        .get_or_init(|| {
            static __FERRO_ASSET_BYTES: &[u8] = include_bytes!("assets/app.js");
            ::ferro::bundle::Bundle::new("assets_app_js", __FERRO_ASSET_BYTES)
                .content_type(::ferro::bundle::mime_from_ext("js"))
                .hashed_url()
        })
        .as_str()
}
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug / fmt] rustfmt style violations in asset.rs and framework/src/lib.rs**
- **Found during:** `cargo fmt --all -- --check` CI gate
- **Issue 1:** `.map(|c| if ... { c } else { '_' })` — rustfmt requires the closure body on multiple lines when it contains an `if/else` expression
- **Issue 2:** `pub use ferro_macros::asset` was placed after `injectable` (insertion point chosen near memoize); rustfmt sorts `pub use` blocks alphabetically, so `asset` belongs right after `action`
- **Fix:** Expanded closure to multi-line in asset.rs; moved the re-export to alphabetical position in framework/src/lib.rs
- **Files modified:** ferro-macros/src/asset.rs, framework/src/lib.rs
- **Commit:** a8572618

**2. [Rule 1 - Path resolution discovery] trybuild resolves include_bytes! relative to fixture file directory**
- **Found during:** Task 2 trybuild run (first attempt)
- **Issue:** The plan noted "if trybuild resolves relative to fixture file, adjust to 'fixture.js'" — and that is exactly what happened. Trybuild compiles fixtures in a temp tree preserving the fixture's directory; `include_bytes!` resolves from the fixture file's location, not the crate root.
- **Fix:** Changed the fixture path literal from `"tests/ui/asset/pass/fixture.js"` to `"fixture.js"` (sibling resolution). The compiler's own help message confirmed this: "help: there is a file with the same name in a different directory — use 'fixture.js'"
- **Files modified:** ferro-macros/tests/ui/asset/pass/minimal.rs
- **Not a commit** (fix was applied before the Task 2 commit)

## CI Gate Outcome

```
cargo fmt --all -- --check          → exit 0
cargo clippy --all --all-targets    → exit 0, 0 warnings
  -- -D warnings
cargo test --all-features           → exit 0
  (137 test suites, all test result: ok)
  asset_macro_ui: tests/ui/asset/pass/minimal.rs ... ok
```

## Known Stubs

None. The macro is fully wired: `include_bytes!` + `OnceLock` + `Bundle::new` + `mime_from_ext` + `.hashed_url()` + `.as_str()`. The returned URL is a real content-hashed path once the bundle route is mounted (that is app-side wiring, not a stub in the macro).

## Threat Flags

No new security surface introduced. The macro's only input is a compile-time string literal; path resolution is via `include_bytes!` at build time with no runtime file access.

## Self-Check: PASSED

- `ferro-macros/src/asset.rs`: found
- `ferro-macros/src/lib.rs` contains `mod asset;` and `pub fn asset`: found
- `framework/src/lib.rs` contains `pub use ferro_macros::asset`: found
- `ferro-macros/tests/asset_macro.rs`: found
- `ferro-macros/tests/ui/asset/pass/minimal.rs`: found
- `ferro-macros/tests/ui/asset/pass/fixture.js`: found
- Commits 485879f0, 30e8e3f9, a8572618: verified in git log
