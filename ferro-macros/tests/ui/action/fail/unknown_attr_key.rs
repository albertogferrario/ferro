//! Compile-fail: unknown attribute key.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

#[action(redirect_to = "/dashboard", banana = "yellow")]
pub async fn h(_req: Request) -> ActionResult {
    Ok(())
}

fn main() {}
