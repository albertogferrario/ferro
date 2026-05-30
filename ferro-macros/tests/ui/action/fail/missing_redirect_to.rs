//! Compile-fail: #[action] with no attributes — `redirect_to` is required.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

#[action]
pub async fn h(_req: Request) -> ActionResult {
    Ok(())
}

fn main() {}
