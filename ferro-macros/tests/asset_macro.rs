//! Trybuild UI tests for the `asset!()` proc-macro.
//!
//! - `tests/ui/asset/pass/*.rs` — fixtures that MUST compile cleanly.
//!
//! Update snapshots after intentional changes:
//!     TRYBUILD=overwrite cargo test -p ferro-macros --test asset_macro

#[test]
fn asset_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/asset/pass/*.rs");
}
