//! Trybuild UI tests for the `#[offload]` helper attribute (consumed by `#[service]`).
//!
//! - `tests/ui/offload/pass/*.rs` — fixtures that MUST compile cleanly.
//! - `tests/ui/offload/fail/*.rs` + `*.stderr` — fixtures that MUST emit the
//!   exact compile error captured in the matching `.stderr` snapshot.
//!
//! Update `.stderr` snapshots after intentional message changes:
//!     TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro

#[test]
fn offload_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/offload/pass/*.rs");
    t.compile_fail("tests/ui/offload/fail/*.rs");
}
