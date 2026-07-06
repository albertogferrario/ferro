//! Compile-pass: user body records success-side overrides via `req` setters (D-02).

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

#[action(redirect_to = "/dashboard/pagine")]
pub async fn create(req: Request) -> ActionResult {
    let new_id: i64 = 42;
    req.redirect_to(format!("/dashboard/pagine/{new_id}"));
    req.flash("created");
    Ok(())
}

fn main() {}
