//! Compile-pass: `?` on `Result<_, FrameworkError>`.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, FrameworkError, Request};

fn fallible() -> Result<i32, FrameworkError> {
    Ok(42)
}

#[action(redirect_to = "/dashboard")]
pub async fn h(_req: Request) -> ActionResult {
    let _n = fallible()?;
    Ok(())
}

fn main() {}
