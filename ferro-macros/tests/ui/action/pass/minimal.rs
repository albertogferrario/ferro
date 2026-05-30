//! Compile-pass: minimal `#[action]` — `Ok(())` and no `?`.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

#[action(redirect_to = "/dashboard")]
pub async fn h(_req: Request) -> ActionResult {
    Ok(())
}

fn main() {}
