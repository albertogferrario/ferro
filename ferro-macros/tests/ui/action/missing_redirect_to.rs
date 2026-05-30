//! Compile-fail: `#[action]` with no `redirect_to` argument.

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

#[action]
pub async fn no_redirect(_req: Request) -> ActionResult {
    Ok(())
}

fn main() {}
