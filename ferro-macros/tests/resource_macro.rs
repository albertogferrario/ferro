//! Trybuild UI tests for the `#[resource_get]` / `#[resource_post]` proc-macros.
//!
//! - `tests/ui/resource/pass/*.rs` — fixtures that MUST compile cleanly.
//! - `tests/ui/resource/fail/*.rs` + `*.stderr` — fixtures that MUST emit the
//!   exact compile error captured in the matching `.stderr` snapshot.
//!
//! Update `.stderr` snapshots after intentional message changes:
//!     TRYBUILD=overwrite cargo test -p ferro-macros --test resource_macro

#[test]
fn resource_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/resource/pass/*.rs");
    t.compile_fail("tests/ui/resource/fail/*.rs");
}
