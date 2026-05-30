//! Trybuild UI tests for the `#[action]` proc-macro.
//!
//! - `tests/ui/action/pass/*.rs` — fixtures that MUST compile cleanly.
//! - `tests/ui/action/fail/*.rs` + `*.stderr` — fixtures that MUST emit the
//!   exact compile error captured in the matching `.stderr` snapshot.
//!
//! Update `.stderr` snapshots after intentional message changes:
//!     TRYBUILD=overwrite cargo test -p ferro-macros --test action_macro

#[test]
fn action_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/action/pass/*.rs");
    t.compile_fail("tests/ui/action/fail/*.rs");
}
