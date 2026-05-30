//! Compile-fail: `#[action]` with a non-string-literal value for `redirect_to`.

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

#[action(redirect_to = 42)]
pub async fn h(_req: Request) -> ActionResult {
    Ok(())
}

fn main() {}
