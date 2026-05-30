//! Compile-fail: `#[action]` with an unknown attribute key.

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

#[action(redirect_to = "/x", bogus = "y")]
pub async fn h(_req: Request) -> ActionResult {
    Ok(())
}

fn main() {}
