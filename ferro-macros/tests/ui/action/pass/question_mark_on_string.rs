//! Compile-pass: `?` on `Result<_, String>` inside the body.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

fn fallible() -> Result<i32, String> {
    Ok(42)
}

#[action(redirect_to = "/dashboard")]
pub async fn h(_req: Request) -> ActionResult {
    let _n = fallible()?;
    Ok(())
}

fn main() {}
