//! Compile-fail: `#[action]` on a method with `&self` receiver.

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult};

struct Controller;

impl Controller {
    #[action(redirect_to = "/x")]
    pub async fn h(&self) -> ActionResult {
        Ok(())
    }
}

fn main() {}
