//! Compile-pass: minimal `asset!()` — embeds a file and returns &'static str.
#![allow(unused_imports)]

extern crate ferro_rs as ferro;

fn main() {
    // asset!() returns &'static str; the path is resolved relative to THIS source
    // file (call-site-source-relative), so "fixture.js" resolves to the sibling
    // file in the same directory as this fixture.
    let _url: &'static str = ferro::asset!("fixture.js");
}
